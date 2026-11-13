//! A theme's icon set (#45): files named by what MDM needs them for (`tb-add-url`, `cat-music`,
//! `ft-pdf`…, the list in [`NEEDS`]), at 16 pixels and at 24 (drawn at 24 and up), pictures at
//! 32 and 48 too (a screen at 150 % to 300 %, [`SIZES`]). Graphite's set is built in and drawn
//! by default; a theme file can bring a set of its own, and a need its set lacks is drawn from
//! Graphite's.
//!
//! Each file is an SVG drawing or a PNG picture, and says by its name how it is inked:
//! `<need>.png` and `<need>.svg` keep their colours, `<need>-symbolic.svg` and
//! `<need>-symbolic.png` are one colour (as the freedesktop icon themes name theirs): their
//! greys are drawn in the theme's text colour (greyed when disabled), their colours kept, as
//! GNOME and KDE recolour theirs. A set of SVGs all one colour says so once (`icon-mono = true`).
//!
//! Graphite's set is carried in the program (`assets/themes/graphite/icons/{16,24}/`), the only
//! built-in one (plan 05.5c). A theme file
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
    ("gx-computer", "Server: this computer"),
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
            "gx-adjust.svg", "gx-apps.svg", "gx-browser.svg", "gx-bug.svg", "gx-computer.svg",
            "gx-copy.svg",
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
            "gx-adjust.svg", "gx-apps.svg", "gx-browser.svg", "gx-bug.svg", "gx-computer.svg",
            "gx-copy.svg",
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
    sets.insert(
        "graphite".to_owned(),
        Arc::new(Set::from_files(graphite, true)),
    );
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

/// Whether Graphite's set has `need` drawn at `px` (16 or 24).
#[cfg(test)]
pub fn graphite_has(need: &str, px: u32) -> bool {
    set("graphite").is_some_and(|s| s.files.contains_key(&(need.to_owned(), px)))
}

fn set(name: &str) -> Option<Arc<Set>> {
    SETS.lock().ok()?.get(name).cloned()
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
    let name = crate::theme::current().icon_set.clone()?;
    source_in(name, need, px, ink, enabled)
}

/// [`source`] from Graphite's own set, whatever the theme: the places Graphite's set alone
/// draws (plan 05.5a: the Options pages' headers, the helpers window, the drag handle, the
/// Scheduler's tree, the Server row).
pub fn graphite(
    need: &str,
    px: f32,
    ink: Color32,
    enabled: bool,
) -> Option<(ImageSource<'static>, Color32)> {
    source_in(Cow::Borrowed("graphite"), need, px, ink, enabled)
}

/// [`source`] from the set called `name`.
fn source_in(
    name: Cow<'static, str>,
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
/// has none for it (the caller draws Graphite's, [`paint_graphite`]).
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
    paint_source(ui, painter, source(need, px, ink, enabled), rc)
}

/// [`paint`] from Graphite's own set, whatever the theme ([`graphite`]).
pub fn paint_graphite(
    ui: &egui::Ui,
    painter: &egui::Painter,
    need: &str,
    rc: egui::Rect,
    enabled: bool,
) -> bool {
    let px = rc.width() * ui.ctx().pixels_per_point();
    paint_source(ui, painter, graphite(need, px, ink(enabled), enabled), rc)
}

/// [`paint_graphite`] in `ink` (a status banner's icon in its kind's colour).
#[allow(dead_code)] // ponytail: first used by the 07c-07e waves (status banners)
pub fn paint_graphite_in(
    ui: &egui::Ui,
    painter: &egui::Painter,
    need: &str,
    rc: egui::Rect,
    ink: Color32,
) -> bool {
    let px = rc.width() * ui.ctx().pixels_per_point();
    paint_source(ui, painter, graphite(need, px, ink, true), rc)
}

