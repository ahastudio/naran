# Implementation Plan: Staging

## Summary

The feature is built Outside-In with stubs: the acceptance tests that drive the window are written first and fail, then each part is built with the part below it stubbed, and each next step replaces that stub with the real part.
The acceptance tests pass only when the last stub is gone.
The window turns drops, picks and diff links into staging changes that run in the background, and the core runs the system `git`, building a patch of only the selected lines for hunks and lines.

## Requirements

1. Two lists with status badges, every file of a new folder listed on its own, and a renamed file shown as `new path <- old path`.
2. Drag whole files between the lists, singly or as a picked set.
3. Pick files with click modifiers, Shift+Up/Down and Cmd+A.
4. Stage and unstage a hunk or picked lines; untracked and conflicted files only as a whole.

## Critical Files

### New Files

- `app/src/workspace/tests/staging_acceptance.rs`: acceptance tests driven by clicks, keys and drags.
- `app/src/workspace/tests/staging_diff_acceptance.rs`: acceptance tests for hunks, lines, whole-file staging and badges.
- `app/src/workspace/tests/renamed_files_acceptance.rs`: the acceptance test for the row of a renamed file.
- `app/src/model/tests/labels.rs`: the tests of the row text.

### Modified Files

- `core/src/ops/staging.rs`: staging, unstaging and line staging against the index.
- `core/src/patch.rs`: the patch of only the selected lines.
- `core/src/status.rs`: reading the changed files, each untracked file on its own, and the old path of a renamed file.
- `app/src/model/files.rs`: which list a file shows in, its badge letter and its row text.
- `app/src/workspace/detail/file_lists.rs`: the two lists.
- `app/src/workspace/detail/file_rows.rs`: rows with their text, dragging and dropping.
- `app/src/workspace/detail/picking.rs`: picking by click, keys and Cmd+A.
- `app/src/workspace/detail/marks.rs`: the set of picked files.
- `app/src/workspace/detail/staging.rs`: running a change in the background.
- `app/src/workspace/detail/diff_view.rs`: hunk links and the links for picked lines.
- `app/src/workspace/detail/viewing.rs`: whether the open file allows partial staging.
- `app/src/model/diff.rs`: hunks and changed lines of a diff.

### Reference Files

- `app/src/workspace/detail/stack.rs`: the stacked diff of the picked files, read only.
- `app/src/workspace/navigation.rs`: routes the arrow keys and Cmd+A to the lists.
- `core/src/diff.rs`: the staged and unstaged diff of one file.
- `core/src/ops.rs`: shared helpers that run `git` with paths.

## Architecture

### User Flow

```text
drop a row on the other list ----> move the file or the picked files
click a hunk or lines link ------> stage or unstage that part of the file
click / Shift / Cmd / Cmd+A -----> pick files (inside one list)
                |
        change runs in the background
                |
        result reported, repository read again
```

### Event Flow

```text
1. A row is dragged: the dragged files are the picked files, or just that file
       v
2. The other list takes the drop
       v
3. A stage or unstage change starts (ignored while another change runs)
       v
4. The core stages or unstages the paths
       v
5. Success clears the picks and the stacked diff; failure shows a notice
       v
6. The repository is read again and the lists update
```

### Domain Model

```text
Changed file (index letter, working directory letter, conflicted)
└── File state: kind (tracked / untracked / conflicted), staged, unstaged
    └── Shows in the Staged list, the Unstaged list, or both
Picked files: one list, a set of paths, an anchor for ranges
Open diff: file, list, kind, lines, picked lines
    └── Partial staging allowed only for tracked files
Line selection: file, list, picked line numbers
    └── Unstaged list: patch applied forward; Staged list: applied in reverse
```

## Implementation Steps

Step 0 is the outer frame: the acceptance tests that drive the whole window.
Steps 1 to 4 are the parts from the window inward: the user interface, the workspace actions, the model, then the core.
Every test is named in exactly one step, and every test named here exists.

### Step 0: Acceptance Tests

