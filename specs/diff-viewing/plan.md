# Implementation Plan: Viewing Changes

## Summary

The feature is built Outside-In with stubs: the acceptance tests are written first and fail, the outermost part is built with everything below it stubbed, and each next part replaces the stub of the one above it with the real part while stubbing its own inside.
The acceptance tests pass only when the innermost part replaces the last stub.
The diff pane shows the stacked diffs, the open file of a picked commit, or the open file of the working lists, and all reading is done in the background.

## Requirements

1. Text diff with line numbers, sideways scrolling, a kept file name row and selectable text.
2. Before and After pictures for images.
3. Stacked diffs with limits, an image note and a sticky file head.
4. Files of past commits, with picking of several files.

## Critical Files

### New Files

- `app/src/workspace/detail/diff_view.rs`, `diff_rows.rs`, `viewing.rs`, `versions.rs`, `pictures.rs`, `stack.rs`, `stack_view.rs`, `commit_view.rs`, `commit_picking.rs`
- `app/src/model/diff.rs`, `app/src/selectable/`
- `core/src/diff.rs`, `core/src/blob.rs`

### Modified Files

none

### Reference Files

- `app/src/workspace/detail/diff_view.rs`: chooses the stacked, commit or working view.
- `app/src/workspace/detail/diff_rows.rs`: rows, line numbers, the line cut, sideways scrolling.
- `app/src/workspace/detail/viewing.rs`: the open working file, background reads, the left-edge reset.
- `app/src/workspace/detail/versions.rs`: which two versions are compared, untracked files.
- `app/src/workspace/detail/pictures.rs`: image formats and the Before and After pair.
- `app/src/workspace/detail/stack.rs`: the stacked rows and their limits.
- `app/src/workspace/detail/stack_view.rs`: the list of visible rows and the sticky head.
- `app/src/workspace/detail/commit_view.rs`: the diff of a commit file.
- `app/src/workspace/detail/commit_picking.rs`: picking commit files and their stack.
- `app/src/model/diff.rs`: line kinds and diff rows.
- `app/src/selectable/`: selectable and copyable text.
- `core/src/diff.rs`, `core/src/blob.rs`: the git reads.

## Architecture

### User Flow

```text
click a file / Shift or Cmd click / Cmd+A / Shift+Up or Down
        |
        v
one file open ----------------> text diff, or Before and After
two or more files picked -----> stacked diffs
        |
        v
scroll down or sideways, drag to select, copy
```

### Event Flow

```text
1. The user opens a file or changes the picked files
       |
2. The pane shows "Reading…" and a background read starts
       |
3. The core reads the diff, or the bytes of both pictures
       |
4. The result is kept only if the same file or picks are still shown
       |
5. The pane draws diff rows, the picture pair, or the stacked list
```

### Domain Model

```text
Open working file         diff lines or pictures
Open commit               picked files, anchor of the range, file list
Stacked diffs             picked files, rows (or an error)
Stacked row               file head | diff line | note
Diff row                  hunk title | note | line (kind, old number, new number, text)
Pictures                  before (may be missing), after (may be missing)
```

## Implementation Steps

Step 0 is the outer frame: every test that assembles the whole workspace on real repositories and checks a user scenario end to end.
Steps 1 to 4 are the parts from the window inward, each with its own loop, and every test is named in one step only.
Scenarios of spec.md marked as inner level are covered by the Red of the part that owns them.

### Step 0: Acceptance Tests

**Red:** Write the tests under `app/src/workspace/tests/` that drive the whole workspace, grouped by story of spec.md.
The new ones are in `app/src/workspace/tests/diff_viewing_acceptance.rs` and `app/src/workspace/tests/diff_viewing_files_acceptance.rs`; the others are in `diffs.rs`, `stack.rs`, `scrolling.rs`, `sticky_names.rs` and `keys.rs` of the same folder.
No test of this feature is in `app/src/tests.rs`.
- Text diff (story 1): `shows_the_diff_of_a_file_and_follows_it_through_staging`, `shows_an_untracked_file_whole`, `a_modified_file_shows_its_lines_without_the_git_header_or_markers`.
- Long files and sideways scrolling (story 2): `the_file_name_stays_above_the_diff_while_it_scrolls`, `the_name_of_a_past_commit_file_stays_above_its_diff_too`, `a_diff_with_a_long_line_scrolls_sideways`, `the_diff_of_a_past_commit_scrolls_sideways_too`, `another_file_opens_at_the_left_edge_of_its_diff`, `another_file_of_a_past_commit_opens_at_the_left_edge_too`.
- Copying text (story 3): `dragging_over_the_text_of_a_diff_selects_it_for_copying`, `a_drag_over_three_lines_copies_the_cut_of_each_line`, `a_drag_upwards_copies_the_same_text_as_a_drag_downwards`, `a_plain_click_on_a_diff_copies_nothing`, `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button`.
- Limits and files that cannot be shown (story 4): `a_long_file_is_cut_and_a_note_says_how_much_is_left`, `an_untracked_file_that_is_big_or_not_text_is_told_so_in_the_pane`, `a_picture_over_the_size_limit_is_refused_with_an_error`.
- Images (story 5): `an_image_shows_before_on_the_left_and_after_on_the_right`, `a_staged_image_shows_the_committed_picture_and_the_staged_one`, `an_image_in_a_past_commit_has_no_before_when_it_was_added`.
- Stacked diffs and the sticky head (stories 6 and 7): `picking_several_files_shows_all_their_diffs_one_after_another`, `a_past_commit_hides_the_stack_and_coming_back_shows_it_again`, `the_stack_is_limited_so_a_huge_pick_stays_light`, `an_image_among_the_picked_files_shows_a_note_in_the_stack`, `select_all_in_a_list_of_one_file_opens_it_without_a_stack`, `stacked_diffs_scroll_sideways_too`, `stacked_diffs_keep_the_name_of_the_file_at_the_top_until_the_next_file`, `the_sticky_file_head_stays_at_the_left_edge_while_the_stack_scrolls_sideways`.
- Past commit files (story 8): `the_files_of_a_commit_are_picked_with_shift_cmd_and_cmd_a_and_stacked`, `shift_with_the_arrow_keys_grows_the_picked_files_of_a_commit`, `the_stack_of_a_commit_has_its_lines_and_a_note_for_its_image`.

