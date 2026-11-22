//! The per-download progress window (plan 13): the download's state and data here; its layout,
//! the compact view and Details, in `progress_view.rs` (plan 13's "Upstream merge cost": upstream
//! fixes to the data side still apply here as they are). Coordinates are client points.

use egui::{Color32, Pos2, Ui, ViewportBuilder, ViewportId, vec2};
use mdm_engine::Snapshot;
use mdm_engine::download::ConnStatus;

use crate::model::{Item, Status};

// The layout, a child module so it reads this window's state as its own (crates/app/src/
// progress_view.rs).
#[path = "progress_view.rs"]
pub(crate) mod view;
// Laid out in English pixels: `fill` makes each one where it is on screen (a right-to-left
// interface mirrors it), `text` starts at an English pen.
use crate::winui::{self, fill_l as fill, r, text};

use crate::winui::caption::CaptionButton;

/// The drawn title bar's height (#63): the frame is ours (win32::progress_frame).
const TITLE_H: f32 = crate::winui::caption::CAPTION_H;

/// The frame applied to a progress window (Windows): its client size, pixels per point × 1000
/// and its system menu's "Hide to the notification area" (translated, when there is one).
type Framed = ([f32; 2], u32, Option<String>);

/// The window's frames while its bar glides ([`Glide`]): as often as the bar moves a pixel, 25 a
/// second at most, only while its download runs and its figures move ([`glide_frame`]).
const GLIDE_FRAME: std::time::Duration = std::time::Duration::from_millis(40);
/// The same while the main window is shown: each frame draws it again too, about three times a
/// progress window's work (measured: 25 a second then took 5 % of a core more, 12.5 half that).
const GLIDE_FRAME_MAIN: std::time::Duration = std::time::Duration::from_millis(80);
/// The app's own frames while a download runs (app.rs): a bar slower than a pixel in this long
/// needs no frames of its own.
const APP_FRAME: f64 = 0.25;
/// The strip's width ([`view::STRIP_W`]).
const BAR_WIDTH: f64 = view::STRIP_W as f64;
/// A figure reached in about the time the last one took to come, a quarter more (seconds): the
/// next one comes before the bar has stopped, so it never halts between figures that keep
/// coming. Within these bounds (a figure after a long pause glides in half a second).
const GLIDE_SLACK: f64 = 1.25;
const GLIDE_MIN: f64 = 0.05;
const GLIDE_MAX: f64 = 0.5;
/// Figures that moved this recently (seconds) keep the frames coming, so the next one is drawn
/// as it comes, not at the app's next quarter second.
const GLIDE_HOLD: f64 = 1.0;
/// How long a finished download's window shows it whole before it gives way to Download
/// complete (app.rs): its bar, a glide behind the figures, would otherwise close short of the
/// end.
pub const SHOWN_WHOLE: std::time::Duration = std::time::Duration::from_millis(300);

/// The bar's fill and the segment map between the download's figures, which come about four
/// times a second: each part moves from where it was drawn to its new figure over about the
/// time between figures, so the bar glides where it jumped (the text keeps the figures as they
/// are). Never past a figure, never back: a figure lower than what is drawn (a download started
/// over) is drawn at once. The bar and the map move on the same clock, so they agree as their
/// figures do.
#[derive(Default)]
struct Glide {
    /// The figures last read: bytes done, each segment's start and bytes done.
    done: u64,
    segments: Vec<(u64, u64)>,
    /// Where the drawing set off from towards them (the same segments, in order).
    from_done: f64,
    from_segments: Vec<f64>,
    /// When they were read (egui's clock, seconds) and how long it takes to reach them.
    at: f64,
    span: f64,
    seen: bool,
    /// The bar's speed on its way to them (bytes a second).
    rate: f64,
}

/// What a frame draws ([`Glide::step`]).
struct Drawn {
    done: f64,
    segments: Vec<(u64, f64)>,
    /// How fast the bar moves (bytes a second), while it is on its way to the figures or they
    /// moved lately; else 0: no frames of its own wanted.
    rate: f64,
}

/// When the progress window wants its next frame while its bar moves at `rate` bytes a second
/// of `total`, on a screen of `ppp` pixels a point: when the bar has moved a pixel, 25 frames a
/// second at most (12.5 while the main window is shown, `main`); none when it moves less than a
/// pixel between the app's own frames (a slow download, an unknown size, a bar standing still).
fn glide_frame(rate: f64, total: Option<u64>, ppp: f32, main: bool) -> Option<std::time::Duration> {
    let total = total.filter(|t| *t > 0)? as f64;
    let pixels = rate / total * BAR_WIDTH * f64::from(ppp);
    let every = 1.0 / pixels;
    let cap = if main { GLIDE_FRAME_MAIN } else { GLIDE_FRAME };
    (pixels > 0.0 && every < APP_FRAME).then(|| std::time::Duration::from_secs_f64(every).max(cap))
}

impl Glide {
    /// How far the drawing is from `from` to the figures at `now` (0 to 1).
    fn part(&self, now: f64) -> f64 {
        if self.span <= 0.0 {
            1.0
        } else {
            ((now - self.at) / self.span).clamp(0.0, 1.0)
        }
    }

    /// The figures read at `now` (`done` bytes; the segments' starts and bytes done): what to
    /// draw. `finished`: the download is done, its figures drawn as they are at once (the
    /// window shows it whole before it gives way to Download complete, not a glide behind).
    fn step(&mut self, now: f64, done: u64, segments: &[(u64, u64)], finished: bool) -> Drawn {
        if !self.seen || finished {
            self.seen = true;
            self.at = now;
            self.span = 0.0;
            self.rate = 0.0;
            self.from_done = done as f64;
            self.from_segments = segments.iter().map(|&(_, d)| d as f64).collect();
            self.done = done;
            self.segments = segments.to_vec();
        } else if done != self.done || segments != self.segments.as_slice() {
            // Off from where it is drawn now; a part whose figure went below that, at once.
            let u = self.part(now);
            let at = |from: f64, to: u64| from + (to as f64 - from) * u;
            let drawn = at(self.from_done, self.done);
            let drawn_segments: Vec<(u64, f64)> = self
                .segments
                .iter()
                .zip(&self.from_segments)
                .map(|(&(start, d), &from)| (start, at(from, d)))
                .collect();
            // A segment that comes (a connection split one) grows from nothing, as the bar's
            // part of it does: the map's segments together stay what the bar shows.
            self.from_segments = segments
                .iter()
                .map(|&(start, d)| {
                    drawn_segments
                        .iter()
                        .find(|(s, _)| *s == start)
                        .map_or(0.0, |&(_, x)| x.min(d as f64))
                })
                .collect();
            let done_f = done as f64;
            self.from_done = drawn.min(done_f);
            self.span = ((now - self.at) * GLIDE_SLACK).clamp(GLIDE_MIN, GLIDE_MAX);
            self.rate = (done_f - self.from_done) / self.span;
            self.at = now;
            self.done = done;
            self.segments = segments.to_vec();
        }
        let u = self.part(now);
        let at = |from: f64, to: u64| from + (to as f64 - from) * u;
        let busy = u < 1.0 || now - self.at < GLIDE_HOLD;
        Drawn {
            done: at(self.from_done, self.done),
            segments: self
                .segments
                .iter()
                .zip(&self.from_segments)
                .map(|(&(start, d), &from)| (start, at(from, d)))
                .collect(),
            rate: if busy { self.rate } else { 0.0 },
        }
    }
}

