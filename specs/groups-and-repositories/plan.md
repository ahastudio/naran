# Implementation Plan: Groups and Repositories

## Summary

The feature is built Outside-In with stubs in the `app` crate.
Acceptance tests that drive the window are written first and fail until the last step.
Each part is then built with the part below it stubbed, and the next step replaces that stub with the real part.
The workspace view holds the groups, repositories and selection, saves them as JSON files, and runs the system folder picker and Finder in separate processes.

## Requirements

1. Create, rename and delete groups; a repository can be in several groups.
2. Add folders by button, system picker or drop; refuse non-Git folders with a red notice.
3. Reorder groups and repositories by drag, Reveal in Finder, Remove.
4. Show counts and state words in both columns.
5. Restore the last group and repository.

## Critical Files

### New Files

- `app/src/workspace/tests/groups_and_repositories_acceptance.rs`: acceptance tests that drive the window for the scenarios of spec.md

- `app/src/workspace/group_order.rs`: dragging a group row and moving it to the slot of the row it is dropped on

### Modified Files

- `app/src/workspace/tests.rs`: one line that registers the new acceptance test file

### Reference Files

- `app/src/workspace/groups.rs`, `app/src/workspace/repositories.rs`, `app/src/workspace/menu.rs`: the group column, the repository column and the right-click menu
- `app/src/workspace/membership.rs`, `app/src/workspace/navigation.rs`, `app/src/workspace/reading.rs`, `app/src/workspace/feedback.rs`: the actions behind them
- `app/src/workspace.rs`, `app/src/workspace/state.rs`, `app/src/workspace/chrome.rs`: startup, state kinds and the window drop
- `app/src/model/settings.rs`, `app/src/model.rs`, `app/src/widgets.rs`: groups, selection, state words and rows
- `app/src/picker.rs`, `app/src/finder.rs`, `app/src/store.rs`: the other-process helpers and the JSON storage
- `app/src/text_input.rs` and `app/src/text_input/`: the single-line name input
- `core/src/status.rs`: parsing of the Git status output

## Architecture

### User Flow

```text
User -> click "+ New Group", right-click menu, or drag a row
     -> press Cmd+O, click "+ Add Repository", or drop folders on the window
     -> group column and repository column
     -> saved groups, saved repository list, saved selection
```

### Event Flow

```text
1. Add: the folder picker answers (or folders are dropped on the window)
       |
2. Each folder is read in the background as a repository
       |
3. Readable folders join the selected group; unreadable ones become a notice
       |
4. Groups and repository list are saved, a status message is shown,
   and the last added repository is selected
```

```text
Group name: the input emits Enter or Escape
       -> the name is applied (create or rename) or refused
       -> groups are saved, the group is selected, the selection is saved
```

### Domain Model

```text
Settings (settings.json)
└── groups: each has a name and an ordered list of repository paths
Selection (selection.json): last group name, last repository path
Repository list (repositories.json): every known repository path
Repository entry: readable (with its summary) or failed (with its error)
```

## Implementation Steps

The feature is built Outside-In.
Step 0 is the outer frame: acceptance tests that drive the whole window.
Steps 1 to 4 are the parts inward, each with its own loop of a failing test, the smallest behavior that passes, and tidying.
Every test is named in exactly one step.

### Step 0: Acceptance Tests

