//! Picking a commit or the Uncommitted changes row in the commit list, and
//! opening the files of the picked commit.

use gpui::{Context, ImageFormat, px};
use naran_core::{CommitFile, Error};

use super::super::Workspace;
use super::commit::{
    CommitFileKey, CommitKey, Inspecting, read_commit_detail_in_background,
    read_commit_files_in_background, read_commit_lines_in_background,
    read_commit_pictures_in_background,
};
use super::pictures::image_format;

impl Workspace {
    /// Picks the Uncommitted changes row: the Staged and Unstaged lists.
    pub(in crate::workspace) fn pick_uncommitted(&mut self, cx: &mut Context<Self>) {
        if self.shown.inspecting.take().is_some() {
            self.shown.stacked = None;
            self.refresh_stacked(cx);
            cx.notify();
        }
    }

    /// Picks a past commit in the commit list. Its files and diff replace the
    /// Staged and Unstaged lists, and nothing can be staged from there.
    pub(in crate::workspace) fn pick_commit(&mut self, hash: String, cx: &mut Context<Self>) {
        if let Some(key) = self.commit_key(hash) {
            self.show_commit(&key);
            cx.notify();
            Self::read_commit_detail(key.clone(), cx);
            Self::read_commit_files(key, cx);
        }
    }

    /// The key to pick a commit with, unless it is already picked.
    fn commit_key(&self, hash: String) -> Option<CommitKey> {
        let inspecting = self.shown.inspecting.as_ref();
        if inspecting.is_some_and(|inspecting| inspecting.hash == hash) {
            return None;
        }
        let repository = self.selection.repository.clone()?;
        Some(CommitKey { repository, hash })
    }

    fn show_commit(&mut self, key: &CommitKey) {
        let refs = self.commit_refs(&key.hash);
        self.shown.stacked = None;
        self.shown.inspecting = Some(Inspecting::new(key, refs));
    }

    /// Branch and tag names on a commit of the loaded list.
    fn commit_refs(&self, hash: &str) -> Vec<String> {
        let history = self
            .shown
            .detail
            .as_ref()
            .and_then(|detail| detail.history.as_ref().ok());
        let commit = history.and_then(|commits| commits.iter().find(|commit| commit.hash == hash));
        commit.map(|commit| commit.refs.clone()).unwrap_or_default()
    }

    fn read_commit_detail(key: CommitKey, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let info = read_commit_detail_in_background(cx, &key).await;
            this.update(cx, |this, cx| {
                this.with_inspecting(&key, |inspecting| inspecting.detail = Some(info), cx)
            })
            .ok();
        })
        .detach();
    }

    fn read_commit_files(key: CommitKey, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let files = read_commit_files_in_background(cx, &key).await;
            this.update(cx, |this, cx| this.set_commit_files(&key, files, cx))
                .ok();
        })
        .detach();
    }

    /// Lists the files of the picked commit and opens the first one.
    fn set_commit_files(
        &mut self,
        key: &CommitKey,
        files: Result<Vec<CommitFile>, Error>,
        cx: &mut Context<Self>,
    ) {
        let first = files
            .as_ref()
            .ok()
            .and_then(|files| files.first())
            .map(|file| file.path.clone());
        let shown = self.with_inspecting(
            key,
            |inspecting| {
                inspecting.files = Some(files);
                if let Some(first) = &first {
                    inspecting.pick_only(first);
                }
            },
            cx,
        );
        if let (true, Some(first)) = (shown, first) {
            self.open_commit_file(first, cx);
        }
    }

    /// Changes the picked commit when it is still the one `key` was read for;
    /// tells whether it was.
    pub(super) fn with_inspecting(
        &mut self,
        key: &CommitKey,
        change: impl FnOnce(&mut Inspecting),
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(inspecting) = self
            .shown
            .inspecting
            .as_mut()
            .filter(|inspecting| key.is_shown(inspecting))
        else {
            return false;
        };
        change(inspecting);
        cx.notify();
        true
    }

    /// Changes the picked commit when its open file is still the one `key` was read for.
    pub(super) fn with_inspected_file(
        &mut self,
        key: &CommitFileKey,
        change: impl FnOnce(&mut Inspecting),
        cx: &mut Context<Self>,
    ) {
        let inspecting = self.shown.inspecting.as_ref();
        if inspecting.is_some_and(|inspecting| inspecting.is_open(&key.file)) {
            self.with_inspecting(&key.commit, change, cx);
        }
    }

    /// Shows the diff of one file of the picked commit.
    pub(in crate::workspace) fn open_commit_file(&mut self, file: String, cx: &mut Context<Self>) {
        let Some(commit) = self.shown.inspecting.as_ref().map(Inspecting::key) else {
            return;
        };
        let key = CommitFileKey { commit, file };
        self.show_commit_file(&key, cx);
        Self::read_commit_file(key, cx);
    }

    fn read_commit_file(key: CommitFileKey, cx: &mut Context<Self>) {
        match image_format(&key.file) {
            Some(format) => Self::read_commit_pictures(key, format, cx),
            None => Self::read_commit_lines(key, cx),
        }
    }

    pub(super) fn show_commit_file(&mut self, key: &CommitFileKey, cx: &mut Context<Self>) {
        if let Some(inspecting) = self.shown.inspecting.as_mut() {
            // Another file starts at its left edge, not where the last one was scrolled to.
            if !inspecting.is_open(&key.file) {
                self.scrolls
                    .diff_sideways
                    .set_offset(gpui::point(px(0.), px(0.)));
            }
            inspecting.start_file(key.file.clone());
            cx.notify();
        }
    }

    fn read_commit_pictures(key: CommitFileKey, format: ImageFormat, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let pictures = read_commit_pictures_in_background(cx, &key, format).await;
            this.update(cx, |this, cx| {
                let show = |inspecting: &mut Inspecting| inspecting.pictures = Some(pictures);
                this.with_inspected_file(&key, show, cx);
            })
            .ok();
        })
        .detach();
    }

    fn read_commit_lines(key: CommitFileKey, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let lines = read_commit_lines_in_background(cx, &key).await;
            this.update(cx, |this, cx| {
                this.with_inspected_file(&key, |inspecting| inspecting.lines = Some(lines), cx);
            })
            .ok();
        })
        .detach();
    }
}
