# Feature Specification: Live Updates

## Overview

The repository list follows the real state of the file system automatically, and can also be refreshed by hand.
Each repository is summarized by its branch, change counts and a one-word state.

## User Scenarios & Testing (mandatory)

Acceptance tests live in `app/src/workspace/tests/live_updates_acceptance.rs` unless a path is given.

### User Story 1: Outside changes show at once

- As: a developer who watches several repositories
- I: can see changes made in a terminal or editor without refreshing
- So: the screen never disagrees with the real state

#### Acceptance Scenarios

Scenario 1: **A file is added**

- Given: a repository shows "No upstream"
- When: a new file is created in it from a terminal
- Then: its state becomes "Changes" without a refresh
- Test: `shows_changes_made_outside_the_app_without_a_refresh` in `app/src/workspace/tests/notices.rs`

Scenario 2: **A burst of writes**

- Given: ten files are written to one repository in a row
- When: the writes pause
- Then: the repository ends up with all ten files counted
- Test: `a_burst_of_writes_ends_with_every_file_counted`; the single report per burst is proved by `reports_a_burst_of_writes_once_with_the_registered_path` and `debounce_groups_a_burst_and_stops_when_either_side_is_gone` in `app/src/watcher/tests.rs`

Scenario 3: **Ignored files**

- Given: a repository ignores `*.log`
- When: an ignored file and then a real file are written
- Then: only the real file is counted
- Test: `ignored_files_are_not_counted_but_the_next_real_change_is`; the relevance rules are in `ignores_noise` and `watches_worktree_files_and_git_state` in `app/src/watcher/tests.rs`

Scenario 4: **Folders that cannot be watched**

- Given: a registered folder no longer exists
- When: the window opens
- Then: it is listed as failed, it is left out of watching and keeps its own name, and the other repositories still show their states
- Test: `a_registered_folder_that_is_gone_is_listed_as_failed_and_the_rest_still_work`; inner: `a_folder_that_cannot_be_watched_is_left_out_and_keeps_its_own_name` in `app/src/watcher/tests.rs`

Scenario 5: **Events for the wrong place or failed events**

- Given: events arrive for a file in the work folder, an unneeded `.git` folder, a folder outside every repository, and a failed event
- When: the events are routed
- Then: only the work folder file is reported
- Test: `routes_relevant_events_to_their_repository_and_skips_failed_ones` in `app/src/watcher/tests.rs`; inner level, not reachable through the window

Scenario 6: **A lost event**

- Given: an event was lost
- When: the periodic tick comes
- Then: every repository is reported again, until nobody listens
- Test: `the_tick_reports_every_repository_until_nobody_listens` in `app/src/watcher/tests.rs`; inner level, not reachable through the window (the tick comes every 60 seconds)

Scenario 7: **The window is gone**

- Given: the window is closed while changes are still reported
- When: a change arrives
- Then: nothing is read and the loop ends
- Test: `changes_stop_being_read_once_the_window_is_gone` in `app/src/workspace/tests/controls.rs`

Scenario 8: **Watching cannot start**

- Given: the file watcher fails to start
- When: the workspace sets up watching
- Then: a notice says "Could not watch the repositories for changes. Use Refresh." and nothing is watched
- Test: `watching_that_cannot_start_is_reported_and_leaves_nothing_watched` in `app/src/workspace/tests/notices.rs`

Scenario 9: **A change report while lines are picked**

- Given: lines of a diff are picked and the file's diff has not changed
- When: the repository is read again because of a change report
- Then: the picked lines stay picked and can be staged
- Test: `a_change_report_that_leaves_the_diff_alone_keeps_the_picked_lines`

Scenario 10: **A change report that changes the diff**

- Given: lines of a diff are picked
- When: the file changes and the repository is read again
- Then: the picks are dropped, because they point at lines that moved
- Test: `a_change_report_that_changes_the_diff_drops_the_picked_lines`

### User Story 2: Refresh by hand

- As: a user
- I: can read every repository again with the Refresh button or Cmd+R
- So: I get the latest state without waiting for automatic updates

#### Acceptance Scenarios

Scenario 1: **Button and key**

- Given: a repository changed outside the app
- When: the Refresh button and then Cmd+R are used
- Then: every repository is read again and reading has ended
- Test: `refresh_reads_again_from_the_button_and_from_the_keys` in `app/src/workspace/tests/controls.rs`

Scenario 2: **While reading**

- Given: a refresh has started
- When: the reading is not finished
- Then: the workspace is marked as reading, which makes the button read "Reading…" and not respond, and it returns to normal afterward
- Test: `the_refresh_button_waits_while_reading_and_returns_after`; it starts the refresh directly because a simulated click lets the read finish at once, so the button label itself is not asserted

Scenario 3: **Repeated refreshes**

- Given: a file was added and refreshed, then another file was added
- When: Cmd+R is used again
- Then: each refresh shows the latest count (1, then 2)
- Test: `refreshing_twice_in_a_row_reads_the_latest_state_each_time`

Scenario 4: **Nothing to read**

- Given: the only group has no repositories
- When: Cmd+R is used
- Then: no entries are shown and the workspace is not reading
- Test: `a_group_without_repositories_has_nothing_to_read`

Scenario 5: **Startup reads everything**

- Given: three registered repositories
- When: the window opens
- Then: all three are listed, the first is selected and its details are loaded
- Test: `reads_every_repository_and_selects_the_first` in `app/src/workspace/tests/groups.rs`

Scenario 6: **A repository leaves its last group**

