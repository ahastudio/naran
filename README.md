# Naran

[![CI](https://github.com/ahastudio/naran/actions/workflows/ci.yml/badge.svg)](https://github.com/ahastudio/naran/actions/workflows/ci.yml)

A Git client for reading changes across many repositories on one screen.
Built with Rust and [GPUI](https://www.gpui.rs/). macOS, dark only.

Naran is for **reading**.
It stages and unstages; it never commits, fetches, pulls or pushes.

## Why Naran

- **Many repositories, one window.** Group them, and see branch, ahead and behind, staged, modified, untracked and conflicted counts, and a one-word state for each.
- **Always current.** File changes are watched, and only the repositories that changed are read again, so edits from a terminal or an editor show up at once.
- **Fast on big repositories.** The commit list loads older commits as you scroll, and the diffs build only the rows in view.
- **Precise staging.** Drag files between Staged and Unstaged, or stage a single hunk or the lines you pick.
- **Native feel.** Remembers the window, every divider and your last selection.

## Features

### Groups and repositories

- Create groups, rename or delete them (always asked first), and put a repository in as many groups as you like.
- Drag a group onto another to reorder the groups.
- Add folders with the button or by dropping them on the window.
  Folders that are not Git repositories are refused with a notice.
- Drag a repository to reorder it within its group.
  Right-click for Reveal in Finder or Remove (the folder is never touched).

### History

- A commit graph with branches, merges and ref badges, and the commit HEAD is on stands out.
- **Go to HEAD** jumps to it, loading older commits if needed.
- **Remote branches** and **All refs** switch the remote branches and Git's own refs (stash, notes, `refs/original`) on and off.
- Click a past commit to see its message, details, files and diff (read only).
  Images are shown as Before and After.

### Worktrees

- A repository that has other worktrees shows an arrow and their number at the right end of its row; click it to open them below the repository.
- Each worktree has its own branch, state and counts, follows the files in its folder, and can be selected like a repository to read and stage its changes.
- Naran only shows worktrees and moves between them; it never creates or removes one.

### Staging

- Every file of a new folder is listed on its own.
- Pick several files with **Cmd+A**, **Shift** (range) or **Cmd** (one by one), in the working tree and in past commits.
  Their diffs are stacked one after another, and you can drag them together.
- Untracked and conflicted files can only be staged as a whole.
- A staged rename shows only the lines that changed, and unstaging lines of it keeps the rename.

### Keys

| Key | Action |
| --- | --- |
| Up, Down | Move through the list you clicked last |
| Shift + Up, Down | Grow the picked files |
| Cmd + A | Pick every file of the list |
| Cmd + R | Refresh |
| Cmd + O | Add repositories |

## Install

Naran has no release downloads yet.
Build it from source:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo install cargo-about --locked --features cli
release/make-app.sh
open target/release
```

This makes `target/release/Naran.app`, one universal executable for Apple Silicon and Intel, signed ad hoc.
A copy on another Mac may be refused on first open: right-click it and choose Open, or run `xattr -cr Naran.app` once.

The Rust version is set by `rust-toolchain.toml`.
The system `git` must be installed.

## Development

```sh
cargo run -p naran-app    # run the app
cargo test --workspace    # tests
mise run check            # format, lint, docs, quality rules, scripts, specs and 100% coverage
```

| Folder | Content |
| --- | --- |
| `core/` | The core library. Runs the system `git` and parses its output. Knows nothing about the UI |
| `app/` | The GPUI app, `naran-app` |
| `tools/quality/` | The rules of the coding guide that clippy does not check |
| `scripts/` | `coverage.sh` with `uncovered.py`, and `check-specs.py` for the documents in `specs/`, and `check-python.py` for the scripts |
| `specs/` | One folder per feature with its requirements, plan, tasks, findings and progress |
| `release/` | `make-app.sh`, the app icon and the script that draws it, and the settings of the license list shipped in the app |
| `fixtures/` | A script that makes a large repository for measuring, used with `core/examples/measure.rs` |

Every check must pass, and every line of `core/`, `app/` and `tools/` must be covered by a test.
Work follows the file-based planning workflow: a feature is specified and planned in `specs/<feature>/` first, and built Outside-In with tests.
The rules, the commands and the workflow are listed in [AGENTS.md](AGENTS.md).
`mise run audit` checks the dependencies against the RustSec advisories.

Settings are JSON files in the system config folder under `dev.naran.app`: `repositories.json`, `settings.json`, `selection.json`, `window.json` and `layout.json`.

## Contributing

Bug reports, ideas and pull requests are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).
Please report a security problem privately, as [SECURITY.md](SECURITY.md) says.

## License

Naran is released under the [MIT License](LICENSE).
`app/src/text_input.rs` and `app/src/text_input/` are adapted from GPUI and stay under the [Apache License 2.0](LICENSES/Apache-2.0.txt); the source and the changes are noted at the top of `text_input.rs`.
