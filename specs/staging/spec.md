# Feature Specification: Staging

## Overview

The user sees the changes of the selected repository in a Staged files list and an Unstaged files list.
The user stages and unstages whole files, hunks and picked lines, and can pick several files and move them together.
Naran never commits, fetches, pulls or pushes, so this is the only way it changes a repository.
Each scenario names the test that proves it.
Tests in `app/src/workspace/tests/staging_acceptance.rs` and `app/src/workspace/tests/staging_diff_acceptance.rs` drive the window with real clicks, keys and drags.
A scenario marked inner level cannot be reached through the window.

## User Scenarios & Testing (mandatory)

### User Story 1: Stage by dragging files

- As: a user reading changes
- I: can drag a file row to the other list
- So: I can decide quickly what goes into the index

#### Acceptance Scenarios

Scenario 1: **Stage**

- Given: a file is in the Unstaged files list
- When: the user drags it onto the Staged files list
- Then: the file is in the Staged files list and git shows it as added
- Test: `dragging_a_row_stages_and_unstages_it_and_the_same_list_does_nothing`

Scenario 2: **Unstage**

- Given: a file is in the Staged files list
- When: the user drags it onto the Unstaged files list
- Then: the file is back in the Unstaged files list
- Test: `dragging_a_row_stages_and_unstages_it_and_the_same_list_does_nothing`

Scenario 3: **Same list**

- Given: a file is in the Unstaged files list
- When: the user drops it on the same list
- Then: nothing changes
- Test: `dragging_a_row_stages_and_unstages_it_and_the_same_list_does_nothing`, `dropping_a_file_on_the_other_list_stages_or_unstages_it`

Scenario 4: **Before the first commit**

- Given: the repository has no commit yet and a file is staged
- When: the user drags it back to the Unstaged files list
- Then: unstaging succeeds
- Test: `unstaging_works_before_the_first_commit`

Scenario 5: **Click opens the diff**

- Given: a file row
- When: the user clicks it
- Then: its diff opens, and a drag still stages it
- Test: `clicking_a_row_opens_its_diff_and_the_diff_follows_the_file_across_lists`, `clicking_a_file_opens_it_and_dragging_it_to_the_other_list_stages_it`

Scenario 6: **A change that fails**

- Given: a listed file was deleted on disk before it was dropped
- When: the user drops it on the Staged files list
- Then: a notice starting with "Stage failed:" shows, and the next successful change clears it
- Test: `a_failed_drop_shows_a_notice_and_the_next_one_clears_it`, `a_failed_stage_shows_a_notice_and_the_next_success_clears_it`

Scenario 7: **Whole files by command**

- Given: untracked files
- When: they are staged and unstaged as whole files
- Then: the last operation is reported as succeeded
- Test: `stages_and_unstages_whole_files`

### User Story 2: Pick several files and move them together

- As: a user reading changes
- I: can pick files with Cmd+A, a Shift range, Cmd toggles and Shift+Up/Down
- So: I can move many files at once

#### Acceptance Scenarios

Scenario 1: **Plain, range and toggle clicks**

- Given: four files in a list
- When: the user clicks one, Shift-clicks another, and Cmd-clicks to remove and add
- Then: the picks are that file, the range between them, then the range minus or plus the Cmd-clicked file
- Test: `clicks_and_keys_pick_files_and_dragging_one_takes_them_all`, `shift_and_cmd_clicks_pick_files_and_one_action_moves_them_all`

Scenario 2: **Grow and shrink with keys**

- Given: a range is picked and a file diff is open
- When: the user presses Shift+Down, then Shift+Up
- Then: the range grows by one row, then shrinks by one row
- Test: `clicks_and_keys_pick_files_and_dragging_one_takes_them_all`, `shift_arrows_grow_the_picked_range_and_a_plain_arrow_picks_one`

Scenario 3: **A plain arrow picks one**

- Given: several files are picked
- When: the user presses a plain arrow
- Then: only the next file is picked
- Test: `shift_arrows_grow_the_picked_range_and_a_plain_arrow_picks_one`

Scenario 4: **Pick all**

