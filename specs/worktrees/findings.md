# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- Worktrees are shown with the repository they belong to, folded under its row.
- Only reading and moving: a worktree can be selected, never created, removed, locked or pruned.
- The design of the screen is a first version; the user will look at it and change it.

## Research Findings

### Codebase structure

- `core/` runs the system `git` and knows nothing about the UI; `app/` is the GPUI app (AGENTS.md).
- A repository is read by `summarize` in `core/src/lib.rs`; the app keeps one `Entry` per registered repository in `Workspace::entries`.
- Details, history and staging run `git` in the selected path, so they work in a worktree folder as they are.
- The watcher matches an event to a root by folder and reports the registered path of the root.

### Existing patterns

- The repository column is one flat list, and the arrow keys and scrolling count its children, so worktree rows can be children of the same list.
- A failed read is an `Entry::Failed` that keeps its path, so a missing worktree needs no new kind of row.

## Resources

### Documents

- `README.md`: screen, features and folder layout
- `AGENTS.md`: commands and enforced rules

### Code References

- The other worktrees and the porcelain parser: `core/src/worktree.rs`
- The summary that carries them: `core/src/lib.rs`
- Worktrees as entries: `app/src/model.rs`
- Rows on screen, owner, folding and watching: `app/src/workspace/worktrees.rs`
- The toggle column and the worktree row: `app/src/widgets/worktree.rs`
- Watching a worktree folder for its repository: `app/src/watcher.rs`

### API Endpoints

none

## Technical Decisions

| Decision | Rationale |
| -------- | --------- |
| The summary of a repository carries its other worktrees, each read like a repository | Every refresh, partial refresh and equality check of entries works for worktrees without a second path |
| A worktree is `Result<Summary, Error>` inside the summary | A missing folder keeps its path and shows as a problem, like an unreadable repository |
| The summary of a worktree listed this way has no worktrees of its own | No loop: a linked worktree registered as a repository lists the main one, and the main one is not listed again below it |
| A failed `git worktree list` leaves the list empty | Worktrees are extra; they must never make a repository unreadable |
| Worktrees are not repositories of the group | They follow their repository: no drag, no reorder, no removal from the list |
| Folded at first, open state kept only while the window is open | Folded rows keep the repository list short, and the open state is not part of the saved files |
| Arrow keys use the rows on screen; keeping the selection uses all worktrees of the group | A folded worktree is skipped by the keys, but a selected worktree is not dropped when its repository is folded elsewhere |
| Folding with a worktree selected selects the repository | The selection never hides |
| After staging, the owner repository is read again | The selected path can be a worktree, which is not registered, so reading it would replace nothing |
| A worktree folder is watched as a root that reports its repository | The existing debounce, ignore rules and partial refresh are reused |
| The `worktrees` entry of `.git` is relevant | Staging or committing in a linked worktree changes `.git/worktrees/<name>/` of the repository, not its own folder |
| Worktrees are watched after every read, one watch per folder | New worktrees are picked up without a restart; a folder that is watched already is left alone |
| A missing folder is told by `MISSING_FOLDER` of the core and shown as "Missing"; a worktree row is named by its folder, a failed repository by its path | The long error pushed the name out of the row; a repository that cannot be read keeps its path and Git's words because the user registered it by path |
| A worktree row has its own bottom line, drawn by the frame, not by the row | The row's border color follows its state and is transparent when clean, so the row ran into the repository below it |
| Rows without worktrees keep an empty toggle column when another row has worktrees | Names and states of all rows line up |

## Issues Encountered

none recorded

## Learnings

### Bounds of rows drawn earlier stay in the test window (2026-11-25)

After a row is folded away, `debug_bounds` still answers with the bounds it had when it was drawn, so a test cannot prove a row is gone that way.
The tests read the list the column is drawn from instead (`rows_on_screen`).

### Git prints worktree paths with links resolved (2026-11-25)

The folder of a worktree is shown as `/private/var/...` on macOS where the temporary folder was made as `/var/...`.
Tests compare with the resolved path.

### A removed worktree keeps its watch until the window closes (2026-11-25)

Events from the removed folder match no root and report nothing.
