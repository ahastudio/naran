//! Picking files in the Files list of a picked commit: plain, Shift and Cmd
//! clicks, Shift with the arrow keys, and Cmd+A. Two or more picked files
//! show their diffs stacked.

use std::collections::BTreeSet;

use gpui::{Context, Modifiers};
use naran_core::{CommitFile, Error, Staging};

use super::super::Workspace;
use super::super::navigation::stepped;
use super::commit::{CommitKey, Inspecting};
use super::commit_view::badge_letter;
use super::follow;
use super::pictures::image_format;
use super::stack::{
    IMAGE_NOTE, STACK_FILES, StackRequest, StackRow, omitted_note, stack_head, stack_lines,
};

impl Inspecting {
    /// Picks one file alone, where a Shift range starts.
    pub(super) fn pick_only(&mut self, path: &str) {
        self.picked = BTreeSet::from([path.to_string()]);
        self.anchor = Some(path.to_string());
    }

    /// Whether `path` is one of the picked files.
    pub(super) fn holds(&self, path: &str) -> bool {
        self.picked.contains(path)
    }

    fn file_paths(&self) -> Vec<String> {
        self.file_position()
            .map(|(paths, _)| paths)
            .unwrap_or_default()
    }

    /// Picks the files from the anchor (or `to` itself) to `to`, in the order of the list.
    fn pick_range(&mut self, to: &str) {
        let paths = self.file_paths();
        let at = |name: &str| paths.iter().position(|path| path == name).unwrap_or(0);
        let (from, to) = (at(self.anchor.as_deref().unwrap_or(to)), at(to));
        self.picked = paths[from.min(to)..=from.max(to)].iter().cloned().collect();
    }

    /// Adds or removes a file; false when it was removed.
    fn toggle_pick(&mut self, path: &str) -> bool {
        let added = self.picked.insert(path.to_string());
        if !added {
            self.picked.remove(path);
        }
        self.anchor = Some(path.to_string());
        added
    }

    /// Picks files for a click; false when the click took the file off the picks.
    fn pick_by_click(&mut self, path: &str, modifiers: Modifiers) -> bool {
        let range = modifiers.shift && self.anchor.is_some();
        let toggle = (modifiers.platform || modifiers.control) && !self.picked.is_empty();
        match (range, toggle) {
            (true, _) => self.pick_range(path),
            (false, true) => return self.toggle_pick(path),
            (false, false) => self.pick_only(path),
        }
        true
    }

    /// Grows the picks one row from the open file, from their anchor (the
    /// reached row when no file is open). The row and file it reached.
    pub(super) fn extend(&mut self, delta: isize) -> Option<(usize, String)> {
        let (paths, current) = self.file_position()?;
        let next = stepped(paths.len(), current, delta)?;
        self.keep_anchor(&paths[current.unwrap_or(next)]);
        self.pick_range(&paths[next]);
        Some((next, paths[next].clone()))
    }

    /// The anchor stays; without one it is `fallback`.
    fn keep_anchor(&mut self, fallback: &str) {
        let anchor = self.anchor.as_deref().unwrap_or(fallback).to_string();
        self.anchor = Some(anchor);
    }

    /// Moves the open file one row and picks it alone. The row and file it reached.
    pub(super) fn step_to(&mut self, delta: isize) -> Option<(usize, String)> {
        let (paths, current) = self.file_position()?;
        let next = stepped(paths.len(), current, delta)?;
        self.pick_only(&paths[next]);
        Some((next, paths[next].clone()))
    }

    /// Picks every file, anchored at the first.
    fn pick_all(&mut self) {
        let paths = self.file_paths();
        self.anchor = paths.first().cloned();
        self.picked = paths.into_iter().collect();
    }

    /// The picked files in the order of the list.
    fn picked_files(&self) -> Vec<CommitFile> {
        let files = self.files.iter().flatten().flatten();
        files
            .filter(|file| self.picked.contains(&file.path))
            .cloned()
            .collect()
    }
}

impl Workspace {
    /// A click on a file of the commit. Plain: that file only. Shift: from the
    /// anchor to this file. Cmd or Ctrl: add or remove this file. The diff of
    /// the clicked file opens unless the click took it off the picks.
    pub(in crate::workspace) fn click_commit_file(
        &mut self,
        path: String,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let keeps_open = self.pick_commit_file(&path, modifiers);
        self.refresh_stacked(cx);
        if keeps_open {
            self.open_commit_file(path, cx);
        } else {
            cx.notify();
        }
    }

