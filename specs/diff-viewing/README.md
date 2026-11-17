# Viewing Changes

## Background

Naran is a Git client for reading changes across many repositories.
Reading a change takes more than the diff of one file: images need a before and an after, several files need to be skimmed one after another, and the files of a past commit need the same views.
A long line or a huge file must not freeze the window.

## Goal

Show the changes of the picked files so they are easy to read.
Files of the working folder and files of a past commit are shown the same way, and the text on screen can be selected and copied.
This feature is read only; staging lines and hunks is covered by another feature.

## How it works

### Text diff

- The diff of the open file shows old and new line numbers, a `+` or `-` marker and a title for each hunk.
- A long line is never cut; it scrolls sideways while the file name row stays in sight above it.
- Opening another file starts at the left edge.
- The text of the lines can be dragged to select and copied.
- A diff longer than 3000 lines is cut, and a note says how many lines are left out.
- An untracked file is shown whole as added lines; one over 512 KB or not text only gets a message saying so.

### Images

- An image file is shown as Before on the left and After on the right.
- A new file has no Before and a deleted file has no After; that side stays empty.

### Stacked diffs

- When two or more files are picked, their diffs are shown one after another.
- At most 40 files are read, and at most 500 lines of each; a note says what is left out.
- An image gets only a note.
- The head of the file at the top stays until the next head pushes it out, and it stays at the left edge while scrolling sideways.

### Files of a past commit

- Opening a file in the Files list of a commit shows what that commit changed in it, with the same diff view.
- Shift click (range), Cmd click (one by one), Cmd+A (all) and Shift+Up/Down (grow by one row) pick several files, which are then stacked.

## Related Documents

- [spec.md](spec.md): requirements and details
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task plan and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): work log per session
