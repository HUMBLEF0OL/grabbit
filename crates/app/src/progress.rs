//! The per-download progress dialog, painted to IDM 6.42's pixels
//! (notes/ui-spec/progress-dialog.md). Coordinates are client pixels from the spec.

use egui::{Color32, Pos2, Ui, ViewportBuilder, ViewportId, vec2};
use mdm_engine::Snapshot;
use mdm_engine::download::ConnStatus;

use crate::dialogs::display_path;
use crate::model::{self, Item, Status, size3};
// Laid out in English pixels: `fill` and `px` make each one where it is on screen (bars fill
// from the right in a right-to-left interface), `text` starts at an English pen.
use crate::winui::{self, ButtonKind, fill_l as fill, px_l as px, r, text};

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
/// The bar's width (513 points less its frame) and the segment map's (511).
const BAR_WIDTH: f64 = 511.0;
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

/// The segment / piece bar's 13 rows, top to bottom: (not here yet, here, a connection's
/// start), from the theme (IDM's gradients in the classic look; its dark variant dims the
/// "not here yet" rows, a bright white strip in the dark window).
fn seg_rows() -> impl Iterator<Item = (Color32, Color32, Color32)> {
    let s = &crate::theme::colors().segments;
    (0..13).map(|i| (s.missing[i], s.done[i], s.start[i]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Status,
    Limiter,
    Completion,
}

impl Progress {
    /// Open on a tab by its English name in lower case: a top tab ("status", "limit", "done") or
    /// a torrent's details tab ("peers"; /show progress:peers, for the docs' screenshots and the
    /// fit audit); any other name leaves it as it is. A top tab hidden with "Hide tab" comes back.
    /// States for the screenshots, each a look only (plan 09a): "error" shows Download
    /// status as for a failed download (a sample reason; the download itself is left as it
    /// is), "menu" opens the tab row's right-click menu; for a video (upstream issues 769, 736)
    /// "merging" shows its picture and sound being joined (45 %), "helper" the line saying why
    /// yt-dlp downloads it itself (an RTMP stream), "mergefail" a failed join's reason;
    /// "recording" and "saving" a live stream being recorded (its button "Stop and save") and
    /// being saved.
    pub fn show_tab(&mut self, name: &str) {
        // One sample state at a time: each one asked for replaces the one before.
        if matches!(
            name,
            "error" | "mergefail" | "merging" | "helper" | "recording" | "saving"
        ) {
            self.sample_error = None;
            self.sample_merging = None;
            self.sample_helper = None;
            self.sample_live = None;
        }
        match name {
            "recording" | "saving" => {
                self.sample_live = Some(if name == "recording" {
                    mdm_tools::video::LiveState::Recording
                } else {
                    mdm_tools::video::LiveState::Saving
                });
                self.sample_helper = Some(mdm_tools::streams::Helper::Live);
                self.tab = Tab::Status;
                return;
            }
            "error" => {
                self.sample_error = Some(crate::errors::describe(&mdm_engine::Error::Status(
                    reqwest::StatusCode::NOT_FOUND,
                )));
                self.tab = Tab::Status;
                return;
            }
            "mergefail" => {
                self.sample_error = Some(crate::model::merge_failure(
                    "Postprocessing: Conversion failed!",
                    &[
                        std::path::PathBuf::from("open-movie-trailer.f137.mp4"),
                        std::path::PathBuf::from("open-movie-trailer.f140.m4a"),
                    ],
                ));
                self.tab = Tab::Status;
                return;
            }
            "merging" => {
                self.sample_merging = Some(0.45);
                self.tab = Tab::Status;
                return;
            }
            "helper" => {
                self.sample_helper = Some(mdm_tools::streams::Helper::Protocol("rtmp".into()));
                self.tab = Tab::Status;
                return;
            }
            "menu" => {
                self.menu_asked = true;
                return;
            }
            _ => {}
        }
        let top = match name {
            "status" => Some(Tab::Status),
            "limit" => Some(Tab::Limiter),
            "done" => Some(Tab::Completion),
            _ => None,
        };
        if let Some(t) = top {
            // A tab asked for by name shows the download as it is again.
            self.sample_error = None;
            self.sample_merging = None;
            self.sample_helper = None;
            self.sample_live = None;
            self.hidden_tabs.retain(|h| *h != t);
            self.tab = t;
            return;
        }
        let tab = match name {
            "files" => TTab::Files,
            "peers" => TTab::Peers,
            "trackers" => TTab::Trackers,
            "pieces" => TTab::Pieces,
            "general" => TTab::General,
            _ => return,
        };
        self.ttab = tab;
        self.details = true;
    }
}

/// A torrent's details tabs (M15, D-014), in the place of the connections table.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TTab {
    Files,
    Peers,
    Trackers,
    Pieces,
    General,
}

pub struct Progress {
    pub id: u64,
    details: bool,
    tab: Tab,
    /// Tabs taken off the strip with their "Hide tab" button (#326), for this window.
    hidden_tabs: Vec<Tab>,
    ttab: TTab,
    limiter: bool,
    /// KBytes/sec typed on the Speed Limiter tab.
    limit_kb: String,
    /// The tab has been initialised from the download's current limit.
    limit_init: bool,
    /// A limit change to report to the app (`Action::Limit`), and whether to remember it.
    pending_limit: Option<(Option<u64>, bool)>,
    remember: bool,
    /// A torrent's own upload limit on the Speed Limiter tab: on, and its KBytes/sec box.
    up_on: bool,
    up_kb: String,
    /// An upload limit change to report (`Action::UploadLimit`).
    pending_upload: Option<Option<u64>>,
    /// Files ticked or unticked in a torrent's Files tab, to report (`Action::Choose`).
    pending_choose: Option<Action>,
    show_complete: bool,
    /// Torrent settings' "Keep sharing torrents after they finish" (the General tab says so).
    pub keep_sharing: bool,
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
    /// 1191): its Transfer rate names it.
    pub limit: Option<crate::model::Limit>,
    /// The bar and the segment map between figures.
    glide: Glide,
    /// Options on completion, used when "Show download complete dialog" is off.
    hang_up: bool,
    exit: bool,
    turn_off_on: bool,
    turn_off: usize,
    force: bool,
    /// `/show progress:error`: Download status painted as failed with this reason, the
    /// download left as it is (a screenshot's state).
    sample_error: Option<String>,
    /// `/show progress:merging`: Download status and the bar as while a video's streams are
    /// joined, this far (a screenshot's state).
    sample_merging: Option<f64>,
    /// `/show progress:helper`: the Server row as for a video yt-dlp downloads itself, for this
    /// reason (a screenshot's state).
    sample_helper: Option<mdm_tools::streams::Helper>,
    /// `/show progress:menu`: the tab row's menu opens at the next frame.
    menu_asked: bool,
    /// `/show progress:recording` / `progress:saving`: Download status and the Pause button as
    /// for a live stream being recorded, or saved (a screenshot's state).
    sample_live: Option<mdm_tools::video::LiveState>,
    /// A live stream's window, once seen as one: its Pause button is laid out for "Stop and
    /// save" too, and keeps that size when the label changes back.
    live_stream: bool,
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
}

impl Progress {
    /// `show_complete`: Options' "Show download complete dialog", this file's first state.
    pub fn new(id: u64, show_complete: bool) -> Self {
        Progress {
            id,
            details: true,
            tab: Tab::Status,
            hidden_tabs: Vec::new(),
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
            show_complete,
            keep_sharing: false,
            host: None,
            hidden: false,
            tray: false,
            minimize_to_tray: false,
            main_shown: true,
            limit: None,
            glide: Glide::default(),
            hang_up: false,
            exit: false,
            turn_off_on: false,
            turn_off: 0,
            force: false,
            sample_error: None,
            sample_merging: None,
            sample_helper: None,
            menu_asked: false,
            sample_live: None,
            live_stream: false,
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
        let body = if self.details {
            [548.0, 412.0 + torrent_drop(item)]
        } else {
            [548.0, 241.0]
        };
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
        if let Some(p) = crate::dialogs::place(ctx, key, [548.0, 412.0]) {
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
                        self.body(ui, o, item, snap, frac, (bar, &drawn.segments))
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
            if ctx
                .data(|d| d.get_temp::<Framed>(key.with("framed")))
                .as_ref()
                != Some(&want)
                && let Some(h) =
                    crate::win32::thin_frame(&title_now, shown, scale, want.2.as_deref())
            {
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

    /// `frac`: the download's fraction as it is (the text); `drawn`: the bar's fill and the
    /// segments as they glide ([`Glide`]).
    fn body(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        item: &Item,
        snap: Option<&Snapshot>,
        frac: Option<f64>,
        drawn: (f64, &[(u64, f64)]),
    ) -> Action {
        let p = ui.painter().clone();
        self.tabs(ui, o);
        // The state painted: the download's, or a screenshot's failed one (`sample_error`).
        let state = if self.sample_error.is_some() {
            Status::Error
        } else {
            item.status
        };
        let hide_tab = match self.tab {
            Tab::Status => {
                status_page(
                    ui,
                    o,
                    item,
                    snap,
                    frac,
                    self.limit,
                    Samples {
                        failed_with: self.sample_error.as_deref(),
                        merging: self.sample_merging,
                        helper: self.sample_helper.as_ref(),
                        live: self.sample_live,
                    },
                );
                false
            }
            Tab::Limiter => self.limiter_page(ui, o, item),
            Tab::Completion => self.completion_page(ui, o, item),
        };
        if hide_tab {
            self.hide(self.tab);
            // The strip above was drawn with the tab still on it: draw the window again at once,
            // without showing this pass, so the tab goes in one step. A download's window
            // otherwise waits for its next timed repaint (250 ms while it downloads) on the old page.
            ui.ctx().request_discard("Hide tab");
            ui.ctx().request_repaint();
        }

        // The progress bar on the shared, themed track. After the last byte, while the file is
        // checked or a video joined (upstream issue 736), it starts again in the accent colour
        // and shows how far that is.
        let bar = r(o, 17.0, 180.0, 513.0, 16.0);
        match self.sample_merging.or(finishing_fraction(snap)) {
            Some(f) => winui::meter_in(&p, bar, f as f32, crate::theme::colors().accent),
            None => winui::meter(&p, bar, drawn.0 as f32),
        }

        let mut action = Action::None;
        let details_label = if self.details {
            crate::i18n::mark("<< Hide details")
        } else {
            crate::i18n::mark("Show details >>")
        };
        // Graphite's row (plan 07): Details at the start; Cancel, then Pause / Resume last and
        // filled. A look only: Enter presses nothing here and Escape hides the window, as
        // before. Each button is laid out for the longer of its two labels, so it keeps its
        // size when they change; Tab keeps its order (Details, Pause, Cancel).
        let width = |label: &str| {
            (winui::text_width(&p, crate::i18n::t(label)) + 28.0)
                .ceil()
                .max(73.0)
        };
        // The site sent its page instead of the file (upstream issue 701): a button after
        // Details opens the link in the browser, which gets past the page (its download then
        // comes back to MDM).
        // i18n: button in a download's progress window when the site sent a web page (a waiting page, a captcha) instead of the file: opens the download's link in the web browser
        let open_label = crate::i18n::mark("Open in browser");
        let web_page = item.got_web_page();
        // A live stream being recorded or saved (or a screenshot's of one).
        let live = if self.sample_error.is_some() {
            None
        } else {
            self.sample_live.or_else(|| item.live_state())
        };
        self.live_stream |=
            live.is_some() || matches!(item.video_helper(), Some(mdm_tools::streams::Helper::Live));
        let widths = [
            width(winui::widest_label(
                &p,
                &["<< Hide details", "Show details >>"],
            )),
            width(winui::widest_label(&p, pause_labels(self.live_stream))),
            width("Cancel"),
            width(open_label),
        ];
        let mut row = vec![
            (widths[0], winui::Side::Leading, false),
            (widths[1], winui::Side::Trailing, true),
            (widths[2], winui::Side::Trailing, false),
        ];
        if web_page {
            row.push((widths[3], winui::Side::Leading, false));
        }
        // Within the bar's span (17..530): the footer's edges laid out on 545 points, moved
        // right by 1.
        let slots: Vec<f32> = winui::footer_slots(&row, 545.0)
            .into_iter()
            .map(|x| x + 1.0)
            .collect();
        let (details_at, pause_at, cancel_at) = (
            (slots[0], widths[0]),
            (slots[1], widths[1]),
            (slots[2], widths[2]),
        );
        // Plain: Pause / Resume is the row's one emphasised answer.
        if winui::button(
            ui,
            r(o, details_at.0, BUTTONS_Y, details_at.1, winui::CONTROL_H),
            details_label,
            ButtonKind::Normal,
            false,
            None,
        )
        .clicked()
        {
            self.details = !self.details;
        }
        if let Some(&open_x) = slots.get(3)
            && winui::button(
                ui,
                r(o, open_x, BUTTONS_Y, widths[3], winui::CONTROL_H),
                open_label,
                ButtonKind::Normal,
                false,
                None,
            )
            .clicked()
        {
            crate::launch::open_page(item.url.as_str());
        }
        let running = state == Status::Downloading;
        // Filled as the row's last answer; its look only (no key presses it).
        let (label, pkind) = pause_button(state, live);
        if winui::button(
            ui,
            r(o, pause_at.0, BUTTONS_Y, pause_at.1, winui::CONTROL_H),
            label,
            pkind,
            false,
            None,
        )
        .clicked()
        {
            // A screenshot's state resumes nothing (nor saves): the press only ends it.
            action = if self.sample_error.take().is_some() || self.sample_live.take().is_some() {
                Action::None
            } else if running {
                Action::Pause
            } else {
                Action::Resume
            };
        }
        if winui::button(
            ui,
            r(o, cancel_at.0, BUTTONS_Y, cancel_at.1, winui::CONTROL_H),
            "Cancel",
            ButtonKind::Normal,
            false,
            None,
        )
        .clicked()
        {
            action = Action::Cancel;
        }

        if self.details && item.torrent.is_some() {
            self.torrent_details(ui, &p, o + egui::vec2(0.0, torrent_drop(item)), item);
        } else if self.details {
            winui::text_in_window(
                ui,
                &p,
                o.x + 137.0,
                o.y + 242.0,
                crate::i18n::t("Start positions and download progress by connections"),
                winui::TEXT(),
            );
            segment_bar(&p, o, item, drawn.1);
            connections(ui, &p, o, snap);
        }
        action
    }

    /// Classic 3-D tabs over a raised page frame (spec "Tab strip").
    fn tabs(&mut self, ui: &mut Ui, o: Pos2) {
        // The shared tab strip (the torrent tabs' too): one look in both themes, one Tab stop with
        // the arrows, sized to the translated labels. It had two of its own (classic 3-D in dark).
        let tabs: Vec<(Tab, &str)> = [
            // i18n: tab title: how the download is going
            (Tab::Status, crate::i18n::mark("Download status")),
            // i18n: tab title: the tool that limits the download speed
            (Tab::Limiter, crate::i18n::mark("Speed Limiter")),
            // i18n: tab title: what happens when the download finishes
            (Tab::Completion, crate::i18n::mark("Options on completion")),
        ]
        .into_iter()
        .filter(|t| !self.hidden_tabs.contains(&t.0))
        .collect();
        let p = ui.painter().clone();
        let english: Vec<&str> = tabs.iter().map(|t| t.1).collect();
        let right = 525.0;
        let light: Vec<winui::LightTab> = winui::tab_row(&p, &english, 20.0, right, 2.0, 8.0)
            .into_iter()
            .zip(tabs.iter())
            .map(|((x0, x1, _), (_, label))| winui::LightTab {
                label,
                sel: (x0 - 2.0, x1 + 2.0),
                unsel: (x0, x1),
            })
            .collect();
        let current = tabs.iter().position(|t| t.0 == self.tab).unwrap_or(0);
        // The page: x 18..529, y 28..171 (tabs_light's pane starts 20 px under its top).
        if let Some(i) = winui::tabs_light(ui, o, r(o, 18.0, 8.0, 514.0, 165.0), &light, current) {
            self.tab = tabs[i].0;
        }
        self.tab_row_menu(ui, r(o, 18.0, 6.0, 514.0, 22.0));
    }

    /// The tab row's right-click menu (#326): "Show hidden tabs" puts back the tabs hidden in
    /// this window, greyed while none is. From the keyboard too, on the focused tab: Shift+F10
    /// or the Menu key.
    fn tab_row_menu(&mut self, ui: &Ui, row: egui::Rect) {
        use crate::chrome::{MenuEntry, MenuKind, PopupResult};
        let ctx = ui.ctx();
        let key = egui::Id::unique(("progress-tab-menu", self.id));
        let by_pointer = ui.input(|i| {
            i.pointer
                .secondary_clicked()
                .then(|| i.pointer.interact_pos())
                .flatten()
                .filter(|p| row.contains(*p))
        });
        let by_keys = || {
            let tab = ctx
                .memory(|m| m.focused())
                .and_then(|f| ctx.read_response(f))
                .map(|r| r.rect)
                .filter(|r| row.contains(r.center()))?;
            ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::SHIFT, egui::Key::F10)
                    || i.consume_key(egui::Modifiers::NONE, egui::Key::Menu)
            })
            // Under the tab, from its leading edge (a menu's top-right in right to left).
            .then(|| {
                if winui::rtl() {
                    tab.right_bottom()
                } else {
                    tab.left_bottom()
                }
            })
        };
        // `/show progress:menu` (a screenshot's): under the strip's first tab.
        let asked =
            std::mem::take(&mut self.menu_asked).then(|| row.left_bottom() + vec2(2.0, 0.0));
        if let Some(at) = asked.or(by_pointer).or_else(by_keys) {
            ctx.data_mut(|d| d.insert_temp(key, at));
        }
        let Some(at) = ctx.data(|d| d.get_temp::<Pos2>(key)) else {
            return;
        };
        let entries = [MenuEntry::Item(
            // i18n: right-click menu of a download window's tab row: bring back the tabs taken off with "Hide tab"
            "&Show hidden tabs",
            (),
            !self.hidden_tabs.is_empty(),
            false,
        )];
        let id = format!("progress-tab-menu-{}", self.id);
        match crate::chrome::popup(ctx, MenuKind::Context, &id, at, &entries, &|_| None) {
            PopupResult::Open | PopupResult::Neighbor(_) => return,
            PopupResult::Chosen(()) => {
                self.show_hidden_tabs();
                ctx.request_repaint();
            }
            PopupResult::Dismissed => {}
        }
        ctx.data_mut(|d| d.remove::<Pos2>(key));
    }

    /// Puts every tab hidden with "Hide tab" back on the strip, in its own place.
    fn show_hidden_tabs(&mut self) {
        self.hidden_tabs.clear();
    }

    /// Takes `tab` off the strip (its "Hide tab" button, #326) and shows Download status, which
    /// has no such button and so always stays.
    fn hide(&mut self, tab: Tab) {
        if tab != Tab::Status && !self.hidden_tabs.contains(&tab) {
            self.hidden_tabs.push(tab);
        }
        self.tab = Tab::Status;
    }

    /// The Speed Limiter tab, live for this download. Returns true when "Hide tab" is pressed.
    fn limiter_page(&mut self, ui: &mut Ui, o: Pos2, item: &Item) -> bool {
        let p = ui.painter().clone();
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
        text(
            &p,
            o.x + 33.0,
            o.y + 51.0,
            crate::i18n::t("Transfer rate"),
            winui::TEXT(),
        );
        // Every byte in (upstream issue 736): nothing comes in while the file is checked or
        // joined.
        let rate = if item.status == Status::Downloading && !item.past_last_byte() {
            model::rate(item.rate())
        } else {
            String::new()
        };
        text(&p, o.x + 136.0, o.y + 51.0, &rate, winui::TEXT());
        let was_on = self.limiter;
        // A torrent's upload limit takes the right half (x 290): the download side's captions
        // stop before it.
        let torrent = item.torrent.is_some();
        winui::checkbox_until(
            ui,
            o + vec2(33.0, 64.0),
            &mut self.limiter,
            "Use Speed Limiter",
            true,
            torrent.then_some(286.0),
        );
        let label = if self.limiter {
            winui::TEXT()
        } else {
            disabled()
        };
        winui::fit(
            ui,
            &p,
            o.x + 33.0,
            o.y + 92.0,
            crate::i18n::t("Maximum download speed:"),
            label,
            o.x + if torrent { 284.0 } else { 525.0 },
            ("limit-label", self.id),
        );
        let rate_box = winui::edit_right(
            ui,
            r(o, 33.0, 100.0, 77.0, 23.0),
            &mut self.limit_kb,
            self.limiter,
        );
        self.limit_kb.retain(|c| c.is_ascii_digit());
        // A typed rate applies once typed (focus leaves, Enter), not at each digit (typing 100
        // would throttle to 1, then 10 KB/s on the way). A rate a screen reader or UI Automation
        // sets (Set value) applies at once: there is no typing to wait for, and often no focus
        // to leave, so it never applied.
        let typed = rate_box.lost_focus()
            || (rate_box.has_focus() && winui::enter_pressed(ui))
            || winui::value_set_by_assistive(&rate_box);
        // i18n: unit after an input box: kilobytes per second
        text(
            &p,
            o.x + 118.0,
            o.y + 113.0,
            crate::i18n::t("KBytes/sec"),
            label,
        );
        let remembered = self.remember;
        let remember_box = winui::checkbox(ui, o + vec2(33.0, 134.0), &mut self.remember, "", true);
        // Its caption is drawn apart (two lines, below): screen readers get it as its name.
        let name = format!(
            "{} {}",
            crate::i18n::t("Remember Speed Limiter settings for this file"),
            crate::i18n::t("on download stop/resume")
        );
        winui::expose(
            &remember_box,
            egui::Role::CheckBox,
            &name,
            true,
            Some(self.remember),
        );
        // Its caption is two lines drawn below; a click on them ticks it too.
        winui::caption_click(
            ui,
            r(o, 51.0, 127.0, 450.0, 27.0),
            "remember-limit",
            &mut self.remember,
            true,
        );
        // An empty or zero rate isn't "unlimited" while the box is ticked: the limit stays
        // as it was until a rate is typed.
        let kb: u64 = self.limit_kb.parse().unwrap_or(0);
        if (was_on != self.limiter || (self.limiter && typed) || remembered != self.remember)
            && !(self.limiter && kb == 0)
        {
            let limit = self.limiter.then_some(kb * 1024);
            self.pending_limit = Some((limit, self.remember));
        }
        text(
            &p,
            o.x + 51.0,
            o.y + 137.0,
            crate::i18n::t("Remember Speed Limiter settings for this file"),
            winui::TEXT(),
        );
        text(
            &p,
            o.x + 51.0,
            o.y + 150.0,
            crate::i18n::t("on download stop/resume"),
            winui::TEXT(),
        );
        if item.torrent.is_some() {
            self.upload_limit(ui, &p, o);
        }
        hide_tab(ui, o)
    }

    /// A torrent's own upload limit, right of the download one (#39): kept with the torrent,
    /// so it applies again whenever it runs or shares.
    fn upload_limit(&mut self, ui: &mut Ui, p: &egui::Painter, o: Pos2) {
        let was_on = self.up_on;
        winui::checkbox_until(
            ui,
            o + vec2(290.0, 64.0),
            &mut self.up_on,
            // i18n: a torrent's progress window, Speed Limiter tab: a limit for what this torrent sends to others
            "Limit uploading",
            true,
            Some(525.0),
        );
        let label = if self.up_on {
            winui::TEXT()
        } else {
            disabled()
        };
        winui::fit(
            ui,
            p,
            o.x + 290.0,
            o.y + 92.0,
            // i18n: a torrent's progress window, Speed Limiter tab: label above the upload speed box
            crate::i18n::t("Maximum upload speed:"),
            label,
            o.x + 525.0,
            ("upload-label", self.id),
        );
        let rate_box = winui::edit_right(
            ui,
            r(o, 290.0, 100.0, 77.0, 23.0),
            &mut self.up_kb,
            self.up_on,
        );
        self.up_kb.retain(|c| c.is_ascii_digit());
        let typed = rate_box.lost_focus()
            || (rate_box.has_focus() && winui::enter_pressed(ui))
            || winui::value_set_by_assistive(&rate_box);
        text(
            p,
            o.x + 375.0,
            o.y + 113.0,
            crate::i18n::t("KBytes/sec"),
            label,
        );
        let kb: u64 = self.up_kb.parse().unwrap_or(0);
        if (was_on != self.up_on || (self.up_on && typed)) && !(self.up_on && kb == 0) {
            self.pending_upload = Some(self.up_on.then_some(kb * 1024));
        }
    }

    /// Options on completion → "Show download complete dialog" for this file.
    pub fn shows_complete(&self) -> bool {
        self.show_complete
    }

    /// Options on completion's other settings, when they apply (the complete dialog is off).
    pub fn when_done(&self) -> Option<crate::queue::WhenDone> {
        (!self.show_complete).then(|| crate::queue::WhenDone {
            hang_up: self.hang_up,
            exit: self.exit,
            turn_off: self
                .turn_off_on
                .then(|| crate::queue::TurnOff::ALL[self.turn_off]),
            force: self.turn_off_on && self.force,
        })
    }

    /// Returns true when "Hide tab" is pressed.
    fn completion_page(&mut self, ui: &mut Ui, o: Pos2, item: &Item) -> bool {
        let p = ui.painter().clone();
        // The label as wide as its language makes it (half the row at most), then the path up
        // to "Hide tab" (at the page's top right, on this row), shortened in the middle so the
        // file's name stays in sight (#62).
        let (left, right) = (o.x + 33.0, o.x + hide_tab_left(&p) - 8.0);
        let label = winui::fit(
            ui,
            &p,
            left,
            o.y + 45.0,
            crate::i18n::t("Save To:"),
            winui::TEXT(),
            left + 237.0,
            "save-to-label",
        );
        // From the label's width: its rect is mirrored in a right-to-left interface.
        let at = (left + label.width() + 6.0).max(o.x + 90.0);
        crate::fileicons::paint_at(
            ui,
            &p,
            crate::fileicons::Kind::File(&item.file_name),
            egui::pos2(at, o.y + 33.0),
            16.0,
        );
        winui::fit_middle(
            ui,
            &p,
            at + crate::fileicons::INDENT,
            o.y + 45.0,
            &display_path(&item.dest),
            winui::TEXT(),
            right,
            "save-to-path",
        );
        winui::checkbox(
            ui,
            o + vec2(33.0, 55.0),
            &mut self.show_complete,
            "Show download complete dialog",
            true,
        );
        winui::etched_h(&p, o.x + 33.0, o.y + 74.0, 474.0);
        text(
            &p,
            o.x + 33.0,
            o.y + 89.0,
            crate::i18n::t(
                "These settings are unavailable when \"Show download complete dialog\" is turned on",
            ),
            winui::TEXT(),
        );
        // Used when the complete dialog is off (it asks what to do itself).
        let on = !self.show_complete;
        // Dial-up is hung up through Windows' rasdial.
        winui::checkbox(
            ui,
            o + vec2(33.0, 96.0),
            &mut self.hang_up,
            "Hang up modem when done",
            on,
        );
        winui::checkbox(
            ui,
            o + vec2(33.0, 114.0),
            &mut self.exit,
            "Exit Grabbit when done",
            on,
        );
        winui::checkbox_until(
            ui,
            o + vec2(33.0, 132.0),
            &mut self.turn_off_on,
            "Turn off computer when done",
            on,
            Some(o.x + 267.0),
        );
        winui::checkbox(
            ui,
            o + vec2(52.0, 148.0),
            &mut self.force,
            "Force processes to terminate",
            on && self.turn_off_on,
        );
        if let Some(i) = winui::combo_light(
            ui,
            r(o, 267.0, 134.0, 119.0, 21.0),
            &crate::queue::TurnOff::LABELS,
            self.turn_off,
            on && self.turn_off_on,
            "progress-turn-off",
        ) {
            self.turn_off = i;
        }
        hide_tab(ui, o)
    }
}

/// "Hide tab" at the page's top right, as the mock has it (plan 09a): 30 high, 8 inside the
/// page's frame on the right (x 18..529) and 6 under its top (y 28), so it ends at y 64, where
/// the Speed Limiter's check boxes start.
const HIDE_TAB_RIGHT: f32 = 521.0;
const HIDE_TAB_Y: f32 = 34.0;
// Inside the page's frame, and clear of the Speed Limiter's check boxes (y 64) and Options on
// completion's rule (y 74).
const _: () = assert!(HIDE_TAB_Y >= 28.0 && HIDE_TAB_RIGHT <= 529.0);
const _: () = assert!(HIDE_TAB_Y + winui::CONTROL_H <= 64.0);

/// "Hide tab"'s English width: its label's, a longer translation widening it to the left.
const HIDE_TAB_W: f32 = 74.0;
const HIDE_TAB_MAX_W: f32 = 190.0;

/// Where "Hide tab" starts (English x). What shares its rows (Options on completion's Save To
/// path) stops before it.
fn hide_tab_left(p: &egui::Painter) -> f32 {
    HIDE_TAB_RIGHT - winui::button_fit_width(p, "Hide tab", HIDE_TAB_W, HIDE_TAB_MAX_W)
}

fn hide_tab(ui: &mut Ui, o: Pos2) -> bool {
    let rc = winui::button_rect_for(
        ui.painter(),
        r(
            o,
            HIDE_TAB_RIGHT - HIDE_TAB_W,
            HIDE_TAB_Y,
            HIDE_TAB_W,
            winui::CONTROL_H,
        ),
        "Hide tab",
        HIDE_TAB_MAX_W,
    );
    winui::button(ui, rc, "Hide tab", ButtonKind::Normal, false, None).clicked()
}

/// Right end for text on the tab page (its frame is x 18..529).
const PAGE_TEXT_RIGHT: f32 = 520.0;

/// The top of the Details / Pause / Cancel row: 30-point buttons between the progress bar
/// (ending at 196) and the connections' caption (its capitals from 233).
const BUTTONS_Y: f32 = 201.0;

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

/// The Graphite status icon (`st-*`) for a download's state, before its status word.
fn status_icon(status: Status) -> &'static str {
    match status {
        Status::Downloading => "st-downloading",
        Status::Stopped => "st-paused",
        Status::Queued => "st-queued",
        Status::Complete => "st-complete",
        Status::Error => "st-error",
    }
}

/// A screenshot's states for Download status (`/show progress:<state>`, looks only): failed for
/// this reason, a video's streams joined this far, a video yt-dlp downloads itself for this
/// reason, a live stream being recorded or saved.
#[derive(Clone, Copy, Default)]
struct Samples<'a> {
    failed_with: Option<&'a str>,
    merging: Option<f64>,
    helper: Option<&'a mdm_tools::streams::Helper>,
    live: Option<mdm_tools::video::LiveState>,
}

