//! The progress window's layout and look (plan 13b's structure, 13c's look), a child module of
//! progress.rs: the compact view, 440 points wide, and Details, which widens the window to 760 to
//! the right (the compact block stays at the left in a column of 400, the details in a right
//! column of 288). What it shows comes from progress.rs (`status_word`, `finishing_fraction`, the
//! torrent details), so upstream fixes there still apply as they are.
//!
//! The look is the mockup canvas's (boards 1, 2, 3, 6, 10-12): two edges only (x 20 and the
//! right edge), steps of the 4 px scale; the name, one large reading with the time left, a 6 px
//! strip drawn from the real data (a file's segments, a torrent's pieces, else one plain fill),
//! a detail line and the footer. Coordinates are English client points from the body's origin.

use std::collections::VecDeque;

use egui::{Color32, FontFamily, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, pos2, vec2};
use mdm_engine::Snapshot;
use mdm_engine::download::ConnStatus;

use super::{Action, Progress, Samples, TTab, TorrentCmd};
use crate::model::{Item, Status};
use crate::queue::{TurnOff, WhenDone};
use crate::torrents::Details;
use crate::winui::{self, ButtonKind, r};

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
/// The strip: as wide as the left column, 6 high.
pub const STRIP_W: f32 = LEFT_W;
const STRIP_H: f32 = 6.0;
/// The compact block (boards 1, 6): the name's baseline (13/500), the reading's (28/500), the
/// strip's top, the detail line's baseline (12).
const NAME_BASE: f32 = 21.0;
const READ_BASE: f32 = 64.0;
const STRIP_Y: f32 = 80.0;
const DETAIL_BASE: f32 = 106.0;
const COMPACT_FOOTER_Y: f32 = 156.0;
/// Details' facts under the compact block: a key column of 88, then the values.
const FACTS_Y: f32 = 130.0;
const VALUE_X: f32 = EDGE + 88.0;
/// The footer's buttons, and the room under them.
const BUTTON_H: f32 = 28.0;
const BOTTOM: f32 = 20.0;
const GAP: f32 = 8.0;
/// The right column's top, a header's height (its text, 8 and its line), the speed graph's.
const TOP: f32 = 8.0;
const HEAD: f32 = 25.0;
const GRAPH_H: f32 = 44.0;
/// A table's rows (each with its line under it), its column titles' row, rows shown at once.
const ROW_H: f32 = 26.0;
const TITLES_H: f32 = 24.0;
const ROWS_SHOWN: usize = 5;
/// An icon-only button.
const ICON_BUTTON: f32 = 24.0;

/// What a download is, for the window's rows and right column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A file the engine downloads: its segments, Server, Can resume, the connections.
    File,
    Torrent,
    /// A video or a gallery, one progress for the whole job: Format and Downloaded by.
    Video,
}

impl Kind {
    pub fn of(item: &Item) -> Kind {
        if item.torrent.is_some() {
            Kind::Torrent
        } else if item.video.is_some() || item.gallery.is_some() {
            Kind::Video
        } else {
            Kind::File
        }
    }
}

/// Details' rows in the left column, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fact {
    Saved,
    Source,
    Server,
    CanResume,
    Format,
    By,
    Peers,
    Sent,
    Pieces,
    Limit,
    Remember,
    Note,
    WhenDone,
    Force,
}

impl Fact {
    /// A row of text is 16 high; one with a text box or a drop-down 26.
    fn height(self) -> f32 {
        match self {
            Fact::Limit | Fact::WhenDone => ROW_H,
            _ => 16.0,
        }
    }
}

fn facts(kind: Kind) -> Vec<Fact> {
    let middle: &[Fact] = match kind {
        Kind::File => &[Fact::Server, Fact::CanResume],
        Kind::Torrent => &[Fact::Peers, Fact::Sent, Fact::Pieces],
        Kind::Video => &[Fact::Format, Fact::By],
    };
    [Fact::Saved, Fact::Source]
        .into_iter()
        .chain(middle.iter().copied())
        .chain([
            Fact::Limit,
            Fact::Remember,
            Fact::Note,
            Fact::WhenDone,
            Fact::Force,
        ])
        .collect()
}

/// Where the facts start under a state's message of `lines` lines in the detail row's place
/// ([`under_lines`]): where they always do for one line or none, a message line lower for each
/// line after the first (the compact window keeps its height: its footer is under them all).
const fn facts_top(lines: usize) -> f32 {
    FACTS_Y + lines.saturating_sub(1) as f32 * MESSAGE_LINE
}

/// `phrase` as a sentence: its first letter upper case ("the server answered 404" → "The
/// server answered 404").
fn sentence(phrase: &str) -> String {
    let mut chars = phrase.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The lines Details keeps for the row under the strip, whatever the state: a message's two at
/// most (a web page's reason and advice end as low, the strip raised into its reading's place:
/// the asserts after [`MESSAGE_LINES`]). Every state the same, so Details keeps one height: a
/// state with a message (Stop on a server that can't resume) doesn't move the footer under the
/// pointer. ponytail: a 20-point gap under a detail row is the price.
pub fn under_lines(_view: &StateView) -> usize {
    MESSAGE_LINES
}

fn facts_end(kind: Kind, top: f32) -> f32 {
    facts(kind).iter().fold(top, |y, f| y + f.height() + GAP) - GAP
}

/// A table's height at its fewest rows.
fn table_height() -> f32 {
    TITLES_H + ROWS_SHOWN as f32 * ROW_H
}

/// What a torrent's commands take under its table: 12, then two rows of check boxes.
const COMMANDS_H: f32 = 12.0 + 16.0 + GAP + 16.0;

/// The whole rows a table whose titles start at `y` shows down to `bottom` (the left column's
/// end), five at the fewest.
fn rows_down_to(y: f32, bottom: f32) -> usize {
    (((bottom - y - TITLES_H) / ROW_H).floor().max(0.0) as usize).max(ROWS_SHOWN)
}

/// The right column's end: the speed graph and the connections, the graph only, or the tabs
/// with their table and commands.
fn right_end(kind: Kind) -> f32 {
    let graph = TOP + HEAD + GAP + GRAPH_H;
    match kind {
        Kind::File => graph + 16.0 + HEAD + table_height(),
        Kind::Video => graph,
        Kind::Torrent => TOP + HEAD + table_height() + COMMANDS_H,
    }
}

fn footer_y(details: bool, kind: Kind, lines: usize) -> f32 {
    if details {
        facts_end(kind, facts_top(lines)).max(right_end(kind)) + 12.0
    } else {
        COMPACT_FOOTER_Y
    }
}

/// The window's body (under the title bar), compact or with Details; `lines`: the state's
/// message lines under the strip ([`under_lines`]).
pub fn size(details: bool, kind: Kind, lines: usize) -> [f32; 2] {
    let w = if details { DETAILS_W } else { COMPACT_W };
    [w, footer_y(details, kind, lines) + BUTTON_H + BOTTOM]
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

/// The state as the window shows it this frame: the phase, and what the compact block shows in
/// it (plan 13d). Every state's lines fit the compact block's fixed height ([`paint_state`]).
pub(super) struct Shown {
    pub phase: Phase,
    pub view: StateView,
}

pub(super) fn shown(
    pr: &mut Progress,
    item: &Item,
    snap: Option<&Snapshot>,
    d: Option<&Details>,
) -> Shown {
    // Stop's question is asked only while it downloads.
    if item.status != Status::Downloading {
        pr.confirm_stop = false;
    }
    let sharing = item.status == Status::Complete
        && d.and_then(|d| d.stats.as_ref())
            .is_some_and(|s| s.phase == mdm_torrent::Phase::Finished);
    let look = pr.sample_look;
    let phase = look.map_or_else(|| phase_of(item, snap, pr, sharing), Look::phase);
    let look_word = look.and_then(Look::word);
    let status = super::status_word(
        item,
        snap,
        Samples {
            failed_with: pr.sample_error.as_deref(),
            merging: pr.sample_merging,
            live: pr.sample_live,
            word: look_word.as_deref(),
        },
    );
    let facts = facts_of(pr, item, snap, d, status);
    Shown {
        phase,
        view: state_view(phase, &facts),
    }
}

/// What a state's look is drawn from ([`state_view`]): the download's figures as the window
/// shows them, each read from what the app already has (plan 13: no new engine states).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    /// The file's name ("Naming it ..." while it is written out).
    pub name: String,
    /// How far, with a size to take a share of.
    pub frac: Option<f64>,
    pub downloaded: u64,
    pub size: Option<u64>,
    /// The time left ("3 min 10 sec"), empty when there is none.
    pub left: String,
    /// The detail line's speed while bytes come (the limit holding it back beside it), and the
    /// speed alone (a size not known reads it beside the amount).
    pub rate: Option<String>,
    pub speed: Option<String>,
    /// The download's status word ("Stopped", "Receiving data...").
    pub word: String,
    /// The queue it waits in, by name.
    pub queue: Option<String>,
    /// Stopped, it goes on where it stopped (its server said so, or a torrent).
    pub resumes: bool,
    /// Why it failed, as `errors::describe` says it.
    pub reason: Option<String>,
    /// After the last byte: which step, and how far it is when known.
    pub after: After,
    pub after_frac: Option<f64>,
    /// A torrent checking its files: bytes checked, of all.
    pub checked: Option<(u64, u64)>,
    /// At a debrid service: its name, how far it is when it says.
    pub service: Option<(String, Option<f64>)>,
    /// A torrent's sharing.
    pub sharing: Option<Sharing>,
}

/// A finished torrent's sharing (board 9): what it sent in all its runs, its ratio, the peers
/// connected now (when the session says), and how long it has been shared
/// (`TorrentSpec::seeded_secs`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sharing {
    pub sent: String,
    pub ratio: String,
    pub peers: Option<usize>,
    pub secs: u64,
}

/// The step after the last byte ([`Phase::Finishing`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum After {
    /// A video's picture and sound joined (ffmpeg).
    Join,
    /// The file checked against its checksum.
    Check,
    /// The file written out and named.
    #[default]
    Write,
}

/// The facts of this download for its state's look; a screenshot's state (`/show progress:`)
/// gets the figures its board shows where the download has none of its own.
fn facts_of(
    pr: &Progress,
    item: &Item,
    snap: Option<&Snapshot>,
    d: Option<&Details>,
    word: String,
) -> Facts {
    use mdm_engine::download::State;
    let look = pr.sample_look;
    let no_size = look == Some(Look::UnknownSize);
    let bytes_come = item.status == Status::Downloading && !item.past_last_byte();
    let speed = bytes_come.then(|| crate::model::rate(item.rate()));
    // The limit holding it back beside the speed (upstream issue 1191).
    let rate = speed.clone().map(|rate| match pr.limit {
        Some(limit) => limit.after(&rate),
        None => rate,
    });
    let state = snap.map(|s| s.state);
    let after = if pr.sample_merging.is_some() {
        After::Join
    } else if look.is_some() {
        After::Write
    } else if state == Some(State::Merging) {
        After::Join
    } else if state == Some(State::Verifying) {
        After::Check
    } else {
        After::Write
    };
    let checked = if look == Some(Look::Checking) {
        let total = item.size.filter(|s| *s > 0).unwrap_or(1 << 30);
        Some((total / 100 * 37, total))
    } else {
        snap.filter(|s| item.torrent.is_some() && s.state == State::Verifying)
            .map(|s| (s.downloaded, s.size.unwrap_or(0)))
    };
    let service = if look == Some(Look::Service) {
        Some(("Real-Debrid".to_owned(), Some(0.62)))
    } else {
        match &item.live {
            Some(crate::model::Live {
                dl: crate::model::Job::Remote(r),
                ..
            }) => r
                .at_service()
                .map(|(s, p)| (s.to_owned(), p.map(f64::from))),
            _ => None,
        }
    };
    let stats = d.and_then(|d| d.stats.as_ref());
    let sample = (look == Some(Look::Sharing)).then(|| Sharing {
        sent: crate::model::size(308 * 1024 * 1024),
        ratio: "0.42".to_owned(), // not translated: a number
        peers: Some(6),
        secs: 4320,
    });
    let sharing = sample.or_else(|| {
        item.torrent.as_ref().map(|spec| {
            let done = stats.map_or(item.downloaded, |s| s.done);
            Sharing {
                sent: crate::model::size(spec.uploaded),
                ratio: crate::torrents::share_ratio_text(spec.uploaded, done),
                peers: stats.map(|s| s.peers_live),
                secs: spec.seeded_secs,
            }
        })
    });
    Facts {
        name: item.file_name.clone(),
        frac: if no_size { None } else { item.fraction() },
        downloaded: item.downloaded,
        size: if no_size { None } else { item.size },
        left: if no_size {
            String::new()
        } else {
            item.time_left()
        },
        rate,
        speed,
        word,
        queue: queue_named(
            pr.queue.as_deref(),
            pr.main_queue.as_deref(),
            look == Some(Look::Waiting),
        ),
        resumes: item.resumable == Some(true) || item.torrent.is_some(),
        reason: pr.sample_error.clone().or_else(|| item.error.clone()),
        after,
        after_frac: pr.sample_merging.or(super::finishing_fraction(snap)),
        checked,
        service,
        sharing,
    }
}

/// The queue a waiting download names: its own; for the screenshot's state (`sample`) of a
/// download in no queue, the main queue (board 8: "in Main queue").
pub fn queue_named(own: Option<&str>, main: Option<&str>, sample: bool) -> Option<String> {
    own.or(main.filter(|_| sample)).map(str::to_owned)
}

/// What is beside the reading, at the row's end.
#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    None,
    /// A figure (in the numbers face) and its unit after it, smaller: "3 min 10 sec left".
    Figure(String, Option<String>),
    /// A few words: "Resumes where it stopped".
    Text(String),
    /// Words before a name in the ink: "in" and "Main queue".
    Named(String, String),
}

/// The row under the strip.
#[derive(Clone, Debug, PartialEq)]
pub enum Under {
    /// Two small texts, at the start and at the end: "2.38 of 5.80 GB", "18.4 MB/s".
    Detail(String, String),
    /// The state's message in its place, read out at once (an alert), in two lines at most;
    /// `error`: in the error colour (a failed download's reason); `advice`: a line of what to do
    /// under it, in the secondary ink (a web page's, board 9).
    Message {
        text: String,
        error: bool,
        advice: Option<String>,
    },
}

/// What the compact block shows in a state (plan 13d, boards 1, 4, 8, 9): the word beside the
/// name (in the error colour, or not), the reading greyed or not, what is beside it, the strip
/// (the download's own, or the state's) drawn grey or in the accent, the row under it.
#[derive(Clone, Debug, PartialEq)]
pub struct StateView {
    pub word: Option<(String, bool)>,
    pub reading: Reading,
    pub dim: bool,
    pub side: Side,
    /// None: the download's own strip ([`strip_of`]).
    pub strip: Option<Strip>,
    pub grey: bool,
    pub under: Under,
    /// The strip's value for screen readers, 0 to 1 (a percent's, or the state's own: whole
    /// once done or sharing); None: none to say.
    pub value: Option<f64>,
}

