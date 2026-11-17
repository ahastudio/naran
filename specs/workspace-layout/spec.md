# Feature Specification: Workspace Layout and Keyboard

## Overview

A one-window screen with three columns, dividers dragged to resize, keyboard movement, and the memory of all this state across runs.
It is dark only and targets macOS.

## User Scenarios & Testing (mandatory)

### User Story 1: Read on one screen

- As: a user who watches many repositories
- I: see groups, repositories and detail in one window
- So: the reading flow is not broken by screens that replace each other

#### Acceptance Scenarios

Scenario 1: **One window with three columns**

- Given: the app starts
- When: the window opens
- Then: one window shows the groups, the repositories and the detail, with a divider between them and one below the commit list
- Test: `the_three_columns_and_their_dividers_are_on_screen`, `the_app_opens_one_window_with_a_workspace_and_saves_where_it_is`

Scenario 2: **The selected repository is the window title**

- Given: a repository is selected
- When: another repository is selected
- Then: the window title and the toolbar title become the folder name of that repository
- Test: `the_window_title_is_the_selected_repository`

Scenario 3: **No selection shows the app name**

- Given: no repository is selected
- When: the user looks at the title
- Then: the title is "Naran"
- Test: `the_window_title_is_the_selected_repository`

Scenario 4: **Refresh by button and by key**

- Given: a repository has a new untracked file
- When: the user clicks Refresh and presses Cmd+R
- Then: the repositories are read again and the file is counted
- Test: `refresh_reads_again_from_the_button_and_from_the_keys`

Scenario 5: **Refresh waits while reading**

- Given: the repositories are being read
- When: the user clicks the faded Refresh button
- Then: the click starts no new read
- Test: `refresh_does_nothing_while_the_repositories_are_being_read`

Scenario 6: **Narrow commit list**

- Given: the side columns are very wide
- When: the commit list is left little room
- Then: its column heads stay on one line
- Test: `the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow`

Scenario 7: **Notes and unreadable rows**

- Given: a note, a problem text and a row that could not be read are drawn
- When: the window is drawn
- Then: each takes room in the window
- Test: `a_repository_that_cannot_be_read_still_takes_a_row_and_counts_as_a_problem`, `the_notes_and_a_row_that_could_not_be_read_take_room_in_the_window` (inner level)

Scenario 8: **Notice dismissed by a click**

- Given: an error notice is shown above the repository column
- When: the user clicks it
- Then: the notice goes away
- Test: `a_notice_goes_away_when_it_is_clicked`

Scenario 9: **Colors of changed files**

- Given: a changed file is listed
- When: its letter badge is drawn
- Then: each letter has its own color
- Test: `every_letter_of_a_changed_file_has_its_color`; inner level, not reachable through the window

### User Story 2: Resize with dividers

- As: a user
- I: drag a divider to resize the parts of the window
- So: give more room to what I am reading

#### Acceptance Scenarios

Scenario 1: **Dragging resizes the neighbor and letting go ends the drag**

- Given: the pointer is pressed on the groups divider
- When: the user moves it, lets go, and moves again
- Then: the groups column follows until the button is released, and later moves change nothing
- Test: `dragging_a_divider_with_the_mouse_resizes_it_and_letting_go_ends_the_drag`

Scenario 2: **The commit list divider moves up and down**

- Given: the pointer is pressed on the divider below the commit list
- When: the user moves it down
- Then: the commit list height follows
- Test: `dragging_a_divider_with_the_mouse_resizes_it_and_letting_go_ends_the_drag`, `dragging_a_divider_resizes_the_part_next_to_it`

Scenario 3: **Pressing a divider starts a drag**

- Given: the pointer is over a divider
- When: the user presses the button
- Then: that divider is the one being dragged
- Test: `pressing_a_divider_starts_dragging_it`, `a_notice_is_dismissed_and_a_divider_is_grabbed_and_let_go`

Scenario 4: **The repositories divider moves their column**

- Given: the groups and repositories divider is dragged to the edge of the repositories column
- When: the pointer moves
- Then: the repositories column keeps its width
- Test: `the_divider_next_to_the_repositories_moves_their_column`

Scenario 5: **Sizes stop at the ends of their range**

- Given: a divider is being dragged
- When: the pointer goes far beyond either end and then comes back
- Then: the size stops at the minimum or maximum and follows again on the way back
- Test: `a_divider_stops_at_both_ends_of_its_range`, `dragging_a_divider_resizes_the_part_next_to_it`

Scenario 6: **Every change is saved at once**

- Given: dividers and the arrow-key list are changed
- When: the layout file is read
- Then: it holds what is on screen
- Test: `the_layout_is_saved_as_it_changes`

Scenario 7: **A broken layout file cannot hide a part**