/// The window's title: IDM truncates the percentage in it (44.51 % → "44%"); without a size to
/// take a share of, only the name. The name keeps its own direction (an Arabic one doesn't pull
/// the percentage after it).
fn window_title(frac: Option<f64>, name: &str) -> String {
    let name = crate::i18n::isolate(name);
    match frac {
        Some(f) => format!("{}% {name}", (f * 100.0).floor() as u32),
        None => name,
    }
}

/// The title bar's buttons, from the window's end: Close, Maximize (greyed: a fixed size, as
/// IDM's), Minimize, and with a tray our "hide to the notification area".
fn caption_buttons(tray: bool) -> Vec<(CaptionButton, bool)> {
    let mut b = vec![
        (CaptionButton::Close, true),
        (CaptionButton::Maximize, false),
        (CaptionButton::Minimize, true),
    ];
    if tray {
        b.push((CaptionButton::ToTray, true));
    }
    b
}

/// The title bar, `width` points wide at `o`; the button pressed. Where
/// its buttons and icon are goes to the window's hit test (win32::set_caption_hits), in client
/// pixels, each frame (`hwnd`: the window, once its frame is applied).
fn draw_caption(
    ui: &mut Ui,
    o: Pos2,
    width: f32,
    title: &str,
    buttons: &[(CaptionButton, bool)],
    hwnd: Option<isize>,
) -> Option<CaptionButton> {
    // The one drawn title bar of every window (dialogs.rs), with its hit test.
    crate::dialogs::drawn_title(ui, o, width, title, buttons, hwnd)
}

/// A greyed label (the limits' captions while off).
fn disabled() -> Color32 {
    crate::theme::colors().progress_label_disabled
}

impl Progress {
    /// `/show progress:<name>` (the docs' screenshots and the fit audit): "status" the compact
    /// view as the download is, "details" the expanded one, "files", "peers", "trackers" a
    /// torrent's tab in it; any other name a state, each a look only (plan 09a, 13b): drawn for
    /// the screenshot, the download itself left as it is. "error" a failed download (a sample
    /// reason), "mergefail" a failed join's reason; for a video (upstream issues 769, 736)
    /// "merging" its picture and sound being joined (45 %), "helper" Details' "Downloaded by"
    /// row saying why yt-dlp downloads it itself (an RTMP stream; shown for a video only, so
    /// ask it of a video's window); "recording" and "saving" a live stream being
    /// recorded (its button "Stop and save") and being saved; and [`view::Look`]'s ("stopped",
    /// "unknown-size", "waiting", "finishing", "checking", "sharing", "webpage", "service",
    /// "cant-resume"). Unknown names leave it as it is.
    pub fn show_tab(&mut self, name: &str) {
        let look = view::Look::named(name);
        // One sample state at a time: each one asked for replaces the one before.
        if look.is_some()
            || matches!(
                name,
                "error" | "mergefail" | "merging" | "helper" | "recording" | "saving" | "status"
            )
        {
            self.clear_samples();
        }
        if let Some(look) = look {
            // The site's page: the reason a failed download gives for it.
            if look == view::Look::WebPage {
                self.sample_error = Some(crate::errors::describe(&mdm_engine::Error::GotWebPage));
            }
            self.sample_look = Some(look);
            return;
        }
        match name {
            "recording" | "saving" => {
                self.sample_live = Some(if name == "recording" {
                    mdm_tools::video::LiveState::Recording
                } else {
                    mdm_tools::video::LiveState::Saving
                });
                self.sample_helper = Some(mdm_tools::streams::Helper::Live);
            }
            "error" => {
                self.sample_error = Some(crate::errors::describe(&mdm_engine::Error::Status(
                    reqwest::StatusCode::NOT_FOUND,
                )));
            }
            "mergefail" => {
                self.sample_error = Some(crate::model::merge_failure(
                    "Postprocessing: Conversion failed!",
                    &[
                        std::path::PathBuf::from("open-movie-trailer.f137.mp4"),
                        std::path::PathBuf::from("open-movie-trailer.f140.m4a"),
                    ],
                ));
            }
            "merging" => self.sample_merging = Some(0.45),
            "helper" => {
                self.sample_helper = Some(mdm_tools::streams::Helper::Protocol("rtmp".into()));
                // Its row is in Details.
                self.details = true;
            }
            // The download as it is, compact.
            "status" => self.details = false,
            "details" => self.details = true,
            "files" | "peers" | "trackers" => {
                self.ttab = match name {
                    "files" => TTab::Files,
                    "peers" => TTab::Peers,
                    _ => TTab::Trackers,
                };
                self.details = true;
            }
            _ => {}
        }
    }

    /// Every screenshot's state off: the download drawn as it is.
    fn clear_samples(&mut self) {
        self.sample_error = None;
        self.sample_merging = None;
        self.sample_helper = None;
        self.sample_live = None;
        self.sample_look = None;
    }
}

/// A torrent's tabs in Details' right column (M15, D-014; plan 13b drops Pieces and General:
/// the strip is the piece map).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TTab {
    Files,
    Peers,
    Trackers,
}

/// A torrent's own commands in Details (plan 13b), the row menu's Torrent submenu's: the app runs
/// the same `Cmd::Torrent*` ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TorrentCmd {
    AddTrackers,
    AskTrackersAgain,
    Recheck,
    InOrder,
    EndsFirst,
    CopyMagnet,
}

