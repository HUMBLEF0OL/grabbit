//! The progress window's layout (plan 13b), a child module of progress.rs: the compact view,
//! 440 points wide, and Details, which widens the window to 760 to the right (the compact block
//! stays at the left in a column of 400, the details in a right column of 288). What it shows
//! comes from progress.rs (`status_page`'s status, `finishing_fraction`, the torrent details),
//! so upstream fixes there still apply as they are; the look is 13c's and 13d's to restyle.
//! Coordinates are English client points from the body's origin.

use egui::{Pos2, Ui, vec2};
use mdm_engine::Snapshot;

use super::{Action, Progress, Samples, TTab, TorrentCmd};
use crate::model::{Item, Status};
use crate::queue::{TurnOff, WhenDone};
use crate::torrents::Details;
use crate::winui::{self, ButtonKind, r, text};

/// The compact window's width and Details'.
pub const COMPACT_W: f32 = 440.0;
pub const DETAILS_W: f32 = 760.0;
/// The window's edges, either side.
const EDGE: f32 = 20.0;
/// The left column (the compact block) and the right one, 32 apart.
const LEFT_W: f32 = 400.0;
const RIGHT_X: f32 = EDGE + LEFT_W + 32.0;
const RIGHT_W: f32 = 288.0;
const _: () = assert!(RIGHT_X + RIGHT_W + EDGE == DETAILS_W);
const _: () = assert!(EDGE + LEFT_W + EDGE == COMPACT_W);
/// A torrent's tables in the right column (their tab pane a few points wider), and the pane's
/// height (the tables and their commands).
pub const TABLE_W: f32 = RIGHT_W - 6.0;
pub const PANE_H: f32 = 250.0;
/// The tab strip's top in the right column.
const TABS_Y: f32 = 8.0;
/// The strip's columns (its frame takes one more): the left column's width.
pub const STRIP_COLS: f32 = LEFT_W - 1.0;
const STRIP_Y: f32 = 168.0;
/// The state's line under the strip (its baseline).
const MSG_BASE: f32 = 202.0;
const COMPACT_FOOTER_Y: f32 = 214.0;
/// Details' rows in the left column: Saved to, From, a torrent's Peers, Sent and Pieces, then
/// Limit, Remember for this file and its note, When done, Force apps to close.
const ROW: f32 = 22.0;
const SAVED_Y: f32 = 222.0;
const LIMIT_Y: f32 = 270.0;
const REMEMBER_Y: f32 = 300.0;
const NOTE_BASE: f32 = 330.0;
const WHEN_Y: f32 = 342.0;
const FORCE_Y: f32 = 372.0;
const DETAILS_FOOTER_Y: f32 = 398.0;
/// Where the values start after their labels.
const VALUE_X: f32 = 108.0;
const BOTTOM: f32 = 20.0;
const GAP: f32 = 8.0;

/// How much lower a torrent's rows from Limit down sit: its Peers, Sent and Pieces rows.
fn facts_drop(torrent: bool) -> f32 {
    if torrent { 3.0 * ROW } else { 0.0 }
}

fn footer_y(details: bool, torrent: bool) -> f32 {
    if details {
        DETAILS_FOOTER_Y + facts_drop(torrent)
    } else {
        COMPACT_FOOTER_Y
    }
}

/// The window's body (under the title bar), compact or with Details.
pub fn size(details: bool, torrent: bool) -> [f32; 2] {
    let w = if details { DETAILS_W } else { COMPACT_W };
    [w, footer_y(details, torrent) + winui::CONTROL_H + BOTTOM]
}

/// When done (plan 13b): one list for every download, torrents too.
pub const WHEN_DONE: [&str; 6] = [
    // i18n: a download window's When done list: open the Done window when the download finishes
    crate::i18n::mark("Show the Done window"),
    // i18n: a download window's When done list: nothing happens when the download finishes
    crate::i18n::mark("Do nothing"),
    // i18n: a download window's When done list: Grabbit closes when the download finishes
    crate::i18n::mark("Exit Grabbit"),
    TurnOff::LABELS[0],
    TurnOff::LABELS[1],
    TurnOff::LABELS[2],
];

/// When done's first choice: the Done window when Options show it, else nothing.
pub fn when_done_index(show_complete: bool) -> usize {
    if show_complete { 0 } else { 1 }
}

/// What When done's `choice` does: show the Done window, and what else the app does then
/// (exit, or shut down, sleep or hibernate, `force` closing the apps for those three only).
pub fn when_done_of(choice: usize, force: bool) -> (bool, Option<WhenDone>) {
    match choice {
        0 => (true, None),
        2 => (
            false,
            Some(WhenDone {
                exit: true,
                ..Default::default()
            }),
        ),
        3..=5 => (
            false,
            Some(WhenDone {
                turn_off: Some(TurnOff::ALL[choice - 3]),
                force,
                ..Default::default()
            }),
        ),
        _ => (false, None),
    }
}

/// The state the window is drawn in: what its footer offers and its line says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Running,
    /// A live stream being recorded, its recording being saved.
    Recording,
    Saving,
    Stopped,
    /// Failed; `new_address`: its link expired or was refused (plan 12's E2).
    Failed {
        new_address: bool,
    },
    /// The site sent a web page instead of the file (upstream issue 701).
    WebPage,
    Waiting,
    /// After the last byte: the file written out, checked, a video joined.
    Finishing,
    /// A torrent checking its files.
    Checking,
    /// A torrent at a debrid service (#44).
    Service,
    /// A finished torrent still shared.
    Sharing,
    Done,
    /// Stop pressed where the server can't resume: Keep downloading, or Stop anyway.
    CantResume,
}

/// A footer button's press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    Stop,
    Resume,
    Refresh,
    OpenBrowser,
    KeepDownloading,
    StopAnyway,
    Cancel,
    Close,
}

