# Implementation Plan: Workspace Layout and Keyboard

## Summary

One workspace entity draws the three columns, the dividers and the key handling, and a small store saves state as JSON files.
The window state is saved by the library entry file, and the layout and the selection are saved by the workspace module.
The feature is built Outside-In with stubs: the acceptance tests are written first and fail, the outermost part is built with everything below it stubbed, and each next part inward replaces the stub the previous step used, until no stub is left and the acceptance tests pass.

## Requirements

1. One window, dark theme, three columns, the selected repository as the window title.
2. Dividers resize the parts, and sizes are kept in range and saved.
3. The window position, size and state are saved and restored.
4. Up, Down, Shift+Up, Shift+Down, Cmd+A, Cmd+R, Cmd+O.
5. JSON files in the config folder and shared widgets.

## Critical Files

### New Files

- `app/src/workspace/tests/layout_acceptance.rs`: acceptance tests driven by the mouse and the keyboard.

### Modified Files

- `app/src/workspace/chrome.rs`: the faded Refresh button does not answer clicks.
- `app/src/workspace/tests.rs`: registers the new test file.

### Reference Files

- `app/src/main.rs`: starts the event loop.
- `app/src/lib.rs`: opens the window, restores and saves the window state.
- `app/src/store.rs`: reads and writes the JSON files and finds the config folder.
- `app/src/theme.rs`: the dark colors.
- `app/src/widgets.rs`, `app/src/widgets/badges.rs`: buttons, links, titles, badges.
- `app/src/workspace.rs`: the workspace, its key bindings and its loading.
- `app/src/workspace/layout.rs`: sizes, dividers, dragging, the saved layout.
- `app/src/workspace/navigation.rs`: moving through lists and Cmd+A.
- `app/src/workspace/state.rs`: the grouped screen state and the file paths.
- `app/src/workspace/feedback.rs`: saving and telling the user how it went.
- `app/src/workspace/detail/columns.rs`: layout of the detail column.
- `app/src/workspace/detail/stepping.rs`: one-row moves in the commit and file lists.
- Tests: `app/src/tests.rs`, `app/src/widgets/tests.rs`, `app/src/workspace/tests/layout.rs`, `keys.rs`, `controls.rs`, `notices.rs`, `scrolling.rs`.

## Architecture

### User Flow

```text
Start the app → window opens at the saved place → saved state loads
   ├─ drag a divider → the part resizes → the layout is saved
   ├─ click a list → it becomes the arrow-key list → Up/Down/Shift/Cmd+A
   ├─ Cmd+R → read again, Cmd+O → add repositories
   └─ move, resize or close the window → the window state is saved
```

### Event Flow

```text
1. Executable starts the event loop and hands over to the library
       ↓
2. Library binds the keys, opens the window at the saved place, saves it on every change and on close
       ↓
3. Workspace loads the layout, repository list, settings and selection
       ↓
4. Each frame sets the title if it changed and maps key presses to moving, picking or refreshing
       ↓
5. A press on a divider starts a drag; each pointer move resizes within range and saves the layout
       ↓
6. A failed save shows in the status bar and as a notice above the repository column
```

### Domain Model

```text
Saved window: position, size, mode (windowed, maximized, full screen)   → window.json
Saved layout: sizes, arrow-key list, refs flags                          → layout.json
Sizes: groups, repositories, commits, files (each kept in range)
Dividers: groups, repositories, commits, files
Arrow-key list: groups, repositories, commits, files
Last pressed: a list, or text                                            → decides what Cmd+A picks
```

## Implementation Steps

### Step 0: Acceptance Tests

