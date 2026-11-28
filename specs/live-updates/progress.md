# Progress Log

> **Update when a phase is finished or a problem occurs.**

## Session 2026-11-16

### Phase 1: Requirements & Discovery ✅

**Work log**:

1. Requirements were set: outside changes show at once, only changed repositories are read again, manual refresh, one-word state.
2. The watcher, background reading, state rules and core summary were analyzed with their tests.
3. The spec was written as 3 user stories with numbered scenarios, each tied to a test.

**Created/Modified files**:

- `specs/live-updates/spec.md`

### Phase 2: Planning & Structure ✅

**Work log**:

1. The plan was written Outside-In in five steps: window, workspace, watcher, app model, core.
2. Each step names the failing tests that drive it, and every scenario of the spec is covered by one of them.

**Created/Modified files**:

- `specs/live-updates/plan.md`
- `specs/live-updates/README.md`

### Phase 3: Implementation ✅

**Work log**:

1. Step 0: seventeen workspace tests assemble the whole workspace; ten are new in `app/src/workspace/tests/live_updates_acceptance.rs` (registered in `app/src/workspace/tests.rs`) and pass against the existing code; seven are in `notices.rs`, `controls.rs` and `groups.rs`; the refresh-button test starts the refresh directly because a simulated click lets the read finish at once.
2. Step 1: the window has no tests of its own; code in `app/src/workspace/chrome.rs` and `app/src/workspace.rs`.
3. Step 2: the Workspace has no tests of its own; code in `app/src/workspace/reading.rs`.
4. Step 3: watcher tests in `app/src/watcher/tests.rs`; code in `app/src/watcher.rs`.
5. Step 4: model tests in `app/src/model/tests.rs`; code in `app/src/model.rs`.
6. Step 5: core tests in `core/tests/summarize.rs`, `core/src/status.rs` and `core/src/batch.rs`; code in `core/src/lib.rs`, `core/src/status.rs` and `core/src/batch.rs`.

**Created/Modified files**:

- `app/src/workspace/tests/live_updates_acceptance.rs` (new)
- `app/src/workspace/tests.rs` (one appended module line)
- `specs/live-updates/tasks.md`
- `specs/live-updates/findings.md`

### Phase 4: Testing ✅

**Work log**:

1. The acceptance tests and the existing feature tests of the app and core were run.
2. Results are in the table below.
3. The picks stay across a read of the same diff, and window tests open without a watcher unless they read from it; the causes are in Issues Encountered of `findings.md`.

**Created/Modified files**:

- `app/src/workspace/detail/viewing.rs`
- `app/src/workspace/tests.rs`
- `app/src/workspace/tests/live_updates_acceptance.rs`
- `app/src/workspace/tests/{notices,groups,adding,worktrees_acceptance}.rs` (`open_watched`)
- `app/src/watcher/tests.rs`
- `app/src/workspace/tests/sticky_names.rs`

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `shows_changes_made_outside_the_app_without_a_refresh` | new file written from outside | No upstream becomes Changes | passed | ✅ |
| `a_burst_of_writes_ends_with_every_file_counted` | 10 files written in a row to one repository | 10 untracked, state Changes | passed | ✅ |
| `ignored_files_are_not_counted_but_the_next_real_change_is` | an ignored `.log` file, then `real.txt` | 1 untracked | passed | ✅ |
| `a_registered_folder_that_is_gone_is_listed_as_failed_and_the_rest_still_work` | alpha folder deleted before the window opens | alpha Failed, beta Changes, gamma No upstream | passed | ✅ |
| `changes_stop_being_read_once_the_window_is_gone` | reported change, closed window | loop ends | passed | ✅ |
| `watching_that_cannot_start_is_reported_and_leaves_nothing_watched` | watcher start error | no watcher, notice has "Could not watch" | passed | ✅ |
| `refresh_reads_again_from_the_button_and_from_the_keys` | button click and Cmd+R after a new file | not reading, 2 untracked in beta | passed | ✅ |
| `the_refresh_button_waits_while_reading_and_returns_after` | refresh started | reading on, then off after the read | passed | ✅ |
| `refreshing_twice_in_a_row_reads_the_latest_state_each_time` | a file, Cmd+R, another file, Cmd+R | 1 untracked, then 2 | passed | ✅ |
| `a_group_without_repositories_has_nothing_to_read` | group with no repositories, Cmd+R | no entries, not reading | passed | ✅ |
| `reads_every_repository_and_selects_the_first` | open the window | 3 entries, first selected, details loaded, beta Changes | passed | ✅ |
| `drops_a_repository_that_leaves_its_last_group` | remove alpha from its only group | not listed or saved, folder remains | passed | ✅ |
| `an_entry_is_replaced_only_when_it_is_known_and_different` | unknown, same, changed entry | replaced only when known and different | passed | ✅ |
| `the_state_shows_the_most_urgent_word_after_a_refresh` | conflicting merge, then Cmd+R | beta Changes, gamma Merging | passed | ✅ |
| `behind_then_diverged_follow_the_upstream_after_refreshes` | branch tracking main; commit on main; commit on branch | Clean, Behind, Diverged | passed | ✅ |
| `ahead_shows_after_a_local_commit_and_changes_outrank_it` | local commit, then untracked file | Ahead, then Changes | passed | ✅ |
| `a_repository_that_cannot_be_read_shows_its_error_as_a_problem` | alpha folder deleted, Cmd+R | alpha Failed with problem tone, others Ready | passed | ✅ |
| `a_change_report_that_leaves_the_diff_alone_keeps_the_picked_lines` | one line picked, change report, Stage lines | 1 pick kept, staged `alpha\nmore\n` | passed | ✅ |
| `a_change_report_that_changes_the_diff_drops_the_picked_lines` | one line picked, file rewritten, change report | no picks | passed | ✅ |
| `reports_a_burst_of_writes_once_with_the_registered_path` | 5 files written | one report with the registered path; watch twice adds nothing | passed | ✅ |
| `debounce_groups_a_burst_and_stops_when_either_side_is_gone` | /a, /a, /b | one burst of /a and /b; ends when a side is gone | passed | ✅ |
| `watches_worktree_files_and_git_state` | source file and `.git` HEAD, index, refs | relevant | passed | ✅ |
| `ignores_noise` | ignored paths, `.git/objects`, lock file, outside path | not relevant | passed | ✅ |
| `routes_relevant_events_to_their_repository_and_skips_failed_ones` | failed event, mixed paths | only the work folder file reported | passed | ✅ |
| `the_tick_reports_every_repository_until_nobody_listens` | 1ms tick | first report is the repository, loop ends | passed | ✅ |
| `a_folder_that_cannot_be_watched_is_left_out_and_keeps_its_own_name` | missing folder | no root added | passed | ✅ |
| `describes_the_most_urgent_state_first` | statuses and operations | words by priority; failed entry is a problem | passed | ✅ |
| `counts_changes_and_spots_problems` | staged 1, unstaged 2, untracked 3; conflict; failure | 6 changes; problems spotted | passed | ✅ |
| `changes` (`core/src/status.rs`) | mixed records | staged 3, unstaged 2, conflicted 1, untracked 1 | passed | ✅ |
| `counts_changes` (`core/tests/summarize.rs`) | staged, unstaged, untracked files | counts 1, 1, 1, no operation | passed | ✅ |
| `detects_merge_conflict` | conflicting merge | conflicted 1, operation Merge | passed | ✅ |
| `reports_non_repository` | plain folder | command error | passed | ✅ |
| `every_file_in_a_new_folder_is_listed_and_counted` | new folder with 2 files | 2 untracked | passed | ✅ |
| `empty_repository` | header with initial oid | branch main, no head, no upstream | passed | ✅ |
| `branch_with_upstream` | upstream and ab headers | head, upstream, ahead 2, behind 3 | passed | ✅ |
| `detached_head` | detached header | no branch | passed | ✅ |
| `an_ahead_behind_part_that_is_neither_is_skipped` | `+3 ? -4` | ahead 3, behind 4 | passed | ✅ |
| `keeps_input_order_and_runs_everything` | 20 paths, limit 4 | 20 results in input order | passed | ✅ |
| `handles_no_paths` | no paths | empty result | passed | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| none recorded | CI run: `a_line_added_after_an_unterminated_last_line_stages_with_its_newline` staged `alpha`; `stacked_diffs_keep_the_name_of_the_file_at_the_top_until_the_next_file` failed on "a.txt is gone once b.txt has fully come in" | 1 | Reproduced with `refresh_some` between the steps; picks stay across a read of the same diff, `open` has no watcher, the sticky test asserts the head that is drawn |
| none recorded | `mise run check`: coverage missed lines of `watcher.rs`, `membership.rs` and `reading.rs` after `open` dropped the watcher | 1 | The tests that watch, unwatch and add repositories use `open_watched`; the worktree watcher test removes the folder before `keep_linked` |

## 5-Question Reboot Check

Questions to recover context when work resumes:

| Question | Answer |
| -------- | ------ |
| 1. Which phase is current? | Phase 4: Testing, done |
| 2. What is next? | None (all done) |
| 3. What is the goal? | Changes made outside the app show at once, reading only changed repositories in the background |
| 4. What was learned? | Simulated clicks finish background reads; no window test pins the watcher rule; see findings.md |
| 5. What was completed? | Spec, Outside-In plan, twelve window-driven acceptance tests, all listed results passing |
