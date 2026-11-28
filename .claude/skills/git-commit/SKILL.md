---
name: git-commit
description: Commit changes to Naran. Splits the work by purpose and writes each message with a short intent-centered subject and a body. Use when asked to commit, split commits, or write or fix a commit message.
---

# Git Commit

Commit only when asked.
Every rule here is mandatory, for a one-line change too.
A commit that breaks one is redone before you report it.
When a rule and your habit disagree, the rule wins.

Before writing the message:

- Read the last ten commits with `git log` and copy their style. Do this every time, even if you did it earlier in the session.
- Do not run `mise run check` when the request is only to commit; it is slow. Say in the report that it was not run, and never imply it passed.
- Read `git status` and `git diff HEAD`. Stage by file name, never `git add -A` or `git add .`.
- One commit holds one purpose. A dependency bump, a code change with its tests and specs, a spec-only change and a refactor are separate commits. When asked to split, split by purpose, not by file.
- Say the intent in one plain sentence first: who needs what, and why. If you cannot, you do not know the reason yet; ask, and do not write a message.

The subject:

- Imperative, English, at most 50 characters, no trailing period. A longer one is shortened, never squeezed in.
- States the intent, the goal the change serves, not the mechanics: "Keep live updates working after the upgrade", not "Match events against both paths".
- Names no code at all (file, function, type, crate, command), so it never needs backticks. A subject that cannot do without one is about the mechanics; rewrite it.

The body, always present; a subject-only commit is never allowed:

- Starts after a blank line and is wrapped at 72 characters.
- Is about intent: what the user or the project needs and why, then what is now possible or true.
- Opens with the need or the problem, never with what the code does.
- Leaves mechanics (renamed functions, shortened comments, moved code) to the diff, and mentions them only when they explain a side effect.
- Names the side effects: lock file changes, and the specs or docs updated with the code.
- Never invents a reason. When the reason is unknown, state only what changed, or ask.
- Puts every code name in backticks wherever it appears: crates, files, functions, types, commands, flags and paths.
- Keeps the backticks few by writing about the intent, not by leaving them out. Many backticks mean the message is about the code; rewrite the content around the need and the result, so fewer names are needed at all.
- Is passed to `git commit -F -` with a quoted heredoc (`<<'E'`), so the shell never expands a backtick.
- Ends with the attribution line the session gives.

After committing:

- Show `git log` with the full messages and confirm `git status` is clean.
- Re-read each message against this list: subject length, intent in the subject and the body, body present, every code name in backticks, no message made of code names, wrapping. Redo any that fail, and say that you redid them.
- To redo commits, `git reset --soft` to the commit before them, unstage with `git reset`, and commit again by purpose.
- Never amend, rebase, hard reset or force-push without being asked, except to redo a commit that broke these rules.