pub struct Progress {
    pub id: u64,
    /// Details shown: the window widened to the right (plan 13b), else the compact view.
    details: bool,
    ttab: TTab,
    limiter: bool,
    /// KB/s typed in Details' Limit row.
    limit_kb: String,
    /// The Limit row has been initialised from the download's current limit.
    limit_init: bool,
    /// A limit change to report to the app (`Action::Limit`), and whether to remember it.
    pending_limit: Option<(Option<u64>, bool)>,
    remember: bool,
    /// A torrent's own upload limit in the Limit row: on, and its KB/s box.
    up_on: bool,
    up_kb: String,
    /// An upload limit change to report (`Action::UploadLimit`).
    pending_upload: Option<Option<u64>>,
    /// Files ticked or unticked in a torrent's Files tab, to report (`Action::Choose`).
    pending_choose: Option<Action>,
    /// When done's choice, an index into [`view::WHEN_DONE`] ([`view::when_done_of`]).
    when: usize,
    /// With Shut down, Sleep or Hibernate: "Force apps to close".
    force: bool,
    /// Stop was pressed on a download whose server can't resume: the window asks first (Keep
    /// downloading, Stop anyway).
    confirm_stop: bool,
    /// Torrent settings' "Keep sharing torrents after they finish" (When done says "Then keeps
    /// sharing").
    pub keep_sharing: bool,
    /// The queue it waits in, by name (set by the app each frame): "Waiting in Main queue".
    pub queue: Option<String>,
    /// The torrent session (the app's), for a finished torrent still shared without a `Run`:
    /// its tabs read the session directly.
    pub host: Option<crate::torrents::Host>,
    /// Hidden in the notification area (#63): not shown, its tab and choices kept for when it
    /// comes back (the tray's "Downloads in progress", or the download double-clicked).
    pub hidden: bool,
    /// A tray shows MDM's icon, so the window can be hidden there (set by the app each frame).
    pub tray: bool,
    /// Options → General → "Minimize to the notification area": Minimize hides it there too.
    pub minimize_to_tray: bool,
    /// The main window is shown (set by the app each frame): the glide's frames draw it too.
    pub main_shown: bool,
    /// The speed limit holding the download back (set by the app each frame, upstream issue
    /// 1191): the detail line names it after the speed, the speed graph draws it dashed.
    pub limit: Option<crate::model::Limit>,
    /// The bar and the segment map between figures.
    glide: Glide,
    /// The last minute of its speed for Details' graph (plan 13c): kept while the window is
    /// open, gone with it, never saved.
    speed: view::Sampler,
    /// `/show progress:error`: the window drawn as failed with this reason, the download left
    /// as it is (a screenshot's state).
    sample_error: Option<String>,
    /// `/show progress:merging`: the status word and the strip as while a video's streams are
    /// joined, this far (a screenshot's state).
    sample_merging: Option<f64>,
    /// `/show progress:helper`: Details' "Downloaded by" row as for a video yt-dlp downloads
    /// itself, for this reason (a screenshot's state; a video's row only).
    sample_helper: Option<mdm_tools::streams::Helper>,
    /// `/show progress:stopped` and the other states of plan 13b (a screenshot's state).
    sample_look: Option<view::Look>,
    /// `/show progress:recording` / `progress:saving`: the status word and the Stop and save
    /// button as for a live stream being recorded, or saved (a screenshot's state).
    sample_live: Option<mdm_tools::video::LiveState>,
}

pub enum Action {
    None,
    Pause,
    Resume,
    /// Stop the download and close the dialog.
    Cancel,
    /// Set this download's speed limit (bytes/s, None = off).
    /// This download's limit (None: unlimited); `true`: remember it for the file.
    Limit(Option<u64>, bool),
    /// A torrent's own upload limit (bytes/s, None: off), kept with it (`TorrentSpec`).
    UploadLimit(Option<u64>),
    /// A torrent's files to download from now on (#39, its Files tab): indexes into its list,
    /// whether that is every file, and their size together.
    Choose {
        files: Vec<usize>,
        all: bool,
        size: u64,
    },
    /// Close the dialog, keep downloading.
    Close,
    /// Hide the dialog in the notification area, keep downloading (#63).
    Hide,
    /// Refresh download address... on an expired link (plan 12's E2): the row menu's command.
    RefreshAddress,
    /// The From row's copy button: the download's address (a torrent's: `Torrent(CopyMagnet)`).
    CopyAddress,
    /// Saved to's folder button: the download in its folder.
    OpenFolder,
    /// A torrent's command from Details (plan 13b).
    Torrent(TorrentCmd),
}

