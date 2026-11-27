# Feature Specification: Worktrees

## Overview

A repository row can be opened to show the other worktrees of the same repository.
Each worktree is read like a repository and can be selected to see its changes, history and staging.

## User Scenarios & Testing (mandatory)

Acceptance tests live in `app/src/workspace/tests/worktrees_acceptance.rs` unless a path is given.

### User Story 1: See the worktrees of a repository

- As: a developer who keeps several branches checked out in several folders
- I: can see them under the repository they belong to
- So: I know which folder holds which branch and which one has changes

#### Acceptance Scenarios

Scenario 1: **Folded at first**

- Given: a repository with two other worktrees and a repository with none
- When: the window opens
- Then: the first shows a toggle with the number 2, the second has no toggle, and no worktree row is on screen
- Test: `worktrees_are_folded_until_the_toggle_is_clicked`

Scenario 2: **Open and fold**

- Given: a repository with a worktree
- When: its toggle is clicked twice
- Then: the worktree row is shown after the first click and gone after the second
- Test: `worktrees_are_folded_until_the_toggle_is_clicked`

Scenario 3: **State of a worktree**

- Given: a worktree on its own branch with one untracked file
- When: the repository is read
- Then: the worktree has its own branch, one untracked file and the state "Changes", while the repository stays clean
- Test: `a_worktree_has_its_own_branch_and_state`

Scenario 4: **A worktree whose folder is gone**

- Given: a worktree whose folder was deleted
- When: the repository is read again
- Then: the worktree is listed as a problem, named by its folder, with the one word "Missing" as its state
- Test: `a_worktree_whose_folder_is_gone_is_shown_as_a_problem`, `a_missing_worktree_folder_is_one_word_named_by_its_folder` in `app/src/model/tests/worktrees.rs`

### User Story 2: Move into a worktree

- As: a developer
- I: can select a worktree like a repository
- So: I can read and stage the changes of that folder

#### Acceptance Scenarios

Scenario 1: **Select**

- Given: an open worktree row
- When: it is clicked
- Then: the worktree is selected and the changes column shows the files of that folder
- Test: `selecting_a_worktree_shows_the_changes_of_its_folder`

Scenario 2: **Stage in a worktree**

- Given: a selected worktree with one untracked file
- When: the file is staged
- Then: the worktree row shows one staged file and no untracked file, and the repository row is unchanged
- Test: `staging_in_a_worktree_updates_its_row`

Scenario 3: **Keys**

- Given: a repository with an open worktree and a repository below it
- When: Down is pressed from the repository
- Then: the worktree is selected, then the next repository
- Test: `the_arrow_keys_pass_through_open_worktrees_only`

Scenario 4: **Fold with the worktree selected**

- Given: a selected worktree
- When: its repository is folded
- Then: the repository is selected
- Test: `folding_a_repository_selects_it_when_its_worktree_was_selected`

Scenario 5: **A worktree disappears**

- Given: a selected worktree
- When: the worktree is removed from a terminal and the list is refreshed
- Then: its row is gone and a repository of the group is selected
- Test: `a_removed_worktree_leaves_the_selection_on_a_repository`

Scenario 6: **The detail says whose it is**

- Given: a selected worktree
- When: its changes, history and files are shown
- Then: a bar above them reads "Worktree of <repository>" with the folder and the branch, and it is gone when the repository itself is selected
- Test: `the_detail_of_a_worktree_says_which_repository_it_belongs_to`, `the_detail_of_a_missing_worktree_names_it_without_a_branch` in `app/src/workspace/tests/worktrees_detail_acceptance.rs`

Scenario 7: **A removal seen by the watcher**

- Given: a selected worktree
- When: it is removed from a terminal and only its repository is read again, as the watcher does
- Then: a repository of the group is selected instead of the missing folder
- Test: `a_worktree_removed_while_selected_moves_the_selection_on_a_partial_refresh` in `app/src/workspace/tests/worktrees_detail_acceptance.rs`

### User Story 3: Worktrees follow the files

- As: a developer who edits in a worktree from a terminal or an editor
- I: can see the worktree row change without a refresh
- So: the screen agrees with the real state

#### Acceptance Scenarios

Scenario 1: **A file in a worktree**

- Given: a repository whose worktree is in a sibling folder
- When: a file is created in the worktree
- Then: the worktree shows an untracked file without a refresh
- Test: `a_file_created_in_a_worktree_shows_without_a_refresh`

Scenario 2: **Where events are reported**