- Given: the layout file is missing, not JSON, or holds out-of-range numbers
- When: the app starts
- Then: defaults or in-range sizes are used and the dividers are on screen
- Test: `a_broken_layout_file_opens_the_window_with_every_part_in_range`, `a_broken_layout_file_cannot_hide_a_part`

### User Story 3: Start where I left off

- As: a user
- I: find the window and the selection as I left them
- So: not set them up again every time

#### Acceptance Scenarios

Scenario 1: **The window comes back exactly**

- Given: a window was saved windowed, maximized or full screen
- When: it is read back
- Then: position, size and mode are the same
- Test: `a_saved_window_comes_back_exactly`, `a_saved_window_opens_where_it_was`

Scenario 2: **Saved mode, or the default place**

- Given: a saved window exists, or none
- When: the app starts
- Then: it opens in the saved mode, or windowed in the default place
- Test: `a_saved_window_opens_in_its_mode_and_an_unsaved_one_in_the_default_place`; inner level, not reachable through the window

Scenario 3: **An older file without a mode**

- Given: a window file has no mode
- When: it is read
- Then: the window is a normal window
- Test: `a_file_without_a_mode_is_a_normal_window`; inner level, not reachable through the window

Scenario 4: **A vanished screen only moves the position**

- Given: the saved position is on no screen
- When: the app starts
- Then: the size is kept and the position moves onto the first screen, and a window bigger than the screen goes to its corner
- Test: `the_size_never_changes_only_the_position_moves_onto_a_screen`; inner level, not reachable through the window

Scenario 5: **Saved on every resize**

- Given: the window is resized
- When: the window file is read
- Then: it holds the new size
- Test: `the_app_opens_one_window_with_a_workspace_and_saves_where_it_is`

Scenario 6: **Saved on close, and the last window quits**

- Given: the window is closed
- When: the window file is checked and the last window is removed
- Then: the file exists and the app has no windows left
- Test: `the_window_is_saved_once_more_when_it_closes_and_closing_the_last_one_quits`; drives the real window in `app/src/tests.rs`

Scenario 7: **Closing one of two windows does not quit**

- Given: two windows are open
- When: one is closed
- Then: one window remains
- Test: `closing_one_of_two_windows_does_not_quit`; drives the real window in `app/src/tests.rs`

Scenario 8: **The app quits when its window cannot open**

- Given: the window failed to open
- When: the app checks the result
- Then: it quits, and it does not when the window opened
- Test: `the_app_quits_when_its_window_could_not_open`; drives the real window in `app/src/tests.rs`

Scenario 9: **Layout, list and selection come back**

- Given: the user selected the second repository, dragged a divider and clicked the commit list
- When: the app is started again on the same folder
- Then: the same repository, divider size and arrow-key list are used
- Test: `the_layout_and_the_selection_come_back_after_a_restart`

Scenario 10: **A saved group that is gone is forgotten**

- Given: the saved selection names a group that no longer exists
- When: the app starts
- Then: the first group is selected and the saved repository stays selected
- Test: `forgets_a_saved_group_that_no_longer_exists`

Scenario 11: **A save that fails is reported**

- Given: the selection file or the repository list file cannot be written
- When: the app saves
- Then: the status bar says it could not save
- Test: `a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice`

Scenario 12: **Missing and broken files, and the folder**

- Given: a saved file is missing or not JSON, or cannot be written
- When: it is read or written
- Then: a missing or broken file gives the default, an unwritable one is an error, and the folder is `dev.naran.app` in the config or temporary folder
- Test: `round_trips_and_falls_back_to_default`, `the_folder_is_inside_the_config_folder_or_the_temporary_one`, `a_value_that_is_not_json_cannot_be_saved`, `a_file_inside_something_that_is_not_a_folder_cannot_be_saved`; inner level, not reachable through the window

### User Story 4: Read with the keyboard

- As: a user
- I: move through the lists with keys
- So: skim changes without the mouse

#### Acceptance Scenarios

Scenario 1: **Keys move the list clicked last, and the repository list wraps**

- Given: the second repository row was clicked
- When: the user presses Down, Down, Up
- Then: the selection goes to the third, wraps to the first, then back to the third
- Test: `the_arrow_keys_move_the_repository_list_and_wrap_around_its_ends`

Scenario 2: **Clicking the group column moves the keys there**

- Given: two groups exist
- When: the second group is clicked
- Then: the arrow keys now belong to the groups and its repositories show
- Test: `pressing_the_group_column_and_a_group_moves_the_keys_there`

Scenario 3: **Arrow keys through the groups**

- Given: two groups exist and the group list is the key list
- When: the user presses Down
- Then: the next group is selected
- Test: `the_arrow_keys_move_through_the_groups`

