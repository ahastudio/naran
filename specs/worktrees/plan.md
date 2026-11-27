# Implementation Plan: Worktrees

## Summary

The feature is built Outside-In with stubs: acceptance tests that drive the window are written first and fail, the outermost part is built with everything below it stubbed, and each next step builds one part inward and replaces the stub the step before it used.
Core lists the other worktrees of a repository inside its summary, the app turns them into entries, the watcher reports a repository for changes in its worktree folders, and the repository column folds them under the repository row.

## Requirements

1. A toggle with the number of worktrees, folded at first (FR-1, FR-2)
2. A worktree row with the same facts as a repository row (FR-3, FR-8)
3. Selecting a worktree and moving through the rows on screen (FR-4 to FR-6)
4. Worktree folders and their Git entries are watched for their repository (FR-7)
5. Reading only, never changing a worktree (FR-9)

## Critical Files

### New Files

- `core/src/worktree.rs`: parsing of the porcelain list and the other worktrees of a repository
- `app/src/workspace/worktrees.rs`: which repositories are open, the rows on screen, selecting, folding, and who owns a worktree
- `app/src/widgets/worktree.rs`: the indented worktree row
- `app/src/workspace/tests/worktrees_acceptance.rs`

### Modified Files

- `core/src/lib.rs`: the summary carries the other worktrees
- `app/src/model.rs`: the worktrees of an entry as entries
- `app/src/watcher.rs`: watching a worktree folder for its repository, and the `worktrees` Git entry
- `app/src/workspace/repositories.rs`: the toggle and the rows under a repository
- `app/src/workspace/navigation.rs`: stepping and keeping the selection over the rows on screen
- `app/src/workspace/reading.rs`: reading and watching for the selected worktree's repository
- `app/src/workspace/detail/staging.rs`: the repository to read again after staging

### Reference Files

- `app/src/widgets.rs`: the repository row whose parts the worktree row reuses
- `app/src/workspace/tests/live_updates_acceptance.rs`, `app/src/workspace/tests/fixtures.rs`
- `core/tests/summarize.rs`, `core/tests/common`

## Architecture

### User Flow

```text
Window opens → each repository is read with its other worktrees → rows folded
Click the toggle → worktree rows open below the repository
Click a worktree row → changes, history and staging of that folder
Edit a file in a worktree → its repository is read again → the worktree row follows
```

### Event Flow

```text
1. Event in a worktree folder or in the worktrees folder of the Git entries
       ↓
2. Map to the repository the folder belongs to, drop ignored paths
       ↓
3. The existing quiet period, then the repository is read again with its worktrees
       ↓
4. Entries are replaced; the selected worktree reloads when its repository was read
```

### Domain Model

```text
Summary (repository)
├── status, operation
└── other worktrees ← path, locked, prunable, and a summary or an error
Entry of a repository → entries of its worktrees (ready or failed)
```

## Implementation Steps

The work starts with the acceptance tests, then goes strictly from the outside in: screen, workspace, watcher, app model, core.
Every test is named in exactly one step.

### Step 0: Acceptance Tests

**Red:** Write the tests that assemble the whole workspace on real repositories with real linked worktrees and check a user scenario end to end.
They are in `app/src/workspace/tests/worktrees_acceptance.rs`; the feature has none in `app/src/tests.rs`.
They are written first and fail until the last step.
Seeing the worktrees (User Story 1): `worktrees_are_folded_until_the_toggle_is_clicked`, `a_worktree_has_its_own_branch_and_state` and `a_worktree_whose_folder_is_gone_is_shown_as_a_problem`.
Moving into a worktree (User Story 2): `selecting_a_worktree_shows_the_changes_of_its_folder`, `staging_in_a_worktree_updates_its_row`, `the_arrow_keys_pass_through_open_worktrees_only`, `folding_a_repository_selects_it_when_its_worktree_was_selected` and `a_removed_worktree_leaves_the_selection_on_a_repository`.
Knowing whose worktree it is (User Story 2): `the_detail_of_a_worktree_says_which_repository_it_belongs_to`, `the_detail_of_a_missing_worktree_names_it_without_a_branch` and `a_worktree_removed_while_selected_moves_the_selection_on_a_partial_refresh`, in `app/src/workspace/tests/worktrees_detail_acceptance.rs`.
Following the files (User Story 3): `a_file_created_in_a_worktree_shows_without_a_refresh` and `worktrees_are_listed_when_watching_is_off`.

**Green:** They keep failing until the last step and pass when it is done; no code is written for them directly.

**Refactoring:** The acceptance file shares small helpers that add a worktree to a fixture repository and read the worktree entries of a repository.

### Step 1: Screen - the repository column

