# Progress Log

> **Update this file when you finish a phase or hit a problem.**

## Session 2026-11-16

### Phase 1: Requirements & Discovery ✅

**Work log**:

1. README.md and AGENTS.md were read for the screen, the keys and the project rules.
2. The window, divider, saving and keyboard code in `app/src/lib.rs` and `app/src/workspace/` was read.
3. Four user stories with numbered scenarios were written, each traced to a test.

**Created/modified files**:

- `specs/workspace-layout/spec.md` (created)

### Phase 2: Planning & Structure ✅

**Work log**:

1. A four-step Outside-In plan was written, from the window down to the store.
2. The files, saved file names and tests named in the plan were confirmed to exist.

**Created/modified files**:

- `specs/workspace-layout/plan.md` (created)
- `specs/workspace-layout/README.md` (created)

### Phase 3: Implementation ✅

**Work log**:

1. Step 0: the tests under `app/src/workspace/tests/` (eight of them added in `layout_acceptance.rs`) and `app/src/tests.rs` drive the app and window.
2. Step 1: widgets, theme and drawing in `app/src/widgets.rs`, `app/src/widgets/badges.rs`, `app/src/theme.rs` and `app/src/workspace/chrome.rs`; the Refresh rule is in `chrome.rs`.
3. Step 2: layout, navigation and saving in `app/src/workspace/layout.rs`, `navigation.rs`, `feedback.rs` and `app/src/workspace/detail/stepping.rs`.
4. Step 3: sizes, steps and the saved window in `layout.rs`, `navigation.rs` and `app/src/lib.rs`.
5. Step 4: the store in `app/src/store.rs`.

**Created/modified files**:

- `app/src/workspace/chrome.rs` (modified)
- `app/src/workspace/tests/layout_acceptance.rs` (created)
- `app/src/workspace/tests.rs` (modified: one line registers the new file)

### Phase 4: Testing ✅

**Work log**:

1. The tests of this feature ran with `cargo test -p naran-app` and name filters, and all passed.
2. tasks.md, findings.md and progress.md were written.

**Created/modified files**:

- `specs/workspace-layout/tasks.md`, `findings.md`, `progress.md` (created)

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `the_three_columns_and_their_dividers_are_on_screen` | open the window | four dividers on screen, in order | as expected | ✅ |
| `the_app_opens_one_window_with_a_workspace_and_saves_where_it_is` | start the app, resize the window | one window, 900 by 600 saved | as expected | ✅ |
| `the_window_title_is_the_selected_repository` | select, move down, deselect | alpha, beta, Naran | as expected | ✅ |
| `refresh_reads_again_from_the_button_and_from_the_keys` | new file, click Refresh, press Cmd+R | two untracked files in beta | as expected | ✅ |
| `refresh_does_nothing_while_the_repositories_are_being_read` | reading is on, click Refresh | still reading, no new read | passed | ✅ |
| `the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow` | side columns 700 and 800 wide | list under 450 wide, head height unchanged | as expected | ✅ |
| `a_repository_that_cannot_be_read_still_takes_a_row_and_counts_as_a_problem` | delete beta's folder, open, click the second row | beta selected, one repository with a problem | as expected | ✅ |
| `a_notice_goes_away_when_it_is_clicked` | report a failure, click the notice | no notice | as expected | ✅ |
| `dragging_a_divider_with_the_mouse_resizes_it_and_letting_go_ends_the_drag` | press the groups divider, move to 260, let go, move to 300; then drag the commits divider to 344 | groups 260 and unchanged after letting go; commits 300 | as expected | ✅ |
| `dragging_a_divider_resizes_the_part_next_to_it` | drag groups to 260, commits to 344 and to 5 | 260, 300, 80 | as expected | ✅ |
| `pressing_a_divider_starts_dragging_it` | press the groups divider | dragging groups | as expected | ✅ |
| `a_notice_is_dismissed_and_a_divider_is_grabbed_and_let_go` | dismiss, start and release a drag | no notice, dragging, then none | as expected | ✅ |
| `the_divider_next_to_the_repositories_moves_their_column` | drag to the edge of the repositories column | width 300 | as expected | ✅ |
| `a_divider_stops_at_both_ends_of_its_range` | drag the groups divider to 900, 3, then 150 | 400, 120, 150 | as expected | ✅ |
| `the_layout_is_saved_as_it_changes` | three drags and a pane change | layout.json equals the screen | as expected | ✅ |
| `a_broken_layout_file_opens_the_window_with_every_part_in_range` | layout.json with wild numbers, then not JSON | in-range sizes, then the defaults | as expected | ✅ |
| `a_broken_layout_file_cannot_hide_a_part` | missing, broken, out-of-range layout.json | defaults, clamped sizes, kept pane | as expected | ✅ |
| `a_saved_window_comes_back_exactly` | windowed, maximized, full screen bounds | same after a JSON round trip | as expected | ✅ |
| `a_saved_window_opens_where_it_was` | window.json with a place | window opens at that place | as expected | ✅ |
| `a_saved_window_opens_in_its_mode_and_an_unsaved_one_in_the_default_place` | three modes, and none saved | saved mode, or windowed default | as expected | ✅ |
| `a_file_without_a_mode_is_a_normal_window` | window.json without mode | windowed | as expected | ✅ |
| `the_size_never_changes_only_the_position_moves_onto_a_screen` | places on, off and bigger than the screen | only the position moves | as expected | ✅ |
| `the_window_has_the_app_name_and_the_given_place` | window options for a place | that place, title Naran | as expected | ✅ |
| `the_window_is_saved_once_more_when_it_closes_and_closing_the_last_one_quits` | close the window, remove it | file saved, no windows | as expected | ✅ |
| `closing_one_of_two_windows_does_not_quit` | two windows, close one | one window | as expected | ✅ |
| `the_app_quits_when_its_window_could_not_open` | opened and failed results | quits only on failure | passes | ✅ |
| `the_layout_and_the_selection_come_back_after_a_restart` | click the second repository, drag a divider, click the commit list, open again | same repository, size 260, commits list | as expected | ✅ |
| `forgets_a_saved_group_that_no_longer_exists` | selection.json names a missing group | first group, saved repository kept | as expected | ✅ |
| `a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice` | unwritable selection and list files | a failed message for each | as expected | ✅ |
| `the_arrow_keys_move_the_repository_list_and_wrap_around_its_ends` | click the second row, press Down, Down, Up | third, first, third | as expected | ✅ |
| `pressing_the_group_column_and_a_group_moves_the_keys_there` | click the second group | groups pane, beta listed | as expected | ✅ |
| `the_arrow_keys_move_through_the_groups` | group list is the key list, Down | second group | as expected | ✅ |
| `arrow_keys_move_through_the_list_that_was_clicked_last` | files, then commits, step down and up | files and commit pick move and return | as expected | ✅ |
| `stepping_stays_inside_the_list` | steps at the ends and with no current row | stays in the list, none when empty | as expected | ✅ |
| `the_arrow_keys_do_nothing_in_a_list_that_is_empty` | empty group, no groups, Down | nothing selected | as expected | ✅ |
| `shift_arrows_outside_the_file_lists_pick_nothing` | Shift+Down, Shift+Up in the repositories | nothing picked | as expected | ✅ |
| `the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row` | 40 steps down and 40 up | scrolled down 600 or more, then 0 | as expected | ✅ |
| `the_arrow_keys_scroll_the_file_lists_too` | 60 steps down in 80 files | scrolled 600 or more | as expected | ✅ |
| `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button` | Cmd+A after a file, then after the diff | four files, then the diff text only | as expected | ✅ |
| `adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it` | Cmd+O, a second request while open, Cancel | no error, no second picker, the open one stopped | as expected | ✅ |
| `copying_puts_the_selected_text_on_the_clipboard_and_nothing_else_when_none_is_selected` | copy selected, then unselected text | hello stays | as expected | ✅ |
| `copy_puts_the_picked_text_on_the_clipboard` | pick text, Cmd+C | the text | as expected | ✅ |
| `the_window_asks_the_workspace_for_its_focus_handle` | read the focus handle | the workspace handle | as expected | ✅ |
| `the_notes_and_a_row_that_could_not_be_read_take_room_in_the_window` | draw a note, a problem, a failed row | each taller than 0 | as expected | ✅ |
| `every_letter_of_a_changed_file_has_its_color` | letters A D ! M T R C ? | each maps to its theme color | as expected | ✅ |
| `round_trips_and_falls_back_to_default` | missing, valid, broken file | default, round trip, default | as expected | ✅ |
| `the_folder_is_inside_the_config_folder_or_the_temporary_one` | config folder present or absent | `dev.naran.app` inside it | as expected | ✅ |
| `a_value_that_is_not_json_cannot_be_saved` | map with pair keys | save fails | as expected | ✅ |
| `a_file_inside_something_that_is_not_a_folder_cannot_be_saved` | path under a file, empty path | save fails | as expected | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| 2026-11-16 | `refresh_does_nothing_while_the_repositories_are_being_read` failed: the click started a new read | 1 | Attach the click handler only when not reading |

## 5-Question Reboot Check

Questions to recover context when resuming work. If all five can be answered, work can resume:

| Question | Answer |
| -------- | ------ |
| 1. Which phase is it now? | All four phases are done. |
| 2. What is next? | Nothing planned; update these files when the feature changes. |
| 3. What is the goal? | Document and test the screen layout, dividers, keyboard, and saving and restoring. |
| 4. What was learned so far? | See findings.md: the Refresh rule and the wrapping repository list. |
| 5. What was completed? | The six documents, eight acceptance tests in `app/src/workspace/tests/layout_acceptance.rs`, and the Refresh rule in `app/src/workspace/chrome.rs`. |
