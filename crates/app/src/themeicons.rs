//! A theme's icon set (#45): files named by what MDM needs them for (`tb-add-url`, `cat-music`,
//! `ft-pdf`…, the list in [`NEEDS`]), at 16 pixels and at 24 (drawn at 24 and up), pictures at
//! 32 and 48 too (a screen at 150 % to 300 %, [`SIZES`]). A theme that
//! has a set draws its icons where the classic look draws its own (the toolbar's outlines, the
//! categories' Silk icons, the file types' system icons); a need the set lacks keeps the classic
//! icon.
//!
//! Each file is an SVG drawing or a PNG picture, and says by its name how it is inked:
//! `<need>.png` and `<need>.svg` keep their colours, `<need>-symbolic.svg` and
//! `<need>-symbolic.png` are one colour (as the freedesktop icon themes name theirs): their
//! greys are drawn in the theme's text colour (greyed when disabled), their colours kept, as
//! GNOME and KDE recolour theirs. A set of SVGs all one colour says so once (`icon-mono = true`).
//!
//! Built-in sets are carried in the program (`assets/themes/<set>/icons/{16,24,32,48}/`). A theme file
//! in a folder of its own (`themes/<id>/theme.toml`) can bring one beside it
//! (`themes/<id>/icons/16/…`), read when the theme is chosen.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};

use egui::{Color32, ImageSource};

/// Every icon MDM can take from a theme, with what it stands for (the Themes help page lists
/// them).
pub const NEEDS: &[(&str, &str)] = &[
    ("tb-add-url", "Toolbar: Add URL"),
    ("tb-add-torrent", "Toolbar: Add torrent"),
    ("tb-resume", "Toolbar: Resume"),
    ("tb-stop", "Toolbar: Stop"),
    ("tb-stop-all", "Toolbar: Stop all"),
    ("tb-delete", "Toolbar: Delete"),
    ("tb-delete-completed", "Toolbar: Delete completed"),
    ("tb-options", "Toolbar: Options"),
    ("tb-scheduler", "Toolbar: Scheduler"),
    ("tb-start-queue", "Toolbar: Start queue"),
    ("tb-stop-queue", "Toolbar: Stop queue"),
    ("cat-all", "Categories: All downloads"),
    ("cat-compressed", "Categories: Compressed"),
    ("cat-documents", "Categories: Documents (and General)"),
    ("cat-music", "Categories: Music"),
    ("cat-programs", "Categories: Programs"),
    ("cat-video", "Categories: Video"),
    ("cat-torrents", "Categories: Torrents"),
    ("cat-unfinished", "Categories: Unfinished"),
    ("cat-finished", "Categories: Finished"),
    ("cat-queues", "Categories: Queues"),
    (
        "cat-folder",
        "A folder (a category of your own, a torrent's folder)",
    ),
    ("cat-folder-open", "An open folder"),
    (
        "misc-emblem-ok",
        "The check badge (Finished, a synchronized queue)",
    ),
    ("misc-up", "Scheduler: Move up"),
    ("misc-down", "Scheduler: Move down"),
    ("misc-remove", "Scheduler: Remove"),
    ("ft-generic", "A file of no known type"),
    ("ft-archive", "Archives (zip, 7z, rar…)"),
    ("ft-audio", "Sound files"),
    ("ft-video", "Video files"),
    ("ft-image", "Pictures and drawings"),
    ("ft-document", "Documents (doc, odt, epub…)"),
    ("ft-pdf", "PDF documents"),
    ("ft-spreadsheet", "Spreadsheets"),
    ("ft-presentation", "Presentations"),
    ("ft-text", "Plain text"),
    ("ft-code", "Web pages and scripts"),
    ("ft-executable", "Programs and installers"),
    ("ft-disk-image", "Disk images (iso, dmg…)"),
    ("ft-font", "Fonts"),
    ("ft-torrent", "Torrent files"),
    ("st-complete", "Status: complete"),
    ("st-downloading", "Status: downloading"),
    ("st-paused", "Status: paused"),
    ("st-queued", "Status: queued"),
    ("st-error", "Status: failed"),
    ("st-seeding", "Status: seeding"),
    ("ui-minimize", "Title bar: Minimize"),
    ("ui-maximize", "Title bar: Maximize"),
    ("ui-restore", "Title bar: Restore"),
    ("ui-close", "Title bar: Close"),
    ("ui-to-tray", "Title bar: Hide to the notification area"),
];

/// Icons only the Graphite layout draws (`[controls] chrome = "graphite"`): its menu rows,
/// buttons, Options pages, banners and tiles. No classic need stands for them, so a set that
/// lacks one draws nothing there; only Graphite's set carries them.
pub const EXTRAS: &[(&str, &str)] = &[
    (
        "gx-link",
        "An address: the paste field, address boxes, Refresh download address",
    ),
    (
        "gx-download",
        "Download: the paste bar's button, updates, the drop target",
    ),
    ("gx-search", "Find, Find next, Look up"),
    ("gx-sidebar", "Toolbar: the details pane"),
    ("gx-refresh", "Synchronization queue, Redownload"),
    ("gx-gauge", "Download limits, Speed limiter"),
    ("gx-filter", "A filtered column, Clear all filters"),
    ("gx-window", "Show progress window, Open MDM"),
    ("gx-external", "Open"),
    ("gx-apps", "Open with…"),
    ("gx-edit", "Move/Rename, Edit queue"),
    ("gx-shield", "Verify checksum, Options: Proxy / Socks"),
    ("gx-info", "Properties, Help contents"),
    ("gx-plus", "Add: a category, a queue"),
    ("gx-minus", "Delete from queue"),
    ("gx-dots", "Browse (…)"),
    ("gx-grip", "Download complete: the drag handle"),
    ("gx-pointer", "On double-click"),
    ("gx-hash", "Add batch download"),
    ("gx-copy", "Add batch download from clipboard"),
    ("gx-rss", "RSS feeds"),
    ("gx-upload", "Export"),
    ("gx-merge", "Video helpers (joining streams)"),
    ("gx-power", "When the queue is done: the countdown"),
    ("gx-browser", "A browser extension"),
    ("gx-bug", "Report a problem"),
    ("gx-keyboard", "Keyboard shortcuts"),
    ("gx-world", "Home page, Options: Regional"),
    ("gx-send", "Tell a friend"),
    ("gx-gift", "Support MDM"),
    ("gx-adjust", "Options: General"),
    ("gx-network", "Options: Connection"),
    ("gx-device", "Options: Remote"),
    ("gx-key", "Options: Sites logins"),
    ("gx-user", "Options: Accounts"),
    ("gx-volume", "Options: Sounds"),
];

/// Whether a set's file named for `need` is read: a need, or an extra.
fn known(need: &str) -> bool {
    NEEDS.iter().chain(EXTRAS).any(|(n, _)| *n == need)
}

/// One icon file of a set: its bytes, whether it is a picture (PNG) rather than a drawing (SVG),
/// and whether it is drawn in one colour (white, tinted with the theme's text colour).
#[derive(Clone)]
struct File {
    bytes: Arc<[u8]>,
    png: bool,
    mono: bool,
}

/// The sizes a set's icons are drawn at, in pixels: 16 and 24 (drawn at 24 and up), and for
/// pictures, which don't scale cleanly, 32 and 48 (a screen at 150 % to 300 %).
pub const SIZES: [u32; 4] = [16, 24, 32, 48];

