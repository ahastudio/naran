# Feature Specification: Commit History

## Overview

The user reads a repository's history as a list of commits with a branch graph.
Older commits load while scrolling, HEAD is easy to find, and the refs shown can be changed and are remembered.
Picking a past commit shows its message, details, refs and files, read-only.

## User Scenarios & Testing (mandatory)

### User Story 1: Read the commit list

- As: a reader of a repository
- I: can see the commits as rows with the columns Graph, Description, Date, Author and Commit
- So: find a commit and its author at a glance

#### Acceptance Scenarios

Scenario 1: **Column heads stay on one line**

- Given: the commit list is very narrow
- When: the window is laid out
- Then: the column names stay on one line
- Test: `the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow`

Scenario 2: **Uncommitted changes row first**

- Given: the repository has uncommitted changes
- When: the list is shown
- Then: an Uncommitted changes row comes first, below the column heads and above the first commit
- Test: `the_uncommitted_row_comes_first_under_the_column_heads_when_there_are_changes`

Scenario 3: **Clean repository has no Uncommitted row**

- Given: the repository has no uncommitted changes
- When: the list is shown
- Then: there is no Uncommitted changes row and the top commit is picked
- Test: `the_uncommitted_row_comes_first_under_the_column_heads_when_there_are_changes`, `a_clean_repository_starts_on_its_top_commit`

Scenario 4: **History cannot be read**

- Given: the repository cannot be read
- When: the detail is read
- Then: there are no rows, no more pages and no HEAD to go to
- Test: `a_repository_whose_history_cannot_be_read_shows_the_problem_instead_of_rows`, `a_history_that_could_not_be_read_has_no_rows_pages_head_or_top`, `details_that_cannot_be_read_stay_empty_and_a_failed_page_adds_nothing`

### User Story 2: Read the branch graph and HEAD

- As: a reader
- I: can see lanes, merges, ref badges and where HEAD is
- So: tell how branches relate

#### Acceptance Scenarios

Scenario 1: **Linear history**

- Given: commits form one line
- When: lanes are laid out
- Then: every commit sits in lane 0 and nothing passes by
- Test: `linear_history_stays_in_one_lane` (inner level, not reachable through the window)

Scenario 2: **Merge**

- Given: a merge commit has two parents
- When: lanes are laid out
- Then: a second lane opens at the merge and closes at the common parent
- Test: `merge_opens_and_closes_a_lane` (inner level, not reachable through the window)

Scenario 3: **Unrelated tip**

- Given: two newest commits share the same parent
- When: lanes are laid out
- Then: the second sits in a free lane beside the first and the first lane passes by
- Test: `unrelated_tip_takes_a_free_lane` (inner level, not reachable through the window)

Scenario 4: **Truncated log**

- Given: a commit names a parent that is not in the list
- When: lanes are laid out
- Then: the layout still finishes and the commit leaves in lane 0
- Test: `handles_missing_parents_in_a_truncated_log` (inner level, not reachable through the window)

Scenario 5: **What counts as HEAD**

- Given: commits carry ref labels
- When: HEAD is looked for
- Then: only HEAD and HEAD -> branch count, and other labels such as origin/HEAD do not
- Test: `head_is_the_commit_labeled_head_or_head_arrow_branch` (inner level, not reachable through the window)

Scenario 6: **Detached HEAD**

- Given: HEAD is detached
- When: HEAD is looked for
- Then: the commit labeled HEAD is the HEAD commit
- Test: `a_detached_head_counts_too` (inner level, not reachable through the window)

Scenario 7: **Link straight down**

- Given: HEAD is the top commit
- When: the Uncommitted link is drawn
- Then: it ends at the HEAD dot in the same lane and nothing is drawn below it
- Test: `the_link_goes_straight_to_a_head_at_the_top` (inner level, not reachable through the window)

Scenario 8: **Link passes commits above**

- Given: HEAD is on another branch below newer commits
- When: the Uncommitted link is drawn
- Then: it passes the rows above in the free lane of HEAD
- Test: `the_link_passes_the_commits_above_a_head_on_another_branch` (inner level, not reachable through the window)

Scenario 9: **Link takes its own lane**

- Given: the HEAD lane is used by commits above
- When: the Uncommitted link is drawn
- Then: it runs in a lane of its own and does not lie over other lines
- Test: `the_link_takes_a_lane_of_its_own_when_the_head_lane_is_busy_above` (inner level, not reachable through the window)

Scenario 10: **No HEAD in the list**

- Given: HEAD is not among the loaded commits
- When: the Uncommitted link is looked for
- Then: there is no link
- Test: `without_a_head_commit_in_the_list_there_is_no_link` (inner level, not reachable through the window)

Scenario 11: **Graph width**

- Given: the link lane is far to the right
- When: the graph column is sized
- Then: it is wide enough for the link lane
- Test: `the_graph_is_wide_enough_for_the_link_lane` (inner level, not reachable through the window)

