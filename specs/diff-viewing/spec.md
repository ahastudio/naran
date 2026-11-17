# Feature Specification: Viewing Changes

## Overview

A read-only view of the changes of the picked files.
It covers the text diff, Before and After pictures of images, stacked diffs of several files, and the diff of files of a past commit.
Staging of lines and hunks is out of scope.
Nothing here is kept across restarts; what is shown always comes from the repository.

## User Scenarios & Testing (mandatory)

### User Story 1: Read the text diff of a file

- As: a developer reviewing changes
- I: can read the diff of the open file with line numbers
- So: I understand exactly what changed

#### Acceptance Scenarios

Scenario 1: **Modified file follows staging**

- Given: a modified file is open from the Unstaged list
- When: it is staged, then unstaged, then committed
- Then: the diff shows the added line, follows the file between the lists, and closes once the file is no longer changed
- Test: `shows_the_diff_of_a_file_and_follows_it_through_staging`

Scenario 2: **Rows with numbers and hunk titles**

- Given: git diff output with a header, context, removed and added lines, a "No newline" marker and two hunks
- When: it is turned into rows
- Then: the file header is dropped, each hunk has a title, and each line has old and new numbers
- Test: `a_modified_file_shows_its_lines_without_the_git_header_or_markers`, `diff_rows_number_lines_and_drop_the_file_header`

Scenario 3: **Line kinds and colors**

- Given: lines starting with `+`, `-`, a space, `@@` or file header markers
- When: they are classified and colored
- Then: added, removed, hunk, meta and context lines get their own colors, and tabs show as four spaces
- Test: `classifies_diff_lines`, `each_kind_of_line_has_its_colors`, inner level, not reachable through the window

Scenario 4: **Untracked file shown whole**

- Given: an untracked file with a single line
- When: it is opened
- Then: the diff is that line as an added line
- Test: `shows_an_untracked_file_whole`, `diff_rows_show_an_untracked_file_as_added_lines_and_keep_messages`

Scenario 5: **Nothing to show**

- Given: the diff of the open file has no lines
- When: the pane is drawn
- Then: no diff lines are drawn
- Test: `a_diff_without_lines_says_so`, inner level, not reachable through the window

### User Story 2: Keep reading long files and long lines

- As: a developer reading a big file
- I: can scroll down and sideways without losing the file name
- So: I always know which file I am reading

#### Acceptance Scenarios

Scenario 1: **Name stays while scrolling down**

- Given: a diff taller than the pane
- When: I scroll down
- Then: the file name row does not move and the lines move up under it
- Test: `the_file_name_stays_above_the_diff_while_it_scrolls`

Scenario 2: **Same for a past commit file**

- Given: a tall file of a past commit
- When: I scroll down
- Then: its name stays too
- Test: `the_name_of_a_past_commit_file_stays_above_its_diff_too`

Scenario 3: **Long line scrolls sideways**

- Given: a line much wider than the pane
- When: I swipe sideways
- Then: the diff moves by the swipe; with only short lines it does not move
- Test: `a_diff_with_a_long_line_scrolls_sideways`

Scenario 4: **Past commit scrolls sideways**

- Given: a past commit file with a long line
- When: I swipe sideways
- Then: the diff moves by the swipe
- Test: `the_diff_of_a_past_commit_scrolls_sideways_too`

Scenario 5: **Another file starts at the left edge**

- Given: the diff is scrolled sideways
- When: I open another file
- Then: the new diff starts at the left edge
- Test: `another_file_opens_at_the_left_edge_of_its_diff`

Scenario 6: **Another commit file starts at the left edge**

- Given: a past commit file is scrolled sideways
- When: I open another file of the commit
- Then: the new diff starts at the left edge
- Test: `another_file_of_a_past_commit_opens_at_the_left_edge_too`

### User Story 3: Select and copy diff text

- As: a developer
- I: can drag over the text of a diff and copy it
- So: I can paste code into a note or a chat

#### Acceptance Scenarios

Scenario 1: **Drag across lines**