// The rows sit 7 px inside the page's frame at the top and at the bottom (the Server row's
// descent included), with a slightly wider break after Status, as before.
/// `samples`: a screenshot's state painted instead of the download's own.
fn status_page(
    ui: &Ui,
    o: Pos2,
    item: &Item,
    snap: Option<&Snapshot>,
    frac: Option<f64>,
    limit: Option<model::Limit>,
    samples: Samples,
) {
    let failed_with = samples.failed_with;
    let p = &ui.painter().clone();
    let state = if failed_with.is_some() {
        Status::Error
    } else {
        item.status
    };
    // Long values are cut with "..." at the page edge and shown whole on hover.
    let fit = |key: &str, x: f32, base: f32, s: &str, color| {
        let (rect, cut) = winui::text_fit(p, o.x + x, o.y + base, s, color, o.x + PAGE_TEXT_RIGHT);
        winui::copyable(ui, rect, key, s, cut);
    };
    // A live stream being recorded or saved (or a screenshot's of one).
    let live = if failed_with.is_some() {
        None
    } else {
        samples.live.or_else(|| item.live_state())
    };
    let status = match state {
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
        // i18n: a download's status (state it is in, not a command): stopped for now
        Status::Stopped => crate::i18n::t("Paused").to_owned(),
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
    };
    fit(
        "url",
        33.0,
        44.0,
        &crate::i18n::ltr(item.url.as_str()),
        winui::TEXT(),
    );
    // A failed download (the mock's error state): its reason in an error banner between the
    // address and Status, whole and copyable on hover; Status then says only "Error". The rows
    // move down under it, without Transfer rate and Time left (both blank unless it is
    // downloading), so they end where they end in every other state.
    let failed = state == Status::Error;
    let status = if failed {
        let at = r(o, 33.0, 50.0, PAGE_TEXT_RIGHT - 33.0, 30.0);
        winui::status_banner(ui, at, winui::BannerKind::Error, "st-error", &status, "");
        winui::copyable(ui, at, "error-banner", &status, true);
        // i18n: a download's status (state it is in, not a command): it failed
        crate::i18n::t("Error").to_owned()
    } else {
        status
    };
    let rows: Vec<(&str, f32, f32, String)> = vec![
        // i18n: noun: label before the download's state
        (crate::i18n::mark("Status"), 61.0, 102.0, status),
        (
            crate::i18n::mark("File size"),
            80.0,
            135.0,
            item.size.map_or(String::new(), size3),
        ),
        (
            // i18n: label before the amount received so far
            crate::i18n::mark("Downloaded"),
            96.0,
            135.0,
            match frac {
                // No size: only how much has come, no 0 % that never moves.
                None => size3(item.downloaded),
                Some(frac) => {
                    format!("{} ( {:.2} % )", size3(item.downloaded), frac * 100.0) // not translated: IDM's spacing
                }
            },
        ),
        (
            crate::i18n::mark("Transfer rate"),
            112.0,
            135.0,
            // Every byte in (upstream issue 736): nothing comes in while the file is checked or
            // joined.
            if state == Status::Downloading && !item.past_last_byte() {
                // The limit holding it back beside the speed (upstream issue 1191).
                let rate = model::rate(item.rate());
                match limit {
                    Some(limit) => limit.after(&rate),
                    None => rate,
                }
            } else {
                String::new()
            },
        ),
        (
            crate::i18n::mark("Time left"),
            128.0,
            135.0,
            item.time_left(),
        ),
        (
            // i18n: label: whether the server lets a stopped download continue (Yes / No)
            crate::i18n::mark("Resume capability"),
            145.0,
            135.0,
            match item.resumable {
                Some(true) => crate::i18n::t("Yes"),
                Some(false) => crate::i18n::t("No"),
                None => "",
            }
            .to_owned(),
        ),
    ];
    // Under the banner: Status, File size, Downloaded and Resume capability 16 apart from 96,
    // and Server at 160.
    let (rows, server_base) = if failed {
        let kept = rows
            .into_iter()
            .filter(|r| r.0 != "Transfer rate" && r.0 != "Time left")
            .enumerate()
            .map(|(i, (label, _, vx, value))| (label, 96.0 + 16.0 * i as f32, vx, value))
            .collect();
        (kept, 160.0)
    } else {
        (rows, 161.0)
    };
    // A value starts 6 px after its label at the least (a long translation); the rows that share
    // the column at 135 (all but Status) keep sharing one.
    let after = |label: &str| 33.0 + winui::text_width(p, crate::i18n::t(label)).ceil() + 6.0;
    let column = rows
        .iter()
        .filter(|r| r.2 == 135.0)
        .map(|r| after(r.0))
        .fold(135.0, f32::max);
    for (label, base, vx, value) in rows {
        let vx = if vx == 135.0 {
            column
        } else {
            vx.max(after(label))
        };
        text(
            p,
            o.x + 33.0,
            o.y + base,
            crate::i18n::t(label),
            winui::TEXT(),
        );
        if label == "Status" {
            // The status word after its state's icon (plan 07: status as icon + word), in the
            // status colour, or the error colour for a failed download.
            let vc = if failed {
                winui::ERROR()
            } else {
                crate::theme::colors().status_value
            };
            let icon = winui::r(o, vx, base - 12.0, 16.0, 16.0);
            crate::themeicons::paint_graphite_in(ui, p, status_icon(state), icon, vc);
            fit(label, vx + 20.0, base, &value, vc);
        } else {
            fit(label, vx, base, &value, winui::TEXT());
        }
    }
    // Where its server is (#140): a file download's, and a video's whose streams the engine
    // downloads (upstream issue 769; a torrent or a gallery has no single server of our own). As
    // the running download sees it, else as its last run did.
    let seen = snap
        .and_then(|s| s.server.as_ref())
        .map(crate::whereis::Seen::from)
        .or_else(|| item.server.clone());
    // A video yt-dlp downloads itself (upstream issue 769): why, in that row.
    if let Some(why) = samples.helper.cloned().or_else(|| item.video_helper()) {
        fit(
            "helper",
            33.0,
            server_base,
            &helper_line(&why),
            winui::TEXT(),
        );
    } else if item.torrent.is_none()
        && item.gallery.is_none()
        && (item.video.is_none() || seen.is_some())
    {
        // i18n: label before where a download's server is (its country and round trip)
        let label = crate::i18n::t("Server");
        text(p, o.x + 33.0, o.y + server_base, label, winui::TEXT());
        crate::whereis::line(
            ui,
            o.x + column.max(33.0 + winui::text_width(p, label).ceil() + 6.0),
            o.y + server_base,
            o.x + PAGE_TEXT_RIGHT,
            seen.as_ref(),
            &format!("progress-{}", item.id),
        );
    }
}

