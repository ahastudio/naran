# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- One window, three columns, dark theme, the selected repository as the window title.
- Resizable dividers with limited ranges.
- Saving and restoring the window, layout and selection.
- Up, Down, Shift+Up, Shift+Down, Cmd+A, Cmd+R, Cmd+O.

## Research Findings

### Codebase Structure

- `app/src/lib.rs` opens and saves the window, and `main.rs` only starts the event loop.
- `Workspace` is one GPUI entity whose methods are spread over child modules as `impl Workspace`.
- `state.rs` groups state into `Files`, `Shown`, `Interaction`, `Scrolls` and others.
- `Files::new` builds the paths `repositories.json`, `settings.json`, `selection.json` and `layout.json`.
- The window state is saved separately by `lib.rs` as `window.json`.
- `detail/columns.rs` lays out the commit list, file lists, diff and dividers of the detail column.

### Existing Patterns

- Saving always goes through `store::save` and reading through `store::load`.
- A failed save goes through `report` to the status bar and a notice.
- Keys are declared with `actions!` and bound in the `Workspace` context by `bind_keys`.
- Tests use `#[gpui::test]` and are split by topic under `app/src/workspace/tests/`.

## Resources

### Documents

- `README.md`, `AGENTS.md`

### Code References

- Saved window type: `app/src/lib.rs:44`
- Moving an off-screen window: `app/src/lib.rs:89`
- Opening the window: `app/src/lib.rs:120`
- JSON load and save: `app/src/store.rs:26`, `app/src/store.rs:35`
- Config folder identifier: `app/src/store.rs:12`
- Dark theme: `app/src/theme.rs:31`
- Key bindings: `app/src/workspace.rs:49`
- Layout clamped on load: `app/src/workspace.rs:113`
- Window title: `app/src/workspace/chrome.rs:40`
- Key and action wiring: `app/src/workspace/chrome.rs:65`
- Divider size rules: `app/src/workspace/layout.rs:54`
- Size clamping: `app/src/workspace/layout.rs:70`
- Saving the layout: `app/src/workspace/layout.rs:157`
- Remembering the arrow-key list: `app/src/workspace/layout.rs:172`
- Moving through a list: `app/src/workspace/navigation.rs:111`
- Cmd+A: `app/src/workspace/navigation.rs:166`
- One-step computation: `app/src/workspace/navigation.rs:186`
- Saving and notices: `app/src/workspace/feedback.rs:10`, `app/src/workspace/feedback.rs:39`
- Commit and file stepping: `app/src/workspace/detail/stepping.rs:17`, `app/src/workspace/detail/stepping.rs:41`
- Shared widgets: `app/src/widgets.rs:24`, `app/src/widgets/badges.rs:7`

### API Endpoints

none

## Technical Decisions

| Decision                                         | Rationale                                                                                        |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| Config folder `dev.naran.app`                    | Code comment: the app identifier names the folder that holds the saved files, next to the repository list and groups. |
| Write a temporary file, then rename              | Code comment: an interrupted write never leaves a broken file behind.                            |
| Missing or broken file gives the default         | Doc comment of `store::load`.                                                                    |
| Clamp sizes when loading                         | Code comment: a hand-edited or broken file cannot make a part vanish.                            |
| Save the window on every change and on close     | Code comment: a crash or a forced quit loses nothing.                                            |
| Move only the position of an off-screen window   | Code comment: when a display was unplugged, the size is kept and the position moves onto the first screen. |
| Draw the title inside the toolbar                | Code comment: macOS hides the system title of a window whose title bar is see-through.           |
| Set the title only when it changes               | Code comment: not on every frame.                                                                |
| Save the last clicked list in `layout.json`      | Code comment: the list the arrow keys move through is remembered across runs.                    |
| `Pressed` tells what Cmd+A picks from            | Code comment: what was pressed last, a list or the text of the commit information or diff.       |
| Group state into per-purpose structs             | Comment in `state.rs`: each field says which part of the screen it serves.                       |

## Issues Encountered

none recorded

## Learnings

### Lists that wrap (2026-11-16)

The repository list wraps around at its ends, while the group, commit and file lists stop at theirs.

### Refresh starts no second read while one runs (2026-11-16)

The faded Refresh button ignores clicks while the repositories are being read (`app/src/workspace/chrome.rs`), and `refresh_does_nothing_while_the_repositories_are_being_read` in `app/src/workspace/tests/layout_acceptance.rs` pins it.
