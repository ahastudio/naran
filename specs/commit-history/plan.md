# Implementation Plan: Commit History

## Summary

The feature is built Outside-In with stubs: acceptance tests that drive the window are written first and stay failing, then each part is built with the part below it stubbed, and each next step replaces that stub with the real part.
The parts are the window and its commit list, the workspace that pages, picks and keeps the layout, the graph rows and lanes, and the core library that reads the history.

## Requirements

1. Columns, rows, the Uncommitted changes row, and the unreadable and empty cases (spec Story 1, FR-1, FR-4, FR-11).
2. Graph lanes, HEAD and the dashed link (Story 2, FR-2, FR-3).
3. Paging while scrolling (Story 3, FR-5).
4. Go to HEAD (Story 4, FR-6).
5. Remote branches and All refs, saved and restored (Story 5, FR-7, FR-8).
6. Picking a commit and stepping through the list (Stories 6 and 7, FR-9, FR-10).

## Critical Files

### New Files

- `app/src/workspace/tests/commit_history_acceptance.rs`: the window-driven acceptance tests

### Modified Files

- `app/src/workspace/tests.rs`: one line registers the acceptance tests

### Reference Files

- `app/src/workspace/detail/history_view.rs`: the list, column heads and footer buttons
- `app/src/workspace/detail/history_rows.rs`: the rows
- `app/src/workspace/detail/history.rs`: loading, paging, ref buttons, Go to HEAD
- `app/src/workspace/detail/loaded.rs`: what is read from a repository, page size, the HEAD jump
- `app/src/workspace/detail/commit_picking.rs`, `app/src/workspace/detail/inspecting.rs`, `app/src/workspace/detail/commit.rs`: the picked commit
- `app/src/workspace/detail/commit_info.rs`, `app/src/workspace/detail/commit_view.rs`: message and files
- `app/src/workspace/detail/stepping.rs`: Up and Down
- `app/src/workspace/detail/lanes.rs`, `app/src/workspace/detail/painting.rs`: lanes, HEAD, the dashed link
- `app/src/workspace/layout.rs`: the saved layout that holds the ref choice
- `core/src/history.rs`, `core/src/log.rs`, `core/src/graph.rs`: reading, parsing and lane layout
- `app/src/workspace/tests/history.rs`, `app/src/workspace/tests/keys.rs`, `app/src/workspace/tests/layout.rs`, `app/src/workspace/tests/scrolling.rs`, `app/src/workspace/detail/tests/`, `core/tests/reading.rs`: the other tests

## Architecture

### User Flow

```text
Select repository -> commit list (Graph | Description | Date | Author | Commit)
  Uncommitted changes row first, when there are changes
  scroll near the end -> older page appended
  Go to HEAD          -> HEAD commit picked, older pages read when needed
  Remote branches / All refs -> list read again, choice saved
  click or Up, Down   -> commit picked -> message, details, refs, files
  Uncommitted row     -> Staged and Unstaged lists
```

### Event Flow

```text
1. Window: click, scroll or key
       -> workspace decides: pick, step, page, jump or toggle
2. Workspace: read in the background
       -> history page, commit message, commit files
3. Workspace: result arrives
       -> applied only when the list or the picked commit is still the same
4. List: rows rebuilt for the visible range
       -> graph lanes, HEAD emphasis, dashed link drawn per row
```

### Domain Model

```text
Ref choice (everything, remotes) -> saved with the layout, older form readable
History -> commits + one graph row per commit
Graph row -> dot lane, incoming, passing and outgoing lanes
Loaded repository -> history, more-pages flag, page-in-progress flag
Picked commit -> hash, refs, details, files, open file
HEAD link -> HEAD position, HEAD lane, link lane
```

## Implementation Steps

Step 0 is the outer frame: every test that assembles the whole workspace on real repositories and checks a user scenario end to end, written first.
Steps 1 to 4 go from the outside in, one part each, and every test is named in exactly one step.
Each part step builds its part with the part below stubbed and replaces the previous step's stub with its real part.
Every scenario of spec.md is covered by Step 0, or by an inner step when the scenario is marked as not reachable through the window.

### Step 0: Acceptance Tests

**Red:** the tests under `app/src/workspace/tests/` stay failing until the last step is done.
Story 1 (the list): `the_uncommitted_row_comes_first_under_the_column_heads_when_there_are_changes`, `a_repository_whose_history_cannot_be_read_shows_the_problem_instead_of_rows`, `the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow`.
Story 3 (paging): `scrolling_to_the_end_adds_older_commits_and_a_short_history_adds_none`, `the_history_loads_older_commits_as_the_end_is_reached`.
Story 4 (Go to HEAD): `go_to_head_picks_the_head_commit_each_time_it_is_pressed`, `the_go_to_head_button_reads_older_pages_until_head_shows_up`, `go_to_head_finds_a_head_commit_that_is_older_than_the_loaded_page`, `go_to_head_picks_the_head_commit_when_it_is_already_loaded`, `a_repository_without_commits_has_an_empty_list_and_nothing_to_pick`.
Story 5 (refs): `the_all_refs_button_adds_the_commits_only_git_keeps_refs_for_and_a_second_press_removes_them`, `the_remote_branches_button_removes_remote_only_commits_and_a_second_press_brings_them_back`, `the_ref_buttons_save_their_choice_and_a_restart_keeps_it`, `a_layout_file_in_the_older_form_still_sets_the_refs`.
Story 6 (picking): `clicking_a_commit_shows_its_message_and_files_and_the_arrows_step_through_the_list`, `clicking_a_commit_with_a_body_and_another_committer_reads_both`, `picking_a_commit_shows_its_files_and_diff_and_the_uncommitted_row_brings_the_lists_back`, `a_clean_repository_starts_on_its_top_commit`.
Story 7 (stepping): `arrow_keys_move_through_the_list_that_was_clicked_last`, `stepping_stays_inside_the_list`, `the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row`.
They are in `app/src/workspace/tests/commit_history_acceptance.rs`, `app/src/workspace/tests/layout.rs`, `app/src/workspace/tests/history.rs`, `app/src/workspace/tests/keys.rs` and `app/src/workspace/tests/scrolling.rs`.
The feature has no tests in `app/src/tests.rs`.

