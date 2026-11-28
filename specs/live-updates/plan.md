# Implementation Plan: Live Updates

## Summary

The feature is built Outside-In with stubs: acceptance tests that drive the window are written first and fail, the outermost part is built with everything below it stubbed, and each next step builds one part inward and replaces the stub the step before it used.
A file watcher reports changed repositories after a short quiet period, the workspace reads only those in the background, and the core library builds each summary that the app turns into a one-word state.

## Requirements

1. Read everything and start watching at startup (FR-1)
2. Read only changed repositories, with grouping of bursts, ignore rules and a 60 second safety net (FR-2 to FR-6)
3. Refresh button and Cmd+R, disabled while reading (FR-7, FR-11)
4. Background reading (FR-8)
5. Status summary and one-word state (FR-9, FR-10)

## Critical Files

### New Files

- `app/src/watcher.rs`: file watching, per-repository grouping, quiet-period delay, safety-net tick
- `app/src/watcher/tests.rs`
- `app/src/workspace/reading.rs`: background reading, full and partial refresh, watcher wiring, dropping repositories no group holds
- `core/src/status.rs`: parsing of the `git status` output
- `core/src/batch.rs`: parallel runs with a limit

### Modified Files

- `app/src/model.rs`: the repository entry and the one-word state rules
- `core/src/lib.rs`: the repository summary and in-progress operation detection
- `app/src/workspace/chrome.rs`: the Refresh button and its action

### Reference Files

- `app/src/workspace.rs`: the Cmd+R key binding and the startup sequence
- `app/src/watcher/tests.rs`, `app/src/workspace/tests/notices.rs`, `app/src/workspace/tests/controls.rs`
- `app/src/model/tests.rs`, `core/tests/summarize.rs`, `core/tests/reading.rs`

## Architecture

### User Flow

```text
Window opens → read all repositories + start watching
Edit a file in a terminal or editor → that repository is read again → list updates
Refresh button or Cmd+R → read all again (button shows "Reading…")
```

### Event Flow

```text
1. File system event
       ↓
2. Map to a watched repository, drop ignored paths
       ↓
3. Wait for 200ms of quiet, then send the set of repositories
       ↓
4. Workspace receives the set and starts a background read
       ↓
5. Background read: summarize each repository (at most 8 at once)
       ↓
6. Replace only the entries that differ; reload details if the selected one changed
(the 60 second tick skips step 2 and sends every repository to step 3)
```

### Domain Model

```text
Repository entry
├── Ready  ← path, name, status, operation in progress
│     └── Status: branch, head, upstream, ahead, behind,
│                 staged, unstaged, untracked, conflicted
└── Failed ← path, error
One-word state → (word, urgency: problem | attention | calm)
```

## Implementation Steps

The work starts with the acceptance tests, then goes strictly from the outside in: window, workspace, watcher, app model, core.
Every test is named in exactly one step.

### Step 0: Acceptance Tests

**Red:** Write the tests that assemble the whole workspace on real repositories and check a user scenario end to end.
They are every test under `app/src/workspace/tests/` that belongs to this feature; the feature has none in `app/src/tests.rs`.
They are written first and fail until the last step.
Outside changes (User Story 1): `shows_changes_made_outside_the_app_without_a_refresh` in `app/src/workspace/tests/notices.rs` (scenario 1), `a_burst_of_writes_ends_with_every_file_counted`, `ignored_files_are_not_counted_but_the_next_real_change_is` and `a_registered_folder_that_is_gone_is_listed_as_failed_and_the_rest_still_work` in `app/src/workspace/tests/live_updates_acceptance.rs` (scenarios 2 to 4), `changes_stop_being_read_once_the_window_is_gone` in `app/src/workspace/tests/controls.rs` (scenario 7), `watching_that_cannot_start_is_reported_and_leaves_nothing_watched` in `notices.rs` (scenario 8).
Refreshing by hand (User Story 2): `refresh_reads_again_from_the_button_and_from_the_keys` in `controls.rs` (scenario 1), `the_refresh_button_waits_while_reading_and_returns_after`, `refreshing_twice_in_a_row_reads_the_latest_state_each_time` and `a_group_without_repositories_has_nothing_to_read` in `app/src/workspace/tests/live_updates_acceptance.rs` (scenarios 2 to 4), `reads_every_repository_and_selects_the_first` and `drops_a_repository_that_leaves_its_last_group` in `app/src/workspace/tests/groups.rs` (scenarios 5 and 6), `an_entry_is_replaced_only_when_it_is_known_and_different` in `notices.rs` (scenario 7).
States on screen (User Story 3): `the_state_shows_the_most_urgent_word_after_a_refresh`, `behind_then_diverged_follow_the_upstream_after_refreshes`, `ahead_shows_after_a_local_commit_and_changes_outrank_it` and `a_repository_that_cannot_be_read_shows_its_error_as_a_problem` in `app/src/workspace/tests/live_updates_acceptance.rs` (scenarios 1 to 5).

