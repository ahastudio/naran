use gpui::{
    Bounds, Context, Entity, Modifiers, Render, SharedString, TestAppContext, VisualTestContext,
    div, point, px,
};

use super::*;
use crate::selectable::SelectableText;

/// Two views stacked in one window, each with its own lines.
struct Host {
    texts: Texts,
    info: Vec<&'static str>,
    diff: Vec<&'static str>,
}

fn view(region: &Region, lines: &[&'static str]) -> Div {
    let highlight = gpui::hsla(0.6, 0.8, 0.5, 0.4);
    lines
        .iter()
        .enumerate()
        .fold(div().flex().flex_col(), |view, (index, line)| {
            view.child(SelectableText::new(region, index, *line).highlighted(highlight))
        })
}

impl Host {
    fn fill(&self) {
        let strings =
            |lines: &[&'static str]| lines.iter().map(|line| SharedString::from(*line)).collect();
        self.texts.info.set_lines(strings(&self.info));
        self.texts.diff.set_lines(strings(&self.diff));
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.texts.begin_frame();
        self.fill();
        let info = area(view(&self.texts.info, &self.info), &self.texts, Which::Info);
        let diff = area(view(&self.texts.diff, &self.diff), &self.texts, Which::Diff);
        div().size_full().flex().flex_col().child(info).child(diff)
    }
}

fn open<'a>(
    cx: &'a mut TestAppContext,
    info: &[&'static str],
    diff: &[&'static str],
) -> (Entity<Host>, &'a mut VisualTestContext) {
    let (info, diff) = (info.to_vec(), diff.to_vec());
    let (host, cx) = cx.add_window_view(|window, cx| {
        let texts = Texts::new(cx.focus_handle());
        window.focus(&texts.focus);
        Host { texts, info, diff }
    });
    cx.run_until_parked();
    (host, cx)
}

fn texts(host: &Entity<Host>, cx: &mut VisualTestContext) -> Texts {
    host.read_with(cx, |host, _| host.texts.clone())
}

/// The place of a painted line: its left edge and its middle.
fn place(texts: &Texts, which: Which, line: usize) -> Point<Pixels> {
    let bounds: Bounds<Pixels> = texts.region(which).placed_bounds(line).unwrap();
    point(bounds.left(), bounds.center().y)
}

#[gpui::test]
fn dragging_across_painted_text_selects_it_and_a_plain_click_selects_nothing(
    cx: &mut TestAppContext,
) {
    let (host, cx) = open(cx, &[], &["alpha beta", "gamma"]);
    let texts = texts(&host, cx);
    let start = place(&texts, Which::Diff, 0);
    let end = place(&texts, Which::Diff, 1) + point(px(1000.), px(0.));

    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());

    assert_eq!(texts.copied().as_deref(), Some("alpha beta\ngamma"));
    assert!(texts.diff.dragged());
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(start, MouseButton::Left, Modifiers::none());
    assert_eq!(texts.copied(), None);
    assert!(!texts.diff.dragged());
}

#[gpui::test]
fn pressing_in_one_view_clears_the_selection_of_the_other(cx: &mut TestAppContext) {
    let (host, cx) = open(cx, &["info line"], &["diff line"]);
    let texts = texts(&host, cx);
    let (diff, info) = (place(&texts, Which::Diff, 0), place(&texts, Which::Info, 0));
    cx.simulate_mouse_down(diff, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(
        diff + point(px(1000.), px(0.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_up(diff, MouseButton::Left, Modifiers::none());
    assert_eq!(texts.copied().as_deref(), Some("diff line"));

    cx.simulate_mouse_down(info, MouseButton::Left, Modifiers::none());

    assert_eq!(texts.copied(), None);
    assert_eq!(texts.last.get(), Which::Info);
    texts.select_all();
    assert_eq!(texts.copied().as_deref(), Some("info line"));
}

#[gpui::test]
fn moving_without_a_press_changes_nothing(cx: &mut TestAppContext) {
    let (host, cx) = open(cx, &[], &["alpha"]);
    let texts = texts(&host, cx);

    cx.simulate_mouse_move(
        place(&texts, Which::Diff, 0),
        MouseButton::Left,
        Modifiers::none(),
    );

    assert_eq!(texts.copied(), None);
    assert!(!texts.diff.drag(point(px(0.), px(0.))));
}

#[gpui::test]
fn a_drag_that_stays_on_the_same_place_does_not_change_the_selection(cx: &mut TestAppContext) {
    let (host, cx) = open(cx, &[], &["alpha"]);
    let texts = texts(&host, cx);
    let start = place(&texts, Which::Diff, 0);

    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());

    assert!(!texts.diff.drag(start));
}

#[gpui::test]
fn an_empty_line_is_painted_as_a_space_and_nothing_can_be_selected_in_no_lines(
    cx: &mut TestAppContext,
) {
    let (host, cx) = open(cx, &[], &["", "x"]);
    let texts = texts(&host, cx);

    assert!(texts.diff.placed_bounds(0).is_some());
    let empty = Region::default();
    empty.select_all();
    assert_eq!(empty.selected_text(), None);
}

#[gpui::test]
fn a_selection_ending_in_the_first_row_of_wrapped_text_paints_only_that_row(
    cx: &mut TestAppContext,
) {
    struct Wrapped(Texts);
    impl Render for Wrapped {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.0.begin_frame();
            let text = "alpha beta gamma delta epsilon zeta eta theta";
            self.0.info.set_lines(vec![text.into()]);
            let line = SelectableText::new(&self.0.info, 0, text)
                .wrapped()
                .highlighted(gpui::hsla(0.6, 0.8, 0.5, 0.4));
            area(div().w(px(80.)).child(line), &self.0, Which::Info)
        }
    }
    let (wrapped, cx) = cx.add_window_view(|window, cx| {
        let texts = Texts::new(cx.focus_handle());
        window.focus(&texts.focus);
        Wrapped(texts)
    });
    cx.run_until_parked();
    let texts = wrapped.read_with(cx, |wrapped, _| wrapped.0.clone());
    let top_left = texts.info.placed_bounds(0).unwrap().origin;
    let start = top_left + point(px(0.), px(5.));

    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(
        start + point(px(30.), px(0.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.run_until_parked();

    assert!(texts.copied().is_some_and(|text| text.starts_with("alp")));
}