/// `s` as a figure and its unit: "6.10 MB/s" is 6.10 and MB/s.
fn figure(s: &str) -> Side {
    match s.rsplit_once(' ') {
        Some((n, unit)) => Side::Figure(n.to_owned(), Some(unit.to_owned())),
        None => Side::Figure(s.to_owned(), None),
    }
}

/// A state's look (plan 13d, boards 1, 4, 8, 9) from its facts.
pub fn state_view(phase: Phase, f: &Facts) -> StateView {
    let t = |s: &str| crate::i18n::t(s).to_owned();
    let fmt = |s: &str, args: &[(&str, &str)]| crate::i18n::fmt(crate::i18n::t(s), args);
    let amount = so_far(f.downloaded, f.size);
    let known = f.size.is_some_and(|s| s > 0);
    let reading = match f.frac {
        Some(x) => Reading::Percent(x),
        None if f.downloaded > 0 => Reading::Amount(f.downloaded),
        None => Reading::Word(f.word.clone()),
    };
    let left = if f.left.is_empty() {
        Side::None
    } else {
        // i18n: after the time left in the download window, smaller: "3 min 10 sec left"
        Side::Figure(f.left.clone(), Some(t("left")))
    };
    let mut v = StateView {
        word: Some((f.word.clone(), false)),
        reading,
        dim: false,
        side: Side::None,
        strip: None,
        grey: false,
        under: Under::Detail(amount.clone(), String::new()),
        value: None,
    };
    match phase {
        // Size not known (board 8): the amount received, its speed beside it, an empty strip.
        Phase::Running if !known && f.downloaded > 0 => {
            // i18n: a download window's word beside the name: the server didn't say the file's size
            v.word = Some((t("Size not known"), false));
            v.side = f.speed.as_deref().map_or(Side::None, figure);
            v.strip = Some(Strip::Empty);
            v.under = Under::Detail(
                // i18n: a download window's line when the server didn't say the file's size
                t("The server didn't say how big it is"),
                // i18n: a download window's line when the file's size isn't known: no time left can be worked out
                t("No time estimate"),
            );
        }
        Phase::Running | Phase::Recording | Phase::Saving | Phase::Done => {
            if phase == Phase::Running {
                v.word = None;
            }
            v.side = left;
            v.under = Under::Detail(amount, f.rate.clone().unwrap_or_default());
        }
        Phase::CantResume => {
            // i18n: a download window's word beside the name after Stop on a download its server can't resume
            v.word = Some((t("Can't resume"), false));
            v.side = left;
            v.under = Under::Message {
                // i18n: a download window's line after Stop on a download whose server can't resume
                text: t(
                    "Stop it now and it starts over from 0. This server doesn't allow resuming.",
                ),
                error: false,
                advice: None,
            };
        }
        Phase::Stopped => {
            v.dim = true;
            v.grey = true;
            if f.resumes {
                // i18n: beside a stopped download's percent: Resume goes on from there
                v.side = Side::Text(t("Resumes where it stopped"));
            }
        }
        Phase::Waiting => {
            v.reading = Reading::Dash;
            v.grey = true;
            v.value = Some(f.frac.unwrap_or(0.0));
            if let Some(q) = &f.queue {
                // i18n: beside a waiting download's dash, before the name of the queue it waits in (in the ink): "in Main queue"
                let before = t("in {queue}");
                let before = before
                    .split("{queue}")
                    .next()
                    .unwrap_or_default()
                    .trim_end();
                v.side = Side::Named(before.to_owned(), crate::i18n::isolate(q));
            }
        }
        Phase::Failed { .. } | Phase::WebPage => {
            let web = phase == Phase::WebPage;
            v.word = Some(if web {
                // i18n: a download window's word beside the name: the site sent a web page instead of the file
                (t("Got a web page"), true)
            } else {
                // i18n: a download window's word beside the name: the download failed
                (t("Stopped by an error"), true)
            });
            v.dim = true;
            if web {
                // Board 9: no reading, the strip, the reason and the advice.
                v.reading = Reading::None;
            } else if f.frac.is_none() && f.downloaded == 0 {
                v.reading = Reading::Dash;
            } else if known {
                v.side = Side::Figure(amount, None);
            }
            v.strip = Some(Strip::Broken(f.frac.unwrap_or(0.0) as f32));
            v.under = if web {
                Under::Message {
                    // i18n: a download window's reason line when the site sent a web page instead of the file
                    text: t(
                        "The site sent a web page instead of the file, such as a waiting page or a check.",
                    ),
                    error: true,
                    // i18n: a download window's line under the reason when the site sent a web page instead of the file
                    advice: Some(t(
                        "Open it in your browser. When the file starts, it comes back to Grabbit.",
                    )),
                }
            } else {
                Under::Message {
                    // The engine's reasons are phrases written to follow "Error: " (lower case);
                    // on a line of their own they start a sentence (board 4).
                    text: f.reason.as_deref().map(sentence).unwrap_or_else(|| {
                        // i18n: a failed download's line when it wasn't told why
                        t("It stopped, and Grabbit wasn't told why.")
                    }),
                    error: true,
                    advice: None,
                }
            };
        }
        Phase::Finishing => {
            let all = fmt(
                // i18n: a download window's line once every byte is in: {size} the whole size, as "All 5.80 GB received"
                "All {size} received",
                &[("size", &crate::model::size(f.size.unwrap_or(f.downloaded)))],
            );
            let (side, naming) = match f.after {
                After::Join => {
                    // i18n: a download window's word beside the name while a video's picture and sound are joined into one file
                    v.word = Some((t("Joining picture and sound"), false));
                    // i18n: beside the percent while a video's picture and sound are joined: "45% joined"
                    (t("joined"), String::new())
                }
                After::Check => {
                    // i18n: a download window's word beside the name while the finished file is checked against its checksum
                    v.word = Some((t("Checking the file"), false));
                    // i18n: beside the percent while the finished file is checked against its checksum: "45% checked"
                    (t("checked"), String::new())
                }
                After::Write => {
                    // i18n: a download window's word beside the name once every byte is in: the file is written out and named
                    v.word = Some((t("Finishing"), false));
                    v.reading = Reading::Percent(1.0);
                    v.strip = Some(Strip::Plain {
                        fraction: 1.0,
                        after: false,
                    });
                    (
                        // i18n: beside "100%" once every byte is in: the file is being written out
                        t("Writing the file"),
                        fmt(
                            // i18n: a download window's line while the finished file is written out: {file} its name
                            "Naming it {file}",
                            &[("file", &crate::i18n::isolate(&f.name))],
                        ),
                    )
                }
            };
            // A join's or a check's percent only when it is known (a dash before that, not the
            // download's own 100%).
            v.side = Side::Text(side);
            if f.after != After::Write {
                match f.after_frac {
                    Some(x) => v.reading = Reading::Percent(x),
                    None => {
                        v.reading = Reading::Dash;
                        v.side = Side::None;
                    }
                }
            }
            v.under = Under::Detail(all, naming);
        }
        Phase::Checking => {
            // i18n: a torrent's status while it reads its files and checks every piece
            v.word = Some((t("Checking files"), false));
            let (done, total) = f.checked.unwrap_or((0, 0));
            let x = if total > 0 {
                (done as f64 / total as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
            v.reading = Reading::Percent(x);
            v.side = Side::Text(t("checked"));
            v.strip = Some(Strip::Plain {
                fraction: x as f32,
                after: false,
            });
            v.grey = true;
            v.under = Under::Detail(
                fmt(
                    // i18n: a torrent's line while it checks its files: {amount} how much is checked, as "272 of 734.60 MB checked"
                    "{amount} checked",
                    &[("amount", &so_far(done, Some(total)))],
                ),
                // i18n: a torrent's line while it checks its files: after the check it downloads the pieces it lacks
                t("Then it downloads what's missing"),
            );
        }
        Phase::Service => {
            let (service, x) = f.service.clone().unwrap_or_default();
            let service = crate::i18n::isolate(&service);
            v.word = Some((
                // i18n: a download window's word beside the name: {service} the debrid service downloading the torrent first ("At Real-Debrid")
                fmt("At {service}", &[("service", &service)]),
                false,
            ));
            v.reading = x.map_or(Reading::Dash, Reading::Percent);
            v.side = Side::Text(fmt(
                // i18n: beside the percent while a debrid service downloads the torrent: "62% at Real-Debrid"
                "at {service}",
                &[("service", &service)],
            ));
            v.strip = Some(Strip::Striped(x.unwrap_or(0.0) as f32));
            v.under = Under::Detail(
                // i18n: a download window's line while a debrid service downloads the torrent first
                t("The service downloads it first"),
                // i18n: a download window's line while a debrid service downloads the torrent: then Grabbit gets the files from it, fast
                t("Then it comes here at full speed"),
            );
        }
        Phase::Sharing => {
            v.word = None;
            // i18n: a finished torrent's reading in its download window: it is shared with other peers
            v.reading = Reading::Word(t("Sharing"));
            let s = f.sharing.clone().unwrap_or_default();
            // i18n: after a torrent's share ratio in its download window, smaller: "0.42 ratio"
            v.side = Side::Figure(s.ratio, Some(t("ratio")));
            v.strip = Some(Strip::Plain {
                fraction: 1.0,
                after: false,
            });
            v.value = Some(1.0);
            let sent = match s.peers {
                Some(n) => fmt(
                    // i18n: a sharing torrent's line: {size} what it sent to other peers in all; {peers} how many are connected now, as "6 peers connected"
                    "Sent {size}, {peers}",
                    &[
                        ("size", &s.sent),
                        // i18n: how many peers a finished torrent is connected to now
                        (
                            "peers",
                            &crate::i18n::tn(n as u64, "{n} peer connected", "{n} peers connected"),
                        ),
                    ],
                ),
                // i18n: a sharing torrent's line: {size} what it sent to other peers in all
                None => fmt("Sent {size}", &[("size", &s.sent)]),
            };
            let shared = if s.secs > 0 {
                fmt(
                    // i18n: a sharing torrent's line: {time} how long it has been shared, as "shared for 1 hour(s) 12 min"
                    "shared for {time}",
                    &[("time", &crate::model::time(s.secs as f64))],
                )
            } else {
                String::new()
            };
            v.under = Under::Detail(sent, shared);
        }
    }
    if v.value.is_none()
        && let Reading::Percent(x) = v.reading
    {
        v.value = Some(x);
    }
    v
}

/// The Done window's look (plan 13d, boards 5, 7): "Done" with the size at the row's end, the
/// strip whole, the number of files under it (nothing for one file).
pub fn done_view(size: u64, files: usize) -> StateView {
    let count = if files > 1 {
        // i18n: under the Done window's strip: how many files the download saved
        crate::i18n::tn(files as u64, "{n} file", "{n} files")
    } else {
        String::new()
    };
    StateView {
        word: None,
        // i18n: the Done window's reading: the download finished
        reading: Reading::Word(crate::i18n::t("Done").to_owned()),
        dim: false,
        side: Side::Figure(crate::model::size(size), None),
        strip: Some(Strip::Plain {
            fraction: 1.0,
            after: false,
        }),
        grey: false,
        under: Under::Detail(count, String::new()),
        value: Some(1.0),
    }
}

// ---- the look's parts ------------------------------------------------------------------------

fn font(size: f32, family: &str) -> FontId {
    FontId::new(size, FontFamily::Name(family.into()))
}

/// The interface's face (Geist under Graphite), at `size`.
fn face(size: f32) -> FontId {
    font(size, "tahoma")
}

/// Its Medium: the name, the tabs.
fn medium(size: f32) -> FontId {
    font(size, "geist-medium")
}

/// Figures that line up as they change (plan 05's `numbers` family).
fn figures(size: f32) -> FontId {
    font(size, crate::theme::NUMBERS)
}

fn ink() -> Color32 {
    winui::TEXT()
}

fn muted() -> Color32 {
    crate::theme::colors().muted
}

fn secondary() -> Color32 {
    crate::theme::colors().text_secondary
}

/// `s` with its ink ending at English x `right`.
fn draw_right(p: &egui::Painter, f: FontId, right: f32, base: f32, s: &str, c: Color32) -> Rect {
    let w = winui::text_width_font(p, f.clone(), s);
    winui::text_font(p, f, right - w, base, s, c)
}

/// `s` from `x` to at most `right` (English x), cut with "..." and then whole on hover; its
/// right-click menu copies it.
#[allow(clippy::too_many_arguments)]
fn cell(
    ui: &Ui,
    p: &egui::Painter,
    f: FontId,
    x: f32,
    base: f32,
    right: f32,
    s: &str,
    c: Color32,
    salt: impl std::hash::Hash + std::fmt::Debug,
) -> Rect {
    let shown = winui::ellipsize(p, &f, s, (right - x).max(0.0));
    let rc = winui::text_font(p, f, x, base, &shown, c);
    winui::copyable(ui, rc, salt, s, shown != s);
    rc
}

/// [`cell`] with its ink ending at `right`, starting no further out than `left`.
#[allow(clippy::too_many_arguments)]
fn cell_right(
    ui: &Ui,
    p: &egui::Painter,
    f: FontId,
    left: f32,
    base: f32,
    right: f32,
    s: &str,
    c: Color32,
    salt: impl std::hash::Hash + std::fmt::Debug,
) -> Rect {
    let shown = winui::ellipsize(p, &f, s, (right - left).max(0.0));
    let rc = draw_right(p, f, right, base, &shown, c);
    winui::copyable(ui, rc, salt, s, shown != s);
    rc
}

/// A flat 24-point button showing only Graphite's `icon` in the muted ink, its face under the
/// pointer (the hover token); `tip` (English) its name and its tooltip, under the pointer and on
/// keyboard focus.
fn icon_button(ui: &mut Ui, rc: Rect, icon: &str, tip: &str) -> Response {
    let id = egui::Id::unique(("progress-icon-button", ui.ctx().viewport_id(), icon));
    let resp = ui.interact(rc, id, Sense::click());
    winui::focus_on_press(&resp);
    let tip = crate::i18n::t(tip);
    winui::expose(&resp, egui::Role::Button, tip, true, None);
    let p = ui.painter();
    if resp.hovered() || resp.has_focus() {
        p.rect_filled(rc, 4.0, crate::theme::colors().button_hot);
    }
    let glyph = Rect::from_center_size(rc.center(), vec2(13.0, 13.0));
    crate::themeicons::paint_graphite_in(ui, p, icon, glyph, muted());
    winui::show_focus(ui, &resp, rc);
    winui::tip(resp, tip)
}

// ---- the reading and the strip ---------------------------------------------------------------

/// The large reading.
#[derive(Clone, Debug, PartialEq)]
pub enum Reading {
    /// How far, with a size to take a share of.
    Percent(f64),
    /// No size: how much has come.
    Amount(u64),
    /// Nothing yet: the status word; or the state's own ("Sharing", "Done").
    Word(String),
    /// Nothing to say yet (a download waiting its turn): a dash, not read out.
    Dash,
    /// No reading at all (a web page instead of the file, board 9): its row left empty.
    None,
}

/// The reading's figure for a fraction: whole percents, one decimal below 10 (a torrent's 6.7),
/// rounded down so 100 shows only when it is all there.
pub fn reading_figure(frac: f64) -> String {
    // A hair over, so 0.29 × 100 (28.999...) reads 29.
    let pct = (frac * 100.0).clamp(0.0, 100.0) + 1e-9;
    if frac <= 0.0 {
        "0".to_owned()
    } else if pct < 10.0 {
        format!("{:.1}", (pct * 10.0).floor() / 10.0) // not translated: a number
    } else {
        format!("{}", pct.floor().min(100.0) as u32) // not translated: a number
    }
}

/// "2.38 of 5.80 GB": the amount so far and the size (the app's sizes), the unit said once when
/// both have it; the amount alone without a size.
pub fn so_far(done: u64, total: Option<u64>) -> String {
    let d = crate::model::size(done);
    let Some(t) = total.filter(|t| *t > 0).map(crate::model::size) else {
        return d;
    };
    let d = match (d.rsplit_once(' '), t.rsplit_once(' ')) {
        (Some((n, u)), Some((_, v))) if u == v => n.to_owned(),
        _ => d,
    };
    crate::i18n::fmt(
        // i18n: the download window's detail line: {done} so far of {total}, as "2.38 of 5.80 GB"
        crate::i18n::t("{done} of {total}"),
        &[("done", &d), ("total", &t)],
    )
}

/// One segment of a file's strip, from the strip's start: its room from `x0` to `x1`, filled
/// to `fill`, with a tick there while it is still being written.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub x0: f32,
    pub x1: f32,
    pub fill: f32,
    pub tick: bool,
}

/// What the strip shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Strip {
    /// A failed download (board 4): what came this far, the part missing hatched in the error
    /// colours.
    Broken(f32),
    /// At a debrid service (board 9): its progress this far, striped (it comes later).
    Striped(f32),
    /// Nothing known to show (a size not known).
    Empty,
    /// One fill this far: a video, a gallery, segments too narrow to tell apart; `after`: a
    /// step after the bytes (a check, a join) on its accent track.
    Plain { fraction: f32, after: bool },
    /// A file's segments.
    Segments(Vec<Part>),
    /// A torrent's pieces in columns, each lit by how much of it is in (0 to 1).
    Pieces(Vec<f32>),
}