/// How much lower a torrent's details sit than a download's (and how much taller its window
/// is): its tab strip, where the connections caption is, would touch the buttons above.
fn torrent_drop(item: &Item) -> f32 {
    if item.torrent.is_some() { 6.0 } else { 0.0 }
}

impl Progress {
    /// A torrent's details: a tab strip where the connections caption is, the piece map in
    /// the segment bar, and the tab's table below.
    fn torrent_details(&mut self, ui: &mut Ui, p: &egui::Painter, o: Pos2, item: &Item) {
        let d = match &item.live {
            Some(crate::model::Live {
                dl: crate::model::Job::Torrent(run),
                ..
            }) => Some(run.details()),
            // Finished and still shared (no Run): the session has it all the same, so the
            // tabs stay, the General tab's share ratio included.
            _ => self
                .host
                .as_ref()
                .and_then(|h| h.get())
                .zip(item.torrent.as_ref())
                .and_then(|(t, spec)| crate::torrents::details_of(&t, &spec.hash)),
        };
        let tabs = [
            // i18n: tab title: the torrent's files
            (TTab::Files, crate::i18n::mark("Files")),
            // i18n: tab title: the other computers sharing the torrent
            (TTab::Peers, crate::i18n::mark("Peers")),
            // i18n: tab title: the servers that list the peers
            (TTab::Trackers, crate::i18n::mark("Trackers")),
            // i18n: tab title: the parts the torrent is made of
            (TTab::Pieces, crate::i18n::mark("Pieces")),
            // i18n: tab title: general information
            (TTab::General, crate::i18n::mark("General")),
        ];
        // The usual tab control (the Scheduler's and Options' look): tabs sized to their
        // labels from the leading edge, the page under them holding the piece map and the
        // table.
        let english: Vec<&str> = tabs.iter().map(|t| t.1).collect();
        let light: Vec<winui::LightTab> = winui::tab_row(p, &english, 16.0, 530.0, 2.0, 8.0)
            .into_iter()
            .zip(tabs.iter())
            .map(|((x0, x1, _), (_, label))| winui::LightTab {
                label,
                sel: (x0 - 2.0, x1 + 2.0),
                unsel: (x0, x1),
            })
            .collect();
        let current = tabs.iter().position(|t| t.0 == self.ttab).unwrap_or(0);
        if let Some(i) = winui::tabs_light(ui, o, r(o, 14.0, 226.0, 521.0, 180.0), &light, current)
        {
            self.ttab = tabs[i].0;
        }
        let pieces = d.as_ref().map(|d| d.pieces.as_slice()).unwrap_or(&[]);
        // General, once the torrent is complete and shared (plan 09a, the mock's): an info
        // banner where the piece map is (every piece is there), the table under it.
        let sharing = d
            .as_ref()
            .and_then(|d| d.stats.as_ref())
            .filter(|s| {
                self.ttab == TTab::General
                    && item.status == Status::Complete
                    && s.phase == mdm_torrent::Phase::Finished
            })
            .map(sharing_line);
        if let Some(line) = &sharing {
            let at = r(o, 17.0, 249.0, 513.0, winui::status_banner_height(0));
            winui::status_banner(
                ui,
                at,
                winui::BannerKind::Info,
                "st-seeding",
                // i18n: a torrent's General tab, in a banner: it is all downloaded and shared with other peers
                crate::i18n::t("Complete, sharing"),
                "",
            );
            let (rc, cut) = winui::text_right(
                p,
                o.x + 17.0 + 513.0 - 12.0,
                o.y + 249.0 + 23.0,
                line,
                crate::theme::colors().text_secondary,
            );
            winui::copyable(ui, rc, "sharing-line", line, cut);
        } else {
            piece_bar(p, o, pieces);
        }
        let Some(d) = &d else {
            table(ui, p, o, "t-none", &[(25.0, "")], &[], &[], false, None);
            text(
                p,
                o.x + 25.0,
                o.y + 312.0,
                crate::i18n::t("Shown while the torrent is downloading."),
                winui::TEXT(),
            );
            return;
        };
        match self.ttab {
            TTab::Files => {
                let rows: Vec<Vec<String>> = d
                    .files
                    .iter()
                    .map(|(_, f, on, done)| {
                        let pct = if !on {
                            // i18n: a torrent file the user chose not to download
                            crate::i18n::t("Skipped").to_owned()
                        } else if f.size == 0 {
                            crate::i18n::percent(1.0, 0)
                        } else {
                            crate::i18n::percent(*done as f64 / f.size as f64, 1)
                        };
                        vec![f.path.display().to_string(), model::size(f.size), pct]
                    })
                    .collect();
                let ticks: Vec<bool> = d.files.iter().map(|f| f.2).collect();
                let clicked = files_table(ui, p, o, "t-files", &rows, &ticks, 6);
                // A box ticked or unticked (#39): the torrent downloads those files from now
                // on; the last one ticked can't be unticked (a torrent of nothing).
                if let Some(row) = clicked {
                    let mut on: Vec<bool> = ticks.clone();
                    on[row] = !on[row];
                    if on.iter().any(|b| *b) {
                        let chosen = d.files.iter().zip(&on).filter(|(_, b)| **b);
                        self.pending_choose = Some(Action::Choose {
                            files: chosen.clone().map(|(f, _)| f.0).collect(),
                            all: on.iter().all(|b| *b),
                            size: chosen.map(|(f, _)| f.1.size).sum(),
                        });
                    }
                }
            }
            TTab::Peers => {
                let rows: Vec<Vec<String>> = d
                    .peers
                    .iter()
                    .map(|q| {
                        vec![
                            q.addr.clone(),
                            q.client.clone().unwrap_or_default(),
                            model::size(q.fetched),
                            model::size(q.uploaded),
                        ]
                    })
                    .collect();
                table(
                    ui,
                    p,
                    o,
                    "t-peers",
                    &[
                        // i18n: column title: the web address (URL)
                        (25.0, crate::i18n::mark("Address")),
                        // i18n: column title: the peer's torrent program
                        (190.0, crate::i18n::mark("Client")),
                        // i18n: column title: how much came from the peer
                        (340.0, crate::i18n::mark("Received")),
                        // i18n: column title: how much went to the peer
                        (425.0, crate::i18n::mark("Sent")),
                    ],
                    &[185.0, 335.0, 420.0],
                    &rows,
                    false,
                    None,
                );
            }
            TTab::Trackers => {
                let rows: Vec<Vec<String>> = if d.trackers.is_empty() {
                    vec![vec![
                        crate::i18n::t("None: peers come from the DHT and other peers").into(),
                    ]]
                } else {
                    d.trackers.iter().map(|t| vec![t.clone()]).collect()
                };
                // i18n: column title: a server that lists the peers
                let head = [(25.0, crate::i18n::mark("Tracker"))];
                table(ui, p, o, "t-trackers", &head, &[], &rows, false, None);
            }
            TTab::Pieces => {
                let have = pieces.iter().filter(|b| **b).count();
                let head = crate::i18n::fmt(
                    // i18n: {have} and {total} = numbers of pieces; {size} = one piece's size
                    crate::i18n::t("{have} of {total} pieces ({size} each)"),
                    &[
                        ("have", &crate::i18n::integer(have as u64)),
                        ("total", &crate::i18n::integer(pieces.len() as u64)),
                        ("size", &model::size(u64::from(d.piece_length))),
                    ],
                );
                table(
                    ui,
                    p,
                    o,
                    "t-pieces",
                    &[(25.0, head.as_str())],
                    &[],
                    &[],
                    false,
                    None,
                );
                piece_legend(p, o);
                piece_grid(p, o, pieces);
            }
            TTab::General => {
                let st = d.stats.as_ref();
                let have = pieces.iter().filter(|b| **b).count();
                // What it sent in all its runs (#39; librqbit counts only this one), against what
                // it has: the same after a pause or a restart.
                let spec = item.torrent.as_ref();
                let sent = spec.map_or(0, |t| t.uploaded);
                let done = st.map_or(item.downloaded, |s| s.done);
                let ratio = crate::torrents::share_ratio_text(sent, done);
                let shared = spec.map_or(0, |t| t.seeded_secs);
                let hash = item
                    .torrent
                    .as_ref()
                    .map_or(String::new(), |t| t.hash.clone());
                let folder = item
                    .torrent
                    .as_ref()
                    .map_or(String::new(), |t| t.folder.display().to_string());
                let t = |s: &str| crate::i18n::t(s).to_owned();
                let rows = vec![
                    // i18n: the torrent's identifier
                    vec![t("Info hash"), hash],
                    vec![
                        t("Pieces"),
                        crate::i18n::fmt(
                            // i18n: {have} and {total} = numbers of pieces; {size} = one piece's size
                            crate::i18n::t("{have} of {total} ({size} each)"),
                            &[
                                ("have", &crate::i18n::integer(have as u64)),
                                ("total", &crate::i18n::integer(pieces.len() as u64)),
                                ("size", &model::size(u64::from(d.piece_length))),
                            ],
                        ),
                    ],
                    vec![
                        t("Peers"),
                        st.map_or(String::new(), |s| {
                            crate::i18n::fmt(
                                // i18n: {live} = peers connected now; {known} = peers heard of
                                crate::i18n::t("{live} connected, {known} known"),
                                &[
                                    ("live", &crate::i18n::integer(s.peers_live as u64)),
                                    ("known", &crate::i18n::integer(s.peers_seen as u64)),
                                ],
                            )
                        }),
                    ],
                    vec![
                        // i18n: how much was sent to other peers
                        t("Sent"),
                        model::size(sent),
                    ],
                    // i18n: sent divided by received
                    vec![t("Share ratio"), ratio],
                    vec![
                        // i18n: a torrent's General tab: how long it has been shared once finished, in all
                        t("Shared for"),
                        model::time(shared as f64),
                    ],
                    // i18n: label before the folder the torrent is saved in
                    vec![t("Saved in"), folder],
                    vec![
                        // i18n: noun: sharing the torrent with others after it is downloaded
                        t("Sharing"),
                        if self.keep_sharing {
                            // i18n: a torrent's sharing, when Torrent settings keep it on
                            t("Goes on after the download finishes")
                        } else {
                            t("Stops when the download finishes")
                        },
                    ],
                ];
                // Under the banner the table starts lower and shows fewer rows (the rest
                // scroll), ending inside the page as before.
                let (at, shown) = if sharing.is_some() {
                    (o + vec2(0.0, GENERAL_BANNER_DROP), GENERAL_BANNER_ROWS)
                } else {
                    (o, 6)
                };
                table_sized(
                    ui,
                    p,
                    at,
                    "t-general",
                    &[
                        (25.0, crate::i18n::mark("Property")),
                        (130.0, crate::i18n::mark("Value")),
                    ],
                    &[125.0],
                    &rows,
                    false,
                    None,
                    shown,
                );
            }
        }
    }
}

