# Feature Specification: Groups and Repositories

## Overview

The user registers repositories in groups and manages them.
The group column and the repository column summarize counts and state, and the last selection is restored on the next launch.
Repository folders are only read, never deleted or changed.

## User Scenarios & Testing (mandatory)

### User Story 1: Create groups

- As: a user with many repositories
- I: create a groups by name
- So: organize repositories by purpose

#### Acceptance Scenarios

Scenario 1: **Create and select**

- Given: the group column is visible
- When: the user clicks `+ New Group`, types a name and presses Enter
- Then: the group is added after the existing ones and becomes selected
- Test: `groups_and_repositories_acceptance::clicking_new_group_and_typing_a_name_creates_and_selects_the_group`

Scenario 2: **Name in Korean**

- Given: the name input is open
- When: the user types Korean text and presses Enter
- Then: the group is created with exactly that name and the input closes
- Test: `creates_and_renames_a_group_by_typing`

Scenario 3: **Name is trimmed**

- Given: a name has spaces around it
- When: the group is created
- Then: it is saved without the surrounding spaces
- Test: `a_group_name_is_saved_without_the_spaces_around_it`, `creates_groups_with_new_non_empty_names_only`

Scenario 4: **Name already taken**

- Given: a group named `work` exists
- When: the user creates a group named `work`
- Then: no group is added and a red notice says the group already exists; one click on the notice dismisses it
- Test: `groups_and_repositories_acceptance::a_taken_name_shows_a_red_notice_that_a_click_dismisses`, `refuses_a_name_that_is_already_taken`

Scenario 5: **Blank name**

- Given: the name input is open
- When: the user types only spaces and presses Enter
- Then: no group is added and no notice appears
- Test: `groups_and_repositories_acceptance::an_empty_name_changes_nothing_and_shows_no_notice`, `creates_groups_with_new_non_empty_names_only`

Scenario 6: **Escape cancels**

- Given: the user has typed a name
- When: the user presses Escape
- Then: the input closes and no group is added
- Test: `creates_and_renames_a_group_by_typing`

### User Story 2: Rename, reorder and delete groups

- As: a user
- I: rename, reorder and delete groups
- So: keep the group list in order

#### Acceptance Scenarios

Scenario 1: **Rename by typing**

- Given: a group exists
- When: the user opens its name, selects all, types a new name and presses Enter
- Then: the group has the new name
- Test: `creates_and_renames_a_group_by_typing`

Scenario 2: **Rename from the menu**

- Given: a group row
- When: the user right-clicks it and chooses Rename
- Then: the name of that group is being edited
- Test: `the_menu_item_rename_starts_editing_the_group_name`

Scenario 3: **Rename refused**

- Given: another group has the new name, or the new name is blank, or the group does not exist
- When: the rename is applied
- Then: nothing changes
- Test: `renaming_from_the_menu_to_a_taken_or_blank_name_changes_nothing`, `renames_unless_the_name_is_empty_or_taken`

Scenario 4: **Delete asks first**

- Given: a group row
- When: the user right-clicks it, chooses Delete and answers Cancel
- Then: no group is deleted
- Test: `the_menu_item_delete_opens_the_confirmation`, `deleting_a_group_asks_first_and_cancel_changes_nothing`

Scenario 5: **Delete a group that is not selected**

- Given: two groups, the first selected
- When: the user deletes the second and confirms
- Then: the first stays selected
- Test: `deleting_a_group_asks_first_and_cancel_changes_nothing`

Scenario 6: **Delete the selected group**

- Given: two groups share one repository and each holds another
- When: the selected group is deleted and confirmed
- Then: the other group becomes selected, and only repositories no group holds leave the saved list
- Test: `deleting_a_group_drops_the_repositories_only_it_held`

Scenario 7: **Reorder by mouse drag**

- Given: the group column holds three groups
- When: the user drags the third group row onto the first
- Then: it takes the first slot and the other two follow
- Test: `group_order_acceptance::dragging_a_group_row_with_the_mouse_reorders_the_groups`

Scenario 8: **Order is saved and the selection stays**

- Given: a group and one of its repositories are selected
- When: a group is dropped on another group, then on itself, then on a name that is not a group
- Then: the saved groups have the new order after the first drop only, and the selected group and repository do not change
- Test: `dropping_a_group_on_another_group_changes_the_order_and_keeps_it`, `moving_a_group_takes_the_slot_of_the_one_it_is_dropped_on`

### User Story 3: Add repositories

- As: a user
- I: add folders with the button, the folder picker or a drop
- So: register repositories quickly

#### Acceptance Scenarios

Scenario 1: **Picked folders are added**

- Given: a group is selected and the picker returns a repository
- When: the answer arrives
- Then: the repository joins the group, the waiting notice is gone
- Test: `the_folder_picker_answers_are_handled_without_waiting_for_it`

Scenario 2: **Picker closed**