- Given: a repository is in only one group
- When: it is removed from that group
- Then: it is no longer listed or saved, and its folder is untouched
- Test: `drops_a_repository_that_leaves_its_last_group` in `app/src/workspace/tests/groups.rs`

Scenario 7: **Unchanged entries**

- Given: a read result equals the current entry, differs from it, or belongs to no known repository
- When: the result is applied
- Then: only a known and different entry is replaced
- Test: `an_entry_is_replaced_only_when_it_is_known_and_different` in `app/src/workspace/tests/notices.rs`

### User Story 3: See a repository's state at a glance

- As: a user
- I: can see the most urgent state of each repository as one word
- So: I know where to look first

#### Acceptance Scenarios

Scenario 1: **Priority**

- Given: statuses with no upstream, upstream only, behind, ahead, both, changes with ahead, conflicts, and each operation in progress
- When: the state is described
- Then: the word follows the order operation, Conflicts, Changes, Diverged, Ahead, Behind, No upstream, Clean
- Test: through the window, `the_state_shows_the_most_urgent_word_after_a_refresh`, `behind_then_diverged_follow_the_upstream_after_refreshes` and `ahead_shows_after_a_local_commit_and_changes_outrank_it` show Changes, Merging, Clean, Behind, Diverged, Ahead and No upstream; the full order, including Conflicts, Rebasing, Cherry-picking and Reverting, is `describes_the_most_urgent_state_first` in `app/src/model/tests.rs`, inner level, not reachable through the window

Scenario 2: **Changes and merging from a terminal**

- Given: one repository has an untracked file and another has none
- When: a conflicting merge is started in the second and Cmd+R is used
- Then: they show "Changes" and "Merging"
- Test: `the_state_shows_the_most_urgent_word_after_a_refresh`

Scenario 3: **Behind, then Diverged**

- Given: a branch tracks `main` and is clean
- When: `main` gets a commit and then the branch gets one too, with a refresh each time
- Then: the state goes Clean, Behind, Diverged
- Test: `behind_then_diverged_follow_the_upstream_after_refreshes`

Scenario 4: **Ahead, then Changes**

- Given: a branch tracks `main`
- When: it gets a local commit and then an untracked file, with a refresh each time
- Then: the state goes Ahead, then Changes
- Test: `ahead_shows_after_a_local_commit_and_changes_outrank_it`

Scenario 5: **Unreadable repository**

- Given: a repository folder is deleted
- When: Cmd+R is used
- Then: that entry fails and shows as a problem, and the other repositories are still read
- Test: `a_repository_that_cannot_be_read_shows_its_error_as_a_problem`

Scenario 6: **Counting changes**

- Given: staged, unstaged and untracked files, a conflict and a failed entry
- When: changes are counted and problems checked
- Then: staged, unstaged and untracked are summed, and conflicts and failures count as problems
- Test: `counts_changes_and_spots_problems` in `app/src/model/tests.rs`; inner level, not reachable through the window

Scenario 7: **Status summary**

- Given: output of `git status` for an empty repository, a tracked branch, a detached head, and mixed changes
- When: it is parsed
- Then: branch, head, upstream, ahead/behind and counts match
- Test: `empty_repository`, `branch_with_upstream`, `detached_head`, `changes` in `core/src/status.rs`; `counts_changes`, `detects_merge_conflict`, `every_file_in_a_new_folder_is_listed_and_counted` in `core/tests/summarize.rs`; inner level, not reachable through the window

Scenario 8: **Not a repository**

- Given: a folder that is not a repository
- When: it is summarized
- Then: a command error is returned
- Test: `reports_non_repository` in `core/tests/summarize.rs`; inner level, not reachable through the window

Scenario 9: **Many repositories**

- Given: many repositories, or none
- When: they are read with a limit
- Then: all results come back in input order, and an empty list gives an empty result
- Test: `keeps_input_order_and_runs_everything`, `handles_no_paths` in `core/src/batch.rs`; `runs_in_parallel_across_repositories` in `core/tests/reading.rs`; inner level, not reachable through the window

## Functional Requirements (mandatory)

- FR-1: MUST read every registered repository and start watching when the window opens.
- FR-2: MUST read again only the repositories where a change was detected.
- FR-3: MUST group a burst of changes into one update.
- FR-4: MUST detect changes in the work folder and in the branch, index and in-progress operation state.
- FR-5: MUST ignore files matched by `.gitignore` and `.git` changes that do not affect the screen.
- FR-6: MUST report every repository again at a regular interval in case an event was lost.
- FR-7: MUST read every repository again from the Refresh button and Cmd+R.
- FR-8: MUST NOT freeze the window while repositories are read.
- FR-9: MUST summarize each repository by branch, ahead/behind, and staged/modified/untracked/conflicted counts.
- FR-10: MUST show the most urgent state as one word.
- FR-11: MUST disable the Refresh button while reading.
- FR-12: SHOULD tell the user when watching cannot start and point to Refresh.
- FR-13: SHOULD neither read nor watch a repository that no group holds.
- FR-14: SHOULD NOT redraw the screen for a repository whose entry did not change.

## Constraints (mandatory)

- CON-1: MUST only read; never commit, fetch, pull or push.
- CON-2: MUST read state with the system `git`.
- CON-3: MUST limit how many `git` processes run at once.
- CON-4: MUST NOT let one unreadable repository affect the others.

## Success Criteria (mandatory)

- SC-1: A file created in a terminal changes the state without a refresh.
- SC-2: A burst of writes causes one update per repository.
- SC-3: After Cmd+R every repository shows its latest state.
- SC-4: The one-word state follows the priority order.
- SC-5: The window keeps responding while reading.