/// The General tab under its sharing banner: the table 19 lower (the banner's 37 from the
/// piece map's top, 249, and 6 under it) with 5 rows, so its frame ends at 403, inside the
/// page (226 + 180).
const GENERAL_BANNER_DROP: f32 = 19.0;
const GENERAL_BANNER_ROWS: usize = 5;

/// The sharing banner's line: the rate sent and to how many peers ("410.0 KB/s to 6 peers").
fn sharing_line(s: &mdm_torrent::Stats) -> String {
    // i18n: how many peers a torrent sends to now
    let peers = crate::i18n::tn(s.peers_live as u64, "{n} peer", "{n} peers");
    crate::i18n::fmt(
        // i18n: a torrent's General tab, under "Complete, sharing": {rate} = what it sends now (such as 410.0 KB/s); {peers} = to how many peers (such as 6 peers)
        crate::i18n::t("{rate} to {peers}"),
        &[("rate", &model::rate(s.up_speed)), ("peers", &peers)],
    )
}

/// The segment bar as a piece map: a column is filled when the pieces under it are there.
fn piece_bar(p: &egui::Painter, o: Pos2, pieces: &[bool]) {
    let (x0, y0) = (o.x + 17.0, o.y + 249.0);
    let border = crate::theme::colors().segment_border;
    fill(p, x0, y0, x0 + 512.0, y0, border);
    fill(p, x0, y0 + 14.0, x0 + 512.0, y0 + 14.0, border);
    fill(p, x0, y0, x0, y0 + 14.0, border);
    fill(p, x0 + 512.0, y0, x0 + 512.0, y0 + 14.0, border);
    let ix = o.x + 18.0;
    // Runs of equal columns, painted as one rectangle per row.
    let have = |c: usize| {
        if pieces.is_empty() {
            return false;
        }
        let a = c * pieces.len() / 511;
        let b = ((c + 1) * pieces.len() / 511).max(a + 1).min(pieces.len());
        pieces[a..b].iter().all(|x| *x)
    };
    let mut c = 0usize;
    while c < 511 {
        let on = have(c);
        let mut e = c;
        while e + 1 < 511 && have(e + 1) == on {
            e += 1;
        }
        for (row, cols) in seg_rows().enumerate() {
            let v = if on { cols.1 } else { cols.0 };
            let y = y0 + 1.0 + row as f32;
            fill(p, ix + c as f32, y, ix + e as f32, y, v);
        }
        c = e + 1;
    }
}

