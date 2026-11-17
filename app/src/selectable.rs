//! Read-only text that can be selected with the mouse and copied.
//!
//! GPUI draws text but has no selection for it, so each line of a view (a
//! commit message, a line of a diff) is a `SelectableText` that registers
//! where it was painted in a shared `Region`. The container of the lines
//! (`area`) turns mouse events into a selection over the lines of the region,
//! across lines, and the lines paint the part that is selected. Only text
//! that is painted can be hit, but copying reads the text of every line of
//! the region, so it works for lines that were scrolled away mid-drag.

mod element;
mod region;
mod texts;

pub use element::SelectableText;
pub use region::Region;
pub use texts::{Texts, Which, area};
