# Live Updates

## Background

Naran reads the changes of many repositories on one screen.
When a file is edited in a terminal or an editor, the screen has to follow at once.
Reading every repository again each time, or freezing the window while reading, would make it unusable.

## Goal

Changes made outside the app show up without a manual refresh.
Only the changed repositories are read again, and the window never waits for the reading.

## How it works

### Activation

- When the window opens, every registered repository is read once and file watching starts.
- When the watcher reports a change, only that repository is read again in the background.
- A repository is summarized by its branch, ahead/behind, and staged/modified/untracked/conflicted counts, plus a one-word state: Merging, Conflicts, Changes, Diverged, Ahead, Behind, No upstream or Clean.
- Every 60 seconds all repositories are reported again, in case an event was lost.
- The Refresh toolbar button or Cmd+R reads every repository again by hand.

### Deactivation

- A repository that no group holds is neither read nor watched.
- When watching cannot start, a notice says so and points to Refresh.
- While reading, the Refresh button shows "Reading…" and cannot be pressed.

## Related Documents

- [spec.md](spec.md): requirements and detailed spec
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task planning and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): per-session work log