- Given: a diff is shown
- When: I press on one line and drag to a later line
- Then: the text between the two points is selected for copying
- Test: `dragging_over_the_text_of_a_diff_selects_it_for_copying`

Scenario 2: **Copy spans lines**

- Given: three lines of text
- When: a selection runs from the middle of the first to the end of the third
- Then: the copy has the cut of each line, joined by line breaks
- Test: `a_drag_over_three_lines_copies_the_cut_of_each_line`, `copies_the_selected_part_of_each_line_across_lines`

Scenario 3: **Selection backwards, empty and split letters**

- Given: a selection made backwards, with both ends equal, or cutting a letter in two
- When: it is copied
- Then: a backwards selection reads the same, and an empty or letter-splitting selection copies nothing
- Test: `a_drag_upwards_copies_the_same_text_as_a_drag_downwards`, `nothing_is_selected_when_both_ends_are_the_same`, `a_selection_that_cuts_a_letter_in_two_copies_nothing`, inner level, not reachable through the window

Scenario 4: **Plain click selects nothing**

- Given: painted text
- When: I click without dragging
- Then: nothing is selected
- Test: `a_plain_click_on_a_diff_copies_nothing`, `dragging_across_painted_text_selects_it_and_a_plain_click_selects_nothing`

Scenario 5: **Select all and clear**

- Given: a selection region with lines
- When: I select all, then new text arrives
- Then: every line is selected, and new text drops the selection
- Test: `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button`, `select_all_takes_every_line_and_new_text_drops_the_selection`

### User Story 4: Files that are too big or not text

- As: a developer
- I: am told when a file cannot be shown, and a huge diff does not freeze the window
- So: I know why there is nothing to read

#### Acceptance Scenarios

Scenario 1: **Diff longer than the limit**

- Given: a file with more lines than the limit
- When: its diff is shown
- Then: it is cut and a note says how many lines are left
- Test: `a_long_file_is_cut_and_a_note_says_how_much_is_left`, `a_diff_longer_than_the_limit_is_cut_with_a_note`

Scenario 2: **Untracked file too big or not text**

- Given: an untracked file over the size limit, and another that is not text
- When: they are read for the diff
- Then: the first says it is too large to show and the second says it is not text
- Test: `an_untracked_file_that_is_big_or_not_text_is_told_so_in_the_pane`, `an_untracked_file_that_is_big_or_not_text_is_told_so_instead_of_shown`

Scenario 3: **Picture over the size limit**

- Given: a file over the picture size limit
- When: it is read as a picture
- Then: it is refused with an error that names the file
- Test: `a_picture_over_the_size_limit_is_refused_with_an_error`, `a_file_over_the_limit_is_refused`

Scenario 4: **Picture reads from each side**

- Given: a picture in HEAD, the index, a commit and its parent, or missing
- When: each side is read
- Then: the bytes are returned, a missing file gives nothing, and a bad hash is refused
- Test: `reads_files_as_they_are_on_each_side`, inner level, not reachable through the window

### User Story 5: See an image change

- As: a developer reviewing an image change
- I: can see the old and the new picture side by side
- So: I can compare them by eye

#### Acceptance Scenarios

Scenario 1: **Changed, new and deleted image**

- Given: a changed, a new (untracked) and a deleted image
- When: each is opened
- Then: the changed one has both pictures, the new one only After, the deleted one only Before, and no text diff is read
- Test: `an_image_shows_before_on_the_left_and_after_on_the_right`

Scenario 2: **Staged image**

- Given: an image committed, then changed and staged
- When: it is opened from the Staged list
- Then: Before is the committed picture and After is the staged one
- Test: `a_staged_image_shows_the_committed_picture_and_the_staged_one`, `a_staged_image_shows_the_committed_picture_before_and_the_staged_one_after`

Scenario 3: **Image added in a past commit**

- Given: a commit that added an image
- When: the commit is picked
- Then: its image has no Before and has an After
- Test: `an_image_in_a_past_commit_has_no_before_when_it_was_added`

### User Story 6: Skim the diffs of several files