**Red:** Every test that assembles the whole workspace on real repositories and checks a user scenario end to end is written first.
They live under `app/src/workspace/tests/`, in `groups_and_repositories_acceptance.rs`, `group_order_acceptance.rs`, `groups.rs`, `repositories.rs`, `adding.rs`, `context_menu.rs` and `notices.rs`.
The feature has no tests in `app/src/tests.rs`.
They are grouped by spec story and stay failing until the last step.
Creating groups (story 1): `clicking_new_group_and_typing_a_name_creates_and_selects_the_group`, `a_group_name_is_saved_without_the_spaces_around_it`, `a_taken_name_shows_a_red_notice_that_a_click_dismisses`, `an_empty_name_changes_nothing_and_shows_no_notice`, `creates_and_renames_a_group_by_typing` and `refuses_a_name_that_is_already_taken`.
Renaming, reordering and deleting (story 2): `renaming_from_the_menu_to_a_taken_or_blank_name_changes_nothing`, `the_menu_item_rename_starts_editing_the_group_name`, `the_menu_item_delete_opens_the_confirmation`, `deleting_a_group_asks_first_and_cancel_changes_nothing` `deleting_a_group_drops_the_repositories_only_it_held`, `dragging_a_group_row_with_the_mouse_reorders_the_groups` and `dropping_a_group_on_another_group_changes_the_order_and_keeps_it`.
Adding (story 3): `a_folder_that_is_not_a_repository_is_listed_in_a_notice_on_screen`, `without_a_group_adding_a_folder_asks_to_create_one_first`, `picking_a_repository_the_group_already_holds_keeps_it_once`, `adds_folders_and_reports_the_ones_that_are_not_repositories`, `the_folder_picker_answers_are_handled_without_waiting_for_it`, `adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it`, `puts_repositories_into_a_group_and_takes_them_out`, `without_groups_nothing_is_listed_and_nothing_can_be_added` and `files_dropped_on_the_window_are_added_to_the_group`.
Tidying (story 4): `dragging_a_repository_row_with_the_mouse_reorders_the_group`, `dropping_a_repository_on_another_row_changes_the_order_and_keeps_it`, `the_menu_item_remove_opens_the_confirmation`, `removing_a_repository_asks_first_and_cancel_changes_nothing`, `drops_a_repository_that_leaves_its_last_group`, `reveal_in_finder_closes_the_menu_and_asks_nothing` and `without_a_selected_group_moving_removing_and_editing_do_nothing`.
Restoring (story 5): `a_new_window_restores_the_last_group_and_repository`, `a_new_window_restores_an_empty_group`, `switching_to_a_group_that_holds_the_same_repository_is_remembered`, `broken_saved_files_start_the_app_with_defaults`, `forgets_a_saved_group_that_no_longer_exists`, `reads_every_repository_and_selects_the_first` and `a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice`.
Reading (story 6): `selecting_a_group_shows_only_its_repositories_and_keeps_the_choice`, `clicking_a_repository_row_selects_it` and `the_arrow_keys_move_through_the_groups`.

**Green:** They fail through Steps 1 to 3 and pass only when Step 4 has replaced the last stub; no code is written for them directly.

**Refactoring:** The acceptance file shares small helpers that type a group name, read the group names, open a group menu by right-clicking down the column, and rename from the menu.

### Step 1: Screen - columns, menus and the group name input

**Red:** Tests of the group name input fail first, using a stub in place of the workspace actions below, one that returns fixed answers instead of reading and saving, in `app/src/text_input/tests.rs`, `app/src/text_input/tests/mouse.rs` and `app/src/text_input/tests/platform.rs`.
Typing and editing: `the_text_starts_selected_and_typing_replaces_it`, `the_cursor_moves_by_letter_and_to_both_ends`, `a_selection_collapses_to_the_side_the_cursor_moves_to`, `the_selection_grows_both_ways_and_turns_around` and `backspace_and_delete_remove_a_letter_or_the_selection`.
Clipboard and keys: `copy_cut_and_paste_use_the_clipboard`, `pasting_with_an_empty_clipboard_does_nothing`, `enter_submits_and_escape_cancels`, `enter_while_composing_does_not_submit` and `the_character_palette_key_asks_the_window_for_the_palette`.
Mouse: `the_mouse_places_the_cursor_and_drags_a_selection`, `shift_with_a_click_extends_the_selection`, `a_point_above_or_below_the_line_means_an_end_of_the_text` and `a_click_in_an_empty_input_stays_at_the_start`.
Text services of the input: `a_range_given_in_utf16_is_replaced_and_a_point_past_the_text_has_no_letter`, `composed_text_is_marked_then_confirmed`, `the_text_of_a_range_and_its_place_are_answered_in_utf16`, `without_a_painted_line_nothing_can_be_located`, `the_cursor_and_the_selection_are_painted_with_their_own_quads`, `offsets_convert_between_utf8_and_utf16` and `the_runs_split_around_the_composed_part`.

