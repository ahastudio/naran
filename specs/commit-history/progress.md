# Progress Log

> **Update this file when you finish a phase or hit a problem.**

## Session 2026-11-16

### Phase 1: Requirements & Discovery ✅

**Work log**:

1. Requirements were defined from README.md: graph, columns, paging, Go to HEAD, ref buttons, read-only picking and stepping.
2. The code was explored in `core/src/history.rs`, `core/src/graph.rs`, `app/src/workspace/detail/` and `app/src/workspace/layout.rs`.
3. The spec was written with seven stories and numbered scenarios, each with a Test line.

**Created/modified files**:

- `specs/commit-history/README.md` (new)
- `specs/commit-history/spec.md` (new)

### Phase 2: Planning & Structure ✅

**Work log**:

1. The plan was written Outside-In in six steps: the window, the ref buttons, picking and stepping, the workspace, the graph, the core.
2. Every spec scenario was mapped to a window-driven test or to an inner test marked as not reachable through the window.

**Created/modified files**:

- `specs/commit-history/plan.md` (new)
- `specs/commit-history/tasks.md` (new)

### Phase 3: Implementation ✅

**Work log**:

1. Step 0: the end-to-end tests are in `app/src/workspace/tests/commit_history_acceptance.rs`, `app/src/workspace/tests/layout.rs`, `app/src/workspace/tests/history.rs`, `app/src/workspace/tests/keys.rs` and `app/src/workspace/tests/scrolling.rs`; one line was appended to `app/src/workspace/tests.rs`.
2. Step 1: the rows and buttons in `app/src/workspace/detail/history_view.rs` and `app/src/workspace/detail/history_rows.rs` are covered by `app/src/workspace/detail/tests/commits.rs`.
3. Step 2: paging, Go to HEAD, picking and the graph decisions in `app/src/workspace/detail/` are covered by `app/src/workspace/detail/tests/commits.rs` and `app/src/workspace/detail/tests/lanes.rs`.
4. Step 3: the app model has no tests of its own for this feature.
5. Step 4: reading and lanes in `core/src/history.rs`, `core/src/log.rs` and `core/src/graph.rs` are covered by `core/tests/reading.rs` and `core/src/graph.rs`.
6. The twelve tests in `commit_history_acceptance.rs` passed against the existing code, so no source file was changed.

**Created/modified files**:

- `app/src/workspace/tests/commit_history_acceptance.rs` (new)
- `app/src/workspace/tests.rs` (one line appended)
- `specs/commit-history/findings.md` (new)

### Phase 4: Testing ✅

**Work log**:

1. The 44 tests named in spec.md and plan.md passed: 36 in `naran-app`, 4 in `core/tests/reading.rs` and 4 in `core/src/graph.rs`.

**Created/modified files**:

- `specs/commit-history/progress.md` (new)

## Test Results