**Green:** They fail until the last step and pass when it is done; no code is written for them directly.
**Refactoring:** The two new files share small helpers that open a file, pick files and drag across lines.

### Step 1: Screen

**Red:** Write failing tests of rows, drawing and selectable text, which use a stub of the part below that hands over fixed diff lines, pictures and stacked rows: `each_kind_of_line_has_its_colors`, `a_diff_longer_than_the_limit_is_cut_with_a_note`, `a_diff_without_lines_says_so`, `the_head_of_the_file_at_the_top_sticks_until_the_next_head_pushes_it`, `only_a_file_head_has_a_name_and_a_badge`, `letters_outside_ascii_take_two_columns`, `the_widest_stacked_row_is_found_by_its_columns`, `copies_the_selected_part_of_each_line_across_lines`, `a_selection_made_backwards_reads_the_same`, `nothing_is_selected_when_both_ends_are_the_same`, `a_selection_that_cuts_a_letter_in_two_copies_nothing`, `select_all_takes_every_line_and_new_text_drops_the_selection`, `dragging_across_painted_text_selects_it_and_a_plain_click_selects_nothing`.
**Green:** Draw the rows under a fixed name row, let lines run past the right edge, draw their text as selectable text, draw the picture pair, and draw the stacked list that builds only the rows in view with the file head kept over it.
Reset the sideways position when another file opens.
The part below is stubbed, which gives fixed lines and pictures.
**Refactoring:** Share the row drawing between the working lists and the commit view, keep the end of a drag from counting as a click on the line, and keep the pure calculations of the stack view apart from the drawing.

### Step 2: Workspace

**Red:** Write failing tests that call the workspace and detail actions and read their state, with the diff rows and the git reads below stubbed to give fixed lines and fixed bytes: `an_untracked_file_that_is_big_or_not_text_is_told_so_instead_of_shown`, `a_staged_image_shows_the_committed_picture_before_and_the_staged_one_after`, `an_image_among_the_picked_files_gets_a_note_instead_of_lines`, `a_file_that_is_no_longer_changed_is_left_out_of_the_stack`, `a_stack_that_cannot_be_read_is_an_error`, `the_same_picked_files_are_not_read_again`, `picking_the_same_files_twice_and_a_single_file_list_are_handled`, `the_stack_of_a_commit_has_its_files_images_and_the_count_left_out`, `picks_cannot_move_through_files_that_are_not_there`.
**Green:** Read files in the background, cut long diffs and long stacks with notes, give images a note in the stack, keep a result only when the same file or picks are still shown, and change the picks of a commit with Shift, Cmd, Cmd+A and Shift with Up or Down.
This step replaces the stub that Step 1 used with the real reading and picking, and stubs the parts below it.
**Refactoring:** Skip a new read for the same picks, and share the stack rows between the working lists and the commits.

### Step 3: Model

**Red:** Write failing unit tests of diff rows, fed with fixed git diff lines in place of the git reads: `classifies_diff_lines`, `diff_rows_number_lines_and_drop_the_file_header`, `diff_rows_show_an_untracked_file_as_added_lines_and_keep_messages`.
**Green:** Turn git diff output into rows with hunk titles, old and new numbers and line kinds.
This step replaces the stub of rows that Step 2 used with the real Model.
**Refactoring:** Treat an untracked file as added lines.

### Step 4: Core

**Red:** Write failing tests of the git reads against real repositories, with no stub below this part: `reads_files_as_they_are_on_each_side`, `a_file_over_the_limit_is_refused`.
**Green:** Read the diff of a file or of a commit file, and the bytes of a picture from HEAD, the index, a commit, its parent or the folder, and refuse a file over 8 MB.
This step replaces the last stub, so no stub is left and the Step 0 tests pass.
**Refactoring:** none.

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

1. Open a changed text file with a long line and scroll sideways.
2. Open an image file and check Before and After.
3. Pick several files and scroll the stack down and sideways.
4. In a past commit, use Shift click, Cmd click, Cmd+A and Shift with Up or Down.

## Considerations

### Reusing Existing Code

The text diff rows are shared by the working lists and the commit view.
The stacked diffs are shared by the working lists and the commits.

### Compatibility

This feature is read only.
The line and hunk links on the same pane belong to the staging feature.