- As: a developer reading a change that touches many files
- I: can see the diffs of the picked files in one view
- So: I do not have to open them one by one

#### Acceptance Scenarios

Scenario 1: **Two or more picked, then one**

- Given: four changed files
- When: I pick one, then a range up to a third file, then one file again
- Then: one file has no stack, three files show a head with its badge and the diff lines of each, and picking one file removes the stack
- Test: `picking_several_files_shows_all_their_diffs_one_after_another`

Scenario 2: **Staging clears the stack**

- Given: several files are stacked
- When: the picked files are staged
- Then: the stack goes away
- Test: `picking_several_files_shows_all_their_diffs_one_after_another`

Scenario 3: **Past commit hides the stack**

- Given: several files are stacked
- When: I pick a past commit and then come back to the uncommitted changes
- Then: the commit shows its own diff, and the stack returns
- Test: `a_past_commit_hides_the_stack_and_coming_back_shows_it_again`

Scenario 4: **File and line limits**

- Given: 45 files of 600 lines each are picked
- When: the stack is shown
- Then: 40 files are read, long files are cut with a note, and a last note says 5 more files are not shown
- Test: `the_stack_is_limited_so_a_huge_pick_stays_light`

Scenario 5: **Image among the files**

- Given: a text file and an image are picked
- When: the stack is shown
- Then: the image has a note instead of lines
- Test: `an_image_among_the_picked_files_shows_a_note_in_the_stack`, `an_image_among_the_picked_files_gets_a_note_instead_of_lines`

Scenario 6: **Read failure**

- Given: the repository cannot be read
- When: the stack is read
- Then: it is an error, not rows
- Test: `a_stack_that_cannot_be_read_is_an_error`, inner level, not reachable through the window

Scenario 7: **File no longer changed**

- Given: a picked path that is no longer in the changes
- When: the stack is built
- Then: that file is left out
- Test: `a_file_that_is_no_longer_changed_is_left_out_of_the_stack`, inner level, not reachable through the window

Scenario 8: **Same picks are not read again**

- Given: the same files are picked again
- When: the stack is refreshed
- Then: it stays shown without a new read
- Test: `the_same_picked_files_are_not_read_again`, inner level, not reachable through the window

Scenario 9: **Stack scrolls sideways**

- Given: a stack with a long line
- When: I swipe sideways
- Then: the stack moves by the swipe
- Test: `stacked_diffs_scroll_sideways_too`

Scenario 10: **Single-file list and ghost files**

- Given: a list with one file, or picks that include a file that is gone
- When: select all, stepping and extending are used
- Then: the view handles them without a stack or a crash
- Test: `select_all_in_a_list_of_one_file_opens_it_without_a_stack`, `picking_the_same_files_twice_and_a_single_file_list_are_handled`

### User Story 7: Keep the file head in sight in the stack

- As: a developer scrolling through the stack
- I: can always see which file the lines belong to
- So: I do not lose my place

#### Acceptance Scenarios

Scenario 1: **Head sticks, then is pushed out**

- Given: two stacked files that are scrolled down
- When: the second file's head comes in
- Then: the first head stays at the top, then is pushed out by the second
- Test: `stacked_diffs_keep_the_name_of_the_file_at_the_top_until_the_next_file`

Scenario 2: **Where the head sticks**

- Given: rows with two heads and scroll positions at the top, mid file, mid push, after the push, past the end and with no head above
- When: the sticky head is looked up
- Then: none at the top, the first head, the first head pushed up, the second head, the last head, and none without a head
- Test: `the_head_of_the_file_at_the_top_sticks_until_the_next_head_pushes_it`, inner level, not reachable through the window

Scenario 3: **Head stays at the left edge**

- Given: a stack scrolled down with a sticky head, then swiped sideways
- When: the list moves sideways
- Then: the sticky head keeps its position at the left edge of the list
- Test: `the_sticky_file_head_stays_at_the_left_edge_while_the_stack_scrolls_sideways`

Scenario 4: **Widest row sets the width**

