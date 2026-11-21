//! The first start's offer of the video helpers (user request 2026-11-01): yt-dlp finds a page's
//! video, ffmpeg joins its picture and sound. Both are asked for once, explained, and downloaded
//! on Yes (through MDM's own engine, checked against their publishers' signed checksums: see
//! mdm_tools); Not now leaves them to Tasks → Download video from a page, which offers them again.

use std::sync::Arc;
use std::sync::Mutex;

use egui::{Pos2, Ui};

use crate::dialogs::{Outcome, window};
use crate::icons::Icon;
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, r};

/// The offer's footer: this far under its last rule, after the line of the way back later.
const OFFER_FOOTER: f32 = 30.0;
/// Helpers' width, and where its footer starts (under the error line).
const PROGRAMS_W: f32 = 500.0;
const PROGRAMS_BODY: f32 = 364.0;
/// Each card's step down Helpers (a card is 79 high).
const CARD_STEP: f32 = 84.0;

/// The helpers' download, one after the other (user decision 2026-11-01: one at a time, one
/// progress bar for both).
#[derive(Default)]
struct State {
    /// The helper downloading now: its name, its place (1-based) and how many there are.
    current: Option<(&'static str, usize, usize)>,
    /// The helpers done: name and version.
    ready: Vec<(&'static str, String)>,
    /// The helper that failed, why, and what to do about it (upstream issue 816; the ones after
    /// it are not tried).
    error: Option<(&'static str, String, Option<String>)>,
    done: bool,
}

struct Job {
    /// The current helper's bytes.
    progress: Arc<mdm_tools::update::Progress>,
    state: Arc<Mutex<State>>,
}

pub struct Helpers {
    dir: Option<std::path::PathBuf>,
    /// Once Yes was pressed.
    job: Option<Job>,
}

/// Whether to offer the helpers at this start: never asked, and one of them is missing.
pub fn worth_asking() -> bool {
    let Some(dir) = mdm_tools::tools_dir() else {
        return false;
    };
    mdm_tools::installed(&dir).is_none() || mdm_tools::ffmpeg::installed(&dir).is_none()
}

impl Helpers {
    pub fn new() -> Self {
        Helpers {
            dir: mdm_tools::tools_dir(),
            job: None,
        }
    }

    /// Download the missing helpers one after the other, in the background: they go on when
    /// the window is closed. Try again starts over with the ones still missing.
    fn start(&mut self, rt: &tokio::runtime::Handle, ctx: &egui::Context) {
        let Some(dir) = self.dir.clone() else {
            return;
        };
        let mut list = Vec::new();
        if mdm_tools::installed(&dir).is_none() {
            list.push("yt-dlp");
        }
        if mdm_tools::ffmpeg::installed(&dir).is_none() {
            list.push("ffmpeg");
        }
        let job = Job {
            progress: Arc::default(),
            state: Arc::default(),
        };
        let (progress, state, ctx) = (job.progress.clone(), job.state.clone(), ctx.clone());
        let net = crate::settings::own_net();
        rt.spawn(async move {
            let total = list.len();
            for (k, name) in list.into_iter().enumerate() {
                {
                    use std::sync::atomic::Ordering::Relaxed;
                    progress.got.store(0, Relaxed);
                    progress.total.store(0, Relaxed);
                }
                state.lock().unwrap().current = Some((name, k + 1, total));
                let res = if name == "ffmpeg" {
                    mdm_tools::ffmpeg::install_latest(&dir, &net, Some(&progress)).await
                } else {
                    mdm_tools::install_latest(&dir, &net, Some(&progress)).await
                };
                let mut s = state.lock().unwrap();
                match res {
                    Ok(installed) => s.ready.push((name, installed.version)),
                    Err(e) => {
                        tracing::warn!("helper download failed: {name}: {e}");
                        s.error = Some((
                            name,
                            crate::errors::tools(&e),
                            crate::errors::tools_advice(&e, name),
                        ));
                        break;
                    }
                }
            }
            let mut s = state.lock().unwrap();
            s.current = None;
            s.done = true;
            drop(s);
            ctx.request_repaint();
        });
        self.job = Some(job);
    }

    /// Open while shown (after Yes too, with the downloads' progress); Closed on Not now, Close
    /// or the window's X.
    pub fn show(&mut self, ctx: &egui::Context, rt: &tokio::runtime::Handle) -> Outcome<()> {
        let (running, failed) = match &self.job {
            Some(job) => {
                let s = job.state.lock().unwrap();
                (!s.done, s.error.is_some())
            }
            None => (false, false),
        };
        if running {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
        let mut answer = None;
        // A long translation's subline makes the banner taller (winui::banner).
        // A failure and its advice, wrapped, make it taller too (upstream issue 816).
        let extra = crate::dialogs::grow_extra(ctx, "helpers-banner")
            + crate::dialogs::grow_extra(ctx, "helpers-error")
            + crate::dialogs::grow_extra(ctx, "helpers-advice");
        window(
            ctx,
            "helpers",
            // i18n: title of the first start's window that offers yt-dlp and ffmpeg
            "Get ready for video downloads",
            // Taller once Yes is pressed: the progress under the last section.
            [
                520.0,
                if self.job.is_some() { 358.0 } else { 322.0 }
                    + OFFER_FOOTER
                    + winui::FOOTER_H
                    + extra,
            ],
            false,
            winui::BG(),
            |ui, o| {
                self.body(ui, o, running, failed, &mut answer);
                match answer.take() {
                    Some(true) => {
                        self.start(rt, ui.ctx());
                        Outcome::Open
                    }
                    Some(false) => Outcome::Closed,
                    None if winui::escape_pressed(ui) => Outcome::Closed,
                    None => Outcome::Open,
                }
            },
        )
    }

    fn body(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        running: bool,
        failed: bool,
        answer: &mut Option<bool>,
    ) {
        let p = ui.painter().clone();
        let (ink, muted) = (winui::TEXT(), winui::MUTED());
        let w = 520.0;
        // The banner: what this is for, in one line, and why now.
        let extra = winui::banner(
            ui,
            &p,
            o,
            w,
            "helpers-banner",
            // i18n: the headline of the first start's helpers window
            crate::i18n::t("Download videos from YouTube and 1,000+ sites"),
            crate::i18n::t(
                "Two free helpers make it work. Get them now, and saving a video is one click away when you find one.",
            ),
        );
        // Everything below it moves down with a taller banner.
        let o = o + egui::vec2(0.0, extra);

        let rows: [(Icon, &str, String); 3] = [
            (
                Icon::HelperVideo,
                // i18n: helper name and what it does (yt-dlp is a program's name)
                "yt-dlp: finds the video",
                crate::i18n::t(
                    "Reads a page and lists every quality the site offers, up to 4K, from YouTube, Vimeo and over a thousand other sites.",
                )
                .to_owned(),
            ),
            (
                Icon::HelperMerge,
                // i18n: helper name and what it does (ffmpeg is a program's name)
                "ffmpeg: joins picture and sound",
                crate::i18n::t(
                    "Sites send their best quality as separate video and audio. ffmpeg puts them together into one file that plays anywhere.",
                )
                .to_owned(),
            ),
            (
                Icon::HelperChecked,
                // i18n: a promise in the helpers window: the helpers are free, official and checked
                "Free, official and checked",
                crate::i18n::t(
                    "Both are open source. Grabbit gets them from their official releases with its own downloader and checks each file's signature before it runs. About 100 MB, once.",
                )
                .to_owned(),
            ),
        ];
        for (k, (icon, name, what)) in rows.iter().enumerate() {
            let y = 100.0 + k as f32 * 72.0;
            icon.image(true)
                .paint_at(ui, r(o, 22.0, y + 2.0, 32.0, 32.0));
            winui::text_font(
                &p,
                winui::tahoma_bold(),
                o.x + 68.0,
                o.y + y + 11.0,
                crate::i18n::t(name),
                ink,
            );
            winui::paragraph(
                &p,
                o.x + 68.0,
                o.y + y + 29.0,
                what,
                muted,
                o.x + w - 18.0,
                14.0,
            );
        }

        // After Yes: one bar for both, centred under the last section, and what it is doing.
        let mut y = 322.0;
        if let Some(job) = &self.job {
            let st = job.state.lock().unwrap();
            let f = job.progress.fraction().unwrap_or(0.0);
            let (fraction, line, color) = if let Some((name, error, _)) = &st.error {
                (
                    st.ready.len() as f32 / (st.ready.len() + 1) as f32,
                    crate::i18n::fmt(
                        // i18n: {name} = yt-dlp or ffmpeg; {error} = why
                        crate::i18n::t("{name} could not be downloaded: {error}"),
                        &[("name", name), ("error", error)],
                    ),
                    winui::ERROR(),
                )
            } else if st.done {
                let list = st
                    .ready
                    .iter()
                    .map(|(n, v)| format!("{n} {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    1.0,
                    crate::i18n::fmt(
                        // i18n: the helpers are downloaded, checked and ready; {list} = names and versions, e.g. "yt-dlp 2026.10.26, ffmpeg 9.0"
                        crate::i18n::t("Ready: {list}"),
                        &[("list", &list)],
                    ),
                    winui::SUCCESS(),
                )
            } else if let Some((name, k, total)) = st.current
                && f >= 1.0
            {
                (
                    k as f32 / total as f32,
                    crate::i18n::fmt(
                        // i18n: a downloaded helper's signature and checksum are being checked; {name} = yt-dlp or ffmpeg
                        crate::i18n::t("Checking {name} ({n} of {total})..."),
                        &[
                            ("name", name),
                            ("n", &crate::i18n::integer(k as u64)),
                            ("total", &crate::i18n::integer(total as u64)),
                        ],
                    ),
                    ink,
                )
            } else if let Some((name, k, total)) = st.current {
                (
                    ((k - 1) as f32 + f) / total as f32,
                    crate::i18n::fmt(
                        // i18n: {name} = yt-dlp or ffmpeg; {n} of {total} = which helper of how many; {percent} = how far, e.g. 45%
                        crate::i18n::t("Downloading {name} ({n} of {total})... {percent}"),
                        &[
                            ("name", name),
                            ("n", &crate::i18n::integer(k as u64)),
                            ("total", &crate::i18n::integer(total as u64)),
                            ("percent", &format!("{}%", (f * 100.0).floor())),
                        ],
                    ),
                    ink,
                )
            } else {
                (0.0, crate::i18n::t("Starting...").to_owned(), ink)
            };
            drop(st);
            let bar_w = 320.0;
            winui::meter(&p, r(o, (w - bar_w) / 2.0, 312.0, bar_w, 8.0), fraction);
            let room = w - 40.0;
            let mut more = 0.0;
            if failed && winui::text_width(&p, &line) > room {
                // A failure is read whole (upstream issue 816): wrapped, the window growing by
                // its lines past the first.
                let n = winui::paragraph(
                    &p,
                    o.x + 20.0,
                    o.y + 340.0,
                    &line,
                    color,
                    o.x + w - 20.0,
                    16.0,
                );
                more = 16.0 * n.saturating_sub(1) as f32;
            } else {
                // Centred on screen (the same place in both directions); cut to the window.
                let shown = winui::ellipsize(&p, &winui::tahoma(), &line, room);
                let tw = winui::text_width(&p, &shown);
                let rc = winui::text_at(
                    &p,
                    winui::tahoma(),
                    (o.x + (w - tw) / 2.0).round(),
                    o.y + 340.0,
                    &shown,
                    color,
                );
                if shown != line {
                    winui::full_on_hover(ui, rc, "helpers-status", &line);
                }
            }
            crate::dialogs::grow_px(ui.ctx(), "helpers-error", more);
            y += 36.0 + more;
        }

        // The way back later, then the answer.
        winui::rule(&p, o, 16.0, y, w - 32.0);
        // What to do about the failure (upstream issue 816), when it says.
        let advice = self.job.as_ref().and_then(|job| {
            let st = job.state.lock().unwrap();
            st.error.as_ref().and_then(|(_, _, a)| a.clone())
        });
        let later = if self.job.is_some() && running {
            crate::i18n::t("They keep downloading if you close this window.")
        } else if self.job.is_some() && !failed {
            crate::i18n::t(
                "All set: on a page with a video, use Tasks → Download video from a page.",
            )
        } else if let Some(advice) = &advice {
            advice.as_str()
        } else if self.job.is_some() {
            crate::i18n::t(
                "Check your connection, then try again, or get them later from the video window.",
            )
        } else {
            crate::i18n::t("Not now? Get them later from Tasks → Download video from a page.")
        };
        // Wrapped: the window grows by the lines past the first.
        let lines = winui::paragraph(
            &p,
            o.x + 16.0,
            o.y + y + 20.0,
            later,
            muted,
            o.x + w - 16.0,
            16.0,
        );
        let more = 16.0 * lines.saturating_sub(1) as f32;
        crate::dialogs::grow_px(ui.ctx(), "helpers-advice", more);
        // The footer under it ([`OFFER_FOOTER`]); Escape is the window's (show).
        let top = y + OFFER_FOOTER + more;
        if self.job.is_none() {
            // Enter Yes, as before.
            let pressed = winui::footer(
                ui,
                o,
                w,
                top,
                &mut [
                    FooterItem::Button(FooterButton::new("Not now", ButtonKind::Normal)),
                    // i18n: the first start's helpers window: download yt-dlp and ffmpeg (the recommended answer)
                    FooterItem::Button(
                        FooterButton::new("Yes, download them (recommended)", ButtonKind::Accent)
                            .enter(true),
                    ),
                ],
            );
            match pressed {
                Some(1) => *answer = Some(true),
                Some(0) => *answer = Some(false),
                _ => {}
            }
        } else {
            // Try again once it failed; Enter Close, as before.
            let again = failed && !running;
            let mut items = Vec::new();
            if again {
                items.push(FooterItem::Button(FooterButton::new(
                    "Try again",
                    ButtonKind::Normal,
                )));
            }
            items.push(FooterItem::Button(
                FooterButton::new("Close", ButtonKind::Default).enter(true),
            ));
            match winui::footer(ui, o, w, top, &mut items) {
                Some(0) if again => *answer = Some(true),
                Some(_) => *answer = Some(false),
                None => {}
            }
        }
    }
}

// ---- Video helpers: MDM's own copies or the person's (#51) ------------------------------------

use mdm_tools::chosen::Helper;

/// Helpers (Download video from a page or a gallery → Helpers...): for yt-dlp, ffmpeg and
/// gallery-dl (#58), where MDM takes it from (its own copy, or a program of the person's), and the
/// way to choose another: one copy of each for every program that uses them (#51). Then the
/// JavaScript runtime yt-dlp needs for YouTube (#1448, #1470): the computer's own (Node.js, Deno),
/// else Deno, MDM's own copy, downloaded and checked here.
pub struct Programs {
    dir: Option<std::path::PathBuf>,
    /// The helpers as found (read again after each change, not every frame: it reads files).
    found: [Option<mdm_tools::Installed>; 3],
    /// MDM's own Deno, read with `found`.
    deno: Option<mdm_tools::Installed>,
    /// The computer's own runtime (`proc::system_js_runtime`), and the look for it under way,
    /// off the window's thread (the PATH's folders may be on a share that doesn't answer).
    runtime: Option<std::path::PathBuf>,
    finding: Option<Finding>,
    /// A chosen program being checked (`--version`), and what it answered.
    checking: Option<(Helper, Answer)>,
    /// MDM's own Deno being downloaded, and how that went.
    installing_deno: Option<Answer>,
    /// A download of Deno started elsewhere (Download video, the session's update) was under
    /// way at the last frame: the card is read again once it ends.
    elsewhere: bool,
    error: Option<String>,
}

/// The computer's own JavaScript runtime, once looked for.
type Finding = Arc<Mutex<Option<Option<std::path::PathBuf>>>>;

const HELPERS: [Helper; 3] = [Helper::YtDlp, Helper::Ffmpeg, Helper::GalleryDl];

/// A check's answer, once it has one (Err: why the program wasn't taken, in words).
type Answer = Arc<Mutex<Option<Result<(), String>>>>;

impl Programs {
    pub fn new() -> Self {
        let mut p = Programs {
            dir: mdm_tools::tools_dir(),
            found: [None, None, None],
            deno: None,
            runtime: None,
            finding: None,
            checking: None,
            installing_deno: None,
            elsewhere: false,
            error: None,
        };
        p.look();
        p
    }

    fn look(&mut self) {
        let Some(dir) = self.dir.as_deref() else {
            return;
        };
        self.found = [
            mdm_tools::installed(dir),
            mdm_tools::ffmpeg::installed(dir),
            mdm_tools::gallery::installed(dir),
        ];
        self.deno = mdm_tools::deno::installed(dir);
        if self.finding.is_none() {
            let slot: Finding = Arc::new(Mutex::new(None));
            let out = slot.clone();
            std::thread::spawn(move || {
                *out.lock().unwrap() = Some(mdm_tools::proc::system_js_runtime());
            });
            self.finding = Some(slot);
        }
    }

    /// MDM's own Deno (#1448): downloaded and checked against its release's checksums, from then
    /// on the runtime yt-dlp is told of (`mdm_tools::proc`). It goes on if the window is closed.
    fn install_deno(
        &mut self,
        dir: std::path::PathBuf,
        rt: &tokio::runtime::Handle,
        ctx: &egui::Context,
    ) {
        let slot: Answer = Arc::new(Mutex::new(None));
        let (out, wake) = (slot.clone(), ctx.clone());
        let net = crate::settings::own_net();
        rt.spawn(async move {
            let res = mdm_tools::deno::install_latest(&dir, &net, None).await;
            if let Err(e) = &res {
                tracing::warn!("Deno download failed: {e}");
            }
            *out.lock().unwrap() = Some(res.map(|_| ()).map_err(|e| crate::errors::tools(&e)));
            wake.request_repaint();
        });
        self.installing_deno = Some(slot);
        self.error = None;
    }

    /// A check or a download is under way: the buttons wait.
    fn busy(&self) -> bool {
        self.checking.is_some() || self.installing_deno.is_some()
    }

    /// Open until Close.
    pub fn show(&mut self, ctx: &egui::Context, rt: &tokio::runtime::Handle) -> Outcome<()> {
        let answered = self
            .checking
            .as_ref()
            .and_then(|(_, slot)| slot.lock().unwrap().take());
        if let Some(res) = answered {
            self.error = res.err();
            self.checking = None;
            self.look();
        }
        let deno = self
            .installing_deno
            .as_ref()
            .and_then(|slot| slot.lock().unwrap().take());
        if let Some(res) = deno {
            self.error = res.err();
            self.installing_deno = None;
            self.look();
        }
        // A download of Deno started elsewhere: read again once it ends.
        let elsewhere = self.installing_deno.is_none() && mdm_tools::deno::downloading();
        if self.elsewhere && !elsewhere {
            self.look();
        }
        self.elsewhere = elsewhere;
        let found = self
            .finding
            .as_ref()
            .and_then(|slot| slot.lock().unwrap().take());
        if let Some(found) = found {
            self.runtime = found;
            self.finding = None;
        }
        // Nothing wakes the window when these end: it looks again now and then meanwhile.
        if elsewhere || self.finding.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        for h in HELPERS {
            if let Some(path) = winui::picked(ctx, &format!("helper-{}", h.name()))
                && let Some(dir) = self.dir.clone()
            {
                let slot = Arc::new(Mutex::new(None));
                let (out, wake) = (slot.clone(), ctx.clone());
                std::thread::spawn(move || {
                    let res = mdm_tools::chosen::choose(&dir, h, &path)
                        .map(|_| ())
                        .map_err(|e| crate::errors::tools(&e));
                    *out.lock().unwrap() = Some(res);
                    wake.request_repaint();
                });
                self.checking = Some((h, slot));
                self.error = None;
            }
        }
        window(
            ctx,
            "video-helpers",
            // i18n: window title: where MDM takes its helper programs (yt-dlp, ffmpeg, gallery-dl) from
            "Helpers",
            [PROGRAMS_W, PROGRAMS_BODY + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| self.body(ui, o, rt),
        )
    }

    fn body(&mut self, ui: &mut Ui, o: Pos2, rt: &tokio::runtime::Handle) -> Outcome<()> {
        let p = ui.painter().clone();
        for (k, h) in HELPERS.into_iter().enumerate() {
            let y = o.y + 8.0 + k as f32 * CARD_STEP;
            card(&p, o, y);
            winui::text_font(
                &p,
                winui::tahoma_bold(),
                o.x + 20.0,
                y + 16.0,
                h.name(),
                winui::TEXT(),
            );
            let found = self.found[k].as_ref();
            let chosen = self
                .dir
                .as_deref()
                .and_then(|d| mdm_tools::chosen::chosen(d, h));
            // One of the person's, maybe passed over (a gallery-dl too old, #236).
            let mine = chosen.is_some();
            let too_old = chosen
                .as_ref()
                .filter(|c| h == Helper::GalleryDl && mdm_tools::gallery::too_old(&c.version));
            let checking = self.checking.as_ref().is_some_and(|(c, _)| *c == h);
            let status = match (too_old, found) {
                _ if checking => {
                    // i18n: Video helpers: a program just chosen is being checked
                    crate::i18n::t("Checking the program...").to_owned()
                }
                (Some(old), _) => crate::i18n::fmt(
                    // i18n: Video helpers: the person's own gallery-dl is too old and not used; {version} = its version, {oldest} = the oldest MDM can use
                    crate::i18n::t(
                        "Your own program ({version}) is too old: Grabbit needs {oldest} or newer",
                    ),
                    &[
                        ("version", &old.version),
                        ("oldest", mdm_tools::gallery::OLDEST),
                    ],
                ),
                (None, Some(i)) if mine => crate::i18n::fmt(
                    // i18n: Video helpers: {path} = the program the person chose, {version} = its version
                    crate::i18n::t("Your own program: {path} ({version})"),
                    &[
                        ("path", &crate::i18n::ltr(&i.exe.display().to_string())),
                        ("version", &i.version),
                    ],
                ),
                (None, Some(i)) => crate::i18n::fmt(
                    // i18n: Video helpers: {version} = the version of MDM's own copy
                    crate::i18n::t("Grabbit's own copy, version {version}"),
                    &[("version", &i.version)],
                ),
                // i18n: Video helpers: no copy of this helper yet
                (None, None) => crate::i18n::t("Not installed yet").to_owned(),
            };
            winui::fit(
                ui,
                &p,
                o.x + 20.0,
                y + 36.0,
                &status,
                winui::MUTED(),
                o.x + PROGRAMS_W - 20.0,
                ("helper", k),
            );
            let busy = self.busy();
            // As wide as their labels in this language (#192), side by side from the left.
            let [other_at, own_at] = winui::button_row_from_left(
                &p,
                &[
                    ("Use another program...", 20.0, 180.0),
                    ("Use Grabbit's own copy", 208.0, 180.0),
                ],
                8.0,
                PROGRAMS_W - 20.0,
                260.0,
            )[..] else {
                unreachable!("two buttons")
            };
            // Every helper's buttons read the same: a screen reader hears which program they are for.
            let spoken = |resp: &egui::Response, label: &str| {
                let name = crate::i18n::fmt(
                    // i18n: what a screen reader calls a button that one row of a window has for each thing it lists; {label} is the button's text, {what} the row's browser or program
                    crate::i18n::t("{label} for {what}"),
                    &[
                        ("label", crate::i18n::t(label)),
                        ("what", &crate::i18n::ltr(h.name())),
                    ],
                );
                resp.ctx
                    .accesskit_node_builder(resp.id, |n| n.set_label(name));
            };
            let other = winui::button(
                ui,
                r(o, other_at.0, y - o.y + 46.0, other_at.1, 21.0),
                // i18n: Video helpers: button: pick a program (yt-dlp or ffmpeg) already on the computer
                "Use another program...",
                if busy {
                    ButtonKind::Disabled
                } else {
                    ButtonKind::Normal
                },
                false,
                None,
            );
            spoken(&other, "Use another program...");
            if other.clicked() {
                // i18n: file picker title; {name} = yt-dlp or ffmpeg
                let title =
                    crate::i18n::fmt(crate::i18n::t("Choose {name}"), &[("name", h.name())]);
                winui::start_pick(ui.ctx(), &format!("helper-{}", h.name()), move || {
                    rfd::FileDialog::new().set_title(title).pick_file()
                });
            }
            let own = winui::button(
                ui,
                r(o, own_at.0, y - o.y + 46.0, own_at.1, 21.0),
                // i18n: Video helpers: button: go back to the copy MDM downloads and checks itself
                "Use Grabbit's own copy",
                if mine && !busy {
                    ButtonKind::Normal
                } else {
                    ButtonKind::Disabled
                },
                false,
                None,
            );
            spoken(&own, "Use Grabbit's own copy");
            if own.clicked()
                && let Some(dir) = self.dir.clone()
            {
                self.error = mdm_tools::chosen::forget(&dir, h)
                    .err()
                    .map(|e| crate::errors::tools(&e));
                self.look();
            }
        }
        self.runtime_row(ui, &p, o, HELPERS.len(), rt);
        if let Some(e) = &self.error {
            winui::fit(
                ui,
                &p,
                // In the cards' text column, under the last card (it ends at 339).
                o.x + 20.0,
                o.y + 356.0,
                e,
                winui::ERROR(),
                o.x + 486.0,
                "helper-error",
            );
        }
        // Close on Enter and on Escape, as before.
        match winui::footer(
            ui,
            o,
            PROGRAMS_W,
            PROGRAMS_BODY,
            &mut [FooterItem::Button(
                FooterButton::new("Close", ButtonKind::Default)
                    .enter(true)
                    .escape(true),
            )],
        ) {
            Some(_) => Outcome::Done(()),
            None => Outcome::Open,
        }
    }

    /// The JavaScript runtime's card, the `k`th (#1448, #1470): YouTube makes yt-dlp solve a
    /// JavaScript challenge, and without a runtime its look-ups are slow, list few qualities or
    /// fail. The computer's own (Node.js, Deno) is said and used; else MDM's own Deno, which
    /// "Use Grabbit's own copy" downloads (where Deno builds one for this processor).
    fn runtime_row(
        &mut self,
        ui: &mut Ui,
        p: &egui::Painter,
        o: Pos2,
        k: usize,
        rt: &tokio::runtime::Handle,
    ) {
        let y = o.y + 8.0 + k as f32 * CARD_STEP;
        card(p, o, y);
        // i18n: Video helpers: the row of the JavaScript runtime yt-dlp needs for YouTube (Deno is a program's name)
        let title = crate::i18n::t("JavaScript runtime (Deno)");
        winui::text_font(
            p,
            winui::tahoma_bold(),
            o.x + 20.0,
            y + 16.0,
            title,
            winui::TEXT(),
        );
        // What it is for, in one line after its name.
        let after = winui::text_width_font(p, winui::tahoma_bold(), title) + 8.0;
        winui::fit(
            ui,
            p,
            o.x + 20.0 + after,
            y + 16.0,
            // i18n: Video helpers: what the JavaScript runtime is for, after its name (yt-dlp is a program's name)
            crate::i18n::t("lets yt-dlp list all of YouTube's qualities"),
            winui::MUTED(),
            o.x + PROGRAMS_W - 20.0,
            ("helper-what", k),
        );
        let available = mdm_tools::deno::available();
        // One download of Deno at a time: one started from Download video shows here too.
        let downloading = self.installing_deno.is_some() || mdm_tools::deno::downloading();
        let status = if downloading {
            // i18n: Video helpers: MDM's own copy of a helper is being downloaded
            crate::i18n::t("Downloading Grabbit's own copy...").to_owned()
        } else if let Some(own) = &self.deno {
            crate::i18n::fmt(
                crate::i18n::t("Grabbit's own copy, version {version}"),
                &[("version", &own.version)],
            )
        } else if self.finding.is_some() {
            // i18n: Video helpers: looking for a JavaScript runtime (Node.js, Deno) on this computer
            crate::i18n::t("Looking for one on this computer...").to_owned()
        } else if let Some(path) = &self.runtime {
            crate::i18n::fmt(
                // i18n: Video helpers: {path} = the computer's own JavaScript runtime (node.exe, deno.exe), used as it is: MDM needn't download one
                crate::i18n::t("Your system's, nothing to download: {path}"),
                &[("path", &crate::i18n::ltr(&path.display().to_string()))],
            )
        } else if available {
            crate::i18n::t("Not installed yet").to_owned()
        } else {
            // i18n: Video helpers: MDM can't download Deno on this system (32-bit Windows, an unusual processor); Deno and Node.js are program names
            crate::i18n::t("Grabbit has no copy for this system: install Deno or Node.js")
                .to_owned()
        };
        winui::fit(
            ui,
            p,
            o.x + 20.0,
            y + 36.0,
            &status,
            winui::MUTED(),
            o.x + PROGRAMS_W - 20.0,
            ("helper", k),
        );
        let [own_at] = winui::button_row_from_left(
            p,
            &[("Use Grabbit's own copy", 20.0, 180.0)],
            8.0,
            PROGRAMS_W - 20.0,
            260.0,
        )[..] else {
            unreachable!("one button")
        };
        let own = winui::button(
            ui,
            r(o, own_at.0, y - o.y + 46.0, own_at.1, 21.0),
            "Use Grabbit's own copy",
            if available && self.deno.is_none() && !self.busy() && !downloading {
                ButtonKind::Normal
            } else {
                ButtonKind::Disabled
            },
            false,
            None,
        );
        // A screen reader hears which program it is for, as the helpers' buttons say theirs.
        let name = crate::i18n::fmt(
            crate::i18n::t("{label} for {what}"),
            &[
                ("label", crate::i18n::t("Use Grabbit's own copy")),
                ("what", &crate::i18n::ltr("Deno")),
            ],
        );
        own.ctx
            .accesskit_node_builder(own.id, |n| n.set_label(name));
        if own.clicked()
            && let Some(dir) = self.dir.clone()
        {
            self.install_deno(dir, rt, ui.ctx());
        }
    }
}

/// One of Helpers' cards at `y` (the mockup's box), its parts 12 inside (its buttons end at 67
/// of its 79).
fn card(p: &egui::Painter, o: Pos2, y: f32) {
    let c = crate::theme::colors();
    let card = r(o, 8.0, y - o.y, PROGRAMS_W - 16.0, 79.0);
    p.rect_filled(card, 8.0, c.panel);
    p.rect_stroke(
        card,
        8.0,
        egui::Stroke::new(1.0, c.menubar_line),
        egui::StrokeKind::Inside,
    );
}