Scenario 4: **Files and commits**

- Given: the file list, then the commit list, is the key list
- When: the user presses Down and Up
- Then: the open file moves one row, and the commit pick moves from Uncommitted changes to the top commit and back
- Test: `arrow_keys_move_through_the_list_that_was_clicked_last`

Scenario 5: **One step stays inside a list**

- Given: a list of any length, with or without a current row
- When: one step is asked for from either end
- Then: the row stays inside the list, and an empty list gives none
- Test: `stepping_stays_inside_the_list`; inner level, not reachable through the window

Scenario 6: **Empty lists**

- Given: a group without repositories, or no groups
- When: the user presses Down
- Then: nothing is selected
- Test: `the_arrow_keys_do_nothing_in_a_list_that_is_empty`

Scenario 7: **Shift arrows outside the file lists**

- Given: the repository list is the key list
- When: the user presses Shift+Down and Shift+Up
- Then: nothing is picked and the selection stays
- Test: `shift_arrows_outside_the_file_lists_pick_nothing`

Scenario 8: **The lists scroll to follow the pick**

- Given: a long commit list or file list is the key list
- When: the user steps down 40 or 60 rows, and back
- Then: the list scrolls down to keep the row in view, and back to the top
- Test: `the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row`, `the_arrow_keys_scroll_the_file_lists_too`

Scenario 9: **Cmd+A picks where it was pressed**

- Given: a file list was clicked, then the diff text
- When: the user presses Cmd+A each time
- Then: every file of the list is picked, and after the diff only its text
- Test: `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button`

Scenario 10: **Cmd+O opens the picker once**

- Given: the app is open
- When: the user presses Cmd+O while a picker is already open
- Then: a closed picker is no error, no second picker opens, and Cancel stops the open one
- Test: `adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it`

Scenario 11: **Copying**

- Given: text was selected, then other text was set without selecting
- When: the user copies
- Then: the selected text is on the clipboard and the clipboard is left alone otherwise
- Test: `copying_puts_the_selected_text_on_the_clipboard_and_nothing_else_when_none_is_selected`, `copy_puts_the_picked_text_on_the_clipboard`

Scenario 12: **The key target**

- Given: the window is open
- When: the window asks for its focus
- Then: it gets the workspace focus handle
- Test: `the_window_asks_the_workspace_for_its_focus_handle`

## Functional Requirements (mandatory)

- FR-1: MUST show groups, repositories and detail as three columns in one window (US1 S1).
- FR-2: MUST show the folder name of the selected repository as the window title, or "Naran" without a selection (US1 S2, S3).
- FR-3: MUST let the user drag the dividers between the columns and below the commit list to resize (US2 S1-S4).
- FR-4: MUST keep every size inside an allowed range (US2 S5, S7).
- FR-5: MUST save and restore the divider sizes, the arrow-key list and the refs flags (US2 S6, US3 S9).
- FR-6: MUST save and restore the window position, size, and windowed, maximized or full screen state (US3 S1-S6).
- FR-7: MUST keep the size and move only the position onto a screen when the saved window is on none (US3 S4).
- FR-8: MUST move the list clicked last by one row with Up and Down, keep the row in view, and wrap around at the ends of the repository list (US4 S1-S8).
- FR-9: MUST grow the picked files of the file lists with Shift+Up and Shift+Down.
- FR-10: MUST provide Cmd+A, Cmd+R and Cmd+O (US4 S9, US1 S4, US4 S10).
- FR-11: MUST save the repository list, group settings, selection, layout and window state as JSON files in the config folder `dev.naran.app` (US3 S12).
- FR-12: MUST report a failed save in the status bar and as a notice above the repository column (US3 S11).
- FR-13: MUST NOT start a new read when Refresh is clicked while reading (US1 S5).
- FR-14: SHOULD use the shared buttons, badges and titles across the columns (US1 S7, S9).

## Constraints (mandatory)

- CON-1: MUST be dark only.
- CON-2: MUST keep one window; selecting changes only a column and never replaces the whole screen.
- CON-3: MUST start with defaults when a saved file is missing or broken (US2 S7, US3 S12).
- CON-4: MUST NOT leave a broken file behind when a save is interrupted.
- CON-5: MUST keep the layout files in the config folder `dev.naran.app`, next to the repository list and the groups.
- CON-6: MUST NOT commit, fetch, pull or push.

## Success Criteria (mandatory)

- SC-1: After a restart, the window, dividers, selection and arrow-key list are as they were at quit (US3 S1, S9).
- SC-2: Breaking any saved file still lets the app open with every part visible (US2 S7).
- SC-3: The lists can be read with Up, Down, Shift and Cmd keys without the mouse (US4).
- SC-4: The window title alone tells which repository is shown (US1 S2).
