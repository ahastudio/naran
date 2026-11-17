# Tasks: Groups and Repositories

## Goal

Let the user group repositories, add and tidy them, read their state in two columns, and find the last selection restored on the next launch, without ever touching the folders.

## Current Phase

✅ Phase 4: Testing

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

### Phase 4: Testing ✅

- [x] Confirm the full build
- [x] Confirm all tests pass
- [x] Acceptance tests: automated, driven through the window by gpui tests

## Notes

- Every spec scenario has a test line; scenarios that had no outermost test got one in `app/src/workspace/tests/groups_and_repositories_acceptance.rs`.
- Decisions are in findings.md under Technical Decisions.
- Problems are in findings.md under Issues Encountered.
