use super::*;

fn region(lines: &[&str]) -> Region {
    let region = Region::default();
    region.set_lines(
        lines
            .iter()
            .map(|line| SharedString::from(line.to_string()))
            .collect(),
    );
    region
}

fn select(region: &Region, from: (usize, usize), to: (usize, usize)) {
    let mut inner = region.0.borrow_mut();
    inner.anchor = Some(Spot {
        line: from.0,
        offset: from.1,
    });
    inner.focus = Some(Spot {
        line: to.0,
        offset: to.1,
    });
}

#[test]
fn copies_the_selected_part_of_each_line_across_lines() {
    let region = region(&["first line", "second", "third line"]);
    select(&region, (0, 6), (2, 5));
    assert_eq!(
        region.selected_text().as_deref(),
        Some("line\nsecond\nthird")
    );
    assert_eq!(region.highlight(0), Some(6..10));
    assert_eq!(region.highlight(1), Some(0..6));
    assert_eq!(region.highlight(2), Some(0..5));
}

#[test]
fn a_selection_that_cuts_a_letter_in_two_copies_nothing() {
    let region = region(&["한글"]);
    select(&region, (0, 1), (0, 4));
    assert_eq!(region.selected_text(), None);
}

#[test]
fn a_selection_over_no_lines_or_with_one_end_missing_is_empty() {
    let none = region(&[]);
    select(&none, (0, 0), (0, 1));
    assert_eq!(none.0.borrow().span(), None);
    assert_eq!(none.highlight(3), None);
    let half = region(&["abc"]);
    half.0.borrow_mut().anchor = Some(Spot { line: 0, offset: 1 });
    assert_eq!(half.0.borrow().span(), None);
    half.0.borrow_mut().anchor = None;
    half.0.borrow_mut().focus = Some(Spot { line: 0, offset: 1 });
    assert_eq!(half.0.borrow().span(), None);
}

#[test]
fn pressing_where_nothing_was_painted_selects_nothing() {
    let region = Region::default();
    region.press(point(px(5.), px(5.)));
    assert!(!region.0.borrow().dragging);
    assert!(!region.drag(point(px(6.), px(6.))));
}

#[test]
fn a_selection_made_backwards_reads_the_same() {
    let region = region(&["abc", "def"]);
    select(&region, (1, 2), (0, 1));
    assert_eq!(region.selected_text().as_deref(), Some("bc\nde"));
}

#[test]
fn nothing_is_selected_when_both_ends_are_the_same() {
    let region = region(&["abc"]);
    select(&region, (0, 1), (0, 1));
    assert_eq!(region.selected_text(), None);
    assert_eq!(region.highlight(0), None);
}

#[test]
fn select_all_takes_every_line_and_new_text_drops_the_selection() {
    let region = region(&["ab", "", "cd"]);
    region.select_all();
    assert_eq!(region.selected_text().as_deref(), Some("ab\n\ncd"));
    region.set_lines(vec!["ab".into(), "".into(), "cd".into()]);
    assert!(region.selected_text().is_some());
    region.set_lines(vec!["other".into()]);
    assert_eq!(region.selected_text(), None);
}

#[test]
fn a_region_that_was_not_shown_forgets_its_selection() {
    let region = region(&["abc"]);
    region.select_all();
    // It was shown in the last frame, so the first call keeps it.
    region.begin_frame();
    // Nothing gave it lines in this frame: it is gone.
    region.begin_frame();
    region.set_lines(vec!["abc".into()]);
    assert_eq!(region.selected_text(), None);
}