### User Story 3: Load older commits while scrolling

- As: a reader of a big repository
- I: can scroll to older commits
- So: avoid waiting for the whole history

#### Acceptance Scenarios

Scenario 1: **Next page at the end**

- Given: a repository has 230 commits and 200 are loaded
- When: the user scrolls to the end of the list
- Then: older commits are added
- Test: `scrolling_to_the_end_adds_older_commits_and_a_short_history_adds_none`

Scenario 2: **Short history adds nothing**

- Given: the repository has one commit
- When: the user scrolls to the end
- Then: nothing more is loaded
- Test: `scrolling_to_the_end_adds_older_commits_and_a_short_history_adds_none`

Scenario 3: **Graph covers every loaded commit**

- Given: a page was added
- When: the list is counted
- Then: the graph has one row per commit, a short page ends paging, and a refresh keeps the loaded length
- Test: `the_history_loads_older_commits_as_the_end_is_reached`

Scenario 4: **Pages equal the whole**

- Given: history is read in pages of two
- When: the pages are joined
- Then: they equal the full history and its graph, and a page past the end is empty
- Test: `history_pages_join_into_the_full_history` (inner level, not reachable through the window)

Scenario 5: **Stale page dropped**

- Given: the list changed while a page was being read
- When: the page arrives
- Then: it is dropped and the loading flag is cleared
- Test: `a_page_for_a_list_that_changed_meanwhile_is_dropped` (inner level, not reachable through the window)

Scenario 6: **Failed page**

- Given: a page read fails
- When: the page arrives
- Then: nothing is added and no more pages are expected
- Test: `details_that_cannot_be_read_stay_empty_and_a_failed_page_adds_nothing` (inner level, not reachable through the window)

### User Story 4: Go to HEAD

- As: a reader
- I: can jump to the HEAD commit
- So: see where I am

#### Acceptance Scenarios

Scenario 1: **HEAD is loaded**

- Given: HEAD is among the loaded commits
- When: the user presses Go to HEAD
- Then: the HEAD commit is picked
- Test: `go_to_head_picks_the_head_commit_each_time_it_is_pressed`

Scenario 2: **Pressed again**

- Given: another row was picked after a jump
- When: the user presses Go to HEAD again
- Then: HEAD is picked again
- Test: `go_to_head_picks_the_head_commit_each_time_it_is_pressed`

Scenario 3: **HEAD is older than the first page**

- Given: HEAD is not in the first 200 commits
- When: the user presses Go to HEAD
- Then: older pages are read until HEAD is found and picked
- Test: `the_go_to_head_button_reads_older_pages_until_head_shows_up`, `go_to_head_finds_a_head_commit_that_is_older_than_the_loaded_page`

Scenario 4: **HEAD with a loaded commit and changes**

- Given: there are uncommitted changes
- When: Go to HEAD is called
- Then: the HEAD commit is picked, not the Uncommitted row
- Test: `go_to_head_picks_the_head_commit_when_it_is_already_loaded`

Scenario 5: **No commits**

- Given: the repository has no commits
- When: it is selected and the user presses Down in the commit list
- Then: the list is empty and nothing is picked
- Test: `a_repository_without_commits_has_an_empty_list_and_nothing_to_pick`

Scenario 6: **No HEAD to find**

- Given: the repository has no commits
- When: Go to HEAD is asked for
- Then: a notice says HEAD is not in the commit list and no jump is pending
- Test: `a_repository_without_commits_has_no_head_to_go_to` (inner level, not reachable through the window: the empty list has no Go to HEAD button)

### User Story 5: Choose the refs shown

- As: a reader
- I: can switch remote branches and Git's own refs on and off
- So: see only the history I mean

#### Acceptance Scenarios

Scenario 1: **All refs button**

- Given: the list is shown with the default refs
- When: the user presses All refs
- Then: the list is read again and the label reads on, then off after a second press
- Test: `the_all_refs_button_adds_the_commits_only_git_keeps_refs_for_and_a_second_press_removes_them`, `showing_all_refs_reloads_the_list_and_flips_the_label`

Scenario 2: **Remote branches button**

- Given: remote branches are shown
- When: the user presses Remote branches
- Then: the label reads off, then on after a second press
- Test: `the_remote_branches_button_removes_remote_only_commits_and_a_second_press_brings_them_back`, `hiding_remote_branches_reloads_the_list_and_flips_the_label`

Scenario 3: **Git's own refs are hidden by default**

- Given: a backup ref of Git exists
- When: history is read with the default refs
- Then: its commit is left out, and it is shown only when everything is asked for
- Test: `the_all_refs_button_adds_the_commits_only_git_keeps_refs_for_and_a_second_press_removes_them`, `history_hides_refs_git_keeps_for_itself_unless_asked`

Scenario 4: **Remote branches can be left out**

