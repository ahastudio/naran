# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- Two lists, Staged files and Unstaged files, with status badges.
- Every file of a new folder listed on its own.
- A renamed file shown as `new path <- old path`.
- Drag files between lists, singly or as a picked set.
- Pick with click modifiers, Shift+Up/Down and Cmd+A.
- Stage or unstage a hunk or picked lines; untracked and conflicted files only as a whole.

## Research Findings

### Codebase structure

- `core/` runs the system `git` and parses its output; `app/` is the GPUI app (see `AGENTS.md`).
- Staging operations live in `core/src/ops/staging.rs`; the patch builder is `core/src/patch.rs`.
- The app side is split by job under `app/src/workspace/detail/`: lists (`file_lists.rs`), rows and drag (`file_rows.rs`), picking (`picking.rs`, `marks.rs`), changes (`staging.rs`), diff links (`diff_view.rs`).
- File state and badges are in `app/src/model/files.rs`.

### Existing patterns

- A change runs through `change_repository`: mark busy, run in the background, report, read the repository again.
- Lists use `uniform_list`, so only rows in view are built.
- Line numbers in a diff row are indexes into the `git diff` text, which the core uses to name lines.

## Resources

### Documents

- `README.md` and `AGENTS.md` of the project.

### Code References

- Stage and unstage: `core/src/ops/staging.rs:9`, `core/src/ops/staging.rs:13`
- Line staging: `core/src/ops/staging.rs:60`
- Patch builder: `core/src/patch.rs:12`
- Status arguments: `core/src/status.rs:7`
- Old path of a rename: `core/src/status.rs:55`
- `Staging` and its direction: `core/src/diff.rs:12`
- File state, badge and row text: `app/src/model/files.rs:36`, `app/src/model/files.rs:81`, `app/src/model/files.rs:98`
- Lists: `app/src/workspace/detail/file_lists.rs:50`
- Drag and drop: `app/src/workspace/detail/file_rows.rs:26`, `app/src/workspace/detail/file_rows.rs:66`
- Move files: `app/src/workspace/detail/staging.rs:34`
- Picking: `app/src/workspace/detail/picking.rs:44`, `app/src/workspace/detail/marks.rs:37`
- Partial staging guard: `app/src/workspace/detail/viewing.rs:44`
- Hunk link: `app/src/workspace/detail/diff_view.rs:252`

### API Endpoints

none

## Technical Decisions

| Decision | Rationale |
| -------- | --------- |
| The diff of a staged file is asked for with its old path too, when `git diff --cached -M --name-status` lists it as a rename | Git pairs a rename only when it sees both paths; with the new path alone the whole file shows as added (`core/src/diff.rs`) |
| A patch made from a rename diff names the new path alone | A rename header would make `git apply --reverse` move the file back instead of undoing some lines (`plain_header` in `core/src/patch.rs`) |
| Diffs are read with `core.quotePath=false` | A rename header would otherwise quote a non-ASCII name and the patch header would not match it |
| A renamed file's old path is kept in `Change::original` | The next record of a rename in the status output is its old path (`status::next_entry`) |
| `new path <- old path` as the row text | One text holds both paths, and the same text names the row for tests (`model::change_label`) |
| A rename header becomes a note, `Renamed from <old path>`, and the other header lines stay hidden | A rename without edits has no hunk, so the pane was empty; the note is drawn at the left edge like a hunk title, because indented to the code column it was pushed off the pane by a long path (`DiffBuilder::meta` in `app/src/model/diff.rs`, `note_row`) |
| Unstaging a renamed file includes its old path | Restoring only the new path leaves the old one staged as deleted (`with_origins` in `ops/staging.rs`) |
| The old path shows only on the side that has the rename | The label follows the badge letter R or C of that list (`model::change_label`) |
| Commit file lists stay without renames | Their reads use `--no-renames`, so a renamed file is a delete and an add there |
| `--untracked-files=all` in every status read | Each untracked file is listed on its own; git would otherwise show a new folder as one entry (`status.rs`) |
| Unstage with `rm --cached` before the first commit | `git restore --staged` needs HEAD (`has_commits` in `ops/staging.rs`) |
| One patch builder for both directions | Unstaging is the same patch applied in reverse (`Staging::direction`) |
| Unselected changes become context or are dropped | They keep their current state (`patch.rs`, `Direction::unselected`) |
| Last line without a newline is removed and added again | The new file has a newline there once lines are added after it (`reopen_last_line`) |
| `git apply --cached --recount --whitespace=nowarn` | Counts are recomputed for the patch that was cut down (`apply_arguments`) |
| Untracked and conflicted files only whole | `Viewing::partial` is true only for tracked files |
| A conflicted file waits in the Unstaged list | Its letters only describe the two sides (`FileState::of`) |
| Picking stays inside one list | `Marked` holds one `staging` and one set of paths |
| Dragging a picked file takes all picks | `dragged_with` returns the picked files in list order |
| Picks and stack cleared after a change | `change_done` resets `marked` and `stacked` |
| One change at a time | `begin_change` returns nothing while `busy` is set |

## Issues Encountered

none recorded

## Learnings

none recorded