- Given: a file list was clicked last
- When: the user presses Cmd+A
- Then: every file of that list is picked
- Test: `cmd_a_picks_every_file_of_the_list_and_there_is_no_select_all_button`

Scenario 5: **Pick all, one file and none**

- Given: a list with one file, or a repository with no changes
- When: the user presses Cmd+A
- Then: the one file is picked, or nothing is picked
- Test: `cmd_a_picks_a_single_file_list_and_does_nothing_on_an_empty_one`

Scenario 6: **Drag one, move all**

- Given: three files are picked
- When: the user drags one of them to the other list
- Then: all three move, the unpicked file stays, and nothing stays picked
- Test: `clicks_and_keys_pick_files_and_dragging_one_takes_them_all`, `dropping_the_picked_files_moves_them_together`

Scenario 7: **Drag preview**

- Given: several files are dragged
- When: the preview is drawn
- Then: it reads "N files"
- Test: `dragging_several_files_shows_how_many_and_the_picks_can_be_cleared` (inner level, not reachable through the window)

Scenario 8: **Picks stay in one list**

- Given: a file is picked in the Staged list
- When: the user Shift-clicks a file in the Unstaged list
- Then: only the Unstaged file is picked
- Test: `picking_in_one_list_does_not_carry_over_to_the_other`

Scenario 9: **A pick that leaves the list**

- Given: three files are picked
- When: one is deleted on disk and the repository is refreshed with Cmd+R
- Then: only the two remaining files stay picked
- Test: `a_pick_that_leaves_the_list_is_dropped_when_the_repository_is_read_again`, `picks_of_files_that_left_the_list_are_dropped`

Scenario 10: **No anchor**

- Given: nothing is picked
- When: a Shift range is asked for
- Then: no range is picked
- Test: `a_shift_click_without_an_anchor_picks_only_that_file`, `picks_without_an_anchor_start_from_the_cursor`

Scenario 11: **Stacked diffs**

- Given: two or more files are picked
- When: the picks change
- Then: their diffs show one after another, limited for a huge pick
- Test: `picking_several_files_shows_all_their_diffs_one_after_another`, `the_stack_is_limited_so_a_huge_pick_stays_light`

### User Story 3: Stage hunks and lines

- As: a user reading changes
- I: can stage or unstage one hunk or the lines I pick in a diff
- So: I can split the changes of one file

#### Acceptance Scenarios

Scenario 1: **Pick and unpick a line**

- Given: the diff of a tracked file is open
- When: the user clicks an added line twice
- Then: it is picked, then not picked; Clear removes the picks
- Test: `a_line_toggles_clear_drops_the_picks_and_no_link_shows_without_picks`, `the_links_of_a_diff_clear_and_stage_the_picked_lines_and_hunks`, `clicking_a_changed_line_picks_it_for_staging`

Scenario 2: **Stage and unstage picked lines**

- Given: a diff with two added lines
- When: the user picks one and clicks Stage lines, then picks it in the Staged diff and clicks Unstage lines
- Then: only that line enters the index, then leaves it
- Test: `picked_lines_stage_and_unstage_from_the_diff_links`, `stages_only_the_picked_lines_and_unstages_them_again`

Scenario 3: **Missing final newline**

- Given: the file's last line has no newline and a line is added after it
- When: that line is staged
- Then: the kept line gets its newline back
- Test: `a_line_added_after_an_unterminated_last_line_stages_with_its_newline`, `a_kept_last_line_gets_its_newline_back_when_a_line_is_added_after_it` (inner level for the patch)

Scenario 4: **Stage and unstage a hunk**

- Given: a changed file
- When: the user clicks Stage hunk, then Unstage hunk in the Staged diff
- Then: the hunk enters the index, then leaves it
- Test: `a_hunk_is_staged_and_unstaged_from_the_diff_links`

Scenario 5: **One hunk of two**

- Given: a file with two separate hunks
- When: the first hunk is staged
- Then: the second stays unstaged
- Test: `stages_a_whole_hunk`

Scenario 6: **No selection**

- Given: no line is picked
- When: the user looks at the diff header
- Then: no Stage lines link is offered, and an empty selection yields no patch
- Test: `a_line_toggles_clear_drops_the_picks_and_no_link_shows_without_picks`, `returns_none_without_a_selection_or_hunks` (inner level for the patch)