/// A segment narrower than this, and more segments than this, make the strip one plain fill.
const MIN_PART: f32 = 3.0;
const MAX_PARTS: usize = 48;
/// The gap between segments, and the cursor's tick.
const PART_GAP: f32 = 1.0;
const TICK_W: f32 = 2.0;

/// A file's segments (`segments`: each one's start and bytes done, as drawn) across `width`
/// points of a file of `total` bytes: each as wide as its share of the file (from its start to
/// the next one's, as the engine splits the file), filled from its start as far as it has
/// come, a tick at its cursor while it is still being written. None when one would be narrower
/// than 3 points or there are more than 48: one plain fill then.
pub fn segment_parts(segments: &[(u64, f64)], total: u64, width: f32) -> Option<Vec<Part>> {
    if total == 0 || segments.is_empty() || segments.len() > MAX_PARTS {
        return None;
    }
    let mut s = segments.to_vec();
    s.sort_by_key(|g| g.0);
    let x = |b: u64| (b.min(total) as f64 / total as f64 * f64::from(width)) as f32;
    let mut parts = Vec::with_capacity(s.len());
    for (i, &(start, done)) in s.iter().enumerate() {
        let end = s.get(i + 1).map_or(total, |n| n.0);
        let (a, b) = (x(start), x(end));
        if b - a < MIN_PART {
            return None;
        }
        let x1 = if i + 1 == s.len() { b } else { b - PART_GAP };
        let share = (done / end.saturating_sub(start).max(1) as f64).clamp(0.0, 1.0) as f32;
        parts.push(Part {
            x0: a,
            x1,
            fill: a + (x1 - a) * share,
            tick: share < 1.0,
        });
    }
    Some(parts)
}

/// What the strip shows for this download (`drawn`: the fraction and the segments as they
/// glide, progress.rs's `Glide`).
fn strip_of(
    item: &Item,
    snap: Option<&Snapshot>,
    merging: Option<f64>,
    d: Option<&Details>,
    drawn: (f64, &[(u64, f64)]),
    no_size: bool,
) -> Strip {
    if no_size {
        return Strip::Empty;
    }
    if let Some(f) = merging.or(super::finishing_fraction(snap)) {
        return Strip::Plain {
            fraction: f as f32,
            after: true,
        };
    }
    if let Some(d) = d.filter(|d| !d.pieces.is_empty()) {
        let cols = d.pieces.len().min(STRIP_W as usize);
        return Strip::Pieces(
            (0..cols)
                .map(|c| super::piece_column(&d.pieces, c, cols))
                .collect(),
        );
    }
    if Kind::of(item) == Kind::File
        && let Some(total) = item.size.filter(|s| *s > 0)
        && let Some(parts) = segment_parts(drawn.1, total, STRIP_W)
    {
        return Strip::Segments(parts);
    }
    Strip::Plain {
        fraction: drawn.0 as f32,
        after: false,
    }
}

/// The strip at the compact block's place, filled in the accent, or `grey` (stopped, waiting, a
/// torrent's check: nothing moving, or nothing downloaded yet).
fn paint_strip(p: &egui::Painter, o: Pos2, strip: &Strip, grey: bool) {
    let c = crate::theme::colors();
    let track = c.meter_track;
    let fill = if grey { muted() } else { c.progress_bar };
    let at = |x0: f32, x1: f32| r(o, EDGE + x0, STRIP_Y, (x1 - x0).max(0.0), STRIP_H);
    match strip {
        Strip::Broken(fraction) => {
            let x = STRIP_W * fraction.clamp(0.0, 1.0);
            winui::solid(p, at(0.0, x), fill);
            // The missing part: the error track, hatched at 135° (5 points a stripe, 2 of them
            // the hatch's), as the board's repeating gradient.
            let missing = at(x, STRIP_W);
            winui::solid(p, missing, c.strip_error);
            let clip = p.with_clip_rect(missing.intersect(p.clip_rect()));
            let ink = Stroke::new(2.0, c.strip_error_hatch);
            let mut sx = missing.left() - STRIP_H;
            while sx < missing.right() {
                clip.line_segment(
                    [
                        pos2(sx, missing.bottom()),
                        pos2(sx + STRIP_H, missing.top()),
                    ],
                    ink,
                );
                sx += 7.0;
            }
        }
        Strip::Striped(fraction) => {
            winui::solid(p, at(0.0, STRIP_W), track);
            // 4 points of the accent, 2 of its track, as far as the service has come.
            let end = STRIP_W * fraction.clamp(0.0, 1.0);
            let mut x = 0.0;
            while x < end {
                winui::solid(p, at(x, (x + 4.0).min(end)), fill);
                winui::solid(p, at(x + 4.0, (x + 6.0).min(end)), c.strip_accent);
                x += 6.0;
            }
        }
        Strip::Empty => winui::solid(p, at(0.0, STRIP_W), track),
        Strip::Plain { fraction, after } => {
            let back = if *after { c.strip_accent } else { track };
            winui::solid(p, at(0.0, STRIP_W), back);
            winui::solid(p, at(0.0, STRIP_W * fraction.clamp(0.0, 1.0)), fill);
        }
        Strip::Segments(parts) => {
            for part in parts {
                winui::solid(p, at(part.x0, part.x1), track);
                winui::solid(p, at(part.x0, part.fill), fill);
            }
            // The ticks over the fills, as high as the strip (the boards clip them to it).
            for part in parts.iter().filter(|g| g.tick) {
                let x = part.fill.min(part.x1 - TICK_W).max(part.x0);
                winui::solid(p, at(x, x + TICK_W), ink());
            }
        }
        Strip::Pieces(cols) => {
            winui::solid(p, at(0.0, STRIP_W), track);
            let w = STRIP_W / cols.len().max(1) as f32;
            for (i, lit) in cols.iter().enumerate().filter(|(_, l)| **l > 0.0) {
                let x = i as f32 * w;
                winui::solid(p, at(x, x + w), fill.gamma_multiply(*lit));
            }
        }
    }
}

/// The compact block: the name (the state's word at its end), then the state's look
/// ([`paint_state`]).
#[allow(clippy::too_many_arguments)]
fn compact(
    pr: &Progress,
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    item: &Item,
    snap: Option<&Snapshot>,
    drawn: (f64, &[(u64, f64)]),
    shown: &Shown,
    d: Option<&Details>,
) {
    let view = &shown.view;
    paint_name(ui, p, o, &item.file_name, view.word.as_ref());
    let no_size = pr.sample_look == Some(Look::UnknownSize);
    let strip = view
        .strip
        .clone()
        .unwrap_or_else(|| strip_of(item, snap, pr.sample_merging, d, drawn, no_size));
    paint_state(ui, p, o, view, &strip);
}

/// The name (13/500) at the top, the state's `word` at the row's end (in the error colour for
/// a failure).
fn paint_name(ui: &Ui, p: &egui::Painter, o: Pos2, name: &str, word: Option<&(String, bool)>) {
    let right = o.x + EDGE + LEFT_W;
    let mut name_right = right;
    if let Some((word, error)) = word {
        let c = if *error { winui::ERROR() } else { muted() };
        let room = LEFT_W / 2.0;
        let rc = cell_right(
            ui,
            p,
            face(12.0),
            right - room,
            o.y + NAME_BASE,
            right,
            word,
            c,
            "progress-word",
        );
        name_right = rc.left() - 16.0;
    }
    cell(
        ui,
        p,
        medium(13.0),
        o.x + EDGE,
        o.y + NAME_BASE,
        name_right,
        name,
        ink(),
        "progress-name",
    );
}

/// The reading as it is read out: "41%", "214.00 MB", a word; nothing for a dash.
fn spoken_reading(reading: &Reading) -> Option<String> {
    match reading {
        Reading::Percent(f) => Some(format!("{}%", reading_figure(*f))), // not translated: a percentage
        Reading::Amount(b) => Some(crate::model::size(*b)),
        Reading::Word(w) => Some(w.clone()),
        Reading::Dash | Reading::None => None,
    }
}

/// What the strip says to a screen reader in a state: its word, the reading, what is beside it
/// and the row under it (a message is read out on its own, as an alert).
pub fn spoken(view: &StateView) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some((w, _)) = &view.word {
        parts.push(w.clone());
    }
    parts.extend(spoken_reading(&view.reading));
    match &view.side {
        Side::Figure(v, Some(unit)) => parts.push(format!("{v} {unit}")), // not translated: a figure and its unit
        Side::Figure(v, None) | Side::Text(v) => parts.push(v.clone()),
        Side::Named(before, name) => parts.push(format!("{before} {name}")), // not translated: words and a name
        Side::None => {}
    }
    if let Under::Detail(start, end) = &view.under {
        parts.extend([start, end].into_iter().filter(|s| !s.is_empty()).cloned());
    }
    parts.join(", ") // not translated: a list
}

/// A state's look under the name (plan 13d; the Done window's too): the reading (greyed for a
/// download that isn't moving) with what is beside it at the row's end, the strip (one progress
/// value for screen readers, its parts hidden from them) and the row under it, a detail line or
/// the state's message in its place.
pub(crate) fn paint_state(ui: &Ui, p: &egui::Painter, o: Pos2, view: &StateView, strip: &Strip) {
    let right = o.x + EDGE + LEFT_W;
    let base = o.y + READ_BASE;
    let x = o.x + EDGE;
    // What is beside the reading first: the reading ends before it.
    let side_left = match &view.side {
        Side::None => right,
        Side::Figure(v, unit) => {
            let end = match unit {
                Some(u) => draw_right(p, face(12.0), right, base, u, muted()).left() - 4.0,
                None => right,
            };
            draw_right(p, figures(15.0), end, base, v, secondary()).left()
        }
        Side::Text(s) => cell_right(
            ui,
            p,
            face(15.0),
            x + LEFT_W / 2.0,
            base,
            right,
            s,
            secondary(),
            "progress-side",
        )
        .left(),
        // The name in the ink, the words before it in the secondary ink (board 8).
        Side::Named(before, name) => {
            let rc = cell_right(
                ui,
                p,
                face(15.0),
                x + LEFT_W / 2.0,
                base,
                right,
                name,
                ink(),
                "progress-side",
            );
            draw_right(p, face(15.0), rc.left() - 4.0, base, before, secondary()).left()
        }
    };
    let (number, unit_ink) = if view.dim {
        (secondary(), secondary())
    } else {
        (ink(), secondary())
    };
    match &view.reading {
        Reading::Percent(f) => {
            let n = reading_figure(*f);
            let rc = winui::text_font(p, figures(28.0), x, base, &n, number);
            winui::text_font(p, figures(16.0), rc.right(), base, "%", unit_ink);
        }
        Reading::Amount(b) => {
            let s = crate::model::size(*b);
            let (n, unit) = s.rsplit_once(' ').unwrap_or((&s, ""));
            let rc = winui::text_font(p, figures(28.0), x, base, n, number);
            winui::text_font(p, figures(16.0), rc.right() + 4.0, base, unit, unit_ink);
        }
        Reading::Word(w) => {
            cell(
                ui,
                p,
                medium(28.0),
                x,
                base,
                side_left - 16.0,
                w,
                number,
                "progress-reading",
            );
        }
        Reading::Dash => {
            winui::text_font(p, figures(28.0), x, base, "-", muted()); // not translated: a dash
        }
        Reading::None => {}
    }

    // No reading (board 9): the strip and the lines under it go up into its row's place.
    let o = if view.reading == Reading::None {
        o - vec2(0.0, NO_READING_LIFT)
    } else {
        o
    };
    paint_strip(p, o, strip, view.grey);
    let id = egui::Id::unique(("progress-strip", ui.ctx().viewport_id()));
    let resp = ui.interact(r(o, EDGE, STRIP_Y, STRIP_W, STRIP_H), id, Sense::hover());
    winui::expose_value(&resp, egui::Role::ProgressIndicator, &spoken(view), true);
    // i18n: the download window's progress strip, for screen readers
    let name = crate::i18n::t("Progress").to_owned();
    let value = view.value;
    resp.ctx.accesskit_node_builder(resp.id, |n| {
        n.set_label(name);
        if let Some(f) = value {
            n.set_numeric_value((f * 100.0).clamp(0.0, 100.0));
            n.set_min_numeric_value(0.0);
            n.set_max_numeric_value(100.0);
        }
    });

    // The row under the strip.
    let detail = o.y + DETAIL_BASE;
    match &view.under {
        Under::Detail(start, end) => {
            let mut start_right = right;
            if !end.is_empty() {
                // Cut in the middle, so a file name's end shows ("Naming it ubuntu-...iso").
                let f = figures(12.0);
                let shown = winui::ellipsize_middle(p, &f, end, LEFT_W / 2.0);
                let rc = draw_right(p, f, right, detail, &shown, muted());
                winui::copyable(ui, rc, "progress-rate", end, shown != *end);
                start_right = rc.left() - 16.0;
            }
            cell(
                ui,
                p,
                figures(12.0),
                x,
                detail,
                start_right,
                start,
                muted(),
                "progress-amount",
            );
        }
        Under::Message {
            text,
            error,
            advice,
        } => {
            let c = if *error { winui::ERROR() } else { ink() };
            let base = detail + 2.0;
            let (rc, n) = message_lines(ui, p, x, base, right, text, c, "progress-reason");
            let id = egui::Id::unique(("progress-reason", ui.ctx().viewport_id()));
            let resp = ui.interact(rc, id, Sense::hover());
            winui::expose(&resp, egui::Role::Alert, text, true, None);
            if let Some(advice) = advice {
                // 4 under the reason (board 9).
                let base = base + n as f32 * MESSAGE_LINE + 4.0;
                let (rc, _) = message_lines(
                    ui,
                    p,
                    x,
                    base,
                    right,
                    advice,
                    secondary(),
                    "progress-advice",
                );
                let id = egui::Id::unique(("progress-advice", ui.ctx().viewport_id()));
                let resp = ui.interact(rc, id, Sense::hover());
                winui::expose(&resp, egui::Role::Label, advice, true, None);
            }
        }
    }
}