/// The Pieces tab's key, at the header's right: have / some / missing.
fn piece_legend(p: &egui::Painter, o: Pos2) {
    let (done, partial, missing) = piece_colors();
    let mut x = o.x + 505.0;
    for (c, label) in [
        // i18n: legend: pieces not downloaded yet
        (missing, crate::i18n::mark("Missing")),
        // i18n: legend: pieces partly downloaded
        (partial, crate::i18n::mark("Some")),
        // i18n: legend: pieces downloaded
        (done, crate::i18n::mark("Have")),
    ] {
        let label = crate::i18n::t(label);
        x -= winui::text_width(p, label);
        text(p, x, o.y + 290.0, label, winui::TEXT());
        x -= 10.0;
        p.rect_filled(
            winui::rect(egui::Rect::from_min_size(
                egui::pos2(x, o.y + 282.0),
                egui::vec2(7.0, 7.0),
            )),
            1.0,
            c,
        );
        x -= 14.0;
    }
}

/// Pieces there, partly there (a square standing for several), missing.
fn piece_colors() -> (Color32, Color32, Color32) {
    let c = crate::theme::colors();
    (c.piece, c.piece_partial, c.piece_missing)
}

/// The Pieces tab: one square per piece (or per group of pieces when there are many).
fn piece_grid(p: &egui::Painter, o: Pos2, pieces: &[bool]) {
    if pieces.is_empty() {
        return;
    }
    let (left, top, w, h) = (o.x + 22.0, o.y + 302.0, 498.0, 94.0);
    let cols = (w / 7.0) as usize;
    let rows = (h / 7.0) as usize;
    let cells = (cols * rows).min(pieces.len());
    let (done, partial, missing) = piece_colors();
    for k in 0..cells {
        let a = k * pieces.len() / cells;
        let b = ((k + 1) * pieces.len() / cells)
            .max(a + 1)
            .min(pieces.len());
        let got = pieces[a..b].iter().filter(|x| **x).count();
        let c = if got == b - a {
            done
        } else if got > 0 {
            partial
        } else {
            missing
        };
        let (x, y) = (
            left + (k % cols) as f32 * 7.0,
            top + (k / cols) as f32 * 7.0,
        );
        p.rect_filled(
            winui::rect(egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(6.0, 6.0),
            )),
            1.0,
            c,
        );
    }
}

