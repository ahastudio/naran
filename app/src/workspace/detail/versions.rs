//! Which two versions of a working file are compared, and reading the diff
//! between them.

use std::path::Path;

use naran_core::{Error, Staging};

use crate::model::{self, FileState};

/// Untracked files bigger than this are not read for the diff view.
pub(super) const UNTRACKED_LIMIT: u64 = 512 * 1024;

/// Which two versions of a working file a diff or a picture compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Compared {
    /// HEAD and the index: the file is in the Staged list.
    Staged,
    /// The index and the folder: the file is in the Unstaged list.
    Unstaged,
    /// Only the folder: git does not know the file yet.
    Untracked,
}

impl Compared {
    /// What a file of the `staging` list compares. An untracked file is only
    /// ever listed as unstaged.
    pub(super) fn of(staging: Staging, state: FileState) -> Self {
        match staging {
            Staging::Staged => Self::Staged,
            Staging::Unstaged if state.untracked() => Self::Untracked,
            Staging::Unstaged => Self::Unstaged,
        }
    }
}

/// The diff lines of a working file; an untracked file is shown whole, as
/// added lines.
pub(super) fn read_diff(
    repository: &Path,
    file: &str,
    compared: Compared,
) -> Result<Vec<String>, Error> {
    let staging = match compared {
        Compared::Untracked => return Ok(read_untracked(&repository.join(file))),
        Compared::Staged => Staging::Staged,
        Compared::Unstaged => Staging::Unstaged,
    };
    let text = naran_core::file_diff(repository, file, staging)?;
    Ok(text.lines().map(str::to_string).collect())
}

pub(super) fn read_untracked(path: &Path) -> Vec<String> {
    let size = std::fs::metadata(path).map_or(0, |metadata| metadata.len());
    if size > UNTRACKED_LIMIT {
        return vec![format!(
            "The file is too large to show ({} KB).",
            size / 1024
        )];
    }
    std::fs::read_to_string(path).map_or_else(
        |_| vec!["The file is not text.".to_string()],
        |content| model::untracked_as_diff(&content),
    )
}