/// One set: each need's (and extra's, [`EXTRAS`]) icon at the sizes it has ([`SIZES`]).
#[derive(Default)]
pub struct Set {
    files: HashMap<(String, u32), File>,
    /// Its Finished and Queues icons say so already: the categories tree draws them without
    /// the classic look's badges ([`tree_badges`]).
    no_badges: bool,
}

impl Set {
    #[cfg(test)]
    fn has(&self, need: &str) -> bool {
        SIZES
            .iter()
            .any(|s| self.files.contains_key(&(need.to_owned(), *s)))
    }

    /// The file for `px` (the screen's pixels): the smallest drawn for at least that size, else
    /// the largest. A picture is drawn at most a pixel larger than its size (16 up to 17 pixels,
    /// 32 up to 34): larger, the next size is sharper. A drawing scales cleanly, a quarter up (16
    /// up to 20 pixels; 24 above, where it is the largest).
    fn file(&self, need: &str, px: f32) -> Option<(u32, File)> {
        let have = SIZES
            .iter()
            .filter_map(|&s| self.files.get(&(need.to_owned(), s)).map(|f| (s, f)));
        let mut largest = None;
        for (s, f) in have {
            let stretch = if f.png { 1.0625 } else { 1.25 };
            if px <= s as f32 * stretch {
                return Some((s, f.clone()));
            }
            largest = Some((s, f.clone()));
        }
        largest
    }

    /// A set from its files: each `(size, file name, bytes)`, the file named after its need
    /// (`<need>.svg`, `<need>.png`, `<need>-symbolic.svg` or `<need>-symbolic.png`). An SVG is
    /// one colour when the set says so (`mono`) or its name ends in `-symbolic` (the freedesktop
    /// icon themes' one-colour icons, GNOME's and KDE's): its greys take the theme's ink when
    /// drawn. A PNG keeps its colours, unless its name ends in `-symbolic`: its greys are made
    /// white, to be tinted with the ink.
    /// Files of no known need (nor extra, [`EXTRAS`]) are left out.
    fn from_files<'a>(
        files: impl IntoIterator<Item = (u32, &'a str, Arc<[u8]>)>,
        mono: bool,
    ) -> Self {
        let mut set = Set::default();
        for (px, name, bytes) in files {
            let Some((stem, ext)) = name.rsplit_once('.') else {
                continue;
            };
            let (need, symbolic) = match stem.strip_suffix("-symbolic") {
                Some(n) => (n, true),
                None => (stem, false),
            };
            if !known(need) {
                continue;
            }
            let file = match ext.to_ascii_lowercase().as_str() {
                "png" if !symbolic => File {
                    bytes,
                    png: true,
                    mono: false,
                },
                // A one-colour picture: its greys made white once, to be tinted.
                "png" => File {
                    bytes: whitened(&bytes),
                    png: true,
                    mono: true,
                },
                "svg" if symbolic => File {
                    bytes,
                    png: false,
                    mono: true,
                },
                "svg" => File {
                    bytes,
                    png: false,
                    mono,
                },
                _ => continue,
            };
            // One file per need and size: a one-colour icon before a picture, as a desktop's
            // icon theme prefers its symbolic icons.
            let key = (need.to_owned(), px);
            if set.files.get(&key).is_some_and(|f| f.mono && !file.mono) {
                continue;
            }
            set.files.insert(key, file);
        }
        set
    }
}

/// A one-colour SVG in `ink` (`#rrggbb`), as GNOME and KDE recolour theirs: each grey it names
/// (`fill`, `stroke`, `color`, in attributes or styles: the icon's ink, whatever shade its
/// author drew it in) becomes the ink, and so does the drawing's default. Its colours (a
/// warning's orange, KDE's highlight blue) stay.
fn inked(svg: &[u8], ink: &str) -> Arc<[u8]> {
    let Ok(text) = std::str::from_utf8(svg) else {
        return Arc::from(svg);
    };
    const KEYS: [&str; 7] = [
        "fill=\"",
        "stroke=\"",
        "fill:",
        "stroke:",
        "stop-color=\"",
        "color=\"",
        "color:",
    ];
    let mut out = String::with_capacity(text.len() + 16);
    let mut rest = text;
    'scan: while let Some(c) = rest.chars().next() {
        for key in KEYS {
            if let Some(after) = rest.strip_prefix(key)
                && let Some((len, grey)) = colour(after)
            {
                out.push_str(key);
                out.push_str(if grey { ink } else { &after[..len] });
                rest = &after[len..];
                continue 'scan;
            }
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    // A shape that names no fill is drawn black: the ink is the drawing's default instead.
    if let Some(at) = out.find("<svg")
        && let Some(end) = out[at..].find('>')
        && !out[at..at + end].contains(" fill=")
    {
        out.insert_str(at + 4, &format!(" fill=\"{ink}\""));
    }
    Arc::from(out.into_bytes())
}

/// A one-colour PNG (`<need>-symbolic.png`) made white where it is grey, its transparency
/// kept, so the theme's ink tints it as GNOME and KDE recolour theirs; its colours stay. The
/// bytes as they are when they aren't a PNG.
fn whitened(png: &[u8]) -> Arc<[u8]> {
    let Ok(img) = image::load_from_memory_with_format(png, image::ImageFormat::Png) else {
        return Arc::from(png);
    };
    let mut img = img.to_rgba8();
    for px in img.pixels_mut() {
        let [r, g, b, _] = px.0;
        if r.max(g).max(b) - r.min(g).min(b) <= 24 {
            px.0[..3].copy_from_slice(&[255, 255, 255]);
        }
    }
    let mut out = Vec::new();
    match img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png) {
        Ok(()) => Arc::from(out),
        Err(_) => Arc::from(png),
    }
}

/// The colour `s` starts with (`#rgb`, `#rrggbb`, `#rrggbbaa` or a few named inks): its length,
/// and whether it is a grey (its channels within 24 of one another). None for anything else
/// (`none`, `url(#…)`, `currentColor`).
fn colour(s: &str) -> Option<(usize, bool)> {
    if let Some(hex) = s.strip_prefix('#') {
        let n = hex.bytes().take_while(u8::is_ascii_hexdigit).count();
        let ch = |i: usize, w: usize| u8::from_str_radix(&hex[i * w..i * w + w], 16).ok();
        let rgb = match n {
            3 => [ch(0, 1)? * 17, ch(1, 1)? * 17, ch(2, 1)? * 17],
            6 | 8 => [ch(0, 2)?, ch(1, 2)?, ch(2, 2)?],
            _ => return None,
        };
        let spread = rgb.iter().max()? - rgb.iter().min()?;
        return Some((n + 1, spread <= 24));
    }
    ["black", "white", "gray", "grey"]
        .iter()
        .find(|w| s.starts_with(**w))
        .map(|w| (w.len(), true))
}

/// A built-in set's files, by size: their names as in its folder
/// (`assets/themes/<set>/icons/<size>/<name>`).
macro_rules! builtin_files {
    ($dir:literal; $($px:literal: [$($f:literal),* $(,)?]);* $(;)?) => {{
        let files: Vec<(u32, &'static str, Arc<[u8]>)> = vec![$($((
            $px,
            $f,
            Arc::from(&include_bytes!(concat!(
                "../assets/themes/", $dir, "/icons/", stringify!($px), "/", $f
            ))[..]),
        ),)*)*];
        files
    }};
}