/// Draws an icon made by [`source`] into `rc`; false when there is none.
fn paint_source(
    ui: &egui::Ui,
    painter: &egui::Painter,
    made: Option<(ImageSource<'static>, Color32)>,
    rc: egui::Rect,
) -> bool {
    let Some((src, tint)) = made else {
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

/// A file's need, by its extension (lowercase; None: a file without one): its kind, the plain
/// page (`ft-text`) for one of no known kind.
pub fn file_need(ext: Option<&str>) -> &'static str {
    let Some(ext) = ext else { return "ft-text" };
    match ext {
        "torrent" => "ft-torrent",
        "pdf" => "ft-pdf",
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "ts"
        | "m2ts" | "3gp" | "ogv" | "vob" | "rm" | "rmvb" | "asf" => "ft-video",
        "mp3" | "flac" | "ogg" | "oga" | "opus" | "wav" | "m4a" | "aac" | "wma" | "ape"
        | "alac" | "aif" | "aiff" | "mid" | "midi" => "ft-audio",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tif" | "tiff" | "ico" | "heic"
        | "heif" | "avif" | "psd" | "raw" | "cr2" | "nef" | "dng" | "svg" | "eps" | "ai"
        | "odg" | "vsd" | "vsdx" | "otg" => "ft-image",
        "zip" | "rar" | "7z" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" | "txz" | "zst" | "tar"
        | "cab" | "lz" | "lzma" | "z" | "arj" | "deb" | "rpm" | "pkg" | "apk" | "jar" | "whl"
        | "nupkg" | "snap" | "flatpak" => "ft-archive",
        "exe" | "msi" | "msix" | "appx" | "com" | "scr" | "appimage" | "run" | "bin" | "app"
        | "dll" | "so" | "dylib" | "sys" => "ft-executable",
        "iso" | "img" | "dmg" | "vhd" | "vhdx" | "vmdk" | "nrg" | "mdf" | "cue" | "toast" => {
            "ft-disk-image"
        }
        "doc" | "docx" | "odt" | "rtf" | "wpd" | "pages" | "epub" | "mobi" | "azw" | "azw3"
        | "djvu" | "xps" | "fb2" | "cbz" | "cbr" | "dot" | "dotx" | "ott" => "ft-document",
        "xls" | "xlsx" | "xlsm" | "ods" | "csv" | "tsv" | "numbers" | "xlt" | "xltx" | "ots" => {
            "ft-spreadsheet"
        }
        "ppt" | "pptx" | "odp" | "key" | "pps" | "ppsx" | "pot" | "potx" | "otp" => {
            "ft-presentation"
        }
        "html" | "htm" | "xhtml" | "mht" | "mhtml" | "xml" | "sh" | "bash" | "py" | "js"
        | "mjs" | "ps1" | "bat" | "cmd" | "pl" | "rb" | "php" | "lua" | "vbs" => "ft-code",
        "ttf" | "otf" | "woff" | "woff2" | "fon" | "pfb" => "ft-font",
        // Certificates and signatures, calendars, address cards: a file of its own kind.
        "crt" | "cer" | "pem" | "der" | "p12" | "pfx" | "p7b" | "sig" | "asc" | "gpg" | "ics"
        | "ical" | "vcs" | "vcf" | "vcard" => "ft-generic",
        _ => "ft-text",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                // The Graphite icon pattern (plan 05.5a): the root's attributes, exactly.
                let root = &raw[..raw.find('>').unwrap()];
                for want in [
                    format!(r#"width="{px}""#),
                    format!(r#"height="{px}""#),
                    r#"viewBox="0 0 24 24""#.to_owned(),
                    r#"fill="none""#.to_owned(),
                    r##"stroke="#ffffff""##.to_owned(),
                    r#"stroke-width="1.75""#.to_owned(),
                    r#"stroke-linecap="round""#.to_owned(),
                    r#"stroke-linejoin="round""#.to_owned(),
                ] {
                    assert!(root.contains(&want), "{need} {px}: the root lacks {want}");
                }
                assert!(!raw.contains("class="), "{need} {px}: a class");
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

    /// Every file in a built-in set's folders is named for a need MDM asks for, and is in the
    /// program (the lists above match the folders).
    #[test]
    fn every_built_in_icon_file_is_a_need_and_carried() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/themes");
        {
            let name = "graphite";
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

    /// The extras (plan 03b) have ids of their own, none a need's, and Graphite's set carries
    /// them, at 16 and 24 pixels.
    #[test]
    fn only_graphite_carries_the_extras() {
        let mut ids: Vec<&str> = NEEDS.iter().chain(EXTRAS).map(|(n, _)| *n).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "an id twice");
        assert_eq!(EXTRAS.len(), 37);
        let graphite = set("graphite").unwrap();
        for (extra, _) in EXTRAS {
            assert!(extra.starts_with("gx-"), "{extra}");
            for px in [16u32, 24] {
                assert!(
                    graphite.files.contains_key(&((*extra).to_owned(), px)),
                    "graphite lacks {extra} at {px}"
                );
            }
        }
    }

    /// The built-in set carries the sizes the loader asks for up to a screen at 300 %: a need
    /// whose largest file is a picture has it at 48 pixels and at every size between (a
    /// drawing's largest scales cleanly), so no picture is stretched.
    #[test]
    fn pictures_come_in_every_size_the_loader_asks_for() {
        // Graphite's are all drawings today (graphite_has_every_need_in_one_colour checks its
        // sizes); listed so a picture added to it later is held to the same rule.
        {
            let name = "graphite";
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
        for (ext, need) in [
            (Some("pdf"), "ft-pdf"),
            (Some("torrent"), "ft-torrent"),
            (Some("zip"), "ft-archive"),
            (Some("ts"), "ft-video"),
            (Some("flac"), "ft-audio"),
            (Some("svg"), "ft-image"),
            (Some("otg"), "ft-image"),
            (Some("exe"), "ft-executable"),
            (Some("iso"), "ft-disk-image"),
            (Some("dotx"), "ft-document"),
            (Some("xltx"), "ft-spreadsheet"),
            (Some("potx"), "ft-presentation"),
            (Some("py"), "ft-code"),
            (Some("ttf"), "ft-font"),
            (Some("pem"), "ft-generic"),
            (Some("ics"), "ft-generic"),
            (Some("xyz"), "ft-text"),
            (None, "ft-text"),
        ] {
            assert_eq!(file_need(ext), need, "{ext:?}");
        }
    }
}