Scenario 7: **The rest stays**

- Given: part of a file is staged
- When: the lists are read again
- Then: the file is in both lists
- Test: `stages_only_the_picked_lines_and_unstages_them_again`

Scenario 8: **Unselected changes keep their state**

- Given: a selection inside a hunk
- When: the patch is built
- Then: unselected changes become context or are dropped
- Test: `unselected_changes_keep_their_original_state`, `selecting_one_added_line_turns_the_other_into_nothing_or_context`, `selects_a_whole_hunk` (inner level, not reachable through the window)

### User Story 4: Whole-file staging only

- As: a user reading changes
- I: expect untracked and conflicted files to be staged only as a whole
- So: partial staging is never attempted where it makes no sense

#### Acceptance Scenarios

Scenario 1: **No partial controls**

- Given: an untracked file and a conflicted file
- When: each diff is opened and a line is clicked
- Then: there is no hunk link, no Stage lines link and no picked line
- Test: `untracked_and_conflicted_files_have_no_partial_staging_but_stage_whole`

Scenario 2: **Stage whole by drag**

- Given: the same two files
- When: each is dragged to the Staged files list
- Then: the conflict is resolved to staged and the untracked file is listed as staged
- Test: `untracked_and_conflicted_files_have_no_partial_staging_but_stage_whole`

Scenario 3: **Forced line staging does nothing**

- Given: a conflicted file with all diff lines passed to the line staging
- When: the lines are applied
- Then: the conflict stays
- Test: `a_conflicted_file_can_only_be_staged_whole`

### User Story 5: Badges, renames and new folders

- As: a user reading changes
- I: can tell the kind of each change from a one-letter badge, and see where a renamed file came from
- So: I can grasp the changes quickly

#### Acceptance Scenarios

Scenario 1: **New folder**

- Given: a new folder holds three files
- When: the user looks at the Unstaged files list
- Then: each file has its own row
- Test: `every_file_of_a_new_folder_is_listed_and_a_conflict_is_only_in_unstaged`, `every_file_in_a_new_folder_is_listed_and_counted`

Scenario 2: **Conflict**

- Given: a merge stopped on a conflict
- When: the user looks at the lists
- Then: the conflicted file is in the Unstaged files list only
- Test: `every_file_of_a_new_folder_is_listed_and_a_conflict_is_only_in_unstaged`

Scenario 3: **Letters**

- Given: changes of each kind
- When: badges are chosen
- Then: A, D, R, M, ? and ! are shown, and a conflict is always `!`
- Test: `badges_name_the_change_and_a_conflict_is_always_marked` (inner level, not reachable through the window)

Scenario 4: **Which list**

- Given: status letters in the index and the working directory
- When: files are sorted into lists
- Then: a file with both kinds of change is in both lists
- Test: `a_file_with_staged_and_unstaged_changes_is_in_both_lists`, `sorts_files_into_the_staged_and_working_directory_lists`

Scenario 5: **The open diff follows the file**

- Given: a diff is open
- When: the file changes list
- Then: the diff stays on its side while the file has changes there, else follows the file
- Test: `clicking_a_row_opens_its_diff_and_the_diff_follows_the_file_across_lists`, `shows_the_diff_of_a_file_and_follows_it_through_staging`, `an_open_diff_stays_on_its_side_or_follows_the_file`

Scenario 6: **Renamed file**

- Given: a file was renamed with `git mv`, and another file is new
- When: the user looks at the lists
- Then: the renamed file's row reads `new path <- old path` and the new file's row reads its path only
- Test: `a_renamed_file_shows_its_new_path_then_the_old_one_and_the_others_show_their_path`, `a_renamed_file_is_labeled_with_its_new_path_then_the_old_one`, `a_file_that_was_not_renamed_is_labeled_with_its_path_only`, `a_staged_rename_lists_the_new_path_and_the_old_one`

Scenario 7: **Rename with a later edit**

- Given: a staged rename whose new file was edited again, so it is in both lists
- When: the user looks at the lists
- Then: the Staged row reads `new path <- old path` and the Unstaged row reads the new path only
- Test: `a_rename_that_is_staged_is_not_shown_as_one_in_the_unstaged_list`