/// The footer's buttons after Show details, in order: label, look, press. The filled-button
/// rule (plan 12's C3, plan 13): a state's one main action is filled; Stop and Cancel never are.
pub fn footer(phase: Phase) -> Vec<(&'static str, ButtonKind, Press)> {
    use ButtonKind::{Accent, Disabled, Normal};
    // i18n: verb: stop the download (it can be resumed)
    let stop = |kind| (crate::i18n::mark("Stop"), kind, Press::Stop);
    // i18n: verb: stop the download and close its window
    let cancel = (crate::i18n::mark("Cancel"), Normal, Press::Cancel);
    // i18n: verb: start a failed download again
    let retry = |kind| (crate::i18n::mark("Retry"), kind, Press::Resume);
    match phase {
        Phase::Running | Phase::Checking | Phase::Service => vec![stop(Normal), cancel],
        // i18n: button in a live stream's progress window: stop recording it and keep what was recorded as a video file
        Phase::Recording => vec![
            (crate::i18n::mark("Stop and save"), Normal, Press::Stop),
            cancel,
        ],
        Phase::Saving => vec![
            (crate::i18n::mark("Stop and save"), Disabled, Press::Stop),
            cancel,
        ],
        Phase::Waiting | Phase::Finishing => vec![stop(Disabled), cancel],
        // i18n: verb: continue the download
        Phase::Stopped => vec![(crate::i18n::mark("Resume"), Accent, Press::Resume), cancel],
        Phase::Failed { new_address: true } => vec![
            // i18n: button in a download window whose link expired: opens the page it came from and Properties, to give it a new address
            (
                crate::i18n::mark("Refresh download address..."),
                Accent,
                Press::Refresh,
            ),
            cancel,
        ],
        Phase::Failed { .. } => vec![retry(Accent), cancel],
        Phase::WebPage => vec![
            // i18n: button in a download's progress window when the site sent a web page (a waiting page, a captcha) instead of the file: opens the download's link in the web browser
            (
                crate::i18n::mark("Open in browser"),
                Accent,
                Press::OpenBrowser,
            ),
            retry(Normal),
            cancel,
        ],
        Phase::CantResume => vec![
            // i18n: button in a download window, after Stop on a download its server can't resume: go on downloading
            (
                crate::i18n::mark("Keep downloading"),
                Accent,
                Press::KeepDownloading,
            ),
            // i18n: button in a download window, after Stop on a download its server can't resume: stop it all the same (it starts over)
            (crate::i18n::mark("Stop anyway"), Normal, Press::StopAnyway),
        ],
        // i18n: verb: close the window (the download goes on, or it is finished)
        Phase::Sharing | Phase::Done => vec![(crate::i18n::mark("Close"), Normal, Press::Close)],
    }
}

/// Stop asks first (Keep downloading, Stop anyway) when stopping would start the download over:
/// a file download running whose server can't resume.
pub fn stop_asks_first(phase: Phase, resumable: Option<bool>, torrent: bool) -> bool {
    phase == Phase::Running && resumable == Some(false) && !torrent
}

/// A running torrent checking its files (`checking`; its snapshot then says Verifying too, so
/// it is asked first, as `Item::status_text` does), or a download past its last byte
/// (`finishing`: written out, checked, a video joined; `merging`: a screenshot's join). A
/// stopped torrent being checked stays Stopped (Resume): only a running one stops.
fn after_the_bytes(running: bool, merging: bool, checking: bool, finishing: bool) -> Option<Phase> {
    if merging {
        Some(Phase::Finishing)
    } else if running && checking {
        Some(Phase::Checking)
    } else if running && finishing {
        Some(Phase::Finishing)
    } else {
        None
    }
}

/// The download's state as the window draws it.
fn phase_of(item: &Item, snap: Option<&Snapshot>, pr: &Progress, sharing: bool) -> Phase {
    use mdm_tools::video::LiveState;
    if pr.sample_error.is_some() {
        return Phase::Failed { new_address: false };
    }
    match pr.sample_live.or_else(|| item.live_state()) {
        Some(LiveState::Recording) => return Phase::Recording,
        Some(LiveState::Saving) => return Phase::Saving,
        None => {}
    }
    let running = item.status == Status::Downloading;
    if let Some(phase) = after_the_bytes(
        running,
        pr.sample_merging.is_some(),
        item.torrent_checking().is_some(),
        super::finishing_fraction(snap).is_some()
            || item.finishing(false).is_some()
            || item.past_last_byte(),
    ) {
        return phase;
    }
    match item.status {
        Status::Downloading if item.at_service().is_some() => Phase::Service,
        Status::Downloading if pr.confirm_stop => Phase::CantResume,
        Status::Downloading => Phase::Running,
        Status::Stopped => Phase::Stopped,
        Status::Queued => Phase::Waiting,
        Status::Error if item.got_web_page() => Phase::WebPage,
        Status::Error => Phase::Failed {
            new_address: item.wants_new_address(),
        },
        Status::Complete if sharing => Phase::Sharing,
        Status::Complete => Phase::Done,
    }
}

/// Plan 13b's screenshot states (`/show progress:<name>`): looks only, drawn over the download.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Stopped,
    UnknownSize,
    Waiting,
    Finishing,
    Checking,
    Sharing,
    WebPage,
    Service,
    CantResume,
}