**Green:** This part is built with the parts below it stubbed, and it is the first real part. The group column lists groups with counts and a new-group link that opens a single-line name input, the repository column lists rows that can be clicked and dragged and shows a red notice a click dismisses, the window accepts dropped folders and Cmd+O, the group rows can be dragged onto each other, and the right-click menus offer Rename, Delete, Reveal in Finder and Remove (`app/src/workspace/groups.rs`, `app/src/workspace/group_order.rs`, `app/src/workspace/repositories.rs`, `app/src/workspace/menu.rs`, `app/src/workspace/chrome.rs`, `app/src/text_input.rs`).

**Refactoring:** Keep drawing separate from the actions it calls, and keep each file within the size limit of `AGENTS.md`.

### Step 2: Workspace

**Red:** The workspace-level tests that call actions and state belong to Step 0, so this part has no tests of its own and is built against the Step 0 tests, with the group rules and the saved files below it stubbed, for example a temporary config folder holding real repositories.

**Green:** This step replaces the stub that Step 1 used with the real actions, while the parts below stay stubbed. Adding reads folders in the background, keeps the readable ones in the selected group, reports the others, saves the files and selects the last one; moving a group or a repository saves the new order, removing and deleting ask first and drop repositories no group holds; startup loads the saved files, makes sure a group is selected and every selection change is saved (`app/src/workspace/membership.rs`, `app/src/workspace/groups.rs`, `app/src/workspace/navigation.rs`, `app/src/workspace/reading.rs`, `app/src/workspace/feedback.rs`, `app/src/workspace.rs`).

**Refactoring:** Split the add flow into short functions so each stays within the limits of `AGENTS.md`.

### Step 3: Model

**Red:** Unit tests of the group rules and the state words in `app/src/model/tests.rs` and `app/src/model/tests/groups.rs` fail first, using a stub in place of the storage and processes below, for example statuses and groups built in memory: `creates_groups_with_new_non_empty_names_only`, `renames_unless_the_name_is_empty_or_taken`, `adds_and_removes_repositories`, `lists_members_in_group_order_and_skips_unknown_paths`, `moving_a_repository_takes_the_slot_of_the_one_it_is_dropped_on`, `moving_a_group_takes_the_slot_of_the_one_it_is_dropped_on`, `counts_changes_and_spots_problems` and `describes_the_most_urgent_state_first`.
The last two cover counts and state words, which the window does not expose to tests.

**Green:** This step replaces the stub that Step 2 used with the real group rules and state words. The group rules, including the order of groups and of their repositories, the saved selection and the one-word state live in `app/src/model/settings.rs` and `app/src/model.rs`.

**Refactoring:** Keep the state rules in one ordered list so the most urgent word wins.

### Step 4: Platform - storage and processes

**Red:** Unit tests of storage and processes fail first, using a program that prints folders or closes at once in place of the system picker, and a program that only starts in place of Finder: `round_trips_and_falls_back_to_default` in `app/src/store.rs`, `a_closed_picker_is_a_cancel_not_an_error` and `cancel_stops_only_the_process_with_the_marker` in `app/src/picker.rs`, and `a_program_that_starts_reveals_and_a_missing_one_is_an_error` in `app/src/finder.rs`.

**Green:** This step replaces the last stub with the real storage and processes, so no stub is left and the Step 0 tests can pass. JSON files are read with defaults and written through a temporary file, and the picker and Finder run as other processes (`app/src/store.rs`, `app/src/picker.rs`, `app/src/finder.rs`).

**Refactoring:** Let tests swap the program so no panel or Finder window opens.

## Verification

### Build

```bash
cargo run -p naran-app
```

### Test

```bash
cargo test --workspace
mise run check
```

`mise run check` also runs format, lint, docs, the quality rules and the coverage rule.

### Manual Test

1. Create, rename and delete a group.
2. Add a Git folder and a plain folder together and read the red notice.
3. Drag a group and a repository row to reorder, quit, reopen, and see both orders and the selection kept.

## Considerations

### Reusing Existing Code

The name input is adapted from GPUI's input example and stays under Apache-2.0, as the header of `app/src/text_input.rs` says.

### Compatibility

The saved groups and repository list are JSON files in the config folder `dev.naran.app` (`app/src/store.rs`).
Missing fields fall back to defaults.