**Green:** they fail until the last step is done and pass when it is; no code is written for them directly.

**Refactoring:** the acceptance file shares small helpers that read the picked commit, count the listed commits, turn the scroll wheel and select a repository.

### Step 1: Screen - rows, buttons and drawing

**Red:** the tests that click or draw elements of the commit list: `the_commit_list_rows_and_buttons_pick_and_move` and `hiding_remote_branches_reloads_the_list_and_flips_the_label` in `app/src/workspace/detail/tests/commits.rs`.
These tests use a stub of the Workspace that gives back a fixed loaded repository.

**Green:** this part is built with the Workspace stubbed: `app/src/workspace/detail/history_view.rs` and `app/src/workspace/detail/history_rows.rs` show the heads, an Uncommitted changes row only when there are changes, one row per commit, the footer buttons, and the reason when the history cannot be read.
The list asks for the next page when it is scrolled near its end.

**Refactoring:** build only the rows in view and keep each function short.

### Step 2: Workspace - workspace actions and graph decisions

**Red:** the tests that call workspace actions or graph decisions directly, with the reading stubbed by a loaded repository that holds a given list of commits or a given failure.
Reading failures and paging: `a_page_for_a_list_that_changed_meanwhile_is_dropped`, `details_that_cannot_be_read_stay_empty_and_a_failed_page_adds_nothing`, `a_history_that_could_not_be_read_has_no_rows_pages_head_or_top`.
Go to HEAD and the ref button: `a_repository_without_commits_has_no_head_to_go_to`, `showing_all_refs_reloads_the_list_and_flips_the_label`.
Picking: `a_commit_with_a_body_and_another_committer_shows_both`.
Graph, HEAD and the dashed link in `app/src/workspace/detail/tests/lanes.rs`: `the_link_goes_straight_to_a_head_at_the_top`, `the_link_passes_the_commits_above_a_head_on_another_branch`, `the_link_takes_a_lane_of_its_own_when_the_head_lane_is_busy_above`, `without_a_head_commit_in_the_list_there_is_no_link`, `a_detached_head_counts_too`, `head_is_the_commit_labeled_head_or_head_arrow_branch`, `the_graph_is_wide_enough_for_the_link_lane`.

**Green:** this part is built with the core library stubbed, and it replaces the workspace stub of Step 1 with the real part: `app/src/workspace/detail/history.rs`, `app/src/workspace/detail/loaded.rs`, `app/src/workspace/detail/commit_picking.rs`, `app/src/workspace/detail/inspecting.rs`, `app/src/workspace/detail/stepping.rs`, `app/src/workspace/detail/lanes.rs` and `app/src/workspace/detail/painting.rs` page, jump, pick, step, find HEAD and draw the link, and drop stale results; the ref choice is kept in `app/src/workspace/layout.rs`.

**Refactoring:** the jump to HEAD goes on by itself each time an older page arrives, and the drawing stays apart from the lane decisions.

### Step 3: Model - the app model

This part has no tests of its own for this feature: the commit history uses the app model only for the age shown in the Date column.

**Green:** it is built against the Step 0 tests; `app/src/model.rs` gives the age text of a commit.

**Refactoring:** none needed.

### Step 4: Core - reading history and laying out lanes

**Red:** `history_pages_join_into_the_full_history`, `history_hides_refs_git_keeps_for_itself_unless_asked`, `remote_branches_can_be_left_out_of_the_history` and `refs_read_the_older_saved_form` in `core/tests/reading.rs`, and `linear_history_stays_in_one_lane`, `merge_opens_and_closes_a_lane`, `unrelated_tip_takes_a_free_lane` and `handles_missing_parents_in_a_truncated_log` in `core/src/graph.rs`.

**Green:** no stub is left: the real core replaces the stub of Step 2, and the Step 0 tests pass; `core/src/history.rs`, `core/src/log.rs` and `core/src/graph.rs` choose the refs, read pages, parse commits and lay out lanes.

**Refactoring:** one argument builder serves the first page, later pages and search.

## Verification

### Build

```bash
cargo test --workspace --no-run
```

### Test

```bash
cargo test --workspace
mise run check
```

### Manual Test

1. Open a repository with more than 200 commits and scroll to the end.
2. Press Go to HEAD on an old checkout.
3. Toggle Remote branches and All refs, restart, and check they are kept.

## Considerations

### Reusing Existing Code

The same way of choosing refs serves the first page, later pages and search, all in `core/src/history.rs`.

### Compatibility

Saved layout files in the older ref form must keep loading.