- Given: a remote branch is ahead
- When: history is read with remotes off, with and without everything
- Then: the remote-only commit is absent in both cases
- Test: `the_remote_branches_button_removes_remote_only_commits_and_a_second_press_brings_them_back`, `remote_branches_can_be_left_out_of_the_history`

Scenario 5: **Choice survives a restart**

- Given: the user pressed both buttons
- When: the app is opened again
- Then: the saved layout holds the choice and the new window uses it
- Test: `the_ref_buttons_save_their_choice_and_a_restart_keeps_it`

Scenario 6: **Older saved form**

- Given: the layout file holds the old scope name
- When: the app is opened
- Then: the choice is read, and pressing All refs then turns the Git refs off
- Test: `a_layout_file_in_the_older_form_still_sets_the_refs`, `refs_read_the_older_saved_form`

### User Story 6: Pick a past commit

- As: a reader
- I: can click a commit
- So: read its message, details, refs and files without changing anything

#### Acceptance Scenarios

Scenario 1: **Pick by click**

- Given: the list is shown
- When: the user clicks a commit row
- Then: its message and its files are shown and its first file opens
- Test: `the_commit_list_rows_and_buttons_pick_and_move`, `clicking_a_commit_shows_its_message_and_files_and_the_arrows_step_through_the_list`

Scenario 2: **Message, files and diff**

- Given: a commit added a file
- When: it is picked
- Then: the file list, the diff, the message, parent and dated author are read
- Test: `picking_a_commit_shows_its_files_and_diff_and_the_uncommitted_row_brings_the_lists_back`

Scenario 3: **Body and another committer**

- Given: a commit has a message body and a different committer
- When: it is picked
- Then: both the body and the committer are shown
- Test: `clicking_a_commit_with_a_body_and_another_committer_reads_both`, `a_commit_with_a_body_and_another_committer_shows_both`

Scenario 4: **Back to the Uncommitted row**

- Given: a commit is picked
- When: the user clicks the Uncommitted changes row
- Then: the Staged and Unstaged lists come back
- Test: `picking_a_commit_shows_its_files_and_diff_and_the_uncommitted_row_brings_the_lists_back`

Scenario 5: **Clean repository**

- Given: the repository is clean
- When: it is opened
- Then: the top commit is picked
- Test: `a_clean_repository_starts_on_its_top_commit`

### User Story 7: Step through the commits with the keys

- As: a reader
- I: can press Up and Down in the commit list
- So: walk through the history one commit at a time

#### Acceptance Scenarios

Scenario 1: **Down and up**

- Given: the commit list was clicked last
- When: the user presses Down then Up
- Then: the pick moves from the Uncommitted row to the top commit and back
- Test: `arrow_keys_move_through_the_list_that_was_clicked_last`

Scenario 2: **Edges**

- Given: the Uncommitted row is first and one commit is the last row
- When: the user presses Up at the top and Down at the bottom
- Then: the pick stays where it is
- Test: `clicking_a_commit_shows_its_message_and_files_and_the_arrows_step_through_the_list`, `stepping_stays_inside_the_list`

Scenario 3: **List follows the pick**

- Given: 100 commits are loaded
- When: the user steps down 40 rows and back up
- Then: the list scrolls down with the pick and returns to the top
- Test: `the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row`

## Functional Requirements (mandatory)

- FR-1: MUST show the columns Graph, Description, Date, Author and Commit (Story 1).
- FR-2: MUST draw lanes, merges and ref badges in the Graph column (Story 2).
- FR-3: MUST make the HEAD commit stand out (Story 2).
- FR-4: MUST show an Uncommitted changes row first when there are changes, joined to HEAD by a dashed line (Stories 1 and 2).
- FR-5: MUST load older commits as the list is scrolled near its end (Story 3).
- FR-6: MUST offer Go to HEAD, loading older commits when needed (Story 4).
- FR-7: MUST offer Remote branches and All refs buttons that read the list again (Story 5).
- FR-8: MUST save the two choices and read the older saved form (Story 5).
- FR-9: MUST show the message, details, refs and files of a picked commit (Story 6).
- FR-10: MUST step through the list with Up and Down (Story 7).
- FR-11: SHOULD show an error or "No commits yet." instead of rows when the history cannot be read or is empty (Story 1).

## Constraints (mandatory)

- CON-1: MUST be read-only; a picked commit offers no staging.
- CON-2: MUST NOT build rows that are out of view.
- CON-3: MUST use the system `git` for all reading.
- CON-4: MUST ignore a read that finishes after the user moved on or the list changed (Story 3, Scenarios 5 and 6).
- CON-5: The graph MUST show at most 12 lanes.

## Success Criteria (mandatory)

- SC-1: A repository with more than one page of history shows its first 200 commits and adds older ones while scrolling (Story 3).
- SC-2: Go to HEAD ends with the HEAD commit picked, whatever page it is on (Story 4).
- SC-3: Toggling a ref choice changes the list and survives a restart (Story 5).
- SC-4: Every line of this feature is covered by a test (`mise run check` enforces coverage).
