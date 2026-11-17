# Staging

## Background

Naran is a Git client for reading changes across many repositories on one screen.
While reading, the user must be able to sort changes into the index right away, by file, hunk or picked lines.
Naran never commits, fetches, pulls or pushes, so staging and unstaging are the only changes it makes to a repository.

## Goal

Show the changed files of a repository in two lists, Staged files and Unstaged files.
Let the user stage and unstage whole files, hunks, and picked lines.

## How it works

### File lists

- The Staged files list sits above the Unstaged files list, and each list scrolls on its own.
- Every file of a new folder is listed on its own, not as one folder entry.
- Each row starts with a status badge (M, A, D, R, C, T, ?, !).
- A renamed or copied file shows its new path, then `<-` and the path it came from (`new name.txt <- old name.txt`).
- A file with changes in both the index and the working directory is listed in both lists.

### Moving whole files

- Dropping a file row on the other list stages or unstages it.
- Dropping on the same list does nothing.

### Picking several files

- A plain click picks one file, Shift picks a range, Cmd (or Ctrl) adds or removes one file.
- Shift+Up and Shift+Down grow the picked range by one row, and Cmd+A picks every file of the list.
- Dragging one of the picked files takes all of them along, and two or more picked files show their diffs stacked one after another.
- Picking stays inside one list, and the list header shows `N selected`.

### Hunks and lines

- In the diff of a tracked file, a hunk link or the picked lines stage or unstage part of the file.
- In the Staged list the same links read Unstage.
- Untracked and conflicted files can only be staged as a whole.

## Related Documents

- [spec.md](spec.md): requirements and detailed spec
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task plan and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): per-session work log
