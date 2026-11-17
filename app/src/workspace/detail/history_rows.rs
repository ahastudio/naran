//! One row of the commit list: the Uncommitted changes row, or a commit with
//! its part of the graph, its labels, date, author and short hash.

use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{AnyElement, Context, Div, FontWeight, Hsla, Stateful, div, prelude::*, px};
use naran_core::log::Commit;

use super::super::Workspace;
use super::HISTORY_ROW;
use super::commit::Inspecting;
use super::commit_info::short_hash;
use super::lanes::{HeadLink, graph_width, head_position, is_head_ref, link_piece};
use super::loaded::Detail;
use super::painting::{GraphCell, graph_cell, uncommitted_cell};
use super::truncating;
use crate::model;
use crate::theme::Theme;
use crate::widgets::{badge, head_badge};

/// A column cell of a row of the commit list.
pub(super) fn history_cell(width: f32, theme: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(width))
        .truncate()
        .text_color(theme.muted)
}

/// A row of the commit list: its frame, on `background` when it is the
/// picked one.
fn history_row(
    id: impl Into<gpui::ElementId>,
    background: Option<Hsla>,
    theme: &Theme,
) -> Stateful<Div> {
    let hover = theme.hover;
    div()
        .id(id)
        .w_full()
        .h(px(HISTORY_ROW))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .cursor_pointer()
        .when_some(background, Styled::bg)
        .hover(move |style| style.bg(hover))
}

fn unix_now() -> i64 {
    let since = SystemTime::now().duration_since(UNIX_EPOCH);
    let seconds = since.map(|duration| i64::try_from(duration.as_secs()));
    seconds.ok().and_then(Result::ok).unwrap_or_default()
}

/// What the rows of the commit list are built from.
pub(super) struct HistoryView<'a> {
    theme: Theme,
    detail: &'a Detail,
    commits: &'a [Commit],
    link: Option<HeadLink>,
    graph_width: f32,
    now: i64,
    head: Option<usize>,
    picked: Option<&'a str>,
    uncommitted: usize,
}

impl<'a> HistoryView<'a> {
    pub(super) fn new(
        detail: &'a Detail,
        commits: &'a [Commit],
        inspecting: Option<&'a Inspecting>,
    ) -> Self {
        let link = detail.history_link(commits);
        Self {
            theme: Theme::dark(),
            detail,
            commits,
            link,
            graph_width: graph_width(&detail.rows, link),
            now: unix_now(),
            head: head_position(commits),
            picked: inspecting.map(|inspecting| inspecting.hash.as_str()),
            uncommitted: detail.changes.as_ref().map_or(0, Vec::len),
        }
    }

    /// The Uncommitted changes row comes first when there are changes.
    fn offset(&self) -> usize {
        usize::from(self.uncommitted > 0)
    }

    pub(super) fn row(&self, index: usize, cx: &mut Context<Workspace>) -> Option<AnyElement> {
        match index.checked_sub(self.offset()) {
            None => Some(self.uncommitted_row(cx)),
            Some(position) => self
                .commits
                .get(position)
                .map(|commit| self.commit_row(position, commit, cx)),
        }
    }

    fn uncommitted_row(&self, cx: &mut Context<Workspace>) -> AnyElement {
        let theme = &self.theme;
        let title = truncating()
            .font_weight(FontWeight::SEMIBOLD)
            .child("Uncommitted changes");
        let graph = uncommitted_cell(self.link, self.graph_width, theme.background);
        let background = self.picked.is_none().then_some(theme.selected);
        history_row("uncommitted", background, theme)
            .debug_selector(|| "uncommitted-row".into())
            .on_click(cx.listener(|this, _, _, cx| this.pick_uncommitted(cx)))
            .child(graph)
            .child(title)
            .child(history_cell(96., theme).child(""))
            .child(history_cell(140., theme).child(format!("{} files", self.uncommitted)))
            .child(history_cell(64., theme).child(""))
            .into_any_element()
    }

    fn commit_row(
        &self,
        position: usize,
        commit: &Commit,
        cx: &mut Context<Workspace>,
    ) -> AnyElement {
        let hash = commit.hash.clone();
        let theme = &self.theme;
        let picked = self.picked == Some(commit.hash.as_str());
        let short = history_cell(64., theme).font_family("Menlo");
        history_row(
            ("commit", position),
            picked.then_some(theme.selected),
            theme,
        )
        .debug_selector(|| format!("commit-row-{position}"))
        .on_click(cx.listener(move |this, _, _, cx| this.pick_commit(hash.clone(), cx)))
        .child(self.graph_for(position))
        .child(self.description(position, commit))
        .child(history_cell(96., theme).child(model::elapsed(self.now - commit.time)))
        .child(history_cell(140., theme).child(commit.author.clone()))
        .child(short.child(short_hash(&commit.hash)))
        .into_any_element()
    }

    /// Whether the commit at `position` is the one HEAD is on.
    fn is_head(&self, position: usize) -> bool {
        self.head == Some(position)
    }

    fn graph_for(&self, position: usize) -> AnyElement {
        let drawing = self.detail.rows.get(position).map(|row| GraphCell {
            row: row.clone(),
            background: self.theme.background,
            piece: self.link.and_then(|link| link_piece(link, position)),
            head_ring: self.is_head(position).then_some(self.theme.text),
        });
        graph_cell(drawing, self.graph_width)
    }

    fn description(&self, position: usize, commit: &Commit) -> Div {
        let refs = commit.refs.iter().map(|name| ref_badge(name, &self.theme));
        let subject = truncating()
            .when(self.is_head(position), |subject| {
                subject.font_weight(FontWeight::SEMIBOLD)
            })
            .child(commit.subject.clone());
        div()
            .flex_1()
            .min_w(px(0.))
            .flex()
            .items_center()
            .gap_1()
            .children(refs)
            .child(subject)
    }
}

fn ref_badge(name: &str, theme: &Theme) -> AnyElement {
    if is_head_ref(name) {
        return head_badge(name.to_string(), theme).into_any_element();
    }
    badge(name.to_string(), theme.accent).into_any_element()
}