| Test | Input | Expected | Actual | Status |
| ---- | ----- | -------- | ------ | ------ |
| `the_uncommitted_row_comes_first_under_the_column_heads_when_there_are_changes` | alpha (clean) then beta (changed) | no Uncommitted row for alpha; for beta heads, Uncommitted row and first commit row are in that order | passed | ✅ |
| `a_repository_whose_history_cannot_be_read_shows_the_problem_instead_of_rows` | the .git folder removed, then refresh | the problem is shown and the history is an error | passed | ✅ |
| `the_column_heads_of_the_commit_list_stay_on_one_line_when_it_is_narrow` | wide side columns leave a narrow list | column heads keep the same height (one line) | passed | ✅ |
| `scrolling_to_the_end_adds_older_commits_and_a_short_history_adds_none` | 230 commits, wheel down 6000 px; then a 1-commit repository | more than 200 listed; the short one stays at 1 | passed | ✅ |
| `the_history_loads_older_commits_as_the_end_is_reached` | 230 commits, load more, refresh | 200 first, then more; rows equal commits; no more; refresh keeps length | passed | ✅ |
| `go_to_head_picks_the_head_commit_each_time_it_is_pressed` | click Go to HEAD, another row, Go to HEAD | HEAD commit picked both times | passed | ✅ |
| `the_go_to_head_button_reads_older_pages_until_head_shows_up` | HEAD 230 commits down, click Go to HEAD | more than 200 listed and HEAD picked | passed | ✅ |
| `go_to_head_finds_a_head_commit_that_is_older_than_the_loaded_page` | same, by calling the jump | older commits read, HEAD picked, jump flag cleared | passed | ✅ |
| `go_to_head_picks_the_head_commit_when_it_is_already_loaded` | uncommitted file, jump | HEAD commit picked | passed | ✅ |
| `a_repository_without_commits_has_an_empty_list_and_nothing_to_pick` | empty repository selected, Down in the commit list | history empty, nothing picked | passed | ✅ |
| `the_all_refs_button_adds_the_commits_only_git_keeps_refs_for_and_a_second_press_removes_them` | a commit left only by a backup ref, click All refs twice | 1 commit, then 2, then 1 | passed | ✅ |
| `the_remote_branches_button_removes_remote_only_commits_and_a_second_press_brings_them_back` | a commit left only by a remote branch, click Remote branches twice | 2 commits, then 1, then 2 | passed | ✅ |
| `the_ref_buttons_save_their_choice_and_a_restart_keeps_it` | click both buttons, read layout file, reopen | saved and reopened choice is everything on, remotes off | passed | ✅ |
| `a_layout_file_in_the_older_form_still_sets_the_refs` | layout file with the old name All, then click All refs | refs ALL, then VISIBLE | passed | ✅ |
| `clicking_a_commit_shows_its_message_and_files_and_the_arrows_step_through_the_list` | click a commit, then Up, Down, Down, Up | info and file shown; pick None, hash, hash, None | passed | ✅ |
| `clicking_a_commit_with_a_body_and_another_committer_reads_both` | click a commit with a body and another committer | message holds the body; author differs from committer | passed | ✅ |
| `picking_a_commit_shows_its_files_and_diff_and_the_uncommitted_row_brings_the_lists_back` | pick a commit with one file | files, diff, message and parent read; Uncommitted row clears the pick | passed | ✅ |
| `a_clean_repository_starts_on_its_top_commit` | clean repository opened | top commit picked | passed | ✅ |
| `arrow_keys_move_through_the_list_that_was_clicked_last` | Down, Down, Up in files; then commits | file moves; commit picked then back to Uncommitted | passed | ✅ |
| `stepping_stays_inside_the_list` | stepped over 0 and 3 rows | stays within the list | passed | ✅ |
| `the_arrow_keys_scroll_the_commit_list_to_follow_the_picked_row` | 100 commits, 40 down and 40 up | scrolled at least 600 px down, 0 after going back | passed | ✅ |
| `the_commit_list_rows_and_buttons_pick_and_move` | click rows, file, Uncommitted row, pane, buttons | picked, file opened, back to uncommitted, pane Commits, refs ALL | passed | ✅ |
| `hiding_remote_branches_reloads_the_list_and_flips_the_label` | click Remote branches, toggle again | remotes off with label off, then on | passed | ✅ |
| `a_page_for_a_list_that_changed_meanwhile_is_dropped` | page for another skip count | loading flag cleared, list unchanged | passed | ✅ |
| `details_that_cannot_be_read_stay_empty_and_a_failed_page_adds_nothing` | missing repository, then a failed page | empty rows, no more pages, HEAD jump Missing | passed | ✅ |
| `a_history_that_could_not_be_read_has_no_rows_pages_head_or_top` | a history that failed | no page, no commit rows, no HEAD jump, no clean top | passed | ✅ |
| `a_repository_without_commits_has_no_head_to_go_to` | empty repository, jump and step | notice says HEAD is not in the commit list; no jump pending | passed | ✅ |
| `showing_all_refs_reloads_the_list_and_flips_the_label` | toggle All refs twice | refs ALL then VISIBLE; label on | passed | ✅ |
| `a_commit_with_a_body_and_another_committer_shows_both` | commit with body and other committer | message holds the body; author differs from committer | passed | ✅ |
| `the_link_goes_straight_to_a_head_at_the_top` | HEAD on the top commit | link in lane 0, ends at the dot, nothing below | passed | ✅ |
| `the_link_passes_the_commits_above_a_head_on_another_branch` | HEAD on a feature branch under newer commits | link passes row 0 in lane 1 and ends at row 1 | passed | ✅ |
| `the_link_takes_a_lane_of_its_own_when_the_head_lane_is_busy_above` | commit above uses the HEAD lane | link lane 1, HEAD lane 0 | passed | ✅ |
| `without_a_head_commit_in_the_list_there_is_no_link` | no HEAD label | no link | passed | ✅ |
| `a_detached_head_counts_too` | commit labeled HEAD | link position 0 | passed | ✅ |
| `head_is_the_commit_labeled_head_or_head_arrow_branch` | labels HEAD, HEAD -> branch, HEADLINE, origin/HEAD | only the first two count | passed | ✅ |
| `the_graph_is_wide_enough_for_the_link_lane` | link lane 5 | graph wider than without the link | passed | ✅ |
| `history_pages_join_into_the_full_history` | pages of 2 over 5 commits | pages equal the full history and graph; a page past the end is empty | passed | ✅ |
| `history_hides_refs_git_keeps_for_itself_unless_asked` | backup ref of Git | visible 1 commit, all 2, page 1 | passed | ✅ |
| `remote_branches_can_be_left_out_of_the_history` | remote-only commit | 2 commits with remotes, 1 without, for both everything values | passed | ✅ |
| `refs_read_the_older_saved_form` | "All", "Visible", flags | same refs; flags round-trip | passed | ✅ |
| `linear_history_stays_in_one_lane` | three commits in a line | all in lane 0, nothing passing | passed | ✅ |
| `merge_opens_and_closes_a_lane` | merge of two parents | second lane opens and closes | passed | ✅ |
| `unrelated_tip_takes_a_free_lane` | two commits with one parent | second in lane 1, lane 0 passes | passed | ✅ |
| `handles_missing_parents_in_a_truncated_log` | one commit with an unlisted parent | outgoing lane 0 | passed | ✅ |

## Error Log

| Timestamp | Error | Attempt | Resolution |
| --------- | ----- | ------- | ---------- |
| none recorded | none recorded | 0 | none recorded |

## 5-Question Reboot Check

Questions to recover context when resuming work:

| Question | Answer |
| -------- | ------ |
| 1. Which phase is it now? | Phase 4 (done) |
| 2. What is next? | Nothing open; re-run the listed tests when the commit list code changes |
| 3. What is the goal? | Read a repository's commit history as a branch graph with paging, Go to HEAD, ref buttons and read-only commit picking |
| 4. What have I learned so far? | See findings.md |
| 5. What is done? | The feature, its six-step plan, the listed passing tests and these six documents |
