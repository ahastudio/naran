# Tasks: Viewing Changes

## Goal

Show the changes of picked files, from the working folder and from past commits, as a readable read-only diff, Before and After pictures, or stacked diffs.

## Current Phase

✅ Phase 4: Testing

## Phases

### Phase 1: Requirements & Discovery ✅

- [x] Define requirements
- [x] Analyze existing code
- [x] Write the spec document (spec.md)

### Phase 2: Planning & Structure ✅

- [x] Write the implementation plan (plan.md)
- [x] Analyze existing code in detail
- [x] Confirm that the related files and behaviors exist

### Phase 3: Implementation ✅

- [x] Step 0: Acceptance Tests - 31 workspace tests that drive the whole feature
- [x] Step 1: Screen - rows, drawing, selectable text and the stack view calculations
- [x] Step 2: Workspace - reading, limits, stacking and picking
- [x] Step 3: Model - diff rows
- [x] Step 4: Core - git reads
- [x] Verify compilation at each step

### Phase 4: Testing ✅

- [x] Confirm the full build
- [x] Confirm all tests pass
- [x] Acceptance tests: automated, driven through the window by gpui tests

## Notes

- Update the phase status as you go: ⏸️ waiting → 🔄 in progress → ✅ done.
- Record decisions in Technical Decisions of findings.md.
- Record errors in Issues Encountered of findings.md.
- Window tests added for this feature are in `app/src/workspace/tests/diff_viewing_acceptance.rs` and `app/src/workspace/tests/diff_viewing_files_acceptance.rs`.