/// The Files tab's table of a torrent's files, ticked or not: the progress window's (`shown`: 6
/// rows) and the Files window's (torrentfiles.rs, taller). The row whose box was clicked.
pub(crate) fn files_table(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    id: &str,
    rows: &[Vec<String>],
    checks: &[bool],
    shown: usize,
) -> Option<usize> {
    table_sized(
        ui,
        p,
        o,
        id,
        &[
            (25.0, crate::i18n::mark("File")),
            (335.0, crate::i18n::mark("Size")),
            (430.0, crate::i18n::mark("Done")),
        ],
        &[330.0, 425.0],
        rows,
        true,
        Some(checks),
        shown,
    )
}

/// The height of a [`table_sized`] of `shown` rows' body (the progress window's 6: 100 px).
pub(crate) fn table_body_height(shown: usize) -> f32 {
    17.0 * shown as f32 - 2.0
}

/// A table in the connections table's frame: headers at x, column lines at `seps`, rows of
/// text cut to their column (whole on hover), scrolled by the wheel. `checks`: a check box
/// before each row's icon, ticked or not (the Files tab); the row whose box was clicked.
#[allow(clippy::too_many_arguments)]
fn table(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    id: &str,
    heads: &[(f32, &str)],
    seps: &[f32],
    rows: &[Vec<String>],
    icons: bool,
    checks: Option<&[bool]>,
) -> Option<usize> {
    table_sized(ui, p, o, id, heads, seps, rows, icons, checks, 6)
}

/// [`table`] showing `shown` rows: it grows down from the same top.
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
) -> Option<usize> {
    let mut clicked = None;
    let body_h = table_body_height(shown);
    let outer = r(o, 17.0, 273.0, 513.0, body_h + 28.0);
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
        o.x + 528.0,
        o.y + 299.0 + body_h,
        bg,
    );
    fill(p, o.x + 19.0, o.y + 275.0, o.x + 527.0, o.y + 298.0, head);
    for x in seps {
        fill(p, o.x + x, o.y + 275.0, o.x + x, o.y + 298.0, line);
    }
    for (x, h) in heads {
        text(p, o.x + x, o.y + 290.0, crate::i18n::t(h), winui::TEXT());
    }
    let visible = shown;
    // Per window: each progress window scrolls its own tables.
    let key = egui::Id::unique(("torrent-table", id, ui.ctx().viewport_id()));
    let body = r(o, 19.0, 299.0, 508.0, body_h);
    let first = first_row(ui, key, body, rows.len(), visible);
    let clip = p.with_clip_rect(body);
    let right_of = |c: usize| seps.get(c).map_or(o.x + 520.0, |x| o.x + x - 4.0);
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
    let track = r(o, 511.0, 275.0, 17.0, body_h + 24.0);
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

/// How much wider than IDM's the connections table's number column is drawn (#1101): its
/// numbers (`widest`, the widest one's width) have 13 px before the column's line, its title
/// (`title`'s width) 25 px; the columns after it move over by the rest. 0 when both fit (one
/// and two digits in the classic look).
fn number_column_extra(widest: f32, title: f32) -> f32 {
    (widest - 13.0).max(title - 25.0).max(0.0).ceil().min(60.0)
}

