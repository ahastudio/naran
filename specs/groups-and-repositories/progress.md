# Progress Log

> **Update this file when a phase finishes or a problem appears.**

## Session 2026-11-16

### Phase 1: Requirements & Discovery ✅

**Work log**:

1. Read `README.md` and `AGENTS.md` for the screen, the features and the enforced rules.
2. Explored the workspace actions, the model and the helpers behind the two columns in `app/src/workspace/`, `app/src/model/`, `app/src/picker.rs`, `app/src/finder.rs` and `app/src/store.rs`.
3. Wrote the user stories and scenarios, each traced to a test.

**Created/modified files**:

- `specs/groups-and-repositories/README.md` (created)
- `specs/groups-and-repositories/spec.md` (created)

### Phase 2: Planning & Structure ✅

**Work log**:

1. Wrote the Outside-In plan: window, workspace actions, model, then storage and processes.
2. Assigned every scenario test to a step of the plan.
3. Recorded the decisions that code comments evidence.

**Created/modified files**:

- `specs/groups-and-repositories/plan.md` (created)
- `specs/groups-and-repositories/tasks.md` (created)
- `specs/groups-and-repositories/findings.md` (created)
- `specs/groups-and-repositories/progress.md` (created)

### Phase 3: Implementation ✅

**Work log**:

1. Step 0, acceptance tests: `app/src/workspace/tests/groups_and_repositories_acceptance.rs`, `group_order_acceptance.rs`, `groups.rs`, `repositories.rs`, `adding.rs`, `context_menu.rs` and `notices.rs` under `app/src/workspace/tests/`.
2. Step 1, Screen: `app/src/workspace/groups.rs`, `group_order.rs`, `repositories.rs`, `menu.rs`, `chrome.rs`, the shared drag preview in `app/src/widgets.rs` and `app/src/text_input.rs`, with tests in `app/src/text_input/tests.rs` and `app/src/text_input/tests/`.
3. Step 2, Workspace: `app/src/workspace/membership.rs`, `navigation.rs`, `reading.rs` and `feedback.rs`, covered by the Step 0 tests.
4. Step 3, Model: `app/src/model/settings.rs` and `app/src/model.rs`, with tests in `app/src/model/tests.rs` and `app/src/model/tests/groups.rs`.
5. Step 4, Platform: `app/src/store.rs`, `app/src/picker.rs` and `app/src/finder.rs`, with their unit tests.

**Created/modified files**:

- `app/src/workspace/tests/groups_and_repositories_acceptance.rs` (new)
- `app/src/workspace/tests/group_order_acceptance.rs` (new)
- `app/src/workspace/group_order.rs` (new)
- `app/src/model/tests/groups.rs` (new)
- `app/src/workspace/tests.rs` (two module lines appended)

### Phase 4: Testing ✅

**Work log**:

1. Added acceptance tests that drive the window for the scenarios that lacked one.
2. Ran `cargo test -p naran-app --lib`: 298 passed, 0 failed.

**Created/modified files**:

