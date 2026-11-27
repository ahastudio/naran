---
name: check
description: Verify a change to Naran end to end before calling it done. Runs the full check (format, lint, docs, quality rules, Python rules, specs, 100% coverage), the audit, and then reviews the change against the specs and the repository rules. Use after any code or spec change, or when asked to verify, check or review the work.
---

# Check

A change is done only when everything below passes and the report says so with real results.

## 1. Run everything

1. `mise run check` runs `format`, `lint`, `docs`, `quality`, `python`, `specs` and `coverage` in parallel. It must end without `ERROR`.
2. `mise run audit` checks the dependencies against the RustSec advisories. Run it when `Cargo.toml` or `Cargo.lock` changed.
3. When something fails, run that task alone to read its output: `mise run <task>`.

| Task | A failure usually means |
| --- | --- |
| `format` | Run `cargo fmt --all`. rustfmt can split a line, so re-read the code it changed before editing it again |
| `lint` | A clippy warning; fix the cause, and give any `allow` a reason |
| `docs` | A broken doc link or a warning in a doc comment |
| `quality` | A function over 20 lines or five statements, more than three parameters, a `bool` parameter, a condition over more than one line, two parameters of one type in a row, a file over 300 lines, or `unwrap`, `expect`, `panic`, printing or `unsafe` outside tests |
| `python` | A script function over five lines of code |
| `specs` | A spec folder strays from the templates, a scenario has no `Test:` line or names a test that does not exist, a test is in no plan step, results are incomplete or out of plan order, or a banned word is used (`backfill`, `written afterward`, `after the fact`, `not run`; note that "not running" contains "not run") |
| `coverage` | A line, function or region no test runs. The output lists the file and position; add a test that reaches it, or remove the code. Error branches of `?` need a test that makes the call fail |

Do not weaken a check, skip it or add an `allow` to get past it.

## 2. Test the behavior, not only the checks

- `cargo test --workspace` runs every test; read the counts, not only the exit code.
- A test that needs the window finds elements by their selector, which is the element id. Bounds of rows drawn earlier stay in the test window, so prove that a row is gone by the list it is drawn from, not by its bounds.
- Git prints resolved paths (`/private/var/...` on macOS); compare with resolved paths in tests.
- A bug fix needs a test that fails without the fix.
- The app itself can be run with `cargo run -p naran-app`. When a change is about what the screen looks like, say plainly if it was not looked at by hand.

## 3. Review the change

Read the changed code once more as a reviewer, against these points:

1. It does what was asked and nothing more.
2. It reads like the code around it, with short English comments that say why.
3. Edge cases: empty lists, a missing folder, a failed `git` call, a name that is not ASCII, the same name in two places.
4. Anything that changes or deletes: Naran never commits, fetches, pulls or pushes, and only stages and unstages.
5. Element ids that can repeat, state that is never cleared, work done on every refresh that could be skipped.

## 4. Keep the documents in step

For the feature the change belongs to, in `specs/<feature>/`:

- `spec.md`: every new behavior is a scenario with a test that exists.
- `plan.md`: every test is named in exactly one step, with no code in it.
- `tasks.md`, `findings.md`, `progress.md`: phases ticked, decisions and learnings written, and the test results real. Never invent sessions, dates, errors or results.
- Other specs and `README.md` that describe the changed behavior, and the table of folders when one is added.

Run `python3 -I scripts/check-specs.py` after editing the documents.

## 5. Report

Say what ran and what it printed: which tasks passed, the test counts, and anything that was not done or not looked at. A failure is reported with its output, and a skipped step is named as skipped.
