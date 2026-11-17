# Commit History

## Background

Naran is for reading changes across many repositories.
To read a repository, the user needs to see how its commits relate: branches, merges, and where HEAD is.
Big repositories have too many commits to load at once.

## Goal

Show a repository's commits as a graph the user can scroll, pick from, and step through.
Keep it fast on big repositories by loading older commits only when needed.
Everything about a picked commit is read-only.

## How it works

### The commit list

- The list has the columns Graph, Description, Date, Author and Commit.
- The graph draws lanes, merges and ref badges; the commit HEAD is on stands out.
- When there are uncommitted changes, an Uncommitted changes row comes first, joined to the HEAD commit by a dashed line.
- The first page is 200 commits; older pages are added as the list is scrolled near its end.

### Go to HEAD

- The Go to HEAD button scrolls to the HEAD commit and picks it.
- If HEAD is older than what is loaded, older pages are read until it is found or there are no more.

### Remote branches and All refs

- Two toggles under the list switch remote branches and Git's own refs (stash, notes, `refs/original`).
- The choice is saved in `layout.json`; the older saved form is still readable.

### Picking a commit

- Clicking a commit, or pressing Up or Down, shows its message, details, refs and changed files.
- Clicking the Uncommitted changes row brings the Staged and Unstaged lists back.

## Related Documents

- [spec.md](spec.md): requirements and detailed spec
- [plan.md](plan.md): technical implementation plan
- [tasks.md](tasks.md): task plan and tracking
- [findings.md](findings.md): technical findings and decisions
- [progress.md](progress.md): per-session work log