impl Progress {
    /// `show_complete`: Options' "Show download complete dialog", this file's first state.
    pub fn new(id: u64, show_complete: bool) -> Self {
        Progress {
            id,
            details: false,
            ttab: TTab::Files,
            limiter: false,
            limit_kb: "10".into(),
            limit_init: false,
            pending_limit: None,
            remember: false,
            up_on: false,
            up_kb: "50".into(),
            pending_upload: None,
            pending_choose: None,
            when: view::when_done_index(show_complete),
            force: false,
            confirm_stop: false,
            keep_sharing: false,
            queue: None,
            host: None,
            hidden: false,
            tray: false,
            minimize_to_tray: false,
            main_shown: true,
            limit: None,
            glide: Glide::default(),
            speed: view::Sampler::default(),
            sample_error: None,
            sample_merging: None,
            sample_helper: None,
            sample_look: None,
            sample_live: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, item: &Item, snap: Option<&Snapshot>) -> Action {
        crate::crash::window("progress");
        let frac = item.fraction();
        let title = window_title(frac, &item.file_name);
        // The bar and the segment map glide between the figures (the text shows them as they
        // are): frames come faster for that only while the download runs and its figures move.
        let segments: Vec<(u64, u64)> = match snap {
            Some(s) if !s.segments.is_empty() => s
                .segments
                .iter()
                .map(|g| (g.start, g.cursor.saturating_sub(g.start)))
                .collect(),
            _ => vec![(0, item.downloaded)],
        };
        let drawn = self.glide.step(
            ctx.input(|i| i.time),
            item.downloaded,
            &segments,
            item.status == Status::Complete,
        );
        // Never past the fraction: a video's bar holds short of the end until it is finished,
        // as its percentage does ([`Item::fraction`]); no size, no fill.
        let bar = match (frac, item.size.filter(|t| *t > 0)) {
            (Some(f), Some(t)) => (drawn.done / t as f64).clamp(0.0, 1.0).min(f),
            _ => 0.0,
        };
        // The speed graph's reading, one a second (0 while it doesn't download).
        let rate = if item.status == Status::Downloading {
            item.rate()
        } else {
            0.0
        };
        self.speed.feed(ctx.input(|i| i.time), rate);
        // The torrent's details, and the state as drawn: its line takes room in Details.
        let d = if item.torrent.is_some() {
            self.torrent_data(item)
        } else {
            None
        };
        let state = view::shown(self, item, snap, d.as_ref());
        let body = view::size(self.details, view::Kind::of(item), state.line.is_some());
        // Windows: the title bar is ours (#63), drawn in the client above IDM's layout, so the
        // hide button sits in it, left of Minimize, as in IDM. Elsewhere the system draws it.
        let client = [body[0], body[1] + TITLE_H];
        // As large as the screen allows at the interface size (#262): the rest scrolls.
        let shown = crate::dialogs::shown_size(ctx, client);
        let vid = ViewportId::from_hash_of(("progress", self.id));
        let buttons = caption_buttons(self.tray);
        let key = egui::Id::unique(("progress-pos", self.id));
        let title_now = title.clone();
        // Hidden until dialogs::keep_theme has given it its theme.
        let mut builder = crate::quiet(ViewportBuilder::default())
            .with_visible(false)
            // The taskbar shows it: without isolates (upstream issue 919).
            .with_title(crate::i18n::os_text(&title))
            .with_inner_size(shown)
            // IDM's progress window has 2 px borders and a fixed size. The sizing frame is asked
            // for and win32::progress_frame trims its borders, disables resizing and gives the
            // caption to the client, which draws it.
            .with_resizable(true)
            .with_icon(crate::app_icon())
            .with_maximize_button(false);
        if let Some(p) = crate::dialogs::place(ctx, key, client) {
            builder = builder.with_position(p);
        }
        // Minimize hides it in the notification area when asked (and a tray can bring it back).
        let minimize_hides = self.tray && self.minimize_to_tray;
        // The window, once its frame is applied (Windows: its caption parts go to its hit test).
        let hwnd = ctx.data(|d| d.get_temp::<isize>(key.with("hwnd")));
        let action = ctx.show_viewport_immediate(vid, builder, |ui, _| {
            // Painted in the pass that closes it too (see dialogs::close_requested).
            let close = crate::dialogs::close_requested(ui);
            if !close && minimize_hides && ui.ctx().input(|i| i.viewport().minimized == Some(true))
            {
                return Action::Hide;
            }
            let action = egui::CentralPanel::no_frame()
                .frame(egui::Frame::new().fill(winui::BG()))
                .show(ui, |ui| {
                    let o = ui.max_rect().min;
                    // The window's width, laid out from the right in a right-to-left
                    // interface (the title bar; the layout under it has its own).
                    let _surface = winui::Surface::enter(o.x, o.x + shown[0]);
                    let caption = draw_caption(ui, o, shown[0], &title_now, &buttons, hwnd);
                    // The layout under the title bar, scrolled when the window is smaller.
                    let below = egui::Rect::from_min_max(o + vec2(0.0, TITLE_H), ui.max_rect().max);
                    let mut rest = ui.new_child(egui::UiBuilder::new().max_rect(below));
                    let action = crate::dialogs::scroll(&mut rest, key, body, |ui, o| {
                        let _surface = winui::Surface::enter(o.x, o.x + body[0]);
                        let drawn = (bar, drawn.segments.as_slice());
                        self.body(ui, o, item, snap, drawn, &state, d.as_ref())
                    });
                    // The system menu's "Hide to the notification area" (Alt+Space).
                    if let Some(h) = hwnd
                        && crate::win32::take_hide_asked(h)
                    {
                        return Action::Hide;
                    }
                    match caption {
                        Some(CaptionButton::Close) => Action::Close,
                        Some(CaptionButton::ToTray) => Action::Hide,
                        Some(CaptionButton::Minimize) if minimize_hides => Action::Hide,
                        Some(CaptionButton::Minimize) => {
                            crate::dialogs::minimize(ui.ctx(), hwnd);
                            action
                        }
                        _ => action,
                    }
                })
                .inner;
            if close {
                return Action::Close;
            }
            on_escape(action, winui::escape_pressed(ui))
        });
        // Elsewhere the window manager is asked for its size at a new interface size (#262).
        crate::dialogs::follow_scale(ctx, vid, key, shown);
        // The glide's frames: while the download runs, its figures move and the window can be
        // seen (minimized, or hidden in the tray where it isn't drawn at all, it waits).
        let minimized = ctx.input_for(vid, |i| i.viewport().minimized) == Some(true);
        let ppp = ctx.input_for(vid, |i| i.pixels_per_point);
        if item.live.is_some()
            && !minimized
            && let Some(every) = glide_frame(drawn.rate, item.size, ppp, self.main_shown)
        {
            ctx.request_repaint_after(every);
        }
        // Apply the frame while the window is still hidden, and again whenever the client size
        // (details), the scale (another display, the interface size) or the system menu's item
        // changes.
        {
            let scale = ctx.input_for(vid, |i| i.pixels_per_point);
            let hide = buttons
                .iter()
                .any(|(b, _)| *b == CaptionButton::ToTray)
                .then(|| crate::i18n::t("Hide to the notification area").to_owned());
            let want: Framed = (shown, (scale * 1000.0).round() as u32, hide);
            let framed = ctx.data(|d| d.get_temp::<Framed>(key.with("framed")));
            if framed.as_ref() != Some(&want)
                && let Some(h) =
                    crate::win32::thin_frame(&title_now, shown, scale, want.2.as_deref())
            {
                // Widened (Details, plan 13b): it grows to the right, and moves left when that
                // would take it past its screen's work area.
                if framed.is_some_and(|f| f.0[0] < shown[0]) {
                    view::keep_on_screen(&title_now);
                }
                ctx.data_mut(|d| {
                    d.insert_temp(key.with("framed"), want);
                    d.insert_temp(key.with("hwnd"), h);
                });
            }
        }
        let closing = matches!(action, Action::Close | Action::Cancel | Action::Hide);
        crate::dialogs::keep_theme(
            ctx,
            vid,
            key,
            &title_now,
            crate::theme::dark_surface(),
            !closing,
        );
        let action = match (action, self.pending_limit.take()) {
            (Action::None, Some((limit, remember))) => Action::Limit(limit, remember),
            (a, _) => a,
        };
        // An upload change waits a frame when a download one goes first (one action a frame);
        // so does a change of files.
        let action = match action {
            Action::None => self
                .pending_upload
                .take()
                .map_or(Action::None, Action::UploadLimit),
            a => a,
        };
        let action = match action {
            Action::None => self.pending_choose.take().unwrap_or(Action::None),
            a => a,
        };
        if matches!(action, Action::Close | Action::Cancel) {
            crate::dialogs::forget_place(ctx, key);
        }
        // Gone (hidden keeps its place): the next window gets its thin frame again.
        if closing {
            if let Some(h) = hwnd {
                crate::win32::forget_caption(h);
            }
            ctx.data_mut(|d| {
                d.remove::<Framed>(key.with("framed"));
                d.remove::<isize>(key.with("hwnd"));
            });
        }
        action
    }

    /// `drawn`: the bar's fill and the segments as they glide ([`Glide`]); `state`: as
    /// [`view::shown`] has it (the figures as they are); `d`: a torrent's details.
    #[allow(clippy::too_many_arguments)]
    fn body(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        item: &Item,
        snap: Option<&Snapshot>,
        drawn: (f64, &[(u64, f64)]),
        state: &view::Shown,
        d: Option<&crate::torrents::Details>,
    ) -> Action {
        // The layout (progress_view.rs); this window's state and data stay here.
        view::body(self, ui, o, item, snap, drawn, state, d)
    }

    /// The torrent's details as the session has them (its files, peers, trackers, pieces,
    /// numbers): from its `Run`, or for a finished torrent still shared without one, from the
    /// session all the same.
    fn torrent_data(&self, item: &Item) -> Option<crate::torrents::Details> {
        match &item.live {
            Some(crate::model::Live {
                dl: crate::model::Job::Torrent(run),
                ..
            }) => Some(run.details()),
            // Finished and still shared (no Run): the session has it all the same, so the
            // tabs stay.
            _ => self
                .host
                .as_ref()
                .and_then(|h| h.get())
                .zip(item.torrent.as_ref())
                .and_then(|(t, spec)| crate::torrents::details_of(&t, &spec.hash)),
        }
    }
}

impl Progress {
    /// Details' Limit row from the download's limits as they are, once: its own (a remembered
    /// one ticks Remember for this file), and a torrent's upload limit.
    fn init_limits(&mut self, item: &Item) {
        if !self.limit_init {
            if let Some(rate) = item.torrent.as_ref().and_then(|t| t.upload_limit) {
                self.up_on = true;
                self.up_kb = (rate / 1024).max(1).to_string();
            }
            if let Some(rate) = item.live.as_ref().and_then(|l| l.dl.speed_limit()) {
                self.limiter = true;
                self.limit_kb = (rate / 1024).max(1).to_string();
            }
            // A remembered limit: the box is ticked (and the limit shows even while stopped).
            if let Some(rate) = item.own_limit {
                self.limiter = true;
                self.limit_kb = (rate / 1024).max(1).to_string();
                self.remember = true;
            }
            self.limit_init = true;
        }
    }