**Red:** write first every test that assembles the whole workspace or app on real repositories and checks a user scenario end to end, grouped by spec story.
They are the tests under `app/src/workspace/tests/` (including `layout_acceptance.rs`, `controls.rs`, `keys.rs`, `scrolling.rs`, `adding.rs`, `groups.rs`, `layout.rs` and `notices.rs`) and in `app/src/tests.rs`, which opens and closes the real window.
They certainly fail at first and stay failing until the last step.
Story 1 (read on one screen): `the_three_columns_and_their_dividers_are_on_screen`, `the_app_opens_one_window_with_a_workspace_and_saves_where_it_is`, `the_window_title_is_the_selected_repository`, `refresh_reads_again_from_the_button_and_from_the_keys`, `refresh_does_nothing_while_the_repositories_are_being_read`, `the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow`, `a_repository_that_cannot_be_read_still_takes_a_row_and_counts_as_a_problem`, `a_notice_goes_away_when_it_is_clicked`.
Story 2 (resize with dividers): `dragging_a_divider_with_the_mouse_resizes_it_and_letting_go_ends_the_drag`, `dragging_a_divider_resizes_the_part_next_to_it`, `pressing_a_divider_starts_dragging_it`, `a_notice_is_dismissed_and_a_divider_is_grabbed_and_let_go`, `the_divider_next_to_the_repositories_moves_their_column`, `a_divider_stops_at_both_ends_of_its_range`, `the_layout_is_saved_as_it_changes`, `a_broken_layout_file_opens_the_window_with_every_part_in_range`, `a_broken_layout_file_cannot_hide_a_part`.
Story 3 (start where I left off): `a_saved_window_comes_back_exactly`, `a_saved_window_opens_where_it_was`, `a_saved_window_opens_in_its_mode_and_an_unsaved_one_in_the_default_place`, `a_file_without_a_mode_is_a_normal_window`, `the_size_never_changes_only_the_position_moves_onto_a_screen`, `the_window_has_the_app_name_and_the_given_place`, `the_window_is_saved_once_more_when_it_closes_and_closing_the_last_one_quits`, `closing_one_of_two_windows_does_not_quit`, `the_app_quits_when_its_window_could_not_open`, `the_layout_and_the_selection_come_back_after_a_restart`, `forgets_a_saved_group_that_no_longer_exists`, `a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice`.
Story 4 (read with the keyboard): `the_arrow_keys_move_the_repository_list_and_wrap_around_its_ends`, `pressing_the_group_column_and_a_group_moves_the_keys_there`, `the_arrow_keys_move_through_the_groups`, `arrow_keys_move_through_the_list_that_was_clicked_last`, `stepping_stays_inside_the_list`, `the_arrow_keys_do_nothing_in_a_list_that_is_empty`, `shift_arrows_outside_the_file_lists_pick_nothing`, `the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row`, `the_arrow_keys_scroll_the_file_lists_too`, `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button`, `adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it`, `copying_puts_the_selected_text_on_the_clipboard_and_nothing_else_when_none_is_selected`, `copy_puts_the_picked_text_on_the_clipboard`, `the_window_asks_the_workspace_for_its_focus_handle`.

**Green:** the acceptance tests fail until the last step and pass when it is done, because only then is no stub left; no code is written for them directly.

**Refactoring:** the acceptance files share small helpers for pressing, moving and releasing the mouse, and for reading the sizes and the selection.

### Step 1: Widgets - widgets, theme and drawing

**Red:** write the failing tests of the components, against a stub of the Workspace that gives back the default sizes and an unchanged selection: `the_notes_and_a_row_that_could_not_be_read_take_room_in_the_window`, `every_letter_of_a_changed_file_has_its_color`.
`app/src/widgets/tests.rs` and `app/src/widgets/badges.rs` hold them.

**Green:** build this part with the Workspace stubbed: the chrome file draws the three columns, the toolbar and the status bar and wires the pointer and the keys, the faded Refresh button gets no click handler, and the widgets and theme files give the buttons, titles, notes and badges.
This step keeps its stub for the parts below.

**Refactoring:** keep each drawing function short, keep the key wiring in one place and keep the colors in the theme only.

### Step 2: Workspace - workspace layout, navigation and saving

**Red:** this part has no tests of its own; it is built against the Step 0 tests, with a stub of the Model that gives back every size unchanged and every step as staying in place.

**Green:** build this part with the Model stubbed and replace the Step 1 stub with this real part: the layout file turns a drag into new sizes and saves them and the list clicked last, the navigation and stepping files move the pick one row and scroll it into view, and the feedback file reports a failed save.

**Refactoring:** keep one step computation shared by the lists that stop at their ends.

### Step 3: Model - sizes, steps and the saved window

**Red:** this part has no tests of its own; it is built against the Step 0 tests, with a stub of the Platform that gives back the default for every file and accepts every write.

**Green:** build this part with the Platform stubbed and replace the Step 2 stub with this real part: the sizes are pulled into their ranges, one step stays inside a list, and a saved window keeps its size and moves onto a screen only when it is on none.

**Refactoring:** keep one table of ranges, used by both dragging and loading.

### Step 4: Platform - the store

**Red:** write the failing tests of reading and writing the JSON files: `round_trips_and_falls_back_to_default`, `the_folder_is_inside_the_config_folder_or_the_temporary_one`, `a_value_that_is_not_json_cannot_be_saved`, `a_file_inside_something_that_is_not_a_folder_cannot_be_saved`.
`app/src/store.rs` holds them.

**Green:** this is the innermost step, so no stub is left: it replaces the Step 3 stub with the real store, and the Step 0 tests pass.
The store reads JSON with a default fallback and writes through a temporary file.

**Refactoring:** keep reading and writing in the store only, so every caller treats failures the same way.

## Verification

### Build

```bash
mise run check
```

### Test

```bash
cargo test --workspace
```

### Manual Test

1. Run `cargo run -p naran-app`, drag dividers, move the window, quit and start again.
2. Click a list, then press Up, Down, Shift+Up, Cmd+A, Cmd+R and Cmd+O.

## Considerations

### Reusing Existing Code

The workspace behavior is spread over child modules, so the screen state stays in one entity.

### Compatibility

The layout and window files are saved in the `dev.naran.app` folder, next to `repositories.json` and the groups.
Fields missing from the layout and window files are read as defaults.