impl Look {
    const ALL: [(&'static str, Look); 9] = [
        ("stopped", Look::Stopped),
        ("unknown-size", Look::UnknownSize),
        ("waiting", Look::Waiting),
        ("finishing", Look::Finishing),
        ("checking", Look::Checking),
        ("sharing", Look::Sharing),
        ("webpage", Look::WebPage),
        ("service", Look::Service),
        ("cant-resume", Look::CantResume),
    ];

    pub fn named(name: &str) -> Option<Look> {
        Self::ALL.iter().find(|(n, _)| *n == name).map(|(_, l)| *l)
    }

    fn phase(self) -> Phase {
        match self {
            Look::Stopped => Phase::Stopped,
            Look::UnknownSize => Phase::Running,
            Look::Waiting => Phase::Waiting,
            Look::Finishing => Phase::Finishing,
            Look::Checking => Phase::Checking,
            Look::Sharing => Phase::Sharing,
            Look::WebPage => Phase::WebPage,
            Look::Service => Phase::Service,
            Look::CantResume => Phase::CantResume,
        }
    }

    /// Its status word, as the download's would say it.
    fn word(self) -> Option<String> {
        let t = |s: &str| crate::i18n::t(s).to_owned();
        Some(match self {
            Look::Stopped => t("Stopped"),
            Look::UnknownSize => t("Receiving data..."),
            Look::Waiting => t("Waiting"),
            Look::Finishing => t("Finishing..."),
            // i18n: a torrent's status while it reads its files and checks every piece
            Look::Checking => t("Checking files"),
            // i18n: a finished torrent's status: it is all downloaded and shared with other peers
            Look::Sharing => t("Complete, sharing"),
            Look::Service => crate::i18n::fmt(
                crate::i18n::t("{percent} at {service}"),
                &[
                    ("service", "Real-Debrid"),
                    ("percent", &crate::i18n::percent(0.62, 0)),
                ],
            ),
            Look::WebPage | Look::CantResume => return None,
        })
    }
}

/// The window's body (progress.rs's `Progress::body`): the compact block, Details when shown,
/// and the footer. The action asked for.
pub(super) fn body(
    pr: &mut Progress,
    ui: &mut Ui,
    o: Pos2,
    item: &Item,
    snap: Option<&Snapshot>,
    frac: Option<f64>,
    drawn: (f64, &[(u64, f64)]),
) -> Action {
    let p = ui.painter().clone();
    let torrent = item.torrent.is_some();
    let d = if torrent { pr.torrent_data(item) } else { None };
    // Stop's question is asked only while it downloads.
    if item.status != Status::Downloading {
        pr.confirm_stop = false;
    }
    let sharing = item.status == Status::Complete
        && d.as_ref()
            .and_then(|d| d.stats.as_ref())
            .is_some_and(|s| s.phase == mdm_torrent::Phase::Finished);
    let look = pr.sample_look;
    let phase = look.map_or_else(|| phase_of(item, snap, pr, sharing), Look::phase);

    // The name, then the download's rows (status_page's labels at the window's edge).
    winui::fit(
        ui,
        &p,
        o.x + EDGE,
        o.y + 22.0,
        &item.file_name,
        winui::TEXT(),
        o.x + EDGE + LEFT_W,
        "progress-name",
    );
    let word = look.and_then(Look::word);
    super::status_page(
        ui,
        o + vec2(EDGE - 33.0, -4.0),
        33.0 + LEFT_W,
        item,
        snap,
        frac,
        pr.limit,
        Samples {
            failed_with: pr.sample_error.as_deref(),
            merging: pr.sample_merging,
            helper: pr.sample_helper.as_ref(),
            live: pr.sample_live,
            word: word.as_deref(),
            no_size: look == Some(Look::UnknownSize),
        },
    );
    strip(
        &p,
        o,
        item,
        snap,
        pr.sample_merging,
        d.as_ref(),
        drawn,
        look == Some(Look::UnknownSize),
    );
    if let Some((line, alert)) = message(pr, phase, d.as_ref()) {
        let (rc, cut) = winui::text_fit(
            &p,
            o.x + EDGE,
            o.y + MSG_BASE,
            &line,
            winui::TEXT(),
            o.x + EDGE + LEFT_W,
        );
        let resp = winui::copyable(ui, rc, "progress-message", &line, cut);
        let role = if alert {
            egui::Role::Alert
        } else {
            egui::Role::Label
        };
        winui::expose(&resp, role, &line, true, None);
    }

    let mut action = Action::None;
    if pr.details {
        pr.init_limits(item);
        action = left_details(pr, ui, &p, o, item, d.as_ref());
        let right = if torrent {
            pr.torrent_details(
                ui,
                &p,
                o + vec2(RIGHT_X - 17.0, TABS_Y - 226.0),
                item,
                d.as_ref(),
            )
        } else {
            if item.video.is_none() && item.gallery.is_none() {
                // i18n: caption over a download's connections table in its window's details
                let caption = crate::i18n::t("Connections");
                text(&p, o.x + RIGHT_X, o.y + 22.0, caption, winui::TEXT());
                super::connections(
                    ui,
                    &p,
                    o + vec2(RIGHT_X - 17.0, 30.0 - 273.0),
                    snap,
                    RIGHT_W,
                );
            }
            // A video's speed graph comes with 13c.
            Action::None
        };
        if matches!(action, Action::None) {
            action = right;
        }
    }
    let footer = footer_row(pr, ui, &p, o, item, phase);
    if matches!(action, Action::None) {
        footer
    } else {
        action
    }
}

/// The strip under the rows: how far the check or the join after the last byte has come (in the
/// accent), a torrent's pieces, a file's segments, else one plain fill (13c restyles it);
/// empty for a size not known (`no_size`).
#[allow(clippy::too_many_arguments)]
fn strip(
    p: &egui::Painter,
    o: Pos2,
    item: &Item,
    snap: Option<&Snapshot>,
    merging: Option<f64>,
    d: Option<&Details>,
    drawn: (f64, &[(u64, f64)]),
    no_size: bool,
) {
    let (x0, y0) = (o.x + EDGE, o.y + STRIP_Y);
    let rc = r(o, EDGE, STRIP_Y, LEFT_W, 15.0);
    if no_size {
        // A size not known (a screenshot's): an empty strip.
        winui::meter(p, rc, 0.0);
    } else if let Some(f) = merging.or(super::finishing_fraction(snap)) {
        winui::meter_in(p, rc, f as f32, crate::theme::colors().accent);
    } else if let Some(d) = d.filter(|d| !d.pieces.is_empty()) {
        super::piece_bar(p, x0, y0, STRIP_COLS as usize, &d.pieces);
    } else if item.torrent.is_none()
        && item.video.is_none()
        && item.gallery.is_none()
        && item.size.is_some_and(|s| s > 0)
    {
        super::segment_bar(p, x0, y0, STRIP_COLS, item, drawn.1);
    } else {
        winui::meter(p, rc, drawn.0 as f32);
    }
}

/// The state's line under the strip, and whether it is an alert (read out at once).
fn message(pr: &Progress, phase: Phase, d: Option<&Details>) -> Option<(String, bool)> {
    let t = |s: &str| crate::i18n::t(s).to_owned();
    match phase {
        Phase::CantResume => Some((
            // i18n: a download window's line after Stop on a download whose server can't resume
            t("Stop it now and it starts over from 0. This server doesn't allow resuming."),
            true,
        )),
        Phase::WebPage => Some((
            // i18n: a download window's line when the site sent a web page instead of the file
            t("Open it in your browser. When the file starts, it comes back to Grabbit."),
            false,
        )),
        Phase::Waiting => pr.queue.as_deref().map(|q| {
            (
                crate::i18n::fmt(
                    // i18n: a waiting download's line: {queue} the queue it waits in ("Main queue")
                    crate::i18n::t("Waiting in {queue}"),
                    &[("queue", &crate::i18n::isolate(q))],
                ),
                false,
            )
        }),
        Phase::Sharing => d
            .and_then(|d| d.stats.as_ref())
            .map(|s| (super::sharing_line(s), false)),
        _ => None,
    }
}

/// Details' left column under the compact block: Saved to, From, a torrent's Peers, Sent and
/// Pieces, Limit (with Remember for this file and its note; a torrent's upload limit too), When
/// done. The action asked for (the folder or copy buttons).
fn left_details(
    pr: &mut Progress,
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    item: &Item,
    d: Option<&Details>,
) -> Action {
    let mut action = Action::None;
    let label = |y: f32, s: &str| {
        text(
            p,
            o.x + EDGE,
            o.y + y + 14.0,
            crate::i18n::t(s),
            winui::TEXT(),
        );
    };
    // Values stop before a row's 24-point button at the column's end.
    let value_right = o.x + EDGE + LEFT_W - 28.0;
    let button_at = |y: f32| r(o, EDGE + LEFT_W - 24.0, y - 4.0, 24.0, 24.0);

    // Saved to: its folder (plan 12's F2: "Save To:" left with the Options on completion tab).
    let y = SAVED_Y;
    // i18n: label in a download window's details: the folder the download is saved in
    label(y, crate::i18n::mark("Saved to"));
    let folder = match &item.torrent {
        Some(spec) => spec.folder.clone(),
        None => item
            .dest
            .parent()
            .map(|f| f.to_path_buf())
            .unwrap_or_default(),
    };
    winui::fit_middle(
        ui,
        p,
        o.x + VALUE_X,
        o.y + y + 14.0,
        &crate::dialogs::display_path(&folder),
        winui::TEXT(),
        value_right,
        "saved-to",
    );
    // i18n: a download window's button beside its folder: shows the download in its folder
    if winui::icon_only_button(
        ui,
        button_at(y),
        "cat-folder-open",
        crate::i18n::mark("Open folder"),
        true,
    )
    .clicked()
    {
        action = Action::OpenFolder;
    }

    // From: its address, copied by the button (a torrent's magnet link, its info hash on hover).
    let y = SAVED_Y + ROW;
    // i18n: label in a download window's details: where the download comes from (its address)
    label(y, crate::i18n::mark("From"));
    if let Some(spec) = &item.torrent {
        // i18n: a torrent's From row in its download window: it comes from a magnet link (the button beside it copies it)
        let rc = text(
            p,
            o.x + VALUE_X,
            o.y + y + 14.0,
            crate::i18n::t("Magnet link"),
            winui::TEXT(),
        );
        let tip = crate::i18n::fmt(
            // i18n: tooltip on a torrent's From row: {hash} = the torrent's info hash
            crate::i18n::t("Info hash {hash}"),
            &[("hash", &crate::i18n::ltr(&spec.hash))],
        );
        winui::full_on_hover(ui, rc, "from-hash", &tip);
        if winui::icon_only_button(
            ui,
            button_at(y),
            "gx-copy",
            crate::i18n::mark("Copy magnet link"),
            true,
        )
        .clicked()
        {
            action = Action::Torrent(TorrentCmd::CopyMagnet);
        }
    } else {
        let url = crate::i18n::ltr(item.url.as_str());
        let (rc, cut) = winui::text_fit(
            p,
            o.x + VALUE_X,
            o.y + y + 14.0,
            &url,
            winui::TEXT(),
            value_right,
        );
        winui::copyable(ui, rc, "from-url", &url, cut);
        if winui::icon_only_button(
            ui,
            button_at(y),
            "gx-copy",
            crate::i18n::mark("Copy address"),
            true,
        )
        .clicked()
        {
            action = Action::CopyAddress;
        }
    }

    // A torrent's numbers (its General tab's, plan 13b; 13c lays them out as the boards do).
    if let Some(spec) = &item.torrent {
        let st = d.and_then(|d| d.stats.as_ref());
        let value = |y: f32, s: &str, x: f32, right: f32| {
            let (rc, cut) =
                winui::text_fit(p, o.x + x, o.y + y + 14.0, s, winui::TEXT(), o.x + right);
            winui::copyable(ui, rc, ("torrent-fact", x as i32, y as i32), s, cut);
        };
        let y = SAVED_Y + 2.0 * ROW;
        label(y, crate::i18n::mark("Peers"));
        let peers = st.map_or(String::new(), |s| {
            crate::i18n::fmt(
                // i18n: {live} = peers connected now; {known} = peers heard of
                crate::i18n::t("{live} connected, {known} known"),
                &[
                    ("live", &crate::i18n::integer(s.peers_live as u64)),
                    ("known", &crate::i18n::integer(s.peers_seen as u64)),
                ],
            )
        });
        value(y, &peers, VALUE_X, EDGE + LEFT_W);
        // What it sent in all its runs (#39; librqbit counts only this one), against what it has.
        let y = y + ROW;
        // i18n: how much was sent to other peers
        label(y, crate::i18n::mark("Sent"));
        let done = st.map_or(item.downloaded, |s| s.done);
        value(y, &crate::model::size(spec.uploaded), VALUE_X, 210.0);
        // i18n: sent divided by received
        let ratio_label = crate::i18n::t("Share ratio");
        text(p, o.x + 220.0, o.y + y + 14.0, ratio_label, winui::TEXT());
        let ratio_x = 220.0 + winui::text_width(p, ratio_label).ceil() + 8.0;
        value(
            y,
            &crate::torrents::share_ratio_text(spec.uploaded, done),
            ratio_x,
            EDGE + LEFT_W,
        );
        let y = y + ROW;
        label(y, crate::i18n::mark("Pieces"));
        let pieces = d.map_or(String::new(), |d| {
            crate::i18n::fmt(
                // i18n: {have} and {total} = numbers of pieces; {size} = one piece's size
                crate::i18n::t("{have} of {total} ({size} each)"),
                &[
                    (
                        "have",
                        &crate::i18n::integer(d.pieces.iter().filter(|b| **b).count() as u64),
                    ),
                    ("total", &crate::i18n::integer(d.pieces.len() as u64)),
                    ("size", &crate::model::size(u64::from(d.piece_length))),
                ],
            )
        });
        value(y, &pieces, VALUE_X, EDGE + LEFT_W);
    }

    let drop = facts_drop(item.torrent.is_some());
    limits(pr, ui, p, o, item.torrent.is_some(), drop);
    when_done(pr, ui, p, o, item.torrent.is_some(), drop);
    action
}

/// The Limit row (the old Speed Limiter tab's, plan 13b): this download's limit, a torrent's upload
/// limit beside it, Remember for this file and its note. Reported to the app as typed.
fn limits(pr: &mut Progress, ui: &mut Ui, p: &egui::Painter, o: Pos2, torrent: bool, drop: f32) {
    let y = LIMIT_Y + drop;
    // i18n: label in a download window's details: its speed limit
    text(
        p,
        o.x + EDGE,
        o.y + y + 16.0,
        crate::i18n::t("Limit"),
        winui::TEXT(),
    );
    // A typed rate applies once typed (focus leaves, Enter), not at each digit (typing 100 would
    // throttle to 1, then 10 KB/s on the way). A rate a screen reader or UI Automation sets (Set
    // value) applies at once: there is no typing to wait for, and often no focus to leave.
    let typed = |ui: &Ui, rate_box: &egui::Response| {
        rate_box.lost_focus()
            || (rate_box.has_focus() && winui::enter_pressed(ui))
            || winui::value_set_by_assistive(rate_box)
    };
    let off = |on: bool| if on { winui::TEXT() } else { super::disabled() };
    let was_on = pr.limiter;
    let (down_label, box_x, box_w) = if torrent {
        // i18n: a torrent's Limit row: the download limit's check box (then the upload one, "Up")
        (crate::i18n::mark("Down"), VALUE_X + 60.0, 56.0)
    } else {
        // i18n: a download's Limit row: the check box before its speed limit box
        (crate::i18n::mark("Limit to"), VALUE_X + 76.0, 72.0)
    };
    winui::checkbox_until(
        ui,
        o + vec2(VALUE_X, y + 5.0),
        &mut pr.limiter,
        down_label,
        true,
        Some(o.x + box_x - 6.0),
    );
    let rate_box = winui::edit_right(
        ui,
        r(o, box_x, y, box_w, 23.0),
        &mut pr.limit_kb,
        pr.limiter,
    );
    pr.limit_kb.retain(|c| c.is_ascii_digit());
    let down_typed = typed(ui, &rate_box);
    let mut unit_x = box_x + box_w + 8.0;
    if torrent {
        let was_up = pr.up_on;
        let up_x = unit_x + 4.0;
        winui::checkbox_until(
            ui,
            o + vec2(up_x, y + 5.0),
            &mut pr.up_on,
            // i18n: a torrent's Limit row: the upload limit's check box (a limit for what it sends to others)
            "Up",
            true,
            Some(o.x + up_x + 50.0),
        );
        let up_box = winui::edit_right(
            ui,
            r(o, up_x + 56.0, y, 56.0, 23.0),
            &mut pr.up_kb,
            pr.up_on,
        );
        pr.up_kb.retain(|c| c.is_ascii_digit());
        let kb: u64 = pr.up_kb.parse().unwrap_or(0);
        if (was_up != pr.up_on || (pr.up_on && typed(ui, &up_box))) && !(pr.up_on && kb == 0) {
            pr.pending_upload = Some(pr.up_on.then_some(kb * 1024));
        }
        unit_x = up_x + 56.0 + 56.0 + 8.0;
    }
    // i18n: unit after a speed limit box: kilobytes per second
    text(
        p,
        o.x + unit_x,
        o.y + y + 16.0,
        crate::i18n::t("KB/s"),
        off(pr.limiter || pr.up_on),
    );
    let remembered = pr.remember;
    winui::checkbox(
        ui,
        o + vec2(VALUE_X, REMEMBER_Y + drop),
        &mut pr.remember,
        // i18n: a download window's check box under its speed limit: keep the limit when the download stops and resumes
        "Remember for this file",
        true,
    );
    text(
        p,
        o.x + VALUE_X,
        o.y + NOTE_BASE + drop,
        // i18n: the note under "Remember for this file" in a download window
        crate::i18n::t("Keeps this limit when the download stops and resumes."),
        crate::theme::colors().text_secondary,
    );
    // An empty or zero rate isn't "unlimited" while the box is ticked: the limit stays as it was
    // until a rate is typed.
    let kb: u64 = pr.limit_kb.parse().unwrap_or(0);
    if (was_on != pr.limiter || (pr.limiter && down_typed) || remembered != pr.remember)
        && !(pr.limiter && kb == 0)
    {
        pr.pending_limit = Some((pr.limiter.then_some(kb * 1024), pr.remember));
    }
}

/// When done's list, "Then keeps sharing" beside its first two choices for a torrent that will,
/// and "Force apps to close" under Shut down, Sleep and Hibernate.
fn when_done(pr: &mut Progress, ui: &mut Ui, p: &egui::Painter, o: Pos2, torrent: bool, drop: f32) {
    let y = WHEN_Y + drop;
    // i18n: label in a download window's details: what happens when the download finishes
    text(
        p,
        o.x + EDGE,
        o.y + y + 15.0,
        crate::i18n::t("When done"),
        winui::TEXT(),
    );
    if let Some(i) = winui::combo_light(
        ui,
        r(o, VALUE_X, y, 200.0, 21.0),
        &WHEN_DONE,
        pr.when,
        true,
        "progress-when-done",
    ) {
        pr.when = i;
    }
    if torrent && pr.keep_sharing && pr.when <= 1 {
        winui::fit(
            ui,
            p,
            o.x + VALUE_X + 212.0,
            o.y + y + 15.0,
            // i18n: beside a torrent's When done choice: it goes on sharing after it finishes
            crate::i18n::t("Then keeps sharing"),
            crate::theme::colors().text_secondary,
            o.x + EDGE + LEFT_W,
            "keeps-sharing",
        );
    }
    if pr.when >= 3 {
        winui::checkbox(
            ui,
            o + vec2(VALUE_X, FORCE_Y + drop),
            &mut pr.force,
            // i18n: under When done's Shut down, Sleep or Hibernate: don't let other programs delay it
            "Force apps to close",
            true,
        );
    }
}

/// A torrent's commands under its tab's table (plan 13b), at `at`: Files has Download in order,
/// First and last pieces first and Recheck files; Trackers has Add trackers... and Ask trackers
/// again. The one pressed.
pub(super) fn torrent_commands(
    pr: &mut Progress,
    ui: &mut Ui,
    at: Pos2,
    item: &Item,
    d: Option<&Details>,
) -> Action {
    let Some(spec) = &item.torrent else {
        return Action::None;
    };
    let busy = pr.host.as_ref().is_some_and(|h| h.busy(&spec.hash));
    // Running or shared now: Ask trackers again has someone to ask.
    let going = d.and_then(|d| d.stats.as_ref()).is_some_and(|s| {
        matches!(
            s.phase,
            mdm_torrent::Phase::Downloading | mdm_torrent::Phase::Finished
        )
    });
    let p = ui.painter().clone();
    let kind = |on: bool| {
        if on {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        }
    };
    let width = |label: &str| {
        (winui::text_width(&p, crate::i18n::t(label)) + 28.0)
            .ceil()
            .max(73.0)
    };
    let mut action = Action::None;
    match pr.ttab {
        TTab::Files => {
            // The boxes show the torrent's own choices; a click asks the app to change them.
            let mut in_order = spec.in_order;
            // i18n: a torrent's Files tab, a check box: download its files from their start, to watch a video while it downloads
            if winui::checkbox(
                ui,
                at + vec2(0.0, 4.0),
                &mut in_order,
                "Download in order",
                true,
            )
            .clicked()
            {
                action = Action::Torrent(TorrentCmd::InOrder);
            }
            let mut ends = spec.in_order && spec.ends_first;
            if winui::checkbox(
                ui,
                at + vec2(22.0, 26.0),
                &mut ends,
                // i18n: a torrent's Files tab, a check box under Download in order: each file's first and last pieces first, which a video player needs to start
                "First and last pieces first",
                spec.in_order,
            )
            .clicked()
            {
                action = Action::Torrent(TorrentCmd::EndsFirst);
            }
            // i18n: a torrent's Files tab, a button: read every piece of the torrent's files again and check it
            let label = crate::i18n::mark("Recheck files");
            let resp = winui::button(
                ui,
                r(at, 0.0, 48.0, width(label), winui::CONTROL_H),
                label,
                kind(!busy && item.status != Status::Queued),
                false,
                None,
            );
            if resp.clicked() {
                action = Action::Torrent(TorrentCmd::Recheck);
            }
            // i18n: tooltip of a torrent's Recheck files button
            resp.on_hover_text(crate::i18n::t("Reads every piece again and checks it"));
        }
        TTab::Trackers => {
            let add = crate::i18n::mark("Add trackers...");
            // i18n: a torrent's Trackers tab, a button: ask the torrent's trackers for peers again now
            let again = crate::i18n::mark("Ask trackers again");
            let add_w = width(add);
            if winui::button(
                ui,
                r(at, 0.0, 0.0, add_w, winui::CONTROL_H),
                add,
                kind(!busy),
                false,
                None,
            )
            .clicked()
            {
                action = Action::Torrent(TorrentCmd::AddTrackers);
            }
            if winui::button(
                ui,
                r(at, add_w + GAP, 0.0, width(again), winui::CONTROL_H),
                again,
                kind(!busy && going),
                false,
                None,
            )
            .clicked()
            {
                action = Action::Torrent(TorrentCmd::AskTrackersAgain);
            }
        }
        TTab::Peers => {}
    }
    action
}

/// The footer: Show details / Hide details with its chevron at the start, the state's buttons
/// at the end ([`footer`]). What a press asks for.
fn footer_row(
    pr: &mut Progress,
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    item: &Item,
    phase: Phase,
) -> Action {
    let y = footer_y(pr.details, item.torrent.is_some());
    let w = if pr.details { DETAILS_W } else { COMPACT_W };
    // Plan 12's F9: "Show details" / "Hide details", a chevron after it (down: it opens).
    let details_label = if pr.details {
        // i18n: button at a download window's footer: hide the details (the window gets narrow again)
        crate::i18n::mark("Hide details")
    } else {
        // i18n: button at a download window's footer: show the details (the window widens to the right)
        crate::i18n::mark("Show details")
    };
    let dw = (winui::text_width(p, crate::i18n::t(details_label)) + 40.0).ceil();
    let rc = r(o, EDGE, y, dw, winui::CONTROL_H);
    if winui::button(ui, rc, details_label, ButtonKind::Ghost, false, None).clicked() {
        pr.details = !pr.details;
    }
    chevron(p, rc, pr.details);

    let buttons = footer(phase);
    let widths: Vec<f32> = buttons
        .iter()
        .map(|(label, ..)| {
            (winui::text_width(p, crate::i18n::t(label)) + 28.0)
                .ceil()
                .max(73.0)
        })
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + GAP * (widths.len().saturating_sub(1)) as f32;
    let mut x = w - EDGE - total;
    let mut pressed = None;
    for ((label, kind, press), bw) in buttons.iter().zip(&widths) {
        if winui::button(
            ui,
            r(o, x, y, *bw, winui::CONTROL_H),
            label,
            *kind,
            false,
            None,
        )
        .clicked()
        {
            pressed = Some(*press);
        }
        x += bw + GAP;
    }
    let Some(press) = pressed else {
        return Action::None;
    };
    // A screenshot's state resumes, stops, saves or cancels nothing: the press only ends it
    // (Close closes the window, the download going on, as always).
    let sample = pr.sample_look.is_some() || pr.sample_error.is_some() || pr.sample_live.is_some();
    match press {
        Press::Close => Action::Close,
        _ if sample => {
            pr.clear_samples();
            Action::None
        }
        Press::Cancel => Action::Cancel,
        Press::Stop if stop_asks_first(phase, item.resumable, item.torrent.is_some()) => {
            pr.confirm_stop = true;
            Action::None
        }
        Press::Stop | Press::StopAnyway => {
            pr.confirm_stop = false;
            Action::Pause
        }
        Press::KeepDownloading => {
            pr.confirm_stop = false;
            Action::None
        }
        Press::Resume => Action::Resume,
        Press::Refresh => Action::RefreshAddress,
        Press::OpenBrowser => {
            crate::launch::open_page(item.url.as_str());
            Action::None
        }
    }
}

/// The details button's chevron in its room at the end: down while closed, up while open.
fn chevron(p: &egui::Painter, rc: egui::Rect, open: bool) {
    let c = egui::pos2(rc.right() - 13.0, rc.center().y);
    let (a, b) = if open { (2.0, -2.0) } else { (-2.0, 2.0) };
    p.line(
        vec![c + vec2(-3.5, a), c + vec2(0.0, b), c + vec2(3.5, a)],
        egui::Stroke::new(1.4, winui::TEXT()),
    );
}

/// The window, just widened (Details), moved left as far as it must to stay inside its screen's
/// work area ([`widened_left`]).
pub(super) fn keep_on_screen(title: &str) {
    let Some(([left, top, right, bottom], _)) = crate::win32::window_rect(title) else {
        return;
    };
    let screens = crate::win32::screens();
    let monitors: Vec<[i32; 4]> = screens.iter().map(|(m, _)| *m).collect();
    let Some(on) = screen_for([left, top, right, bottom], &monitors) else {
        return;
    };
    let work = screens[on].1;
    let x = widened_left(left, right - left, work[0], work[2]);
    if x != left {
        crate::win32::move_window(title, x, top);
    }
}

/// The screen a window ([left, top, right, bottom], screen pixels) is on: the one it overlaps
/// most, else the nearest; None without screens.
pub fn screen_for(window: [i32; 4], screens: &[[i32; 4]]) -> Option<usize> {
    let overlap = |s: &[i32; 4]| {
        let w = (window[2].min(s[2]) - window[0].max(s[0])).max(0) as i64;
        let h = (window[3].min(s[3]) - window[1].max(s[1])).max(0) as i64;
        w * h
    };
    // How far the window's centre is from the screen (0 inside it).
    let distance = |s: &[i32; 4]| {
        let (cx, cy) = (
            (window[0] + window[2]) as i64 / 2,
            (window[1] + window[3]) as i64 / 2,
        );
        let dx = (s[0] as i64 - cx).max(cx - s[2] as i64).max(0);
        let dy = (s[1] as i64 - cy).max(cy - s[3] as i64).max(0);
        dx * dx + dy * dy
    };
    let most = (0..screens.len()).max_by_key(|&i| overlap(&screens[i]))?;
    if overlap(&screens[most]) > 0 {
        return Some(most);
    }
    (0..screens.len()).min_by_key(|&i| distance(&screens[i]))
}

/// Where a window `width` wide whose left edge is at `left` goes so it stays inside a work area
/// from `work_left` to `work_right` (screen pixels): where it is when it fits, else as far left
/// as it must, never past the work area's left edge.
pub fn widened_left(left: i32, width: i32, work_left: i32, work_right: i32) -> i32 {
    if left + width <= work_right {
        left
    } else {
        (work_right - width).max(work_left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// When done's list maps onto the Done window and the app's when-done choices; Force apps to
    /// close counts only with Shut down, Sleep or Hibernate; Hang up modem is gone.
    #[test]
    fn when_done_maps_its_choices() {
        assert_eq!(when_done_of(0, true), (true, None));
        assert_eq!(when_done_of(1, true), (false, None));
        let exit = when_done_of(2, true).1.unwrap();
        assert!(exit.exit && exit.turn_off.is_none() && !exit.force && !exit.hang_up);
        for (i, mode) in TurnOff::ALL.into_iter().enumerate() {
            let (shown, w) = when_done_of(3 + i, true);
            let w = w.unwrap();
            assert!(!shown && w.turn_off == Some(mode) && w.force && !w.exit && !w.hang_up);
            assert!(!when_done_of(3 + i, false).1.unwrap().force);
        }
        assert_eq!(WHEN_DONE.len(), 6);
        assert_eq!(WHEN_DONE[3..], TurnOff::LABELS);
        // The first choice follows Options' "Show download complete dialog".
        assert!(when_done_of(when_done_index(true), false).0);
        assert!(!when_done_of(when_done_index(false), false).0);
    }

    /// The filled-button rule: at most one main action, filled; Stop and Cancel never are.
    #[test]
    fn only_a_states_main_action_is_filled() {
        let phases = [
            Phase::Running,
            Phase::Recording,
            Phase::Saving,
            Phase::Stopped,
            Phase::Failed { new_address: true },
            Phase::Failed { new_address: false },
            Phase::WebPage,
            Phase::Waiting,
            Phase::Finishing,
            Phase::Checking,
            Phase::Service,
            Phase::Sharing,
            Phase::Done,
            Phase::CantResume,
        ];
        for phase in phases {
            let row = footer(phase);
            let filled: Vec<_> = row.iter().filter(|b| b.1 == ButtonKind::Accent).collect();
            assert!(filled.len() <= 1, "{phase:?}");
            for (label, kind, _) in &row {
                if ["Stop", "Cancel", "Stop and save", "Stop anyway"].contains(label) {
                    assert_ne!(*kind, ButtonKind::Accent, "{phase:?} {label}");
                }
            }
        }
        let main = |phase| {
            footer(phase)
                .into_iter()
                .find(|b| b.1 == ButtonKind::Accent)
                .map(|b| b.0)
        };
        assert_eq!(main(Phase::Stopped), Some("Resume"));
        assert_eq!(
            main(Phase::Failed { new_address: true }),
            Some("Refresh download address...")
        );
        assert_eq!(main(Phase::Failed { new_address: false }), Some("Retry"));
        assert_eq!(main(Phase::WebPage), Some("Open in browser"));
        assert_eq!(main(Phase::CantResume), Some("Keep downloading"));
        assert_eq!(main(Phase::Running), None);
        // Waiting and finishing: Stop greyed; a finished torrent sharing: Close only.
        assert_eq!(footer(Phase::Waiting)[0].1, ButtonKind::Disabled);
        assert_eq!(footer(Phase::Finishing)[0].1, ButtonKind::Disabled);
        assert_eq!(footer(Phase::Sharing).len(), 1);
        // Can't resume asks; nothing cancels there.
        assert!(
            footer(Phase::CantResume)
                .iter()
                .all(|b| b.2 != Press::Cancel)
        );
    }

    /// Stop asks first only for a running file download whose server can't resume.
    #[test]
    fn stop_asks_first_only_where_it_would_start_over() {
        assert!(stop_asks_first(Phase::Running, Some(false), false));
        assert!(!stop_asks_first(Phase::Running, Some(true), false));
        assert!(!stop_asks_first(Phase::Running, None, false));
        assert!(
            !stop_asks_first(Phase::Running, Some(false), true),
            "a torrent"
        );
        assert!(
            !stop_asks_first(Phase::Recording, Some(false), false),
            "a live stream saves"
        );
    }

    /// Widened to Details, the window stays on its screen: where it is when it fits, moved left
    /// when it would cross the work area's right edge, never past its left.
    #[test]
    fn the_window_stays_on_screen_when_expanded() {
        assert_eq!(widened_left(100, 760, 0, 1920), 100);
        assert_eq!(widened_left(1500, 760, 0, 1920), 1160);
        assert_eq!(widened_left(1160, 760, 0, 1920), 1160, "just fits");
        // A second screen to the right.
        assert_eq!(widened_left(3500, 760, 1920, 3840), 3080);
        // A screen narrower than the window: its left edge.
        assert_eq!(widened_left(200, 760, 0, 640), 0);
    }

    /// The screen a widened window keeps to: the one it overlaps most, a secondary one to the
    /// right or one at negative coordinates on the left too, else the nearest.
    #[test]
    fn the_window_keeps_to_its_own_screen() {
        let primary = [0, 0, 1920, 1080];
        let right = [1920, 0, 3840, 1080];
        let left = [-1280, 0, 0, 1024];
        let screens = [primary, right, left];
        // On the right screen, its top-left corner just past every screen's top (outside them).
        assert_eq!(screen_for([3300, -20, 3740, 400], &screens), Some(1));
        // Mostly on the right one, a little over the primary.
        assert_eq!(screen_for([1800, 100, 2240, 400], &screens), Some(1));
        // On the left one, at negative coordinates.
        assert_eq!(screen_for([-900, 100, -460, 400], &screens), Some(2));
        // Off every screen: the nearest.
        assert_eq!(screen_for([4000, 100, 4440, 400], &screens), Some(1));
        assert_eq!(screen_for([-2000, 100, -1600, 400], &screens), Some(2));
        assert_eq!(screen_for([0, 0, 10, 10], &[]), None);
        // Widened there, it moves left inside that screen's work area.
        let (x, w) = (3300, 760);
        assert_eq!(widened_left(x, w, right[0], right[2]), 3080);
    }

    /// A running torrent checking its files is Checking (Stop works), though its snapshot says
    /// Verifying as a file being finished does; a stopped one being checked keeps Stopped.
    #[test]
    fn a_running_torrent_checking_is_not_finishing() {
        assert_eq!(
            after_the_bytes(true, false, true, true),
            Some(Phase::Checking)
        );
        assert_eq!(
            footer(Phase::Checking)[0].1,
            ButtonKind::Normal,
            "Stop enabled"
        );
        assert_eq!(
            after_the_bytes(true, false, false, true),
            Some(Phase::Finishing)
        );
        assert_eq!(after_the_bytes(false, false, true, true), None);
        assert_eq!(
            after_the_bytes(false, true, false, false),
            Some(Phase::Finishing)
        );
        assert_eq!(after_the_bytes(true, false, false, false), None);
    }

    /// Every state name of plan 13b reads back as its look.
    #[test]
    fn looks_are_named() {
        for (name, look) in Look::ALL {
            assert_eq!(Look::named(name), Some(look));
        }
        assert_eq!(Look::named("pieces"), None);
        assert_eq!(Look::named("menu"), None);
    }

    /// The compact window is 440 wide, Details 760; a torrent's details are taller (its rows).
    #[test]
    fn details_widen_the_window() {
        assert_eq!(size(false, false)[0], COMPACT_W);
        assert_eq!(size(true, false)[0], DETAILS_W);
        assert!(size(true, true)[1] > size(true, false)[1]);
        assert_eq!(size(false, true), size(false, false));
    }
}