    /// When done → "Show the Done window" for this file.
    pub fn shows_complete(&self) -> bool {
        view::when_done_of(self.when, self.force).0
    }

    /// When done's other choices, when one is chosen (Exit Grabbit, Shut down, Sleep, Hibernate).
    pub fn when_done(&self) -> Option<crate::queue::WhenDone> {
        view::when_done_of(self.when, self.force).1
    }
}

/// Escape (`escape`) hides the window as the × does, and the download goes on: Close, never
/// Cancel or Pause. Nothing in the row answers it (plan 07: the row's look changed, not its
/// keys); a button pressed in the same frame wins.
fn on_escape(action: Action, escape: bool) -> Action {
    if matches!(action, Action::None) && escape {
        Action::Close
    } else {
        action
    }
}

/// A screenshot's states for the status word (`/show progress:<state>`, looks only): failed for
/// this reason, a video's streams joined this far, a live stream being recorded or saved; plan
/// 13b's states as their status word says them (`word`).
#[derive(Clone, Copy, Default)]
struct Samples<'a> {
    failed_with: Option<&'a str>,
    merging: Option<f64>,
    live: Option<mdm_tools::video::LiveState>,
    word: Option<&'a str>,
}

/// The download's status word ("Receiving data...", "Stopped", "Error: {error}"): beside its
/// name and in its line (progress_view.rs). `samples`: a screenshot's state said instead of the
/// download's own.
fn status_word(item: &Item, snap: Option<&Snapshot>, samples: Samples) -> String {
    let failed_with = samples.failed_with;
    let state = if failed_with.is_some() {
        Status::Error
    } else {
        item.status
    };
    // A live stream being recorded or saved (or a screenshot's of one).
    let live = if failed_with.is_some() {
        None
    } else {
        samples.live.or_else(|| item.live_state())
    };
    match state {
        // A screenshot's state (plan 13b), as its status word says it.
        _ if let Some(word) = samples.word => word.to_owned(),
        // A live stream being recorded or saved shows no progress of its own: said first.
        _ if let Some(live) = live_status(live) => live.to_owned(),
        // A torrent reading and checking its files says so, not "Receiving data" (#39).
        _ if let Some(checking) = item.torrent_checking().filter(|_| failed_with.is_none()) => {
            checking
        }
        // After the last byte: the file checked, a video's streams joined (upstream issue 736),
        // as the list says, not "Connecting...".
        _ if let Some(f) = samples.merging.filter(|_| failed_with.is_none()) => {
            crate::i18n::fmt(
                // i18n: a video's status while its picture and sound are joined into one file (ffmpeg); {percent} how far, e.g. "45%"
                crate::i18n::t("Merging... ({percent})"),
                &[("percent", &crate::i18n::percent(f, 0))],
            )
        }
        _ if let Some(finishing) = item.finishing(false).filter(|_| failed_with.is_none()) => {
            finishing
        }
        // A torrent its debrid service is downloading (#44).
        Status::Downloading if item.at_service().is_some() => item.at_service().unwrap_or_default(),
        Status::Downloading => {
            // A video (yt-dlp) has no engine connections: data arriving is the sign.
            let receiving = snap.is_some_and(|s| {
                s.connections
                    .iter()
                    .any(|c| matches!(c.status, ConnStatus::Receiving))
            }) || ((item.video.is_some() || item.torrent.is_some())
                && item.rate() > 0.0);
            // Every byte in and none on its way (upstream issue 736): the last of it written
            // out, the file named; nothing is connecting.
            let completing = item.torrent.is_none()
                && snap.is_some_and(|s| {
                    s.state == mdm_engine::download::State::Completing
                        || (s.size.is_some_and(|t| t > 0 && s.downloaded >= t)
                            && !s
                                .connections
                                .iter()
                                .any(|c| matches!(c.status, ConnStatus::Connecting)))
                });
            if completing {
                // i18n: a download's status: every byte is in, the file is being written out and given its name
                crate::i18n::t("Finishing...")
            } else if receiving {
                crate::i18n::t("Receiving data...")
            } else {
                crate::i18n::t("Connecting...")
            }
            .to_owned()
        }
        // i18n: a download's status (state it is in, not a command): finished
        Status::Complete => crate::i18n::t("Complete").to_owned(),
        // i18n: a download's status (state it is in, not a command): stopped for now (plan 12's A4: the list's word)
        Status::Stopped => crate::i18n::t("Stopped").to_owned(),
        // i18n: a download's status (state it is in, not a command): waiting in a queue
        Status::Queued => crate::i18n::t("Waiting").to_owned(),
        Status::Error => crate::i18n::fmt(
            // i18n: a download's status: it failed, with the reason
            crate::i18n::t("Error: {error}"),
            &[(
                "error",
                // i18n: the reason a download failed is not known
                failed_with
                    .or(item.error.as_deref())
                    .unwrap_or(crate::i18n::t("unknown")),
            )],
        ),
    }
}

/// How much of the pieces under column `c` of `cols` are there (a torrent's strip, plan 13c: the
/// column lit that much), 0 to 1.
fn piece_column(pieces: &[bool], c: usize, cols: usize) -> f32 {
    if pieces.is_empty() || cols == 0 {
        return 0.0;
    }
    let a = c * pieces.len() / cols;
    let b = ((c + 1) * pieces.len() / cols).max(a + 1).min(pieces.len());
    pieces[a..b].iter().filter(|x| **x).count() as f32 / (b - a) as f32
}

/// The Files tab's table of a torrent's files, ticked or not: the Files window's (torrentfiles.rs,
/// `shown` rows, 513 wide). The row whose box was clicked.
pub(crate) fn files_table(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    id: &str,
    rows: &[Vec<String>],
    checks: &[bool],
    shown: usize,
) -> Option<usize> {
    files_table_w(ui, p, o, id, rows, checks, shown, FILES_WINDOW_TABLE_W)
}

/// [`files_table`] `w` wide: narrower than the Files window's, the Size and Done columns keep to
/// the right (the progress window's Details, plan 13b).
#[allow(clippy::too_many_arguments)]
fn files_table_w(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    id: &str,
    rows: &[Vec<String>],
    checks: &[bool],
    shown: usize,
    w: f32,
) -> Option<usize> {
    let right = 17.0 + w;
    let (size_x, done_x) = if w >= FILES_WINDOW_TABLE_W {
        (335.0, 430.0)
    } else {
        (right - 125.0, right - 55.0)
    };
    table_sized(
        ui,
        p,
        o,
        id,
        &[
            (25.0, crate::i18n::mark("File")),
            (size_x, crate::i18n::mark("Size")),
            (done_x, crate::i18n::mark("Done")),
        ],
        &[size_x - 5.0, done_x - 5.0],
        rows,
        true,
        Some(checks),
        shown,
        w,
    )
}

/// The Files window's table width (and the progress window's before plan 13b): x 17..530.
const FILES_WINDOW_TABLE_W: f32 = 513.0;

/// The height of a [`table_sized`] of `shown` rows' body (the progress window's 6: 100 px).
pub(crate) fn table_body_height(shown: usize) -> f32 {
    17.0 * shown as f32 - 2.0
}

