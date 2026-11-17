//! The diffs of the picked files one after another, shown when two or more
//! files of one list are picked, and read again when the picks change.

use std::path::{Path, PathBuf};

use gpui::{AsyncApp, Context};
use naran_core::{Change, Error, Staging};

use super::super::Workspace;
use super::pictures::image_format;
use super::versions::{Compared, read_diff};
use crate::model::{self, FileState};

/// How many picked files the stacked diff reads, and how many lines of each.
pub(super) const STACK_FILES: usize = 40;
const STACK_LINES: usize = 500;

pub(super) const IMAGE_NOTE: &str = "An image: pick it alone to see Before and After.";

/// One row of the diff that stacks the picked files.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::workspace) enum StackRow {
    /// The head of one file.
    File {
        path: String,
        badge: char,
    },
    Line(String),
    Note(String),
}

/// The diffs of the picked files, one after another, shown when two or more
/// files are picked. Read only; staging is done from the list header.
pub(in crate::workspace) struct Stacked {
    pub(in crate::workspace) repository: PathBuf,
    pub(in crate::workspace) staging: Staging,
    pub(in crate::workspace) paths: Vec<String>,
    pub(in crate::workspace) rows: Option<Result<Vec<StackRow>, Error>>,
}

/// A file the stacked diff reads: its path, what it compares, its badge.
pub(super) struct StackJob {
    pub(super) path: String,
    pub(super) compared: Compared,
    pub(super) badge: char,
}

/// The picked files the stacked diff is for.
pub(super) struct StackRequest {
    pub(super) repository: PathBuf,
    pub(super) staging: Staging,
    pub(super) paths: Vec<String>,
}

impl StackRequest {
    pub(super) fn is_shown(&self, stacked: &Stacked) -> bool {
        let same_files = stacked.staging == self.staging && stacked.paths == self.paths;
        stacked.repository == self.repository && same_files
    }

    /// The stacked diff of these files, being read.
    pub(super) fn reading(&self) -> Stacked {
        Stacked {
            repository: self.repository.clone(),
            staging: self.staging,
            paths: self.paths.clone(),
            rows: None,
        }
    }
}

pub(super) fn stack_job(changes: &[Change], path: &str, staging: Staging) -> Option<StackJob> {
    let change = changes.iter().find(|change| change.path == path)?;
    Some(StackJob {
        path: path.to_string(),
        compared: Compared::of(staging, FileState::of(change)),
        badge: model::change_badge(change, staging),
    })
}

async fn read_stack_in_background(
    cx: &AsyncApp,
    request: &StackRequest,
    jobs: Vec<StackJob>,
    omitted: usize,
) -> Result<Vec<StackRow>, Error> {
    let repository = request.repository.clone();
    let reading = cx
        .background_executor()
        .spawn(async move { read_stack(&repository, &jobs, omitted) });
    reading.await
}

/// Reads the diffs of the picked files for the stacked view, in the background.
pub(super) fn read_stack(
    repository: &Path,
    jobs: &[StackJob],
    omitted: usize,
) -> Result<Vec<StackRow>, Error> {
    let mut rows = Vec::new();
    for job in jobs {
        rows.extend(stack_file_rows(repository, job)?);
    }
    rows.extend(omitted_note(omitted));
    Ok(rows)
}

/// The name of one file, then its diff lines.
pub(super) fn stack_file_rows(repository: &Path, job: &StackJob) -> Result<Vec<StackRow>, Error> {
    let head = stack_head(&job.path, job.badge);
    if image_format(&job.path).is_some() {
        return Ok(vec![head, StackRow::Note(IMAGE_NOTE.into())]);
    }
    let lines = read_diff(repository, &job.path, job.compared)?;
    Ok(stack_lines(head, lines))
}

/// The note that says how many picked files the stack leaves out, if any.
pub(super) fn omitted_note(omitted: usize) -> Option<StackRow> {
    (omitted > 0).then(|| StackRow::Note(format!("… and {omitted} more files are not shown.")))
}

