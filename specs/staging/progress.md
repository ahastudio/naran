# Progress Log

> **Update this file when you finish a phase or hit a problem.**

## Session 2026-11-16

### Phase 1: Requirements & Discovery ✅

**Work done**:

1. Read `README.md` and `AGENTS.md`: Naran only stages and unstages and never commits, fetches, pulls or pushes.
2. Explored the staging code in `core/` and `app/` and the tests beside it.
3. Wrote the stories and scenarios, each with the test that proves it.

**Created/modified files**:

- `specs/staging/README.md` (new)
- `specs/staging/spec.md` (new)

### Phase 2: Planning & Structure ✅

**Work done**:

1. Planned four steps from the window inward: Screen, workspace actions, model, core.
2. Gave each step its Red tests, and covered every spec scenario by a step.
3. Checked that every named test and file exists.

**Created/modified files**:

- `specs/staging/plan.md` (new)

### Phase 3: Implementation ✅

**Work done**:

1. Step 0: the workspace tests in `app/src/workspace/tests/` (`staging_acceptance.rs`, `staging_diff_acceptance.rs`, `renamed_files_acceptance.rs`, `staging.rs`, `picking.rs`, `keys.rs`, `stack.rs`, `notices.rs`, `diffs.rs`).
2. Step 1: lists, rows, drag and drop and diff links in `app/src/workspace/detail/file_lists.rs`, `file_rows.rs`, `diff_view.rs` and `viewing.rs`, with tests in `app/src/workspace/detail/tests/files.rs` and `diff.rs`.
3. Step 2: picking and background changes in `app/src/workspace/detail/picking.rs`, `marks.rs` and `staging.rs`, with state tests in `app/src/workspace/detail/tests/files.rs`.
4. Step 3: file state, badges and row text in `app/src/model/files.rs` and `diff.rs`, with tests in `app/src/model/tests/diff.rs` and `labels.rs`.
5. Step 4: staging, line patches and status (with the old path of a rename) in `core/src/ops/staging.rs`, `core/src/patch.rs` and `core/src/status.rs`, the diff of a staged rename in `core/src/diff.rs` and its patch header in `core/src/patch.rs`, with tests in `core/tests/staging.rs` and `core/src/patch/tests.rs`.

**Created/modified files**:

- `app/src/workspace/tests/staging_acceptance.rs` (new)
- `app/src/workspace/tests/staging_diff_acceptance.rs` (new)
- `app/src/workspace/tests/renamed_files_acceptance.rs` (new)
- `app/src/model/tests/labels.rs` (new)
- `app/src/workspace/tests.rs` (two `mod` lines appended)
- `specs/staging/findings.md` (new)
- `specs/staging/tasks.md` (new)

### Phase 4: Testing ✅

**Work done**:

1. Wrote an acceptance test through the window for every scenario of the spec.
2. Ran the staging tests of the app and the core, including the patch tests; all passed.
3. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo run -p quality -- core app tools` and `python3 -I scripts/check-specs.py` passed.

**Created/modified files**:

- `specs/staging/progress.md` (new)

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `dragging_a_row_stages_and_unstages_it_and_the_same_list_does_nothing` (staging_acceptance.rs) | drag a row onto the same list, the other list, and back | no change on the same list; staged, then unstaged | passed | ✅ |
| `unstaging_works_before_the_first_commit` (staging_acceptance.rs) | repository without a commit, stage then unstage by drag | the file is staged, then listed as untracked again | passed | ✅ |
| `a_failed_drop_shows_a_notice_and_the_next_one_clears_it` (staging_diff_acceptance.rs) | drag a file that was deleted on disk, then another file | a Stage failed notice, then no notice | passed | ✅ |
| `clicking_a_row_opens_its_diff_and_the_diff_follows_the_file_across_lists` (staging_acceptance.rs) | click a row, drag it to Staged and back | the diff opens, then follows the file to Staged and back to Unstaged | passed | ✅ |
| `stages_and_unstages_whole_files` (workspace/tests/staging.rs) | stage and unstage an untracked file | added, then untracked again, last report succeeded | passed | ✅ |
| `dropping_a_file_on_the_other_list_stages_or_unstages_it` (workspace/tests/staging.rs) | move a file to its own list, the other, and back | no change, staged, unstaged | passed | ✅ |
| `a_failed_stage_shows_a_notice_and_the_next_success_clears_it` (workspace/tests/notices.rs) | stage a missing file, then a real one | a Stage failed notice, then none | passed | ✅ |
| `shows_the_diff_of_a_file_and_follows_it_through_staging` (workspace/tests/diffs.rs) | open a diff, stage, unstage, then commit the file | the diff follows the file between lists and closes when the file has no changes | passed | ✅ |
| `clicks_and_keys_pick_files_and_dragging_one_takes_them_all` (staging_acceptance.rs) | clicks, Shift and Cmd clicks, Shift+Down/Up, then a drag | picks follow the keys; the drag moves all picked files and clears the picks | passed | ✅ |
| `cmd_a_picks_a_single_file_list_and_does_nothing_on_an_empty_one` (staging_acceptance.rs) | Cmd+A with one file, then with no changes | one file picked, then nothing picked | passed | ✅ |
| `a_pick_that_leaves_the_list_is_dropped_when_the_repository_is_read_again` (staging_acceptance.rs) | three files picked, one deleted, Cmd+R | only the two remaining files stay picked | passed | ✅ |
| `picking_in_one_list_does_not_carry_over_to_the_other` (staging_acceptance.rs) | pick in Staged, Shift-click in Unstaged | only the Unstaged file is picked | passed | ✅ |
| `a_shift_click_without_an_anchor_picks_only_that_file` (staging_acceptance.rs) | Shift-click a file with nothing picked | only that file is picked | passed | ✅ |
| `shift_and_cmd_clicks_pick_files_and_one_action_moves_them_all` (workspace/tests/picking.rs) | plain, Shift and Cmd clicks, then one move | picks as clicked; the picked files move together | passed | ✅ |
| `shift_arrows_grow_the_picked_range_and_a_plain_arrow_picks_one` (workspace/tests/picking.rs) | Shift+Down twice, Shift+Up, plain arrow | range grows and shrinks; a plain arrow picks one | passed | ✅ |
| `dropping_the_picked_files_moves_them_together` (workspace/tests/picking.rs) | move two files to Staged and back | both move each way | passed | ✅ |
| `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button` (workspace/tests/keys.rs) | Cmd+A on the list, then on the diff text | all four files picked; after pressing the diff, the text is picked instead | passed | ✅ |
| `picking_several_files_shows_all_their_diffs_one_after_another` (workspace/tests/stack.rs) | pick several files | their diffs are stacked | passed | ✅ |
| `the_stack_is_limited_so_a_huge_pick_stays_light` (workspace/tests/stack.rs) | 45 files of 600 lines picked | the stack is limited | passed | ✅ |
| `a_hunk_is_staged_and_unstaged_from_the_diff_links` (staging_diff_acceptance.rs) | click Stage hunk, then Unstage hunk in the Staged diff | the hunk is in the index, then out | passed | ✅ |
| `picked_lines_stage_and_unstage_from_the_diff_links` (staging_diff_acceptance.rs) | pick one of two added lines, Stage lines, then Unstage lines | only that line is staged, then it is not staged | passed | ✅ |
| `a_line_toggles_clear_drops_the_picks_and_no_link_shows_without_picks` (staging_diff_acceptance.rs) | click a line twice, again, then Clear | no Stage lines link without picks; second click unpicks; Clear drops the picks | passed | ✅ |
| `a_line_added_after_an_unterminated_last_line_stages_with_its_newline` (staging_diff_acceptance.rs) | stage the added line after a last line without newline | the staged file ends with a newline | passed | ✅ |
| `stages_only_the_picked_lines_and_unstages_them_again` (workspace/tests/staging.rs) | pick an added line, apply, then unstage it | only that line is staged; the rest stays unstaged; then nothing is staged | passed | ✅ |
| `stages_a_whole_hunk` (workspace/tests/staging.rs) | a file with two hunks, apply the first | the first hunk is staged, the second is not | passed | ✅ |
| `untracked_and_conflicted_files_have_no_partial_staging_but_stage_whole` (staging_diff_acceptance.rs) | open an untracked and a conflicted file, click a line, drag each to Staged | no hunk or line links, no picked line; both stage whole | passed | ✅ |
| `a_conflicted_file_can_only_be_staged_whole` (workspace/tests/staging.rs) | apply all lines of a conflicted diff | the conflict stays | passed | ✅ |
| `every_file_of_a_new_folder_is_listed_and_a_conflict_is_only_in_unstaged` (staging_diff_acceptance.rs) | new folder with three files, a conflicted file | each file has a row; the conflict is in Unstaged only | passed | ✅ |
| `a_file_with_staged_and_unstaged_changes_is_in_both_lists` (staging_diff_acceptance.rs) | a file staged, then changed again | a row in each list | passed | ✅ |
| `badges_name_the_change_and_a_conflict_is_always_marked` (staging_acceptance.rs) | changes with letters A, D, R, M, ?, a conflict | badge letters A D R M ? and ! for a conflict | passed | ✅ |
| `a_renamed_file_shows_only_the_lines_that_changed` (renamed_files_acceptance.rs) | a committed file renamed with `git mv`, one line changed, staged, its row clicked | the diff has `+TWO` and `-two` and no `+one` | passed | ✅ |
| `a_rename_without_edits_opens_a_diff_that_says_what_the_file_was` (renamed_files_acceptance.rs) | a committed file renamed with `git mv`, not changed, its row clicked | the diff has `rename from notes.txt` and no hunk line | passed | ✅ |
| `a_renamed_file_shows_its_new_path_then_the_old_one_and_the_others_show_their_path` (renamed_files_acceptance.rs) | a staged `git mv` and a new file | the row reads `GUIDE.md <- README.md`, the new file's row its path only | passed | ✅ |
| `clicking_a_file_opens_it_and_dragging_it_to_the_other_list_stages_it` (detail/tests/files.rs) | click a row, then a mouse drag to Staged | the diff opens; the file is in Staged files | passed | ✅ |
| `dragging_several_files_shows_how_many_and_the_picks_can_be_cleared` (detail/tests/files.rs) | a drag of two files, then clearing picks | preview reads 2 files; picked lines cleared | passed | ✅ |
| `clicking_a_changed_line_picks_it_for_staging` (detail/tests/diff.rs) | click lines of an open diff | a changed line gets picked | passed | ✅ |
| `the_links_of_a_diff_clear_and_stage_the_picked_lines_and_hunks` (detail/tests/diff.rs) | pick, unpick, Clear, Stage lines, Stage hunk | picks toggle and clear; the line is staged with its newline; the hunk is staged | passed | ✅ |
| `picks_without_an_anchor_start_from_the_cursor` (detail/tests/files.rs) | a Shift range with no anchor | no range; an extension anchors at the cursor | passed | ✅ |
| `picks_of_files_that_left_the_list_are_dropped` (detail/tests/files.rs) | picks of files no longer listed | they are dropped | passed | ✅ |
| `sorts_files_into_the_staged_and_working_directory_lists` (model/tests/diff.rs) | status letter pairs | list membership by letters | passed | ✅ |
| `an_open_diff_stays_on_its_side_or_follows_the_file` (model/tests/diff.rs) | a file changing lists | the diff stays on its side or follows the file | passed | ✅ |
| `finds_hunks_and_their_changed_lines` (model/tests/diff.rs) | a diff text | hunk ranges and changed line indexes | passed | ✅ |
| `a_rename_says_what_the_file_was_even_when_no_line_changed` (model/tests/diff.rs) | a rename header with similarity 100% | one note, `Renamed from a.txt` | passed | ✅ |
| `a_renamed_file_is_labeled_with_its_new_path_then_the_old_one` (model/tests/labels.rs) | a change with an old path | `new path <- old path` | passed | ✅ |
| `a_file_that_was_not_renamed_is_labeled_with_its_path_only` (model/tests/labels.rs) | a change without an old path | its path only | passed | ✅ |
| `a_rename_that_is_staged_is_not_shown_as_one_in_the_unstaged_list` (model/tests/labels.rs) | a staged rename with a later edit | the Staged label has both paths, the Unstaged label the new path | passed | ✅ |
| `stages_unstages_and_commits` (core/tests/staging.rs) | stage and unstage paths, also before the first commit | index changes accordingly | passed | ✅ |
| `stages_and_unstages_selected_hunks_and_lines` (core/tests/staging.rs) | a hunk, then lines, staged and unstaged | only those parts move | passed | ✅ |
| `discards_tracked_and_untracked_changes` (core/tests/staging.rs) | discard tracked and untracked changes | changes are gone | passed | ✅ |
| `every_file_in_a_new_folder_is_listed_and_counted` (core/tests/summarize.rs) | a new folder with files | each file listed and counted | passed | ✅ |
| `a_staged_rename_lists_the_new_path_and_the_old_one` (core/tests/staging.rs) | `git mv` of a committed file and a new file | the rename lists both paths, the new file has no old path | passed | ✅ |
| `a_staged_rename_shows_only_what_changed_and_its_lines_unstage_without_undoing_the_rename` (core/tests/staging.rs) | a staged rename with a changed line and a Korean file name; that line unstaged | the diff has only `-line2` and `+CHANGED`; the status reads `RM old.txt -> 한글.txt` | passed | ✅ |
| `unstaging_a_renamed_file_brings_back_the_old_path` (core/tests/staging.rs) | a staged rename, its new path unstaged | the old path is deleted in the working directory only, the new path is untracked | passed | ✅ |
| `selects_a_whole_hunk` (core/src/patch/tests.rs) | a whole hunk selected | the patch holds the hunk | passed | ✅ |
| `unselected_changes_keep_their_original_state` (core/src/patch/tests.rs) | a partial selection | unselected changes become context or are dropped | passed | ✅ |
| `selecting_one_added_line_turns_the_other_into_nothing_or_context` (core/src/patch/tests.rs) | one of two added lines selected | the other becomes nothing or context | passed | ✅ |
| `keeps_no_newline_markers_only_after_kept_lines` (core/src/patch/tests.rs) | the no-newline marker | kept only after a kept line | passed | ✅ |
| `a_kept_last_line_gets_its_newline_back_when_a_line_is_added_after_it` (core/src/patch/tests.rs) | a line added after an unterminated last line | the last line is removed and added again | passed | ✅ |
| `a_kept_last_line_with_nothing_added_after_it_stays_context` (core/src/patch/tests.rs) | an unterminated last line with nothing added | it stays context | passed | ✅ |
| `returns_none_without_a_selection_or_hunks` (core/src/patch/tests.rs) | no selection or no hunks | no patch | passed | ✅ |
| `a_header_that_is_not_a_hunk_header_is_refused` (core/src/patch/tests.rs) | a broken hunk header | it is refused | passed | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| none recorded | none recorded | 0 | none recorded |

## 5-Question Reboot Check

When resuming work, recover the context with these questions:

| Question | Answer |
| -------- | ------ |
| 1. Which phase are we in? | Phase 4: Testing (all four phases done) |
| 2. What is next? | None (all done) |
| 3. What is the goal? | Staged and Unstaged lists with drag, multi-pick, and hunk and line staging; untracked and conflicted files whole only |
| 4. What have we learned so far? | See findings.md |
| 5. What is done? | The feature, its tests and 18 acceptance tests in three files exist; all staging tests, clippy, the quality check and the specs check passed |