    /// Picks files for a click; false when the click took the file off the picks.
    fn pick_commit_file(&mut self, path: &str, modifiers: Modifiers) -> bool {
        let inspecting = self.shown.inspecting.as_mut();
        inspecting.is_some_and(|inspecting| inspecting.pick_by_click(path, modifiers))
    }

    /// Cmd+A in the Files list of a commit: every file is picked.
    pub(in crate::workspace) fn select_all_commit_files(&mut self, cx: &mut Context<Self>) {
        if let Some(inspecting) = self.shown.inspecting.as_mut() {
            inspecting.pick_all();
        }
        self.refresh_stacked(cx);
        cx.notify();
    }

    /// Shift with Up or Down in the files of a commit: the picks grow by one row.
    pub(super) fn extend_commit_files(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some((next, path)) = self
            .shown
            .inspecting
            .as_mut()
            .and_then(|inspecting| inspecting.extend(delta))
        else {
            return;
        };
        self.scrolls
            .commit_files
            .scroll_to_item(next, follow(delta));
        self.refresh_stacked(cx);
        self.open_commit_file(path, cx);
    }

    /// Keeps the stacked diff of the commit in step with the picked files.
    pub(super) fn refresh_commit_stack(&mut self, cx: &mut Context<Self>) {
        match self.commit_stack_request() {
            Some(reading) if !self.is_stack_shown(&reading.request) => {
                self.begin_commit_stack(reading, cx);
            }
            Some(_) => {}
            None => self.clear_stacked(cx),
        }
    }

    fn is_stack_shown(&self, request: &StackRequest) -> bool {
        let stacked = self.shown.stacked.as_ref();
        stacked.is_some_and(|stacked| request.is_shown(stacked))
    }

    fn begin_commit_stack(&mut self, reading: CommitStack, cx: &mut Context<Self>) {
        self.shown.stacked = Some(reading.request.reading());
        cx.notify();
        Self::read_commit_stack_rows(reading, cx);
    }

    /// What the stacked diff of the commit needs, when two or more files are picked.
    fn commit_stack_request(&self) -> Option<CommitStack> {
        let inspecting = self.shown.inspecting.as_ref()?;
        let files = inspecting.picked_files();
        let paths: Vec<String> = files.iter().map(|file| file.path.clone()).collect();
        (paths.len() >= 2).then(|| CommitStack {
            request: StackRequest {
                repository: inspecting.repository.clone(),
                staging: Staging::Staged,
                paths,
            },
            key: inspecting.key(),
            files,
        })
    }

    fn read_commit_stack_rows(reading: CommitStack, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let rows = read_commit_stack_in_background(cx, &reading).await;
            this.update(cx, |this, cx| {
                let inspecting = this.shown.inspecting.as_ref();
                let shown = inspecting.is_some_and(|inspecting| reading.key.is_shown(inspecting));
                shown.then(|| this.set_stack_rows(&reading.request, rows, cx));
            })
            .ok();
        })
        .detach();
    }
}

/// The stacked diff of a commit to read: the picked files, in list order.
struct CommitStack {
    request: StackRequest,
    key: CommitKey,
    files: Vec<CommitFile>,
}

impl CommitStack {
    /// The files the stack reads, and how many more there are.
    fn shown_files(&self) -> (Vec<CommitFile>, usize) {
        let shown = self.files.iter().take(STACK_FILES).cloned().collect();
        (shown, self.files.len().saturating_sub(STACK_FILES))
    }
}

async fn read_commit_stack_in_background(
    cx: &gpui::AsyncApp,
    reading: &CommitStack,
) -> Result<Vec<StackRow>, Error> {
    let key = reading.key.clone();
    let (files, omitted) = reading.shown_files();
    let job = cx
        .background_executor()
        .spawn(async move { read_commit_stack(&key, &files, omitted) });
    job.await
}

/// Reads the diffs of the picked files of a commit for the stacked view.
pub(super) fn read_commit_stack(
    key: &CommitKey,
    files: &[CommitFile],
    omitted: usize,
) -> Result<Vec<StackRow>, Error> {
    let mut rows = Vec::new();
    for file in files {
        rows.extend(commit_stack_file_rows(key, file)?);
    }
    rows.extend(omitted_note(omitted));
    Ok(rows)
}

fn commit_stack_file_rows(key: &CommitKey, file: &CommitFile) -> Result<Vec<StackRow>, Error> {
    let head = stack_head(&file.path, badge_letter(file.status));
    if image_format(&file.path).is_some() {
        return Ok(vec![head, StackRow::Note(IMAGE_NOTE.into())]);
    }
    let text = naran_core::commit_diff(&key.repository, &key.target(&file.path))?;
    Ok(stack_lines(
        head,
        text.lines().map(str::to_string).collect(),
    ))
}