**Red:** Write every test that assembles the whole workspace on real repositories and checks a user scenario end to end, grouped by spec story (`app/src/workspace/tests/staging_acceptance.rs`, `app/src/workspace/tests/staging_diff_acceptance.rs`, `app/src/workspace/tests/renamed_files_acceptance.rs`, `app/src/workspace/tests/staging.rs`, `app/src/workspace/tests/picking.rs`, `app/src/workspace/tests/keys.rs`, `app/src/workspace/tests/stack.rs`, `app/src/workspace/tests/notices.rs`, `app/src/workspace/tests/diffs.rs`).
They are written first and stay failing until the last step.
Dragging and drops (story 1): `dragging_a_row_stages_and_unstages_it_and_the_same_list_does_nothing`, `unstaging_works_before_the_first_commit`, `a_failed_drop_shows_a_notice_and_the_next_one_clears_it`, `clicking_a_row_opens_its_diff_and_the_diff_follows_the_file_across_lists`, `stages_and_unstages_whole_files`, `dropping_a_file_on_the_other_list_stages_or_unstages_it`, `a_failed_stage_shows_a_notice_and_the_next_success_clears_it`, `shows_the_diff_of_a_file_and_follows_it_through_staging`.
Picking (story 2): `clicks_and_keys_pick_files_and_dragging_one_takes_them_all`, `cmd_a_picks_a_single_file_list_and_does_nothing_on_an_empty_one`, `a_pick_that_leaves_the_list_is_dropped_when_the_repository_is_read_again`, `picking_in_one_list_does_not_carry_over_to_the_other`, `a_shift_click_without_an_anchor_picks_only_that_file`, `shift_and_cmd_clicks_pick_files_and_one_action_moves_them_all`, `shift_arrows_grow_the_picked_range_and_a_plain_arrow_picks_one`, `dropping_the_picked_files_moves_them_together`, `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button`, `picking_several_files_shows_all_their_diffs_one_after_another`, `the_stack_is_limited_so_a_huge_pick_stays_light`.
Hunks and lines (story 3): `a_hunk_is_staged_and_unstaged_from_the_diff_links`, `picked_lines_stage_and_unstage_from_the_diff_links`, `a_line_toggles_clear_drops_the_picks_and_no_link_shows_without_picks`, `a_line_added_after_an_unterminated_last_line_stages_with_its_newline`, `stages_only_the_picked_lines_and_unstages_them_again`, `stages_a_whole_hunk`.
Whole files only (story 4): `untracked_and_conflicted_files_have_no_partial_staging_but_stage_whole`, `a_conflicted_file_can_only_be_staged_whole`.
Lists and badges (story 5): `every_file_of_a_new_folder_is_listed_and_a_conflict_is_only_in_unstaged`, `a_file_with_staged_and_unstaged_changes_is_in_both_lists`, `badges_name_the_change_and_a_conflict_is_always_marked`, `a_renamed_file_shows_its_new_path_then_the_old_one_and_the_others_show_their_path`.

**Green:** They fail through Steps 1 to 3 and pass when the last step is done and no stub is left; no code is written for them directly.

**Refactoring:** The two acceptance files share the helpers that drag a row onto a list, click a row with modifiers, and make a conflict.

### Step 1: Screen - Rows, widgets and drawing

**Red:** Write the tests of rows, drag previews and the diff drawing: `clicking_a_file_opens_it_and_dragging_it_to_the_other_list_stages_it`, `dragging_several_files_shows_how_many_and_the_picks_can_be_cleared`, `clicking_a_changed_line_picks_it_for_staging` and `the_links_of_a_diff_clear_and_stage_the_picked_lines_and_hunks` (`app/src/workspace/detail/tests/files.rs`, `app/src/workspace/detail/tests/diff.rs`).
These tests use a stub of the workspace actions that always reports the change as done and changes no repository.

**Green:** Draw the Staged and Unstaged lists with a badge on each row, make rows draggable and lists accept drops, and show the hunk and line links in the diff only for tracked files.
The files are `app/src/workspace/detail/file_lists.rs`, `app/src/workspace/detail/file_rows.rs`, `app/src/workspace/detail/diff_view.rs` and `app/src/workspace/detail/viewing.rs`.
This part is built with the workspace actions stubbed.

**Refactoring:** Keep one file per job, and share the row and drag types between the two lists.

### Step 2: Workspace - staging actions

