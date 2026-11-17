//! Small labels: outlined, filled for HEAD, and the letter of a changed file.

use gpui::{Div, FontWeight, Hsla, SharedString, div, prelude::*, px};

use crate::theme::Theme;

pub fn badge(text: impl Into<SharedString>, color: Hsla) -> Div {
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .text_size(px(11.))
        .text_color(color)
        .whitespace_nowrap()
        .border_1()
        .border_color(color.opacity(0.4))
        .child(text.into())
}

/// The label of the commit HEAD is on (`HEAD -> main`): filled and bold, so it
/// stands out from the outlined labels of other branches and tags.
pub fn head_badge(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .text_size(px(11.))
        .font_weight(FontWeight::BOLD)
        .text_color(theme.background)
        .bg(theme.accent)
        .whitespace_nowrap()
        .child(text.into())
}

/// A small colored square with one letter for the state of a changed file.
pub fn status_badge(letter: char, theme: &Theme) -> Div {
    let color = letter_color(letter, theme);
    div()
        .flex_none()
        .w(px(18.))
        .h(px(18.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_sm()
        .text_size(px(11.))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .bg(color.opacity(0.16))
        .child(letter.to_string())
}

fn letter_color(letter: char, theme: &Theme) -> Hsla {
    match letter {
        'A' => theme.added,
        'D' | '!' => theme.removed,
        _ => edit_letter_color(letter, theme),
    }
}

fn edit_letter_color(letter: char, theme: &Theme) -> Hsla {
    match letter {
        'M' | 'T' => theme.attention,
        'R' | 'C' => theme.accent,
        _ => theme.muted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_letter_of_a_changed_file_has_its_color() {
        let theme = Theme::dark();
        let colors = [
            ('A', theme.added),
            ('D', theme.removed),
            ('!', theme.removed),
            ('M', theme.attention),
            ('T', theme.attention),
            ('R', theme.accent),
            ('C', theme.accent),
            ('?', theme.muted),
        ];
        for (letter, color) in colors {
            assert_eq!(letter_color(letter, &theme), color, "{letter}");
        }
    }
}