/// A table in the Files window's frame (torrentfiles.rs), `w` wide from x 17, showing `shown`
/// rows: headers at x, column lines at `seps`, rows of text cut to their column (whole on hover),
/// scrolled by the wheel. `checks`: a check box before each row's icon, ticked or not; the row
/// whose box was clicked.
#[allow(clippy::too_many_arguments)]
fn table_sized(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    id: &str,
    heads: &[(f32, &str)],
    seps: &[f32],
    rows: &[Vec<String>],
    icons: bool,
    checks: Option<&[bool]>,
    shown: usize,
    w: f32,
) -> Option<usize> {
    let mut clicked = None;
    let body_h = table_body_height(shown);
    // Its right edge (530 for the Files window's 513).
    let re = 17.0 + w;
    let outer = r(o, 17.0, 273.0, w, body_h + 28.0);
    winui::solid(p, outer, winui::FIELD_EDGE());
    let (bg, head, line) = (
        winui::FIELD(),
        winui::LIST_HEADER(),
        crate::theme::colors().header_sep,
    );
    fill(
        p,
        o.x + 18.0,
        o.y + 274.0,
        o.x + re - 2.0,
        o.y + 299.0 + body_h,
        bg,
    );
    fill(
        p,
        o.x + 19.0,
        o.y + 275.0,
        o.x + re - 3.0,
        o.y + 298.0,
        head,
    );
    for x in seps {
        fill(p, o.x + x, o.y + 275.0, o.x + x, o.y + 298.0, line);
    }
    for (x, h) in heads {
        text(p, o.x + x, o.y + 290.0, crate::i18n::t(h), winui::TEXT());
    }
    let visible = shown;
    // Per window: each progress window scrolls its own tables.
    let key = egui::Id::unique(("torrent-table", id, ui.ctx().viewport_id()));
    let body = r(o, 19.0, 299.0, w - 5.0, body_h);
    let first = first_row(ui, key, body, rows.len(), visible);
    let clip = p.with_clip_rect(body);
    let right_of = |c: usize| seps.get(c).map_or(o.x + re - 10.0, |x| o.x + x - 4.0);
    for (k, row) in rows.iter().skip(first).take(visible + 1).enumerate() {
        let top = o.y + 299.0 + 17.0 * k as f32;
        for (c, cell) in row.iter().enumerate() {
            let mut x = heads.get(c).map_or(o.x + 25.0, |h| o.x + h.0);
            // A table of files (the Files tab): each name after its type's icon, and its
            // check box first when files can be chosen (#39). Only rows wholly shown take a
            // click (the one cut off under the frame doesn't).
            if icons
                && c == 0
                && let Some(&ticked) = checks.and_then(|t| t.get(first + k))
            {
                let mut on = ticked;
                let at = egui::pos2(x - 4.0, top + 2.0);
                if k < visible && winui::row_check(ui, &clip, at, &mut on, cell, (id, first + k)) {
                    clicked = Some(first + k);
                }
                x += 17.0;
            }
            if icons && c == 0 {
                crate::fileicons::paint_at(
                    ui,
                    &clip,
                    crate::fileicons::Kind::File(cell),
                    egui::pos2(x - 4.0, top),
                    16.0,
                );
                x += crate::fileicons::INDENT - 4.0;
            }
            let (rect, cut) =
                winui::text_fit(&clip, x, top + 12.0, cell, winui::TEXT(), right_of(c));
            // Cells hold info hashes, folders, trackers, peers: copyable (whole on hover).
            winui::copyable(ui, rect.intersect(body), (id, first + k, c), cell, cut);
        }
    }
    let track = r(o, re - 19.0, 275.0, 17.0, body_h + 24.0);
    rows_bar(ui, key, track, rows.len(), visible, first);
    clicked
}

/// The first row a table of `rows` rows showing `visible` at once can start at: the last ones
/// fill its body (#1101: 16 connections in 6 rows scroll to the 11th, 32 to the 27th).
fn last_first_row(rows: usize, visible: usize) -> usize {
    rows.saturating_sub(visible)
}

/// The first row a table (`key`, the window's own) shows: where it was left, moved by the wheel
/// over its `body`, never past the last rows. The header stays where it is: only the rows below
/// it scroll.
fn first_row(ui: &Ui, key: egui::Id, body: egui::Rect, rows: usize, visible: usize) -> usize {
    let last = last_first_row(rows, visible);
    let mut first = ui
        .ctx()
        .data(|d| d.get_temp::<usize>(key))
        .unwrap_or(0)
        .min(last);
    if ui.rect_contains_pointer(body) {
        let step = winui::wheel_steps(ui, key, body.height() / visible as f32, visible as f32);
        first = first.saturating_add_signed(step).min(last);
        ui.ctx().data_mut(|d| d.insert_temp(key, first));
    }
    first
}

/// A table's scroll bar over `track` (the main list's, `scrollbar::bar`) when its rows don't
/// all fit: dragged, clicked or its arrows pressed, it moves the table's first row
/// ([`first_row`]).
fn rows_bar(
    ui: &mut Ui,
    key: egui::Id,
    track: egui::Rect,
    rows: usize,
    visible: usize,
    first: usize,
) {
    if rows <= visible {
        return;
    }
    let mut pos = first as f32;
    if crate::scrollbar::bar(
        ui,
        key.with("bar"),
        track,
        true,
        rows as f32,
        visible as f32,
        1.0,
        &mut pos,
    ) {
        let first = (pos.round().max(0.0) as usize).min(last_first_row(rows, visible));
        ui.ctx().data_mut(|d| d.insert_temp(key, first));
    }
}

/// How far the check or the merge after the last byte has come (upstream issue 736), when it
/// is under way and that is known.
fn finishing_fraction(snap: Option<&Snapshot>) -> Option<f64> {
    use mdm_engine::download::State;
    let s = snap.filter(|s| matches!(s.state, State::Verifying | State::Merging))?;
    s.finishing.map(|f| f.clamp(0.0, 1.0))
}

/// A live stream's status (yt-dlp says no progress as it records): being saved, or recording;
/// before what any other download's says ("Receiving data...", "Connecting...").
fn live_status(live: Option<mdm_tools::video::LiveState>) -> Option<&'static str> {
    use mdm_tools::video::LiveState;
    match live? {
        // i18n: a live stream's status: stopped, and what was recorded being made into the video file kept
        LiveState::Saving => Some(crate::i18n::t("Saving...")),
        // i18n: a live stream's status: yt-dlp records it (it shows no progress, only the size so far)
        LiveState::Recording => Some(crate::i18n::t("Recording...")),
    }
}

