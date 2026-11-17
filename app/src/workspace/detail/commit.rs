//! A commit picked in the commit list, and the reads of its message, its
//! files and the diff of one of them, each done in the background.

use std::collections::BTreeSet;
use std::path::PathBuf;

use gpui::{AsyncApp, ImageFormat};
use naran_core::{CommitDetail, CommitFile, Error};

use super::pictures::{PictureSource, Pictures, pictures_from, read_picture_bytes};

/// A commit picked in the commit list: its changed files and the diff of
/// one of them. Read-only.
pub(in crate::workspace) struct Inspecting {
    pub(in crate::workspace) repository: PathBuf,
    pub(in crate::workspace) hash: String,
    pub(in crate::workspace) files: Option<Result<Vec<CommitFile>, Error>>,
    /// The message and meta information, read in the background.
    pub(in crate::workspace) detail: Option<Result<CommitDetail, Error>>,
    /// Branch and tag names on the commit, from the commit list.
    pub(in crate::workspace) refs: Vec<String>,
    pub(in crate::workspace) file: Option<String>,
    /// The files picked in the list, and where a Shift range starts.
    pub(in crate::workspace) picked: BTreeSet<String>,
    pub(in crate::workspace) anchor: Option<String>,
    pub(in crate::workspace) lines: Option<Result<Vec<String>, Error>>,
    pub(in crate::workspace) pictures: Option<Result<Pictures, Error>>,
}

impl Inspecting {
    /// A commit just picked: nothing of it is read yet.
    pub(super) fn new(key: &CommitKey, refs: Vec<String>) -> Self {
        Self {
            repository: key.repository.clone(),
            hash: key.hash.clone(),
            detail: None,
            refs,
            files: None,
            file: None,
            picked: BTreeSet::new(),
            anchor: None,
            lines: None,
            pictures: None,
        }
    }

    pub(super) fn key(&self) -> CommitKey {
        CommitKey {
            repository: self.repository.clone(),
            hash: self.hash.clone(),
        }
    }

    /// Opens a file of the commit: its diff is read again.
    pub(super) fn start_file(&mut self, file: String) {
        self.file = Some(file);
        self.lines = None;
        self.pictures = None;
    }

    /// The paths of the commit's files and the place of the open one.
    pub(super) fn file_position(&self) -> Option<(Vec<String>, Option<usize>)> {
        let files = self.files.as_ref()?.as_ref().ok()?;
        let paths: Vec<String> = files.iter().map(|file| file.path.clone()).collect();
        let current = paths
            .iter()
            .position(|path| Some(path) == self.file.as_ref());
        Some((paths, current))
    }

    /// Whether `file` is the open file of the commit.
    pub(super) fn is_open(&self, file: &str) -> bool {
        self.file.as_deref() == Some(file)
    }
}

/// The commit a read was started for.
#[derive(Clone)]
pub(super) struct CommitKey {
    pub(super) repository: PathBuf,
    pub(super) hash: String,
}

impl CommitKey {
    /// A file of this commit.
    pub(super) fn target<'a>(&'a self, file: &'a str) -> naran_core::FileInCommit<'a> {
        naran_core::FileInCommit {
            hash: &self.hash,
            file,
        }
    }

    pub(super) fn is_shown(&self, inspecting: &Inspecting) -> bool {
        inspecting.repository == self.repository && inspecting.hash == self.hash
    }
}

/// A file of the commit a read was started for.
pub(super) struct CommitFileKey {
    pub(super) commit: CommitKey,
    pub(super) file: String,
}

pub(super) async fn read_commit_detail_in_background(
    cx: &AsyncApp,
    key: &CommitKey,
) -> Result<CommitDetail, Error> {
    let (repository, hash) = (key.repository.clone(), key.hash.clone());
    let reading = cx
        .background_executor()
        .spawn(async move { naran_core::commit_detail(&repository, &hash) });
    reading.await
}

pub(super) async fn read_commit_files_in_background(
    cx: &AsyncApp,
    key: &CommitKey,
) -> Result<Vec<CommitFile>, Error> {
    let (repository, hash) = (key.repository.clone(), key.hash.clone());
    let reading = cx
        .background_executor()
        .spawn(async move { naran_core::commit_files(&repository, &hash) });
    reading.await
}

pub(super) async fn read_commit_pictures_in_background(
    cx: &AsyncApp,
    key: &CommitFileKey,
    format: ImageFormat,
) -> Result<Pictures, Error> {
    let source = PictureSource::Commit(key.commit.hash.clone());
    let (repository, file) = (key.commit.repository.clone(), key.file.clone());
    let reading = cx
        .background_executor()
        .spawn(async move { read_picture_bytes(&repository, &file, &source) });
    reading.await.map(|bytes| pictures_from(format, bytes))
}

pub(super) async fn read_commit_lines_in_background(
    cx: &AsyncApp,
    key: &CommitFileKey,
) -> Result<Vec<String>, Error> {
    let (repository, hash) = (key.commit.repository.clone(), key.commit.hash.clone());
    let file = key.file.clone();
    let reading = cx.background_executor().spawn(async move {
        naran_core::commit_diff(
            &repository,
            &naran_core::FileInCommit {
                hash: &hash,
                file: &file,
            },
        )
    });
    reading
        .await
        .map(|text| text.lines().map(str::to_string).collect())
}
