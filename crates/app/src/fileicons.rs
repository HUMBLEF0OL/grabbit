//! The icon before a file's name, in every list that shows files (the download list, Download
//! File Info, Properties, a torrent's files, the Scheduler's): the theme's icon set's icon for the
//! file's kind (`crate::themeicons`), Graphite's where the set has none (plan 05.5c).

use egui::emath::GuiRounding as _;
use egui::{Painter, Rect, Ui};

/// What the icon is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind<'a> {
    /// A file, by its name (the extension decides; case doesn't matter).
    File(&'a str),
    /// A folder (a torrent's folder, a tree's branch).
    Folder,
}

/// A folder, or a file by its extension (lowercase; None: a file without one): what the icon
/// stands for, and what Windows' shell is asked for when a file is dragged out (win32.rs).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum What {
    Folder,
    File(Option<String>),
}

impl What {
    pub fn of(kind: Kind) -> Self {
        match kind {
            Kind::Folder => What::Folder,
            Kind::File(name) => What::File(extension(name)),
        }
    }

    /// The icon set's need for it.
    fn need(&self) -> &'static str {
        match self {
            What::Folder => "cat-folder",
            What::File(ext) => crate::themeicons::file_need(ext.as_deref()),
        }
    }
}

/// An icon as Windows' shell gives it (win32.rs, the picture of a file dragged out).
pub enum Native {
    /// `side` × `side` RGBA pixels, alpha not premultiplied.
    Rgba { side: u32, rgba: Vec<u8> },
}

/// The extension of `name` (after its last dot, lowercase), if it has one.
fn extension(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    match base.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() && ext.len() <= 16 => {
            Some(ext.to_ascii_lowercase())
        }
        _ => None,
    }
}

/// Draw `kind`'s icon in `rc` (on screen) through `painter`, so it keeps to the list's clip. On
/// whole pixels, so it stays sharp at every scale.
pub fn paint(ui: &Ui, painter: &Painter, kind: Kind, rc: Rect) {
    let rc = rc.round_to_pixels(ui.ctx().pixels_per_point());
    let need = What::of(kind).need();
    if !crate::themeicons::paint(ui, painter, need, rc, true) {
        crate::themeicons::paint_graphite(ui, painter, need, rc, true);
    }
}

/// [`paint`] at English point `min` (a list's layout), `side` points square: placed where that
/// is on screen, so in a right-to-left window the icon is at the row's start there too.
pub fn paint_at(ui: &Ui, painter: &Painter, kind: Kind, min: egui::Pos2, side: f32) {
    let rc = crate::winui::rect(Rect::from_min_size(min, egui::vec2(side, side)));
    paint(ui, painter, kind, rc);
}

/// How far a name moves to make room for its 16-point icon (the icon and a gap).
pub const INDENT: f32 = 20.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_managers_files_get_their_kind() {
        let of = |name: &str| What::of(Kind::File(name)).need();
        for (name, need) in [
            ("movie.mp4", "ft-video"),
            ("stream.TS", "ft-video"),
            ("song.flac", "ft-audio"),
            ("setup.exe", "ft-executable"),
            ("setup.msi", "ft-executable"),
            ("app.dmg", "ft-disk-image"),
            ("ubuntu.iso", "ft-disk-image"),
            ("tool.deb", "ft-archive"),
            ("photos.7z", "ft-archive"),
            ("backup.tar.gz", "ft-archive"),
            ("report.pdf", "ft-pdf"),
            ("C:\\dl\\Letter.DOCX", "ft-document"),
            ("sheet.xlsx", "ft-spreadsheet"),
            ("talk.pptx", "ft-presentation"),
            ("page.html", "ft-code"),
            ("film.torrent", "ft-torrent"),
        ] {
            assert_eq!(of(name), need, "{name}");
        }
        // No extension, a dot file, an unknown type: the plain page; folders: the set's.
        assert_eq!(What::of(Kind::File("README")), What::File(None));
        assert_eq!(What::of(Kind::File(".hidden")), What::File(None));
        assert_eq!(of("README"), "ft-text");
        assert_eq!(of("data.xyz"), "ft-text");
        assert_eq!(What::Folder.need(), "cat-folder");
        // Every one is in Graphite's set, the fallback.
        for what in [What::Folder, What::File(None)] {
            assert!(crate::themeicons::graphite_has(what.need(), 16));
        }
    }
}
