# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- Edits from a terminal or editor show up without a refresh.
- Only changed repositories are read again.
- Cmd+R and the Refresh button read everything again.
- The window never waits for reading.
- Each repository is summarized with counts and a one-word state.

## Research Findings

### Codebase structure

- `core/` runs the system `git` and knows nothing about the UI; `app/` is the GPUI app (AGENTS.md).
- `app/src/watcher.rs` watches folders and sends changed repository paths through a channel.
- `app/src/workspace/reading.rs` reads repositories on the background executor and merges the results.
- `app/src/model.rs` holds `Entry` and the pure state rules.
- `core/src/lib.rs` has `summarize`, which runs `git status` and `git rev-parse --absolute-git-dir`.

### Existing patterns

- `Entry` is either `Ready(Summary)` or `Failed`, so an unreadable repository does not stop the others.
- State rules are a priority table in `state_words`; the first applying rule wins.
- A debounced burst arrives as a set of paths, so a repository appears once.

## Resources

### Documents

- `README.md`: screen, features and folder layout
- `AGENTS.md`: commands and enforced rules

### Code References

- Debounce, safety net, git entries: `app/src/watcher.rs:16`, `app/src/watcher.rs:19`, `app/src/watcher.rs:22`
- Relevance filter: `app/src/watcher.rs:144`
- Burst collection: `app/src/watcher.rs:182`
- Concurrency limit: `app/src/workspace/reading.rs:16`
- Full refresh: `app/src/workspace/reading.rs:20`
- Partial refresh: `app/src/workspace/reading.rs:45`
- Watcher start-up failure notice: `app/src/workspace/reading.rs:96`
- State rules: `app/src/model.rs:108`
- Summary: `core/src/lib.rs:70`
- Bounded parallel run: `core/src/batch.rs:50`

### API Endpoints

none

## Technical Decisions

| Decision | Rationale |
| -------- | --------- |
| Debounce events per repository (200ms) | "A burst of file writes becomes one report" (watcher.rs header comment) |
| 60 second safety-net tick | Reports again "in case an event was lost"; unchanged repositories cost one `git status` each (comment on `SAFETY_NET`) |
| Watch only some `.git` entries (`HEAD`, `index`, `refs`, merge/rebase markers, and `worktrees` since the worktrees feature) | These change the state shown; large folders such as `objects` and `logs` are ignored (comment on `GIT_ENTRIES`) |
| Match events against the registered and the resolved path, report the registered path | Events carry either path: the resolved one, such as `/private/var` on macOS, or the registered one (comment on `Root`) |
| Skip failed watcher events | A failed event carries no paths; the periodic tick makes up for it (comment on `route`) |
| Tolerate a poisoned lock on the roots | The list itself stays whole (comment on `locked`) |
| Limit to 8 concurrent `git` reads | `READ_CONCURRENCY`: how many `git` processes run at once when reading every repository |
| Replace only entries that differ; notify only then | `replace_entry` returns true only when something changed |
| Reload details only when the selected repository changed | Doc comment on `refresh_some` |
| Prune repositories no group holds | "They could never be seen again, so they should not be read or watched" (comment on `prune`) |
| List every untracked file separately | "git would otherwise show a new folder as one entry" (comment on `status::ARGUMENTS`) |
| Tell the user when watching fails | The notice says "Use Refresh." |
| Keep the picked lines while a diff is read again with the same lines | The picks are indexes into the lines (`Viewing::read`); other lines drop them |
| Clear the picks when a staging change went through | They are used up; a second click would apply old indexes to the next diff (`use_up_picks`) |
| Window tests open without a watcher; `open_watched` for the tests that need it | A late report reads a repository in the middle of a test (`open` in `tests.rs`) |

## Issues Encountered

### Tests failed or waited when the watcher reported late

CI failed in `a_line_added_after_an_unterminated_last_line_stages_with_its_newline` (staged `alpha`, expected `alpha\nmore\n`) and `stacked_diffs_keep_the_name_of_the_file_at_the_top_until_the_next_file` (`a.txt is gone once b.txt has fully come in`).
Every test built its fixture with file writes and then opened a window whose real file watcher reported those writes some time later.
The report ran `refresh_some` in the middle of the test.
A refresh opened the file again and dropped the picked lines, so `apply-lines` staged nothing.
Calling `refresh_some` between picking and applying reproduced the failure on every run; with a refresh before the last scroll the sticky test failed on every run too.
`open` in `app/src/workspace/tests.rs` now drops the watcher, and the seven tests that read from the real watcher or run its watch and unwatch calls use `open_watched`.
`Viewing::read` keeps the picks while a diff is read again with the same lines and drops them when the lines differ.
The sticky test failed for a second reason: GPUI keeps the bounds of drawn elements in two frame buffers and `Frame::clear` does not clear them, so `debug_bounds(..).is_none()` after a repaint depends on how many frames were drawn since the element was last drawn.
One extra frame, such as the one a refresh causes, makes the stale bounds of `sticky-head-0` visible, although only one sticky head is drawn per frame.
Without a watcher in `open`, only tests that use `open_watched` or the watcher tests run the watch and unwatch lines, which coverage requires.
The test asserts the head that is drawn (`sticky-head-<second>` at the top of the list), which holds with or without a refresh before the last scroll.

## Learnings

### Simulated clicks finish background reads (2026-11-16)

In a GPUI test, a simulated click or keystroke runs pending background work before it returns.
To observe the reading state, start the refresh directly and look before the executor runs.

### The watcher rule is pinned below the window (2026-11-16)

No window test pins the `.git` folder name in the watcher's relevance rule; `ignores_noise` and `watches_worktree_files_and_git_state` do.