/// How far up the strip and the lines under it go without a reading: its row and the room
/// above it (board 9: the strip 12 under the name).
const NO_READING_LIFT: f32 = 44.0;
/// A message's lines (13/20, the boards' `.msg`), two at most.
const MESSAGE_LINE: f32 = 20.0;
const MESSAGE_LINES: usize = 2;
// Two lines of a message fit above the footer; without a reading, two of the reason and two of
// the advice under it (board 9). Each line's descent is under 8.
const _: () = assert!(DETAIL_BASE + 2.0 + MESSAGE_LINE + 8.0 <= COMPACT_FOOTER_Y - 16.0);
const _: () = assert!(
    DETAIL_BASE - NO_READING_LIFT + 2.0 + 3.0 * MESSAGE_LINE + 4.0 + 8.0 <= COMPACT_FOOTER_Y - 16.0
);
// In Details the facts start under the message's last line (its descent 8; a fact's folder
// button reaches 4 above its row): two lines, and a web page's reason and advice raised.
const _: () =
    assert!(DETAIL_BASE + 2.0 + MESSAGE_LINE + 8.0 <= facts_top(MESSAGE_LINES) - FACT_BUTTON_UP);
const _: () = assert!(
    DETAIL_BASE - NO_READING_LIFT + 2.0 + 3.0 * MESSAGE_LINE + 4.0 + 8.0
        <= facts_top(MESSAGE_LINES) - FACT_BUTTON_UP
);
/// How far above its row a fact's 24-point button starts (Saved to's, From's).
const FACT_BUTTON_UP: f32 = 4.0;

/// `lines` as a message shows them: as they are when they are few enough; else the first ones
/// and the rest joined in the last (cut with "..." when drawn, whole on hover).
pub fn at_most(mut lines: Vec<String>, n: usize) -> Vec<String> {
    if lines.len() > n && n > 0 {
        let rest = lines.split_off(n - 1).join(" ");
        lines.push(rest);
    }
    lines
}

/// A state's message from English `x` to `right`, its first baseline at `base`: wrapped in at
/// most two lines (the last cut with "..." when it is longer, whole on hover). The rect it
/// takes and its lines.
#[allow(clippy::too_many_arguments)]
fn message_lines(
    ui: &Ui,
    p: &egui::Painter,
    x: f32,
    base: f32,
    right: f32,
    text: &str,
    c: Color32,
    salt: &str,
) -> (Rect, usize) {
    let f = face(13.0);
    let lines = at_most(
        winui::wrap_lines_font(p, f.clone(), text, right - x),
        MESSAGE_LINES,
    );
    let mut rc = Rect::NOTHING;
    for (k, line) in lines.iter().enumerate() {
        let y = base + k as f32 * MESSAGE_LINE;
        let shown = winui::ellipsize(p, &f, line, right - x);
        rc = rc.union(winui::text_font(p, f.clone(), x, y, &shown, c));
        if shown != *line {
            winui::copyable(ui, rc, (salt, k), text, true);
        }
    }
    (rc, lines.len())
}

// ---- the body --------------------------------------------------------------------------------

/// The window's body (progress.rs's `Progress::body`): the compact block, Details when shown,
/// and the footer. The action asked for.
#[allow(clippy::too_many_arguments)]
pub(super) fn body(
    pr: &mut Progress,
    ui: &mut Ui,
    o: Pos2,
    item: &Item,
    snap: Option<&Snapshot>,
    drawn: (f64, &[(u64, f64)]),
    shown: &Shown,
    d: Option<&Details>,
) -> Action {
    let p = ui.painter().clone();
    let kind = Kind::of(item);
    compact(pr, ui, &p, o, item, snap, drawn, shown, d);

    let mut action = Action::None;
    if pr.details {
        pr.init_limits(item);
        let top = facts_top(under_lines(&shown.view));
        action = left_details(pr, ui, &p, o, item, snap, d, top);
        // The right column's tables reach down to the left column's end.
        let bottom = facts_end(kind, top);
        let right = match kind {
            Kind::Torrent => torrent_column(pr, ui, &p, o, item, d, bottom),
            Kind::File => {
                let y = graph(ui, &p, o, pr);
                connections(ui, &p, o, y + 16.0, snap, bottom);
                Action::None
            }
            Kind::Video => {
                graph(ui, &p, o, pr);
                Action::None
            }
        };
        if matches!(action, Action::None) {
            action = right;
        }
    }
    let footer = footer_row(pr, ui, &p, o, item, shown, kind);
    if matches!(action, Action::None) {
        footer
    } else {
        action
    }
}

/// A fact's key at the start of its row (12, muted), naming the box or list after it.
fn key(p: &egui::Painter, o: Pos2, y: f32, tall: bool, label: &str) {
    let base = if tall { 17.0 } else { 12.0 };
    winui::caption_font(
        p,
        face(12.0),
        o.x + EDGE,
        o.y + y + base,
        crate::i18n::t(label),
        muted(),
    );
}

/// Details' left column under the compact block, from `top`: the facts of `kind` ([`facts`]).
/// The action asked for (the folder or copy buttons).
#[allow(clippy::too_many_arguments)]
fn left_details(
    pr: &mut Progress,
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    item: &Item,
    snap: Option<&Snapshot>,
    d: Option<&Details>,
    top: f32,
) -> Action {
    let mut action = Action::None;
    let right = o.x + EDGE + LEFT_W;
    let x = o.x + VALUE_X;
    let st = d.and_then(|d| d.stats.as_ref());
    // A value with a 24-point button right after it (2 apart).
    let button_after = |rc: Rect, y: f32| {
        let bx = (rc.right() + 2.0).min(right - ICON_BUTTON) - o.x;
        r(o, bx, y - 4.0, ICON_BUTTON, ICON_BUTTON)
    };
    let mut y = top;
    for fact in facts(Kind::of(item)) {
        let base = o.y + y + 12.0;
        match fact {
            Fact::Saved => {
                // i18n: label in a download window's details: the folder the download is saved in
                key(p, o, y, false, crate::i18n::mark("Saved to"));
                let folder = match &item.torrent {
                    Some(spec) => spec.folder.clone(),
                    None => item
                        .dest
                        .parent()
                        .map(|f| f.to_path_buf())
                        .unwrap_or_default(),
                };
                let path = crate::dialogs::display_path(&folder);
                let f = face(13.0);
                let shown = winui::ellipsize_middle(p, &f, &path, right - ICON_BUTTON - 2.0 - x);
                let rc = winui::text_font(p, f, x, base, &shown, ink());
                winui::copyable(ui, rc, "saved-to", &path, shown != path);
                // i18n: a download window's button beside its folder: shows the download in its folder
                let tip = crate::i18n::mark("Open folder");
                if icon_button(ui, button_after(rc, y), "cat-folder-open", tip).clicked() {
                    action = Action::OpenFolder;
                }
            }
            Fact::Source => {
                // i18n: label in a download window's details: where the download comes from (its address)
                key(p, o, y, false, crate::i18n::mark("From"));
                if let Some(spec) = &item.torrent {
                    // i18n: a torrent's From row in its download window: it comes from a magnet link (the button beside it copies it)
                    let label = crate::i18n::t("Magnet link");
                    let rc = winui::text_font(p, face(13.0), x, base, label, secondary());
                    let tip = crate::i18n::fmt(
                        // i18n: tooltip on a torrent's From row: {hash} = the torrent's info hash
                        crate::i18n::t("Info hash {hash}"),
                        &[("hash", &crate::i18n::ltr(&spec.hash))],
                    );
                    // A Tab stop of its own, so the keyboard reads its info hash too.
                    let id = egui::Id::unique(("from-hash", ui.ctx().viewport_id()));
                    let resp = ui.interact(rc, id, Sense::focusable_noninteractive());
                    winui::expose(&resp, egui::Role::Label, label, true, None);
                    // Its info hash read after its name, as its tooltip shows it.
                    let described = tip.clone();
                    resp.ctx
                        .accesskit_node_builder(resp.id, |n| n.set_description(described));
                    winui::show_focus(ui, &resp, rc.expand(2.0));
                    winui::tip(resp, &tip);
                    let copy = crate::i18n::mark("Copy magnet link");
                    if icon_button(ui, button_after(rc, y), "gx-copy", copy).clicked() {
                        action = Action::Torrent(TorrentCmd::CopyMagnet);
                    }
                } else {
                    let url = crate::i18n::ltr(item.url.as_str());
                    let rc = cell(
                        ui,
                        p,
                        figures(13.0),
                        x,
                        base,
                        right - ICON_BUTTON - 2.0,
                        &url,
                        secondary(),
                        "from-url",
                    );
                    let copy = crate::i18n::mark("Copy address");
                    if icon_button(ui, button_after(rc, y), "gx-copy", copy).clicked() {
                        action = Action::CopyAddress;
                    }
                }
            }
            Fact::Server => {
                // i18n: label before where a download's server is (its country and round trip)
                key(p, o, y, false, crate::i18n::mark("Server"));
                // Where its server is (#140), as the running download sees it, else as its
                // last run did.
                let seen = snap
                    .and_then(|s| s.server.as_ref())
                    .map(crate::whereis::Seen::from)
                    .or_else(|| item.server.clone());
                crate::whereis::line(
                    ui,
                    x,
                    base,
                    right,
                    seen.as_ref(),
                    &format!("progress-{}", item.id),
                );
            }
            Fact::CanResume => {
                // i18n: label in a download window's details: whether the server lets a stopped download continue (Yes / No)
                key(p, o, y, false, crate::i18n::mark("Can resume"));
                let v = match item.resumable {
                    Some(true) => crate::i18n::t("Yes"),
                    Some(false) => crate::i18n::t("No"),
                    None => "",
                };
                winui::text_font(p, face(13.0), x, base, v, ink());
            }
            Fact::Format => {
                // i18n: label in a video's download window details: the kind of file it makes
                key(p, o, y, false, crate::i18n::mark("Format"));
                cell(
                    ui,
                    p,
                    face(13.0),
                    x,
                    base,
                    right,
                    &format_of(item),
                    ink(),
                    "format",
                );
            }
            Fact::By => {
                // i18n: label in a video's download window details: the program that downloads it
                key(p, o, y, false, crate::i18n::mark("Downloaded by"));
                let why = pr.sample_helper.clone().or_else(|| item.video_helper());
                let by = match (&why, item.gallery.is_some()) {
                    (Some(why), _) => super::helper_line(why),
                    // i18n: a gallery's "Downloaded by" row: the helper program that downloads it
                    (None, true) => crate::i18n::t("gallery-dl").to_owned(),
                    // i18n: a video's "Downloaded by" row: Grabbit downloads its streams itself
                    (None, false) => crate::i18n::t("Grabbit").to_owned(),
                };
                cell(ui, p, face(13.0), x, base, right, &by, secondary(), "by");
            }
            Fact::Peers => {
                key(p, o, y, false, crate::i18n::mark("Peers"));
                let peers = st.map_or(String::new(), |s| {
                    crate::i18n::fmt(
                        // i18n: {live} = peers connected now; {known} = peers heard of
                        crate::i18n::t("{live} connected, {known} known"),
                        &[
                            ("live", &crate::i18n::grouped(s.peers_live as u64)),
                            ("known", &crate::i18n::grouped(s.peers_seen as u64)),
                        ],
                    )
                });
                cell(ui, p, figures(13.0), x, base, right, &peers, ink(), "peers");
            }
            Fact::Sent => {
                // What it sent in all its runs (#39; librqbit counts only this one), against
                // what it has; Ratio beside it (the boards' 88 | 1fr | 56 | 1fr).
                // i18n: how much was sent to other peers
                key(p, o, y, false, crate::i18n::mark("Sent"));
                if let Some(spec) = &item.torrent {
                    let half = (LEFT_W - 88.0 - 56.0) / 2.0;
                    let ratio_x = VALUE_X + half;
                    let sent = crate::model::size(spec.uploaded);
                    let end = o.x + ratio_x - GAP;
                    cell(ui, p, figures(13.0), x, base, end, &sent, ink(), "sent");
                    // i18n: label in a torrent's download window details: sent divided by received
                    let ratio_label = crate::i18n::t("Ratio");
                    winui::text_font(p, face(12.0), o.x + ratio_x, base, ratio_label, muted());
                    let done = st.map_or(item.downloaded, |s| s.done);
                    let ratio = crate::torrents::share_ratio_text(spec.uploaded, done);
                    let rx = o.x + ratio_x + 56.0;
                    cell(
                        ui,
                        p,
                        figures(13.0),
                        rx,
                        base,
                        right,
                        &ratio,
                        ink(),
                        "ratio",
                    );
                }
            }
            Fact::Pieces => {
                key(p, o, y, false, crate::i18n::mark("Pieces"));
                let pieces = d.map_or(String::new(), |d| pieces_line(&d.pieces, d.piece_length));
                cell(
                    ui,
                    p,
                    figures(13.0),
                    x,
                    base,
                    right,
                    &pieces,
                    ink(),
                    "pieces",
                );
            }
            Fact::Limit => limits(pr, ui, p, o, item.torrent.is_some(), y),
            Fact::WhenDone => when_done(pr, ui, p, o, item.torrent.is_some(), y),
            // Drawn with Limit and with When done.
            Fact::Remember | Fact::Note | Fact::Force => {}
        }
        y += fact.height() + GAP;
    }
    action
}

