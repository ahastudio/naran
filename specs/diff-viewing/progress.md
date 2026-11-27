# Progress Log

> **Update this file when a phase is finished or a problem occurs.**

## Session 2026-11-16

### Phase 1: Requirements & Discovery ✅

**Work log**:

1. Read `README.md` and `AGENTS.md` for the screen, the rules and the checks.
2. Read the diff, picture, stacked diff and commit picking code under `app/src/workspace/detail/`.
3. Read the existing window, unit and core tests for the feature.
4. Wrote eight user stories with numbered scenarios in `specs/diff-viewing/spec.md`, each with a Test line.

**Created/modified files**:

- `specs/diff-viewing/spec.md` (created)
- `specs/diff-viewing/README.md` (created)

### Phase 2: Planning & Structure ✅

**Work log**:

1. Wrote a five-step Outside-In plan in `specs/diff-viewing/plan.md`, from window tests inward to the core reads.
2. Mapped every scenario of spec.md to a window test, or marked it as inner level where the window cannot reach it.
3. Checked with `scripts/check-specs.py` that every named test exists.

**Created/modified files**:

- `specs/diff-viewing/plan.md` (created)
- `specs/diff-viewing/tasks.md` (created)
- `specs/diff-viewing/findings.md` (created)
- `specs/diff-viewing/progress.md` (created)

### Phase 3: Implementation ✅

**Work log**:

1. Step 0: the acceptance tests are in `app/src/workspace/tests/diff_viewing_acceptance.rs`, `diff_viewing_files_acceptance.rs`, `diffs.rs`, `stack.rs`, `scrolling.rs`, `sticky_names.rs` and `keys.rs`.
2. Step 1: the pane and stack drawing are in `app/src/workspace/detail/diff_view.rs`, `diff_rows.rs`, `stack_view.rs` and `pictures.rs`, and the text selection in `app/src/selectable/`, with tests in `app/src/selectable/region/tests.rs`, `app/src/selectable/texts/tests.rs`, `app/src/workspace/detail/tests/diff.rs` and `app/src/workspace/detail/tests/stack.rs`.
3. Step 2: reading, stacking and picking are in `viewing.rs`, `versions.rs`, `stack.rs` and `commit_picking.rs` under `app/src/workspace/detail/`, with tests in `app/src/workspace/detail/tests/`.
4. Step 3: the rows are in `app/src/model/diff.rs`, with tests in `app/src/model/tests/diff.rs`.
5. Step 4: the git reads are in `core/src/diff.rs` and `core/src/blob.rs`, with tests in `core/tests/reading.rs` and `core/src/blob.rs`.
6. No source file was changed.

**Created/modified files**:

- none

### Phase 4: Testing ✅

**Work log**:

1. Added 14 window tests in `app/src/workspace/tests/diff_viewing_acceptance.rs` and `app/src/workspace/tests/diff_viewing_files_acceptance.rs`.
2. Ran the feature tests: 74 passed in `naran-app`, 2 passed in `naran-core`.

**Created/modified files**:

