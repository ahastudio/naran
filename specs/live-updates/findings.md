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
| Watch only some `.git` entries (`HEAD`, `index`, `refs`, merge/rebase markers) | These change the state shown; large folders such as `objects` and `logs` are ignored (comment on `GIT_ENTRIES`) |
| Match events against resolved paths, report the registered path | Events arrive with resolved paths, such as `/private/var` on macOS (comment on `Root`) |
| Skip failed watcher events | A failed event carries no paths; the periodic tick makes up for it (comment on `route`) |
| Tolerate a poisoned lock on the roots | The list itself stays whole (comment on `locked`) |
| Limit to 8 concurrent `git` reads | `READ_CONCURRENCY`: how many `git` processes run at once when reading every repository |
| Replace only entries that differ; notify only then | `replace_entry` returns true only when something changed |
| Reload details only when the selected repository changed | Doc comment on `refresh_some` |
| Prune repositories no group holds | "They could never be seen again, so they should not be read or watched" (comment on `prune`) |
| List every untracked file separately | "git would otherwise show a new folder as one entry" (comment on `status::ARGUMENTS`) |
| Tell the user when watching fails | The notice says "Use Refresh." |

## Issues Encountered

none recorded

## Learnings

### Simulated clicks finish background reads (2026-11-16)

In a GPUI test, a simulated click or keystroke runs pending background work before it returns.
To observe the reading state, start the refresh directly and look before the executor runs.

### The watcher rule is pinned below the window (2026-11-16)

No window test pins the `.git` folder name in the watcher's relevance rule; `ignores_noise` and `watches_worktree_files_and_git_state` do.
