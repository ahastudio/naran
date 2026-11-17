//! Reading the selected repository in the background, adding older pages of
//! its history as the list is scrolled, and going to the commit HEAD is on.

use std::path::{Path, PathBuf};

use gpui::{AsyncApp, Context, ScrollStrategy};
use naran_core::log::Commit;
use naran_core::{Error, Staging};

use super::super::Workspace;
use super::super::state::Outcome;
use super::loaded::{Detail, HISTORY_PAGE, HeadJump};
use crate::model::FileState;

/// The open file and its side, and the state of that file in the new changes.
type ViewedState = (Option<(String, Staging)>, Option<FileState>);

async fn read_history_page(
    cx: &AsyncApp,
    path: PathBuf,
    skip: usize,
    refs: naran_core::Refs,
) -> Result<Vec<Commit>, Error> {
    let reading = cx.background_executor().spawn(async move {
        let page = naran_core::Page {
            skip,
            limit: HISTORY_PAGE,
        };
        naran_core::history_page(&path, page, refs)
    });
    reading.await
}

async fn read_detail(cx: &AsyncApp, path: PathBuf, limit: usize, refs: naran_core::Refs) -> Detail {
    let reading = cx
        .background_executor()
        .spawn(async move { Detail::read(&path, limit, refs) });
    reading.await
}

impl Workspace {
    pub(in crate::workspace) fn load_detail(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let limit = self.history_limit(&path);
        let refs = self.layout.refs;
        cx.spawn(async move |this, cx| {
            let detail = read_detail(cx, path.clone(), limit, refs).await;
            this.update(cx, |this, cx| this.apply_detail(&path, detail, cx))
                .ok();
        })
        .detach();
    }

    /// A refresh reads as many commits as are already listed, so the list
    /// keeps its length and the scroll position.
    fn history_limit(&self, path: &Path) -> usize {
        let listed = self
            .shown
            .detail
            .as_ref()
            .filter(|detail| detail.path == path);
        let loaded = listed.and_then(|detail| detail.history.as_ref().ok());
        loaded.map_or(0, Vec::len).max(HISTORY_PAGE)
    }

    fn apply_detail(&mut self, path: &Path, detail: Detail, cx: &mut Context<Self>) {
        // Ignore results that arrive after the user moved on.
        if self.selection.repository.as_deref() != Some(path) {
            return;
        }
        let viewed = self.viewed_state(path, &detail);
        self.shown.detail = Some(detail);
        self.settle_after_detail(viewed, cx);
    }

    /// The open file and side, and the state of that file in the new changes.
    fn viewed_state(&self, path: &Path, detail: &Detail) -> ViewedState {
        let viewing = self
            .shown
            .viewing
            .as_ref()
            .filter(|viewing| viewing.repository == path);
        let viewed = viewing.map(|viewing| (viewing.file.clone(), viewing.staging));
        let found = viewed
            .as_ref()
            .and_then(|(file, _)| detail.file_state(file));
        (viewed, found)
    }

    fn settle_after_detail(&mut self, viewed: ViewedState, cx: &mut Context<Self>) {
        self.drop_unlisted_marks();
        self.refresh_stacked(cx);
        self.pick_top_when_clean(cx);
        self.follow_viewed(viewed, cx);
        cx.notify();
    }

    /// Without uncommitted changes the top commit is the first row.
    fn pick_top_when_clean(&mut self, cx: &mut Context<Self>) {
        if self.shown.inspecting.is_some() {
            return;
        }
        let Some(top) = self.shown.detail.as_ref().and_then(Detail::clean_top) else {
            return;
        };
        self.pick_commit(top, cx);
    }

    /// Keeps the open diff on the side it was on while the file still has
    /// changes there, otherwise follows it across.
    fn follow_viewed(&mut self, (viewed, found): ViewedState, cx: &mut Context<Self>) {
        let reopened = viewed.zip(found).and_then(|((file, staging), state)| {
            let side = state.open_side(staging);
            side.map(|side| (file, side, state))
        });
        match reopened {
            Some((file, side, state)) => self.open_file(file, side, state, cx),
            None => self.shown.viewing = None,
        }
    }

    /// Reads the next page of older commits and adds it to the list.
    pub(in crate::workspace) fn load_more_history(&mut self, cx: &mut Context<Self>) {
        let Some((path, skip)) = self.start_history_page() else {
            return;
        };
        let refs = self.layout.refs;
        cx.spawn(async move |this, cx| {
            let page = read_history_page(cx, path.clone(), skip, refs).await;
            this.update(cx, |this, cx| this.add_history_page(&path, skip, page, cx))
                .ok();
        })
        .detach();
    }

    /// The repository and the count of commits already listed, once a page is due.
    pub(super) fn start_history_page(&mut self) -> Option<(PathBuf, usize)> {
        let detail = self.shown.detail.as_mut()?;
        let skip = detail.begin_page()?;
        Some((detail.path.clone(), skip))
    }

    pub(super) fn add_history_page(
        &mut self,
        path: &Path,
        skip: usize,
        page: Result<Vec<Commit>, Error>,
        cx: &mut Context<Self>,
    ) {
        let detail = self.shown.detail.as_mut();
        if let Some(detail) = detail.filter(|detail| detail.path == path) {
            detail.finish_page(skip, page);
            self.follow_jump(cx);
            cx.notify();
        }
    }

    /// Go to HEAD goes on once an older page came in.
    fn follow_jump(&mut self, cx: &mut Context<Self>) {
        if self.progress.jumping_to_head {
            self.jump_to_head(cx);
        }
    }

    /// Shows or hides the refs Git keeps for itself (the `refs/original`
    /// backup of `filter-branch`, the stash, notes) in the commit graph.
    pub(in crate::workspace) fn toggle_all_refs(&mut self, cx: &mut Context<Self>) {
        self.layout.refs.everything = !self.layout.refs.everything;
        self.reload_refs(cx);
    }

    /// Shows or hides the remote branches in the commit graph.
    pub(in crate::workspace) fn toggle_remote_branches(&mut self, cx: &mut Context<Self>) {
        self.layout.refs.remotes = !self.layout.refs.remotes;
        self.reload_refs(cx);
    }

    fn reload_refs(&mut self, cx: &mut Context<Self>) {
        self.save_layout();
        if let Some(path) = self.shown.detail.as_ref().map(|detail| detail.path.clone()) {
            self.load_detail(path, cx);
        }
        cx.notify();
    }

    /// Scrolls the commit list to the commit HEAD is on and picks it. When the
    /// commit is older than what is loaded, older commits are read until it is
    /// found or there are no more.
    pub(in crate::workspace) fn jump_to_head(&mut self, cx: &mut Context<Self>) {
        let Some(jump) = self.shown.detail.as_ref().and_then(Detail::head_jump) else {
            return;
        };
        self.progress.jumping_to_head = jump == HeadJump::Older;
        self.follow_head(jump, cx);
    }

    fn follow_head(&mut self, jump: HeadJump, cx: &mut Context<Self>) {
        match jump {
            HeadJump::At { row, hash } => self.scroll_to_head(row, hash, cx),
            HeadJump::Older => self.load_more_history(cx),
            HeadJump::Missing => {
                self.report("HEAD is not in the commit list.", Outcome::Failed);
                cx.notify();
            }
        }
    }

    fn scroll_to_head(&mut self, row: usize, hash: String, cx: &mut Context<Self>) {
        self.scrolls
            .history
            .scroll_to_item_strict(row, ScrollStrategy::Center);
        self.pick_commit(hash, cx);
        cx.notify();
    }
}
