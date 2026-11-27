# Tasks: Worktrees

## Goal

A repository row shows its other worktrees, folded until opened; a worktree can be selected like a repository, and it follows the files in its folder.

## Current Phase

✅ Phase 4: Testing

## Phases

### Phase 1: Requirements & Discovery ✅

- [x] Define requirements
- [x] Analyze existing code
- [x] Write the spec (spec.md)

### Phase 2: Planning & Structure ✅

- [x] Write the implementation plan (plan.md)
- [x] Analyze existing code in detail
- [x] Confirm that the related classes/methods exist

### Phase 3: Implementation ✅

- [x] Step 0: Acceptance Tests - whole-workspace tests for every reachable scenario
- [x] Step 1: Screen - the repository column
- [x] Step 2: Workspace - opening, selecting and stepping
- [x] Step 3: Platform - the watcher
- [x] Step 4: Model - worktrees as entries
- [x] Step 5: Core - the other worktrees

### Phase 4: Testing ✅

- [x] Confirm the full build
- [x] Confirm all tests pass
- [x] Acceptance tests: automated, driven through the window by gpui tests
- [x] Check the screen by hand: a missing folder shows "Missing" and each row has its own bottom line

## Notes

- Keep the Phase status current as work moves.
- Update the Phase status as you go: ⏸️ waiting → 🔄 in progress → ✅ done.
- Record decisions in Technical Decisions in findings.md.
- Record errors in Issues Encountered in findings.md; none are recorded.
