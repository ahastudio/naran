# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- Text diff with line numbers, sideways scroll, sticky file name, selectable text.
- Before and After pictures for images.
- Stacked diffs of two or more files: 40 files, 500 lines per file, image note, sticky head.
- Diffs of past commit files, with multi-pick by Shift, Cmd, Cmd+A and Shift+Up/Down.

## Research Findings

### Codebase Structure

- `core/` runs the system `git` and has no UI; `app/` is the GPUI app (`AGENTS.md`).
- `app/src/workspace/detail/` holds the views: `diff_view.rs`, `diff_rows.rs`, `viewing.rs`, `versions.rs`, `pictures.rs`, `stack.rs`, `stack_view.rs`, `commit_view.rs`, `commit_picking.rs`.
- `app/src/model/diff.rs` turns `git diff` lines into rows; `app/src/selectable/` paints and copies text.
- `render_diff` picks the stacked view first, then the commit view, then the working view.

### Existing Patterns

- Reads run on `cx.background_executor()` and results apply only if the same file or picks are still shown (`with_viewing`, `set_stack_rows`).
- Files are at most 300 lines and functions at most 20 lines (`AGENTS.md`), which shapes the many small files.
- Tests sit beside the code and in `app/src/workspace/tests/`.

## Resources

### Documents

- `README.md`, `AGENTS.md` of the repository.

### Code References

- Diff cut: `app/src/workspace/detail/diff_rows.rs:14`
- Untracked limit: `app/src/workspace/detail/versions.rs:11`
- Stack limits: `app/src/workspace/detail/stack.rs:15`, `app/src/workspace/detail/stack.rs:16`
- Stack lines skip file header: `app/src/workspace/detail/stack.rs:130`
- Sideways scroll: `app/src/workspace/detail/diff_rows.rs:209`
- Sticky head: `app/src/workspace/detail/stack_view.rs:123`
- Column width of non-ASCII text: `app/src/workspace/detail/stack_view.rs:248`
- Picture limit: `core/src/blob.rs:21`
- Left-edge reset: `app/src/workspace/detail/viewing.rs:128`
- Commit click picking: `app/src/workspace/detail/commit_picking.rs:57`

### API Endpoints

none (the app has no API)

## Technical Decisions

| Decision | Rationale |
| -------- | --------- |
| Cut a diff at 3000 lines | "so a huge file cannot freeze the window" (`diff_rows.rs`) |
| Never cut a line; scroll sideways | "a long line makes the diff wider and it scrolls sideways" (`diff_rows.rs`) |
| Fixed name row above scrolling lines | The pane does not scroll itself, so the file name stays in sight (`diff_view.rs`) |
| Stack uses `uniform_list` | It builds only the rows in view (`stack_view.rs`) |
| Stack drops the file head lines | "the file head lines repeat the name" (`stack.rs`) |
| Images get only a note in the stack | The note says to pick the image alone to see Before and After (`stack.rs`) |
| Non-ASCII text counts as two columns | Letters such as Korean "are drawn about twice as wide" when finding the widest row (`stack_view.rs`) |
| Untracked files over 512 KB are not read | "not read for the diff view" (`versions.rs`) |
| Files over 8 MB are not read as pictures | `BLOB_LIMIT` in `core/src/blob.rs` |
| Same file shown again keeps its lines; another starts at the left edge | Comment on `show_file` in `viewing.rs` |
| Selection end is not a click | "The end of a selection is not a click on the line" (`diff_view.rs`) |

## Issues Encountered

none recorded

## Learnings

### An unchanged open file does not leave an empty diff open (2026-11-16)

A window test that opened an unchanged file and expected an open diff with no lines did not find one, so "No changes to show." is covered only at the inner level.

### Why the sticky head stays left (2026-11-16)

The sticky head is placed absolutely inside the container that does not scroll sideways, while the heads inside the list are moved back by the sideways offset.
