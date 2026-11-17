# Workspace Layout and Keyboard

## Background

Naran is a Git client for reading changes across many repositories.
With many repositories, you move between lists, history, files and diffs all the time.
So the screen stays in one window, and its sizes, position and last selection must survive a restart.

## Goal

Let the user read along the lists with a one-window, three-column screen and the keyboard alone.
The window position, divider sizes, selection and the list the arrow keys move through are remembered, so the next run starts in the same place.

## How it works

### Activation

- The screen is always on: one window, dark only, with the groups, the repositories of the selected group and the detail of the selected repository side by side.
- The detail column splits into the commit list on top, and the file lists and the diff below.
- The window title, and the title in the middle of the toolbar, is the folder name of the selected repository, or "Naran" when none is selected.
- The toolbar has a Refresh button, and the status bar shows the last result and the path of the selected repository.
- Four dividers can be dragged to resize, and every size is kept inside a range.
- Up and Down move through the list that was clicked last, and Shift+Up and Shift+Down grow the picked files.
- Cmd+A picks every file of the file list pressed last, or else all the text of the view pressed last.
- Cmd+R refreshes, and Cmd+O opens the picker to add repositories.
- The window position, size and state go to `window.json`, and the sizes, the arrow-key list and the refs flags go to `layout.json`.
- The repository list, the group settings and the last selection go to `repositories.json`, `settings.json` and `selection.json`.
- All files live in the config folder `dev.naran.app`.

### Deactivation

- There is no switch to turn this off.
- A missing or broken saved file falls back to defaults, and sizes outside their range are pulled back in.

## Related Documents

- [spec.md](spec.md): requirements and detailed spec
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task plan and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): per-session work log