/// The sets loaded so far, by name: the built-in ones, and the theme folders' as their themes
/// are chosen.
static SETS: LazyLock<Mutex<HashMap<String, Arc<Set>>>> = LazyLock::new(|| {
    let mut sets = HashMap::new();
    // Microsoft's Fluent UI System Icons (MIT, assets/themes/metro/icons/LICENSE), recoloured
    // white to be tinted; the magnet and the torrent file are MDM's own at their weight, Hide
    // to the notification area MDM's own one-colour picture.
    let metro = builtin_files!("metro";
        16: ["cat-all.svg", "cat-compressed.svg", "cat-documents.svg", "cat-finished.svg",
            "cat-folder-open.svg", "cat-folder.svg", "cat-music.svg", "cat-programs.svg",
            "cat-queues.svg", "cat-torrents.svg", "cat-unfinished.svg", "cat-video.svg",
            "ft-archive.svg", "ft-audio.svg", "ft-code.svg", "ft-disk-image.svg", "ft-document.svg",
            "ft-executable.svg", "ft-font.svg", "ft-generic.svg", "ft-image.svg", "ft-pdf.svg",
            "ft-presentation.svg", "ft-spreadsheet.svg", "ft-text.svg", "ft-torrent.svg",
            "ft-video.svg", "misc-down.svg", "misc-emblem-ok.svg", "misc-remove.svg", "misc-up.svg",
            "st-complete.svg", "st-downloading.svg", "st-error.svg", "st-paused.svg",
            "st-queued.svg", "st-seeding.svg", "tb-add-torrent.svg", "tb-add-url.svg",
            "tb-delete-completed.svg", "tb-delete.svg", "tb-options.svg", "tb-resume.svg",
            "tb-scheduler.svg", "tb-start-queue.svg", "tb-stop-all.svg", "tb-stop-queue.svg",
            "tb-stop.svg", "ui-close.svg", "ui-maximize.svg", "ui-minimize.svg", "ui-restore.svg",
            "ui-to-tray-symbolic.png"];
        24: ["cat-all.svg", "cat-compressed.svg", "cat-documents.svg", "cat-finished.svg",
            "cat-folder-open.svg", "cat-folder.svg", "cat-music.svg", "cat-programs.svg",
            "cat-queues.svg", "cat-torrents.svg", "cat-unfinished.svg", "cat-video.svg",
            "ft-archive.svg", "ft-audio.svg", "ft-code.svg", "ft-document.svg", "ft-executable.svg",
            "ft-font.svg", "ft-generic.svg", "ft-image.svg", "ft-pdf.svg", "ft-presentation.svg",
            "ft-spreadsheet.svg", "ft-text.svg", "ft-torrent.svg", "ft-video.svg", "misc-down.svg",
            "misc-emblem-ok.svg", "misc-remove.svg", "misc-up.svg", "st-complete.svg",
            "st-downloading.svg", "st-error.svg", "st-paused.svg", "st-queued.svg",
            "st-seeding.svg", "tb-add-torrent.svg", "tb-add-url.svg", "tb-delete-completed.svg",
            "tb-delete.svg", "tb-options.svg", "tb-resume.svg", "tb-scheduler.svg",
            "tb-stop-all.svg", "tb-stop-queue.svg", "tb-stop.svg", "ui-close.svg",
            "ui-maximize.svg", "ui-minimize.svg", "ui-restore.svg", "ui-to-tray-symbolic.png"];
        32: ["ui-to-tray-symbolic.png"];
        48: ["ui-to-tray-symbolic.png"]);
    sets.insert("metro".to_owned(), Arc::new(Set::from_files(metro, true)));
    // GNOME's adwaita-icon-theme 51.0 (LGPL-3, assets/themes/adwaita/icons/LICENSE), its files
    // as GNOME ships them: colour pictures at 16 pixels and drawings above, one-colour symbolic
    // icons tinted. The PDF, torrent-file and Hide to the notification area icons are MDM's
    // own, in its style.
    let adwaita = builtin_files!("adwaita";
        16: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open-symbolic.svg", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues-symbolic.svg", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image-symbolic.svg",
            "ft-document.png", "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png",
            "ft-pdf.png", "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png",
            "ft-torrent.png", "ft-video.png", "misc-down-symbolic.svg",
            "misc-emblem-ok-symbolic.svg", "misc-remove-symbolic.svg", "misc-up-symbolic.svg",
            "st-complete-symbolic.svg", "st-downloading-symbolic.svg", "st-error-symbolic.svg",
            "st-paused-symbolic.svg", "st-queued-symbolic.svg", "st-seeding-symbolic.svg",
            "tb-add-torrent-symbolic.svg", "tb-add-url-symbolic.svg",
            "tb-delete-completed-symbolic.svg", "tb-delete-symbolic.svg", "tb-options-symbolic.svg",
            "tb-resume-symbolic.svg", "tb-scheduler-symbolic.svg", "tb-start-queue-symbolic.svg",
            "tb-stop-all-symbolic.svg", "tb-stop-queue-symbolic.svg", "tb-stop-symbolic.svg",
            "ui-close-symbolic.svg", "ui-maximize-symbolic.svg", "ui-minimize-symbolic.svg",
            "ui-restore-symbolic.svg", "ui-to-tray-symbolic.png"];
        24: ["cat-all.svg", "cat-compressed.svg", "cat-documents.svg", "cat-finished.svg",
            "cat-folder-open-symbolic.svg", "cat-folder.svg", "cat-music.svg", "cat-programs.svg",
            "cat-queues-symbolic.svg", "cat-torrents.svg", "cat-unfinished.svg", "cat-video.svg",
            "ft-archive.svg", "ft-audio.svg", "ft-code.svg", "ft-disk-image.svg", "ft-document.svg",
            "ft-executable.svg", "ft-font.svg", "ft-generic.svg", "ft-image.svg", "ft-pdf.png",
            "ft-presentation.svg", "ft-spreadsheet.svg", "ft-text.svg", "ft-torrent.png",
            "ft-video.svg", "misc-down-symbolic.svg", "misc-emblem-ok-symbolic.svg",
            "misc-remove-symbolic.svg", "misc-up-symbolic.svg", "st-complete-symbolic.svg",
            "st-downloading-symbolic.svg", "st-error-symbolic.svg", "st-paused-symbolic.svg",
            "st-queued-symbolic.svg", "st-seeding-symbolic.svg", "tb-add-torrent-symbolic.svg",
            "tb-add-url-symbolic.svg", "tb-delete-completed-symbolic.svg", "tb-delete-symbolic.svg",
            "tb-options-symbolic.svg", "tb-resume-symbolic.svg", "tb-scheduler-symbolic.svg",
            "tb-start-queue-symbolic.svg", "tb-stop-all-symbolic.svg", "tb-stop-queue-symbolic.svg",
            "tb-stop-symbolic.svg", "ui-close-symbolic.svg", "ui-maximize-symbolic.svg",
            "ui-minimize-symbolic.svg", "ui-restore-symbolic.svg", "ui-to-tray-symbolic.png"];
        32: ["ft-pdf.png", "ft-torrent.png", "ui-to-tray-symbolic.png"];
        48: ["ft-pdf.png", "ft-torrent.png", "ui-to-tray-symbolic.png"]);
    sets.insert(
        "adwaita".to_owned(),
        Arc::new(Set::from_files(adwaita, false)),
    );
    // KDE's breeze-icons 6.31 (LGPL-3.0-or-later, assets/themes/breeze/icons/LICENSE), its files
    // as KDE ships them: colour drawings for files and categories, one-colour icons (its
    // ColorScheme-Text ones) in the theme's ink, their highlight and warning colours kept. The
    // PDF and Hide to the notification area icons are MDM's own, in its style.
    let breeze = builtin_files!("breeze";
        16: ["cat-all-symbolic.svg", "cat-compressed.svg", "cat-documents.svg",
            "cat-finished-symbolic.svg", "cat-folder-open-symbolic.svg", "cat-folder-symbolic.svg",
            "cat-music.svg", "cat-programs-symbolic.svg", "cat-queues-symbolic.svg",
            "cat-torrents.svg", "cat-unfinished-symbolic.svg", "cat-video.svg", "ft-archive.svg",
            "ft-audio.svg", "ft-code.svg", "ft-disk-image-symbolic.svg", "ft-document.svg",
            "ft-executable-symbolic.svg", "ft-font.svg", "ft-generic.svg", "ft-image.svg",
            "ft-pdf.png", "ft-presentation.svg", "ft-spreadsheet.svg", "ft-text.svg",
            "ft-torrent.svg", "ft-video.svg", "misc-down-symbolic.svg",
            "misc-emblem-ok-symbolic.svg", "misc-remove.svg", "misc-up-symbolic.svg",
            "st-complete-symbolic.svg", "st-downloading-symbolic.svg", "st-error-symbolic.svg",
            "st-paused-symbolic.svg", "st-queued-symbolic.svg", "st-seeding-symbolic.svg",
            "tb-add-url-symbolic.svg", "tb-delete-completed-symbolic.svg", "tb-delete.svg",
            "tb-options-symbolic.svg", "tb-resume-symbolic.svg", "tb-scheduler-symbolic.svg",
            "tb-stop-symbolic.svg", "ui-close-symbolic.svg", "ui-maximize-symbolic.svg",
            "ui-minimize-symbolic.svg", "ui-restore-symbolic.svg", "ui-to-tray-symbolic.png"];
        24: ["cat-all-symbolic.svg", "cat-compressed.svg", "cat-documents.svg",
            "cat-finished-symbolic.svg", "cat-folder-open-symbolic.svg", "cat-folder-symbolic.svg",
            "cat-music.svg", "cat-programs.svg", "cat-queues-symbolic.svg", "cat-torrents.svg",
            "cat-unfinished-symbolic.svg", "cat-video.svg", "ft-archive.svg", "ft-audio.svg",
            "ft-code.svg", "ft-disk-image.svg", "ft-document.svg", "ft-executable.svg",
            "ft-font.svg", "ft-generic.svg", "ft-image.svg", "ft-pdf.png", "ft-presentation.svg",
            "ft-spreadsheet.svg", "ft-text.svg", "ft-torrent.svg", "ft-video.svg",
            "misc-down-symbolic.svg", "misc-emblem-ok-symbolic.svg", "misc-remove.svg",
            "misc-up-symbolic.svg", "st-complete-symbolic.svg", "st-downloading-symbolic.svg",
            "st-error-symbolic.svg", "st-paused-symbolic.svg", "st-queued-symbolic.svg",
            "st-seeding-symbolic.svg", "tb-add-torrent-symbolic.svg", "tb-add-url-symbolic.svg",
            "tb-delete-completed-symbolic.svg", "tb-delete.svg", "tb-options-symbolic.svg",
            "tb-resume-symbolic.svg", "tb-scheduler-symbolic.svg", "tb-start-queue-symbolic.svg",
            "tb-stop-all-symbolic.svg", "tb-stop-queue-symbolic.svg", "tb-stop-symbolic.svg",
            "ui-close-symbolic.svg", "ui-maximize-symbolic.svg", "ui-minimize-symbolic.svg",
            "ui-restore-symbolic.svg", "ui-to-tray-symbolic.png"];
        32: ["ft-pdf.png", "ui-to-tray-symbolic.png"];
        48: ["ft-pdf.png", "ui-to-tray-symbolic.png"]);
    sets.insert(
        "breeze".to_owned(),
        Arc::new(Set::from_files(breeze, false)),
    );
    // The era themes' sets (#45): pictures drawn for MDM in the manner of Windows XP, Windows 7
    // and Mac OS X, in colour, at 16, 24, 32 and 48 pixels. The title bar's buttons are MDM's own
    // vector glyphs (drawn by `winui::caption`), but for Hide to the notification area.
    let luna = builtin_files!("luna";
        16: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        24: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        32: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        48: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"]);
    sets.insert("luna".to_owned(), Arc::new(Set::from_files(luna, false)));
    let aero = builtin_files!("aero";
        16: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        24: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        32: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        48: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"]);
    sets.insert("aero".to_owned(), Arc::new(Set::from_files(aero, false)));
    let aqua = builtin_files!("aqua";
        16: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        24: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        32: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"];
        48: ["cat-all.png", "cat-compressed.png", "cat-documents.png", "cat-finished.png",
            "cat-folder-open.png", "cat-folder.png", "cat-music.png", "cat-programs.png",
            "cat-queues.png", "cat-torrents.png", "cat-unfinished.png", "cat-video.png",
            "ft-archive.png", "ft-audio.png", "ft-code.png", "ft-disk-image.png", "ft-document.png",
            "ft-executable.png", "ft-font.png", "ft-generic.png", "ft-image.png", "ft-pdf.png",
            "ft-presentation.png", "ft-spreadsheet.png", "ft-text.png", "ft-torrent.png",
            "ft-video.png", "misc-down.png", "misc-emblem-ok.png", "misc-remove.png", "misc-up.png",
            "st-complete.png", "st-downloading.png", "st-error.png", "st-paused.png",
            "st-queued.png", "st-seeding.png", "tb-add-torrent.png", "tb-add-url.png",
            "tb-delete-completed.png", "tb-delete.png", "tb-options.png", "tb-resume.png",
            "tb-scheduler.png", "tb-start-queue.png", "tb-stop-all.png", "tb-stop-queue.png",
            "tb-stop.png", "ui-to-tray.png"]);
    sets.insert("aqua".to_owned(), Arc::new(Set::from_files(aqua, false)));
    // Tabler Icons 3.31.0's outline icons (Paweł Kuna, MIT, assets/themes/graphite/icons/LICENSE),
    // drawn white at stroke 1.75 to be tinted, every need and extra (EXTRAS) at 16 and 24
    // pixels; SOURCES.txt says which icon each is. Written by
    // design/graphite/source/export_icons.py.
    let graphite = builtin_files!("graphite";
        16: ["cat-all.svg", "cat-compressed.svg", "cat-documents.svg", "cat-finished.svg",
            "cat-folder-open.svg", "cat-folder.svg", "cat-music.svg", "cat-programs.svg",
            "cat-queues.svg", "cat-torrents.svg", "cat-unfinished.svg", "cat-video.svg",
            "ft-archive.svg", "ft-audio.svg", "ft-code.svg", "ft-disk-image.svg", "ft-document.svg",
            "ft-executable.svg", "ft-font.svg", "ft-generic.svg", "ft-image.svg", "ft-pdf.svg",
            "ft-presentation.svg", "ft-spreadsheet.svg", "ft-text.svg", "ft-torrent.svg",
            "ft-video.svg",
            "gx-adjust.svg", "gx-apps.svg", "gx-browser.svg", "gx-bug.svg", "gx-copy.svg",
            "gx-device.svg", "gx-dots.svg", "gx-download.svg", "gx-edit.svg", "gx-external.svg",
            "gx-filter.svg", "gx-gauge.svg", "gx-gift.svg", "gx-grip.svg", "gx-hash.svg",
            "gx-info.svg", "gx-key.svg", "gx-keyboard.svg", "gx-link.svg", "gx-merge.svg",
            "gx-minus.svg", "gx-network.svg", "gx-plus.svg", "gx-pointer.svg", "gx-power.svg",
            "gx-refresh.svg", "gx-rss.svg", "gx-search.svg", "gx-send.svg", "gx-shield.svg",
            "gx-sidebar.svg", "gx-upload.svg", "gx-user.svg", "gx-volume.svg", "gx-window.svg",
            "gx-world.svg",
            "misc-down.svg", "misc-emblem-ok.svg", "misc-remove.svg", "misc-up.svg",
            "st-complete.svg", "st-downloading.svg", "st-error.svg", "st-paused.svg",
            "st-queued.svg", "st-seeding.svg", "tb-add-torrent.svg", "tb-add-url.svg",
            "tb-delete-completed.svg", "tb-delete.svg", "tb-options.svg", "tb-resume.svg",
            "tb-scheduler.svg", "tb-start-queue.svg", "tb-stop-all.svg", "tb-stop-queue.svg",
            "tb-stop.svg", "ui-close.svg", "ui-maximize.svg", "ui-minimize.svg", "ui-restore.svg",
            "ui-to-tray.svg"];
        24: ["cat-all.svg", "cat-compressed.svg", "cat-documents.svg", "cat-finished.svg",
            "cat-folder-open.svg", "cat-folder.svg", "cat-music.svg", "cat-programs.svg",
            "cat-queues.svg", "cat-torrents.svg", "cat-unfinished.svg", "cat-video.svg",
            "ft-archive.svg", "ft-audio.svg", "ft-code.svg", "ft-disk-image.svg", "ft-document.svg",
            "ft-executable.svg", "ft-font.svg", "ft-generic.svg", "ft-image.svg", "ft-pdf.svg",
            "ft-presentation.svg", "ft-spreadsheet.svg", "ft-text.svg", "ft-torrent.svg",
            "ft-video.svg",
            "gx-adjust.svg", "gx-apps.svg", "gx-browser.svg", "gx-bug.svg", "gx-copy.svg",
            "gx-device.svg", "gx-dots.svg", "gx-download.svg", "gx-edit.svg", "gx-external.svg",
            "gx-filter.svg", "gx-gauge.svg", "gx-gift.svg", "gx-grip.svg", "gx-hash.svg",
            "gx-info.svg", "gx-key.svg", "gx-keyboard.svg", "gx-link.svg", "gx-merge.svg",
            "gx-minus.svg", "gx-network.svg", "gx-plus.svg", "gx-pointer.svg", "gx-power.svg",
            "gx-refresh.svg", "gx-rss.svg", "gx-search.svg", "gx-send.svg", "gx-shield.svg",
            "gx-sidebar.svg", "gx-upload.svg", "gx-user.svg", "gx-volume.svg", "gx-window.svg",
            "gx-world.svg",
            "misc-down.svg", "misc-emblem-ok.svg", "misc-remove.svg", "misc-up.svg",
            "st-complete.svg", "st-downloading.svg", "st-error.svg", "st-paused.svg",
            "st-queued.svg", "st-seeding.svg", "tb-add-torrent.svg", "tb-add-url.svg",
            "tb-delete-completed.svg", "tb-delete.svg", "tb-options.svg", "tb-resume.svg",
            "tb-scheduler.svg", "tb-start-queue.svg", "tb-stop-all.svg", "tb-stop-queue.svg",
            "tb-stop.svg", "ui-close.svg", "ui-maximize.svg", "ui-minimize.svg", "ui-restore.svg",
            "ui-to-tray.svg"]);
    // Its Finished is a circled tick and its Queues a numbered list, as the mockup draws the tree:
    // no badge over them (the tick would repeat Finished's).
    let mut graphite = Set::from_files(graphite, true);
    graphite.no_badges = true;
    sets.insert("graphite".to_owned(), Arc::new(graphite));
    Mutex::new(sets)
});

/// Reads the set in `dir` (`16/<need>.svg`, `.png`, `-symbolic.svg` or `-symbolic.png`, the
/// same in `24`, and in `32` and `48` for pictures) under `name`, for a theme file's own folder.
/// Only the known needs and extras are read; other files are left alone. False when the folder
/// holds none.
pub fn load_folder(name: &str, dir: &Path, mono: bool) -> bool {
    let mut files = Vec::new();
    for px in SIZES {
        for (need, _) in NEEDS.iter().chain(EXTRAS) {
            for f in [
                format!("{need}-symbolic.svg"),
                format!("{need}-symbolic.png"),
                format!("{need}.svg"),
                format!("{need}.png"),
            ] {
                let path = dir.join(px.to_string()).join(&f);
                // An icon file is small: anything over 256 KB isn't one.
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() <= 256 * 1024
                    && let Ok(bytes) = std::fs::read(&path)
                {
                    files.push((px, f, Arc::<[u8]>::from(bytes)));
                }
            }
        }
    }
    if files.is_empty() {
        return false;
    }
    let set = Set::from_files(
        files.iter().map(|(px, f, b)| (*px, f.as_str(), b.clone())),
        mono,
    );
    if let Ok(mut sets) = SETS.lock() {
        sets.insert(name.to_owned(), Arc::new(set));
    }
    true
}

fn set(name: &str) -> Option<Arc<Set>> {
    SETS.lock().ok()?.get(name).cloned()
}

/// Whether the categories tree draws the badges that repeat a row's icon (Finished's tick and
/// Queues' page stack over the folder, each queue as the stack): yes in the classic look and in
/// every set but one whose Finished and Queues icons say so already (Graphite's), which then
/// draws them alone and each queue as its Queues icon. A synchronized queue keeps its tick.
pub fn tree_badges() -> bool {
    badges_with(crate::theme::current().icon_set.as_deref())
}

/// [`tree_badges`] for a theme with this set (None: none, or one not loaded).
fn badges_with(icon_set: Option<&str>) -> bool {
    icon_set.and_then(set).is_none_or(|s| !s.no_badges)
}

/// How an icon's picture takes the ink: its own colours, the ink written into a one-colour
/// drawing, or a one-colour picture (made white) tinted with it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Paint {
    Colour,
    Inked,
    Tinted,
}

