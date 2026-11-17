# Findings & Decisions

> **Update this file immediately whenever there is a technical finding or an important decision.**

## Requirements

- Columns Graph, Description, Date, Author, Commit.
- Lanes, merges, ref badges, HEAD emphasis, dashed Uncommitted link.
- Paging, Go to HEAD, Remote branches and All refs toggles saved in `layout.json`.
- Picking a commit, read-only, with Up and Down stepping.

## Research Findings

### Codebase Structure

- `core/` runs `git log` and parses it; `app/` holds the GPUI views under `app/src/workspace/detail/`.
- `Detail` (loaded.rs) holds changes, commits, graph rows, `more` and `loading_more`.
- `Inspecting` (commit.rs) holds the picked commit and its reads.
- Reads run on the background executor and are applied only if still current.

### Existing Patterns

- `uniform_list` builds only the rows in view.
- Results that come back after a refresh or a repository change are dropped.
- Row indexes are shifted by one when the Uncommitted changes row is present.

## Resources

### Documents

- `README.md` (History section) and `AGENTS.md`.

### Code References

- `Refs` and its saved form: `core/src/history.rs:19`, `core/src/history.rs:71`
- `history`, `history_page`: `core/src/history.rs:126`, `core/src/history.rs:147`
- Lane layout: `core/src/graph.rs:24`
- Page size: `app/src/workspace/detail/loaded.rs:13`
- Paging: `app/src/workspace/detail/loaded.rs:87`, `app/src/workspace/detail/history.rs:122`
- Go to HEAD: `app/src/workspace/detail/loaded.rs:130`, `app/src/workspace/detail/history.rs:188`
- Toggles: `app/src/workspace/detail/history.rs:166`, `app/src/workspace/detail/history.rs:172`
- Footer: `app/src/workspace/detail/history_view.rs:65`
- Column head: `app/src/workspace/detail/history_view.rs:140`
- HEAD link: `app/src/workspace/detail/lanes.rs:61`
- Dashed line: `app/src/workspace/detail/painting.rs:11`
- Stepping: `app/src/workspace/detail/stepping.rs:17`
- Picking: `app/src/workspace/detail/inspecting.rs:27`
- Saved layout: `app/src/workspace/layout.rs:101`

### API Endpoints

None.

## Technical Decisions

| Decision | Rationale |
| -------- | --------- |
| A lane keeps its number once assigned (graph.rs) | Lanes passing through stay in the same place |
| First parent continues in the commit's lane | Comment in `Lanes::leave` |
| Link runs down the HEAD lane unless busy above, else a lane of its own | It never lies over another line (`head_link` comment) |
| Footer under the list for the buttons | They never cover rows (`history_footer` comment) |
| Refresh reads as many commits as already listed | The list keeps its length and scroll position (`history_limit`) |
| Next page loads when the view is within 40 rows of the end | Implemented in `history_list` |
| A page whose list changed meanwhile is dropped | Comment in `Detail::finish_page` |
| Default refs hide Git's own refs | `Refs::everything` doc: branches, tags and HEAD are what the user means by history |
| HEAD left out of `git log` without commits | It names nothing yet and `git log` would fail (`Refs::arguments` comment) |
| `Refs` saved as flags, older scope names still read | `saved` module comment |
| Graph capped at 12 lanes | `MAX_LANES` in lanes.rs |

## Issues Encountered

none recorded

## Learnings

### The window keeps the last bounds of a removed element (2026-11-16)

Asking the test window for the bounds of an element that is no longer drawn still returns where it was last drawn.
Tests that check an element is gone read the workspace state instead.

### An empty repository has no Go to HEAD button (2026-11-16)

The commit list returns early with "No commits yet." before it builds the footer, so the buttons are not drawn.
