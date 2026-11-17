# Groups and Repositories

## Background

Naran is a Git client for reading changes across many repositories on one screen.
Once there are many repositories, the user needs to group them by project or purpose.
Naran never commits, fetches, pulls or pushes, so choosing which repositories to read is the most basic setup the user does.

## Goal

The user groups repositories, sees the state of each at a glance, and finds the same selection again on the next launch.
The repository folders are never touched.

## How it works

The left column lists groups, and the middle column lists the repositories of the selected group.
Each group row shows its name, its repository count and the number of repositories with a problem (`!N`).
The repository column header shows the group name and the count, and each row shows the name, a one-word state, the branch and the counts that are not zero.

### Groups

- `+ New Group` in the group column opens a name input.
- Rename and Delete in the right-click menu rename or delete a group. Deleting always asks first.
- An empty name or a name that is already taken is refused, and a taken name shows a notice.
- A repository can be in several groups.

### Adding repositories

- `+ Add Repository` in the repository column, or Cmd+O, opens the system folder picker. Several folders can be chosen at once.
- Dropping folders on the window adds them too.
- A folder that is not a Git repository is not added, and a red notice at the top of the repository column says why. A click dismisses the notice.
- Without a group, the user is told to create one first.

### Order and removal

- Dragging a group row onto another group row reorders the group column, and the order is saved.
- Dragging a repository row reorders it inside its group, and the order is saved.
- The right-click menu of a repository has Reveal in Finder and Remove. Remove takes it out of the group after asking.
- A repository that is in no group leaves Naran. Its folder is untouched.

### Restoring the last selection

- The last selected group and repository are saved and restored on the next launch.
- If the saved group no longer exists, the first group is selected.

## Related Documents

- [spec.md](spec.md): requirements and detailed spec
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task plan and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): per-session work log