Scenario 8: **Unstage a rename**

- Given: a staged rename
- When: the user moves its row to the Unstaged files list
- Then: the old path is not left staged as deleted; the index is as before the rename
- Test: `unstaging_a_renamed_file_brings_back_the_old_path`

Scenario 9: **Diff of a staged rename**

- Given: a file was renamed with `git mv` and one line of it was changed before it was staged
- When: the user opens the file in the Staged list
- Then: the diff shows only the changed lines, not the whole file as added, and unstaging lines of it keeps the rename staged
- Test: `a_renamed_file_shows_only_the_lines_that_changed`, `a_staged_rename_shows_only_what_changed_and_its_lines_unstage_without_undoing_the_rename`

Scenario 10: **A rename without edits**

- Given: a file was renamed with `git mv` and not changed
- When: the user opens the file in the Staged list
- Then: the diff shows one note, `Renamed from <old path>`, instead of an empty pane; a rename with edits shows the same note above its changed lines
- Test: `a_rename_without_edits_opens_a_diff_that_says_what_the_file_was`, `a_rename_says_what_the_file_was_even_when_no_line_changed` in `app/src/model/tests/diff.rs`; inner level, the text of the note is made by the model

Scenario 11: **A rename of a name Git quotes**

- Given: a staged rename whose new name has a tab, a quote or a backslash in it, so Git writes it in quotes
- When: lines of it are staged or unstaged
- Then: the patch names the new path alone, with `a/` and `b/` inside the quotes
- Test: `a_renamed_file_is_named_by_its_new_path_alone_even_when_git_quotes_it` in `core/src/patch/tests.rs`; inner level, not reachable through the window

## Functional Requirements (mandatory)

- FR-1: MUST show changed files in a Staged files list and an Unstaged files list (US5).
- FR-2: MUST list a file in both lists when it has staged and unstaged changes (US5-4, US3-7).
- FR-3: MUST list every file of a new folder on its own (US5-1).
- FR-4: MUST stage or unstage files dropped on the other list (US1-1 to US1-4).
- FR-5: MUST pick files with click, Shift click, Cmd or Ctrl click, Shift+Up/Down and Cmd+A, inside one list (US2).
- FR-6: MUST take all picked files along when one of them is dragged (US2-6).
- FR-7: MUST show the diffs of two or more picked files stacked (US2-11).
- FR-8: MUST stage or unstage one hunk or the picked lines of a tracked file (US3).
- FR-9: MUST stage untracked and conflicted files only as a whole (US4).
- FR-10: MUST show a status badge on each row (US5-3).
- FR-11: MUST report the result of each change, and show the reason in a notice when it fails (US1-6).
- FR-12: SHOULD show the number of picked files in the list header (US2).
- FR-13: MUST read the repository again after a change (US2-9).
- FR-14: MUST show a file renamed or copied in the index as `new path <- old path` in the Staged list (US5-6, US5-7).
- FR-15: MUST unstage a renamed file together with its old path (US5-8).

## Constraints (mandatory)

- CON-1: MUST NOT commit, fetch, pull or push.
- CON-2: MUST run one change at a time; a request made while another runs is ignored.
- CON-3: MUST run changes in the background so the window stays responsive.
- CON-4: MUST keep picks inside one list (US2-8).
- CON-5: MUST use the system `git`.
- CON-6: Picking files of a past commit is outside this feature.

## Success Criteria (mandatory)

- SC-1: A file dropped on the other list appears there (US1-1, US1-2).
- SC-2: Only the picked lines or the chosen hunk reach the index; the rest stay in the working directory (US3-2, US3-4, US3-5).
- SC-3: Untracked and conflicted files offer no partial staging (US4-1).
- SC-4: Dragging several picked files moves all of them in one change (US2-6).
- SC-5: Every file of a new folder has its own row (US5-1).
- SC-6: A failed change shows its reason, and the next success clears the notice (US1-6).
- SC-7: A renamed file's row names both its new and its old path (US5-6).
- SC-8: Unstaging a renamed file leaves nothing staged as deleted (US5-8).
