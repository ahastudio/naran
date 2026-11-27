# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- Create, rename and delete groups; a repository can be in several groups.
- Add folders by button, system picker or drop; refuse non-Git folders with a red notice.
- Reorder groups and repositories by drag, Reveal in Finder, Remove.
- Show counts and state words in the group and repository columns.
- Restore the last group and repository.
- Use a single-line input to name groups.
- Move a repository to another group by dragging its row onto a group row.

## Research Findings

### Codebase Structure

- The UI lives in `app/src/workspace/`, split by job: `groups.rs`, `membership.rs`, `repositories.rs`, `menu.rs`, `feedback.rs`, `navigation.rs`, `reading.rs`.
- Pure rules and persisted types live in `app/src/model/settings.rs` and `app/src/model.rs`, with no UI.
- Process and file helpers are `app/src/picker.rs`, `app/src/finder.rs` and `app/src/store.rs`.
- Three JSON files hold this feature's state: `repositories.json` (known paths), `settings.json` (groups) and `selection.json` (last group and repository).
- `core/src/status.rs` parses `git status`; `naran_core::summarize` is what `Entry::read` calls.

### Existing Patterns

- Tests drive the real `Workspace` with GPUI's `TestAppContext`, in `app/src/workspace/tests/`.
- Tests inject another program for the picker and Finder, so no panel or Finder window opens (comments in `picker.rs` and `finder.rs`).
- Save failures are reported through one `report` function instead of being ignored.
- Repository rows are dragged with `on_drag`, `drag_over` and `on_drop`; group rows use the same three calls with their own dragged type.

## Resources

### Documents

- [README.md](../../README.md) and [AGENTS.md](../../AGENTS.md) for the rules the checks enforce.

### Code References

- Group and Selection types: `app/src/model/settings.rs:11`, `app/src/model/settings.rs:121`
- Create and rename rules: `app/src/model/settings.rs:56`, `app/src/model/settings.rs:69`
- Reorder rule for repositories: `app/src/model/settings.rs:97`
- Reorder rule for groups: `app/src/model/settings.rs:104`
- Repository drag source: `app/src/workspace/repositories.rs:139`; group drag source: `app/src/workspace/group_order.rs:29`
- Group drag and drop, and the move it makes: `app/src/workspace/group_order.rs:18`, `app/src/workspace/group_order.rs:29`
- Repository dropped on a group row: `app/src/workspace/group_order.rs:50`; the action `app/src/workspace/membership.rs:187`; the rule `app/src/model/settings.rs:115`
- State words: `app/src/model.rs:108`
- Group edit flow: `app/src/workspace/groups.rs:67`
- Group delete with confirmation: `app/src/workspace/groups.rs:92`
- Add flow: `app/src/workspace/membership.rs:72`
- Remove with confirmation: `app/src/workspace/membership.rs:195`
- Orphan pruning: `app/src/workspace/reading.rs:110`
- Notice and status bar: `app/src/workspace/feedback.rs:39`
- Window drop: `app/src/workspace/chrome.rs:80`
- Startup: `app/src/workspace.rs:98`, `app/src/workspace.rs:120`
- Picker: `app/src/picker.rs:61`
- File save: `app/src/store.rs:35`

### API Endpoints

none

## Technical Decisions

| Decision | Rationale |
| -------- | --------- |
| Run the folder picker through `osascript` in another process | Per the comment in `picker.rs`, GPUI's own panel waits on the main thread for a slow system service and freezes the window |
| Show a waiting banner with Cancel that kills the marked picker | A plain TERM does not end `osascript` while its panel is open (comment on `Picker::cancel`) |
| Reveal in Finder with `open -R` in another process | Naran never waits for it (`finder.rs` header) |
| Save by writing a temporary file and renaming | An interrupted write never leaves a broken file (`store.rs`) |
| Load falls back to defaults for a missing or broken file | The app still starts (`store::load` doc) |
| Repositories only exist inside groups; one in no group is dropped | `members` doc and the confirmation texts say a repository left in no group leaves Naran |
| A dropped group takes the slot of the group it lands on, by the same rule as a repository in a group | One rule explains both drags, and the saved order is the order of `groups` in `settings.json` |
| A repository dropped on another group row moves: it leaves the group it was dragged from and joins the end of the target group, once | The user decided it is a move and not a copy; there is no copy gesture, and the repository stays in the list because the target group holds it |
| The repository drag carries the group it came from | A drop on the same group then does nothing, and a repository in two groups leaves only the one it was dragged from |
| The drag-over border is one shared function, `drop_border` in `widgets.rs` | The group reorder, the repository reorder and the repository-onto-group drop look the same |
| Ask before deleting a group or removing a repository | The functions are documented "Asks first" |
| Every error is also a red notice at the top of the repository column | So it cannot be missed (`report` doc) |
| Draw destructive menu choices in the problem color | So they are not picked by mistake (`menu.rs` comment) |

## Issues Encountered

none recorded

## Learnings

### The drop-target border cannot be read in a test (2026-11-24)

GPUI paints `drag_over` styles but gives a test no way to read them.
The scenario "the row under the pointer tells where it will go" is therefore pinned in two parts: the test drags over the row and asserts the drag is active, and asserts the rule `DraggedRepository::moves_onto` that decides the border.
The border itself is looked at by hand (plan, Manual Test 4).

### The selection after a move is the neighbour, not the first row (2026-11-24)

Removing a repository selects the group's first row (`visible_selection`), but story 7 says the next row.
`neighbour` in `navigation.rs` picks the row after the moved one, the row before it when it was last, and nothing when it was alone; it reads the order before the move.

### A file row dropped on a group row goes nowhere (2026-11-24)

The group row accepts only `DraggedGroup` and `DraggedRepository`, so a `DraggedFile` dropped on it does nothing, and the test pins that git status stays unchanged.

### Group name trimming happens twice (2026-11-16)

The window path trims the typed name before it reaches the group rules, and the group rules trim it again.
The window test passes with or without the trim in the group rules, so the model test is the one that pins it.

### Selection saving is pinned only through the save itself (2026-11-16)

The two restore tests also move the repository selection, and that save covers the group too, so they do not pin the save call inside group selection.
The test `switching_to_a_group_that_holds_the_same_repository_is_remembered` selects a group that holds the same repository, so only that call can remember the group.
