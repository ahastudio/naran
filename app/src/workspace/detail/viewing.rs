//! The open file of the Staged or Unstaged list: its diff, or its two
//! pictures, read in the background, and the lines picked in it.

use std::collections::BTreeSet;
use std::path::PathBuf;

use gpui::{AsyncApp, Context, ImageFormat, px};
use naran_core::{Error, Staging};

use super::super::Workspace;
use super::marks::ListedFile;
use super::pictures::{PictureSource, Pictures, image_format, pictures_from, read_picture_bytes};
use super::versions::{Compared, read_diff};
use crate::model::{FileKind, FileState};

/// What a file shown again keeps: its lines and the picks on them.
type Kept = (Option<Result<Vec<String>, Error>>, BTreeSet<usize>);

/// The diff of one file, with the lines picked for staging or unstaging.
pub(in crate::workspace) struct Viewing {
    pub(in crate::workspace) repository: PathBuf,
    pub(in crate::workspace) file: String,
    /// The list the file is open from, which decides the diff shown.
    pub(in crate::workspace) staging: Staging,
    pub(in crate::workspace) kind: FileKind,
    pub(in crate::workspace) lines: Option<Result<Vec<String>, Error>>,
    /// The pictures, when the file is an image.
    pub(in crate::workspace) pictures: Option<Result<Pictures, Error>>,
    /// Indexes into `lines`.
    pub(in crate::workspace) picked: BTreeSet<usize>,
}

impl Viewing {
    fn new(key: &ViewKey, kind: FileKind, kept: Kept) -> Self {
        let (lines, picked) = kept;
        Self {
            repository: key.repository.clone(),
            file: key.file.clone(),
            staging: key.staging,
            kind,
            lines,
            pictures: None,
            picked,
        }
    }

    /// Untracked and conflicted files can only be staged as a whole.
    pub(super) fn partial(&self) -> bool {
        self.kind == FileKind::Tracked
    }

    /// The place of this file and side among the rows of both lists.
    pub(super) fn row_in(&self, rows: &[ListedFile]) -> Option<usize> {
        rows.iter()
            .position(|row| self.shows(&row.path, row.staging))
    }

    /// Whether this is the diff of `file` in the `staging` list.
    pub(super) fn shows(&self, file: &str, staging: Staging) -> bool {
        self.file == file && self.staging == staging
    }

    /// Shows the diff that was read. Picks stay only while the lines are the
    /// same, since they are indexes into them.
    fn read(&mut self, lines: Result<Vec<String>, Error>) {
        if !matches!((&self.lines, &lines), (Some(Ok(old)), Ok(new)) if old == new) {
            self.picked.clear();
        }
        self.lines = Some(lines);
    }

    /// Picks a line, or takes it off the picks when it was picked.
    fn toggle(&mut self, index: usize) {
        if !self.picked.remove(&index) {
            self.picked.insert(index);
        }
    }
}

/// The repository, file and side a read of the open file was started for.
struct ViewKey {
    repository: PathBuf,
    file: String,
    staging: Staging,
}

impl ViewKey {
    fn is_shown(&self, viewing: &Viewing) -> bool {
        viewing.repository == self.repository && viewing.shows(&self.file, self.staging)
    }
}

async fn read_pictures_in_background(
    cx: &AsyncApp,
    key: &ViewKey,
    source: PictureSource,
    format: ImageFormat,
) -> Result<Pictures, Error> {
    let (repository, file) = (key.repository.clone(), key.file.clone());
    let reading = cx
        .background_executor()
        .spawn(async move { read_picture_bytes(&repository, &file, &source) });
    reading.await.map(|bytes| pictures_from(format, bytes))
}

async fn read_diff_in_background(
    cx: &AsyncApp,
    key: &ViewKey,
    compared: Compared,
) -> Result<Vec<String>, Error> {
    let (repository, file) = (key.repository.clone(), key.file.clone());
    let reading = cx
        .background_executor()
        .spawn(async move { read_diff(&repository, &file, compared) });
    reading.await
}

impl Workspace {
    /// Opens the diff of a file from the Staged or Unstaged list.
    pub(in crate::workspace) fn open_file(
        &mut self,
        file: String,
        staging: Staging,
        state: FileState,
        cx: &mut Context<Self>,
    ) {
        if let Some(repository) = self.selection.repository.clone() {
            let key = ViewKey {
                repository,
                file,
                staging,
            };
            self.show_file(&key, state);
            cx.notify();
            Self::read_viewing(key, state, cx);
        }
    }

    /// The lines already read and the picks stay while the same file is shown
    /// again, as a refresh does; another file starts at its left edge.
    fn show_file(&mut self, key: &ViewKey, state: FileState) {
        let same = self
            .shown
            .viewing
            .as_ref()
            .filter(|viewing| key.is_shown(viewing));
        let kept = same.map(|viewing| (viewing.lines.clone(), viewing.picked.clone()));
        if same.is_none() {
            self.scrolls
                .diff_sideways
                .set_offset(gpui::point(px(0.), px(0.)));
        }
        self.shown.viewing = Some(Viewing::new(key, state.kind, kept.unwrap_or_default()));
    }

    fn read_viewing(key: ViewKey, state: FileState, cx: &mut Context<Self>) {
        let compared = Compared::of(key.staging, state);
        match image_format(&key.file) {
            Some(format) => Self::read_pictures(key, compared, format, cx),
            None => Self::read_lines(key, compared, cx),
        }
    }

    fn read_pictures(
        key: ViewKey,
        compared: Compared,
        format: ImageFormat,
        cx: &mut Context<Self>,
    ) {
        let source = PictureSource::Working(compared);
        cx.spawn(async move |this, cx| {
            let pictures = read_pictures_in_background(cx, &key, source, format).await;
            this.update(cx, |this, cx| {
                this.with_viewing(&key, |viewing| viewing.pictures = Some(pictures), cx);
            })
            .ok();
        })
        .detach();
    }

    fn read_lines(key: ViewKey, compared: Compared, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let lines = read_diff_in_background(cx, &key, compared).await;
            this.update(cx, |this, cx| {
                this.with_viewing(&key, |viewing| viewing.read(lines), cx);
            })
            .ok();
        })
        .detach();
    }

    /// Changes the open file when it is still the one `key` was read for.
    fn with_viewing(
        &mut self,
        key: &ViewKey,
        change: impl FnOnce(&mut Viewing),
        cx: &mut Context<Self>,
    ) {
        let viewing = self.shown.viewing.as_mut();
        if let Some(viewing) = viewing.filter(|viewing| key.is_shown(viewing)) {
            change(viewing);
            cx.notify();
        }
    }

    pub(in crate::workspace) fn toggle_pick(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(viewing) = self.shown.viewing.as_mut() {
            viewing.toggle(index);
        }
        cx.notify();
    }
}