pub(super) fn stack_head(path: &str, badge: char) -> StackRow {
    StackRow::File {
        path: path.to_string(),
        badge,
    }
}

pub(super) fn stack_lines(head: StackRow, lines: Vec<String>) -> Vec<StackRow> {
    // The hunks are what matters here; the file head lines repeat the name.
    let first = lines
        .iter()
        .position(|line| line.starts_with("@@"))
        .unwrap_or(0);
    let hidden = (lines.len() - first).saturating_sub(STACK_LINES);
    let shown = lines
        .into_iter()
        .skip(first)
        .take(STACK_LINES)
        .map(StackRow::Line);
    let note = (hidden > 0).then(|| StackRow::Note(format!("… {hidden} more lines in this file.")));
    [head].into_iter().chain(shown).chain(note).collect()
}

impl Workspace {
    /// Keeps the stacked diff in step with the picked files: it exists while
    /// two or more files are picked and is read again when they change.
    pub(in crate::workspace) fn refresh_stacked(&mut self, cx: &mut Context<Self>) {
        if self.shown.inspecting.is_some() {
            return self.refresh_commit_stack(cx);
        }
        match self.stack_request() {
            Some(request) => self.restack(request, cx),
            None => self.clear_stacked(cx),
        }
    }

    /// Reads the stack again unless the same files are already shown.
    fn restack(&mut self, request: StackRequest, cx: &mut Context<Self>) {
        let stacked = self.shown.stacked.as_ref();
        if !stacked.is_some_and(|stacked| request.is_shown(stacked)) {
            let jobs = self.begin_stack(&request, cx);
            Self::read_stack_rows(request, jobs, cx);
        }
    }

    pub(super) fn clear_stacked(&mut self, cx: &mut Context<Self>) {
        if self.shown.stacked.take().is_some() {
            cx.notify();
        }
    }

    /// The picked files the stacked diff is read for, when there are two or more.
    fn stack_request(&self) -> Option<StackRequest> {
        let repository = self.selection.repository.clone()?;
        let staging = self.shown.marked.staging;
        let listed = self.list_files(staging).into_iter().map(|(file, _)| file);
        let paths: Vec<String> = listed
            .filter(|file| self.shown.marked.paths.contains(file))
            .collect();
        (paths.len() >= 2).then_some(StackRequest {
            repository,
            staging,
            paths,
        })
    }

    /// Shows the stacked diff as being read; returns what the read needs.
    fn begin_stack(&mut self, request: &StackRequest, cx: &mut Context<Self>) -> Vec<StackJob> {
        let jobs = self.stack_jobs(request);
        self.shown.stacked = Some(request.reading());
        cx.notify();
        jobs
    }

    /// What the background job needs for every file: its state and its badge.
    pub(super) fn stack_jobs(&self, request: &StackRequest) -> Vec<StackJob> {
        let Some(Ok(changes)) = self.shown.detail.as_ref().map(|detail| &detail.changes) else {
            return Vec::new();
        };
        let shown = request.paths.iter().take(STACK_FILES);
        shown
            .filter_map(|path| stack_job(changes, path, request.staging))
            .collect()
    }

    fn read_stack_rows(request: StackRequest, jobs: Vec<StackJob>, cx: &mut Context<Self>) {
        let omitted = request.paths.len().saturating_sub(STACK_FILES);
        cx.spawn(async move |this, cx| {
            let rows = read_stack_in_background(cx, &request, jobs, omitted).await;
            this.update(cx, |this, cx| this.set_stack_rows(&request, rows, cx))
                .ok();
        })
        .detach();
    }

    pub(super) fn set_stack_rows(
        &mut self,
        request: &StackRequest,
        rows: Result<Vec<StackRow>, Error>,
        cx: &mut Context<Self>,
    ) {
        let stacked = self.shown.stacked.as_mut();
        if let Some(stacked) = stacked.filter(|stacked| request.is_shown(stacked)) {
            stacked.rows = Some(rows);
            cx.notify();
        }
    }
}
