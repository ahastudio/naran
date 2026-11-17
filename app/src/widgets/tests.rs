use gpui::{Context, Window};

use super::*;

/// Draws the elements in a window, each inside a box that can be found again.
struct Sheet;

impl Render for Sheet {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        let failed = Entry::Failed {
            path: "/gone".into(),
            error: naran_core::Error::InvalidArgument {
                message: "not a repository".into(),
            },
        };
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .debug_selector(|| "muted".into())
                    .child(muted("Reading…", &theme)),
            )
            .child(
                div()
                    .debug_selector(|| "problem".into())
                    .child(problem("It failed", &theme)),
            )
            .child(
                div()
                    .debug_selector(|| "failed".into())
                    .child(repository_row(0, &failed, &theme)),
            )
    }
}

#[gpui::test]
fn the_notes_and_a_row_that_could_not_be_read_take_room_in_the_window(
    cx: &mut gpui::TestAppContext,
) {
    let (_, cx) = cx.add_window_view(|_, _| Sheet);
    cx.run_until_parked();

    for name in ["muted", "problem", "failed"] {
        let bounds = cx.debug_bounds(name);
        assert!(
            bounds.is_some_and(|bounds| bounds.size.height > px(0.)),
            "{name}"
        );
    }
}
