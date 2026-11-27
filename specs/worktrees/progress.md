# Progress Log

> **Update when a phase is finished or a problem occurs.**

## Session 2026-11-25

### Phase 1: Requirements & Discovery ✅

**Work log**:

1. Scoped the feature to viewing and moving worktrees, folded under the repository row.
2. The summary, the entries, the repository column, the selection and the watcher were read to see where worktrees fit.
3. The spec was written as 4 user stories with numbered scenarios, each tied to a test.
4. A worktree whose folder is gone shows the state "Missing" with the folder name, and each worktree row has its own bottom line. `mise run check` passed.

**Created/Modified files**:

- `specs/worktrees/spec.md`
- `specs/worktrees/README.md`

### Phase 2: Planning & Structure ✅

**Work log**:

1. The plan was written Outside-In in five steps after the acceptance tests: screen, workspace, watcher, app model, core.
2. Each step names the tests that drive it, and every scenario of the spec is covered by one of them.

**Created/Modified files**:

- `specs/worktrees/plan.md`

### Phase 3: Implementation ✅

**Work log**:

1. Step 0: ten workspace tests in `app/src/workspace/tests/worktrees_acceptance.rs` (registered in `app/src/workspace/tests.rs`) use real linked worktrees of the fixture repositories and drive the toggle, the rows, the keys and the staging lists.
2. Step 1: the screen has no tests of its own; code in `app/src/widgets/worktree.rs`, `app/src/widgets.rs` and `app/src/workspace/repositories.rs`.
3. Step 2: the workspace has no tests of its own; code in `app/src/workspace/worktrees.rs`, `app/src/workspace/navigation.rs`, `app/src/workspace/reading.rs`, `app/src/workspace/membership.rs`, `app/src/workspace/detail/staging.rs` and `app/src/workspace/state.rs`.
4. Step 3: watcher tests in `app/src/watcher/tests.rs`; code in `app/src/watcher.rs`.
5. Step 4: model test in `app/src/model/tests/worktrees.rs`; code in `app/src/model.rs`.
6. Step 5: core tests in `core/src/worktree.rs` and `core/tests/summarize.rs` (helper in `core/tests/common/mod.rs`); code in `core/src/worktree.rs` and `core/src/lib.rs`.

**Created/Modified files**:

- `core/src/worktree.rs` (new), `core/src/lib.rs`
- `app/src/model.rs`, `app/src/model/tests/worktrees.rs` (new), `app/src/model/tests.rs`
- `app/src/watcher.rs`, `app/src/watcher/tests.rs`
- `app/src/widgets/worktree.rs` (new), `app/src/widgets.rs`
- `app/src/workspace/worktrees.rs` (new), `app/src/workspace.rs`, `app/src/workspace/state.rs`, `app/src/workspace/repositories.rs`, `app/src/workspace/navigation.rs`, `app/src/workspace/reading.rs`, `app/src/workspace/membership.rs`, `app/src/workspace/detail/staging.rs`
- `app/src/workspace/tests/worktrees_acceptance.rs` (new), `app/src/workspace/tests.rs`
- `core/tests/summarize.rs`, `core/tests/common/mod.rs`
- `specs/worktrees/tasks.md`, `specs/worktrees/findings.md`

### Phase 4: Testing ✅

**Work log**:

1. `cargo test --workspace` passed, and `format`, `lint`, `docs`, `quality` and `coverage` of `mise run check` passed.
2. Manual end-to-end check on real repositories: `alpha` with two linked worktrees (`feature` with an untracked file, `hotfix` with a modified file) and `beta`: the folded toggle showed `2`; a click opened the worktrees with their own branch and counts; a click on a worktree showed its title, branch and its file in Unstaged; dragging the file to Staged ran `git add` (`git status` in the worktree said `A  wt.txt`) and the row read `1 staged` while `alpha` stayed unchanged; files written outside the app showed in the row without a refresh (`1 modified 2 untracked`); with a worktree selected, `git worktree remove` and a click on Refresh dropped its row, the toggle read `1` and `alpha` was selected.