**Red:** The feature has no tests of its own for rows, widgets or drawing; this part is built against the Step 0 tests, using a stub of the Workspace that answers with a fixed list of rows.

**Green:** Build this part with the parts below it stubbed; this is the first real part.
A repository row gets a toggle with an arrow and a number when its entry has worktrees.
Open repositories get the worktree rows below them, indented, with name, branch, state and counts.
The files are `app/src/workspace/repositories.rs` and `app/src/widgets/worktree.rs`.

**Refactoring:** The worktree row reuses the title line, the details line and the problem marker of the repository row.

### Step 2: Workspace - opening, selecting and stepping

**Red:** The feature's workspace tests assemble the whole workspace, so they are in Step 0 and this part has no tests of its own; it is built against them, using a stub of the watcher and a stub that returns fixed entries for worktrees.

**Green:** Build this part with the watcher and the entries stubbed, and replace the Step 1 stub with this real part.
The workspace keeps which repositories are open, lists the rows on screen for the arrow keys and scrolling, keeps a selected worktree while its repository is in the group, selects the repository when it is folded, reads the owner of a worktree again after staging, and reloads the selection when the owner was read.
The files are `app/src/workspace/worktrees.rs`, `app/src/workspace/navigation.rs`, `app/src/workspace/reading.rs` and `app/src/workspace/detail/staging.rs`.

**Refactoring:** Keep the rows on screen and the rows that belong to the group as two small functions that share one walk over the entries.

### Step 3: Platform - the watcher

**Red:** Write watcher tests in `app/src/watcher/tests.rs`, using a stub that stands in for the file system events and a channel that collects the reports: `reports_the_repository_for_a_change_in_its_worktree`, `watches_worktree_files_and_git_state`, `a_worktree_folder_that_is_no_longer_listed_is_not_watched_any_more` and `a_folder_that_is_also_a_repository_stays_watched_when_it_is_no_longer_a_worktree`.
They cover User Story 3 scenarios 2, 3, 4 and 6.

**Green:** Build this part with the file events stubbed, and replace the Step 2 stub of the watcher with this real part.
A worktree folder is watched as a root that reports its repository, and the `worktrees` entry of the Git folder counts as Git state.
The file is `app/src/watcher.rs`, and the workspace asks to watch every worktree after each read.

**Refactoring:** A root keeps the path it reports apart from the folder it matches.

### Step 4: Model - worktrees as entries

**Red:** Write a model test in `app/src/model/tests.rs`, using stubs that return fixed summaries: `turns_worktrees_into_entries` and `a_missing_worktree_folder_is_one_word_named_by_its_folder` (`app/src/model/tests/worktrees.rs`).
It covers User Story 4 scenario 7.

**Green:** Build this part with the summary stubbed, and replace the Step 2 stub of the entries with this real part.
An entry gives the entries of its worktrees, and a missing worktree is a failed entry with its path.
The file is `app/src/model.rs`.

**Refactoring:** Keep the conversion in one small function next to the entry.

### Step 5: Core - the other worktrees

**Red:** Write core tests: `parses_the_porcelain_list` in `core/src/worktree.rs`, which runs on fixed text, and `lists_the_other_worktrees_of_a_repository`, `a_linked_worktree_lists_the_main_one_and_not_itself`, `a_repository_without_other_worktrees_has_an_empty_list` and `a_worktree_whose_folder_is_gone_is_listed_with_an_error` in `core/tests/summarize.rs`, which run on real repositories.
`a_failed_listing_leaves_the_list_empty` in `core/src/worktree.rs` runs on a folder that is not a repository.
They cover User Story 4 scenarios 1 to 6.

**Green:** This is the innermost step, so no stub is left: the last stub is replaced and the Step 0 tests pass.
The core library runs the list command, parses path and flags, leaves out the bare entry and the folder itself, and summarizes every other worktree.
A failure of the list command leaves the list empty.
The files are `core/src/worktree.rs` and `core/src/lib.rs`.

**Refactoring:** Keep parsing in small functions, one per line kind.

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

1. Run `git worktree add ../example-feature` in a repository that is in a group, then open the app with `cargo run -p naran-app`.
2. Check that the repository row shows an arrow and the number 1, then click the arrow.
3. Click the worktree row and check that the columns on the right show that folder.
4. Create a file in the worktree from a terminal and check that its row changes without a refresh.

## Considerations

### Reusing Existing Code

- A worktree is read by the same summary as a repository, and shown by the same row parts.
- Details, staging and history already work on any folder, so they need no change for a worktree.

### Compatibility

- Without `git worktree`, or when the command fails, the list is empty and the screen is as before.
- Which repositories are open is not saved; this is the first version of the screen and the design is expected to change.