/// The ink a one-colour icon is drawn in: the text colour, greyed when disabled.
pub fn ink(enabled: bool) -> Color32 {
    let c = crate::theme::colors();
    if enabled { c.text } else { c.icon_disabled }
}

/// The active theme's icon for `need` at about `px` pixels on screen, a one-colour icon in
/// `ink`, and the tint to draw it with (the ink's opacity; a coloured icon its own colours,
/// faded when not `enabled`). None when the theme has no set, or its set lacks this need. Made
/// once per set, need, size and ink, then reused: drawing it each frame allocates nothing.
pub fn source(
    need: &str,
    px: f32,
    ink: Color32,
    enabled: bool,
) -> Option<(ImageSource<'static>, Color32)> {
    use std::cell::RefCell;
    type Key = (Cow<'static, str>, String, u32, [u8; 3]);
    thread_local! {
        static MADE: RefCell<HashMap<Key, Option<(ImageSource<'static>, Paint)>>> = RefCell::default();
    }
    let name = crate::theme::current().icon_set.clone()?;
    let [r, g, b, a] = ink.to_srgba_unmultiplied();
    let (src, paint) = MADE.with(|made| {
        made.borrow_mut()
            .entry((name.clone(), need.to_owned(), px.round() as u32, [r, g, b]))
            .or_insert_with(|| {
                let set = set(&name)?;
                let (size, file) = set.file(need, px)?;
                let ext = if file.png { "png" } else { "svg" };
                let (ink_name, bytes) = if file.mono && file.png {
                    // White, tinted with the ink when drawn.
                    ("white".to_owned(), file.bytes)
                } else if file.mono {
                    let hex = format!("#{r:02x}{g:02x}{b:02x}");
                    let bytes = inked(&file.bytes, &hex);
                    (hex[1..].to_owned(), bytes)
                } else {
                    ("color".to_owned(), file.bytes)
                };
                // Kept for the program's life, as its texture is.
                let uri: &'static str = Box::leak(
                    format!("bytes://mdm/icon-set/{name}/{size}/{ink_name}/{need}.{ext}")
                        .into_boxed_str(),
                );
                let paint = match (file.mono, file.png) {
                    (true, true) => Paint::Tinted,
                    (true, false) => Paint::Inked,
                    _ => Paint::Colour,
                };
                Some((
                    ImageSource::Bytes {
                        uri: uri.into(),
                        bytes: egui::load::Bytes::Shared(bytes),
                    },
                    paint,
                ))
            })
            .clone()
    })?;
    let tint = match (paint, enabled) {
        (Paint::Tinted, _) => ink,
        (Paint::Inked, _) => Color32::WHITE.gamma_multiply(f32::from(a) / 255.0),
        (Paint::Colour, true) => Color32::WHITE,
        (Paint::Colour, false) => Color32::WHITE.gamma_multiply(0.45),
    };
    Some((src, tint))
}

/// Paints the active theme's icon for `need` into `rc` (enabled or greyed); false when the theme
/// has none for it (the caller draws the classic icon).
pub fn paint(
    ui: &egui::Ui,
    painter: &egui::Painter,
    need: &str,
    rc: egui::Rect,
    enabled: bool,
) -> bool {
    paint_in(ui, painter, need, rc, ink(enabled), enabled)
}

/// [`paint`], a one-colour icon in `ink` rather than the text's colour (a colour icon keeps
/// its own colours).
pub fn paint_in(
    ui: &egui::Ui,
    painter: &egui::Painter,
    need: &str,
    rc: egui::Rect,
    ink: Color32,
    enabled: bool,
) -> bool {
    let px = rc.width() * ui.ctx().pixels_per_point();
    let Some((src, tint)) = source(need, px, ink, enabled) else {
        return false;
    };
    let img = egui::Image::new(src);
    if let Ok(egui::load::TexturePoll::Ready { texture }) = img.load_for_size(ui.ctx(), rc.size()) {
        painter.image(
            texture.id,
            rc,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }
    true
}

/// A file type's need, from the classic fallback icon's kind (fileicons.rs).
pub fn file_need(tango: &str, ext: Option<&str>) -> &'static str {
    match ext {
        Some("torrent") => return "ft-torrent",
        Some("pdf") => return "ft-pdf",
        _ => {}
    }
    match tango {
        "folder" => "cat-folder",
        "folder-open" => "cat-folder-open",
        "video-x-generic" => "ft-video",
        "audio-x-generic" => "ft-audio",
        "image-x-generic" | "x-office-drawing" | "x-office-drawing-template" => "ft-image",
        "package-x-generic" => "ft-archive",
        "application-x-executable" => "ft-executable",
        "media-optical" => "ft-disk-image",
        "x-office-document" | "x-office-document-template" => "ft-document",
        "x-office-spreadsheet" | "x-office-spreadsheet-template" => "ft-spreadsheet",
        "x-office-presentation" | "x-office-presentation-template" => "ft-presentation",
        "text-html" | "text-x-script" => "ft-code",
        "font-x-generic" => "ft-font",
        "text-x-generic" => "ft-text",
        _ => "ft-generic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_need_has_one_id_and_metro_has_them_all() {
        let mut ids: Vec<&str> = NEEDS.iter().map(|(n, _)| *n).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
        let metro = set("metro").unwrap();
        for (need, _) in NEEDS {
            assert!(metro.has(need), "metro lacks {need}");
            let (_, f) = metro.file(need, 16.0).unwrap();
            // One colour, to be tinted: Fluent's drawings recoloured white, MDM's own pictures
            // made white where they are grey.
            assert!(f.mono, "{need}");
            if f.png {
                assert_white(&f.bytes, need);
                continue;
            }
            let text = std::str::from_utf8(&f.bytes).unwrap();
            assert!(!text.contains("#212121"), "{need} keeps Fluent's ink");
        }
        // MDM's own Hide to the notification area, where Fluent has none of that meaning: a
        // picture at every size; its torrent icons are drawings at Fluent's weight.
        for px in [16.0, 24.0, 32.0, 48.0] {
            assert!(metro.file("ui-to-tray", px).unwrap().1.png, "{px}");
            assert!(!metro.file("tb-add-torrent", px).unwrap().1.png, "{px}");
        }
    }

    /// Graphite has every need and every extra at 16 and 24 pixels: Tabler's outline drawings,
    /// one colour. As carried, each is stroked white once (on its root) and fills nothing (no
    /// part names a fill but `none`, which the ink would fill), and names no other colour;
    /// inked, the stroke is the ink and no white is left. The 16-pixel one is drawn up to 20
    /// pixels on screen, the 24-pixel one above.
    #[test]
    fn graphite_has_every_need_in_one_colour() {
        let set = set("graphite").unwrap();
        for (need, _) in NEEDS.iter().chain(EXTRAS) {
            for px in [16u32, 24] {
                let f = set.files.get(&((*need).to_owned(), px));
                let f = f.unwrap_or_else(|| panic!("graphite lacks {need} at {px}"));
                assert!(!f.png && f.mono, "{need} {px}");
                let raw = std::str::from_utf8(&f.bytes).unwrap();
                let count = |s: &str| raw.matches(s).count();
                assert_eq!(
                    count("fill="),
                    count(r#"fill="none""#),
                    "{need} {px}: a fill"
                );
                assert_eq!(count("stroke="), 1, "{need} {px}: one stroke, the root's");
                assert_eq!(count(r##"stroke="#ffffff""##), 1, "{need} {px}");
                assert_eq!(count("#"), 1, "{need} {px}: another colour");
                for no in ["currentColor", "style=", "color", "url("] {
                    assert!(!raw.contains(no), "{need} {px}: {no}");
                }
                let out = inked(&f.bytes, "#123456");
                let text = std::str::from_utf8(&out).unwrap();
                assert!(text.contains(r##"stroke="#123456""##), "{need} {px}");
                assert!(!text.to_ascii_lowercase().contains("#fff"), "{need} {px}");
            }
            for (px, size) in [(16.0, 16), (20.0, 16), (21.0, 24), (32.0, 24), (48.0, 24)] {
                assert_eq!(set.file(need, px).unwrap().0, size, "{need} {px}");
            }
        }
        // The tree draws its Finished and Queues icons alone (the mockup's), whichever theme
        // names the set; every other set keeps the badges.
        assert!(set.no_badges);
        assert!(!badges_with(Some("graphite")));
        for name in ["metro", "adwaita", "breeze", "luna", "aero", "aqua"] {
            assert!(!self::set(name).unwrap().no_badges, "{name}");
            assert!(badges_with(Some(name)), "{name}");
        }
        // The classic look, and a theme naming a set that isn't there: the badges.
        assert!(badges_with(None));
        assert!(badges_with(Some("no-such-set")));
    }

    /// A one-colour picture as the set holds it: its greys white, so the ink tints them.
    fn assert_white(png: &[u8], need: &str) {
        let img = image::load_from_memory(png).unwrap().to_rgba8();
        assert!(img.pixels().any(|p| p.0[3] > 0), "{need}: empty");
        for p in img.pixels().filter(|p| p.0[3] > 0) {
            let [r, g, b, _] = p.0;
            let grey = r.max(g).max(b) - r.min(g).min(b) <= 24;
            assert!(!grey || [r, g, b] == [255, 255, 255], "{need}: {:?}", p.0);
        }
    }

    /// `<need>-symbolic.png` is a one-colour picture: its greys made white (their transparency
    /// kept) to take the ink, its colours kept, as the SVGs' are.
    #[test]
    fn a_one_colour_picture_is_made_white() {
        let mut img = image::RgbaImage::new(2, 1);
        img.put_pixel(0, 0, image::Rgba([0x24, 0x24, 0x24, 200]));
        img.put_pixel(1, 0, image::Rgba([0xda, 0x44, 0x53, 255]));
        let mut png = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let out = image::load_from_memory(&whitened(&png)).unwrap().to_rgba8();
        assert_eq!(out.get_pixel(0, 0).0, [255, 255, 255, 200]);
        assert_eq!(out.get_pixel(1, 0).0, [0xda, 0x44, 0x53, 255]);
        let set = Set::from_files([(16, "tb-stop-symbolic.png", Arc::from(png))], false);
        let f = set.file("tb-stop", 16.0).unwrap().1;
        assert!(f.mono && f.png);
        // Not a PNG: kept as it is (and drawn as nothing, as any broken file).
        assert_eq!(&whitened(b"nope")[..], b"nope");
    }

    /// Luna, Aero and Aqua have a colour picture for every need, at 16, 24, 32 and 48 pixels, but
    /// the title bar's four, which their title bars draw themselves; Hide to the notification area
    /// is a picture.
    #[test]
    fn era_sets_have_every_picture_but_the_title_bars_glyphs() {
        for name in ["luna", "aero", "aqua"] {
            let set = set(name).unwrap();
            for (need, _) in NEEDS {
                let bar = matches!(
                    *need,
                    "ui-minimize" | "ui-maximize" | "ui-restore" | "ui-close"
                );
                for px in SIZES {
                    let f = set.files.get(&((*need).to_owned(), px));
                    assert_eq!(f.is_none(), bar, "{name} {need} {px}");
                    let Some(f) = f else { continue };
                    assert!(f.png && !f.mono, "{name} {need} {px}");
                    let img = image::load_from_memory(&f.bytes).unwrap();
                    assert_eq!((img.width(), img.height()), (px, px), "{name} {need}");
                }
            }
            // Each picture up to a pixel over its size on screen, the next one above.
            for (px, size) in [(16.0, 16), (17.0, 16), (18.0, 24), (26.0, 32), (34.0, 32)] {
                assert_eq!(set.file("tb-resume", px).unwrap().0, size, "{name} {px}");
            }
            assert_eq!(set.file("tb-resume", 40.0).unwrap().0, 48);
            assert_eq!(set.file("tb-resume", 96.0).unwrap().0, 48);
            assert!(set.file("ui-close", 16.0).is_none());
        }
    }

    /// Every file in a built-in set's folders is named for a need MDM asks for, and is in the
    /// program (the lists above match the folders).
    #[test]
    fn every_built_in_icon_file_is_a_need_and_carried() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/themes");
        for name in [
            "metro", "adwaita", "breeze", "luna", "aero", "aqua", "graphite",
        ] {
            let carried = set(name).unwrap();
            let mut n = 0;
            for px in SIZES {
                let folder = dir.join(name).join("icons").join(px.to_string());
                let Ok(entries) = std::fs::read_dir(&folder) else {
                    continue;
                };
                for entry in entries {
                    let file = entry.unwrap().file_name().into_string().unwrap();
                    let stem = file.rsplit_once('.').unwrap().0;
                    let need = stem.strip_suffix("-symbolic").unwrap_or(stem);
                    assert!(known(need), "{name}/{px}/{file}: no such need or extra");
                    assert!(
                        carried.files.contains_key(&(need.to_owned(), px)),
                        "{name}/{px}/{file} isn't in the program"
                    );
                    n += 1;
                }
            }
            assert_eq!(n, carried.files.len(), "{name}: one file per need and size");
        }
    }

    /// The extras (plan 03b) have ids of their own, none a need's, and only Graphite's set
    /// carries them, at 16 and 24 pixels: every other set's lookup finds none, so the classic
    /// layout, which never asks, would draw nothing for one.
    #[test]
    fn only_graphite_carries_the_extras() {
        let mut ids: Vec<&str> = NEEDS.iter().chain(EXTRAS).map(|(n, _)| *n).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "an id twice");
        assert_eq!(EXTRAS.len(), 36);
        let graphite = set("graphite").unwrap();
        for (extra, _) in EXTRAS {
            assert!(extra.starts_with("gx-"), "{extra}");
            for px in [16u32, 24] {
                assert!(
                    graphite.files.contains_key(&((*extra).to_owned(), px)),
                    "graphite lacks {extra} at {px}"
                );
            }
            for name in ["metro", "adwaita", "breeze", "luna", "aero", "aqua"] {
                let s = set(name).unwrap();
                assert!(
                    !s.has(extra) && s.file(extra, 16.0).is_none(),
                    "{name} {extra}"
                );
            }
        }
    }

    /// Every built-in set carries the sizes the loader asks for up to a screen at 300 %: a need
    /// whose largest file is a picture has it at 48 pixels and at every size between (a
    /// drawing's largest scales cleanly), so no picture is stretched.
    #[test]
    fn pictures_come_in_every_size_the_loader_asks_for() {
        // Graphite's are all drawings today (graphite_has_every_need_in_one_colour checks its
        // sizes); listed so a picture added to it later is held to the same rule.
        for name in [
            "metro", "adwaita", "breeze", "luna", "aero", "aqua", "graphite",
        ] {
            let set = set(name).unwrap();
            for (need, _) in NEEDS {
                let sizes: Vec<u32> = SIZES
                    .into_iter()
                    .filter(|s| set.files.contains_key(&((*need).to_owned(), *s)))
                    .collect();
                let Some(&largest) = sizes.last() else {
                    continue;
                };
                let f = &set.files[&((*need).to_owned(), largest)];
                if !f.png {
                    continue;
                }
                let from = sizes[0];
                let want: Vec<u32> = SIZES.into_iter().filter(|s| *s >= from).collect();
                assert_eq!(
                    sizes, want,
                    "{name} {need}: a picture at every size up to 48"
                );
                // At 100 %, 150 %, 200 % and 300 % (16-point icons), its own size, never one
                // stretched.
                for (px, size) in [(16.0, 16), (24.0, 24), (32.0, 32), (48.0, 48)] {
                    if px as u32 >= from {
                        assert_eq!(set.file(need, px).unwrap().0, size, "{name} {need} {px}");
                    }
                }
            }
        }
    }

    /// Adwaita has every need at both sizes: colour pictures at 16 pixels and drawings above,
    /// its one-colour (symbolic) icons made white to be tinted, the others keeping their colours.
    #[test]
    fn adwaita_mixes_pictures_and_symbolic_icons() {
        let adw = set("adwaita").unwrap();
        for (need, _) in NEEDS {
            for px in [16u32, 24] {
                let f = adw.files.get(&((*need).to_owned(), px));
                assert!(f.is_some(), "adwaita lacks {need} at {px}");
                let f = f.unwrap();
                if f.png {
                    assert!(f.bytes.starts_with(b"\x89PNG"), "{need} {px}");
                    // MDM's own Hide to the notification area: one colour, as GNOME's toolbar.
                    assert_eq!(f.mono, *need == "ui-to-tray", "{need} {px}");
                    if f.mono {
                        assert_white(&f.bytes, need);
                    }
                    continue;
                }
                let text = std::str::from_utf8(&f.bytes).unwrap();
                assert!(text.contains("<svg"), "{need} {px}");
                if f.mono {
                    let inked = inked(&f.bytes, "#123456");
                    let text = std::str::from_utf8(&inked).unwrap();
                    assert!(!text.contains("#2e3436"), "{need} {px} keeps GNOME's ink");
                    assert!(text.contains("#123456"), "{need} {px}");
                }
            }
        }
        let (_, f) = adw.file("tb-resume", 16.0).unwrap();
        assert!(f.mono);
        let (_, f) = adw.file("cat-music", 16.0).unwrap();
        assert!(f.png && !f.mono);
        // A 16-pixel picture at 16 pixels; at 32 (twice the size), the drawing.
        let (_, f) = adw.file("cat-music", 32.0).unwrap();
        assert!(!f.png && !f.mono);
        // The scale between: at 18 pixels the drawing too, sharper than the picture stretched.
        assert!(!adw.file("ft-audio", 18.0).unwrap().1.png);
        // Our own PDF and torrent file keep their colours.
        assert!(!adw.file("ft-pdf", 16.0).unwrap().1.mono);
        assert!(!adw.file("ft-torrent", 32.0).unwrap().1.mono);
    }

    /// Breeze has every need: KDE's one-colour icons (ColorScheme-Text) take the ink and keep
    /// their highlight and warning colours; its colour icons and our PDF keep theirs.
    #[test]
    fn breeze_has_every_need_and_keeps_its_colours() {
        let set = set("breeze").unwrap();
        for (need, _) in NEEDS {
            assert!(set.has(need), "breeze lacks {need}");
            for px in [16.0, 32.0] {
                let (_, f) = set.file(need, px).unwrap();
                // MDM's own: the PDF in colour, Hide to the notification area in one colour.
                if f.png {
                    assert!(matches!(*need, "ft-pdf" | "ui-to-tray"), "{need}");
                    assert_eq!(f.mono, *need == "ui-to-tray", "{need}");
                    continue;
                }
                let text = std::str::from_utf8(&f.bytes).unwrap();
                assert!(text.contains("<svg"), "{need}");
                assert_eq!(f.mono, text.contains("ColorScheme-Text"), "{need} {px}");
                if f.mono {
                    let inked = inked(&f.bytes, "#123456");
                    let text = std::str::from_utf8(&inked).unwrap();
                    assert!(!text.contains("#232629"), "{need} keeps KDE's ink");
                }
            }
        }
        // st-error's red stays red.
        let (_, f) = set.file("st-error", 16.0).unwrap();
        let text = String::from_utf8(inked(&f.bytes, "#123456").to_vec()).unwrap();
        assert!(text.contains("#da4453"), "{text}");
        assert!(!set.file("ft-pdf", 16.0).unwrap().1.mono);
    }

    /// A one-colour icon takes the ink in its greys (GNOME's #2e3436, KDE's ColorScheme-Text)
    /// and by default; its colours (a warning's orange, KDE's highlight) stay.
    #[test]
    fn a_one_colour_svg_takes_the_ink() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg"><style>.ColorScheme-Text { color:#232629; }</style><path fill="#2e3436" d="M0 0"/><path style="fill:#ff7800;stroke:#aaa" class="warning"/><path fill="url(#a)" stroke="none"/><path/></svg>"##;
        let out = String::from_utf8(inked(svg, "#123456").to_vec()).unwrap();
        assert!(out.starts_with(r##"<svg fill="#123456" "##), "{out}");
        assert!(out.contains("{ color:#123456; }"), "{out}");
        assert!(out.contains(r##"fill="#123456" d="##), "{out}");
        assert!(out.contains("fill:#ff7800;stroke:#123456"), "{out}");
        assert!(out.contains(r##"fill="url(#a)" stroke="none""##), "{out}");
        assert!(!out.contains("2e3436") && !out.contains("232629"), "{out}");
        // A drawing that names its own default keeps one attribute.
        let out = inked(br##"<svg fill="#000"><path/></svg>"##, "#ffffff");
        assert_eq!(&out[..], br##"<svg fill="#ffffff"><path/></svg>"##);
    }

    #[test]
    fn a_folder_set_reads_only_known_needs() {
        let dir = std::env::temp_dir().join(format!("mdm-iconset-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("16")).unwrap();
        std::fs::write(dir.join("16").join("tb-resume.svg"), "<svg/>").unwrap();
        std::fs::write(dir.join("16").join("not-a-need.svg"), "<svg/>").unwrap();
        std::fs::write(
            dir.join("16").join("tb-stop-symbolic.svg"),
            r##"<svg><path fill="#2e3436"/></svg>"##,
        )
        .unwrap();
        std::fs::write(dir.join("16").join("cat-music.png"), b"\x89PNG").unwrap();
        assert!(load_folder("test-folder-set", &dir, false));
        let s = set("test-folder-set").unwrap();
        assert!(s.has("tb-resume") && !s.has("not-a-need") && !s.has("tb-delete"));
        assert_eq!(s.file("tb-resume", 32.0).unwrap().0, 16);
        // By its name: a symbolic icon is one colour, a picture and a plain drawing keep theirs.
        assert!(s.file("tb-stop", 16.0).unwrap().1.mono);
        assert!(!s.file("tb-resume", 16.0).unwrap().1.mono);
        assert!(s.file("cat-music", 16.0).unwrap().1.png);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!load_folder(
            "empty",
            &std::env::temp_dir().join("mdm-no-such-set"),
            false
        ));
    }

    #[test]
    fn file_types_map_to_needs() {
        assert_eq!(file_need("x-office-document", Some("pdf")), "ft-pdf");
        assert_eq!(file_need("text-x-generic", Some("torrent")), "ft-torrent");
        assert_eq!(file_need("package-x-generic", Some("zip")), "ft-archive");
        assert_eq!(file_need("folder", None), "cat-folder");
    }
}
