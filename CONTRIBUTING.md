# Contributing to Naran

Thank you for helping.
Naran is a Git client for reading changes across many repositories; it stages and unstages, and never commits, fetches, pulls or pushes.
Please read [README.md](README.md) first, and follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Get started

You need macOS, the Rust toolchain from `rust-toolchain.toml` (installed by `rustup` on the first build), the system `git`, and `python3`.

```sh
cargo run -p naran-app    # run the app
cargo test --workspace    # tests
mise run check            # everything a change must pass
```

`mise run check` runs the format check, clippy with every warning as an error, the docs build, the coding guide rules in `tools/quality`, the checks of `specs/`, and the tests with 100% coverage.
The rules are listed in [AGENTS.md](AGENTS.md); the main ones are short functions (at most 20 lines and five statements), at most three parameters, no `unwrap` or `panic` outside tests, and a test for every new line.

## Work on a feature

Features are planned in `specs/<feature>/` with the [file-based planning workflow](https://github.com/ahastudio/file-based-planning-workflow).

1. Read the feature's `spec.md`, `plan.md`, `tasks.md`, `findings.md` and `progress.md`.
2. For a new feature or a changed behavior, write or update the scenarios in `spec.md` first. Every scenario names the test that proves it.
3. Build it Outside-In: Step 0 writes the acceptance tests that drive the window; they fail until the end. Then build one part at a time from the user inward (Screen, Widgets, Workspace, Model, Platform, Core; see [AGENTS.md](AGENTS.md)), each with the part below stubbed, replacing the stub of the previous step with the real part, each step Red, Green, Refactoring.
4. Keep `tasks.md`, `findings.md` and `progress.md` true as you go, with real test results. Do not invent sessions, errors or results.
5. Run `mise run check`.

For a small fix, a failing test, the fix and a passing `mise run check` are enough.

## Pull requests

- Keep a pull request to one change, with its tests and its `specs/` updates.
- Say what changed and why, and how you checked it.
- The change must pass `mise run check`.
- Do not add dependencies lightly; Naran ships their licenses in the app, and every license must be one `release/licenses/about.toml` accepts.

## License of your contribution

Naran is under the [MIT License](LICENSE).
By sending a contribution you agree that it is licensed under the same terms.
The files `app/src/text_input.rs` and `app/src/text_input/` are adapted from GPUI and stay under the Apache License 2.0 ([LICENSES/Apache-2.0.txt](LICENSES/Apache-2.0.txt)).

## Reporting problems

Open an issue for a bug or an idea.
For a security problem, follow [SECURITY.md](SECURITY.md) instead.
