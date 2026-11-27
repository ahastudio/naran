//! Repository list, groups, the saved selection, and the pure rules the
//! screen follows. No UI here.

mod diff;
mod files;
mod settings;

use std::path::{Path, PathBuf};

use naran_core::{Error, Operation, Summary, Worktree};

pub use diff::{DiffRow, LineKind, changed_lines, classify, diff_rows, hunks, untracked_as_diff};
pub use files::{FileKind, FileState, change_badge, change_label};
pub use settings::{Group, Selection, Settings, members};

/// A repository as the app last read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Ready(Summary),
    Failed { path: PathBuf, error: Error },
}

impl Entry {
    pub fn read(path: &Path) -> Self {
        match naran_core::summarize(path) {
            Ok(summary) => Self::Ready(summary),
            Err(error) => Self::Failed {
                path: path.to_path_buf(),
                error,
            },
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Ready(summary) => &summary.path,
            Self::Failed { path, .. } => path,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Ready(summary) => summary.name.clone(),
            Self::Failed { path, .. } => path.display().to_string(),
        }
    }

    /// The name of the folder alone, for rows that sit below a repository.
    pub fn folder(&self) -> String {
        match self {
            Self::Ready(summary) => summary.name.clone(),
            Self::Failed { path, .. } => folder_name(path),
        }
    }

    /// Files that differ from HEAD: staged, unstaged and untracked.
    pub fn change_count(&self) -> u32 {
        match self {
            Self::Ready(summary) => {
                let status = &summary.status;
                status.staged + status.unstaged + status.untracked
            }
            Self::Failed { .. } => 0,
        }
    }

    /// The other worktrees of this repository, in the order Git lists them,
    /// each shown the way a repository is.
    pub fn worktrees(&self) -> Vec<Self> {
        match self {
            Self::Ready(summary) => summary.worktrees.iter().map(worktree_entry).collect(),
            Self::Failed { .. } => Vec::new(),
        }
    }

    /// How many other worktrees the repository has.
    pub fn worktree_count(&self) -> usize {
        match self {
            Self::Ready(summary) => summary.worktrees.len(),
            Self::Failed { .. } => 0,
        }
    }

    /// Needs the user's attention before anything else: unreadable,
    /// conflicted, or stopped in the middle of an operation.
    pub fn has_problem(&self) -> bool {
        match self {
            Self::Ready(summary) => summary.status.conflicted > 0 || summary.operation.is_some(),
            Self::Failed { .. } => true,
        }
    }
}

/// A missing or unreadable worktree is a failed entry that keeps its path.
fn worktree_entry(worktree: &Worktree) -> Entry {
    match &worktree.summary {
        Ok(summary) => Entry::Ready(summary.clone()),
        Err(error) => Entry::Failed {
            path: worktree.path.clone(),
            error: error.clone(),
        },
    }
}

/// Moves the selection by `delta` within `paths`, wrapping around at both ends.
pub fn step<'a>(paths: &[&'a Path], current: Option<&Path>, delta: isize) -> Option<&'a Path> {
    let length = paths.len();
    let found = current.and_then(|current| paths.iter().position(|path| *path == current));
    let start = found.map_or_else(|| outside_index(delta, length), |index| index + length);
    let index = start.saturating_add_signed(delta).checked_rem(length)?;
    paths.get(index).copied()
}

/// Where the selection counts from when nothing is selected: just before the
/// first path when moving forward, just after the last one when moving back.
/// Counted from `length`, so the start is never below zero.
fn outside_index(delta: isize, length: usize) -> usize {
    if delta > 0 {
        length.saturating_sub(1)
    } else {
        length * 2
    }
}

/// How urgent a repository's state is. Decides the color of its label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Problem,
    Attention,
    Calm,
}

/// Sums up a repository's state in a few words, most urgent first.
pub fn describe(entry: &Entry) -> (String, Tone) {
    match entry {
        Entry::Failed { error, .. } => (failure_words(error), Tone::Problem),
        Entry::Ready(summary) => {
            let (text, tone) = state_words(summary, entry.change_count());
            (text.to_string(), tone)
        }
    }
}

/// The folder name of an unreadable entry; the whole path when it has none.
fn folder_name(path: &Path) -> String {
    let name = path.file_name().unwrap_or(path.as_os_str());
    name.to_string_lossy().into_owned()
}

/// A gone worktree folder is one word; other failures say what Git said.
fn failure_words(error: &Error) -> String {
    match error {
        Error::InvalidArgument { message } if message == naran_core::MISSING_FOLDER => {
            "Missing".to_string()
        }
        other => other.to_string(),
    }
}

fn state_words(summary: &Summary, changes: u32) -> (&'static str, Tone) {
    if let Some(operation) = summary.operation {
        return (operation_words(operation), Tone::Problem);
    }
    let status = &summary.status;
    let rules = [
        (status.conflicted > 0, "Conflicts", Tone::Problem),
        (changes > 0, "Changes", Tone::Attention),
        (
            status.ahead > 0 && status.behind > 0,
            "Diverged",
            Tone::Attention,
        ),
        (status.ahead > 0, "Ahead", Tone::Attention),
        (status.behind > 0, "Behind", Tone::Attention),
        (status.upstream.is_none(), "No upstream", Tone::Calm),
    ];
    rules
        .into_iter()
        .find(|(applies, ..)| *applies)
        .map_or(("Clean", Tone::Calm), |(_, text, tone)| (text, tone))
}

fn operation_words(operation: Operation) -> &'static str {
    match operation {
        Operation::Merge => "Merging",
        Operation::Rebase => "Rebasing",
        Operation::CherryPick => "Cherry-picking",
        Operation::Revert => "Reverting",
    }
}

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

/// Below the first number of seconds, the second number of seconds is one unit.
const UNITS: [(i64, i64, &str); 5] = [
    (HOUR, MINUTE, "minute"),
    (DAY, HOUR, "hour"),
    (MONTH, DAY, "day"),
    (YEAR, MONTH, "month"),
    (i64::MAX, YEAR, "year"),
];

/// Short description of how long ago something happened.
pub fn elapsed(seconds: i64) -> String {
    let seconds = seconds.max(0);
    if seconds < MINUTE {
        return "just now".to_string();
    }
    let (_, size, unit) = UNITS
        .iter()
        .find(|(limit, ..)| seconds < *limit)
        .unwrap_or(&UNITS[4]);
    plural(seconds / size, unit)
}

fn plural(count: i64, unit: &str) -> String {
    if count == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{count} {unit}s ago")
    }
}

/// A path for the status bar: the home folder as `~`, and when it is still
/// longer than `limit` characters the front is cut, because the end (the
/// repository folder) is the part that matters.
pub fn display_path(path: &Path, home: Option<&Path>, limit: usize) -> String {
    let shown = shorten_home(path, home);
    let count = shown.chars().count();
    if count <= limit {
        return shown;
    }
    format!(
        "…{}",
        shown.chars().skip(count - (limit - 1)).collect::<String>()
    )
}

fn shorten_home(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests;
