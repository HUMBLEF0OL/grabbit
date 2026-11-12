//! The look: light or dark (the mode), the active theme's colours and sizes (#45), the fonts,
//! and egui's own visuals built from them.

use egui::{CornerRadius, Stroke, Visuals};

use std::cell::Cell;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::palette::{CLASSIC, Metrics, Palette, Theme};

static DARK: AtomicBool = AtomicBool::new(true);

/// Is the UI in dark mode? IDM: dark when "View → Dark Mode support" is on and Windows' app
/// mode is dark (IDM 6.37); light otherwise. It picks the active theme's dark or light variant
/// ([`colors`]); whether that variant is a dark surface is [`dark_surface`].
pub fn dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

// ---- the active theme ------------------------------------------------------------------------
//
// Painters read colours and sizes hundreds of times a frame, from every window's thread: each
// thread keeps a reference to the active theme and looks again only when the generation moved
// (one atomic load per read, no lock, no allocation). A theme set from a file is leaked: it
// lives as long as the program (a few KB per change), so a reference to it never dangles.

static ACTIVE: RwLock<&'static Theme> = RwLock::new(&CLASSIC);
static GENERATION: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static SEEN: Cell<(u64, &'static Theme)> = const { Cell::new((0, &CLASSIC)) };
}

/// The active theme.
pub fn current() -> &'static Theme {
    let generation = GENERATION.load(Ordering::Acquire);
    SEEN.with(|seen| {
        let (at, theme) = seen.get();
        if at == generation {
            return theme;
        }
        let theme = *ACTIVE.read().unwrap_or_else(|e| e.into_inner());
        seen.set((generation, theme));
        theme
    })
}

/// The active theme's colours in the current mode.
pub fn colors() -> &'static Palette {
    current().palette(dark())
}

/// The active theme's sizes.
pub fn metrics() -> &'static Metrics {
    &current().metrics
}

/// The font family of numbers that must line up as they change (the download list's sizes,
/// rates, time left and percentages): Segoe UI, whatever the theme's font.
pub const NUMBERS: &str = "numbers";

/// The active theme's layout: IDM's or Graphite's.
pub fn chrome() -> crate::palette::ChromeLook {
    current().controls.chrome
}

/// Is what is drawn now a dark surface (light text on dark)? The classic look's dark variant
/// is; a theme says for each of its variants. Glyph rendering and the window captions follow it.
pub fn dark_surface() -> bool {
    colors().dark
}

/// Makes `theme` the active one.
pub fn set(theme: Theme) {
    let theme: &'static Theme = match crate::palette::builtins()
        .into_iter()
        .find(|b| **b == theme)
    {
        Some(builtin) => builtin,
        None => Box::leak(Box::new(theme)),
    };
    *ACTIVE.write().unwrap_or_else(|e| e.into_inner()) = theme;
    GENERATION.fetch_add(1, Ordering::AcqRel);
}

/// Makes the theme with this id the active one: a built-in, or a file in the themes folder
/// (`themefile`), or a theme file's path; Native is made from what the system says. A theme
/// that can't be read leaves the classic look, logged. Returns what was skipped or went wrong.
pub fn select(id: &str, ctx: &egui::Context) -> Vec<String> {
    let id = id.to_owned();
    // pywal's file is followed again only if this theme is made from it (`themefile::load_in`).
    crate::themefile::forget_pywal();
    *SELECTED.lock().unwrap_or_else(|e| e.into_inner()) = id.clone();
    if id.is_empty() || id == CLASSIC.id {
        set(CLASSIC.clone());
        return Vec::new();
    }
    if id == "native" {
        let system = crate::native::system(ctx);
        *NATIVE_SEEN.lock().unwrap_or_else(|e| e.into_inner()) = Some(system);
        set(crate::native::theme(&system));
        return Vec::new();
    }
    match load(&id) {
        Ok(loaded) => {
            for w in &loaded.warnings {
                tracing::warn!("theme {id}: {w}");
            }
            set(loaded.theme);
            loaded.warnings
        }
        Err(e) => {
            tracing::warn!("theme {id}: {e}; the classic look instead");
            set(CLASSIC.clone());
            vec![e]
        }
    }
}

/// A theme file's theme, by its id or its path.
fn load(id: &str) -> Result<crate::themefile::Loaded, String> {
    let path = std::path::Path::new(id);
    if path.is_file() {
        crate::themefile::load_file(path)
    } else {
        crate::themefile::load(id)
    }
}