- Given: a worktree folder is watched for its repository
- When: a file changes inside it
- Then: the repository is reported, not the worktree folder
- Test: `reports_the_repository_for_a_change_in_its_worktree` in `app/src/watcher/tests.rs`; inner level, not reachable through the window

Scenario 3: **Git state of a worktree**

- Given: the Git entries of a worktree live in the repository's `.git/worktrees` folder
- When: an event arrives for them
- Then: the event is relevant
- Test: `watches_worktree_files_and_git_state` in `app/src/watcher/tests.rs`

Scenario 4: **A removed worktree is not watched any more**

- Given: a repository with two watched worktree folders
- When: one of them is no longer listed
- Then: only the repository and the other folder stay watched
- Test: `a_worktree_folder_that_is_no_longer_listed_is_not_watched_any_more` in `app/src/watcher/tests.rs`; inner level, not reachable through the window

Scenario 6: **A worktree folder that is a repository too**

- Given: a folder watched as a worktree and also registered as a repository
- When: it is no longer listed as a worktree
- Then: it is still watched as a repository
- Test: `a_folder_that_is_also_a_repository_stays_watched_when_it_is_no_longer_a_worktree` in `app/src/watcher/tests.rs`; inner level, not reachable through the window

Scenario 5: **Watching is off**

- Given: the watcher could not start
- When: the repositories are read
- Then: the worktrees are still listed and nothing is watched
- Test: `worktrees_are_listed_when_watching_is_off`

### User Story 4: Reading the list of worktrees

- As: the app
- I: can turn the output of Git into worktrees
- So: the screen shows what Git knows

#### Acceptance Scenarios

Scenario 1: **Parsing**

- Given: the porcelain output with a main worktree, a locked one, a prunable one and a bare one
- When: it is parsed
- Then: each has its path and flags, and the bare one is left out
- Test: `parses_the_porcelain_list` in `core/src/worktree.rs`; inner level, not reachable through the window

Scenario 2: **Real repository**

- Given: a repository with two linked worktrees
- When: it is summarized
- Then: the summary lists the two others and not the folder itself
- Test: `lists_the_other_worktrees_of_a_repository` in `core/tests/summarize.rs`

Scenario 3: **A linked worktree as the repository**

- Given: a worktree folder is the summarized repository
- When: it is summarized
- Then: the main worktree is listed and the folder itself is not
- Test: `a_linked_worktree_lists_the_main_one_and_not_itself` in `core/tests/summarize.rs`

Scenario 4: **No worktrees**

- Given: a repository with a single worktree
- When: it is summarized
- Then: the list is empty
- Test: `a_repository_without_other_worktrees_has_an_empty_list` in `core/tests/summarize.rs`

Scenario 5: **A folder that is gone**

- Given: a worktree whose folder was deleted
- When: the repository is summarized
- Then: the worktree is listed as prunable with an error instead of a summary
- Test: `a_worktree_whose_folder_is_gone_is_listed_with_an_error` in `core/tests/summarize.rs`

Scenario 6: **The list command fails**

- Given: the listing cannot be read
- When: the other worktrees are asked for
- Then: the list is empty and nothing fails
- Test: `a_failed_listing_leaves_the_list_empty` in `core/src/worktree.rs`; inner level, not reachable through the window

Scenario 7: **Entries of a summary**

- Given: a summary with a readable worktree and a missing one
- When: the entries of its worktrees are asked for
- Then: the readable one is ready and the missing one is a failed entry with its path
- Test: `turns_worktrees_into_entries` in `app/src/model/tests.rs`; inner level, not reachable through the window

## Functional Requirements (mandatory)

- FR-1: A repository with other worktrees shows a toggle with their number; one without shows none.
- FR-2: Worktrees are folded when the window opens; the toggle opens and folds them.
- FR-3: A worktree row shows folder name, branch, one-word state and counts.
- FR-4: A worktree can be selected, and then its changes, history and staging are shown.
- FR-5: Up and Down pass through the rows on screen only.
- FR-6: Folding a repository selects it when one of its worktrees was selected.
- FR-7: A change inside a worktree folder or in its Git entries reads its repository again.
- FR-8: A worktree whose folder is gone is shown as a problem.
- FR-9: Naran never creates, removes, locks or prunes a worktree.

## Constraints (mandatory)

- The list comes from `git worktree list --porcelain`; a failure of that command leaves the list empty and never fails the summary.
- Worktrees of the repository are read with the repository, with the same background reading.
- Which repositories are open is kept while the window is open and is not saved.

## Success Criteria (mandatory)

- Every scenario above passes with the test it names.
- `mise run check` passes, with every new line covered.
