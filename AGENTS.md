# AGENTS.md

Naran is a GUI Git client (Rust, GPUI) for reading changes and staging.
It never commits, fetches, pulls or pushes.
See `README.md` for the screen, the features and the folder layout.

## Commands

- `cargo run -p naran-app` runs the app.
- `cargo test --workspace` runs the tests.
- `mise run check` runs everything below; all of it must pass before a change is done.
  - `format`: `cargo fmt --all --check`
  - `lint`: clippy with `-D warnings`
  - `docs`: `cargo doc` without warnings
  - `quality`: the coding guide rules clippy does not check
  - `python`: every function of the Python scripts has at most five lines of code
  - `specs`: the feature documents in `specs/` follow the templates, every scenario names an existing test, and the test results are complete
  - `coverage`: every line, function and region must be covered
- `release/make-app.sh` builds `target/release/Naran.app` (universal, ad hoc signed).

## Rules the checks enforce

- A function has at most 20 lines and five statements (nested blocks count), and at most three parameters besides `self`, window and context.
- No `bool` flag parameters; a condition fits on one line.
- A file has at most 300 lines.
- No `unsafe`, `unwrap`, `expect`, `panic` or printing outside tests; no `allow` without a reason.
- Every new line of `core/`, `app/` and `tools/` needs a test that runs it.

## Workflow

Features are planned with the [file-based planning workflow](https://github.com/ahastudio/file-based-planning-workflow).
Each feature has a folder `specs/<feature>/` with `README.md`, `spec.md`, `plan.md`, `tasks.md`, `findings.md` and `progress.md`, laid out exactly as the templates there.

- Read `tasks.md`, `findings.md` and `progress.md` of the feature before working, and update them as you go: phases in `tasks.md`, decisions and errors in `findings.md`, the session log and the real test results in `progress.md`.
- `spec.md` is the product requirement, with numbered user stories and Given/When/Then scenarios that cover the normal path, the edges and the failures. Every scenario names the test that proves it.
- `plan.md` is an Outside-In TDD plan with no code in it. Step 0 is the acceptance tests: they are written first and fail until the last step. Each next step builds one part inward (Screen, Widgets, Workspace, Model, Platform, Core, as the feature needs them) with the part below it stubbed, and replaces the stub of the step before with the real part; each step is Red, Green, Refactoring. Every test is named in exactly one step.
- `tasks.md` Phase 3 has one line per step of `plan.md`, Step 0 included; a step that is in progress may be split into write the test, implement and refactor.
- Work in this loop: pick the next task from `tasks.md`; write what you find or decide into `findings.md` at once; tick the task in `tasks.md` when it is done; before a session ends, record where you stopped in `progress.md` and keep its 5-Question Reboot Check true, so a new session can resume from it alone.
- When you ask an agent to build a feature, say it in this form: "Use `specs/<feature>` and implement it. After each step, update `tasks.md`, `findings.md` and `progress.md` too."
- Do not merge or rename phases, and do not invent sessions, dates, errors or results.
- Specs are written in English, one sentence per line. The sub-headings and fixed sentences of the templates are written in English too, and titles use Title Case (Commit History).
- The next step in `progress.md` and the 5-Question Reboot Check names only what is really left; when nothing is left it says `None (all done)`.
- Specs hold technical facts only: no instructions, no story of how the work went, no open opinions about the design.

## The parts of the app

Naran has six parts. "Outside" is the side closest to the user, so a feature is built from the screen inward.

| Part | What it is | Where |
| --- | --- | --- |
| Screen | What the user sees and touches in the window: the columns, rows and diff, and the clicks, keys and drops they take | the views in `app/src/workspace/` |
| Widgets | Shared pieces of the screen: text input, selectable text, badges, buttons and the theme | `app/src/widgets/`, `app/src/text_input/`, `app/src/selectable/`, `app/src/theme.rs` |
| Workspace | The state of the window and the actions on it | `app/src/workspace/` |
| Model | The pure rules of the app, with no drawing and no files | `app/src/model/` and the pure parts of the workspace such as sizes and steps |
| Platform | The app's contact with files, processes and the system | `app/src/store.rs`, `app/src/watcher.rs`, `app/src/picker.rs`, `app/src/finder.rs` |
| Core | The `core` crate: runs the system `git` and knows nothing about the UI | `core/` |

## Conventions

- `core/` runs the system `git` and knows nothing about the UI; `app/` is the GPUI app.
- Comments and docs are English, short, and say why.
- Python scripts have the same shape (checked by `python`): a function has at most five lines of code, and a longer one is split by extracting methods.
- Write code that reads like the code around it.
- Do not write to Claude's memory. A lesson or a rule from the user goes into this file or a skill in `.claude/skills/`, where the whole team reads it.
- Do not commit unless asked. When asked, use the `git-commit` skill: its rules are mandatory.
