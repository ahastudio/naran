use gpui::{Context, IntoElement, Render, TestAppContext, VisualTestContext};

use super::*;

fn bounds(x: f32, y: f32, width: f32, height: f32) -> Bounds<Pixels> {
    Bounds::new(point(px(x), px(y)), size(px(width), px(height)))
}

#[test]
fn a_saved_window_comes_back_exactly() {
    for window_bounds in [
        WindowBounds::Windowed(bounds(120., 90., 1000., 700.)),
        WindowBounds::Maximized(bounds(33.5, 44.5, 777., 555.)),
        WindowBounds::Fullscreen(bounds(0., 0., 400., 300.)),
    ] {
        let saved = SavedWindow::of(window_bounds);
        let json = serde_json::to_string(&Some(saved)).unwrap();
        let back: SavedWindow = serde_json::from_str::<Option<SavedWindow>>(&json)
            .unwrap()
            .unwrap();
        assert_eq!(back, saved);
        assert_eq!(back.bounds(), window_bounds.get_bounds());
    }
}

#[test]
fn a_saved_window_opens_in_its_mode_and_an_unsaved_one_in_the_default_place() {
    let screens = [bounds(0., 0., 1920., 1080.)];
    let default_bounds = bounds(10., 10., 100., 100.);
    let place = bounds(5., 5., 300., 200.);
    for window_bounds in [
        WindowBounds::Windowed(place),
        WindowBounds::Maximized(place),
        WindowBounds::Fullscreen(place),
    ] {
        let saved = Some(SavedWindow::of(window_bounds));
        assert_eq!(start_bounds(saved, &screens, default_bounds), window_bounds);
    }
    assert_eq!(
        start_bounds(None, &screens, default_bounds),
        WindowBounds::Windowed(default_bounds)
    );
}

#[test]
fn the_window_has_the_app_name_and_the_given_place() {
    let place = WindowBounds::Windowed(bounds(1., 2., 3., 4.));
    let options = window_options(place);
    assert_eq!(options.window_bounds, Some(place));
    assert_eq!(options.titlebar.unwrap().title, Some("Naran".into()));
}

fn started(cx: &mut TestAppContext, directory: &Path) -> gpui::AnyWindowHandle {
    cx.update(|cx| start(cx, directory));
    cx.update(|cx| cx.windows()[0])
}

#[gpui::test]
fn the_app_opens_one_window_with_a_workspace_and_saves_where_it_is(cx: &mut TestAppContext) {
    let directory = tempfile::tempdir().unwrap();
    let window = started(cx, directory.path());
    assert_eq!(cx.update(|cx| cx.windows().len()), 1);
    let file = directory.path().join(WINDOW_FILE);

    let view = VisualTestContext::from_window(window, cx);
    view.simulate_resize(size(px(900.), px(600.)));
    view.run_until_parked();

    let saved: Option<SavedWindow> = store::load(&file);
    assert_eq!(
        saved.map(|saved| (saved.width, saved.height)),
        Some((900., 600.))
    );
}

#[gpui::test]
fn the_window_is_saved_once_more_when_it_closes_and_closing_the_last_one_quits(
    cx: &mut TestAppContext,
) {
    let directory = tempfile::tempdir().unwrap();
    let window = started(cx, directory.path());
    let file = directory.path().join(WINDOW_FILE);
    assert!(!file.exists());

    let mut view = VisualTestContext::from_window(window, cx);
    assert!(view.simulate_close(), "the window agrees to close");

    assert!(file.exists(), "saved on close");
    cx.update(|cx| {
        window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    });
    cx.run_until_parked();
    cx.update(|cx| assert!(cx.windows().is_empty()));
}

#[gpui::test]
fn closing_one_of_two_windows_does_not_quit(cx: &mut TestAppContext) {
    let directory = tempfile::tempdir().unwrap();
    let first = started(cx, directory.path());
    cx.update(|cx| {
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Blank))
            .unwrap()
    });
    assert_eq!(cx.update(|cx| cx.windows().len()), 2);

    cx.update(|cx| {
        first
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    });
    cx.run_until_parked();

    assert_eq!(cx.update(|cx| cx.windows().len()), 1);
}

struct Blank;

impl Render for Blank {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

#[gpui::test]
fn a_saved_window_opens_where_it_was(cx: &mut TestAppContext) {
    let directory = tempfile::tempdir().unwrap();
    let place = bounds(10., 20., 700., 500.);
    let saved = SavedWindow::of(WindowBounds::Windowed(place));
    store::save(&directory.path().join(WINDOW_FILE), &Some(saved)).unwrap();

    let window = started(cx, directory.path());

    let opened = cx.update(|cx| window.update(cx, |_, window, _| window.bounds()));
    assert_eq!(opened.unwrap(), place);
}

#[gpui::test]
fn the_app_quits_when_its_window_could_not_open(cx: &mut TestAppContext) {
    cx.update(|cx| quit_unless_opened(&Ok::<(), ()>(()), cx));
    cx.update(|cx| quit_unless_opened(&Err::<(), ()>(()), cx));
}

#[test]
fn a_file_without_a_mode_is_a_normal_window() {
    let saved: SavedWindow =
        serde_json::from_str(r#"{"x":1.0,"y":2.0,"width":3.0,"height":4.0}"#).unwrap();
    assert_eq!(saved.mode, Mode::Windowed);
}

#[test]
fn the_size_never_changes_only_the_position_moves_onto_a_screen() {
    let screens = [bounds(0., 0., 1920., 1080.)];
    // On the screen: untouched, even when small or half outside.
    assert_eq!(
        keep_on_screen(bounds(100., 100., 300., 200.), &screens),
        bounds(100., 100., 300., 200.)
    );
    assert_eq!(
        keep_on_screen(bounds(1800., 900., 800., 600.), &screens),
        bounds(1800., 900., 800., 600.)
    );
    // On a screen that is gone: same size, moved inside the first screen.
    assert_eq!(
        keep_on_screen(bounds(4000., 3000., 800., 600.), &screens),
        bounds(1120., 480., 800., 600.)
    );
    // Bigger than the screen: the size stays, the corner goes to the screen corner.
    assert_eq!(
        keep_on_screen(bounds(5000., 5000., 3000., 2000.), &screens),
        bounds(0., 0., 3000., 2000.)
    );
}
