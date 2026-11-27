# Worktrees

## Background

A repository can have more than one working folder: Git calls them worktrees.
Each has its own branch and its own changes, so a developer who works on several branches at once has several folders to watch.
Naran reads changes across many repositories on one screen, and worktrees belong on that screen next to the repository they came from.

## Goal

A repository row shows the worktrees of its repository, folded away until the user opens them.
Clicking a worktree shows its changes, history and staging, like any repository.
Naran only reads and moves between worktrees; it never creates, removes or prunes one.

## How it works

### Activation

- A repository that has other worktrees shows a toggle at the right end of its row: a small arrow and the number of worktrees.
- The rows are folded at first. Clicking the toggle opens them below the repository, indented.
- Each worktree row shows its folder name, branch, one-word state and change counts, like a repository row, with a `└` mark and a tinted background so it reads as part of the repository above.
- A selected worktree puts a bar above its changes: "Worktree of <repository>", its folder and its branch.
- Clicking a worktree row selects it. The commits and files columns then show that worktree.
- Up and Down move through the rows that can be seen, so a folded worktree is skipped.
- A change inside a worktree folder reads its repository again, so the worktree row follows at once.

### Deactivation

- A repository without other worktrees shows no toggle.
- Folding a repository whose worktree is selected selects the repository again.
- A worktree whose folder is gone is shown as a problem instead of being hidden.
- Worktrees are not dragged, reordered or removed from the list: they follow their repository.

## Related Documents

- [spec.md](spec.md): requirements and detailed spec
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task planning and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): per-session work log