**Created/Modified files**:

- `specs/worktrees/progress.md`

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `worktrees_are_folded_until_the_toggle_is_clicked` | alpha with two worktrees, beta with none; toggle clicked twice, beta's column clicked | 3 rows, then 5, then 3; beta's column does nothing | passed | ✅ |
| `a_worktree_has_its_own_branch_and_state` | worktree on `feature` with one untracked file | branch feature, 1 untracked, Changes; alpha No upstream | passed | ✅ |
| `a_worktree_whose_folder_is_gone_is_shown_as_a_problem` | worktree folder deleted before the window opens | listed as a failed entry with a problem, row on screen when open | passed | ✅ |
| `selecting_a_worktree_shows_the_changes_of_its_folder` | open the toggle, click the worktree row | worktree selected, one change `wt.txt` | passed | ✅ |
| `staging_in_a_worktree_updates_its_row` | drag `wt.txt` to Staged in a selected worktree | `A  wt.txt`, worktree 1 staged and 0 untracked, alpha unchanged, selection kept | passed | ✅ |
| `the_arrow_keys_pass_through_open_worktrees_only` | Down while folded, then open and Down, Down, Up, Up | beta; then worktree, beta, alpha | passed | ✅ |
| `folding_a_repository_selects_it_when_its_worktree_was_selected` | fold with the worktree selected | alpha selected, folded | passed | ✅ |
| `a_removed_worktree_leaves_the_selection_on_a_repository` | `git worktree remove`, Cmd+R | no worktrees, not on screen, alpha selected | passed | ✅ |
| `a_file_created_in_a_worktree_shows_without_a_refresh` | file written in a sibling worktree folder | 1 untracked in the worktree entry | passed | ✅ |
| `worktrees_are_listed_when_watching_is_off` | watcher removed, refresh | worktree still listed, nothing watched | passed | ✅ |
| `reports_the_repository_for_a_change_in_its_worktree` | file written in a worktree folder watched for its repository, folder watched twice | one report with the repository path, 2 roots | passed | ✅ |
| `watches_worktree_files_and_git_state` | source file, `.git` HEAD, index, refs, `.git/worktrees/feature/index` | relevant | passed | ✅ |
| `turns_worktrees_into_entries` | summary with a readable and a missing worktree; failed entry | ready entry, failed entry with its path, count 2; none for a failed entry | passed | ✅ |
| `a_missing_worktree_folder_is_one_word_named_by_its_folder` | failed entry with the missing-folder error, and one with another error | state "Missing", name is the folder; other errors keep Git's wording | passed | ✅ |
| `parses_the_porcelain_list` | main, locked, prunable, bare and a stray block | four blocks with their flags, stray block skipped | passed | ✅ |
| `a_failed_listing_leaves_the_list_empty` | folder that does not exist | empty list | passed | ✅ |
| `lists_the_other_worktrees_of_a_repository` | repository with one linked worktree and an untracked file in it | one worktree on `feature`, 1 untracked, no nested list, repository clean | passed | ✅ |
| `a_linked_worktree_lists_the_main_one_and_not_itself` | the linked worktree summarized | the main folder listed once | passed | ✅ |
| `a_repository_without_other_worktrees_has_an_empty_list` | single worktree | empty list | passed | ✅ |
| `a_worktree_whose_folder_is_gone_is_listed_with_an_error` | linked folder deleted | prunable and an error summary | passed | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| none recorded | none recorded | 0 | none recorded |

## 5-Question Reboot Check

Questions to recover context when work resumes:

| Question | Answer |
| -------- | ------ |
| 1. Which phase is current? | Phase 4: Testing, done; every task is ticked |
| 2. What is next? | None (all done) |
| 3. What is the goal? | Show the worktrees of a repository folded under its row and let the user select them like repositories |
| 4. What was learned? | Bounds of rows drawn earlier stay in the test window; Git prints resolved paths; a removed worktree keeps its watch until the window closes |
| 5. What was completed? | Spec, Outside-In plan, ten window tests, watcher, model and core tests, all listed results passing |