/// The Pieces row: "98 of 1,470, 512.00 KB each", the exact count the strip can't show (a
/// torrent stuck on its last piece shows it).
pub fn pieces_line(pieces: &[bool], piece_length: u32) -> String {
    crate::i18n::fmt(
        // i18n: a torrent's Pieces row: {have} and {total} = numbers of pieces; {size} = one piece's size
        crate::i18n::t("{have} of {total}, {size} each"),
        &[
            (
                "have",
                &crate::i18n::grouped(pieces.iter().filter(|b| **b).count() as u64),
            ),
            ("total", &crate::i18n::grouped(pieces.len() as u64)),
            ("size", &crate::model::size(u64::from(piece_length))),
        ],
    )
}

/// A video's Format row: its file's kind, and whether its picture and sound are joined at the
/// end (a selector of two streams); a gallery's pictures.
fn format_of(item: &Item) -> String {
    let Some(video) = &item.video else {
        // i18n: a gallery's Format row in its download window
        return crate::i18n::t("Pictures").to_owned();
    };
    let kind = std::path::Path::new(&item.file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_uppercase())
        .unwrap_or_default();
    if video.selector.contains('+') && kind.is_empty() {
        // i18n: a video's Format row when its file's kind isn't known yet: its picture and sound come apart and are joined when both are in
        crate::i18n::t("Picture and sound joined at the end").to_owned()
    } else if video.selector.contains('+') {
        crate::i18n::fmt(
            // i18n: a video's Format row: {kind} = its file's kind (MP4); its picture and sound come apart and are joined when both are in
            crate::i18n::t("{kind}, picture and sound joined at the end"),
            &[("kind", &kind)],
        )
    } else {
        kind
    }
}

/// The Limit row (the old Speed Limiter tab's, plan 13b) at `y`: this download's limit, a
/// torrent's upload limit beside it; Remember for this file and its note under it. Reported to
/// the app as typed.
fn limits(pr: &mut Progress, ui: &mut Ui, p: &egui::Painter, o: Pos2, torrent: bool, y: f32) {
    // i18n: label in a download window's details: its speed limit
    key(p, o, y, true, crate::i18n::mark("Limit"));
    // A typed rate applies once typed (focus leaves, Enter), not at each digit (typing 100 would
    // throttle to 1, then 10 KB/s on the way). A rate a screen reader or UI Automation sets (Set
    // value) applies at once: there is no typing to wait for, and often no focus to leave.
    let typed = |ui: &Ui, rate_box: &egui::Response| {
        rate_box.lost_focus()
            || (rate_box.has_focus() && winui::enter_pressed(ui))
            || winui::value_set_by_assistive(rate_box)
    };
    let off = |on: bool| if on { muted() } else { super::disabled() };
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
        o + vec2(VALUE_X, y + 7.0),
        &mut pr.limiter,
        down_label,
        true,
        Some(o.x + box_x - 6.0),
    );
    let rate_box = winui::edit_right(
        ui,
        r(o, box_x, y, box_w, ROW_H),
        &mut pr.limit_kb,
        pr.limiter,
    );
    pr.limit_kb.retain(|c| c.is_ascii_digit());
    let down_typed = typed(ui, &rate_box);
    let mut unit_x = box_x + box_w + GAP;
    if torrent {
        let was_up = pr.up_on;
        // Board 3: Up 16 after the Down box, its box 8 after its caption (the check box's
        // caption is 16 after the box's start, winui::checkbox_light_until).
        let up_x = box_x + box_w + 16.0;
        // i18n: a torrent's Limit row: the upload limit's check box (a limit for what it sends to others)
        let up_label = crate::i18n::mark("Up");
        let up_box_x = up_x + 16.0 + winui::text_width(p, crate::i18n::t(up_label)).ceil() + GAP;
        winui::checkbox_until(
            ui,
            o + vec2(up_x, y + 7.0),
            &mut pr.up_on,
            up_label,
            true,
            // Up to its box: checkbox_until keeps 5 before `until` itself, so the caption has
            // its width and 3 to spare (6 less here cut it to nothing: its room was 3 short).
            Some(o.x + up_box_x),
        );
        let up_box = winui::edit_right(ui, r(o, up_box_x, y, 56.0, ROW_H), &mut pr.up_kb, pr.up_on);
        pr.up_kb.retain(|c| c.is_ascii_digit());
        let kb: u64 = pr.up_kb.parse().unwrap_or(0);
        if (was_up != pr.up_on || (pr.up_on && typed(ui, &up_box))) && !(pr.up_on && kb == 0) {
            pr.pending_upload = Some(pr.up_on.then_some(kb * 1024));
        }
        unit_x = up_box_x + 56.0 + GAP;
    }
    winui::text_font(
        p,
        face(12.0),
        o.x + unit_x,
        o.y + y + 17.0,
        // i18n: unit after a speed limit box: kilobytes per second
        crate::i18n::t("KB/s"),
        off(pr.limiter || pr.up_on),
    );
    let remembered = pr.remember;
    let y = y + ROW_H + GAP;
    winui::checkbox(
        ui,
        o + vec2(VALUE_X, y + 2.0),
        &mut pr.remember,
        // i18n: a download window's check box under its speed limit: keep the limit when the download stops and resumes
        "Remember for this file",
        true,
    );
    winui::text_font(
        p,
        face(12.0),
        o.x + VALUE_X,
        o.y + y + 16.0 + GAP + 12.0,
        // i18n: the note under "Remember for this file" in a download window
        crate::i18n::t("Keeps this limit when the download stops and resumes."),
        muted(),
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

/// When done's list at `y`, "Then keeps sharing" beside its first two choices for a torrent
/// that will, and "Force apps to close" under Shut down, Sleep and Hibernate.
fn when_done(pr: &mut Progress, ui: &mut Ui, p: &egui::Painter, o: Pos2, torrent: bool, y: f32) {
    // i18n: label in a download window's details: what happens when the download finishes
    key(p, o, y, true, crate::i18n::mark("When done"));
    let w = if torrent { 184.0 } else { 200.0 };
    if let Some(i) = winui::combo_light(
        ui,
        r(o, VALUE_X, y, w, ROW_H),
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
            o.x + VALUE_X + w + 12.0,
            o.y + y + 17.0,
            // i18n: beside a torrent's When done choice: it goes on sharing after it finishes
            crate::i18n::t("Then keeps sharing"),
            muted(),
            o.x + EDGE + LEFT_W,
            "keeps-sharing",
        );
    }
    if pr.when >= 3 {
        winui::checkbox(
            ui,
            o + vec2(VALUE_X, y + ROW_H + GAP + 2.0),
            &mut pr.force,
            // i18n: under When done's Shut down, Sleep or Hibernate: don't let other programs delay it
            "Force apps to close",
            true,
        );
    }
}

// ---- the right column ------------------------------------------------------------------------

/// The last minute of a download's speed for its window's graph (plan 13): one reading a
/// second, 60 at most, kept while the window is open and never saved.
#[derive(Default)]
pub struct Sampler {
    readings: VecDeque<f64>,
    /// When the last reading was taken (egui's clock, seconds).
    last: Option<f64>,
}

impl Sampler {
    pub const KEPT: usize = 60;

    /// The download's `rate` (bytes a second) at `now` (seconds): kept when a second has gone
    /// since the last reading. The seconds missed in between (the window hidden in the
    /// notification area, no frames while stopped) read 0; the clock moves by whole seconds, so
    /// the readings don't drift.
    pub fn feed(&mut self, now: f64, rate: f64) {
        if let Some(last) = self.last {
            let gone = (now - last).floor();
            if gone < 1.0 {
                return;
            }
            for _ in 0..((gone as usize) - 1).min(Self::KEPT) {
                self.push(0.0);
            }
            self.last = Some(last + gone);
        } else {
            self.last = Some(now);
        }
        self.push(rate.max(0.0));
    }

    fn push(&mut self, rate: f64) {
        if self.readings.len() == Self::KEPT {
            self.readings.pop_front();
        }
        self.readings.push_back(rate);
    }

    /// Oldest first.
    pub fn readings(&self) -> &VecDeque<f64> {
        &self.readings
    }
}

/// A header over a part of the right column at `y`: its title, a figure at its end, a line
/// under it.
fn header(p: &egui::Painter, o: Pos2, y: f32, title: &str, end: &str, end_color: Color32) {
    let base = o.y + y + 12.0;
    winui::text_font(p, face(12.0), o.x + RIGHT_X, base, title, muted());
    draw_right(
        p,
        figures(12.0),
        o.x + RIGHT_X + RIGHT_W,
        base,
        end,
        end_color,
    );
    let line = crate::theme::colors().header_line;
    winui::solid(p, r(o, RIGHT_X, y + HEAD - 1.0, RIGHT_W, 1.0), line);
}

/// The speed graph at the right column's top (boards 2, 11): the last 60 s with the limit as a
/// dashed line; its summary for screen readers. Where it ends.
fn graph(ui: &Ui, p: &egui::Painter, o: Pos2, pr: &Progress) -> f32 {
    let limit = pr.limit.map(|l| l.rate() as f64);
    let limit_text = limit.map_or(String::new(), |l| {
        crate::i18n::fmt(
            // i18n: at the end of the speed graph's header: {rate} = the speed limit
            crate::i18n::t("limit {rate}"),
            &[("rate", &crate::model::limit_rate(l as u64))],
        )
    });
    // i18n: the speed graph's header in a download window's details
    header(
        p,
        o,
        TOP,
        crate::i18n::t("Speed, last 60 s"),
        &limit_text,
        muted(),
    );
    let top = TOP + HEAD + GAP;
    let rc = r(o, RIGHT_X, top, RIGHT_W, GRAPH_H);
    let readings = pr.speed.readings();
    let peak = readings.iter().copied().fold(0.0, f64::max);
    // The higher of the peak and the limit 4 points under the graph's top.
    let scale = peak.max(limit.unwrap_or(0.0));
    let y_of = |v: f64| {
        if scale > 0.0 {
            rc.bottom() - (v / scale) as f32 * (GRAPH_H - 4.0)
        } else {
            rc.bottom() - 1.0
        }
    };
    if let Some(l) = limit {
        let y = y_of(l);
        p.extend(egui::Shape::dashed_line(
            &[pos2(rc.left(), y), pos2(rc.right(), y)],
            Stroke::new(1.0, muted()),
            3.0,
            3.0,
        ));
    }
    let n = readings.len();
    if n > 1 {
        let step = RIGHT_W / (Sampler::KEPT - 1) as f32;
        let points: Vec<Pos2> = readings
            .iter()
            .enumerate()
            .map(|(k, v)| pos2(rc.right() - (n - 1 - k) as f32 * step, y_of(*v)))
            .collect();
        p.add(egui::Shape::line(points, Stroke::new(1.0, secondary())));
    }
    let now = crate::model::rate(readings.back().copied().unwrap_or(0.0));
    let peak_s = crate::model::rate(peak);
    let summary = match limit {
        Some(l) => crate::i18n::fmt(
            // i18n: the speed graph for screen readers: {now} the speed now, {peak} the fastest in the last minute, {limit} the speed limit
            crate::i18n::t("Speed, last minute: {now} now, peak {peak}, limit {limit}"),
            &[
                ("now", &now),
                ("peak", &peak_s),
                ("limit", &crate::model::limit_rate(l as u64)),
            ],
        ),
        None => crate::i18n::fmt(
            // i18n: the speed graph for screen readers: {now} the speed now, {peak} the fastest in the last minute
            crate::i18n::t("Speed, last minute: {now} now, peak {peak}"),
            &[("now", &now), ("peak", &peak_s)],
        ),
    };
    let id = egui::Id::unique(("progress-graph", ui.ctx().viewport_id()));
    let resp = ui.interact(rc, id, Sense::hover());
    winui::expose(&resp, egui::Role::Image, &summary, true, None);
    top + GRAPH_H
}

/// A table's column: its title (English), its width (0: the room left), its text at its end.
struct Col {
    title: &'static str,
    w: f32,
    end: bool,
}

/// A table's cell: its text, its colour, in figures or not.
type Cell = (String, Color32, bool);

/// A table of the right column at `y` (boards 2, 3, 10, 12): its column titles in a 24-point
/// row, then rows 26 high each with a line under it, `shown` at once, scrolled by the wheel and
/// a thin bar at its end (room kept for it). `checks`: a box before each row (a torrent's
/// Files, the first column's); the row whose box was clicked.
#[allow(clippy::too_many_arguments)]
fn table(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    y: f32,
    shown: usize,
    id: &str,
    cols: &[Col],
    rows: &[Vec<Cell>],
    checks: Option<&[bool]>,
) -> Option<usize> {
    const COL_GAP: f32 = 12.0;
    const BAR: f32 = 8.0;
    let width = RIGHT_W - BAR;
    let fixed: f32 = cols.iter().map(|c| c.w).sum::<f32>() + COL_GAP * (cols.len() - 1) as f32;
    let mut x = RIGHT_X;
    let spans: Vec<(f32, f32)> = cols
        .iter()
        .map(|c| {
            let w = if c.w > 0.0 { c.w } else { width - fixed };
            let span = (x, x + w);
            x += w + COL_GAP;
            span
        })
        .collect();
    let grid = crate::theme::colors().grid;
    for (c, &(a, b)) in cols.iter().zip(&spans) {
        let title = crate::i18n::t(c.title);
        let base = o.y + y + 16.0;
        if c.end {
            draw_right(p, face(12.0), o.x + b, base, title, muted());
        } else {
            winui::text_font(p, face(12.0), o.x + a, base, title, muted());
        }
    }
    winui::solid(p, r(o, RIGHT_X, y + TITLES_H - 1.0, RIGHT_W, 1.0), grid);
    let top = y + TITLES_H;
    let body_h = shown as f32 * ROW_H;
    let body = r(o, RIGHT_X, top, RIGHT_W, body_h);
    let key = egui::Id::unique(("progress-table", id, ui.ctx().viewport_id()));
    let first = super::first_row(ui, key, body, rows.len(), shown);
    let clip = p.with_clip_rect(body);
    let mut clicked = None;
    for (k, row) in rows.iter().enumerate().skip(first).take(shown) {
        let row_top = top + (k - first) as f32 * ROW_H;
        let base = o.y + row_top + 17.0;
        let mut cells = row.iter();
        for (c, &(a, b)) in cols.iter().zip(&spans) {
            if c.title.is_empty()
                && let Some(&ticked) = checks.and_then(|t| t.get(k))
            {
                // The box, named by its row's first text.
                let mut on = ticked;
                let label = row.first().map_or("", |c| c.0.as_str());
                let at = pos2(o.x + a, o.y + row_top + 6.5);
                if winui::row_check(ui, &clip, at, &mut on, label, (id, k)) {
                    clicked = Some(k);
                }
                continue;
            }
            let Some((s, color, nums)) = cells.next() else {
                break;
            };
            let f = if *nums { figures(12.0) } else { face(12.0) };
            let salt = (id, k, a as i32);
            if c.end {
                cell_right(ui, &clip, f, o.x + a, base, o.x + b, s, *color, salt);
            } else {
                cell(ui, &clip, f, o.x + a, base, o.x + b, s, *color, salt);
            }
        }
        winui::solid(
            &clip,
            r(o, RIGHT_X, row_top + ROW_H - 1.0, RIGHT_W, 1.0),
            grid,
        );
    }
    // A thin bar inside the room kept at the column's end.
    if rows.len() > shown {
        let track = r(o, RIGHT_X + RIGHT_W - 6.0, top, 6.0, body_h);
        let mut pos = first as f32;
        let (total, page) = (rows.len() as f32, shown as f32);
        if crate::scrollbar::thin(ui, key.with("bar"), track, total, page, &mut pos) {
            let last = super::last_first_row(rows.len(), shown);
            let first = (pos.round().max(0.0) as usize).min(last);
            ui.ctx().data_mut(|d| d.insert_temp(key, first));
        }
    }
    clicked
}