/// The custom-drawn segment bar (spec "Segment bar"): 13-row gradients, red start columns;
/// `segments`: each one's start and bytes done, as drawn ([`Glide`]).
fn segment_bar(p: &egui::Painter, o: Pos2, item: &Item, segments: &[(u64, f64)]) {
    let (x0, y0) = (o.x + 17.0, o.y + 249.0);
    let border = crate::theme::colors().segment_border;
    fill(p, x0, y0, x0 + 512.0, y0, border);
    fill(p, x0, y0 + 14.0, x0 + 512.0, y0 + 14.0, border);
    fill(p, x0, y0, x0, y0 + 14.0, border);
    fill(p, x0 + 512.0, y0, x0 + 512.0, y0 + 14.0, border);
    for (x, y) in [
        (x0, y0),
        (x0 + 512.0, y0),
        (x0, y0 + 14.0),
        (x0 + 512.0, y0 + 14.0),
    ] {
        px(p, x, y, crate::theme::colors().segment_corner);
    }
    // Paint a run of columns [a, b] with one of the three row profiles.
    let run = |a: f32, b: f32, which: usize| {
        if b < a {
            return;
        }
        for (row, cols) in seg_rows().enumerate() {
            let c = match which {
                0 => cols.0,
                1 => cols.1,
                _ => cols.2,
            };
            fill(p, a, y0 + 1.0 + row as f32, b, y0 + 1.0 + row as f32, c);
        }
    };
    let ix = o.x + 18.0;
    run(ix, ix + 510.0, 0);
    let Some(total) = item.size.filter(|t| *t > 0) else {
        return;
    };
    let col = |v: u64| (v as f64 / total as f64 * 511.0).floor() as f32;
    let ppp = p.pixels_per_point();
    for &(start, done) in segments {
        let sx = ix + col(start);
        if let Some((a, b)) = segment_columns(start, done, total, 511.0) {
            run(ix + a, ix + b, 1);
        }
        // The start's tick, as wide in whole pixels wherever it falls (a 1-point column at
        // 150 % came out 2 pixels or 1 by its place, the 1-pixel ones all but gone).
        let column = egui::Rect::from_min_size(egui::pos2(sx, y0), vec2(1.0, 1.0));
        let (a, b) = tick_span(winui::rect(column).left(), ppp);
        for (row, cols) in seg_rows().enumerate() {
            let y = y0 + 1.0 + row as f32;
            winui::solid(
                p,
                egui::Rect::from_min_max(egui::pos2(a, y), egui::pos2(b, y + 1.0)),
                cols.2,
            );
        }
    }
}

/// How far the check or the merge after the last byte has come (upstream issue 736), when it
/// is under way and that is known.
fn finishing_fraction(snap: Option<&Snapshot>) -> Option<f64> {
    use mdm_engine::download::State;
    let s = snap.filter(|s| matches!(s.state, State::Verifying | State::Merging))?;
    s.finishing.map(|f| f.clamp(0.0, 1.0))
}

/// The columns a segment fills in the segment bar, from its start column's right to its end's
/// left, `total` bytes across `width` columns: a finished segment reaches the next one's start
/// column, so a finished download leaves no gap (upstream issue 736). None when it fills none.
fn segment_columns(start: u64, done: f64, total: u64, width: f64) -> Option<(f32, f32)> {
    let col = |v: f64| (v / total as f64 * width).floor();
    let first = col(start as f64);
    let end = col((start as f64 + done).min(total as f64));
    let last = (end - 1.0).min(width - 1.0);
    (last > first).then_some(((first + 1.0) as f32, last as f32))
}

#[cfg(test)]
mod segment_tests {
    use super::segment_columns;

    /// Upstream issue 736: two finished halves fill every column up to the next one's start, and
    /// the last one to the bar's end: no gap at 100 %.
    #[test]
    fn finished_segments_leave_no_gap() {
        let (total, width) = (1000, 511.0);
        assert_eq!(segment_columns(0, 500.0, total, width), Some((1.0, 254.0)));
        // The second one's start column is 255 (drawn red), its fill runs to the end.
        assert_eq!(
            segment_columns(500, 500.0, total, width),
            Some((256.0, 510.0))
        );
        // Under way: up to where it has come.
        assert_eq!(segment_columns(0, 100.0, total, width), Some((1.0, 50.0)));
        // Nothing yet, or less than a column: no fill.
        assert_eq!(segment_columns(0, 0.0, total, width), None);
        assert_eq!(segment_columns(500, 1.0, total, width), None);
    }
}

/// The start tick's columns from point `x` at `ppp` pixels a point: from `x`'s nearest pixel,
/// one point rounded to whole pixels (at least one) wide, so every tick is the same width.
fn tick_span(x: f32, ppp: f32) -> (f32, f32) {
    let a = (x * ppp).round() / ppp;
    (a, a + ppp.round().max(1.0) / ppp)
}

/// The Pause button's label and kind (filled, the row's last answer): a live stream being
/// recorded is stopped and saved (still `Action::Pause`, the app's `halt` saves it), and while it
/// is saved the button waits.
fn pause_button(
    status: Status,
    live: Option<mdm_tools::video::LiveState>,
) -> (&'static str, ButtonKind) {
    use mdm_tools::video::LiveState;
    match live {
        // i18n: button in a live stream's progress window: stop recording it and keep what was recorded as a video file
        Some(LiveState::Recording) => (crate::i18n::mark("Stop and save"), ButtonKind::Accent),
        Some(LiveState::Saving) => (crate::i18n::mark("Stop and save"), ButtonKind::Disabled),
        None => match status {
            // i18n: verb: pause the download
            Status::Downloading => (crate::i18n::mark("Pause"), ButtonKind::Accent),
            // i18n: verb: continue the download
            Status::Stopped | Status::Error => (crate::i18n::mark("Resume"), ButtonKind::Accent),
            _ => (crate::i18n::mark("Pause"), ButtonKind::Disabled),
        },
    }
}

