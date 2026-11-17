//! The message and meta information of the picked commit, shown under its
//! files; every line of it can be selected and copied.

use std::cell::RefCell;

use gpui::{AnyElement, Div, FontWeight, Hsla, SharedString, div, prelude::*, px};
use naran_core::CommitDetail;

use super::commit::Inspecting;
use crate::selectable::{Region, SelectableText, Texts, Which, area};
use crate::theme::Theme;
use crate::widgets::{badge, muted, problem};

/// The message and meta information of the picked commit, above its files.
pub(super) fn render_commit_info(
    inspecting: &Inspecting,
    theme: &Theme,
    texts: &Texts,
) -> AnyElement {
    match commit_detail_content(inspecting, theme) {
        Ok(detail) => render_commit_detail(
            &Info {
                inspecting,
                detail,
                theme,
            },
            texts,
        ),
        Err(shown) => shown,
    }
}

/// The detail of the picked commit, or the element that says why there is none.
fn commit_detail_content<'a>(
    inspecting: &'a Inspecting,
    theme: &Theme,
) -> Result<&'a CommitDetail, AnyElement> {
    match &inspecting.detail {
        None => Err(muted("Reading…", theme)),
        Some(Err(error)) => Err(problem(error.to_string(), theme)),
        Some(Ok(detail)) => Ok(detail),
    }
}

fn render_commit_detail(info: &Info, texts: &Texts) -> AnyElement {
    let lines = Collect {
        region: &texts.info,
        lines: RefCell::new(Vec::new()),
        highlight: info.theme.accent.opacity(0.35),
    };
    let column = info.column(&lines);
    texts.info.set_lines(lines.lines.into_inner());
    area(column, texts, Which::Info).into_any_element()
}

/// The picked commit and what was read of it.
struct Info<'a> {
    inspecting: &'a Inspecting,
    detail: &'a CommitDetail,
    theme: &'a Theme,
}

/// Who wrote or committed a commit, and when.
struct Person<'a> {
    name: &'a str,
    email: &'a str,
    date: &'a str,
}

impl Info<'_> {
    /// The subject, the body and the rows of meta information. The selectable
    /// lines are collected in the order they are built.
    fn column(&self, lines: &Collect) -> Div {
        let (subject, body) = split_message(&self.detail.message);
        let subject = div()
            .font_weight(FontWeight::SEMIBOLD)
            .child(lines.block(&subject));
        let body = (!body.is_empty()).then(|| self.body_block(lines, &body));
        let head = div().flex().flex_col().gap_1().p_2().child(subject);
        head.children(body)
            .child(div().h(px(4.)))
            .children(self.rows(lines))
    }

    fn body_block(&self, lines: &Collect, body: &str) -> Div {
        div()
            .text_color(self.theme.muted)
            .text_size(px(12.))
            .child(lines.block(body))
    }

    fn rows(&self, lines: &Collect) -> Vec<Div> {
        let detail = self.detail;
        let author = Person {
            name: &detail.author,
            email: &detail.author_email,
            date: &detail.author_date,
        };
        let rows = [
            self.row("Commit", mono_block(lines, &detail.hash)),
            self.row("Parents", mono_block(lines, &parent_list(detail))),
            self.row("Author", person_block(lines, &author)),
        ];
        rows.into_iter()
            .chain(self.committer_row(lines))
            .chain(self.labels_row())
            .collect()
    }

    /// The committer, when it is not the author at the same time.
    fn committer_row(&self, lines: &Collect) -> Option<Div> {
        let detail = self.detail;
        let committer = Person {
            name: &detail.committer,
            email: &detail.committer_email,
            date: &detail.committer_date,
        };
        let shown = !same_signature(detail);
        shown.then(|| self.row("Committer", person_block(lines, &committer)))
    }

    /// The branch and tag names, when the commit has any.
    fn labels_row(&self) -> Option<Div> {
        let labelled = !self.inspecting.refs.is_empty();
        labelled.then(|| self.row("Labels", labels_block(self.inspecting, self.theme)))
    }

    fn row(&self, label: &'static str, value: AnyElement) -> Div {
        let name = div()
            .flex_none()
            .w(px(68.))
            .text_size(px(11.))
            .text_color(self.theme.muted)
            .child(label);
        div()
            .flex()
            .gap_2()
            .child(name)
            .child(div().flex_1().min_w(px(0.)).child(value))
    }
}

fn split_message(message: &str) -> (String, String) {
    match message.split_once('\n') {
        Some((subject, rest)) => (subject.to_string(), rest.trim_matches('\n').to_string()),
        None => (message.to_string(), String::new()),
    }
}

fn parent_list(detail: &CommitDetail) -> String {
    if detail.parents.is_empty() {
        return "none (first commit)".to_string();
    }
    let short: Vec<String> = detail
        .parents
        .iter()
        .map(|parent| short_hash(parent))
        .collect();
    short.join("  ")
}

pub(super) fn short_hash(hash: &str) -> String {
    hash.chars().take(7).collect()
}

fn same_signature(detail: &CommitDetail) -> bool {
    detail.author == detail.committer
        && detail.author_email == detail.committer_email
        && detail.author_date == detail.committer_date
}

fn mono_block(lines: &Collect, value: &str) -> AnyElement {
    let mono = div().font_family("Menlo").text_size(px(11.));
    mono.child(lines.block(value)).into_any_element()
}

fn person_block(lines: &Collect, person: &Person) -> AnyElement {
    let (name, email, date) = (person.name, person.email, person.date);
    lines
        .block(&format!("{name} <{email}>\n{date}"))
        .into_any_element()
}

fn labels_block(inspecting: &Inspecting, theme: &Theme) -> AnyElement {
    let names = inspecting
        .refs
        .iter()
        .map(|name| badge(name.clone(), theme.accent));
    div()
        .flex()
        .flex_wrap()
        .gap_1()
        .children(names)
        .into_any_element()
}

/// The lines of the commit information that can be selected, in the order
/// they are built.
struct Collect<'a> {
    region: &'a Region,
    lines: RefCell<Vec<SharedString>>,
    highlight: Hsla,
}

impl Collect<'_> {
    /// `text` as one selectable, wrapping line for each of its lines.
    fn block(&self, text: &str) -> Div {
        let lines = text.split('\n').map(|line| self.selectable_line(line));
        div().flex().flex_col().children(lines)
    }

    fn selectable_line(&self, line: &str) -> SelectableText {
        let line: SharedString = line.replace('\t', "    ").into();
        let index = self.push(line.clone());
        SelectableText::new(self.region, index, line)
            .wrapped()
            .highlighted(self.highlight)
    }

    /// Adds a line to the selectable ones and tells its place.
    fn push(&self, line: SharedString) -> usize {
        let mut lines = self.lines.borrow_mut();
        lines.push(line);
        lines.len() - 1
    }
}