/// Why yt-dlp downloads a video itself rather than the engine (upstream issue 769), as one
/// short line.
fn helper_line(why: &mdm_tools::streams::Helper) -> String {
    use mdm_tools::streams::Helper;
    match why {
        // i18n: progress window, a video's line: the engine can't take a live broadcast, so the helper program yt-dlp records it
        Helper::Live => crate::i18n::t("yt-dlp downloads it itself: a live stream").to_owned(),
        Helper::Pieces(_) => {
            // i18n: progress window, a video's line: a stream sent in many small pieces of a rare kind (Adobe HDS, Smooth Streaming), which the helper program yt-dlp downloads itself
            crate::i18n::t("yt-dlp downloads it itself: a stream in pieces").to_owned()
        }
        Helper::Protocol(protocol) => crate::i18n::fmt(
            // i18n: progress window, a video's line: {protocol} = a streaming protocol's short name (rtmp, rtsp, mms); the engine doesn't speak it, so the helper program yt-dlp downloads it
            crate::i18n::t("yt-dlp downloads it itself: {protocol}"),
            &[(
                "protocol",
                &crate::i18n::ltr(&protocol.to_ascii_uppercase()),
            )],
        ),
        Helper::Protected => {
            // i18n: progress window, a video's line: the stream is copy-protected (DRM), left to the helper program yt-dlp, which refuses it
            crate::i18n::t("yt-dlp downloads it itself: protected (DRM)").to_owned()
        }
        // i18n: progress window, a video's line: the helper program yt-dlp gave no address the engine could use
        Helper::NoAddress => crate::i18n::t("yt-dlp downloads it itself").to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #1101: 16 or 32 connections in the table's 6 rows scroll as far as their last; fewer
    /// than fit don't scroll.
    #[test]
    fn a_table_scrolls_to_its_last_rows() {
        assert_eq!(last_first_row(1, 6), 0);
        assert_eq!(last_first_row(6, 6), 0);
        assert_eq!(last_first_row(8, 6), 2);
        assert_eq!(last_first_row(16, 6), 10);
        assert_eq!(last_first_row(32, 6), 26);
        assert_eq!(last_first_row(0, 6), 0);
    }

    /// A download of unknown size has no percentage in its title (from #384).
    #[test]
    fn the_title_has_a_percentage_only_with_a_size() {
        assert!(!window_title(None, "clip.mp4").contains('%'));
        assert!(window_title(Some(0.445), "clip.mp4").starts_with("44% "));
    }

    /// Escape in the progress window hides it and leaves the download running (plan 07a): it
    /// is Close (keep downloading), never Cancel or Pause; a button pressed with it wins.
    #[test]
    fn escape_leaves_the_download_running() {
        assert!(matches!(on_escape(Action::None, true), Action::Close));
        assert!(matches!(on_escape(Action::None, false), Action::None));
        assert!(matches!(on_escape(Action::Pause, true), Action::Pause));
        assert!(matches!(on_escape(Action::Cancel, true), Action::Cancel));
    }

    /// A torrent's strip: a column is lit by how much of the pieces under it are there; more
    /// columns than pieces repeat a piece, none lit without pieces.
    #[test]
    fn piece_columns_bucket_the_pieces() {
        let pieces = [true, true, false, true];
        let lit: Vec<f32> = (0..2).map(|c| piece_column(&pieces, c, 2)).collect();
        assert_eq!(lit, [1.0, 0.5]);
        let lit: Vec<f32> = (0..8).map(|c| piece_column(&pieces, c, 8)).collect();
        assert_eq!(lit, [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]);
        assert_eq!(piece_column(&[], 0, 4), 0.0);
    }

    /// `/show progress:<state>` (plan 09a's and 13b's screenshots) are looks only: each a sample
    /// drawn over the download, which is left as it is; "status" shows it as it is again,
    /// compact, and the dropped names (menu, limit, done, pieces, general) change nothing.
    #[test]
    fn show_states_are_looks_only() {
        let mut p = Progress::new(1, true);
        p.show_tab("error");
        assert!(p.sample_error.as_deref().is_some_and(|e| e.contains("404")));
        for gone in ["menu", "limit", "done", "pieces", "general"] {
            p.show_tab(gone);
            assert!(
                p.sample_error.is_some() && !p.details,
                "{gone} does nothing"
            );
        }
        p.show_tab("details");
        assert!(
            p.details && p.sample_error.is_some(),
            "details keeps the state"
        );
        p.show_tab("status");
        assert!(
            p.sample_error.is_none() && !p.details,
            "status shows the download again, compact"
        );
        p.show_tab("peers");
        assert!(p.details && p.ttab == TTab::Peers);
        // Plan 13b's states: each one replaces the one before.
        p.show_tab("stopped");
        assert_eq!(p.sample_look, Some(view::Look::Stopped));
        p.show_tab("webpage");
        assert_eq!(p.sample_look, Some(view::Look::WebPage));
        assert!(
            p.sample_error.is_some(),
            "a web page is a failed download's reason"
        );
        p.show_tab("cant-resume");
        assert_eq!(p.sample_look, Some(view::Look::CantResume));
        assert!(p.sample_error.is_none());
        for name in [
            "unknown-size",
            "waiting",
            "finishing",
            "checking",
            "sharing",
            "service",
        ] {
            p.show_tab(name);
            assert!(p.sample_look.is_some(), "{name}");
        }
        p.show_tab("status");
        assert!(p.sample_look.is_none());
        // A video's states (upstream issues 769, 736): looks only, gone with "status".
        p.show_tab("merging");
        assert_eq!(p.sample_merging, Some(0.45));
        // Each one replaces the one before.
        p.show_tab("helper");
        assert!(p.sample_helper.is_some() && p.sample_merging.is_none());
        p.show_tab("mergefail");
        assert!(p.sample_helper.is_none());
        assert!(
            p.sample_error
                .as_deref()
                .is_some_and(|e| e.contains("Conversion failed!") && e.contains("Resume")),
            "{:?}",
            p.sample_error
        );
        p.show_tab("merging");
        p.show_tab("status");
        assert!(p.sample_error.is_none() && p.sample_merging.is_none());
        assert!(p.sample_helper.is_none());
        // A live stream recorded, then saved: looks only, each replacing the one before.
        use mdm_tools::video::LiveState;
        p.show_tab("error");
        p.show_tab("recording");
        assert_eq!(p.sample_live, Some(LiveState::Recording));
        assert!(p.sample_error.is_none());
        p.show_tab("saving");
        assert_eq!(p.sample_live, Some(LiveState::Saving));
        p.show_tab("merging");
        assert!(p.sample_live.is_none());
        p.show_tab("saving");
        p.show_tab("status");
        assert!(p.sample_live.is_none() && p.sample_helper.is_none());
    }

    /// A live stream's status says it is saved, else that it records, before anything else
    /// would ("Receiving data...", "Connecting..."); any other download has none of its own.
    #[test]
    fn a_live_stream_says_it_is_saved_or_recorded_first() {
        use mdm_tools::video::LiveState;
        assert_eq!(live_status(Some(LiveState::Saving)), Some("Saving..."));
        assert_eq!(
            live_status(Some(LiveState::Recording)),
            Some("Recording...")
        );
        assert_eq!(live_status(None), None);
    }
}

#[cfg(test)]
mod glide_tests {
    use super::{BAR_WIDTH, GLIDE_HOLD, GLIDE_MAX, Glide, glide_frame};

    /// Figures every quarter second (`delta` bytes more each), read by frames every 40 ms that
    /// don't fall on them, as the app reads them: (time, the figure, what is drawn) a frame.
    fn run(frames: usize, delta: u64) -> Vec<(f64, u64, f64)> {
        let mut g = Glide::default();
        (0..frames)
            .map(|k| {
                let now = k as f64 * 0.04;
                let real = (now / 0.25).floor() as u64 * delta;
                (now, real, g.step(now, real, &[(0, real)], false).done)
            })
            .collect()
    }

    /// The first figures are drawn as they are.
    #[test]
    fn the_first_figures_are_drawn_as_they_are() {
        let mut g = Glide::default();
        let d = g.step(3.0, 700, &[(0, 300), (1000, 400)], false);
        assert_eq!(d.done, 700.0);
        assert_eq!(d.segments, [(0, 300.0), (1000, 400.0)]);
    }

    /// Between figures that keep coming the bar never stops, never goes back and never passes
    /// the figure read; it stays less than two figures behind.
    #[test]
    fn steady_figures_glide_without_a_stop() {
        let frames = run(200, 500_000);
        let mut last = 0.0;
        for (k, &(now, real, drawn)) in frames.iter().enumerate() {
            assert!(drawn <= real as f64 + 1e-6, "past the figure at {now}");
            assert!(drawn >= last - 1e-6, "back at {now}");
            if now > 0.3 {
                assert!(drawn > last, "stopped at frame {k} ({now:.2} s)");
                assert!(drawn >= real as f64 - 2.0 * 500_000.0, "behind at {now}");
            }
            last = drawn;
        }
    }

    /// A figure below what is drawn (the download started over) is drawn at once.
    #[test]
    fn a_figure_that_goes_back_is_drawn_at_once() {
        let mut g = Glide::default();
        g.step(0.0, 0, &[(0, 0)], false);
        g.step(0.25, 1000, &[(0, 1000)], false);
        let d = g.step(0.40, 1000, &[(0, 1000)], false);
        assert!(d.done > 0.0 && d.done < 1000.0);
        let d = g.step(0.41, 10, &[(0, 10)], false);
        assert_eq!(d.done, 10.0);
        assert_eq!(d.segments, [(0, 10.0)]);
    }

    /// A finished download is drawn whole at once, not a glide behind its last figures: its
    /// window shows it whole before it gives way to Download complete; no frames of its own.
    #[test]
    fn a_finished_download_is_drawn_whole_at_once() {
        let mut g = Glide::default();
        g.step(0.0, 0, &[(0, 0)], false);
        g.step(0.25, 999, &[(0, 999)], false);
        let d = g.step(0.30, 999, &[(0, 999)], false);
        assert!(d.done < 999.0, "{}", d.done);
        let d = g.step(0.31, 1000, &[(0, 1000)], true);
        assert_eq!(d.done, 1000.0);
        assert_eq!(d.segments, [(0, 1000.0)]);
        assert_eq!(d.rate, 0.0);
        let d = g.step(0.35, 1000, &[(0, 1000)], true);
        assert_eq!(d.done, 1000.0);
    }

    /// A file's bar counts what has come before it is written (the engine's map moves later):
    /// it glides on its own figure all the same.
    #[test]
    fn the_bar_glides_on_its_own_figure() {
        let mut g = Glide::default();
        g.step(0.0, 0, &[(0, 0)], false);
        let d = g.step(0.25, 1000, &[(0, 0)], false);
        assert!(d.rate > 0.0);
        let d = g.step(0.40, 1000, &[(0, 0)], false);
        assert!(d.done > 0.0 && d.done < 1000.0, "{}", d.done);
        assert_eq!(d.segments, [(0, 0.0)]);
    }

    /// The segment map moves with the bar: together its segments are always what the bar shows,
    /// and a segment that comes (a connection split one) grows from nothing.
    #[test]
    fn the_segment_map_moves_with_the_bar() {
        let mut g = Glide::default();
        let mut now = 0.0;
        let mut segs = vec![(0u64, 0u64), (5_000, 0)];
        for k in 0..60 {
            if k % 6 == 0 {
                segs[0].1 += 300;
                segs[1].1 += 100;
                if k == 30 {
                    segs.push((8_000, 50));
                }
            }
            let done: u64 = segs.iter().map(|s| s.1).sum();
            let d = g.step(now, done, &segs, false);
            let sum: f64 = d.segments.iter().map(|s| s.1).sum();
            assert!(
                (sum - d.done).abs() < 1e-6,
                "map {sum} vs bar {} at {now}",
                d.done
            );
            for ((_, shown), (_, real)) in d.segments.iter().zip(&segs) {
                assert!(*shown <= *real as f64 + 1e-6);
            }
            now += 0.04;
        }
    }

    /// Once the figures stop moving the bar reaches them and the frames stop being asked for.
    #[test]
    fn frames_are_asked_for_only_while_the_figures_move() {
        let mut g = Glide::default();
        assert_eq!(g.step(0.0, 0, &[(0, 0)], false).rate, 0.0);
        let d = g.step(0.25, 100, &[(0, 100)], false);
        assert!((d.rate - 100.0 / (0.25 * 1.25)).abs() < 1e-9, "{}", d.rate);
        let d = g.step(0.25 + GLIDE_MAX, 100, &[(0, 100)], false);
        assert_eq!(d.done, 100.0);
        assert!(d.rate > 0.0, "moved lately");
        let d = g.step(0.25 + GLIDE_HOLD + 0.01, 100, &[(0, 100)], false);
        assert_eq!(d.done, 100.0);
        assert_eq!(d.rate, 0.0, "still: the app's own pace");
    }

    /// The window is drawn as often as its bar moves a pixel, 25 times a second at most (12.5
    /// while the main window is shown); a bar that moves less than a pixel between the app's own
    /// frames asks for none.
    #[test]
    fn frames_come_as_often_as_the_bar_moves_a_pixel() {
        use std::time::Duration;
        let mb = 1024.0 * 1024.0;
        let total = Some(100 * 1024 * 1024);
        // 100 MB at 10 MB/s: 40 pixels a second of the 400-point strip, at the cap.
        assert_eq!(
            glide_frame(10.0 * mb, total, 1.0, false),
            Some(Duration::from_millis(40))
        );
        assert_eq!(
            glide_frame(10.0 * mb, total, 1.0, true),
            Some(Duration::from_millis(80))
        );
        // At 2 MB/s: 8 pixels a second (12 at 150 %).
        let every = glide_frame(2.0 * mb, total, 1.0, false).unwrap();
        assert!((every.as_secs_f64() - 1.0 / (0.02 * BAR_WIDTH)).abs() < 1e-3);
        assert_eq!(glide_frame(2.0 * mb, total, 1.0, true), Some(every));
        let every = glide_frame(2.0 * mb, total, 1.5, false).unwrap();
        assert!((every.as_secs_f64() - 1.0 / (0.03 * BAR_WIDTH)).abs() < 1e-3);
        // At 0.5 MB/s: 2 pixels a second, the app's 4 frames do.
        for main in [false, true] {
            assert_eq!(glide_frame(0.5 * mb, total, 1.0, main), None);
            assert_eq!(glide_frame(0.0, total, 1.0, main), None);
            assert_eq!(glide_frame(10.0 * mb, None, 1.0, main), None);
            assert_eq!(glide_frame(10.0 * mb, Some(0), 1.0, main), None);
        }
    }
}