- `app/src/workspace/tests/groups_and_repositories_acceptance.rs` (created)
- `app/src/workspace/tests.rs` (one `mod` line appended)

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `groups_and_repositories_acceptance::clicking_new_group_and_typing_a_name_creates_and_selects_the_group` | click `+ New Group`, type `home`, Enter | group list is work, home; home selected | ok | ✅ |
| `groups_and_repositories_acceptance::a_group_name_is_saved_without_the_spaces_around_it` | click `+ New Group`, type `  home  `, Enter | group list is work, home | ok | ✅ |
| `groups_and_repositories_acceptance::a_taken_name_shows_a_red_notice_that_a_click_dismisses` | create a group named `work` that exists | no new group; notice on screen; a click clears the notice | ok | ✅ |
| `groups_and_repositories_acceptance::an_empty_name_changes_nothing_and_shows_no_notice` | type only spaces, Enter | group list unchanged; no notice on screen | ok | ✅ |
| `groups::creates_and_renames_a_group_by_typing` | type a Korean name, rename with cmd-a, then type and Escape | saved names follow; Escape adds nothing | ok | ✅ |
| `groups::refuses_a_name_that_is_already_taken` | create `work` twice | one group; message says already exists | ok | ✅ |
| `groups_and_repositories_acceptance::renaming_from_the_menu_to_a_taken_or_blank_name_changes_nothing` | menu Rename of work to spaces, then to `home` | names unchanged; no notice for blank, notice for taken | ok | ✅ |
| `context_menu::the_menu_item_rename_starts_editing_the_group_name` | right-click a group, click Rename | menu closed; rename of `work` in progress | ok | ✅ |
| `context_menu::the_menu_item_delete_opens_the_confirmation` | right-click a group, click Delete… | a confirmation is pending; Cancel keeps 2 groups | ok | ✅ |
| `context_menu::deleting_a_group_asks_first_and_cancel_changes_nothing` | Cancel, then Delete on the unselected group | groups kept, then only work; selection stays work | ok | ✅ |
| `groups::deleting_a_group_drops_the_repositories_only_it_held` | delete group work after confirming | home selected; saved list is beta, gamma | ok | ✅ |
| `group_order_acceptance::dragging_a_group_row_with_the_mouse_reorders_the_groups` | press on the third group row, drag onto the first, release | group order is c, a, b | ok | ✅ |
| `groups::dropping_a_group_on_another_group_changes_the_order_and_keeps_it` | move c onto a, then a onto itself and onto a name that is not a group | saved order c, a, b; selected group and repository unchanged; the last two drops change nothing | ok | ✅ |
| `groups_and_repositories_acceptance::a_folder_that_is_not_a_repository_is_listed_in_a_notice_on_screen` | add a plain folder | list still alpha only; notice on screen | ok | ✅ |
| `groups_and_repositories_acceptance::without_a_group_adding_a_folder_asks_to_create_one_first` | no group, add a repository | notice starts with `Create a group first` | ok | ✅ |
| `groups_and_repositories_acceptance::picking_a_repository_the_group_already_holds_keeps_it_once` | Cmd+O with a picker that prints alpha again | alpha listed and saved once | ok | ✅ |
| `repositories::adds_folders_and_reports_the_ones_that_are_not_repositories` | add a repository and a plain folder | only the repository added; `Added 1, failed 1`; notice `Not added:` | ok | ✅ |
| `adding::the_folder_picker_answers_are_handled_without_waiting_for_it` | picker answers picked, closed, failed | repository added; cancel is no error; failure notice with reason and hint | ok | ✅ |
| `adding::adding_repositories_runs_the_picker_away_from_the_window_and_cancel_stops_it` | Cmd+O with an instantly closed picker, then Cancel with an open one | no error; open picker stopped by signal | ok | ✅ |
| `groups::puts_repositories_into_a_group_and_takes_them_out` | add known repositories to an empty group, then remove one | order beta, alpha; removed one leaves; saved list keeps 3 | ok | ✅ |
| `groups::without_groups_nothing_is_listed_and_nothing_can_be_added` | no groups, add alpha | no repositories listed or saved; message starts `Create a group first` | ok | ✅ |
| `repositories::files_dropped_on_the_window_are_added_to_the_group` | window file drop | message contains `Added 0` | ok | ✅ |
| `repositories::dragging_a_repository_row_with_the_mouse_reorders_the_group` | mouse drag third row to first | order gamma, alpha, beta | ok | ✅ |
| `repositories::dropping_a_repository_on_another_row_changes_the_order_and_keeps_it` | move gamma onto alpha, then alpha onto itself | order gamma, alpha, beta saved; self drop changes nothing | ok | ✅ |
| `context_menu::the_menu_item_remove_opens_the_confirmation` | right-click a repository, click Remove… | a confirmation is pending; Cancel keeps both | ok | ✅ |
| `context_menu::removing_a_repository_asks_first_and_cancel_changes_nothing` | Cancel, then Remove | Cancel keeps both; Remove leaves beta; folder exists | ok | ✅ |
| `groups::drops_a_repository_that_leaves_its_last_group` | remove alpha from the only group | list is beta, gamma; folder exists | ok | ✅ |
| `context_menu::reveal_in_finder_closes_the_menu_and_asks_nothing` | right-click a repository, click Reveal in Finder | menu closed; folder revealed; no prompt | ok | ✅ |
| `repositories::without_a_selected_group_moving_removing_and_editing_do_nothing` | no selected group, move, remove, edit | group unchanged; no prompt | ok | ✅ |
| `groups_and_repositories_acceptance::a_new_window_restores_the_last_group_and_repository` | select group work and step to beta, open a new workspace on the same config | group work and repository beta selected | ok | ✅ |
| `groups_and_repositories_acceptance::a_new_window_restores_an_empty_group` | select the empty group, open a new workspace | the empty group selected | ok | ✅ |
| `groups_and_repositories_acceptance::switching_to_a_group_that_holds_the_same_repository_is_remembered` | repository alpha in two groups, select the other group, open a new workspace | the other group and alpha selected | ok | ✅ |
| `groups_and_repositories_acceptance::broken_saved_files_start_the_app_with_defaults` | broken selection file, then broken settings file | first group selected; then no groups | ok | ✅ |
| `groups::forgets_a_saved_group_that_no_longer_exists` | saved selection names a missing group | default group selected; saved repository kept | ok | ✅ |
| `groups::reads_every_repository_and_selects_the_first` | open with default group | three repositories, first selected, beta described as Changes | ok | ✅ |
| `notices::a_selection_or_a_repository_list_that_cannot_be_saved_shows_a_notice` | unwritable selection and list files | messages `Could not save the selection` and `...repository list` | ok | ✅ |
| `groups::selecting_a_group_shows_only_its_repositories_and_keeps_the_choice` | select group work | only its repositories; selection saved; keys move through them | ok | ✅ |
| `repositories::clicking_a_repository_row_selects_it` | click a row | that repository selected | ok | ✅ |
| `groups::the_arrow_keys_move_through_the_groups` | group pane focused, step 1 | second group selected | ok | ✅ |
| `text_input::tests::the_text_starts_selected_and_typing_replaces_it` | open with `abc`, type `x` | selection 0..3 first; then content `x`, cursor at 1 | ok | ✅ |
| `text_input::tests::the_cursor_moves_by_letter_and_to_both_ends` | end, left, right, home on `a한b` | cursor follows letters and both ends, never past them | ok | ✅ |
| `text_input::tests::a_selection_collapses_to_the_side_the_cursor_moves_to` | left, then cmd-a and right | cursor at 0, then at 3 | ok | ✅ |
| `text_input::tests::the_selection_grows_both_ways_and_turns_around` | shift with left and right, then cmd-a | selection grows, turns around and covers all | ok | ✅ |
| `text_input::tests::backspace_and_delete_remove_a_letter_or_the_selection` | backspace, delete and cmd-a delete on `abcd` | letter or selection removed; nothing left to delete is harmless | ok | ✅ |
| `text_input::tests::copy_cut_and_paste_use_the_clipboard` | cmd-c, cmd-x, cmd-v with a two-line clipboard | text copied and cut; nothing selected copies nothing; paste joins lines with a space | ok | ✅ |
| `text_input::tests::pasting_with_an_empty_clipboard_does_nothing` | cmd-v with an empty clipboard | content stays `abc` | ok | ✅ |
| `text_input::tests::enter_submits_and_escape_cancels` | Enter then Escape | events `submit name` and `cancel` | ok | ✅ |
| `text_input::tests::enter_while_composing_does_not_submit` | Enter while composing Korean text | no event | ok | ✅ |
| `text_input::tests::the_character_palette_key_asks_the_window_for_the_palette` | ctrl-cmd-space | the window is asked for the palette (expected panic in the test window) | ok | ✅ |
| `text_input::tests::the_mouse_places_the_cursor_and_drags_a_selection` | mouse down, drag, release, move | cursor 0..0, drag 0..6, moving after release changes nothing | ok | ✅ |
| `text_input::tests::shift_with_a_click_extends_the_selection` | shift-click at the far right | selection 0..6 | ok | ✅ |
| `text_input::tests::a_point_above_or_below_the_line_means_an_end_of_the_text` | points above, below and right of the line | indexes 0, 6 and 6 | ok | ✅ |
| `text_input::tests::a_click_in_an_empty_input_stays_at_the_start` | click in an empty input | selection 0..0 | ok | ✅ |
| `text_input::tests::a_range_given_in_utf16_is_replaced_and_a_point_past_the_text_has_no_letter` | replace range 1..2 with `x`, locate a point | content `axb`; no letter at that point | ok | ✅ |
| `text_input::tests::composed_text_is_marked_then_confirmed` | compose, replace, confirm Korean text | marked range, content and cursor follow each stage | ok | ✅ |
| `text_input::tests::the_text_of_a_range_and_its_place_are_answered_in_utf16` | ask for a range, selection, bounds and points | `한`, selection 0..3, bounds inside the area, no letter far away | ok | ✅ |
| `text_input::tests::without_a_painted_line_nothing_can_be_located` | clear the painted layout | no bounds and no index | ok | ✅ |
| `text_input::tests::the_cursor_and_the_selection_are_painted_with_their_own_quads` | end then shift-left | selection 2..3 | ok | ✅ |
| `text_input::tests::offsets_convert_between_utf8_and_utf16` | convert offsets in `a한😀b` | offsets match in both directions, clamped at the end | ok | ✅ |
| `text_input::tests::the_runs_split_around_the_composed_part` | six letters: no composed part, one over letters 3 to 4, one over letters 1 to 2 | one run of 6; runs of 2, 2 and 2 with only the middle one underlined; runs of 2 and 4 | ok | ✅ |
| `model::tests::creates_groups_with_new_non_empty_names_only` | create ` new `, empty, duplicate | `new` added; others refused; 3 groups | ok | ✅ |
| `model::tests::renames_unless_the_name_is_empty_or_taken` | rename to taken, blank, missing, valid | only the valid rename succeeds | ok | ✅ |
| `model::tests::adds_and_removes_repositories` | add twice, remove, unknown group, delete group | no duplicate; removal works; unknown group changes nothing | ok | ✅ |
| `model::tests::lists_members_in_group_order_and_skips_unknown_paths` | group order c, missing, a | members c, a | ok | ✅ |
| `model::tests::moving_a_repository_takes_the_slot_of_the_one_it_is_dropped_on` | move within a group | path takes the target slot | ok | ✅ |
| `model::tests::moving_a_group_takes_the_slot_of_the_one_it_is_dropped_on` | move within four groups, then onto itself and onto missing names | group takes the target slot; the other moves change nothing | ok | ✅ |
| `model::tests::counts_changes_and_spots_problems` | statuses with changes, conflict, failure | 6 changes; conflict and failure are problems | ok | ✅ |
| `model::tests::describes_the_most_urgent_state_first` | each state combination | Clean, No upstream, Behind, Ahead, Diverged, Changes, Conflicts, operation words, error text | ok | ✅ |
| `store::tests::round_trips_and_falls_back_to_default` | missing, saved, broken file | default, value, default | ok | ✅ |
| `picker::tests::a_closed_picker_is_a_cancel_not_an_error` | AppleScript -128, no exit code, no output | Canceled in each case | ok | ✅ |
| `picker::tests::cancel_stops_only_the_process_with_the_marker` | a process with the marker, Cancel | stopped by a signal | ok | ✅ |
| `finder::tests::a_program_that_starts_reveals_and_a_missing_one_is_an_error` | a working and a missing program | Ok then Err | ok | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| none recorded | none recorded | 0 | none recorded |

## 5-Question Reboot Check

Questions to recover context when resuming work:

| Question | Answer |
| -------- | ------ |
| 1. Which phase is it now? | Phase 4: Testing, complete |
| 2. What is next? | Nothing planned; every spec scenario has a test, and the save inside group selection is pinned by `switching_to_a_group_that_holds_the_same_repository_is_remembered` |
| 3. What is the goal? | Group, add and tidy repositories, read their state in two columns, and restore the last selection, without touching folders |
| 4. What was learned so far? | See findings.md |
| 5. What was completed? | spec.md with 44 traced scenarios, the Outside-In plan, the acceptance tests and the results above |