- Given: the picker is open
- When: the user closes it
- Then: nothing is added and no error is shown
- Test: `the_folder_picker_answers_are_handled_without_waiting_for_it`

Scenario 3: **Picker fails**

- Given: the picker cannot show
- When: it reports a failure
- Then: a red notice gives the reason and points to dropping folders
- Test: `the_folder_picker_answers_are_handled_without_waiting_for_it`

Scenario 4: **Cmd+O and Cancel**

- Given: a group is selected
- When: the user presses Cmd+O, and later presses Cancel on the waiting banner while a picker is open
- Then: a picker that is closed at once leaves no error; Cancel stops the open picker; a second picker is not opened meanwhile
- Test: `adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it`

Scenario 5: **Repository and plain folder together**

- Given: a group is selected
- When: a repository and a folder that is not a repository are added
- Then: only the repository joins the group and is selected, the status says `Added 1, failed 1`, and the notice lists the plain folder
- Test: `adds_folders_and_reports_the_ones_that_are_not_repositories`

Scenario 6: **Only a plain folder**

- Given: a group is selected
- When: only a non-Git folder is added
- Then: the group is unchanged and a red notice is on screen
- Test: `groups_and_repositories_acceptance::a_folder_that_is_not_a_repository_is_listed_in_a_notice_on_screen`

Scenario 7: **No group**

- Given: no group exists
- When: a folder is added
- Then: nothing is added and a red notice says to create a group first
- Test: `groups_and_repositories_acceptance::without_a_group_adding_a_folder_asks_to_create_one_first`, `without_groups_nothing_is_listed_and_nothing_can_be_added`

Scenario 8: **Same repository in two groups**

- Given: a repository is in one group
- When: it is added to a second group
- Then: it shows in both and is not read again
- Test: `puts_repositories_into_a_group_and_takes_them_out`

Scenario 9: **Added twice**

- Given: a repository is already in a group
- When: it is added to that group again
- Then: the group still holds it once
- Test: `picking_a_repository_the_group_already_holds_keeps_it_once`, `adds_and_removes_repositories`

Scenario 10: **Dropped on the window**

- Given: a group is selected
- When: a folder drop ends on the window
- Then: the drop is handled as an add and the result is reported in the status bar
- Test: `files_dropped_on_the_window_are_added_to_the_group`

### User Story 4: Tidy a group

- As: a user
- I: reorder, reveal and remove repositories
- So: keep the list the way I want it

#### Acceptance Scenarios

Scenario 1: **Reorder by mouse drag**

- Given: a group holds three repositories
- When: the user drags the third row onto the first
- Then: it takes the first slot
- Test: `dragging_a_repository_row_with_the_mouse_reorders_the_group`

Scenario 2: **Order is saved**

- Given: a repository is dropped on another row
- When: the move is applied
- Then: the saved group has the new order; a drop on itself changes nothing
- Test: `dropping_a_repository_on_another_row_changes_the_order_and_keeps_it`, `moving_a_repository_takes_the_slot_of_the_one_it_is_dropped_on`

Scenario 3: **Remove asks first**

- Given: a repository row
- When: the user chooses Remove and answers Cancel, then Remove
- Then: Cancel keeps everything; Remove takes it out and the folder still exists
- Test: `removing_a_repository_asks_first_and_cancel_changes_nothing`, `the_menu_item_remove_opens_the_confirmation`

Scenario 4: **Removed from one of two groups**

- Given: the repository is also in another group
- When: it is removed from one group
- Then: it stays in the saved list, and the selection moves to the next row
- Test: `puts_repositories_into_a_group_and_takes_them_out`

Scenario 5: **Removed from its last group**

- Given: the repository is in one group
- When: it is removed
- Then: it leaves the saved list and its folder is untouched
- Test: `drops_a_repository_that_leaves_its_last_group`

Scenario 6: **Reveal in Finder**

- Given: a repository row
- When: the user right-clicks it and chooses Reveal in Finder
- Then: the folder is shown, the menu closes and nothing is asked
- Test: `reveal_in_finder_closes_the_menu_and_asks_nothing`

Scenario 7: **No selected group**

- Given: no group is selected
- When: move, remove or a name edit is attempted
- Then: nothing happens and no confirmation opens
- Test: `without_a_selected_group_moving_removing_and_editing_do_nothing`

### User Story 5: Restore the last selection

- As: a user
- I: find my last group and repository selected on the next launch
- So: continue where I stopped

#### Acceptance Scenarios

Scenario 1: **Selection is saved**

- Given: two groups exist
- When: the user selects the second group
- Then: the saved selection holds that group and its first repository
- Test: `selecting_a_group_shows_only_its_repositories_and_keeps_the_choice`

Scenario 2: **A new window restores it**

- Given: a group and a second repository were selected
- When: a new window opens on the same config
- Then: the same group and repository are selected
- Test: `groups_and_repositories_acceptance::a_new_window_restores_the_last_group_and_repository`