/// The id (or path) [`select`] was last given.
static SELECTED: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// While the theme is made from pywal's colours (#45): read again when `wal` wrote them anew,
/// as it does for each new wallpaper. A file that doesn't read (half written as it was read)
/// keeps the theme as it is, and is read again when it is next written. True when the theme
/// changed (the caller applies it: `apply_theme`, `apply_mode`).
pub fn follow_pywal() -> bool {
    if !crate::themefile::pywal_changed() {
        return false;
    }
    let id = SELECTED.lock().unwrap_or_else(|e| e.into_inner()).clone();
    match load(&id) {
        Ok(loaded) => {
            tracing::info!("theme {id}: pywal's colours changed, read again");
            set(loaded.theme);
            true
        }
        Err(e) => {
            tracing::warn!("theme {id}: {e}; the theme kept until pywal's file is written again");
            crate::themefile::pywal_seen();
            false
        }
    }
}

/// The theme to start with: `MDM_THEME` (tests and screenshots: an id or a theme file's path),
/// else the setting.
pub fn startup_id(setting: &str) -> String {
    std::env::var("MDM_THEME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| setting.to_owned())
}

/// What the system said when Native was last made.
static NATIVE_SEEN: std::sync::Mutex<Option<crate::native::System>> = std::sync::Mutex::new(None);

/// While Native is the theme: made again when the system's accent or contrast changed. True
/// when it was (the caller applies it: `apply_theme`, `apply_mode`).
pub fn follow_system(ctx: &egui::Context) -> bool {
    if current().id != "native" {
        return false;
    }
    let system = crate::native::system(ctx);
    let mut seen = NATIVE_SEEN.lock().unwrap_or_else(|e| e.into_inner());
    if *seen == Some(system) {
        return false;
    }
    *seen = Some(system);
    drop(seen);
    set(crate::native::theme(&system));
    true
}

/// Match egui's glyph rendering to the surface: a dark one thickens light-on-dark text
/// (`TwoCoverageMinusCoverageSq`), which makes black-on-white text look heavy, so a light one
/// uses plain coverage. The glyph atlas is shared, so this follows the main window's.
pub fn apply_mode(ctx: &egui::Context) {
    let dark_surface = dark_surface();
    let tf = if dark_surface {
        egui::epaint::FontColorTransferFunction::DARK_MODE_DEFAULT
    } else {
        egui::epaint::FontColorTransferFunction::LIGHT_MODE_DEFAULT
    };
    ctx.all_styles_mut(|s| s.visuals.text_options.color_transfer_function = tf);
    // eframe applies egui's theme preference to every window's caption; keep it in step so it
    // doesn't reset the captions to the system theme. egui's dark or light style is the mode's
    // variant (`apply_theme` puts each variant into its egui theme).
    ctx.set_theme(if dark() {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    // The window's own theme (caption, DWM): otherwise winit follows the system theme.
    ctx.send_viewport_cmd_to(
        egui::ViewportId::ROOT,
        egui::ViewportCommand::SetTheme(system_theme(dark_surface)),
    );
}

/// The winit window theme for a dark (true) or light caption.
pub fn system_theme(dark: bool) -> egui::SystemTheme {
    if dark {
        egui::SystemTheme::Dark
    } else {
        egui::SystemTheme::Light
    }
}

/// Decide the mode from the setting and the system's: Windows' app mode. True if it changed.
/// `always_dark` (View → Always dark, #60): dark whatever the system says.
pub fn update(dark_mode_support: bool, always_dark: bool, ctx: &egui::Context) -> bool {
    let _ = ctx;
    let system_dark = crate::win32::apps_use_dark();
    let want = wanted(dark_mode_support, always_dark, system_dark);
    DARK.swap(want, Ordering::Relaxed) != want
}

/// Dark or not: View → Always dark, else the system's mode while Dark Mode support is on.
fn wanted(dark_mode_support: bool, always_dark: bool, system_dark: bool) -> bool {
    always_dark || (dark_mode_support && system_dark)
}

/// A module's names for the active theme's colours: `pub NAME => role;` becomes `NAME()`, the
/// role's colour in the current mode ([`colors`]).
macro_rules! themed {
    ($($(#[$m:meta])* $vis:vis $name:ident => $role:ident;)*) => {$(
        $(#[$m])*
        #[allow(non_snake_case)]
        $vis fn $name() -> egui::Color32 {
            $crate::theme::colors().$role
        }
    )*};
}
pub(crate) use themed;

/// Marks that the interface font doesn't have (✔ ✖ ↩ ⇧ ⌘ ⌥ ⌦): the system's symbol fonts, last
/// in every family's chain, in every language, each one that is there.
const SYMBOL_FONTS: &[&str] = &["seguisym.ttf"];

/// The symbol fonts' bytes, in fallback order ([`SYMBOL_FONTS`]).
fn symbol_fonts() -> Vec<Vec<u8>> {
    SYMBOL_FONTS
        .iter()
        .filter_map(|f| std::fs::read(font_dir().join(f)).ok())
        .collect()
}

/// The folder the font tables name files in.
fn font_dir() -> std::path::PathBuf {
    std::env::var_os("WINDIR")
        .map(|w| std::path::PathBuf::from(w).join("Fonts"))
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows\Fonts"))
}

/// The fonts for scripts the interface font lacks, loaded the first time a character needs one
/// (#405): a file or video named in Bangla, Japanese, Thai... They come from the system on
/// demand, only when text in that script is shown (CJK fonts are 13–20 MB each, #72).
///
/// The blocks below name the system's font for each script.
struct ScriptFallback;

/// One script's fonts on demand: a key, its code points, the files to try (in order).
struct FallbackFont {
    key: &'static str,
    ranges: &'static [(u32, u32)],
    files: &'static [&'static str],
}

/// Windows' fonts for the scripts egui's fonts and Segoe UI don't cover.
const FALLBACK_FONTS: &[FallbackFont] = &[
    FallbackFont {
        key: "indic",
        ranges: &[(0x0900, 0x0DFF), (0x1C50, 0x1C7F)],
        files: &["Nirmala.ttc", "Nirmala.ttf"],
    },
    FallbackFont {
        key: "thai-lao-khmer",
        ranges: &[(0x0E00, 0x0EFF), (0x1780, 0x17FF), (0x19E0, 0x19FF)],
        files: &["LeelawUI.ttf", "leelawdb.ttf", "tahoma.ttf"],
    },
    FallbackFont {
        key: "myanmar",
        ranges: &[(0x1000, 0x109F), (0xA9E0, 0xA9FF), (0xAA70, 0xAA7F)],
        files: &["mmrtext.ttf"],
    },
    FallbackFont {
        key: "tibetan",
        ranges: &[(0x0F00, 0x0FFF)],
        files: &["himalaya.ttf"],
    },
    FallbackFont {
        key: "ethiopic",
        ranges: &[(0x1200, 0x139F), (0x2D80, 0x2DDF), (0xAB00, 0xAB2F)],
        files: &["ebrima.ttf"],
    },
    FallbackFont {
        key: "cherokee",
        ranges: &[(0x13A0, 0x13FF), (0xAB70, 0xABBF)],
        files: &["gadugi.ttf"],
    },
    FallbackFont {
        key: "hangul",
        ranges: &[
            (0x1100, 0x11FF),
            (0x3130, 0x318F),
            (0xA960, 0xA97F),
            (0xAC00, 0xD7FF),
        ],
        files: &["malgun.ttf"],
    },
    FallbackFont {
        key: "japanese",
        ranges: &[(0x3040, 0x30FF), (0x31F0, 0x31FF), (0xFF66, 0xFF9F)],
        files: &["YuGothM.ttc", "meiryo.ttc", "msgothic.ttc"],
    },
    FallbackFont {
        key: "han",
        ranges: &[
            (0x2E80, 0x2FFF),
            (0x3000, 0x303F),
            (0x3100, 0x312F),
            (0x31A0, 0x31BF),
            (0x3200, 0x33FF),
            (0x3400, 0x4DBF),
            (0x4E00, 0x9FFF),
            (0xF900, 0xFAFF),
            (0xFE30, 0xFE4F),
            (0xFF00, 0xFF65),
            (0xFFE0, 0xFFEF),
            (0x20000, 0x3FFFF),
        ],
        files: &["msyh.ttc", "msjh.ttc", "YuGothM.ttc", "msgothic.ttc"],
    },
];

/// The font in `fonts` for `c`: the first that lists its code point.
fn fallback_entry(fonts: &'static [FallbackFont], c: char) -> Option<&'static FallbackFont> {
    let cp = c as u32;
    fonts
        .iter()
        .find(|f| f.ranges.iter().any(|&(a, b)| (a..=b).contains(&cp)))
}

/// The bytes of the font for `c`, loaded once per font, shared by every family that asks.
fn fallback_font(c: char) -> Option<(String, egui::FontData)> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static LOADED: Mutex<Option<HashMap<String, Option<egui::FontData>>>> = Mutex::new(None);

    let (key, files): (String, Vec<std::path::PathBuf>) = {
        let font = fallback_entry(FALLBACK_FONTS, c)?;
        let dir = font_dir();
        (
            format!("fallback:{}", font.key),
            font.files.iter().map(|f| dir.join(f)).collect(),
        )
    };

    let mut loaded = LOADED.lock().unwrap_or_else(|e| e.into_inner());
    let loaded = loaded.get_or_insert_with(HashMap::new);
    let font = loaded
        .entry(key.clone())
        .or_insert_with(|| {
            let bytes = files.iter().find_map(|f| std::fs::read(f).ok())?;
            let mut tweak = egui::FontTweak::default();
            tweak.set_baseline_fallback(true);
            tweak.hinting_target =
                egui::epaint::text::HintingTarget::Smooth(egui::epaint::text::SmoothHinting {
                    light: false,
                    symmetric_rendering: false,
                    preserve_linear_metrics: false,
                });
            tweak.subpixel_binning = Some(false);
            Some(egui::FontData::from_owned(bytes).tweak(tweak))
        })
        .clone()?;
    Some((key, font))
}

impl egui::FontProvider for ScriptFallback {
    fn font_for(&self, request: &egui::FallbackRequest<'_>) -> Option<egui::FontInsert> {
        let c = request.cluster.chars().next()?;
        let (key, data) = fallback_font(c)?;
        Some(egui::FontInsert::new(&key, data, Vec::new()))
    }
}

/// Tell the text system where the scripts the interface font lacks come from, once.
fn add_script_fallback(ctx: &egui::Context) {
    static ADDED: std::sync::Once = std::sync::Once::new();
    ADDED.call_once(|| ctx.add_font_provider(std::sync::Arc::new(ScriptFallback)));
}

/// The interface font: Segoe UI as IDM has it.
fn ui_font() -> Option<Vec<u8>> {
    std::fs::read(font_dir().join("segoeui.ttf")).ok()
}

/// A font MDM carries for a theme to name (`font = "geist"`): its Regular, Medium and SemiBold.
/// Geist, by Vercel, under the SIL Open Font License 1.1 (`assets/fonts/geist/OFL.txt`).
pub fn bundled_font(name: &str) -> Option<[&'static [u8]; 3]> {
    match name {
        "geist" => Some([
            include_bytes!("../assets/fonts/geist/Geist-Regular.ttf"),
            include_bytes!("../assets/fonts/geist/Geist-Medium.ttf"),
            include_bytes!("../assets/fonts/geist/Geist-SemiBold.ttf"),
        ]),
        _ => None,
    }
}

/// The bundled font leading the families now (set by [`fonts_for`]), which DirectWrite then
/// draws from these bytes instead of a system font file (`crate::win32::text_rasterizer`).
static BUNDLED: std::sync::Mutex<Option<[&'static [u8]; 3]>> = std::sync::Mutex::new(None);

pub fn bundled_face() -> Option<[&'static [u8]; 3]> {
    *BUNDLED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Tests that draw text or make fonts share the process-wide [`BUNDLED`]: one at a time, each
/// starting and ending with the system's fonts.
#[cfg(test)]
pub fn font_tests() -> impl Drop {
    struct One(#[allow(dead_code)] std::sync::MutexGuard<'static, ()>);
    impl Drop for One {
        fn drop(&mut self) {
            *BUNDLED.lock().unwrap_or_else(|e| e.into_inner()) = None;
        }
    }
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let one = One(LOCK.lock().unwrap_or_else(|e| e.into_inner()));
    *BUNDLED.lock().unwrap_or_else(|e| e.into_inner()) = None;
    one
}

/// The font the families were last made with: a theme switch remakes them only when it differs.
static LOADED_FONT: std::sync::Mutex<Option<Option<String>>> = std::sync::Mutex::new(None);

/// The interface's fonts, set up for `ctx`.
fn load_fonts(ctx: &egui::Context) {
    classic_text(ctx);
    fonts(ctx);
}

/// The active theme's fonts.
fn fonts(ctx: &egui::Context) {
    load_font(ctx, current().font.as_deref());
}

/// The families made with `font`, which is remembered for [`fonts_if_changed`].
fn load_font(ctx: &egui::Context, font: Option<&str>) {
    *LOADED_FONT.lock().unwrap_or_else(|e| e.into_inner()) = Some(font.map(str::to_owned));
    fonts_for(ctx, font);
}

/// A theme switch to a theme whose font is `font`: the fonts are made again only when it isn't
/// theirs.
fn fonts_if_changed(ctx: &egui::Context, font: Option<&str>) {
    let loaded = LOADED_FONT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if needs_reload(loaded.as_ref(), font) {
        load_font(ctx, font);
    }
}

/// `loaded`: the font the families were made with (None: not made yet; Some(None): the system's).
fn needs_reload(loaded: Option<&Option<String>>, wanted: Option<&str>) -> bool {
    loaded.is_none_or(|l| l.as_deref() != wanted)
}

/// IDM draws with Segoe UI 9 pt (12 px at 100 %): the system's own copy (else egui's default
/// font). Every family
/// falls back to it (Tahoma has no Armenian or Georgian), then to the symbol fonts; other
/// scripts (a file named in Japanese) come on demand ([`ScriptFallback`]).
///
/// A theme's bundled `font` ([`bundled_font`]) leads instead: Regular for the two built-in
/// families and Tahoma's, SemiBold for the bold ones, and `geist-medium` (Medium) for buttons,
/// column heads and chips; Segoe UI is then the first fallback, on the bundled font's baseline.
fn fonts_for(ctx: &egui::Context, font: Option<&str>) {
    let mut defs = egui::FontDefinitions::default();
    let ui = ui_font();
    if let Some(bytes) = &ui {
        defs.font_data.insert(
            "segoe-ui".into(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes.clone())),
        );
        for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            defs.families
                .entry(fam)
                .or_default()
                .insert(0, "segoe-ui".into());
        }
    }
    crate::winui::install_fonts(&mut defs, Some(&font_dir()));
    // Without a bundled font, Medium is the interface font itself.
    let medium = egui::FontFamily::Name("geist-medium".into());
    let ui_list = defs.families[&egui::FontFamily::Proportional].clone();
    defs.families.insert(medium.clone(), ui_list.clone());
    // The list's numbers (sizes, rates, time left, percentages; plan 05b) keep Segoe UI's
    // tabular figures in every theme: a bundled font's may be proportional (Geist's are, 04).
    defs.families
        .insert(egui::FontFamily::Name(NUMBERS.into()), ui_list);
    let faces = font.and_then(bundled_font);
    if let Some([regular, medium_face, semibold]) = faces {
        use egui::FontFamily::{Monospace, Name, Proportional};
        for (key, bytes) in [
            ("bundled", regular),
            ("bundled-medium", medium_face),
            ("bundled-semibold", semibold),
        ] {
            defs.font_data.insert(
                key.into(),
                std::sync::Arc::new(egui::FontData::from_static(bytes)),
            );
        }
        // Segoe UI right after it, for the letters it lacks, before egui's own fonts.
        let lead = |list: &mut Vec<String>, key: &str| {
            list.retain(|f| f != "segoe-ui");
            list.insert(0, key.into());
            if ui.is_some() {
                list.insert(1, "segoe-ui-fallback".into());
            }
        };
        for (family, key) in [
            (Proportional, "bundled"),
            (Monospace, "bundled"),
            (Name("tahoma".into()), "bundled"),
            (Name("tahoma-bold".into()), "bundled-semibold"),
            (Name("arial-bold".into()), "bundled-semibold"),
            (medium, "bundled-medium"),
        ] {
            let list = defs.families.entry(family).or_default();
            // Tahoma's own files give way too: the family is the bundled face's alone.
            list.retain(|f| !matches!(f.as_str(), "tahoma" | "tahoma-bold" | "arial-bold"));
            lead(list, key);
        }
    }
    *BUNDLED.lock().unwrap_or_else(|e| e.into_inner()) = faces;
    // A fallback's glyphs sit on the baseline of the family's first font (Japanese next to
    // Latin digits, Georgian in a Tahoma line), not centred in its row.
    let on_baseline = |bytes: Vec<u8>| {
        let mut tweak = egui::FontTweak::default();
        tweak.set_baseline_fallback(true);
        std::sync::Arc::new(egui::FontData::from_owned(bytes).tweak(tweak))
    };
    let mut fallbacks: Vec<String> = Vec::new();
    if let Some(bytes) = ui {
        defs.font_data
            .insert("segoe-ui-fallback".into(), on_baseline(bytes));
        fallbacks.push("segoe-ui-fallback".into());
    }
    for (k, bytes) in symbol_fonts().into_iter().enumerate() {
        let key = format!("symbols-{k}");
        defs.font_data.insert(key.clone(), on_baseline(bytes));
        fallbacks.push(key);
    }
    for list in defs.families.values_mut() {
        for f in &fallbacks {
            // Segoe UI already leads the two built-in families (as itself, not a fallback).
            if f == "segoe-ui-fallback" && list.first().is_some_and(|l| l == "segoe-ui") {
                continue;
            }
            if !list.contains(f) {
                list.push(f.clone());
            }
        }
    }
    sharp_text(&mut defs);
    ctx.set_fonts(defs);
    text_styles(ctx);
}

/// Latin, Greek and Cyrillic text drawn the way Windows' classic controls draw it (#262, #323,
/// #334), by DirectWrite ([`crate::win32::text_rasterizer`]): the fonts' shapes and, with them,
/// whole-pixel widths, so a letter's width and the place it is drawn for finally agree. That
/// disagreement is what glued some letters together and pushed others apart. Set once.
fn classic_text(ctx: &egui::Context) {
    static SET: std::sync::Once = std::sync::Once::new();
    SET.call_once(|| {
        ctx.set_glyph_rasterizers(
            crate::win32::text_rasterizer(font_dir())
                .into_iter()
                .collect(),
        );
    });
}

/// The text [`classic_text`] leaves to egui (Arabic, Hebrew, Indic, Thai, CJK; marks clustered
/// with their letter), fitted to whole pixels (#262) with hinting for egui's anti-aliased output.
/// Mono hinting targets black-and-white bitmaps: at small sizes it can distort curves and make
/// diagonals jagged. Smooth hinting keeps those outlines suited to coverage rendering, while
/// allowing horizontal grid-fitting so upright strokes stay sharp. Keep whole-pixel glyph origins
/// and the shaper's advances, so this changes neither layout nor the interface size.
fn sharp_text(defs: &mut egui::FontDefinitions) {
    for data in defs.font_data.values_mut() {
        let data = std::sync::Arc::make_mut(data);
        data.tweak.hinting_target =
            egui::epaint::text::HintingTarget::Smooth(egui::epaint::text::SmoothHinting {
                light: false,
                symmetric_rendering: false,
                preserve_linear_metrics: false,
            });
        data.tweak.subpixel_binning = Some(false);
    }
}

/// egui's text sizes from the theme (tooltips, multi-line boxes).
fn text_styles(ctx: &egui::Context) {
    let m = metrics();
    ctx.all_styles_mut(|s| {
        use egui::{FontId, TextStyle};
        s.text_styles = [
            (TextStyle::Small, FontId::proportional(m.font_small)),
            (TextStyle::Body, FontId::proportional(m.font_body)),
            (TextStyle::Button, FontId::proportional(m.font_body)),
            (TextStyle::Monospace, FontId::monospace(m.font_body)),
            (TextStyle::Heading, FontId::proportional(m.font_heading)),
        ]
        .into();
    });
}

pub fn apply(ctx: &egui::Context) {
    add_script_fallback(ctx);
    load_fonts(ctx);
    // A press held for a while is still a click when let go over the control, as in Windows
    // (egui gives it up after 0.8 s).
    ctx.options_mut(|o| {
        o.input_options.max_click_duration = f64::INFINITY;
        // Ctrl + / Ctrl − / Ctrl 0 step through View → Interface size and are saved (scale.rs),
        // not egui's own 10 % steps.
        o.zoom_with_keyboard = false;
    });
    egui_extras::install_image_loaders(ctx);
    apply_theme(ctx);
}

/// egui's own visuals from the active theme: each variant into its egui theme (the mode
/// switches between them), so tooltips and a multi-line box's scroll bar follow the theme too.
pub fn apply_theme(ctx: &egui::Context) {
    // Choosing Graphite or leaving it swaps the face live.
    fonts_if_changed(ctx, current().font.as_deref());
    let theme = current();
    let m = &theme.metrics;
    let radius = |r: f32| CornerRadius::same(r.round().clamp(0.0, 255.0) as u8);
    let visuals = |base: Visuals, c: &Palette| {
        let mut v = base;
        v.panel_fill = c.egui_panel;
        v.window_fill = c.egui_window;
        v.extreme_bg_color = c.egui_extreme;
        v.faint_bg_color = c.egui_faint;
        v.override_text_color = Some(c.egui_text);
        v.selection.bg_fill = c.selection;
        v.selection.stroke = Stroke::new(1.0, c.egui_selection_text);
        // Square-ish Windows controls rather than egui's rounded ones.
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = radius(m.widget_radius);
        }
        v.widgets.inactive.bg_fill = c.egui_widget;
        v.widgets.inactive.weak_bg_fill = c.egui_widget;
        // Transparent: egui's own (light widgets have none).
        if c.egui_widget_border != egui::Color32::TRANSPARENT {
            v.widgets.inactive.bg_stroke = Stroke::new(1.0, c.egui_widget_border);
        }
        v.window_corner_radius = CornerRadius::same(0);
        // Tooltips as Windows 11's: rounded corners, a hairline border, roomy padding.
        v.menu_corner_radius = radius(m.popup_radius);
        v.window_stroke = Stroke::new(1.0, c.egui_window_border);
        v
    };
    ctx.set_visuals_of(egui::Theme::Dark, visuals(Visuals::dark(), &theme.dark));
    ctx.set_visuals_of(egui::Theme::Light, visuals(Visuals::light(), &theme.light));
    let margin = |v: f32| v.round().clamp(0.0, 127.0) as i8;
    ctx.all_styles_mut(|s| {
        s.spacing.tooltip_width = m.tooltip_width;
        s.spacing.menu_margin = egui::Margin {
            left: margin(m.tooltip_margin_x),
            right: margin(m.tooltip_margin_x),
            top: margin(m.tooltip_margin_top),
            bottom: margin(m.tooltip_margin_bottom),
        };
    });
    text_styles(ctx);
}

#[cfg(test)]
mod mode_tests {
    use super::wanted;

    /// #60: Always dark wins whatever the system says; otherwise Dark Mode support follows it.
    #[test]
    fn always_dark_wins_else_the_system_decides() {
        for system in [false, true] {
            for support in [false, true] {
                assert!(wanted(support, true, system), "always dark");
            }
            assert_eq!(wanted(true, false, system), system, "following the system");
            assert!(!wanted(false, false, system), "support off: light");
        }
    }
}

/// Every mark a window draws (punctuation, arrows, ✔, quotation marks…: what isn't a letter or a
/// digit) has a glyph in the interface's font chain, so none comes out as an empty box (#185:
/// "✔ Connected" in the browser offer window).
#[cfg(test)]
mod glyph_tests {

    /// #405: a script the interface font lacks: Bangla in a file name, Japanese in a
    /// playlist, under an English interface. Before: nothing loaded it, so boxes.
    #[test]
    fn text_in_other_scripts_gets_its_font_on_demand() {
        use egui::epaint::text::{Fonts, TextOptions};
        use egui::{FontFamily, FontId};
        let mut fonts = Fonts::new(TextOptions::default(), egui::FontDefinitions::default())
            .with_font_providers(vec![std::sync::Arc::new(super::ScriptFallback)]);
        let id = FontId::new(13.0, FontFamily::Proportional);
        for text in [
            "\u{09AC}\u{09BE}\u{0982}\u{09B2}\u{09BE}", // Bangla
            "\u{65E5}\u{672C}\u{8A9E}\u{306E}\u{30BF}\u{30A4}\u{30C8}\u{30EB}", // Japanese
            "\u{D55C}\u{AD6D}\u{C5B4}",                 // Korean
            "\u{0E44}\u{0E17}\u{0E22}",                 // Thai
            "\u{4E2D}\u{6587}",                         // Chinese
        ] {
            assert!(fonts.has_glyphs(&id, text), "no font for {text}");
        }
    }

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use unicode_segmentation::UnicodeSegmentation;

    const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

    /// Drawn as nothing on purpose: bidi controls and isolates, joiners, the soft hyphen, the
    /// BOM (craft-i18n's isolates around data, the translations' own marks).
    fn invisible(c: char) -> bool {
        matches!(c, '\u{00ad}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2069}' | '\u{feff}' | '\u{fe00}'..='\u{fe0f}')
    }

    /// The marks in `text`: each grapheme's first character that is neither a letter, a digit,
    /// a space nor invisible (a grapheme's other characters are its accents and joiners).
    fn marks(text: &str, out: &mut BTreeSet<char>) {
        for g in text.graphemes(true) {
            if let Some(c) = g.chars().next()
                && !c.is_alphanumeric()
                && !c.is_whitespace()
                && !c.is_control()
                && !invisible(c)
            {
                out.insert(c);
            }
        }
    }

    /// The marks the code draws ("\u{2714} {}", "→"): every
    /// string literal in the app's sources, escapes read.
    fn literals() -> BTreeSet<char> {
        let mut out = BTreeSet::new();
        let dir = Path::new(ROOT).join("crates/app/src");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let src = std::fs::read_to_string(&path).unwrap();
            // The tests' own data (other scripts, emoji in file names) is not drawn.
            let src = src.split("#[cfg(test)]").next().unwrap_or_default();
            for lit in src.split('"').skip(1).step_by(2) {
                let mut s = String::new();
                let mut rest = lit;
                while let Some(at) = rest.find("\\u{") {
                    s.push_str(&rest[..at]);
                    let end = rest[at..].find('}').map_or(rest.len(), |e| at + e);
                    if let Some(c) = u32::from_str_radix(&rest[at + 3..end], 16)
                        .ok()
                        .and_then(char::from_u32)
                    {
                        s.push(c);
                    }
                    rest = &rest[(end + 1).min(rest.len())..];
                }
                s.push_str(rest);
                // Only what isn't plain ASCII: every font has that. Nor CJK punctuation (the line
                // breaker's table in winui.rs): it only ever shows inside CJK text, a file name
                // whose font comes on demand (ScriptFallback).
                let mut found = BTreeSet::new();
                marks(&s, &mut found);
                out.extend(found.into_iter().filter(|c| {
                    !c.is_ascii() && !matches!(*c as u32, 0x3000..=0x30FF | 0xFF00..=0xFFEF)
                }));
            }
        }
        out
    }

    #[test]
    fn every_mark_has_a_glyph() {
        let needed = literals();
        let families = [
            egui::FontFamily::Name("tahoma".into()),
            egui::FontFamily::Name("tahoma-bold".into()),
            egui::FontFamily::Proportional,
            egui::FontFamily::Name("geist-medium".into()),
        ];
        let _one = super::font_tests();
        let ctx = egui::Context::default();
        let mut missing: BTreeMap<String, BTreeSet<char>> = BTreeMap::new();
        // The system's font and Graphite's bundled one (plan 04).
        for font in [None, Some("geist")] {
            super::fonts_for(&ctx, font);
            let mut out = ctx.run_ui(Default::default(), |_| {});
            out.textures_delta.clear();
            for family in &families {
                let id = egui::FontId::new(11.0, family.clone());
                let lacks: Vec<char> = ctx.fonts_mut(|f| {
                    needed
                        .iter()
                        .copied()
                        .filter(|c| !f.has_glyph(&id, *c))
                        .collect()
                });
                if !lacks.is_empty() {
                    missing
                        .entry(format!("{font:?} {family:?}"))
                        .or_default()
                        .extend(lacks);
                }
            }
        }
        let report: Vec<String> = missing
            .iter()
            .map(|(k, v)| {
                format!(
                    "{k}: {}",
                    v.iter()
                        .map(|c| format!("{c} U+{:04X}", *c as u32))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();
        assert!(
            report.is_empty(),
            "marks without a glyph:\n{}",
            report.join("\n")
        );
    }

    /// Plan 04, step 4: DirectWrite draws Geist from MDM's own bytes, as it draws Segoe UI from
    /// its file: whole-pixel advances (egui's own are fractional), and Geist's figures, unlike
    /// Segoe UI's, are not all one width.
    #[test]
    fn directwrite_draws_the_bundled_face() {
        let _one = super::font_tests();
        let width = |font: Option<&str>, text: &str| {
            let ctx = egui::Context::default();
            ctx.set_glyph_rasterizers(
                crate::win32::text_rasterizer(super::font_dir())
                    .into_iter()
                    .collect(),
            );
            super::fonts_for(&ctx, font);
            let mut out = ctx.run_ui(Default::default(), |_| {});
            out.textures_delta.clear();
            let id = egui::FontId::proportional(12.0);
            ctx.fonts_mut(|f| {
                f.layout_no_wrap(text.into(), id, egui::Color32::WHITE)
                    .rect
                    .width()
            })
        };
        let (zero, one) = (width(Some("geist"), "0"), width(Some("geist"), "1"));
        assert_eq!((zero.fract(), one.fract()), (0.0, 0.0), "{zero} {one}");
        assert!(one < zero, "Geist's 1 is narrower than its 0: {one} {zero}");
        assert_eq!(width(None, "0"), width(None, "1"), "Segoe UI's figures");
    }

    /// Plan 04, step 5: a theme switch remakes the fonts only when the font changes.
    #[test]
    fn fonts_are_remade_only_when_the_font_changes() {
        use super::needs_reload;
        let (system, geist) = (None::<String>, Some("geist".to_owned()));
        assert!(needs_reload(None, None), "not made yet");
        assert!(!needs_reload(Some(&system), None), "Classic to Metro");
        assert!(needs_reload(Some(&system), Some("geist")), "to Graphite");
        assert!(!needs_reload(Some(&geist), Some("geist")), "Graphite again");
        assert!(needs_reload(Some(&geist), None), "leaving Graphite");
    }

    /// Plan 04: from Classic to Graphite and back, the first Proportional font is Geist, then the
    /// system's again; DirectWrite draws Geist's bytes only while Geist leads.
    #[test]
    fn a_theme_switch_swaps_the_face() {
        let first = |ctx: &egui::Context| {
            let mut out = ctx.run_ui(Default::default(), |_| {});
            out.textures_delta.clear();
            ctx.fonts(|f| f.definitions().families[&egui::FontFamily::Proportional][0].clone())
        };
        let _one = super::font_tests();
        let themes = crate::palette::builtins();
        let (classic, graphite) = (themes[0], themes[8]);
        let ctx = egui::Context::default();
        // As `apply` at start, then `apply_theme` at each switch.
        super::load_font(&ctx, classic.font.as_deref());
        let system = first(&ctx);
        assert!(super::bundled_face().is_none());
        super::fonts_if_changed(&ctx, graphite.font.as_deref());
        assert_eq!(first(&ctx), "bundled");
        assert_eq!(super::bundled_face(), super::bundled_font("geist"));
        super::fonts_if_changed(&ctx, classic.font.as_deref());
        assert_eq!(first(&ctx), system);
        assert_ne!(system, "bundled");
        assert!(super::bundled_face().is_none());
    }
}