**Red:** Write the tests that call the workspace actions and state directly: `picks_without_an_anchor_start_from_the_cursor` and `picks_of_files_that_left_the_list_are_dropped` (`app/src/workspace/detail/tests/files.rs`).
These tests use a stub of the git operations that always reports success and leaves the repository as it is.
The rest of this part, the background change and its report, is built against the Step 0 tests and has no tests of its own.

**Green:** Run each change on the selected repository in the background, start nothing while another change runs, report the result, read the repository again, and clear the picks and the stacked diff after a success.
Picking runs in `app/src/workspace/detail/picking.rs` and `app/src/workspace/detail/marks.rs`, and the changes in `app/src/workspace/detail/staging.rs`.
This part is built with the model and the git operations stubbed, and it replaces the stub of the workspace actions that Step 1 used.

**Refactoring:** Send every kind of change through one place that marks busy, reports and refreshes.

### Step 3: Model - file lists, badges and hunks

**Red:** Write `sorts_files_into_the_staged_and_working_directory_lists`, `an_open_diff_stays_on_its_side_or_follows_the_file` and `finds_hunks_and_their_changed_lines` (`app/src/model/tests/diff.rs`), `a_renamed_file_is_labeled_with_its_new_path_then_the_old_one` `a_file_that_was_not_renamed_is_labeled_with_its_path_only` and `a_rename_that_is_staged_is_not_shown_as_one_in_the_unstaged_list` (`app/src/model/tests/labels.rs`).
These tests need no stub of a lower part, because the model reads status letters and diff text only.

**Green:** Decide from the status letters which list a file shows in and which badge it gets, let a conflicted file wait in the Unstaged list, and give a renamed file the row text `new path <- old path`.
Find the hunks and changed lines of a diff.
These are `app/src/model/files.rs` and `app/src/model/diff.rs`.
This part replaces the stubbed file state of the parts above it.

**Refactoring:** Keep the picked-files set as one small type that holds one list, its paths and an anchor.

### Step 4: Core - Git operations and patches

**Red:** Write `stages_unstages_and_commits`, `stages_and_unstages_selected_hunks_and_lines` and `discards_tracked_and_untracked_changes` (`core/tests/staging.rs`), `every_file_in_a_new_folder_is_listed_and_counted` (`core/tests/summarize.rs`), `a_staged_rename_lists_the_new_path_and_the_old_one` and `unstaging_a_renamed_file_brings_back_the_old_path` (`core/tests/staging.rs`), and the patch tests in `core/src/patch/tests.rs`: `selects_a_whole_hunk`, `unselected_changes_keep_their_original_state`, `selecting_one_added_line_turns_the_other_into_nothing_or_context`, `keeps_no_newline_markers_only_after_kept_lines`, `a_kept_last_line_gets_its_newline_back_when_a_line_is_added_after_it`, `a_kept_last_line_with_nothing_added_after_it_stays_context`, `returns_none_without_a_selection_or_hunks`, `a_header_that_is_not_a_hunk_header_is_refused`.
These tests run the real `git` in a temporary repository, so there is no stub below this part.

**Green:** Stage and unstage paths, unstaging differently before the first commit and together with the old path of a renamed file, read every untracked file on its own, and keep the old path of a renamed file.
Build the patch of only the selected lines and apply it to the index, in reverse for the Staged list.
These are `core/src/ops/staging.rs`, `core/src/patch.rs` and `core/src/status.rs`.
This part replaces the last stub, so no stub is left and the Step 0 tests pass.

**Refactoring:** Use one patch builder for both directions and the shared path helpers in `core/src/ops.rs`.

## Verification

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
mise run check
```

### Manual Test

1. Drag a file between the lists, then a picked set of several files.
2. Pick lines and a hunk in a tracked file, stage them, then unstage them.
3. Open an untracked file and a conflicted file and confirm there are no hunk links.
4. Rename a file with `git mv` and confirm its row reads `new path <- old path`, then drag it to the Unstaged list and confirm no deleted file stays staged.

## Considerations

### Reusing Existing Code

One patch builder serves both directions, because unstaging is the same patch applied in reverse.
Staging and unstaging of whole files share the helpers in `core/src/ops.rs`.

### Compatibility

Before the first commit there is no HEAD to restore from, so unstaging removes the paths from the index instead.