**Green:** They keep failing until the last step and pass when it is done; no code is written for them directly.

**Refactoring:** The acceptance file shares small helpers that read an entry's state and untracked count, wait for the watcher, and refresh with the keys.

### Step 1: Screen - the window

**Red:** The feature has no tests of its own for rows, widgets or drawing; this part is built against the Step 0 tests, using a stub of the Workspace that returns a fixed list of repositories.

**Green:** Build this part with the parts below it stubbed; this is the first real part.
The toolbar button and the Cmd+R binding start a full read, and the button is disabled while reading.
A file written from outside reaches the list and each entry shows its one-word state.
The files are `app/src/workspace/chrome.rs`, `app/src/workspace.rs` and `app/src/workspace/reading.rs`.

**Refactoring:** Keep the button state in one small function so the toolbar stays short.

### Step 2: Workspace - reading and applying results

**Red:** The feature's workspace tests assemble the whole workspace, so they are in Step 0 and this part has no tests of its own; it is built against them, using a stub of the watcher and a stub that returns a fixed summary for each repository.

**Green:** Build this part with the watcher and the summary stubbed, and replace the Step 1 stub with this real part.
The workspace reads repositories in the background, replaces only the entries that differ, reloads details only when the selected repository changed, tells the user when watching cannot start, and drops repositories that no group holds.
The file is `app/src/workspace/reading.rs`.

**Refactoring:** Split reading, merging and pruning into small functions that fit the repo's size limits.

### Step 3: Platform - the watcher

**Red:** Write watcher tests in `app/src/watcher/tests.rs`, using a stub that stands in for the file system events and a channel that collects the reports: `reports_a_burst_of_writes_once_with_the_registered_path`, `debounce_groups_a_burst_and_stops_when_either_side_is_gone`, `watches_worktree_files_and_git_state`, `ignores_noise`, `routes_relevant_events_to_their_repository_and_skips_failed_ones`, `the_tick_reports_every_repository_until_nobody_listens`, `a_folder_that_cannot_be_watched_is_left_out_and_keeps_its_own_name`.
These cover User Story 1 scenarios 2 to 6, the ones marked inner level.

**Green:** Build this part with the file events stubbed, and replace the Step 2 stub of the watcher with this real part.
The watcher registers each repository folder, keeps only relevant events, groups a burst into one report per repository, and adds a periodic tick.
Relevant means a non-ignored work folder file or a listed set of entries inside `.git`.
The file is `app/src/watcher.rs`.

**Refactoring:** Keep path matching, grouping and ticking in separate functions.

### Step 4: Model - one-word state

**Red:** Write model tests in `app/src/model/tests.rs`, using stubs that return fixed statuses and operations: `describes_the_most_urgent_state_first` and `counts_changes_and_spots_problems`.
These cover User Story 3 scenarios 1 and 6.

**Green:** Build this part with the summary stubbed, and replace the Step 2 stub of the summary with this real part.
The app picks the most urgent word from the repository summary by a fixed priority, and counts changes and problems.
The file is `app/src/model.rs`.

**Refactoring:** Keep the priority as a table so the order is visible.

### Step 5: Core - the repository summary

**Red:** Write core tests, which run on real repositories and fixed `git status` text: `counts_changes`, `detects_merge_conflict`, `reports_non_repository` and `every_file_in_a_new_folder_is_listed_and_counted` in `core/tests/summarize.rs`; `empty_repository`, `branch_with_upstream`, `detached_head`, `changes` and `an_ahead_behind_part_that_is_neither_is_skipped` in `core/src/status.rs`; `keeps_input_order_and_runs_everything` and `handles_no_paths` in `core/src/batch.rs`.
These cover User Story 3 scenarios 7 to 9.

**Green:** This is the innermost step, so no stub is left: the last stub is replaced and the Step 0 tests pass.
The core library runs `git status`, parses branch, counts and tracking, and detects an operation in progress from markers in the git folder.
Batch reading runs many repositories in parallel with a limit and keeps the input order.
The files are `core/src/lib.rs`, `core/src/status.rs` and `core/src/batch.rs`.

**Refactoring:** Keep parsing in small functions, one per header or record kind.

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

1. Open the app with `cargo run -p naran-app` and create a file in a repository from a terminal.
2. Check that the state becomes "Changes" without a refresh.
3. Press Cmd+R and watch the Refresh button.

## Considerations

### Reusing Existing Code

- A partial refresh reuses the same selection reload as a full refresh, so the selected repository's details stay current.

### Compatibility

- The watcher uses the platform's recommended file watching.
- Comments in `app/src/watcher.rs` explain why matching accepts both the registered and the resolved path (on macOS `/var` becomes `/private/var`).