/// Where a connection sorts: receiving first, then retrying (or failed), connecting, done.
fn conn_rank(s: &ConnStatus) -> u8 {
    match s {
        ConnStatus::Receiving => 0,
        ConnStatus::Retrying(_) | ConnStatus::Failed(_) => 1,
        ConnStatus::Connecting => 2,
        ConnStatus::Finished | ConnStatus::Retired { .. } => 3,
    }
}

/// The connections at `y` under the speed graph (boards 2, 10): number, received, speed or
/// what it is doing; receiving ones first ([`conn_rank`]), by number within each group. As many
/// whole rows as reach down to `bottom`.
fn connections(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    y: f32,
    snap: Option<&Snapshot>,
    bottom: f32,
) {
    let mut conns: Vec<_> = snap
        .map(|s| s.connections.iter().collect())
        .unwrap_or_default();
    // By number within each group: rows that don't move as speeds change.
    conns.sort_by_key(|c| (conn_rank(&c.status), c.id));
    let receiving = conns
        .iter()
        .filter(|c| matches!(c.status, ConnStatus::Receiving))
        .count();
    let count = crate::i18n::fmt(
        // i18n: at the end of the connections table's header: {n} connections, {receiving} of them receiving
        crate::i18n::t("{n}, {receiving} receiving"),
        &[
            ("n", &crate::i18n::grouped(conns.len() as u64)),
            ("receiving", &crate::i18n::grouped(receiving as u64)),
        ],
    );
    // i18n: caption over a download's connections table in its window's details
    header(p, o, y, crate::i18n::t("Connections"), &count, muted());
    let rows: Vec<Vec<Cell>> = conns
        .iter()
        .map(|c| {
            let (info, color, nums) = match &c.status {
                ConnStatus::Receiving => (crate::model::rate(c.speed as f64), secondary(), true),
                ConnStatus::Connecting => {
                    (crate::i18n::t("Connecting...").to_owned(), muted(), false)
                }
                ConnStatus::Retrying(e) => (
                    crate::i18n::fmt(
                        // i18n: a connection's state: it failed and tries again, with the reason
                        crate::i18n::t("Retrying: {error}"),
                        &[("error", &crate::errors::describe(e))],
                    ),
                    muted(),
                    false,
                ),
                // i18n: a connection's state: it has received all of its part
                ConnStatus::Finished => (crate::i18n::t("Done").to_owned(), muted(), false),
                ConnStatus::Failed(e) => (crate::errors::describe(e), winui::ERROR(), false),
                ConnStatus::Retired { host, cap } => (
                    crate::i18n::fmt(
                        // i18n: a connection closed because the server allows only that many
                        &crate::i18n::tn(
                            u64::from(*cap),
                            "{host} allows {n} connection",
                            "{host} allows {n} connections",
                        ),
                        &[("host", &crate::i18n::ltr(host))],
                    ),
                    muted(),
                    false,
                ),
            };
            vec![
                ((c.id + 1).to_string(), muted(), true),
                (crate::model::size(c.downloaded), ink(), true),
                (info, color, nums),
            ]
        })
        .collect();
    let cols = [
        // i18n: column title: connection number
        Col {
            title: crate::i18n::mark("#"),
            w: 20.0,
            end: false,
        },
        Col {
            title: crate::i18n::mark("Received"),
            w: 0.0,
            end: true,
        },
        // i18n: column title: how fast a connection receives, or what it is doing
        Col {
            title: crate::i18n::mark("Speed"),
            w: 116.0,
            end: true,
        },
    ];
    let y = y + HEAD;
    let shown = rows_down_to(y, bottom);
    table(ui, p, o, y, shown, "connections", &cols, &rows, None);
}

/// A row of tabs at the right column's `y`, underlined (board 3): the chosen one in the ink with
/// the accent under it; one Tab stop, the chosen tab; the arrows, Home and End choose. The tab
/// chosen this frame.
fn tabs(
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    y: f32,
    labels: &[&str],
    current: usize,
) -> Option<usize> {
    let line = crate::theme::colors().header_line;
    winui::solid(p, r(o, RIGHT_X, y + HEAD - 1.0, RIGHT_W, 1.0), line);
    let id_of = |ui: &Ui, k: usize| egui::Id::unique(("progress-tab", ui.ctx().viewport_id(), k));
    let mut chosen = None;
    let mut x = RIGHT_X;
    for (i, label) in labels.iter().enumerate() {
        let label = crate::i18n::t(label);
        let w = winui::text_width_font(p, medium(12.0), label).ceil();
        let rc = r(o, x, y, w, HEAD - 1.0);
        let sel = i == current;
        let sense = if sel {
            Sense::click()
        } else {
            winui::pointer_only()
        };
        let resp = ui.interact(rc, id_of(ui, i), sense);
        winui::focus_on_press(&resp);
        winui::expose(&resp, egui::Role::Tab, label, true, Some(sel));
        let resp = winui::enter_is_not_a_click(ui, resp);
        if resp.clicked() && !sel {
            chosen = Some(i);
        }
        if sel {
            winui::keep_keys(ui, &resp, true, false, false);
            if resp.has_focus() {
                let last = labels.len().saturating_sub(1) as isize;
                let to = if winui::take_key(ui, egui::Key::ArrowRight) {
                    Some(i as isize + 1)
                } else if winui::take_key(ui, egui::Key::ArrowLeft) {
                    Some(i as isize - 1)
                } else if winui::take_key(ui, egui::Key::Home) {
                    Some(0)
                } else if winui::take_key(ui, egui::Key::End) {
                    Some(last)
                } else {
                    None
                };
                if let Some(to) = to.map(|t| t.clamp(0, last) as usize).filter(|t| *t != i) {
                    chosen = Some(to);
                    let next = id_of(ui, to);
                    ui.memory_mut(|m| m.request_focus(next));
                }
            }
        }
        let c = if sel || resp.hovered() {
            ink()
        } else {
            muted()
        };
        winui::text_font(p, medium(12.0), o.x + x, o.y + y + 15.0, label, c);
        if sel {
            let accent = crate::theme::colors().accent;
            winui::solid(p, r(o, x, y + HEAD - 3.0, w, 2.0), accent);
        }
        winui::show_focus(ui, &resp, rc.expand(2.0));
        x += w + 20.0;
    }
    chosen
}

/// A torrent's right column (boards 3, 12): Files, Peers and Trackers over their table, Recheck
/// files at the tabs' end on Files, each tab's commands under its table; the tables as long as
/// they can be with their commands under them by `bottom`. The command asked for.
#[allow(clippy::too_many_arguments)]
fn torrent_column(
    pr: &mut Progress,
    ui: &mut Ui,
    p: &egui::Painter,
    o: Pos2,
    item: &Item,
    d: Option<&Details>,
    bottom: f32,
) -> Action {
    let tabs_list = [
        // i18n: tab title: the torrent's files
        (TTab::Files, crate::i18n::mark("Files")),
        // i18n: tab title: the other computers sharing the torrent
        (TTab::Peers, crate::i18n::mark("Peers")),
        // i18n: tab title: the servers that list the peers
        (TTab::Trackers, crate::i18n::mark("Trackers")),
    ];
    let labels: Vec<&str> = tabs_list.iter().map(|t| t.1).collect();
    let current = tabs_list.iter().position(|t| t.0 == pr.ttab).unwrap_or(0);
    if let Some(i) = tabs(ui, p, o, TOP, &labels, current) {
        pr.ttab = tabs_list[i].0;
    }
    let y = TOP + HEAD;
    let shown = rows_down_to(y, bottom - COMMANDS_H);
    let under = y + TITLES_H + shown as f32 * ROW_H + 12.0;
    let action = torrent_commands(pr, ui, o, item, d, under);
    let Some(d) = d else {
        winui::text_font(
            p,
            face(12.0),
            o.x + RIGHT_X,
            o.y + y + TITLES_H + 17.0,
            crate::i18n::t("Shown while the torrent is downloading."),
            muted(),
        );
        return action;
    };
    match pr.ttab {
        TTab::Files => {
            let rows: Vec<Vec<Cell>> = d
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
                    vec![
                        (f.path.display().to_string(), ink(), false),
                        (crate::model::size(f.size), ink(), true),
                        (pct, secondary(), true),
                    ]
                })
                .collect();
            let ticks: Vec<bool> = d.files.iter().map(|f| f.2).collect();
            let cols = [
                Col {
                    title: "",
                    w: 16.0,
                    end: false,
                },
                // i18n: column title: a torrent file's name
                Col {
                    title: crate::i18n::mark("Name"),
                    w: 0.0,
                    end: false,
                },
                Col {
                    title: crate::i18n::mark("Size"),
                    w: 64.0,
                    end: true,
                },
                Col {
                    title: crate::i18n::mark("Done"),
                    w: 40.0,
                    end: true,
                },
            ];
            // A box ticked or unticked (#39): the torrent downloads those files from now on;
            // the last one ticked can't be unticked (a torrent of nothing).
            let clicked = table(ui, p, o, y, shown, "t-files", &cols, &rows, Some(&ticks));
            if let Some(row) = clicked {
                let mut on = ticks.clone();
                on[row] = !on[row];
                if on.iter().any(|b| *b) {
                    let chosen = d.files.iter().zip(&on).filter(|(_, b)| **b);
                    pr.pending_choose = Some(Action::Choose {
                        files: chosen.clone().map(|(f, _)| f.0).collect(),
                        all: on.iter().all(|b| *b),
                        size: chosen.map(|(f, _)| f.1.size).sum(),
                    });
                }
            }
        }
        TTab::Peers => {
            let rows: Vec<Vec<Cell>> = d
                .peers
                .iter()
                .map(|q| {
                    vec![
                        (q.addr.clone(), ink(), true),
                        (q.client.clone().unwrap_or_default(), secondary(), false),
                        (crate::model::size(q.fetched), ink(), true),
                        (crate::model::size(q.uploaded), secondary(), true),
                    ]
                })
                .collect();
            let cols = [
                // i18n: column title: the web address (URL)
                Col {
                    title: crate::i18n::mark("Address"),
                    w: 0.0,
                    end: false,
                },
                // i18n: column title: the peer's torrent program
                Col {
                    title: crate::i18n::mark("Client"),
                    w: 72.0,
                    end: false,
                },
                // i18n: column title: how much came from the peer
                Col {
                    title: crate::i18n::mark("Received"),
                    w: 52.0,
                    end: true,
                },
                // i18n: column title: how much went to the peer
                Col {
                    title: crate::i18n::mark("Sent"),
                    w: 44.0,
                    end: true,
                },
            ];
            table(ui, p, o, y, shown, "t-peers", &cols, &rows, None);
        }
        TTab::Trackers => {
            let rows: Vec<Vec<Cell>> = if d.trackers.is_empty() {
                vec![vec![(
                    // i18n: a torrent's Trackers tab without trackers
                    crate::i18n::t("None. Grabbit still finds peers without trackers.").into(),
                    muted(),
                    false,
                )]]
            } else {
                d.trackers
                    .iter()
                    .map(|t| vec![(t.clone(), ink(), true)])
                    .collect()
            };
            // i18n: column title: a server that lists the peers
            let cols = [Col {
                title: crate::i18n::mark("Tracker"),
                w: 0.0,
                end: false,
            }];
            table(ui, p, o, y, shown, "t-trackers", &cols, &rows, None);
        }
    }
    action
}