- `app/src/workspace/tests/diff_viewing_acceptance.rs` (created)
- `app/src/workspace/tests/diff_viewing_files_acceptance.rs` (created)
- `app/src/workspace/tests.rs` (two `mod` lines appended)

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `shows_the_diff_of_a_file_and_follows_it_through_staging` | modified file, stage, unstage, commit | diff follows the list, closes when unchanged | as expected | ✅ |
| `shows_an_untracked_file_whole` | untracked `draft.txt` | the diff is `+draft` | `+draft` | ✅ |
| `a_modified_file_shows_its_lines_without_the_git_header_or_markers` | a committed file with one line changed, click it, Cmd+A | "keep\ngone\nnew" | "keep\ngone\nnew" | ✅ |
| `the_file_name_stays_above_the_diff_while_it_scrolls` | swipe down 600 | name fixed, lines moved up | as expected | ✅ |
| `the_name_of_a_past_commit_file_stays_above_its_diff_too` | tall commit file, swipe down | name fixed, lines moved up | as expected | ✅ |
| `a_diff_with_a_long_line_scrolls_sideways` | long line, swipe 400 px; short file | moved 400; not moved | as expected | ✅ |
| `the_diff_of_a_past_commit_scrolls_sideways_too` | commit with a long line, swipe | moved 400 | moved 400 | ✅ |
| `another_file_opens_at_the_left_edge_of_its_diff` | scrolled sideways, open another file | offset 0 | offset 0 | ✅ |
| `another_file_of_a_past_commit_opens_at_the_left_edge_too` | scrolled sideways, open another commit file | offset 0 | offset 0 | ✅ |
| `dragging_over_the_text_of_a_diff_selects_it_for_copying` | mouse down on line 1, drag to line 3 of a three-line file | the text of "second" is selected | selected text contains "second" | ✅ |
| `a_drag_over_three_lines_copies_the_cut_of_each_line` | drag from inside line 1 to past the end of line 3 | the cut of each line | "rst\nsecond\nthird" | ✅ |
| `a_drag_upwards_copies_the_same_text_as_a_drag_downwards` | the same two points dragged down, then up | equal copies | equal copies | ✅ |
| `a_plain_click_on_a_diff_copies_nothing` | one click on line 1 | nothing copied | nothing copied | ✅ |
| `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button` | Cmd+A in the list, then after pressing the diff | every file picked; then the diff text picked and files left alone | as expected | ✅ |
| `a_long_file_is_cut_and_a_note_says_how_much_is_left` | an untracked file of 3005 lines | the omitted note is drawn | omitted note drawn | ✅ |
| `an_untracked_file_that_is_big_or_not_text_is_told_so_in_the_pane` | a 600 KB file and a binary file, each opened | a too-large message, then "The file is not text." | both messages shown | ✅ |
| `a_picture_over_the_size_limit_is_refused_with_an_error` | a png of 8 MB plus one byte | an error naming the file | error naming the file | ✅ |
| `an_image_shows_before_on_the_left_and_after_on_the_right` | changed, new and deleted png | both, only After, only Before; no text diff read | as expected | ✅ |
| `a_staged_image_shows_the_committed_picture_and_the_staged_one` | a committed png changed and staged, opened from Staged | both pictures | before and after present | ✅ |
| `an_image_in_a_past_commit_has_no_before_when_it_was_added` | commit that added a png | no Before, an After | as expected | ✅ |
| `picking_several_files_shows_all_their_diffs_one_after_another` | one file, three files, one file, then stage | no stack, 3 heads, no stack, cleared | as expected | ✅ |
| `a_past_commit_hides_the_stack_and_coming_back_shows_it_again` | stack, pick a commit, come back | hidden, then shown again | as expected | ✅ |
| `the_stack_is_limited_so_a_huge_pick_stays_light` | 45 files of 600 lines | 40 heads, line and file notes | as expected | ✅ |
| `an_image_among_the_picked_files_shows_a_note_in_the_stack` | a text file and a png picked | one image note in the stack | one image note | ✅ |
| `select_all_in_a_list_of_one_file_opens_it_without_a_stack` | Cmd+A in a list of one file | no stack, the file's diff is open | no stack, diff `+1` | ✅ |
| `stacked_diffs_scroll_sideways_too` | two files, one wide, swipe | moved 400 | moved 400 | ✅ |
| `stacked_diffs_keep_the_name_of_the_file_at_the_top_until_the_next_file` | two 40-line files, scroll down | head sticks, then the next takes over | as expected | ✅ |
| `the_sticky_file_head_stays_at_the_left_edge_while_the_stack_scrolls_sideways` | two wide files stacked, swipe down, swipe sideways | sticky head x unchanged and at the left edge of the list | x unchanged and at the left edge | ✅ |
| `the_files_of_a_commit_are_picked_with_shift_cmd_and_cmd_a_and_stacked` | click, Shift, Cmd, Cmd+A, Shift+Up on a commit of four files | picks as listed in spec.md | as expected | ✅ |
| `shift_with_the_arrow_keys_grows_the_picked_files_of_a_commit` | Shift+Down twice, Shift+Up once on a commit of three files | 3 picked, then 2 | 3, then 2 | ✅ |
| `the_stack_of_a_commit_has_its_lines_and_a_note_for_its_image` | Cmd+A in a commit of a text file and a png | stack with the added line and one note | line and one note | ✅ |
| `each_kind_of_line_has_its_colors` | each line kind | its colors; tabs as four spaces | as expected | ✅ |
| `a_diff_longer_than_the_limit_is_cut_with_a_note` | a file over the line limit | the omitted note is drawn | omitted note drawn | ✅ |
| `a_diff_without_lines_says_so` | open diff set to no lines | no diff lines are drawn | no diff lines drawn | ✅ |
| `the_head_of_the_file_at_the_top_sticks_until_the_next_head_pushes_it` | rows with two heads at several scroll offsets | none, a, a pushed, b, b, none | as expected | ✅ |
| `only_a_file_head_has_a_name_and_a_badge` | a head, a note, a line | name and badge only for the head | as expected | ✅ |
| `letters_outside_ascii_take_two_columns` | "abc", "한글 a", "" | 3, 6, 0 | 3, 6, 0 | ✅ |
| `the_widest_stacked_row_is_found_by_its_columns` | tabs, Korean letters, a long path | the widest row by columns | as expected | ✅ |
| `copies_the_selected_part_of_each_line_across_lines` | selection from (0,6) to (2,5) | "line\nsecond\nthird" | as expected | ✅ |
| `a_selection_made_backwards_reads_the_same` | selection from (1,2) back to (0,1) | "bc\nde" | "bc\nde" | ✅ |
| `nothing_is_selected_when_both_ends_are_the_same` | both ends at (0,1) | no text, no highlight | none | ✅ |
| `a_selection_that_cuts_a_letter_in_two_copies_nothing` | a Korean text cut inside a letter | nothing | nothing | ✅ |
| `select_all_takes_every_line_and_new_text_drops_the_selection` | select all, same lines, then other lines | "ab\n\ncd", kept, then dropped | as expected | ✅ |
| `dragging_across_painted_text_selects_it_and_a_plain_click_selects_nothing` | drag across two lines, then a click | "alpha beta\ngamma", then nothing | as expected | ✅ |
| `an_untracked_file_that_is_big_or_not_text_is_told_so_instead_of_shown` | a file over the size limit, a binary file | too-large message, "The file is not text." | both messages | ✅ |
| `a_staged_image_shows_the_committed_picture_before_and_the_staged_one_after` | staged changed png | both pictures | as expected | ✅ |
| `an_image_among_the_picked_files_gets_a_note_instead_of_lines` | a png job | head and image note | as expected | ✅ |
| `a_file_that_is_no_longer_changed_is_left_out_of_the_stack` | no changes | no job | no job | ✅ |
| `a_stack_that_cannot_be_read_is_an_error` | a folder that is not a repository | an error | an error | ✅ |
| `the_same_picked_files_are_not_read_again` | select all, refresh | the stack stays shown | stack stays shown | ✅ |
| `picking_the_same_files_twice_and_a_single_file_list_are_handled` | one-file list, a file that is gone | handled without a stack | handled | ✅ |
| `the_stack_of_a_commit_has_its_files_images_and_the_count_left_out` | two files with 3 more; a missing repository | added line, image note, a note about 3 more; an error | as expected | ✅ |
| `picks_cannot_move_through_files_that_are_not_there` | failed or empty files, no repository | nothing moves, no stack | as expected | ✅ |
| `classifies_diff_lines` | line prefixes | line kinds | as expected | ✅ |
| `diff_rows_number_lines_and_drop_the_file_header` | diff with a header and two hunks | rows with titles and numbers | as expected | ✅ |
| `diff_rows_show_an_untracked_file_as_added_lines_and_keep_messages` | "a" and a tab line; a message | added rows; a note | as expected | ✅ |
| `reads_files_as_they_are_on_each_side` | a picture on each side, a missing file, a bad hash | bytes, none, an error | as expected | ✅ |
| `a_file_over_the_limit_is_refused` | a size over 8 MB | an error naming the file | as expected | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| none recorded | none recorded | 0 | none recorded |

## 5-Question Reboot Check

Questions to recover context when resuming work:

| Question | Answer |
| -------- | ------ |
| 1. Which phase are we in? | Phase 4 (all phases done) |
| 2. What is next? | None (all done) |
| 3. What is the goal? | Show the changes of picked files, working and past, as diff, pictures or stacks |
| 4. What have we learned? | See findings.md: why the sticky head stays left |
| 5. What was done? | Spec, plan and notes; 14 window tests added; 74 app and 2 core feature tests pass |
