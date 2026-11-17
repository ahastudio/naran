//! Image files: both versions are read as pictures and shown side by side,
//! Before on the left and After on the right.

use std::path::Path;
use std::sync::Arc;

use gpui::{AnyElement, Div, FontWeight, Image, ImageFormat, ObjectFit, div, img, prelude::*, px};
use naran_core::{Error, Side};

use super::versions::Compared;
use crate::theme::Theme;
use crate::widgets::{muted, problem};

/// The two pictures of an image file. A new file has no `before`, a deleted
/// file has no `after`.
#[derive(Clone)]
pub(in crate::workspace) struct Pictures {
    pub(in crate::workspace) before: Option<Arc<Image>>,
    pub(in crate::workspace) after: Option<Arc<Image>>,
}

/// Where the two versions of a picture come from.
#[derive(Clone)]
pub(super) enum PictureSource {
    Working(Compared),
    Commit(String),
}

const IMAGE_FORMATS: [(&str, ImageFormat); 8] = [
    ("png", ImageFormat::Png),
    ("jpg", ImageFormat::Jpeg),
    ("jpeg", ImageFormat::Jpeg),
    ("gif", ImageFormat::Gif),
    ("webp", ImageFormat::Webp),
    ("bmp", ImageFormat::Bmp),
    ("tif", ImageFormat::Tiff),
    ("tiff", ImageFormat::Tiff),
];

pub(super) fn image_format(path: &str) -> Option<ImageFormat> {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str());
    let extension = extension.map(str::to_ascii_lowercase);
    let found = IMAGE_FORMATS
        .iter()
        .find(|(name, _)| Some(*name) == extension.as_deref());
    found.map(|(_, format)| *format)
}

/// The bytes of a picture before and after the change; a side is `None`
/// when the file is not there.
pub(super) type PictureBytes = (Option<Vec<u8>>, Option<Vec<u8>>);

/// Reads the bytes of both versions; called in the background.
pub(super) fn read_picture_bytes(
    repository: &Path,
    file: &str,
    source: &PictureSource,
) -> Result<PictureBytes, Error> {
    match source {
        PictureSource::Working(compared) => working_picture_bytes(repository, file, *compared),
        PictureSource::Commit(hash) => both(
            naran_core::blob(repository, &Side::ParentOf(hash.clone()), file),
            naran_core::blob(repository, &Side::Commit(hash.clone()), file),
        ),
    }
}

/// A staged picture goes from HEAD to the index, an unstaged one from the
/// index to the folder, and an untracked one comes only from the folder.
fn working_picture_bytes(
    repository: &Path,
    file: &str,
    compared: Compared,
) -> Result<PictureBytes, Error> {
    let blob = |side: Side| naran_core::blob(repository, &side, file);
    let folder = || naran_core::worktree_file(repository, file);
    match compared {
        Compared::Staged => both(blob(Side::Head), blob(Side::Index)),
        Compared::Untracked => both(Ok(None), folder()),
        Compared::Unstaged => both(blob(Side::Index), folder()),
    }
}

/// The two sides together, or the first failure.
fn both(
    before: Result<Option<Vec<u8>>, Error>,
    after: Result<Option<Vec<u8>>, Error>,
) -> Result<PictureBytes, Error> {
    before.and_then(|before| after.map(|after| (before, after)))
}

pub(super) fn pictures_from(format: ImageFormat, bytes: PictureBytes) -> Pictures {
    let make = |bytes: Vec<u8>| Arc::new(Image::from_bytes(format, bytes));
    Pictures {
        before: bytes.0.map(make),
        after: bytes.1.map(make),
    }
}

/// An image file: Before on the left, After on the right. A side that does
/// not exist stays empty.
pub(super) fn render_pictures(
    file: &str,
    pictures: Option<&Result<Pictures, Error>>,
    theme: &Theme,
) -> AnyElement {
    match pictures {
        None => muted("Reading…", theme),
        Some(Err(error)) => problem(error.to_string(), theme),
        Some(Ok(pictures)) => picture_pair(file, pictures, theme),
    }
}

fn picture_pair(file: &str, pictures: &Pictures, theme: &Theme) -> AnyElement {
    let title = div()
        .px_2()
        .py_1()
        .bg(theme.surface)
        .font_weight(FontWeight::SEMIBOLD)
        .child(file.to_string());
    let sides = div()
        .flex_1()
        .min_h(px(0.))
        .flex()
        .flex_row()
        .child(picture_side("Before", pictures.before.as_ref(), theme))
        .child(div().w(px(1.)).h_full().bg(theme.border))
        .child(picture_side("After", pictures.after.as_ref(), theme));
    let page = div().size_full().flex().flex_col();
    page.child(title).child(sides).into_any_element()
}

fn picture_side(label: &'static str, picture: Option<&Arc<Image>>, theme: &Theme) -> Div {
    let name = div()
        .px_2()
        .py_1()
        .text_size(px(11.))
        .text_color(theme.muted)
        .child(label);
    div()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .flex()
        .flex_col()
        .child(name)
        .child(picture_frame(picture, theme))
}

/// The picture in the middle of its side, never bigger than the side.
fn picture_frame(picture: Option<&Arc<Image>>, theme: &Theme) -> Div {
    div()
        .flex_1()
        .min_h(px(0.))
        .flex()
        .items_center()
        .justify_center()
        .p_2()
        .child(picture_body(picture, theme))
}

fn picture_body(picture: Option<&Arc<Image>>, theme: &Theme) -> AnyElement {
    match picture {
        Some(picture) => img(gpui::ImageSource::Image(picture.clone()))
            .size_full()
            .object_fit(ObjectFit::ScaleDown)
            .into_any_element(),
        None => div()
            .text_color(theme.muted)
            .child("No file")
            .into_any_element(),
    }
}