/// The labels the Pause button can show for one download, its width the widest's: "Stop and
/// save" only in a live stream's window, so no other's buttons move.
fn pause_labels(live_stream: bool) -> &'static [&'static str] {
    if live_stream {
        &["Pause", "Resume", "Stop and save"]
    } else {
        &["Pause", "Resume"]
    }
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

/// The connections table (spec "Connections table"): N. · Downloaded · Info.
fn connections(ui: &mut Ui, p: &egui::Painter, o: Pos2, snap: Option<&Snapshot>) {
    let outer = r(o, 17.0, 273.0, 513.0, 128.0);
    winui::solid(p, outer, winui::FIELD_EDGE());
    fill(
        p,
        o.x + 18.0,
        o.y + 274.0,
        o.x + 528.0,
        o.y + 399.0,
        // Every list's background (IDM's light one is mauve, #beb4be: the only one of its kind).
        winui::FIELD(),
    );
    fill(
        p,
        o.x + 19.0,
        o.y + 275.0,
        o.x + 510.0,
        o.y + 298.0,
        winui::LIST_HEADER(),
    );
    // Listed by connection number, like IDM (the engine reports them in arrival order).
    let mut conns: Vec<_> = snap
        .map(|s| s.connections.iter().collect())
        .unwrap_or_default();
    conns.sort_by_key(|c| c.id);
    // i18n: column title: connection number (abbreviation of Number)
    let number = crate::i18n::t("N.");
    // The number column is as wide as its widest number and its title (#1101: with 16
    // connections, 10 to 16 were cut to "..."); the other columns move over.
    let widest = conns
        .iter()
        .map(|c| winui::text_width(p, &(c.id + 1).to_string()))
        .fold(0.0, f32::max);
    let dx = number_column_extra(widest, winui::text_width(p, number));
    for x in [48.0 + dx, 148.0 + dx, 498.0] {
        fill(
            p,
            o.x + x,
            o.y + 275.0,
            o.x + x,
            o.y + 298.0,
            crate::theme::colors().header_sep,
        );
    }
    // Column titles end in "..." before their column's line, whole on hover; a long "N." starts
    // at the list's edge.
    let head = |x: f32, right: f32, label: &str| {
        let (rc, cut) = winui::text_fit(p, o.x + x, o.y + 290.0, label, winui::TEXT(), o.x + right);
        if cut {
            winui::full_on_hover(ui, rc, ("connections-head", x as i32), label);
        }
    };
    let x = if winui::text_width(p, number) > 21.0 {
        21.0
    } else {
        25.0
    };
    head(x, 46.0 + dx, number);
    // i18n: column title: bytes this connection received
    head(55.0 + dx, 146.0 + dx, crate::i18n::t("Downloaded"));
    // i18n: column title: what the connection is doing
    head(155.0 + dx, 496.0, crate::i18n::t("Info"));
    let key = egui::Id::unique(("conn-scroll", ui.ctx().viewport_id()));
    let visible = 6usize;
    let body = r(o, 19.0, 299.0, 492.0, 100.0);
    let first = first_row(ui, key, body, conns.len(), visible);
    let clip = p.with_clip_rect(body);
    for (k, c) in conns.iter().skip(first).take(visible + 1).enumerate() {
        let top = o.y + 299.0 + 17.0 * k as f32;
        // Its room starts at the list's edge, not 6 px into it as a window's text does.
        winui::text_right_within(
            &clip,
            o.x + 33.0 + dx,
            top + 12.0,
            &(c.id + 1).to_string(),
            winui::TEXT(),
            o.x + 21.0,
        );
        text(
            &clip,
            o.x + 56.0 + dx,
            top + 12.0,
            &size3(c.downloaded),
            winui::TEXT(),
        );
        let info = match &c.status {
            ConnStatus::Connecting => crate::i18n::t("Connecting...").to_owned(),
            ConnStatus::Receiving => crate::i18n::t("Receiving data...").to_owned(),
            ConnStatus::Retrying(e) => crate::i18n::fmt(
                // i18n: a connection's state: it failed and tries again, with the reason
                crate::i18n::t("Retrying: {error}"),
                &[("error", &crate::errors::describe(e))],
            ),
            ConnStatus::Finished => String::new(),
            ConnStatus::Failed(e) => crate::i18n::fmt(
                crate::i18n::t("Error: {error}"),
                &[("error", &crate::errors::describe(e))],
            ),
            ConnStatus::Retired { host, cap } => crate::i18n::fmt(
                // i18n: a connection closed because the server allows only that many
                &crate::i18n::tn(
                    u64::from(*cap),
                    "{host} allows {n} connection",
                    "{host} allows {n} connections",
                ),
                &[("host", &crate::i18n::ltr(host))],
            ),
        };
        // The Info column ends at the scrollbar.
        let (rect, cut) = winui::text_fit(
            &clip,
            o.x + 155.0 + dx,
            top + 12.0,
            &info,
            winui::TEXT(),
            o.x + 507.0,
        );
        if cut {
            winui::full_on_hover(ui, rect.intersect(body), ("conn", c.id), &info);
        }
    }
    // The shared scroll bar, the main list's.
    rows_bar(
        ui,
        key,
        r(o, 511.0, 275.0, 17.0, 124.0),
        conns.len(),
        visible,
        first,
    );
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

    /// #1101: the number column keeps IDM's width while its numbers and title fit (one or two
    /// digits, 6 px each in the classic look), and widens by what a wider one needs.
    #[test]
    fn the_number_column_fits_its_widest_number() {
        assert_eq!(number_column_extra(6.0, 11.0), 0.0, "1 to 9, \"N.\"");
        assert_eq!(number_column_extra(12.0, 11.0), 0.0, "10 to 99");
        assert_eq!(number_column_extra(18.0, 11.0), 5.0, "100 to 128");
        assert_eq!(number_column_extra(14.5, 11.0), 2.0, "wider digits");
        assert_eq!(number_column_extra(6.0, 31.0), 6.0, "a longer title");
        assert_eq!(
            number_column_extra(500.0, 11.0),
            60.0,
            "never past the Info column"
        );
    }

    /// A download of unknown size has no percentage in its title (from #384).
    #[test]
    fn the_title_has_a_percentage_only_with_a_size() {
        assert!(!window_title(None, "clip.mp4").contains('%'));
        assert!(window_title(Some(0.445), "clip.mp4").starts_with("44% "));
    }

    /// Every start tick of the connection bar is the same whole number of pixels wide wherever
    /// it falls: 1 at 100 and 125 %, 2 at 150 and 200 % (at 150 % it was 1 or 2 by its place).
    #[test]
    fn start_ticks_are_equally_wide_at_any_scale() {
        for (ppp, want) in [(1.0, 1.0), (1.25, 1.0), (1.5, 2.0), (1.75, 2.0), (2.0, 2.0)] {
            for k in 0..64 {
                let x = 17.5 + k as f32 * 63.875;
                let (a, b) = tick_span(x, ppp);
                let pixels = (b * ppp).round() - (a * ppp).round();
                assert_eq!(pixels, want, "x {x} at {ppp}");
                assert!(
                    ((a * ppp) - (a * ppp).round()).abs() < 1e-3,
                    "on a pixel edge"
                );
            }
        }
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

    #[test]
    fn hide_tab_takes_the_tab_off_the_strip() {
        let mut p = Progress::new(1, true);
        p.tab = Tab::Limiter;
        p.hide(Tab::Limiter);
        assert_eq!(p.tab, Tab::Status, "back on Download status");
        assert_eq!(p.hidden_tabs, [Tab::Limiter], "Speed Limiter off the strip");
        p.hide(Tab::Limiter);
        p.hide(Tab::Status);
        assert_eq!(
            p.hidden_tabs,
            [Tab::Limiter],
            "once each, never Download status"
        );
        p.hide(Tab::Completion);
        assert_eq!(p.hidden_tabs, [Tab::Limiter, Tab::Completion]);
        p.show_tab("limit");
        assert_eq!(p.tab, Tab::Limiter, "asked for, it comes back");
        assert_eq!(p.hidden_tabs, [Tab::Completion], "the other stays hidden");
    }

    #[test]
    fn show_hidden_tabs_brings_them_all_back() {
        let mut p = Progress::new(1, true);
        p.hide(Tab::Limiter);
        p.hide(Tab::Completion);
        p.show_hidden_tabs();
        assert!(p.hidden_tabs.is_empty(), "every tab back on the strip");
        assert_eq!(p.tab, Tab::Status, "the page shown stays as it was");
    }

    /// The General tab's sharing banner says the rate sent and to how many peers, counted.
    #[test]
    fn sharing_line_says_the_rate_and_the_peers() {
        let mut s = mdm_torrent::Stats {
            phase: mdm_torrent::Phase::Finished,
            checked: None,
            done: 100,
            total: 100,
            uploaded: 0,
            fetched: 0,
            down_speed: 0.0,
            up_speed: 410.0 * 1024.0,
            file_done: Vec::new(),
            peers_live: 6,
            peers_seen: 38,
            error: None,
        };
        let line = sharing_line(&s);
        assert!(line.contains("6 peers") && line.contains("410"), "{line}");
        s.peers_live = 1;
        let line = sharing_line(&s);
        assert!(line.contains("1 peer") && !line.contains("peers"), "{line}");
    }

    /// `/show progress:error` and `progress:menu` (plan 09a's screenshots) are looks only: the
    /// failed state is a sample reason on Download status, the menu a flag the next frame takes.
    #[test]
    fn show_states_are_looks_only() {
        let mut p = Progress::new(1, true);
        p.tab = Tab::Limiter;
        p.show_tab("error");
        assert_eq!(p.tab, Tab::Status);
        assert!(p.sample_error.as_deref().is_some_and(|e| e.contains("404")));
        assert!(!p.menu_asked);
        p.show_tab("menu");
        assert!(p.menu_asked);
        assert_eq!(p.tab, Tab::Status, "the menu leaves the page as it is");
        p.show_tab("status");
        assert!(
            p.sample_error.is_none(),
            "a tab by name shows the download again"
        );
        // A video's states (upstream issues 769, 736): looks only, gone with a tab by name.
        p.tab = Tab::Limiter;
        p.show_tab("merging");
        assert_eq!(p.tab, Tab::Status);
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

    /// A live stream being recorded: its Pause button stops and saves it; while it is saved
    /// the button waits; every other download's is Pause or Resume as before. Only a live
    /// stream's window lays the button out for "Stop and save".
    #[test]
    fn a_recording_live_streams_pause_button_stops_and_saves_it() {
        use mdm_tools::video::LiveState;
        let label = |status, live| pause_button(status, live).0;
        let enabled = |status, live| pause_button(status, live).1 != ButtonKind::Disabled;
        let recording = Some(LiveState::Recording);
        let saving = Some(LiveState::Saving);
        assert_eq!(label(Status::Downloading, recording), "Stop and save");
        assert!(enabled(Status::Downloading, recording));
        assert_eq!(label(Status::Downloading, saving), "Stop and save");
        assert!(!enabled(Status::Downloading, saving));
        assert_eq!(label(Status::Downloading, None), "Pause");
        assert!(enabled(Status::Downloading, None));
        for status in [Status::Stopped, Status::Error] {
            assert_eq!(label(status, None), "Resume");
            assert!(enabled(status, None));
        }
        for status in [Status::Queued, Status::Complete] {
            assert_eq!(label(status, None), "Pause");
            assert!(!enabled(status, None));
        }
        for status in [
            Status::Downloading,
            Status::Stopped,
            Status::Error,
            Status::Queued,
        ] {
            for live in [None, recording, saving] {
                assert!(pause_labels(true).contains(&label(status, live)));
            }
            assert!(pause_labels(false).contains(&label(status, None)));
        }
        assert!(!pause_labels(false).contains(&"Stop and save"));
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
    use super::{GLIDE_HOLD, GLIDE_MAX, Glide, glide_frame};

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
        // 100 MB at 10 MB/s: 51 pixels a second, at the cap.
        assert_eq!(
            glide_frame(10.0 * mb, total, 1.0, false),
            Some(Duration::from_millis(40))
        );
        assert_eq!(
            glide_frame(10.0 * mb, total, 1.0, true),
            Some(Duration::from_millis(80))
        );
        // At 2 MB/s: 10 pixels a second (15 at 150 %).
        let every = glide_frame(2.0 * mb, total, 1.0, false).unwrap();
        assert!((every.as_secs_f64() - 1.0 / (0.02 * 511.0)).abs() < 1e-3);
        assert_eq!(glide_frame(2.0 * mb, total, 1.0, true), Some(every));
        let every = glide_frame(2.0 * mb, total, 1.5, false).unwrap();
        assert!((every.as_secs_f64() - 1.0 / (0.03 * 511.0)).abs() < 1e-3);
        // At 0.5 MB/s: 2.6 pixels a second, the app's 4 frames do.
        for main in [false, true] {
            assert_eq!(glide_frame(0.5 * mb, total, 1.0, main), None);
            assert_eq!(glide_frame(0.0, total, 1.0, main), None);
            assert_eq!(glide_frame(10.0 * mb, None, 1.0, main), None);
            assert_eq!(glide_frame(10.0 * mb, Some(0), 1.0, main), None);
        }
    }
}