- Given: rows with tabs, Korean letters and long paths
- When: the widest row is found
- Then: tabs count as four spaces and non-ASCII letters count as two columns
- Test: `letters_outside_ascii_take_two_columns`, `the_widest_stacked_row_is_found_by_its_columns`, inner level, not reachable through the window

### User Story 8: Read the files of a past commit

- As: a developer going back over a past change
- I: can pick one or several files of a commit and see their diffs
- So: I can read what the commit did

#### Acceptance Scenarios

Scenario 1: **Click, range, toggle, all**

- Given: a commit with four files
- When: I click a file, Shift click another, Cmd click two files, and press Cmd+A
- Then: the picks are one file, the range, the file removed or added, and all four, with the stack following the picks
- Test: `the_files_of_a_commit_are_picked_with_shift_cmd_and_cmd_a_and_stacked`

Scenario 2: **Shift with Up and Down**

- Given: a commit with three files and the first picked
- When: I press Shift+Down twice, then Shift+Up
- Then: the picks grow to three, then shrink to two
- Test: `shift_with_the_arrow_keys_grows_the_picked_files_of_a_commit`

Scenario 3: **Stack of a commit**

- Given: two files of a commit, one an image, and three more files left out
- When: the commit stack is read
- Then: it has the added line, the image note and a last note about three more files, and an unreadable repository is an error
- Test: `the_stack_of_a_commit_has_its_lines_and_a_note_for_its_image`, `the_stack_of_a_commit_has_its_files_images_and_the_count_left_out`

Scenario 4: **Picks without files**

- Given: files that failed to read, a commit without files, or no repository
- When: the picks are moved or extended
- Then: nothing moves and no stack is shown
- Test: `picks_cannot_move_through_files_that_are_not_there`, inner level, not reachable through the window

## Functional Requirements (mandatory)

- FR-1: MUST show the diff of working files and of files of a past commit in the same form (US1 S1-S2, US8).
- FR-2: MUST let long lines scroll sideways without cutting them and keep the file name row in sight (US2).
- FR-3: MUST start a newly opened file at the left edge (US2 S5-S6).
- FR-4: MUST let the text of a diff be selected and copied (US3).
- FR-5: MUST cut a long diff and say how many lines are left out (US4 S1).
- FR-6: MUST show an untracked file as added lines, and say so when it is too big or not text (US1 S4, US4 S2).
- FR-7: MUST show image files as Before (left) and After (right), leaving a missing side empty (US5).
- FR-8: MUST stack the diffs of the picked files when two or more are picked (US6 S1).
- FR-9: MUST read at most 40 files and 500 lines per file for the stack, and note what is left out (US6 S4).
- FR-10: MUST show only a note for an image in the stack (US6 S5).
- FR-11: MUST keep the head of the top file in sight, and at the left edge when scrolling sideways (US7).
- FR-12: MUST let several files of a commit be picked with Shift click, Cmd click, Cmd+A and Shift+Up/Down, and stack them (US8).
- FR-13: MUST tell reading, no changes and a failed read apart (US1 S5, US6 S6).
- FR-14: SHOULD build only the rows in view for the stack so a large pick stays light (US6 S4).

## Constraints (mandatory)

- CON-1: MUST be read only; no commit, fetch, pull or push.
- CON-2: MUST NOT read a file over 8 MB as a picture; it is reported as an error (US4 S3).
- CON-3: MUST NOT freeze the window while files are read.
- CON-4: MUST drop a read result that arrives after another file or selection is shown.
- CON-5: MUST NOT include staging of lines and hunks.

## Success Criteria (mandatory)

- SC-1: Opening a changed text file shows a colored diff with line numbers (US1).
- SC-2: The file name stays visible while a long line is scrolled sideways (US2).
- SC-3: An image file shows Before and After side by side (US5).
- SC-4: Picking 45 files shows 40 and a note about the other 5 (US6 S4).
- SC-5: Cmd+A in the files of a past commit stacks the diffs of every file (US8 S1).
- SC-6: The sticky file head stays at the left edge while the stack scrolls sideways (US7 S3).
