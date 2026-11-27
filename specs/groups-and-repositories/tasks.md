# Tasks: Groups and Repositories

## Goal

Let the user group repositories, add and tidy them, read their state in two columns, and find the last selection restored on the next launch, without ever touching the folders.

## Current Phase

✅ Phase 4: Testing (all phases are done)

## Phases

### Phase 1: Requirements & Discovery ✅

- [x] Define requirements
- [x] Analyze existing code
- [x] Write the spec (spec.md)

### Phase 2: Planning & Structure ✅

- [x] Write the plan (plan.md)
- [x] Analyze existing code in detail
- [x] Check that the related files and tests exist

### Phase 3: Implementation ✅

- [x] Step 0: Acceptance Tests - the whole window driven by clicks, keys and menus
- [x] Step 1: Screen - columns, input, menus, drag and drop
- [x] Step 2: Workspace - workspace actions and selection
- [x] Step 3: Model - group rules and state words
- [x] Step 4: Platform - storage and processes
- [x] Step 5: Move a repository to another group (story 7)
  - [x] Red: acceptance tests and model tests for story 7
  - [x] Green: model rule, workspace action, drop target on the group row, source group on the drag
  - [x] Refactoring: share the drop-target border with the group reorder

### Phase 4: Testing ✅

- [x] Confirm the full build
- [x] Confirm all tests pass
- [x] Acceptance tests: automated, driven through the window by gpui tests
- [x] Confirm `mise run check` passes with story 7

## Notes

- Every spec scenario, story 7 included, has a test line that exists. Scenarios that had no outermost test got one in `app/src/workspace/tests/groups_and_repositories_acceptance.rs`.
- Decisions are in findings.md under Technical Decisions.
- Problems are in findings.md under Issues Encountered.