Scenario 3: **A new window restores an empty group**

- Given: a group with no repositories was selected last
- When: a new window opens on the same config
- Then: that empty group is selected
- Test: `groups_and_repositories_acceptance::a_new_window_restores_an_empty_group`

Scenario 4: **Saved group is gone**

- Given: the saved selection names a group that no longer exists
- When: the app opens
- Then: the first group is selected and the saved repository is kept
- Test: `forgets_a_saved_group_that_no_longer_exists`

Scenario 5: **Nothing saved**

- Given: no selection was saved
- When: the app opens
- Then: every repository is read and the first one is selected
- Test: `reads_every_repository_and_selects_the_first`

Scenario 6: **Saving fails**

- Given: the selection file cannot be written
- When: the selection is saved
- Then: a message says the selection could not be saved
- Test: `a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice`

Scenario 7: **Missing or broken file**

- Given: a saved file is missing or not valid JSON
- When: it is loaded
- Then: the default value is used and the app goes on
- Test: `broken_saved_files_start_the_app_with_defaults`, `round_trips_and_falls_back_to_default`

Scenario 8: **Switching to a group that holds the same repository**

- Given: the selected repository is in two groups
- When: the other group is selected and a new window opens on the same config
- Then: the other group is selected and the repository stays selected
- Test: `groups_and_repositories_acceptance::switching_to_a_group_that_holds_the_same_repository_is_remembered`

### User Story 6: Read the two columns

- As: a user
- I: see counts and a one-word state for each group and repository
- So: judge where to look first

#### Acceptance Scenarios

Scenario 1: **Group shows only its repositories**

- Given: two groups
- When: the user selects one
- Then: only its repositories are listed, in its order
- Test: `selecting_a_group_shows_only_its_repositories_and_keeps_the_choice`, `lists_members_in_group_order_and_skips_unknown_paths`

Scenario 2: **Counts and problems**

- Given: repositories with staged, unstaged and untracked files, a conflict, or a read error
- When: the counts are taken
- Then: changes add up, and a conflict or an unreadable repository is a problem
- Test: `counts_changes_and_spots_problems`, inner level, not reachable through the window

Scenario 3: **One-word state**

- Given: a repository in any of the states
- When: its state is described
- Then: the most urgent word wins: Merging or Rebasing and the like, Conflicts, Changes, Diverged, Ahead, Behind, No upstream, Clean; an unreadable one shows its error
- Test: `describes_the_most_urgent_state_first`, inner level, not reachable through the window

Scenario 4: **Click selects a repository**

- Given: the repository column is shown
- When: the user clicks a row
- Then: that repository is selected
- Test: `clicking_a_repository_row_selects_it`

Scenario 5: **Arrow keys through groups**

- Given: the group pane has focus
- When: the user steps down
- Then: the next group is selected
- Test: `the_arrow_keys_move_through_the_groups`

## Functional Requirements (mandatory)

- FR-1: MUST create, rename and delete groups.
- FR-2: MUST refuse an empty group name and a name already taken.
- FR-3: MUST ask before deleting a group and before removing a repository from a group.
- FR-4: MUST allow a repository in several groups.
- FR-5: MUST drop a repository from the list when no group holds it.
- FR-6: MUST add repositories with `+ Add Repository`, Cmd+O, or by dropping folders on the window.
- FR-7: MUST refuse folders that are not Git repositories and say so in a red notice.
- FR-8: MUST reorder groups by drag in the group column and repositories by drag inside a group, and keep both orders.
- FR-9: MUST offer Reveal in Finder and Remove on a repository, and Rename and Delete on a group.
- FR-10: MUST show the repository count and the problem count on a group row, and the group name and count in the repository column header.
- FR-11: MUST show name, one-word state, branch and non-zero counts on a repository row; a repository that cannot be read shows its error and a red mark.
- FR-12: MUST restore the last selected group and repository on the next launch.
- FR-13: SHOULD let the group name input handle composed text (such as Korean) and copy and paste.
- FR-14: SHOULD let a click dismiss a notice, and a fully successful add clears an earlier failure notice.

## Constraints (mandatory)

- CON-1: MUST NOT delete or modify repository folders.
- CON-2: MUST keep the window responsive while the folder picker is open.
- CON-3: MUST start when a saved file is missing or broken.
- CON-4: MUST keep the saved files as JSON in the config folder `dev.naran.app`.
- CON-5: MUST tell the user when saving fails.

## Success Criteria (mandatory)

- SC-1: Creating, renaming and deleting a group works end to end through the input and the confirmation.
- SC-2: Adding a repository and a non-Git folder together adds only the repository and lists the rest in the notice.
- SC-3: A dragged order of groups or of repositories survives a restart.
- SC-4: Reopening the app selects the last group and repository.
- SC-5: `mise run check` passes and every line of the feature is covered by a test.