/// A torrent's commands (plan 13b) on its tabs: Recheck files at the tabs' end on Files, Download
/// in order and First and last pieces first under its table; Add trackers... and Ask trackers
/// again under the Trackers table; `under`: where the commands under a table go. The one
/// pressed.
fn torrent_commands(
    pr: &mut Progress,
    ui: &mut Ui,
    o: Pos2,
    item: &Item,
    d: Option<&Details>,
    under: f32,
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
    let width = |label: &str| (winui::text_width(&p, crate::i18n::t(label)) + 24.0).ceil();
    let mut action = Action::None;
    match pr.ttab {
        TTab::Files => {
            // i18n: a torrent's Files tab, a button: read every piece of the torrent's files again and check it
            let label = crate::i18n::mark("Recheck files");
            let w = width(label);
            let on = !busy && item.status != Status::Queued;
            let rc = r(o, RIGHT_X + RIGHT_W - w, TOP, w, HEAD - 1.0);
            let look = if on {
                ButtonKind::Ghost
            } else {
                ButtonKind::Disabled
            };
            let resp = winui::button(ui, rc, label, look, false, None);
            if on && resp.clicked() {
                action = Action::Torrent(TorrentCmd::Recheck);
            }
            // i18n: tooltip of a torrent's Recheck files button
            winui::tip(
                resp,
                crate::i18n::t("Reads every piece again and checks it"),
            );
            // The boxes show the torrent's own choices; a click asks the app to change them.
            let mut in_order = spec.in_order;
            // i18n: a torrent's Files tab, a check box: download its files from their start, to watch a video while it downloads
            if winui::checkbox(
                ui,
                o + vec2(RIGHT_X, under + 2.0),
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
                o + vec2(RIGHT_X + 22.0, under + 16.0 + GAP + 2.0),
                &mut ends,
                // i18n: a torrent's Files tab, a check box under Download in order: each file's first and last pieces first, which a video player needs to start
                "First and last pieces first",
                spec.in_order,
            )
            .clicked()
            {
                action = Action::Torrent(TorrentCmd::EndsFirst);
            }
        }
        TTab::Trackers => {
            let add = crate::i18n::mark("Add trackers...");
            // i18n: a torrent's Trackers tab, a button: ask the torrent's trackers for peers again now
            let again = crate::i18n::mark("Ask trackers again");
            let add_w = width(add);
            if winui::button(
                ui,
                r(o, RIGHT_X, under, add_w, BUTTON_H),
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
                r(o, RIGHT_X + add_w + GAP, under, width(again), BUTTON_H),
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
    shown: &Shown,
    kind: Kind,
) -> Action {
    let y = footer_y(pr.details, kind, under_lines(&shown.view));
    let w = if pr.details { DETAILS_W } else { COMPACT_W };
    if details_button(ui, p, o, y, pr.details).clicked() {
        pr.details = !pr.details;
    }

    let phase = shown.phase;
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
        if winui::button(ui, r(o, x, y, *bw, BUTTON_H), label, *kind, false, None).clicked() {
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

/// Plan 12's F9: "Show details" / "Hide details" as a link at the footer's start (its text at
/// the window's edge, 6 of face round it under the pointer), a chevron after it (down: it
/// opens); its expanded state for screen readers.
fn details_button(ui: &mut Ui, p: &egui::Painter, o: Pos2, y: f32, open: bool) -> Response {
    let label = if open {
        // i18n: button at a download window's footer: hide the details (the window gets narrow again)
        crate::i18n::t("Hide details")
    } else {
        // i18n: button at a download window's footer: show the details (the window widens to the right)
        crate::i18n::t("Show details")
    };
    let text_w = winui::text_width_font(p, face(12.0), label).ceil();
    let rc = r(o, EDGE - 6.0, y, 6.0 + text_w + 4.0 + 10.0 + 6.0, BUTTON_H);
    let id = egui::Id::unique(("progress-details", ui.ctx().viewport_id()));
    let resp = ui.interact(rc, id, Sense::click());
    winui::focus_on_press(&resp);
    winui::expose(&resp, egui::Role::Button, label, true, None);
    resp.ctx
        .accesskit_node_builder(resp.id, |n| n.set_expanded(open));
    if resp.hovered() {
        p.rect_filled(rc, 4.0, crate::theme::colors().button_hot);
    }
    let base = o.y + y + 18.0;
    winui::text_font(p, face(12.0), o.x + EDGE, base, label, secondary());
    let c = pos2(o.x + EDGE + text_w + 4.0 + 5.0, o.y + y + BUTTON_H / 2.0);
    let (a, b) = if open { (2.0, -2.0) } else { (-2.0, 2.0) };
    p.line(
        vec![c + vec2(-3.0, a), c + vec2(0.0, b), c + vec2(3.0, a)],
        Stroke::new(1.4, secondary()),
    );
    winui::show_focus(ui, &resp, rc);
    resp
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

// ---- the Done window (plan 13d, boards 5, 7, 13, 14) -----------------------------------------
//
// The progress window's layout language (dialogs.rs's `Complete` keeps what it does): the name
// row, "Done" with the size, the strip whole, the number of files, the facts (Saved to, Main
// file), "Don't show this window again", the footer.

/// The Done window's width (the compact progress window's) and its edges.
pub const DONE_W: f32 = COMPACT_W;
pub const DONE_EDGE: f32 = EDGE;
/// The footer's buttons' height.
pub const DONE_BUTTON_H: f32 = BUTTON_H;
/// A facts row, and a line under Saved to (why the file can't open, a notice).
const DONE_ROW: f32 = 16.0;
const DONE_LINE: f32 = 20.0;

/// The Done window's places (`main`: a Main file row; `lines`: the lines under Saved to): the
/// check box's top, the footer's, and the body's height.
pub fn done_layout(main: bool, lines: usize) -> (f32, f32, f32) {
    let rows = 1 + usize::from(main);
    let end = FACTS_Y + rows as f32 * (DONE_ROW + GAP) + lines as f32 * (DONE_LINE + GAP) - GAP;
    let check = end + 16.0;
    let footer = check + DONE_ROW + 16.0;
    (check, footer, footer + BUTTON_H + BOTTOM)
}

/// Where a row of buttons ending at the window's right edge puts each, of the widths given
/// left to right (`joined`: drawn against the one before it, as one split button): each one's
/// English x.
pub fn row_from_right(parts: &[(f32, bool)]) -> Vec<f32> {
    let mut x = DONE_W - EDGE;
    let mut xs = vec![0.0; parts.len()];
    for i in (0..parts.len()).rev() {
        x -= parts[i].0;
        xs[i] = x;
        if !parts[i].1 {
            x -= GAP;
        }
    }
    xs
}

/// The name row's name and description for screen readers: "{name}, file" (a folder download's
/// "{name}, folder") and what it does.
pub fn name_row_text(name: &str, folder: bool) -> (String, String) {
    let template = if folder {
        // i18n: the Done window's name row for screen readers: {file} the downloaded folder's name
        crate::i18n::t("{file}, folder")
    } else {
        // i18n: the Done window's name row for screen readers: {file} the downloaded file's name
        crate::i18n::t("{file}, file")
    };
    let label = crate::i18n::fmt(template, &[("file", &crate::i18n::isolate(name))]);
    // i18n: the Done window's name row for screen readers: what it does
    let described = crate::i18n::t("Drag it into a folder or an app, or press Ctrl+C to copy it");
    (label, described.to_owned())
}

/// The Done window's name row (board 13): the name; under the pointer or with the keys, the
/// grip at its start and "Drag out, or Ctrl+C" at its end. A Tab stop the file is dragged out
/// of, or copied from with Ctrl+C (the caller does both, dialogs.rs). Its response.
pub(crate) fn name_row(ui: &mut Ui, o: Pos2, name: &str, folder: bool, id: egui::Id) -> Response {
    let rc = r(o, 0.0, NAME_BASE - 17.0, EDGE + LEFT_W, 28.0);
    let resp = ui.interact(rc, id, Sense::click_and_drag());
    winui::focus_on_press(&resp);
    let (label, described) = name_row_text(name, folder);
    // A group: it is no list's item, and a click does nothing of its own.
    winui::expose(&resp, egui::Role::Group, &label, true, None);
    resp.ctx
        .accesskit_node_builder(resp.id, |n| n.set_description(described));
    let p = ui.painter().clone();
    let mut name_right = o.x + EDGE + LEFT_W;
    if resp.hovered() || resp.has_focus() {
        p.rect_filled(rc, 4.0, crate::theme::colors().button_hot);
        let grip = Rect::from_min_size(pos2(rc.left(), rc.center().y - 6.0), vec2(12.0, 12.0));
        crate::themeicons::paint_graphite_in(ui, &p, "gx-grip", grip, muted());
        // i18n: the Done window's name row, under the pointer or with the keys: what it does
        let hint = crate::i18n::t("Drag out, or Ctrl+C");
        let at = draw_right(
            &p,
            face(12.0),
            rc.right() - 8.0,
            o.y + NAME_BASE,
            hint,
            muted(),
        );
        name_right = at.left() - 16.0;
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    let f = medium(13.0);
    let shown = winui::ellipsize(&p, &f, name, (name_right - o.x - EDGE).max(0.0));
    winui::text_font(&p, f, o.x + EDGE, o.y + NAME_BASE, &shown, ink());
    winui::show_focus(ui, &resp, rc);
    if shown != name {
        resp.on_hover_text(name)
    } else {
        resp
    }
}

/// The Done window's facts under its strip (boards 5, 7, 14): Saved to, a link that shows the
/// download in its folder (`link`: its id); the `lines` under it in the error colour, each read
/// out at once (why the file can't open, a notice); Main file when there is one. The link's
/// response.
pub(crate) fn done_facts(
    ui: &mut Ui,
    o: Pos2,
    saved: &str,
    lines: &[String],
    main: Option<&str>,
    link: egui::Id,
) -> Response {
    let p = ui.painter().clone();
    let right = o.x + EDGE + LEFT_W;
    let x = o.x + VALUE_X;
    let mut y = FACTS_Y;
    key(&p, o, y, false, crate::i18n::mark("Saved to"));
    let f = face(13.0);
    let shown = winui::ellipsize_middle(&p, &f, saved, right - x);
    let rc = winui::text_font(&p, f, x, o.y + y + 12.0, &shown, ink());
    let resp = ui.interact(rc.expand2(vec2(0.0, 2.0)), link, Sense::click());
    winui::focus_on_press(&resp);
    let spoken = crate::i18n::fmt(
        // i18n: the Done window's Saved to link for screen readers: {path} the folder
        crate::i18n::t("Saved to {path}"),
        &[("path", &crate::i18n::ltr(saved))],
    );
    winui::expose(&resp, egui::Role::Link, &spoken, true, None);
    let under = if resp.hovered() {
        ink()
    } else {
        winui::TEXT_DISABLED()
    };
    winui::solid(
        &p,
        Rect::from_min_max(
            pos2(rc.left(), o.y + y + 15.0),
            pos2(rc.right(), o.y + y + 16.0),
        ),
        under,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    winui::show_focus(ui, &resp, rc.expand(2.0));
    // i18n: tooltip of the Done window's Saved to link
    let tip = crate::i18n::t("Show it in its folder");
    let resp = if shown != saved {
        winui::tip(resp, &format!("{saved}\n{tip}")) // not translated: a path and a sentence
    } else {
        winui::tip(resp, tip)
    };
    y += DONE_ROW + GAP;
    for (k, line) in lines.iter().enumerate() {
        let rc = cell(
            ui,
            &p,
            face(13.0),
            o.x + EDGE,
            o.y + y + 15.0,
            right,
            line,
            winui::ERROR(),
            ("done-line", k),
        );
        let id = egui::Id::unique(("done-line", k, ui.ctx().viewport_id()));
        let read = ui.interact(rc, id, Sense::hover());
        winui::expose(&read, egui::Role::Alert, line, true, None);
        y += DONE_LINE + GAP;
    }
    if let Some(main) = main {
        // i18n: label in the Done window: the file the download is mostly made of (Open opens it)
        key(&p, o, y, false, crate::i18n::mark("Main file"));
        cell(
            ui,
            &p,
            face(13.0),
            x,
            o.y + y + 12.0,
            right,
            main,
            secondary(),
            "done-main",
        );
    }
    resp
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

    /// The compact window is 440 wide (board 1: 204 under its title bar), Details 760; a
    /// torrent's details are taller (its rows), and a state's line makes room for itself there.
    #[test]
    fn details_widen_the_window() {
        assert_eq!(size(false, Kind::File, 0), [COMPACT_W, 204.0]);
        assert_eq!(size(true, Kind::File, 0)[0], DETAILS_W);
        assert!(size(true, Kind::Torrent, 0)[1] > size(true, Kind::File, 0)[1]);
        // A message's lines keep the compact window's height; in Details the facts go under
        // them (plan 13d): one line or none where they always start, a line lower for two.
        assert_eq!(size(false, Kind::Torrent, 2), size(false, Kind::File, 0));
        assert_eq!(facts_top(0), FACTS_Y);
        assert_eq!(facts_top(1), FACTS_Y);
        assert_eq!(facts_top(2), FACTS_Y + MESSAGE_LINE);
        assert_eq!(
            size(true, Kind::File, 2)[1],
            size(true, Kind::File, 0)[1] + MESSAGE_LINE
        );
        // The right column's tables take the whole rows down to the left column's end, five
        // at the fewest; it never runs past the left one.
        assert_eq!(rows_down_to(0.0, TITLES_H + 8.0 * ROW_H + 20.0), 8);
        assert_eq!(rows_down_to(0.0, 10.0), ROWS_SHOWN);
        for kind in [Kind::File, Kind::Torrent, Kind::Video] {
            assert!(right_end(kind) <= facts_end(kind, facts_top(0)), "{kind:?}");
        }
    }

    /// Plan 13c: the speed graph's sampler starts empty, keeps one reading a second (fed twice
    /// within a second it keeps one) and the last 60 only.
    #[test]
    fn the_sampler_keeps_60_readings_and_starts_empty() {
        let mut s = Sampler::default();
        assert!(s.readings().is_empty());
        s.feed(10.0, 100.0);
        s.feed(10.4, 200.0);
        s.feed(10.9, 300.0);
        assert_eq!(s.readings().len(), 1, "one a second");
        assert_eq!(s.readings()[0], 100.0);
        s.feed(11.0, 400.0);
        assert_eq!(s.readings().len(), 2);
        for k in 0..100 {
            s.feed(12.0 + k as f64, k as f64);
        }
        assert_eq!(s.readings().len(), Sampler::KEPT);
        assert_eq!(s.readings().back(), Some(&99.0), "the newest last");
        assert_eq!(s.readings().front(), Some(&40.0), "the oldest dropped");
    }

    /// Seconds without a frame (the window in the notification area) read 0, so a pause shows
    /// as one; the clock moves by whole seconds, not to the frame's time.
    #[test]
    fn the_sampler_fills_missed_seconds_with_nothing() {
        let mut s = Sampler::default();
        s.feed(0.0, 5.0);
        s.feed(3.5, 7.0);
        let got: Vec<f64> = s.readings().iter().copied().collect();
        assert_eq!(got, [5.0, 0.0, 0.0, 7.0]);
        s.feed(3.9, 9.0);
        assert_eq!(
            s.readings().len(),
            4,
            "3.5 counted as second 3: 3.9 is in it"
        );
        s.feed(4.0, 9.0);
        assert_eq!(s.readings().len(), 5, "no drift: second 4 starts at 4.0");
        // A long gap: the last minute is all zeros but the reading.
        s.feed(1000.2, 1.0);
        assert_eq!(s.readings().len(), Sampler::KEPT);
        assert_eq!(s.readings().back(), Some(&1.0));
        assert!(s.readings().iter().rev().skip(1).all(|r| *r == 0.0));
    }

    /// Plan 13c: each segment is as wide as its byte range's share of the file, filled as far
    /// as it has come with a tick at its cursor while it is written; one narrower than 3 points
    /// (or more than 48 of them) makes the strip one plain fill.
    #[test]
    fn segment_widths_follow_byte_ranges() {
        // A quarter and three quarters of 1000 bytes across 400 points, given out of order.
        let parts = segment_parts(&[(250, 375.0), (0, 250.0)], 1000, 400.0).unwrap();
        assert_eq!(parts.len(), 2);
        let (a, b) = (parts[0], parts[1]);
        assert_eq!((a.x0, a.x1), (0.0, 100.0 - PART_GAP));
        assert_eq!(a.fill, a.x1, "a finished segment fills its room");
        assert!(!a.tick, "no tick once it is all in");
        assert_eq!((b.x0, b.x1), (100.0, 400.0), "the last one reaches the end");
        assert_eq!(b.fill, 250.0, "half of its 750 bytes");
        assert!(b.tick, "still being written");
        // Nothing yet: its tick at its start.
        let parts = segment_parts(&[(0, 0.0)], 1000, 400.0).unwrap();
        assert_eq!(parts[0].fill, 0.0);
        assert!(parts[0].tick);
        // A segment of 5 bytes in 1000 is 2 points wide: one plain fill.
        assert_eq!(segment_parts(&[(0, 0.0), (995, 0.0)], 1000, 400.0), None);
        // 49 segments: one plain fill too.
        let many: Vec<(u64, f64)> = (0..49).map(|k| (k * 1000, 0.0)).collect();
        assert_eq!(segment_parts(&many, 49_000, 400.0), None);
        assert_eq!(segment_parts(&[], 1000, 400.0), None);
        assert_eq!(segment_parts(&[(0, 0.0)], 0, 400.0), None);
    }

    /// Plan 13c: the Pieces row counts the pieces that are in, of all of them, with one piece's
    /// size; numbers grouped as counts are.
    #[test]
    fn the_pieces_row_counts_pieces() {
        let pieces = [true, false, true, true];
        assert_eq!(pieces_line(&pieces, 512 * 1024), "3 of 4, 512.00 KB each");
        let mut many = vec![false; 1470];
        many[..98].iter_mut().for_each(|b| *b = true);
        let line = pieces_line(&many, 512 * 1024);
        let digits: String = line.chars().filter(char::is_ascii_digit).collect();
        assert!(digits.starts_with("981470"), "{line}");
        assert!(line.ends_with("512.00 KB each"), "{line}");
    }

    /// The reading: whole percents rounded down, one decimal under 10; the detail line says the
    /// unit once when both sizes share it.
    #[test]
    fn the_reading_and_the_detail_line() {
        assert_eq!(reading_figure(0.4199), "41");
        assert_eq!(reading_figure(0.067), "6.7");
        assert_eq!(reading_figure(0.9999), "99");
        assert_eq!(reading_figure(1.0), "100");
        assert_eq!(reading_figure(0.29), "29");
        assert_eq!(reading_figure(0.0), "0");
        let gb = 1024 * 1024 * 1024;
        assert_eq!(so_far(2 * gb + gb / 2, Some(6 * gb)), "2.50 of 6.00 GB");
        assert_eq!(so_far(1024 * 1024, Some(2 * gb)), "1.00 MB of 2.00 GB");
        assert_eq!(so_far(1024 * 1024, None), "1.00 MB");
    }

    /// A download 41% in, 2.38 of 5.80 GB, as the boards show it.
    fn facts() -> Facts {
        let mb = 1024 * 1024;
        Facts {
            name: "ubuntu.iso".into(),
            frac: Some(0.41),
            downloaded: 2437 * mb,
            size: Some(5939 * mb),
            left: "3 min 10 sec".into(),
            rate: Some("18.40 MB/s".into()),
            speed: Some("18.40 MB/s".into()),
            word: "Receiving data...".into(),
            queue: Some("Main queue".into()),
            resumes: true,
            reason: Some("The download link has expired (HTTP 403).".into()),
            ..Facts::default()
        }
    }

    /// Plan 13d: each state's look from the existing state data (boards 1, 4, 8, 9): the
    /// reading, greyed or not, what is beside it, the strip and the row under it.
    #[test]
    fn each_state_gets_its_boards_look() {
        let f = facts();
        let detail = |v: &StateView| match &v.under {
            Under::Detail(a, b) => (a.clone(), b.clone()),
            Under::Message { .. } => panic!("a message: {:?}", v.under),
        };
        let message = |v: &StateView| match &v.under {
            Under::Message { text, error, .. } => (text.clone(), *error),
            Under::Detail(..) => panic!("a detail row: {:?}", v.under),
        };
        // Running (board 1): no word, the percent, the time left, the download's own strip.
        let v = state_view(Phase::Running, &f);
        assert_eq!(v.word, None);
        assert_eq!(v.reading, Reading::Percent(0.41));
        assert_eq!(
            v.side,
            Side::Figure("3 min 10 sec".into(), Some("left".into()))
        );
        assert!(v.strip.is_none() && !v.grey && !v.dim);
        assert_eq!(
            detail(&v),
            (so_far(f.downloaded, f.size), "18.40 MB/s".into())
        );
        // Size not known (board 8): the amount, its speed, an empty strip.
        let unknown = Facts {
            frac: None,
            size: None,
            left: String::new(),
            ..facts()
        };
        let v = state_view(Phase::Running, &unknown);
        assert_eq!(v.word, Some(("Size not known".into(), false)));
        assert_eq!(v.reading, Reading::Amount(unknown.downloaded));
        assert_eq!(v.side, Side::Figure("18.40".into(), Some("MB/s".into())));
        assert_eq!(v.strip, Some(Strip::Empty));
        assert_eq!(detail(&v).1, "No time estimate");
        // Stopped (board 4): the percent greyed, the strip grey, Resume goes on from there.
        let v = state_view(Phase::Stopped, &f);
        assert!(v.dim && v.grey && v.strip.is_none());
        assert_eq!(v.side, Side::Text("Resumes where it stopped".into()));
        assert_eq!(detail(&v).1, "");
        let restarts = Facts {
            resumes: false,
            ..facts()
        };
        assert_eq!(state_view(Phase::Stopped, &restarts).side, Side::None);
        // An error (board 4): the reason in its own line (no "Error: ..."), the missing part
        // hatched, the amount beside the greyed percent.
        let v = state_view(Phase::Failed { new_address: true }, &f);
        assert_eq!(v.word, Some(("Stopped by an error".into(), true)));
        assert_eq!(v.strip, Some(Strip::Broken(0.41)));
        assert_eq!(message(&v), (f.reason.clone().unwrap(), true));
        assert!(!message(&v).0.starts_with("Error:"));
        assert_eq!(v.side, Side::Figure(so_far(f.downloaded, f.size), None));
        assert!(v.dim);
        // A web page (board 9): said beside the name in the error colour, no reading, the strip
        // hatched, the reason read out.
        let v = state_view(Phase::WebPage, &f);
        assert_eq!(v.word, Some(("Got a web page".into(), true)));
        assert_eq!(v.reading, Reading::None);
        assert_eq!(v.side, Side::None);
        assert_eq!(spoken_reading(&v.reading), None);
        assert!(matches!(v.strip, Some(Strip::Broken(_))));
        // Board 9's own sentences, not the engine's reason; the advice under it.
        assert_eq!(
            message(&v),
            (
                "The site sent a web page instead of the file, such as a waiting page or a check."
                    .into(),
                true
            )
        );
        assert!(
            matches!(&v.under, Under::Message { advice: Some(a), .. } if a.starts_with("Open it in your browser"))
        );
        let failed = state_view(Phase::Failed { new_address: false }, &f);
        assert!(matches!(failed.under, Under::Message { advice: None, .. }));
        // Details' facts make room for a message's two lines, none for a detail row.
        assert_eq!(under_lines(&failed), MESSAGE_LINES);
        assert_eq!(under_lines(&v), MESSAGE_LINES);
        assert_eq!(
            sentence("the server answered 404"),
            "The server answered 404"
        );
        assert_eq!(sentence(""), "");
        // One Details height in every state: no jump when a message comes.
        assert_eq!(under_lines(&state_view(Phase::Running, &f)), MESSAGE_LINES);
        // Two lines at most, the rest joined in the last one (cut when drawn).
        let lines = |n: usize| (1..=n).map(|k| format!("l{k}")).collect::<Vec<_>>();
        assert_eq!(at_most(lines(1), 2), ["l1"]);
        assert_eq!(at_most(lines(2), 2), ["l1", "l2"]);
        assert_eq!(at_most(lines(4), 2), ["l1", "l2 l3 l4"]);
        assert_eq!(at_most(Vec::new(), 2), Vec::<String>::new());
        // A waiting download names its own queue; the screenshot's state of one in no queue
        // the main queue (board 8); a real one in no queue none.
        assert_eq!(
            queue_named(Some("Night"), Some("Main download queue"), true).as_deref(),
            Some("Night")
        );
        assert_eq!(
            queue_named(None, Some("Main download queue"), true).as_deref(),
            Some("Main download queue")
        );
        assert_eq!(queue_named(None, Some("Main download queue"), false), None);
        // Waiting (board 8): a dash, the queue named, the strip grey.
        let v = state_view(Phase::Waiting, &f);
        assert_eq!(v.reading, Reading::Dash);
        assert!(v.grey);
        assert!(
            matches!(&v.side, Side::Named(before, q)
                if before == "in" && craft_i18n::strip_isolates(q) == "Main queue"),
            "{:?}",
            v.side
        );
        assert_eq!(v.value, Some(0.41), "what its grey strip shows");
        let fresh = Facts {
            frac: None,
            ..facts()
        };
        assert_eq!(state_view(Phase::Waiting, &fresh).value, Some(0.0));
        // Finishing (board 8): every byte in, 100%, the strip whole.
        let v = state_view(Phase::Finishing, &f);
        assert_eq!(v.reading, Reading::Percent(1.0));
        assert_eq!(v.side, Side::Text("Writing the file".into()));
        assert!(matches!(v.strip, Some(Strip::Plain { fraction: 1.0, .. })));
        assert!(detail(&v).0.starts_with("All ") && detail(&v).1.starts_with("Naming it "));
        // Joining picture and sound (board 4): the percent joined, no time left.
        let join = Facts {
            after: After::Join,
            after_frac: Some(0.45),
            ..facts()
        };
        let v = state_view(Phase::Finishing, &join);
        assert_eq!(v.word, Some(("Joining picture and sound".into(), false)));
        assert_eq!(v.reading, Reading::Percent(0.45));
        assert_eq!(v.side, Side::Text("joined".into()));
        assert!(v.strip.is_none(), "the join's own fill on its accent track");
        // How far not known yet: a dash, not the download's own 100% "joined".
        for after in [After::Join, After::Check] {
            let unknown = Facts {
                after,
                after_frac: None,
                frac: Some(1.0),
                ..facts()
            };
            let v = state_view(Phase::Finishing, &unknown);
            assert_eq!(v.reading, Reading::Dash, "{after:?}");
            assert_eq!(v.side, Side::None, "{after:?}");
            assert_eq!(v.value, None, "{after:?}");
        }
        // The checksum's check says so, its percent only in the reading.
        let check = Facts {
            after: After::Check,
            after_frac: Some(0.3),
            ..facts()
        };
        assert_eq!(
            state_view(Phase::Finishing, &check).word,
            Some(("Checking the file".into(), false))
        );
        assert_eq!(
            state_view(Phase::Finishing, &f).word,
            Some(("Finishing".into(), false))
        );
        // A torrent checking its files (board 8): the bytes checked, a plain grey fill.
        let checking = Facts {
            checked: Some((37, 100)),
            ..facts()
        };
        let v = state_view(Phase::Checking, &checking);
        assert_eq!(v.reading, Reading::Percent(0.37));
        assert_eq!(
            v.strip,
            Some(Strip::Plain {
                fraction: 0.37,
                after: false
            })
        );
        assert!(v.grey);
        // At a debrid service (board 9): named, its percent, a striped fill.
        let service = Facts {
            service: Some(("Real-Debrid".into(), Some(0.62))),
            ..facts()
        };
        let v = state_view(Phase::Service, &service);
        assert_eq!(v.reading, Reading::Percent(0.62));
        assert_eq!(v.strip, Some(Strip::Striped(0.62)));
        assert!(matches!(&v.side, Side::Text(s) if s.contains("Real-Debrid")));
        let unsaid = Facts {
            service: Some(("Real-Debrid".into(), None)),
            ..facts()
        };
        assert_eq!(state_view(Phase::Service, &unsaid).reading, Reading::Dash);
        // Sharing (board 9): the word in the reading, the ratio beside it, the strip whole.
        let sharing = Facts {
            sharing: Some(Sharing {
                sent: "308.00 MB".into(),
                ratio: "0.42".into(),
                peers: Some(6),
                secs: 72 * 60,
            }),
            ..facts()
        };
        let v = state_view(Phase::Sharing, &sharing);
        assert_eq!(v.word, None);
        assert_eq!(v.reading, Reading::Word("Sharing".into()));
        assert_eq!(v.side, Side::Figure("0.42".into(), Some("ratio".into())));
        assert_eq!(v.value, Some(1.0), "shared whole");
        let (sent, shared) = detail(&v);
        assert_eq!(
            craft_i18n::strip_isolates(&sent),
            "Sent 308.00 MB, 6 peers connected"
        );
        assert_eq!(shared, "shared for 1 hour(s) 12 min");
        // The session not asked yet, nor shared a second: what it has only.
        let early = Facts {
            sharing: Some(Sharing {
                sent: "0 B".into(),
                ratio: "0.00".into(),
                peers: None,
                secs: 0,
            }),
            ..facts()
        };
        assert_eq!(
            detail(&state_view(Phase::Sharing, &early)),
            ("Sent 0 B".into(), String::new())
        );
        // Can't resume (board 9): the percent and time left, the question read out.
        let v = state_view(Phase::CantResume, &f);
        assert_eq!(v.reading, Reading::Percent(0.41));
        assert!(!message(&v).1);
        assert!(message(&v).0.contains("starts over"));
        // The strip says the state too, its parts hidden.
        let said = spoken(&state_view(Phase::Stopped, &f));
        assert!(said.contains("41%"), "{said}");
        assert!(said.contains("Resumes where it stopped"), "{said}");
        assert_eq!(spoken_reading(&Reading::Dash), None);
    }

    /// Plan 13d (boards 5, 7): "Done" with the size, the strip whole, the number of files
    /// (nothing for one); its places from the facts down.
    #[test]
    fn the_done_window_reads_done_with_its_size_and_files() {
        let v = done_view(734 * 1024 * 1024, 2);
        assert_eq!(v.reading, Reading::Word("Done".into()));
        assert_eq!(v.side, Side::Figure("734.00 MB".into(), None));
        assert!(matches!(v.strip, Some(Strip::Plain { fraction: 1.0, .. })));
        assert_eq!(v.value, Some(1.0), "its strip says 100 (board 5)");
        // The name row for screen readers: a file, or a folder download's folder.
        let (file, what) = name_row_text("a.iso", false);
        assert_eq!(craft_i18n::strip_isolates(&file), "a.iso, file");
        assert!(what.contains("Ctrl+C"));
        let (folder, _) = name_row_text("Album", true);
        assert_eq!(craft_i18n::strip_isolates(&folder), "Album, folder");
        let count = |v: &StateView| match &v.under {
            Under::Detail(a, _) => craft_i18n::strip_isolates(a),
            Under::Message { .. } => panic!(),
        };
        assert_eq!(count(&v), "2 files");
        assert_eq!(count(&done_view(1, 248)), "248 files");
        assert_eq!(count(&done_view(1, 1)), "");
        assert_eq!(
            craft_i18n::strip_isolates(&spoken(&v)),
            "Done, 734.00 MB, 2 files"
        );
        // Board 5: Saved to and Main file, the check box, the footer, 268 high with its 20 under
        // it; a line under Saved to (board 14) makes room for itself.
        let (check, footer, height) = done_layout(true, 0);
        assert_eq!(
            (check, footer),
            (FACTS_Y + 16.0 + GAP + 16.0 + 16.0, check + 32.0)
        );
        assert_eq!(height, footer + BUTTON_H + BOTTOM);
        assert_eq!(done_layout(true, 1).2, height + DONE_LINE + GAP);
        assert_eq!(done_layout(false, 0).2, height - DONE_ROW - GAP);
        // The footer from the right edge, a split button's parts touching.
        let xs = row_from_right(&[(56.0, false), (30.0, true), (90.0, false), (60.0, false)]);
        assert_eq!(xs[3], DONE_W - EDGE - 60.0);
        assert_eq!(xs[2], xs[3] - GAP - 90.0);
        assert_eq!(xs[1], xs[2] - GAP - 30.0);
        assert_eq!(xs[0], xs[1] - 56.0, "joined: no gap");
    }
}
