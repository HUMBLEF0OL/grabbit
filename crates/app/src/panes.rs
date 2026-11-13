//! The two optional parts of the main window (Graphite phase 06): the paste bar under the
//! toolbar, which adds a download from an address as Add URL's OK does, and the details pane
//! beside the list. `app.rs` and `chrome.rs` are large, so they get a module of their own.
//!
//! - [`PasteBar`]: the paste bar's state, kept by `App` across frames.
//! - [`PasteEvent`]: what the paste bar reports to `App` in a frame.
//! - [`paste_field_id`]: the address field's egui id (Tab rotation, the typing gate, tests).
//! - [`paste_bar`]: the band's painter; [`paste_geometry`]: where its parts go in a width.
//! - [`submit`]: what a submit does to the field, and the address it hands on.
//! - [`ctrl_v`]: whether this frame's Ctrl+V is the paste bar's, and its text.
//! - [`Stop`], [`next_stop`], [`current_stop`], [`tab_keys`]: the main window's Tab rotation
//!   between the paste field, the Categories tree and the download list.
//! - [`segment_cells`]: the details pane's 8 segment cells, from the byte ranges that have come.
//! - [`details_width`]: the details pane's drawn width, its saved width limited (D11).
//! - [`DetailsView`], [`OneDownload`], [`Primary`], [`DetailsAction`]: what the details pane
//!   shows and asks for; [`shown`], [`one_download`] and their helpers build it without `App`.

use egui::{Pos2, Rect, Ui, pos2, vec2};

use crate::chrome::Layout;
use crate::list::StatusKind;
use crate::model::{self, Item, Status};
use crate::winui::{self, ButtonKind};

/// The paste bar's state, kept by `App` across frames.
#[derive(Debug, Default)]
pub struct PasteBar {
    /// The address field's text: always one line (`fill`, and the field's own paste).
    pub text: String,
    /// "Invalid address" after a rejected submit, until the field is edited (D2).
    pub error: Option<String>,
    /// The history drop list is open.
    pub list_open: bool,
    /// Ask the field to take the focus on the next frame (Tab, Ctrl+V).
    pub focus_next: bool,
    /// A menu or a popup menu of the main window is open (set each frame, as the list's): its
    /// keys (Enter, Escape, the arrows) are the menu's, not the field's.
    pub menu_open: bool,
    /// The list's filter box is open (set each frame): Tab moves among its controls in egui's
    /// order, as before the paste bar, so a focus that order hands the field or the Download
    /// button goes on to the box.
    pub filter_open: bool,
}

/// What the paste bar reports in a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteEvent {
    /// Enter in the field (its list closed) or the Download button.
    Submit,
    /// Escape in the field (its list closed): it gave up the keys.
    Left,
}

/// The paste bar's sizes, in points at 100 % (the mockup's `.paste`, `.fld`, `.btn`).
/// Padding at both ends of the band.
const SIDE: f32 = 10.0;
/// Between the field, the hint and the button.
const GAP: f32 = 8.0;
/// The field's and the button's top inside the band, and their height.
const FIELD_TOP: f32 = 8.0;
const FIELD_H: f32 = 32.0;
/// The narrowest field beside the hint: narrower, the hint is left out.
const FIELD_WITH_HINT: f32 = 160.0;
/// The narrowest field ever: past it the button's label gives way.
const FIELD_MIN: f32 = 120.0;
/// The narrowest button: its icon alone (the label all "...").
const BUTTON_MIN: f32 = FIELD_H;

/// Where the paste bar's parts go in a band `w` wide, from its widths whole: the hint's (or the
/// error's) `hint_w` and the Download button's `button_w`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PasteGeometry {
    /// The field's width.
    pub field: f32,
    /// The hint (or the error) is drawn, between the field and the button.
    pub hint: bool,
    /// The button's width: under `button_w`, its label ends in "..." (whole on hover).
    pub button: f32,
}

/// The narrow-window rules (C8): the hint goes first (not drawn when the field would be under
/// 160 beside it), then the button's label cuts; the field never goes under 120.
pub fn paste_geometry(w: f32, hint_w: f32, button_w: f32) -> PasteGeometry {
    let inner = (w - 2.0 * SIDE).max(0.0);
    let beside_hint = (inner - button_w - GAP - hint_w - GAP).floor();
    if beside_hint >= FIELD_WITH_HINT {
        return PasteGeometry {
            field: beside_hint,
            hint: true,
            button: button_w,
        };
    }
    let alone = (inner - button_w - GAP).floor();
    if alone >= FIELD_MIN {
        return PasteGeometry {
            field: alone,
            hint: false,
            button: button_w,
        };
    }
    PasteGeometry {
        field: FIELD_MIN,
        hint: false,
        button: (inner - FIELD_MIN - GAP)
            .floor()
            .clamp(BUTTON_MIN, button_w.max(BUTTON_MIN)),
    }
}

/// Paint the band from `layout.paste_top()`, `w` wide from `o.x`: on `panel` with its bottom
/// rule; the address field (the `gx-link` icon, the placeholder, the history arrow), the hint
/// "Enter adds it" or the error in the error colour, the accent Download button (`gx-download`).
/// The field takes the focus asked for (`focus_next`); an edit clears the error.
///
/// The field's keys: Up / Down step through `history` (its list closed); Alt+Down or F4, or a
/// click on the arrow, open or close the history drop list, and a pick puts the address in the
/// field (nothing submitted). Escape closes an open list first, then leaves the field (`Left`).
/// `Submit` on Enter in the field (its list closed) or a click on Download. While a menu is
/// open (`menu_open`) the field answers none of these keys.
pub fn paste_bar(
    ui: &mut Ui,
    layout: &Layout,
    o: Pos2,
    w: f32,
    st: &mut PasteBar,
    history: &[String],
) -> Option<PasteEvent> {
    let p = ui.painter().clone();
    let c = crate::theme::colors();
    let top = o.y + layout.paste_top();
    let band = Rect::from_min_size(pos2(o.x, top), vec2(w, layout.paste_h));
    p.rect_filled(winui::rect(band), 0.0, c.panel);
    p.rect_filled(
        Rect::from_min_max(
            pos2(band.left(), band.bottom() - 1.0),
            pos2(band.right(), band.bottom()),
        ),
        0.0,
        c.menubar_line,
    );
    // i18n: the paste bar's hint beside its address field: Enter adds the typed address
    let hint = crate::i18n::t("Enter adds it");
    // As the frame starts: an edit below clears the error from the next frame.
    let (slot, ink) = match st.error.clone() {
        Some(e) => (e, c.error),
        None => (hint.to_owned(), c.muted),
    };
    let font = winui::segoe();
    let slot_w = winui::text_width_font(&p, font.clone(), &slot).ceil();
    // i18n: the paste bar's button: add the typed address as a download
    let label = "Download";
    let g = paste_geometry(w, slot_w, winui::icon_label_button_width(&p, label));
    let y = top + FIELD_TOP;

    let id = paste_field_id();
    let asked = std::mem::take(&mut st.focus_next);
    if asked {
        ui.memory_mut(|m| m.request_focus(id));
    }
    let field_rc = winui::rect(Rect::from_min_size(
        pos2(o.x + SIDE, y),
        vec2(g.field, FIELD_H),
    ));
    let field = winui::icon_combo_edit(
        ui,
        field_rc,
        &mut st.text,
        id,
        "gx-link",
        // i18n: the paste bar's address field while it is empty
        crate::i18n::t("Paste an address or a magnet link"),
    );
    // Unasked, the field's focus came from Tab forward (the box's last control, or nothing
    // focused): it goes on to the button and on.
    let passed = st.filter_open && !asked && pass_tab_focus(ui, &field, true);
    // An error about the address goes once it is edited.
    if field.changed() {
        st.edited();
    }
    // Named for screen readers; the error, while shown, is its description.
    // i18n: noun: the web address (URL); the paste bar's field, to screen readers
    let name = crate::i18n::t("Address");
    let error = st.error.clone();
    ui.ctx().accesskit_node_builder(id, |node| {
        node.set_label(name);
        if let Some(e) = error {
            node.set_description(e);
        }
    });
    let event = field_keys(ui, &field, field_rc, st, history);

    if g.hint {
        winui::text_font(&p, font, o.x + SIDE + g.field + GAP, y + 20.0, &slot, ink);
    }
    // At the band's end, never over the field (a window narrower than the narrowest parts).
    let left = (o.x + w - SIDE - g.button).max(o.x + SIDE + g.field + GAP);
    let button_rc = winui::rect(Rect::from_min_size(pos2(left, y), vec2(g.button, FIELD_H)));
    let download =
        winui::icon_label_button(ui, button_rc, "gx-download", label, ButtonKind::Accent);
    // The button's, from the field (forward) or from Shift+Tab before the box's first control.
    if st.filter_open {
        pass_tab_focus(ui, &download, passed);
    }
    event.or(download.clicked().then_some(PasteEvent::Submit))
}

/// While the list's filter box is open, a focus egui's own Tab order gave `resp` this frame (no
/// pointer press or release, no screen reader's request) is given back, and `forward` the next
/// control takes it: Tab past the box's last control comes back to its first, as before the
/// paste bar. Shift+Tab before the box's first control leaves nothing focused (the "contains"
/// box then focuses its text box itself; a Shift+Tab more reaches the box's last control).
/// Whether the focus was given back.
fn pass_tab_focus(ui: &Ui, resp: &egui::Response, forward: bool) -> bool {
    if !resp.gained_focus() {
        return false;
    }
    let other = ui.input(|i| {
        i.pointer.any_down()
            || i.pointer.any_released()
            || i.events
                .iter()
                .any(|e| matches!(e, egui::Event::AccessKitActionRequest(_)))
    });
    if other {
        return false;
    }
    resp.surrender_focus();
    if forward {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::Next));
    }
    true
}

/// One of the main window's Tab stops (Tab_Stop).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The paste bar's address field.
    Paste,
    /// The Categories tree.
    Tree,
    /// The download list.
    List,
}

/// The next Tab stop from `from` (`back`: Shift+Tab) among the shown parts (`paste`: the paste
/// bar, `tree`: the Categories pane; the list is always shown), in the order Paste → Tree →
/// List → Paste; a hidden stop is skipped. None when there is nowhere to go (only the list).
pub fn next_stop(from: Stop, back: bool, paste: bool, tree: bool) -> Option<Stop> {
    const ORDER: [Stop; 3] = [Stop::Paste, Stop::Tree, Stop::List];
    let shown = |s: Stop| match s {
        Stop::Paste => paste,
        Stop::Tree => tree,
        Stop::List => true,
    };
    let at = ORDER.iter().position(|s| *s == from)?;
    let n = ORDER.len();
    // Every stop passed on the way is hidden.
    (1..n)
        .map(|step| {
            if back {
                (at + n - step) % n
            } else {
                (at + step) % n
            }
        })
        .map(|k| ORDER[k])
        .find(|s| shown(*s))
}

/// What the main window's Tab rotation reads from `App` in a frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct TabState {
    /// The paste bar is shown.
    pub paste: bool,
    /// The Categories pane is shown.
    pub tree: bool,
    /// The tree has the keys (`App::tree_keys`).
    pub tree_keys: bool,
    /// A menu or a popup menu of the main window is open.
    pub menu_open: bool,
    /// The list's filter box is open.
    pub filter_open: bool,
}

/// The stop that has the keys now: the paste field when it has egui's focus (the bar shown),
/// else the tree when it has the keys (the pane shown), else the list.
pub fn current_stop(ctx: &egui::Context, s: &TabState) -> Stop {
    if s.paste && ctx.memory(|m| m.has_focus(paste_field_id())) {
        Stop::Paste
    } else if s.tree && s.tree_keys {
        Stop::Tree
    } else {
        Stop::List
    }
}

/// Tab and Shift+Tab in the main window, before any of its widgets is laid out: the stop Tab
/// moved the keys to from `from` ([`current_stop`]). Moving to the paste field asks it to take the focus (`focus_next`);
/// moving to the tree or the list takes the focus from it. The caller gives the tree or the
/// list the keys.
///
/// - Another widget has egui's focus (a filter box's control, the Download button from a
///   screen reader): Tab is egui's, as before the paste bar.
/// - A menu, a popup menu or the history list is open, or the filter box is (the bar shown):
///   no move. With nothing focused and the bar shown, egui's own move is cancelled under a
///   menu, so the field does not take the focus behind it; under the filter box egui moves
///   among the box's controls (`PasteBar::filter_open` passes the field's turn on).
/// - Only the list is shown: nothing (Tab is left as it was before the paste bar).
/// - Else Tab and Shift+Tab are consumed and, the bar shown, egui's own move is cancelled: with
///   nothing focused it would give the first focusable widget (the field) the focus. The bar
///   hidden, this is the tree ↔ list toggle of before, egui's move untouched.
pub fn tab_keys(ctx: &egui::Context, st: &mut PasteBar, s: TabState, from: Stop) -> Option<Stop> {
    let field = paste_field_id();
    let focused = ctx.memory(|m| m.focused());
    if focused.is_some_and(|id| id != field) {
        return None;
    }
    let cancel = || ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    if s.menu_open || st.list_open {
        if s.paste && focused.is_none() {
            cancel();
        }
        return None;
    }
    if s.paste && s.filter_open {
        return None;
    }
    // Every stop is reached both ways round: none forward is none at all, and Tab stays in the
    // input as before.
    next_stop(from, false, s.paste, s.tree)?;
    let (tab, back) = ctx.input_mut(|i| {
        // Shift+Tab first: Tab's pattern (no modifiers) matches Shift+Tab too.
        let back = i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
        let fwd = i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        (fwd || back, back)
    });
    if !tab {
        return None;
    }
    if s.paste {
        cancel();
    }
    let to = next_stop(from, back, s.paste, s.tree)?;
    match to {
        Stop::Paste => st.focus_next = true,
        Stop::Tree | Stop::List => st.leave(ctx),
    }
    Some(to)
}

/// The address field's arrow, keys and history drop list (AddUrl's address box, C8): what they
/// report this frame.
fn field_keys(
    ui: &mut Ui,
    field: &egui::Response,
    field_rc: Rect,
    st: &mut PasteBar,
    history: &[String],
) -> Option<PasteEvent> {
    // The arrow: the addresses added before. A click on it gives the field the keys, so the
    // list's keys and Escape work from there.
    let arrow = winui::combo_arrow(ui, field_rc, "paste-bar", st.list_open);
    if arrow.clicked() {
        field.request_focus();
    }
    // An open menu (drawn after the bar: a context menu) takes the keys the field would.
    let keys = !st.menu_open;
    if arrow.clicked() || (keys && winui::combo_keys(ui, field)) {
        st.list_open = !st.list_open;
    }
    // Up / Down step through the addresses added before.
    let at = history.iter().position(|h| *h == st.text.trim());
    if keys
        && !st.list_open
        && let Some(i) = winui::history_keys(ui, field, history.len(), at)
    {
        st.text = history[i].clone();
        st.edited();
    }
    // The field keeps Tab (the main window's rotation moves the keys), the arrows and Escape
    // (it answers them below, or its list does), so egui's own focus moves never take them.
    winui::keep_keys_and_tab(ui, field, true, true, true);
    let list_was_open = st.list_open;
    // The open list takes the keys first: Escape closes it, Enter picks; the field keeps the
    // keys (8.4).
    if st.list_open
        && let Some(i) = winui::drop_list(
            ui,
            "paste-bar-history",
            field_rc,
            history,
            // i18n: the address history drop list while no address was added yet
            crate::i18n::t("No addresses yet"),
            &mut st.list_open,
        )
        && let Some(picked) = history.get(i)
    {
        st.text = picked.clone();
        st.edited();
        field.request_focus();
    }
    if list_was_open || !keys || !field.has_focus() {
        return None;
    }
    if winui::take_key(ui, egui::Key::Escape) {
        st.leave(ui.ctx());
        return Some(PasteEvent::Left);
    }
    winui::take_key(ui, egui::Key::Enter).then_some(PasteEvent::Submit)
}

/// Ctrl+V into the paste bar (D13): the text of this frame's paste when it is the bar's, for
/// [`PasteBar::fill`]. It is the bar's while the bar is `shown`, no widget has the keys (a text
/// box with them, the Find box or the field itself, pastes into itself) and no menu or popup is
/// `busy`. egui-winit turns Ctrl+V into `Event::Paste` whatever has the keys, and sends none
/// for a clipboard without text; an empty paste leaves the field as it is too.
pub fn ctrl_v(shown: bool, focused: bool, busy: bool, events: &[egui::Event]) -> Option<&str> {
    if !shown || focused || busy {
        return None;
    }
    events.iter().find_map(|e| match e {
        egui::Event::Paste(t) if !t.is_empty() => Some(t.as_str()),
        _ => None,
    })
}

/// The address field's egui id (Tab rotation, the typing gate, tests).
pub fn paste_field_id() -> egui::Id {
    egui::Id::unique("graphite-paste-field")
}

/// A submit's outcome (Enter or Download) from what `dialogs::accept_address` made of the
/// field's text. Accepted: the field empties and the address goes on as Add URL's OK sends it,
/// without a login (Sites Logins still applies, D3). Rejected: the text stays, Add URL's
/// "Invalid address" takes the hint's place (D2) and the field keeps the keys.
pub fn submit(st: &mut PasteBar, accepted: Option<url::Url>) -> Option<crate::dialogs::NewUrl> {
    match accepted {
        Some(url) => {
            st.text.clear();
            st.error = None;
            Some(crate::dialogs::NewUrl { url, login: None })
        }
        None => {
            st.error = Some(crate::i18n::t("Invalid address").to_owned());
            st.focus_next = true;
            None
        }
    }
}

impl PasteBar {
    /// The field's text was edited (typed, stepped to or picked from the history): an
    /// "Invalid address" about the text before goes (D2).
    pub fn edited(&mut self) {
        self.error = None;
    }

    /// Put text in the field (Ctrl+V): its first line, without control characters and trimmed;
    /// any "Invalid address" goes and the field takes the focus on the next frame. Nothing is
    /// submitted: Enter or Download does that (D13).
    pub fn fill(&mut self, clip: &str) {
        let first = clip.lines().next().unwrap_or_default();
        self.text = crate::winui::single_line(first).trim().to_owned();
        self.error = None;
        self.focus_next = true;
    }

    /// Give up the keyboard focus (Escape, the bar hidden): the field loses egui's focus, its
    /// drop list closes and a focus asked for the next frame is dropped.
    pub fn leave(&mut self, ctx: &egui::Context) {
        let id = paste_field_id();
        ctx.memory_mut(|m| m.surrender_focus(id));
        self.list_open = false;
        self.focus_next = false;
    }
}

/// The details pane's 8 segment cells (C11): cell k is the share, `0..=1`, of the k-th eighth of
/// `size` that has come. `done` holds `(start, end)` byte ranges from the data the progress
/// window's segment bar reads: the engine snapshot's segments `start..cursor`, else
/// `0..downloaded`. Overlapping or unsorted ranges are merged, so no byte counts twice; bytes
/// past `size` and empty ranges (`start >= end`) are left out. All 0 without a size or with a
/// size of 0 (Requirement 13.2).
///
/// A size under 8 bytes has empty parts (no bytes). An empty part is full when the byte just
/// before its edge has come (byte 0 for a part at 0), else 0: it adds no bytes to
/// Σ cellₖ × partₖ, and one range `0..d` still fills the cells in order.
pub fn segment_cells(size: Option<u64>, done: &[(u64, u64)]) -> [f32; 8] {
    let mut cells = [0.0f32; 8];
    let Some(total) = size.filter(|t| *t > 0) else {
        return cells;
    };
    let merged = merged_ranges(done, total);
    for (k, cell) in (0u64..).zip(cells.iter_mut()) {
        let (a, b) = (part_edge(total, k), part_edge(total, k + 1));
        if b <= a {
            let byte = a.saturating_sub(1);
            if merged.iter().any(|&(s, e)| s <= byte && byte < e) {
                *cell = 1.0;
            }
            continue;
        }
        // The merged ranges are disjoint, so `covered <= b - a`.
        let covered: u64 = merged
            .iter()
            .map(|&(s, e)| e.min(b).saturating_sub(s.max(a)))
            .sum();
        *cell = (covered as f64 / (b - a) as f64).clamp(0.0, 1.0) as f32;
    }
    cells
}

/// Where part `k` of 8 starts in `total` bytes (part 8 starts at `total`). In `u128`, as
/// `total * k` overflows `u64` for a size near its top; the result is at most `total`.
fn part_edge(total: u64, k: u64) -> u64 {
    (u128::from(total) * u128::from(k) / 8) as u64
}

/// `done`'s ranges cut to `0..total`, the empty ones dropped, sorted and with overlapping or
/// touching ones joined: disjoint ranges in order.
fn merged_ranges(done: &[(u64, u64)], total: u64) -> Vec<(u64, u64)> {
    let mut ranges: Vec<(u64, u64)> = done
        .iter()
        .map(|&(s, e)| (s, e.min(total)))
        .filter(|(s, e)| s < e)
        .collect();
    ranges.sort_unstable();
    let mut out: Vec<(u64, u64)> = Vec::with_capacity(ranges.len());
    for (s, e) in ranges {
        match out.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => out.push((s, e)),
        }
    }
    out
}

/// The details pane's drawn width (D11): the `saved` width limited to
/// `[200, max(200, min(w / 2, w - list_left - 200))]` points, so the list keeps 200 points from
/// its leading edge `list_left` when a `w`-wide client area allows it. Whole points: the upper
/// bound is taken down to a whole point, so the result stays inside the range and a second call
/// on it gives it back. The saved value is not changed here (Requirement 17.4).
pub fn details_width(saved: u32, w: f32, list_left: f32) -> f32 {
    /// The pane's narrowest.
    const MIN: f32 = 200.0;
    /// What the list keeps beside the pane when the window allows it.
    const LIST_MIN: f32 = 200.0;
    let max = (w * 0.5).min(w - list_left - LIST_MIN).max(MIN).floor();
    (saved as f32).clamp(MIN, max).round()
}

/// What the details pane shows this frame (C10), built from the downloads the list draws in the
/// same frame (`App::details_view`).
#[derive(Debug, PartialEq)]
pub enum DetailsView {
    /// Nothing selected, or the one selected download is gone: a muted "Select a download to
    /// see its details".
    Empty,
    /// Several selected: their count and the sum of their known sizes (D9).
    Many { count: usize, total: u64 },
    /// One download.
    One(Box<OneDownload>),
}

/// The one download the details pane shows: every text as drawn (the list's and the progress
/// window's words and formats).
#[derive(Debug, PartialEq)]
pub struct OneDownload {
    pub id: u64,
    /// As the list's Name column has it.
    pub file_name: String,
    /// The file type's 32-point icon key (`IconKey::ExtLarge`), as File Properties finds it.
    pub icon_ext: String,
    /// "Stopped, 63.00%" ([`status_line`], D8).
    pub status_line: String,
    /// "8 connections" ([`connections_label`], D7), or None.
    pub connections: Option<String>,
    /// "57.96 MB of 92.00 MB" ([`bytes_label`]), or None without a size.
    pub bytes: Option<String>,
    /// The 8 segment cells, each `0..=1` ([`segment_cells`]).
    pub cells: [f32; 8],
    /// The cells' colour, as the list's Status cell has it.
    pub kind: crate::list::StatusKind,
    /// The label/value rows in [`ROWS`]' order, a row without a value left out
    /// ([`detail_rows`]).
    pub rows: Vec<(&'static str, String)>,
    /// Resume (it can start) or Stop (running): the default, filled button.
    pub primary: Option<Primary>,
    /// "Show progress window" is greyed for a finished download (D10).
    pub can_show_progress: bool,
}

/// The details pane's default, filled button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primary {
    Resume,
    Stop,
}

/// What a details pane button asks for. `Cmd` is `app.rs`'s, so `App` maps it, each for the
/// shown download: Resume → `Cmd::Resume`, Stop → `Cmd::Stop`, Open folder → `Cmd::OpenFolder`,
/// Show progress window → `Cmd::Open` (D10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailsAction {
    Resume,
    Stop,
    OpenFolder,
    ShowProgress,
}

/// What a selection shows, before its download's details are looked at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// Nothing selected, or the one selected download is gone.
    Empty,
    /// The one selected download.
    One(u64),
    /// Several selected: their count and the sum of their known sizes (D9).
    Many { count: usize, total: u64 },
}

/// What the selected `ids` show (D9), from the `(id, size)` of the downloads there are: one id
/// is that download while it is there (Requirement 14.3), several are their count and the sum
/// of the sizes known (a download without a size, or gone, adds nothing).
pub fn shown(ids: &[u64], sizes: &[(u64, Option<u64>)]) -> Shown {
    match ids {
        [] => Shown::Empty,
        [id] if sizes.iter().any(|(i, _)| i == id) => Shown::One(*id),
        [_] => Shown::Empty,
        _ => {
            let known: std::collections::HashMap<u64, u64> = sizes
                .iter()
                .filter_map(|&(i, s)| s.map(|s| (i, s)))
                .collect();
            Shown::Many {
                count: ids.len(),
                total: ids
                    .iter()
                    .filter_map(|id| known.get(id))
                    .fold(0, |sum, s| sum.saturating_add(*s)),
            }
        }
    }
}

/// The details pane's rows, in their order (the mockup's, with the code's wording: "Save to",
/// D6).
pub const ROWS: [&str; 8] = [
    // i18n: noun: label before the download's state
    crate::i18n::mark("Status"),
    crate::i18n::mark("Transfer rate"),
    crate::i18n::mark("Time left"),
    // i18n: label: whether the server lets a stopped download continue (Yes / No)
    crate::i18n::mark("Resume capability"),
    // i18n: the details pane: label before the folder the download is saved in
    crate::i18n::mark("Save to"),
    // i18n: noun: the web address (URL) the download comes from
    crate::i18n::mark("Address"),
    // i18n: the details pane: label before the download queue the download is in
    crate::i18n::mark("Queue"),
    crate::i18n::mark("Last Try Date"),
];

/// The rows with a value, in [`ROWS`]' order: `values` holds each row's value at its place; a
/// missing or blank one leaves its row out.
pub fn detail_rows(values: [Option<String>; 8]) -> Vec<(&'static str, String)> {
    ROWS.into_iter()
        .zip(values)
        .filter_map(|(label, v)| v.filter(|v| !v.trim().is_empty()).map(|v| (label, v)))
        .collect()
}

/// A row whose value is cut in the middle when it does not fit (its start and its end stay in
/// sight): the Address (Requirement 12.9) and, as the progress window cuts it, the Save to
/// folder. Every other value is cut at its end.
pub fn cuts_in_middle(label: &str) -> bool {
    label == ROWS[4] || label == ROWS[5]
}

/// A download's state in a word (D8): the Status filter's words (Waiting, Downloading,
/// Stopped, Complete, Error).
pub fn status_word(status: Status) -> &'static str {
    crate::i18n::t(StatusKind::of(status).group())
}

/// The status line under the file's name (D8): its state in a word, then ", {percent}" while
/// the share that has come is known (`fraction`, as the list's Status column reads it) and the
/// download is not complete: "Stopped, 63.00%".
pub fn status_line(status: Status, fraction: Option<f64>) -> String {
    let word = status_word(status);
    match fraction {
        Some(f) if status != Status::Complete => crate::i18n::fmt(
            // i18n: the details pane, under the file's name: {status} is the download's state ("Stopped", "Downloading"), {percent} how much has come ("63.00%")
            crate::i18n::t("{status}, {percent}"),
            &[("status", word), ("percent", &model::percent(f))],
        ),
        _ => word.to_owned(),
    }
}

/// The segment cells' label (D7): while it runs (`running`), the connections working now (the
/// progress window's table: connecting, receiving or waiting to retry); else the segments of
/// the engine's last view of it in this session (`snap`). None without a view (after a
/// restart) or with no connection or segment to count (a video, a torrent).
pub fn connections_label(running: bool, snap: Option<&mdm_engine::Snapshot>) -> Option<String> {
    use mdm_engine::download::ConnStatus;
    let snap = snap?;
    let n = if running {
        snap.connections
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    ConnStatus::Connecting | ConnStatus::Receiving | ConnStatus::Retrying(_)
                )
            })
            .count()
    } else {
        snap.segments.len()
    };
    (n > 0).then(|| {
        // i18n: the details pane, over the segment cells: how many connections the download has
        crate::i18n::tn(n as u64, "{n} connection", "{n} connections")
    })
}

/// The segment cells' bytes label (Requirement 13.7): "57.96 MB of 92.00 MB" in the list's size
/// format, whenever the size is known.
pub fn bytes_label(downloaded: u64, size: Option<u64>) -> Option<String> {
    let total = size?;
    Some(crate::i18n::fmt(
        // i18n: the details pane, over the segment cells: {done} is how much has come of the whole {total}, e.g. "57.96 MB of 92.00 MB"
        crate::i18n::t("{done} of {total}"),
        &[
            ("done", &model::size(downloaded)),
            ("total", &model::size(total)),
        ],
    ))
}

/// The Address row's value: the address without its password (a user name stays, as the
/// address shows elsewhere; "Use authorization"'s login, `Item::login`, is never shown), as
/// left-to-right data.
pub fn address_text(url: &url::Url) -> String {
    let mut shown = url.clone();
    // An address without a host (a magnet link) has no password to take out.
    let _ = shown.set_password(None);
    crate::i18n::ltr(shown.as_str())
}

/// The byte ranges that have come, as the progress window's segment bar reads them: the engine
/// snapshot's segments `start..cursor`, else `0..downloaded`.
pub fn done_ranges(item: &Item) -> Vec<(u64, u64)> {
    match &item.snap {
        Some(s) if !s.segments.is_empty() => {
            s.segments.iter().map(|g| (g.start, g.cursor)).collect()
        }
        _ => vec![(0, item.downloaded)],
    }
}

/// The details of one download (`App::one_download`): `live_rate` is its live transfer's rate
/// while it runs (`Item::rate`; None: not running), `queue` the name of the queue it is in.
/// The rows (Requirement 12.3): Status always; Transfer rate while it runs (the list's
/// column); Time left while it runs and is known; Resume capability while known ("Yes" /
/// "No"); Save to (the folder, as the progress window writes it); the Address; Queue in a
/// queue (its name as the Categories tree shows it); Last Try Date when it was tried (the
/// list's column).
pub fn one_download(item: &Item, live_rate: Option<f64>, queue: Option<&str>) -> OneDownload {
    let running = live_rate.is_some();
    let save_to = item
        .dest
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| {
            let shown = crate::dialogs::display_path(dir);
            // The user's own folder is "" under it: written whole.
            crate::i18n::ltr(&if shown.is_empty() {
                dir.display().to_string()
            } else {
                shown
            })
        });
    let rows = detail_rows([
        Some(status_word(item.status).to_owned()),
        live_rate.map(model::rate),
        live_rate.map(|r| item.time_left_at(r)),
        item.resumable.map(|r| {
            if r {
                crate::i18n::t("Yes")
            } else {
                crate::i18n::t("No")
            }
            .to_owned()
        }),
        save_to,
        Some(address_text(&item.url)),
        queue.map(|q| crate::i18n::t(q).to_owned()),
        item.last_try.as_deref().map(model::date_text),
    ]);
    let primary = if running {
        Some(Primary::Stop)
    } else {
        item.can_start().then_some(Primary::Resume)
    };
    OneDownload {
        id: item.id,
        file_name: item.file_name.clone(),
        icon_ext: std::path::Path::new(&item.file_name)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default(),
        status_line: status_line(item.status, item.fraction()),
        connections: connections_label(running, item.snap.as_ref()),
        bytes: bytes_label(item.downloaded, item.size),
        cells: segment_cells(item.size, &done_ranges(item)),
        kind: StatusKind::of(item.status),
        rows,
        primary,
        can_show_progress: item.status != Status::Complete,
    }
}

/// The details pane's sizes, in points at 100 % (the mockup's `.details`).
/// Inside the pane's edges, and between its blocks.
const PANE_PAD: f32 = 14.0;
/// The file type's icon, and the room after it before the name.
const PANE_ICON: f32 = 32.0;
const PANE_ICON_GAP: f32 = 10.0;
/// The rows' label column, and the room after it before the value.
const LABEL_W: f32 = 96.0;
const LABEL_GAP: f32 = 12.0;
/// A row's line (its text 12.5 points), and the room between two rows.
const ROW_LINE: f32 = 16.0;
const ROW_GAP: f32 = 7.0;
/// A line of the name (SemiBold 13).
const NAME_LINE: f32 = 17.0;
/// The buttons' height, and the room between two.
const PANE_BUTTON_H: f32 = 32.0;
const PANE_BUTTON_GAP: f32 = 6.0;

/// Where the details pane's buttons go in its English rect `rc`, at its bottom: the primary
/// (Resume or Stop) and Open folder side by side, Show progress window under them across both.
fn details_buttons(rc: Rect) -> [Rect; 3] {
    let (l, r) = (rc.left() + 1.0 + PANE_PAD, rc.right() - PANE_PAD);
    let low = rc.bottom() - PANE_PAD - PANE_BUTTON_H;
    let high = low - PANE_BUTTON_GAP - PANE_BUTTON_H;
    let half = ((r - l - PANE_BUTTON_GAP) / 2.0).floor().max(0.0);
    [
        Rect::from_min_size(pos2(l, high), vec2(half, PANE_BUTTON_H)),
        Rect::from_min_size(pos2(r - half, high), vec2(half, PANE_BUTTON_H)),
        Rect::from_min_size(pos2(l, low), vec2((r - l).max(0.0), PANE_BUTTON_H)),
    ]
}

/// The pane's fonts: its rows' (12.5), its small labels' (12) and the name's (SemiBold 13).
fn row_font() -> egui::FontId {
    egui::FontId::proportional(12.5)
}
fn small_font() -> egui::FontId {
    egui::FontId::proportional(12.0)
}
fn name_font() -> egui::FontId {
    egui::FontId::new(13.0, egui::FontFamily::Name("tahoma-bold".into()))
}

/// Text the pane painted in English rect `rc`, readable to screen readers as `text` (keyed by
/// `salt`) and shown whole on hover while it is `cut`.
fn pane_text(
    ui: &Ui,
    rc: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    text: &str,
    cut: bool,
) {
    let id = egui::Id::unique((ui.ctx().viewport_id(), "details-text", salt));
    let resp = ui.interact(winui::rect(rc), id, egui::Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, text));
    if cut {
        resp.on_hover_text(text);
    }
}

/// `s` on at most 2 lines `room` wide from English `x`, the first baseline at `baseline`, each
/// line ending in "..." when it is still too wide ([`winui::fit_lines`]). Its lines' count and
/// whether it was cut.
fn two_lines(
    p: &egui::Painter,
    font: &egui::FontId,
    (x, baseline, line): (f32, f32, f32),
    s: &str,
    room: f32,
    ink: egui::Color32,
) -> (usize, bool) {
    let lines = winui::fit_lines(p, font.clone(), s, room);
    let mut cut = false;
    for (k, l) in lines.iter().enumerate() {
        let shown = winui::ellipsize(p, font, l, room);
        cut |= shown != *l;
        winui::text_font(p, font.clone(), x, baseline + line * k as f32, &shown, ink);
    }
    (lines.len(), cut)
}

/// One label/value row at `y` between English `l` and `r`: the label muted in its 96-point
/// column (on 2 lines where it needs them), the value cut with "..." (in its middle for the
/// Address and Save to, [`cuts_in_middle`]) and whole on hover. Its height.
fn detail_row(ui: &Ui, p: &egui::Painter, (l, r): (f32, f32), y: f32, row: (&str, &str)) -> f32 {
    let (label, value) = row;
    let c = crate::theme::colors();
    let font = row_font();
    let name = crate::i18n::t(label);
    let (n, label_cut) = two_lines(p, &font, (l, y + 12.0, ROW_LINE), name, LABEL_W, c.muted);
    let h = ROW_LINE * n as f32;
    pane_text(
        ui,
        Rect::from_min_size(pos2(l, y), vec2(LABEL_W, h)),
        ("label", label),
        name,
        label_cut,
    );
    let x = l + LABEL_W + LABEL_GAP;
    let room = (r - x).max(0.0);
    let shown = if cuts_in_middle(label) {
        winui::ellipsize_middle(p, &font, value, room)
    } else {
        winui::ellipsize(p, &font, value, room)
    };
    winui::text_font(p, font, x, y + 12.0, &shown, c.text);
    pane_text(
        ui,
        Rect::from_min_size(pos2(x, y), vec2(room, ROW_LINE)),
        ("value", label),
        value,
        shown != value,
    );
    h
}

/// Paint the details pane in English rect `rc` (C10): on `panel` with its leading rule, its
/// content 14 inside. One download: the header (the file type's 32-point icon, the name on at
/// most 2 lines, the status line), the segment cells under their label row coloured by
/// [`StatusKind`] as the list's Status cell, the label/value rows, and at the bottom the
/// buttons (Resume or Stop filled with the accent, Open folder, Show progress window greyed for
/// a finished download). A short pane cuts its content above the buttons, and everything at its
/// own edges. Nothing selected: a muted line in the middle; several: their count and total
/// size. Exposed as a group named "Details" holding its texts. What was pressed.
pub fn details_pane(
    ui: &mut Ui,
    rc: Rect,
    view: &DetailsView,
    icons: &mut crate::chrome::ShellIcons,
) -> Option<DetailsAction> {
    let c = crate::theme::colors();
    let screen = winui::rect(rc);
    let group = ui.interact(
        screen,
        egui::Id::unique((ui.ctx().viewport_id(), "details-pane")),
        egui::Sense::hover(),
    );
    // i18n: the details pane beside the download list, to screen readers
    winui::expose(
        &group,
        egui::Role::Group,
        crate::i18n::t("Details"),
        true,
        None,
    );
    let clip = screen.intersect(ui.clip_rect());
    let mut pane = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(screen)
            .accessibility_parent(group.id),
    );
    pane.set_clip_rect(clip);
    let p = pane.painter().clone();
    p.rect_filled(screen, 0.0, c.panel);
    let rule = Rect::from_min_size(rc.min, vec2(1.0, rc.height()));
    p.rect_filled(winui::rect(rule), 0.0, c.menubar_line);
    let (l, r) = (rc.left() + 1.0 + PANE_PAD, rc.right() - PANE_PAD);
    let top = rc.top() + PANE_PAD;
    let one = match view {
        DetailsView::Empty => {
            // i18n: the details pane while no download is selected
            let text = crate::i18n::t("Select a download to see its details");
            let font = row_font();
            let lines = winui::fit_lines(&p, font.clone(), text, r - l);
            let y = (rc.center().y - ROW_LINE * lines.len() as f32 / 2.0).round();
            for (k, line) in lines.iter().enumerate() {
                let shown = winui::ellipsize(&p, &font, line, r - l);
                let w = winui::text_width_font(&p, font.clone(), &shown);
                let x = ((l + r - w) / 2.0).round();
                let baseline = y + 12.0 + ROW_LINE * k as f32;
                winui::text_font(&p, font.clone(), x, baseline, &shown, c.muted);
            }
            let h = ROW_LINE * lines.len() as f32;
            let at = Rect::from_min_max(pos2(l, y), pos2(r, y + h));
            pane_text(&pane, at, "empty", text, false);
            return None;
        }
        DetailsView::Many { count, total } => {
            // i18n: the details pane's title while several downloads are selected
            let title = crate::i18n::tn(
                *count as u64,
                "{n} download selected",
                "{n} downloads selected",
            );
            let font = name_font();
            let at = (l, top + 13.0, NAME_LINE);
            let (n, cut) = two_lines(&p, &font, at, &title, r - l, c.text);
            let h = NAME_LINE * n as f32;
            let title_rc = Rect::from_min_size(pos2(l, top), vec2(r - l, h));
            pane_text(&pane, title_rc, "many", &title, cut);
            let size = crate::i18n::ltr(&model::size(*total));
            // i18n: the details pane: label before the sum of the selected downloads' sizes
            let row = (crate::i18n::mark("Total size"), size.as_str());
            detail_row(&pane, &p, (l, r), top + h + PANE_PAD, row);
            return None;
        }
        DetailsView::One(one) => one,
    };

    let [first, second, third] = details_buttons(rc);
    // The content above the buttons: a short pane cuts it there.
    let content_bottom = (first.top() - PANE_BUTTON_GAP).max(rc.top());
    let content = Rect::from_min_max(rc.min, pos2(rc.right(), content_bottom));
    // A child of `pane` made before the buttons: a screen reader reads the content, then them.
    let mut body = pane.new_child(egui::UiBuilder::new().max_rect(screen));
    body.set_clip_rect(winui::rect(content).intersect(clip));
    let bp = body.painter().clone();
    let id = one.id;

    // The header: the icon, the name on at most 2 lines, the status line under it.
    let key = crate::chrome::IconKey::ExtLarge(one.icon_ext.clone());
    icons.paint_in(&body, &bp, key, pos2(l, top));
    let nx = l + PANE_ICON + PANE_ICON_GAP;
    let room = (r - nx).max(0.0);
    let at = (nx, top + 13.0, NAME_LINE);
    let (n, cut) = two_lines(&bp, &name_font(), at, &one.file_name, room, c.text);
    let name_h = NAME_LINE * n as f32;
    let name_rc = Rect::from_min_size(pos2(nx, top), vec2(room, name_h));
    pane_text(&body, name_rc, ("name", id), &one.file_name, cut);
    let status_top = top + name_h + 2.0;
    let small = small_font();
    let shown = winui::ellipsize(&bp, &small, &one.status_line, room);
    winui::text_font(
        &bp,
        small.clone(),
        nx,
        status_top + 12.0,
        &shown,
        c.text_secondary,
    );
    let status_rc = Rect::from_min_size(pos2(nx, status_top), vec2(room, 15.0));
    let status_cut = shown != one.status_line;
    pane_text(
        &body,
        status_rc,
        ("status", id),
        &one.status_line,
        status_cut,
    );
    let mut y = top + PANE_ICON.max(name_h + 2.0 + 15.0) + PANE_PAD;

    // The segment cells under their label row: the connections at its start, the bytes at
    // its end.
    if one.connections.is_some() || one.bytes.is_some() {
        if let Some(bytes) = &one.bytes {
            let w = winui::text_width_font(&bp, small.clone(), bytes).ceil();
            winui::text_font(&bp, small.clone(), r - w, y + 12.0, bytes, c.muted);
            let at = Rect::from_min_size(pos2(r - w, y), vec2(w, 15.0));
            pane_text(&body, at, ("bytes", id), bytes, false);
        }
        if let Some(conns) = &one.connections {
            let bytes_w = one.bytes.as_ref().map_or(0.0, |b| {
                winui::text_width_font(&bp, small.clone(), b).ceil() + 8.0
            });
            let room = (r - l - bytes_w).max(0.0);
            let shown = winui::ellipsize(&bp, &small, conns, room);
            winui::text_font(&bp, small.clone(), l, y + 12.0, &shown, c.muted);
            let at = Rect::from_min_size(pos2(l, y), vec2(room, 15.0));
            pane_text(&body, at, ("connections", id), conns, shown != *conns);
        }
        y += 15.0 + 6.0;
    }
    let fill = one.kind.bar_color();
    let cell_w = ((r - l - 7.0 * 2.0) / 8.0).max(0.0);
    for (k, share) in one.cells.iter().enumerate() {
        let cell = Rect::from_min_size(pos2(l + k as f32 * (cell_w + 2.0), y), vec2(cell_w, 8.0));
        bp.rect_filled(winui::rect(cell), 2.0, c.progress_track);
        let done = cell_w * share.clamp(0.0, 1.0);
        if done > 0.5 {
            let part = Rect::from_min_size(cell.min, vec2(done, 8.0));
            bp.rect_filled(winui::rect(part), 2.0, fill);
        }
    }
    y += 8.0 + PANE_PAD;

    // The label/value rows.
    for (label, value) in &one.rows {
        y += detail_row(&body, &bp, (l, r), y, (*label, value.as_str())) + ROW_GAP;
    }

    // The buttons at the bottom.
    let (label, need, primary) = match one.primary {
        Some(Primary::Stop) => (
            // i18n: the details pane's button: stop the shown download
            "Stop",
            "tb-stop",
            Some(DetailsAction::Stop),
        ),
        // Greyed when it can do neither (a finished download).
        resume => (
            // i18n: the details pane's button: continue the shown download
            "Resume",
            "tb-resume",
            resume.map(|_| DetailsAction::Resume),
        ),
    };
    let kind = if primary.is_some() {
        ButtonKind::Accent
    } else {
        ButtonKind::Disabled
    };
    let first = winui::icon_label_button(&mut pane, winui::rect(first), need, label, kind);
    let second = winui::icon_label_button(
        &mut pane,
        winui::rect(second),
        "cat-folder-open",
        // i18n: the details pane's button: open the folder the shown download is saved in
        "Open folder",
        ButtonKind::Normal,
    );
    let progress = if one.can_show_progress {
        ButtonKind::Normal
    } else {
        ButtonKind::Disabled
    };
    let third = winui::icon_label_button(
        &mut pane,
        winui::rect(third),
        "gx-window",
        // i18n: the details pane's button: show the shown download's progress window
        "Show progress window",
        progress,
    );
    if first.clicked() {
        primary
    } else if second.clicked() {
        Some(DetailsAction::OpenFolder)
    } else {
        third.clicked().then_some(DetailsAction::ShowProgress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_keeps_the_first_line_trimmed_without_control_characters() {
        let mut bar = PasteBar {
            text: "typed before".into(),
            error: Some("Invalid address".into()),
            ..Default::default()
        };
        bar.fill("  \thttps://example.com/a\u{7}b.zip \r\nhttps://example.com/second\n");
        assert_eq!(bar.text, "https://example.com/ab.zip");
        assert_eq!(bar.error, None);
        assert!(bar.focus_next);
        // Tabs inside the line are spaces; a lone carriage return is no line break of its own.
        bar.fill("one\ttwo\rthree");
        assert_eq!(bar.text, "one two three");
        // Only spaces: an empty field, still focused next frame.
        bar.fill("   \n");
        assert_eq!(bar.text, "");
        assert!(bar.focus_next);
    }

    /// A wide band has room for everything: the hint beside the field, the button whole.
    #[test]
    fn a_wide_band_shows_the_hint_and_the_whole_button() {
        let g = paste_geometry(1240.0, 80.0, 110.0);
        assert!(g.hint);
        assert_eq!(g.button, 110.0);
        assert_eq!(g.field, 1240.0 - 2.0 * SIDE - 110.0 - 80.0 - 2.0 * GAP);
    }

    /// The hint goes first: left out once the field would be under 160 beside it, the field
    /// then taking its room, the button still whole.
    #[test]
    fn the_hint_is_left_out_before_the_field_goes_under_160() {
        // 20 + 160 + 8 + 80 + 8 + 110 = 386: just room for the field at 160.
        let fits = paste_geometry(386.0, 80.0, 110.0);
        assert!(fits.hint);
        assert_eq!(fits.field, 160.0);
        let narrow = paste_geometry(385.0, 80.0, 110.0);
        assert!(!narrow.hint);
        assert_eq!(narrow.field, 385.0 - 2.0 * SIDE - 110.0 - GAP);
        assert_eq!(narrow.button, 110.0);
    }

    /// Then the button's label gives way (its label ends in "..."): the field stays at 120.
    #[test]
    fn the_field_never_goes_under_120_the_button_cuts_instead() {
        // 20 + 120 + 8 + 110 = 258: the field at 120 with the button whole.
        let whole = paste_geometry(258.0, 80.0, 110.0);
        assert_eq!(
            (whole.field, whole.hint, whole.button),
            (120.0, false, 110.0)
        );
        let cut = paste_geometry(220.0, 80.0, 110.0);
        assert_eq!(cut.field, 120.0);
        assert!(!cut.hint);
        assert_eq!(cut.button, 220.0 - 2.0 * SIDE - 120.0 - GAP);
        assert!(cut.button < 110.0, "the label is cut");
        // Narrower than the narrowest parts: the field still 120, the button its icon alone.
        let tiny = paste_geometry(100.0, 80.0, 110.0);
        assert_eq!((tiny.field, tiny.button), (120.0, BUTTON_MIN));
    }

    /// One frame of the band at the top of a 1240-wide window, AccessKit on: the nodes, what
    /// `paste_bar` returned.
    fn band(
        ctx: &egui::Context,
        bar: &mut PasteBar,
        events: Vec<egui::Event>,
    ) -> (
        std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        Option<PasteEvent>,
    ) {
        band_with(ctx, bar, &[], events)
    }

    /// [`band`] with the addresses added before.
    fn band_with(
        ctx: &egui::Context,
        bar: &mut PasteBar,
        history: &[String],
        events: Vec<egui::Event>,
    ) -> (
        std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        Option<PasteEvent>,
    ) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1240.0, 700.0))),
            events,
            ..Default::default()
        };
        let layout = Layout::GRAPHITE.with_panes(true, None);
        let mut event = None;
        let mut out = ctx.run_ui(input, |ui| {
            event = paste_bar(ui, &layout, Pos2::ZERO, 1240.0, bar, history);
        });
        out.textures_delta.clear();
        let update = out
            .platform_output
            .accesskit_update
            .expect("AccessKit is on");
        (update.nodes.into_iter().collect(), event)
    }

    /// The field takes the focus asked for; the Download button is an accent button a screen
    /// reader presses, and pressing it submits; the field is "Address", the error its
    /// description, and an edit clears the error.
    #[test]
    fn the_band_focuses_its_field_and_download_submits() {
        use egui::accesskit::{Action, ActionRequest, Role, TreeId};
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut bar = PasteBar {
            error: Some("Invalid address".into()),
            focus_next: true,
            ..Default::default()
        };
        let (_, event) = band(&ctx, &mut bar, vec![]);
        assert_eq!(event, None);
        assert!(!bar.focus_next, "the focus asked for is taken");
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())));
        let (nodes, _) = band(&ctx, &mut bar, vec![]);
        let field = nodes
            .values()
            .find(|n| n.role() == Role::TextInput)
            .expect("the field");
        assert_eq!(field.label(), Some("Address"));
        assert_eq!(field.description(), Some("Invalid address"));
        let (&download, _) = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some("Download"))
            .expect("the Download button");
        let click = egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: download,
            data: None,
        });
        // Typing in the focused field clears the error.
        band(&ctx, &mut bar, vec![egui::Event::Text("x".into())]);
        assert_eq!(bar.text, "x");
        assert_eq!(bar.error, None);
        let (_, event) = band(&ctx, &mut bar, vec![click]);
        assert_eq!(event, Some(PasteEvent::Submit));
    }

    /// A key pressed, with `modifiers`.
    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn press(k: egui::Key) -> egui::Event {
        key(k, egui::Modifiers::NONE)
    }

    /// A context whose paste field has had the keys for a frame (its key filter set).
    fn focused(bar: &mut PasteBar, history: &[String]) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        bar.focus_next = true;
        band_with(&ctx, bar, history, vec![]);
        band_with(&ctx, bar, history, vec![]);
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())));
        ctx
    }

    fn addresses() -> Vec<String> {
        [
            "https://example.com/1",
            "https://example.com/2",
            "https://example.com/3",
        ]
        .map(String::from)
        .to_vec()
    }

    /// Enter in the field, its list closed, submits the text; the field keeps the keys.
    #[test]
    fn enter_in_the_field_submits() {
        let _fonts = crate::theme::font_tests();
        let mut bar = PasteBar {
            text: "https://example.com/a.zip".into(),
            ..Default::default()
        };
        let ctx = focused(&mut bar, &[]);
        let (_, event) = band(&ctx, &mut bar, vec![press(egui::Key::Enter)]);
        assert_eq!(event, Some(PasteEvent::Submit));
        assert_eq!(bar.text, "https://example.com/a.zip");
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())));
    }

    /// While a menu of the main window is open, Enter and Escape are the menu's: the field
    /// neither submits nor leaves, and leaves the keys in the input for the menu.
    #[test]
    fn an_open_menu_keeps_enter_and_escape_from_the_field() {
        let _fonts = crate::theme::font_tests();
        let mut bar = PasteBar {
            text: "https://example.com/a.zip".into(),
            ..Default::default()
        };
        let ctx = focused(&mut bar, &[]);
        bar.menu_open = true;
        for k in [egui::Key::Enter, egui::Key::Escape] {
            let (_, event) = band(&ctx, &mut bar, vec![press(k)]);
            assert_eq!(event, None, "{k:?}");
            assert!(ctx.input(|i| i.key_pressed(k)), "{k:?} left for the menu");
        }
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())), "kept");
        bar.menu_open = false;
        let (_, event) = band(&ctx, &mut bar, vec![press(egui::Key::Enter)]);
        assert_eq!(event, Some(PasteEvent::Submit));
    }

    /// Accepted, the field empties and the address goes on once, without a login; rejected,
    /// the text stays with Add URL's "Invalid address" and the field keeps the keys.
    #[test]
    fn submit_hands_on_an_accepted_address_and_keeps_a_rejected_text() {
        let mut bar = PasteBar {
            text: "example.org/a.zip".into(),
            error: Some("Invalid address".into()),
            ..Default::default()
        };
        let accepted = crate::dialogs::accept_address(&bar.text);
        let new = submit(&mut bar, accepted).expect("accepted");
        assert_eq!(new.url.as_str(), "https://example.org/a.zip");
        assert!(new.login.is_none());
        assert_eq!((bar.text.as_str(), bar.error.as_deref()), ("", None));

        bar.text = "not an address".into();
        let accepted = crate::dialogs::accept_address(&bar.text);
        assert!(submit(&mut bar, accepted).is_none());
        assert_eq!(bar.text, "not an address");
        assert_eq!(bar.error.as_deref(), Some("Invalid address"));
        assert!(bar.focus_next, "the field keeps the keys");
    }

    /// Ctrl+V with nothing focused: the clipboard's first line, trimmed, replaces the field's
    /// text, the error goes and the field takes the keys; nothing is submitted.
    #[test]
    fn ctrl_v_fills_the_field_and_submits_nothing() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut bar = PasteBar {
            text: "typed".into(),
            error: Some("Invalid address".into()),
            ..Default::default()
        };
        let events = [egui::Event::Paste(
            "  https://example.com/a.zip \nhttps://example.com/b.zip".into(),
        )];
        let clip = ctrl_v(true, false, false, &events).expect("the bar's paste");
        bar.fill(clip);
        assert_eq!(bar.text, "https://example.com/a.zip");
        assert_eq!(bar.error, None);
        for _ in 0..2 {
            let (_, event) = band(&ctx, &mut bar, vec![]);
            assert_eq!(event, None, "nothing submitted");
        }
        assert_eq!(bar.text, "https://example.com/a.zip");
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())));
    }

    /// No text on the clipboard (no paste, or an empty one), the bar hidden, or a menu or popup
    /// open: Ctrl+V is not the bar's and the field stays as it is.
    #[test]
    fn ctrl_v_without_text_or_with_the_bar_busy_leaves_the_field() {
        let paste = [egui::Event::Paste("https://example.com/".into())];
        assert_eq!(ctrl_v(true, false, false, &[]), None, "no paste");
        let empty = [egui::Event::Paste(String::new())];
        assert_eq!(ctrl_v(true, false, false, &empty), None, "empty");
        let v = [key(egui::Key::V, egui::Modifiers::COMMAND)];
        assert_eq!(ctrl_v(true, false, false, &v), None, "the key alone");
        assert_eq!(ctrl_v(false, false, false, &paste), None, "hidden");
        assert_eq!(ctrl_v(true, false, true, &paste), None, "a menu open");
        assert_eq!(ctrl_v(true, true, false, &paste), None, "a widget focused");
        assert_eq!(
            ctrl_v(true, false, false, &paste),
            Some("https://example.com/")
        );
    }

    /// A paste while another text box has the keys (the Find box) goes into that box, not the
    /// paste bar.
    #[test]
    fn a_paste_into_a_focused_text_box_stays_there() {
        let ctx = egui::Context::default();
        let find_id = egui::Id::unique("find-box");
        let mut find = String::new();
        let mut bar = PasteBar {
            text: "typed".into(),
            ..Default::default()
        };
        let frame = |find: &mut String, events: Vec<egui::Event>, focus: bool| {
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let mut clip = None;
            let mut out = ctx.run_ui(input, |ui| {
                let r = ui.add(egui::TextEdit::singleline(find).id(find_id));
                if focus {
                    r.request_focus();
                }
                // As at the end of `App::paint`.
                let focused = ui.memory(|m| m.focused().is_some());
                clip = ui.input(|i| ctrl_v(true, focused, false, &i.events).map(str::to_owned));
            });
            out.textures_delta.clear();
            clip
        };
        frame(&mut find, vec![], true);
        let clip = frame(
            &mut find,
            vec![egui::Event::Paste("https://example.com/".into())],
            false,
        );
        if let Some(clip) = &clip {
            bar.fill(clip);
        }
        assert_eq!(clip, None, "not the paste bar's");
        assert_eq!(find, "https://example.com/", "the Find box has it");
        assert_eq!(bar.text, "typed");
        assert!(!bar.focus_next);
    }

    /// Escape closes an open list and the field keeps the keys (no `Left`); Escape again leaves
    /// the field.
    #[test]
    fn escape_closes_the_open_list_first_then_leaves_the_field() {
        let _fonts = crate::theme::font_tests();
        let history = addresses();
        let mut bar = PasteBar::default();
        let ctx = focused(&mut bar, &history);
        bar.list_open = true;
        band_with(&ctx, &mut bar, &history, vec![]);
        assert!(bar.list_open);
        let (_, event) = band_with(&ctx, &mut bar, &history, vec![press(egui::Key::Escape)]);
        assert_eq!(event, None);
        assert!(!bar.list_open, "the list closed");
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())), "kept");
        band_with(&ctx, &mut bar, &history, vec![]);
        let (_, event) = band_with(&ctx, &mut bar, &history, vec![press(egui::Key::Escape)]);
        assert_eq!(event, Some(PasteEvent::Left));
        assert!(!ctx.memory(|m| m.has_focus(paste_field_id())), "left");
    }

    /// The address field's node in a frame's AccessKit nodes.
    fn field_node(
        nodes: &std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
    ) -> &egui::accesskit::Node {
        nodes
            .values()
            .find(|n| n.role() == egui::accesskit::Role::TextInput)
            .expect("the field")
    }

    /// Junk and Enter: `Submit`; once `App` has run [`submit`] on it, the next frame has Add
    /// URL's "Invalid address" as the field's description, the text stays and the field keeps
    /// the keys.
    #[test]
    fn junk_and_enter_submits_then_the_error_is_the_fields_description() {
        let _fonts = crate::theme::font_tests();
        let mut bar = PasteBar {
            text: "not an address".into(),
            ..Default::default()
        };
        let ctx = focused(&mut bar, &[]);
        let (nodes, event) = band(&ctx, &mut bar, vec![press(egui::Key::Enter)]);
        assert_eq!(event, Some(PasteEvent::Submit));
        assert_eq!(field_node(&nodes).description(), None, "no error yet");
        // As `App::paint` does on `Submit`.
        let accepted = crate::dialogs::accept_address(&bar.text);
        assert!(submit(&mut bar, accepted).is_none(), "junk is rejected");
        let (nodes, event) = band(&ctx, &mut bar, vec![]);
        assert_eq!(event, None);
        let field = field_node(&nodes);
        assert_eq!(field.label(), Some("Address"));
        assert_eq!(field.description(), Some("Invalid address"));
        assert_eq!(bar.text, "not an address");
        assert!(field_focused(&ctx), "the field keeps the keys");
    }

    /// The bar hidden while its field has the keys (`Cmd::TogglePasteBar` calls `leave`): the
    /// field gives up egui's focus at once, its list closes, a focus asked for is dropped, and
    /// the bar shown again leaves the field unfocused.
    #[test]
    fn hiding_the_bar_while_focused_gives_up_the_focus() {
        let _fonts = crate::theme::font_tests();
        let history = addresses();
        let mut bar = PasteBar::default();
        let ctx = focused(&mut bar, &history);
        bar.list_open = true;
        band_with(&ctx, &mut bar, &history, vec![]);
        assert!(bar.list_open && field_focused(&ctx));
        bar.focus_next = true;
        bar.leave(&ctx);
        assert!(!field_focused(&ctx), "given up at once");
        assert!(!bar.list_open && !bar.focus_next);
        // A frame with the bar hidden, then the bar shown again.
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        assert!(!field_focused(&ctx));
        band_with(&ctx, &mut bar, &history, vec![]);
        assert!(!field_focused(&ctx), "shown again, still unfocused");
    }

    /// Up / Down, the list closed, walk the history into the field and clear an error.
    #[test]
    fn up_and_down_walk_the_history_into_the_field() {
        let _fonts = crate::theme::font_tests();
        let history = addresses();
        let mut bar = PasteBar {
            error: Some("Invalid address".into()),
            ..Default::default()
        };
        let ctx = focused(&mut bar, &history);
        let mut step = |k: egui::Key| {
            let (_, event) = band_with(&ctx, &mut bar, &history, vec![press(k)]);
            assert_eq!(event, None);
            bar.text.clone()
        };
        assert_eq!(step(egui::Key::ArrowDown), history[0]);
        assert_eq!(step(egui::Key::ArrowDown), history[1]);
        assert_eq!(step(egui::Key::ArrowDown), history[2]);
        assert_eq!(step(egui::Key::ArrowDown), history[2], "stays on the last");
        assert_eq!(step(egui::Key::ArrowUp), history[1]);
        assert_eq!(bar.error, None);
        assert!(!bar.list_open);
        assert!(ctx.memory(|m| m.has_focus(paste_field_id())), "kept");
    }

    /// Alt+Down opens the list; picking an entry puts it in the field, closes the list and
    /// submits nothing.
    #[test]
    fn alt_down_opens_the_list_and_a_pick_fills_the_field() {
        let _fonts = crate::theme::font_tests();
        let history = addresses();
        let mut bar = PasteBar {
            text: "typed".into(),
            error: Some("Invalid address".into()),
            ..Default::default()
        };
        let ctx = focused(&mut bar, &history);
        let alt_down = key(egui::Key::ArrowDown, egui::Modifiers::ALT);
        let (_, event) = band_with(&ctx, &mut bar, &history, vec![alt_down]);
        assert_eq!(event, None);
        assert!(bar.list_open, "Alt+Down opens the list");
        assert_eq!(bar.text, "typed", "the field unchanged");
        // Down highlights the first entry, Enter picks it.
        band_with(&ctx, &mut bar, &history, vec![press(egui::Key::ArrowDown)]);
        assert!(bar.list_open);
        let (_, event) = band_with(&ctx, &mut bar, &history, vec![press(egui::Key::Enter)]);
        assert_eq!(event, None, "a pick submits nothing");
        assert!(!bar.list_open);
        assert_eq!(bar.text, history[0]);
        assert_eq!(bar.error, None);
    }

    /// Field texts for the submit: addresses MDM takes (with or without a scheme, magnet
    /// links), near misses and any text at all.
    fn typed() -> impl proptest::strategy::Strategy<Value = String> {
        proptest::prop_oneof![
            "(https?://|ftp://)?[a-z]{1,8}\\.(com|org|net)(/[a-z0-9._-]{0,12})?",
            "magnet:\\?xt=urn:btih:[0-9a-f]{40}",
            "[ \\t]{0,3}[a-z:/.]{0,20}[ \\t]{0,3}",
            proptest::prelude::any::<String>(),
        ]
    }

    /// Clipboard texts: line breaks, tabs and other control characters among address
    /// characters, and any text at all.
    fn clipboard() -> impl proptest::strategy::Strategy<Value = String> {
        proptest::prop_oneof![
            "[ \\t\\r\\na-z:/.\\x00-\\x1f\\x7f\\x{85}\\x{2028}]{0,40}",
            proptest::prelude::any::<String>(),
        ]
    }

    /// Paste → Tree → List → Paste forward, the other way back; a hidden stop is skipped; with
    /// only the list there is nowhere to go.
    #[test]
    fn next_stop_goes_round_the_shown_stops() {
        use Stop::{List, Paste, Tree};
        let all = |from, back| next_stop(from, back, true, true);
        assert_eq!(
            [all(Paste, false), all(Tree, false), all(List, false)],
            [Some(Tree), Some(List), Some(Paste)]
        );
        assert_eq!(
            [all(Paste, true), all(Tree, true), all(List, true)],
            [Some(List), Some(Paste), Some(Tree)]
        );
        // The paste bar hidden: the tree and the list, as before it.
        for back in [false, true] {
            assert_eq!(next_stop(Tree, back, false, true), Some(List));
            assert_eq!(next_stop(List, back, false, true), Some(Tree));
            // The Categories pane hidden: the field and the list.
            assert_eq!(next_stop(Paste, back, true, false), Some(List));
            assert_eq!(next_stop(List, back, true, false), Some(Paste));
            // Only the list.
            assert_eq!(next_stop(List, back, false, false), None);
        }
    }

    /// Property 9 for one `(from, back, paste, tree)`: a hidden stop is never returned; from a
    /// shown stop, `back`'s way round visits every shown stop once before coming back to it, the
    /// other way returns the start, the bar hidden it is Tree ↔ List and with only the list there
    /// is nowhere to go.
    fn check_rotation(
        from: Stop,
        back: bool,
        paste: bool,
        tree: bool,
    ) -> Result<(), proptest::test_runner::TestCaseError> {
        use proptest::{prop_assert, prop_assert_eq};
        let shown = |s: Stop| match s {
            Stop::Paste => paste,
            Stop::Tree => tree,
            Stop::List => true,
        };
        let stops: Vec<Stop> = [Stop::Paste, Stop::Tree, Stop::List]
            .into_iter()
            .filter(|s| shown(*s))
            .collect();
        let to = next_stop(from, back, paste, tree);
        if let Some(t) = to {
            prop_assert!(shown(t), "{t:?} is hidden");
        }
        if !shown(from) {
            return Ok(());
        }
        if stops.len() == 1 {
            prop_assert_eq!(to, None, "only the list");
            return Ok(());
        }
        if !paste {
            let other = if from == Stop::Tree {
                Stop::List
            } else {
                Stop::Tree
            };
            prop_assert_eq!(to, Some(other), "the bar hidden: Tree <-> List");
        }
        // Round from `from`: every shown stop once, then `from` again.
        let mut seen = vec![from];
        let mut at = from;
        for _ in 1..stops.len() {
            at = next_stop(at, back, paste, tree).expect("a stop to go to");
            prop_assert!(!seen.contains(&at), "{at:?} twice in {seen:?}");
            seen.push(at);
        }
        prop_assert_eq!(seen.len(), stops.len());
        prop_assert_eq!(
            next_stop(at, back, paste, tree),
            Some(from),
            "back to the start"
        );
        // There and back again.
        let there = to.expect("a stop to go to");
        prop_assert_eq!(next_stop(there, !back, paste, tree), Some(from));
        Ok(())
    }

    /// Property 9 over the whole input space (24 cases).
    #[test]
    fn the_tab_rotation_holds_for_every_input() {
        for from in [Stop::Paste, Stop::Tree, Stop::List] {
            for back in [false, true] {
                for paste in [false, true] {
                    for tree in [false, true] {
                        if let Err(e) = check_rotation(from, back, paste, tree) {
                            panic!("{from:?} back={back} paste={paste} tree={tree}: {e}");
                        }
                    }
                }
            }
        }
    }

    // Feature: paste-bar-details-pane, Property 9: Tab rotation.
    // **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**
    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(100))]
        /// For any shown sets: the way round visits every shown stop once, forward then back
        /// returns the start, a hidden stop is never returned, the bar hidden it is Tree <-> List
        /// and with only the list it is None.
        #[test]
        fn next_stop_rotates_among_the_shown_stops(
            from in proptest::sample::select(vec![Stop::Paste, Stop::Tree, Stop::List]),
            back in proptest::prelude::any::<bool>(),
            paste in proptest::prelude::any::<bool>(),
            tree in proptest::prelude::any::<bool>(),
        ) {
            check_rotation(from, back, paste, tree)?;
        }
    }

    /// The main window's parts that take the keys, as `App::paint` has them.
    #[derive(Default)]
    struct Window {
        bar: PasteBar,
        s: TabState,
        /// A control of an open filter box (drawn after the band, as the list's is).
        filter: String,
    }

    fn filter_id() -> egui::Id {
        egui::Id::unique("test-filter-box")
    }

    /// One frame of the window: the Tab rotation before any widget (when `rotate`, as
    /// `App::tab_keys` gives the tree or the list the keys), the band while shown, then the
    /// filter box's control while open. Where Tab moved the keys.
    fn window(
        ctx: &egui::Context,
        win: &mut Window,
        events: Vec<egui::Event>,
        rotate: bool,
    ) -> Option<Stop> {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1240.0, 700.0))),
            events,
            ..Default::default()
        };
        let layout = Layout::GRAPHITE.with_panes(true, None);
        let mut to = None;
        let mut out = ctx.run_ui(input, |ui| {
            if rotate {
                let from = current_stop(ui.ctx(), &win.s);
                to = tab_keys(ui.ctx(), &mut win.bar, win.s, from);
                if let Some(t) = to
                    && t != Stop::Paste
                {
                    win.s.tree_keys = t == Stop::Tree;
                }
            }
            if win.s.paste {
                win.bar.filter_open = win.s.filter_open;
                paste_bar(ui, &layout, Pos2::ZERO, 1240.0, &mut win.bar, &[]);
            }
            if win.s.filter_open {
                ui.add(egui::TextEdit::singleline(&mut win.filter).id(filter_id()));
            }
        });
        out.textures_delta.clear();
        to
    }

    fn field_focused(ctx: &egui::Context) -> bool {
        ctx.memory(|m| m.has_focus(paste_field_id()))
    }

    /// Tab from the tree with nothing focused gives the list the keys and egui's focus to no
    /// widget (egui alone gives it to the first focusable one, the paste field); Tab from the
    /// list focuses the field, Tab from the field gives the tree the keys and takes the focus
    /// from the field, Shift+Tab from the tree focuses the field again.
    #[test]
    fn tab_rotates_and_keeps_egui_focus_off_the_field_on_the_way() {
        let _fonts = crate::theme::font_tests();
        let tab = || vec![press(egui::Key::Tab)];
        let shift_tab = || vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)];
        let shown = TabState {
            paste: true,
            tree: true,
            tree_keys: true,
            ..Default::default()
        };
        // Without the rotation, egui's Tab focuses the field: the case it guards.
        let plain = egui::Context::default();
        let mut win = Window {
            s: shown,
            ..Default::default()
        };
        window(&plain, &mut win, vec![], false);
        window(&plain, &mut win, tab(), false);
        assert!(field_focused(&plain), "egui's own Tab");

        let ctx = egui::Context::default();
        let mut win = Window {
            s: shown,
            ..Default::default()
        };
        window(&ctx, &mut win, vec![], true);
        assert_eq!(window(&ctx, &mut win, tab(), true), Some(Stop::List));
        assert!(!ctx.input(|i| i.key_pressed(egui::Key::Tab)), "consumed");
        window(&ctx, &mut win, vec![], true);
        assert!(
            !field_focused(&ctx),
            "Tab to the list leaves the field alone"
        );
        assert!(!win.s.tree_keys);

        assert_eq!(window(&ctx, &mut win, tab(), true), Some(Stop::Paste));
        assert!(field_focused(&ctx), "Tab to the field focuses it");
        window(&ctx, &mut win, vec![], true);
        assert_eq!(window(&ctx, &mut win, tab(), true), Some(Stop::Tree));
        assert!(!field_focused(&ctx), "Tab from the field takes its focus");
        assert!(win.s.tree_keys);
        window(&ctx, &mut win, vec![], true);
        assert!(!field_focused(&ctx));
        assert_eq!(window(&ctx, &mut win, shift_tab(), true), Some(Stop::Paste));
        assert!(field_focused(&ctx), "Shift+Tab from the tree");
    }

    /// The paste bar hidden, Tab toggles the tree and the list as before it; with the
    /// Categories pane hidden too, Tab is left alone.
    #[test]
    fn with_the_bar_hidden_tab_toggles_the_tree_and_the_list() {
        let ctx = egui::Context::default();
        let mut win = Window {
            s: TabState {
                tree: true,
                tree_keys: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let tab = || vec![press(egui::Key::Tab)];
        assert_eq!(window(&ctx, &mut win, tab(), true), Some(Stop::List));
        assert_eq!(window(&ctx, &mut win, tab(), true), Some(Stop::Tree));
        let shift_tab = vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)];
        assert_eq!(window(&ctx, &mut win, shift_tab, true), Some(Stop::List));
        win.s.tree = false;
        assert_eq!(window(&ctx, &mut win, tab(), true), None);
        assert!(
            ctx.input(|i| i.key_pressed(egui::Key::Tab)),
            "left as it was"
        );
        // A menu open: no move either.
        win.s = TabState {
            tree: true,
            menu_open: true,
            ..Default::default()
        };
        assert_eq!(window(&ctx, &mut win, tab(), true), None);
    }

    /// An open filter box keeps Tab among its controls: Tab past its last control (or with
    /// nothing focused) reaches its first again, never the paste field or the button.
    #[test]
    fn tab_in_an_open_filter_box_stays_in_the_box() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        let mut win = Window {
            s: TabState {
                paste: true,
                tree: true,
                filter_open: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let in_box = |ctx: &egui::Context| ctx.memory(|m| m.has_focus(filter_id()));
        window(&ctx, &mut win, vec![], true);
        ctx.memory_mut(|m| m.request_focus(filter_id()));
        window(&ctx, &mut win, vec![], true);
        assert!(in_box(&ctx));
        assert_eq!(
            window(&ctx, &mut win, vec![press(egui::Key::Tab)], true),
            None
        );
        window(&ctx, &mut win, vec![], true);
        assert!(in_box(&ctx), "past the last control, back to the first");
        assert!(!field_focused(&ctx));
        ctx.memory_mut(|m| m.surrender_focus(filter_id()));
        window(&ctx, &mut win, vec![], true);
        window(&ctx, &mut win, vec![press(egui::Key::Tab)], true);
        assert!(in_box(&ctx), "nothing focused: the box's first control");
        assert!(!field_focused(&ctx));
    }

    // Feature: paste-bar-details-pane, Property 5: Field state.
    // **Validates: Requirements 7.3, 7.4, 7.6, 9.1, 9.3, 9.5**
    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]
        /// Accepted by Add URL's rule, the field empties with no error and the address goes on
        /// without a login; rejected, the text stays with "Invalid address" and the field keeps
        /// the keys; any edit after a rejection clears the error.
        #[test]
        fn a_submit_empties_an_accepted_field_and_keeps_a_rejected_one(
            text in typed(),
            error in proptest::option::of("[a-z ]{0,12}"),
            focus_next in proptest::prelude::any::<bool>(),
            edit in proptest::prelude::any::<String>(),
        ) {
            let mut bar = PasteBar {
                text: text.clone(),
                error,
                focus_next,
                ..Default::default()
            };
            let accepted = crate::dialogs::accept_address(&bar.text);
            let new = submit(&mut bar, accepted.clone());
            match accepted {
                Some(url) => {
                    let new = new.expect("an accepted address goes on");
                    proptest::prop_assert_eq!(new.url, url);
                    proptest::prop_assert!(new.login.is_none());
                    proptest::prop_assert!(bar.text.is_empty());
                    proptest::prop_assert_eq!(bar.error, None);
                }
                None => {
                    proptest::prop_assert!(new.is_none());
                    proptest::prop_assert_eq!(&bar.text, &text);
                    proptest::prop_assert_eq!(bar.error.as_deref(), Some("Invalid address"));
                    proptest::prop_assert!(bar.focus_next, "the field keeps the keys");
                    bar.text = edit.clone();
                    bar.edited();
                    proptest::prop_assert_eq!(bar.error, None);
                    proptest::prop_assert_eq!(&bar.text, &edit);
                }
            }
        }

        /// Ctrl+V's fill: the clipboard's first line, tabs and carriage returns as spaces,
        /// other control characters out, trimmed; no error and the focus asked for.
        #[test]
        fn fill_gives_one_trimmed_line_without_control_characters(
            clip in clipboard(),
            text in proptest::prelude::any::<String>(),
            error in proptest::option::of("[a-z ]{0,12}"),
        ) {
            let mut bar = PasteBar { text, error, ..Default::default() };
            bar.fill(&clip);
            let first = clip.lines().next().unwrap_or_default();
            let want: String = first
                .chars()
                .filter_map(|ch| match ch {
                    '\r' | '\t' => Some(' '),
                    ch if ch.is_control() => None,
                    ch => Some(ch),
                })
                .collect();
            proptest::prop_assert_eq!(bar.text.as_str(), want.trim());
            proptest::prop_assert!(!bar.text.chars().any(char::is_control));
            proptest::prop_assert_eq!(bar.text.as_str(), bar.text.trim());
            proptest::prop_assert_eq!(bar.error, None);
            proptest::prop_assert!(bar.focus_next);
        }
    }

    /// Without a size, or with a size of 0, every cell is empty, whatever has come.
    #[test]
    fn segment_cells_are_empty_without_a_size() {
        assert_eq!(segment_cells(None, &[(0, 500)]), [0.0; 8]);
        assert_eq!(segment_cells(Some(0), &[(0, 500)]), [0.0; 8]);
        assert_eq!(segment_cells(Some(800), &[]), [0.0; 8]);
    }

    /// `0..downloaded` fills the cells in order: half the file is the first 4 cells.
    #[test]
    fn segment_cells_fill_in_order_from_the_start() {
        let half = [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(segment_cells(Some(800), &[(0, 400)]), half);
        // A partial cell: 450 of 800 is 4 cells and half the fifth.
        let cells = segment_cells(Some(800), &[(0, 450)]);
        assert_eq!(cells, [1.0, 1.0, 1.0, 1.0, 0.5, 0.0, 0.0, 0.0]);
        // Full, and a range past the size counts no more than the size.
        assert_eq!(segment_cells(Some(800), &[(0, 800)]), [1.0; 8]);
        assert_eq!(segment_cells(Some(800), &[(0, 5_000)]), [1.0; 8]);
        // A size that 8 does not divide: the parts still cover it all.
        assert_eq!(segment_cells(Some(13), &[(0, 13)]), [1.0; 8]);
        assert_eq!(segment_cells(Some(u64::MAX), &[(0, u64::MAX)]), [1.0; 8]);
        // Under 8 bytes some parts are empty; complete is still all full.
        assert_eq!(segment_cells(Some(1), &[(0, 1)]), [1.0; 8]);
        assert_eq!(segment_cells(Some(3), &[(0, 3)]), [1.0; 8]);
        // 3 bytes are parts 2, 5 and 7 (the others are empty). With byte 0 come, part 2 and
        // the empty parts whose edge byte 0 is before are full; the rest are 0.
        let one = segment_cells(Some(3), &[(0, 1)]);
        assert_eq!(one, [1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
    }

    /// Two segments (`start..cursor`) fill the parts they are in; overlapping and unsorted
    /// ranges count each byte once.
    #[test]
    fn segment_cells_merge_two_segments() {
        // One segment from 0 has come to 150, another from 400 to 500 (parts are 100 bytes).
        let cells = segment_cells(Some(800), &[(400, 500), (0, 150)]);
        assert_eq!(cells, [1.0, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        // The same bytes twice, and one range inside another, are not counted twice.
        let twice = segment_cells(Some(800), &[(0, 150), (0, 150), (20, 60), (400, 500)]);
        assert_eq!(twice, cells);
        // Overlapping ranges across cell edges join to 50..250: half of part 0, all of part 1,
        // half of part 2.
        let across = segment_cells(Some(800), &[(150, 250), (50, 200)]);
        assert_eq!(across, [0.5, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0]);
        // An empty or reversed range adds nothing.
        assert_eq!(
            segment_cells(Some(800), &[(300, 300), (500, 400)]),
            [0.0; 8]
        );
    }

    /// The saved width as it is inside the range; under 200 the pane is 200 wide; over the
    /// bound it is the bound (half the window, or what leaves the list 200 points).
    #[test]
    fn details_width_keeps_the_saved_width_inside_its_bounds() {
        // 1240 wide, Categories 224: the bound is min(620, 816) = 620.
        assert_eq!(details_width(276, 1240.0, 224.0), 276.0);
        assert_eq!(details_width(0, 1240.0, 224.0), 200.0);
        assert_eq!(details_width(199, 1240.0, 224.0), 200.0);
        assert_eq!(details_width(100_000, 1240.0, 224.0), 620.0);
        assert_eq!(details_width(u32::MAX, 1240.0, 224.0), 620.0);
        // A wide Categories pane: the list keeps 200, so the bound is 1000 - 500 - 200 = 300.
        assert_eq!(details_width(400, 1000.0, 500.0), 300.0);
    }

    /// A window too narrow for both minimums: the pane stays 200 wide and the list gets less.
    #[test]
    fn details_width_is_200_when_the_window_has_no_room() {
        assert_eq!(details_width(276, 500.0, 224.0), 200.0);
        assert_eq!(details_width(276, 300.0, 0.0), 200.0);
        assert_eq!(details_width(276, 0.0, 0.0), 200.0);
    }

    /// Whole points: a bound between two points is taken down, and a second call gives the
    /// same width.
    #[test]
    fn details_width_is_whole_points_and_gives_itself_back() {
        // 1241 wide: half is 620.5, drawn 620.
        let d = details_width(100_000, 1241.0, 0.0);
        assert_eq!(d, 620.0);
        assert_eq!(details_width(d as u32, 1241.0, 0.0), d);
        // A fractional `list_left`: 1640 - 824.6 - 200 = 615.4 (under half, 820), drawn 615.
        assert_eq!(details_width(700, 1640.0, 824.6), 615.0);
        assert_eq!(details_width(615, 1640.0, 824.6), 615.0);
    }

    /// Property 8 (width clamp), over saved widths, window widths and Categories edges.
    mod details_width_props {
        use proptest::prelude::*;

        use crate::panes::details_width;

        /// Saved widths: ordinary ones, any `u32`, and the very large ones near `u32::MAX`.
        fn saved() -> impl Strategy<Value = u32> {
            prop_oneof![
                0u32..=1200,
                any::<u32>(),
                (u32::MAX - 1000)..=u32::MAX,
                Just(u32::MAX),
            ]
        }

        /// Window widths: narrower than 400 (no room for both minimums), wide, fractional or
        /// whole, and 0.
        fn width() -> impl Strategy<Value = f32> {
            prop_oneof![
                0f32..400.0,
                0f32..4000.0,
                (0u32..4000).prop_map(|w| w as f32),
                Just(0.0),
            ]
        }

        /// The list's leading edge: 0 (Categories hidden), fractional or whole.
        fn list_left() -> impl Strategy<Value = f32> {
            prop_oneof![Just(0.0), 0f32..1000.0, (0u32..1000).prop_map(|l| l as f32)]
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(512))]

            /// Feature: paste-bar-details-pane, Property 8: Width clamp.
            /// The drawn width is the saved one limited to
            /// `[200, max(200, min(w/2, w − list_left − 200))]` (the upper bound taken down to a
            /// whole point), a whole number, and a second call on it gives it back.
            /// **Validates: Requirements 17.1, 17.2, 17.4**
            #[test]
            fn the_width_stays_in_its_range(saved in saved(), w in width(), left in list_left()) {
                let bound = (w / 2.0).min(w - left - 200.0).max(200.0);
                let hi = f64::from(bound.floor());
                let r = details_width(saved, w, left);
                prop_assert!(r >= 200.0, "{r} under 200");
                prop_assert!(f64::from(r) <= hi, "{r} over the bound {hi} ({bound})");
                prop_assert_eq!(r.fract(), 0.0, "{} is not a whole point", r);
                // Inside the range the saved width as it is, outside it the nearer bound.
                prop_assert_eq!(f64::from(r), f64::from(saved).clamp(200.0, hi));
                prop_assert_eq!(details_width(r as u32, w, left), r);
            }
        }
    }

    /// Property 10 (segment cells), over sizes and range lists.
    mod segment_cell_props {
        use proptest::prelude::*;

        use crate::panes::{part_edge, segment_cells};

        /// The largest size whose parts (at most 16 MiB) keep the `f32` cells within one byte.
        const EXACT: u64 = 128 << 20;

        /// The bytes in each of the 8 parts of `size`.
        fn parts(size: u64) -> [u64; 8] {
            std::array::from_fn(|k| part_edge(size, k as u64 + 1) - part_edge(size, k as u64))
        }

        /// Σ cellₖ × partₖ: the bytes the cells say have come.
        fn filled(cells: &[f32; 8], size: u64) -> f64 {
            cells
                .iter()
                .zip(parts(size))
                .map(|(&c, p)| f64::from(c) * p as f64)
                .sum()
        }

        /// One byte per cell up to `EXACT`; past it, the `f32` rounding of each cell
        /// (2⁻²³ of the largest part) plus a byte, per cell.
        fn tolerance(size: u64) -> f64 {
            if size <= EXACT {
                8.0
            } else {
                let part = parts(size).into_iter().max().unwrap_or(0) as f64;
                8.0 * part * 2f64.powi(-23) + 8.0
            }
        }

        /// The bytes of `0..size` inside at least one range, found apart from `merged_ranges`:
        /// cut `0..size` at every range end and add up the pieces some range holds.
        fn covered(size: u64, done: &[(u64, u64)]) -> u64 {
            let mut points: Vec<u64> = done
                .iter()
                .flat_map(|&(s, e)| [s.min(size), e.min(size)])
                .collect();
            points.sort_unstable();
            points.dedup();
            points
                .windows(2)
                .filter(|w| done.iter().any(|&(s, e)| s <= w[0] && w[1] <= e))
                .map(|w| w[1] - w[0])
                .sum()
        }

        /// Sizes up to `EXACT`, often tiny (fewer bytes than cells).
        fn exact_size() -> impl Strategy<Value = u64> {
            prop_oneof![1u64..=64, 1u64..=EXACT]
        }

        /// Any size above 0, up to `u64::MAX`.
        fn any_size() -> impl Strategy<Value = u64> {
            prop_oneof![1u64..=64, 1u64..=EXACT, 1u64..=u64::MAX, Just(u64::MAX)]
        }

        /// A range end: mostly within a quarter past `size`, sometimes 0, `size` or anything.
        fn endpoint(size: u64) -> impl Strategy<Value = u64> {
            let past = size.saturating_add(size / 4);
            prop_oneof![
                4 => 0..=past,
                1 => Just(0),
                1 => Just(size),
                1 => any::<u64>(),
            ]
        }

        /// A size and up to 8 ranges for it (unsorted, overlapping, empty or reversed).
        fn sized_ranges() -> impl Strategy<Value = (u64, Vec<(u64, u64)>)> {
            any_size().prop_flat_map(|size| {
                (
                    Just(size),
                    prop::collection::vec((endpoint(size), endpoint(size)), 0..8),
                )
            })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(256))]

            /// Every cell is a share in `[0, 1]`.
            /// **Validates: Requirements 13.1**
            #[test]
            fn every_cell_is_a_share((size, done) in sized_ranges()) {
                let cells = segment_cells(Some(size), &done);
                prop_assert!(cells.iter().all(|c| (0.0..=1.0).contains(c)), "{cells:?}");
            }

            /// `0..d` fills the cells in order and they hold `min(d, size)` bytes, within one
            /// byte per cell.
            /// **Validates: Requirements 13.1**
            #[test]
            fn one_range_fills_in_order(
                (size, d) in exact_size().prop_flat_map(|s| (Just(s), endpoint(s))),
            ) {
                let cells = segment_cells(Some(size), &[(0, d)]);
                prop_assert!(cells.windows(2).all(|w| w[0] >= w[1]), "not in order: {cells:?}");
                let (got, want) = (filled(&cells, size), d.min(size) as f64);
                prop_assert!((got - want).abs() <= 8.0, "{got} for {want}: {cells:?}");
            }

            /// The same for any size, within the `f32` cells' relative error.
            /// **Validates: Requirements 13.1**
            #[test]
            fn one_range_fills_in_order_at_any_size(
                (size, d) in any_size().prop_flat_map(|s| (Just(s), endpoint(s))),
            ) {
                let cells = segment_cells(Some(size), &[(0, d)]);
                prop_assert!(cells.windows(2).all(|w| w[0] >= w[1]), "not in order: {cells:?}");
                let (got, want) = (filled(&cells, size), d.min(size) as f64);
                let tol = tolerance(size);
                prop_assert!((got - want).abs() <= tol, "{got} for {want} (± {tol}): {cells:?}");
            }

            /// For any ranges, the cells hold the bytes the ranges cover, each byte once.
            /// **Validates: Requirements 13.1**
            #[test]
            fn cells_hold_the_covered_bytes((size, done) in sized_ranges()) {
                let cells = segment_cells(Some(size), &done);
                let (got, want) = (filled(&cells, size), covered(size, &done) as f64);
                let tol = tolerance(size);
                prop_assert!((got - want).abs() <= tol, "{got} for {want} (± {tol}): {cells:?}");
            }

            /// The order of the ranges does not change the cells.
            /// **Validates: Requirements 13.1**
            #[test]
            fn range_order_does_not_matter(
                (size, done, shuffled) in sized_ranges().prop_flat_map(|(s, v)| {
                    (Just(s), Just(v.clone()), Just(v).prop_shuffle())
                }),
            ) {
                prop_assert_eq!(
                    segment_cells(Some(size), &done),
                    segment_cells(Some(size), &shuffled)
                );
            }

            /// Without a size, or with a size of 0, every cell is empty.
            /// **Validates: Requirements 13.2**
            #[test]
            fn no_size_no_cells(
                zero in any::<bool>(),
                done in prop::collection::vec((any::<u64>(), any::<u64>()), 0..8),
            ) {
                let size = zero.then_some(0);
                prop_assert_eq!(segment_cells(size, &done), [0.0; 8]);
            }
        }
    }
}

/// The details pane's view model (task 8.5): example tables.
#[cfg(test)]
mod details_tests {
    use super::*;
    use crate::model::Category;
    use mdm_engine::Snapshot;
    use mdm_engine::download::{ConnStatus, ConnView, SegmentView, State};

    const MB: u64 = 1024 * 1024;

    /// The engine's view of a download: `segments` segments, connections in `conns`.
    fn snap(segments: u64, conns: &[ConnStatus]) -> Snapshot {
        Snapshot {
            state: State::Cancelled,
            url: url::Url::parse("https://example.org/a.zip").unwrap(),
            file: None,
            size: Some(800),
            downloaded: 0,
            resumable: Some(true),
            segments: (0..segments)
                .map(|k| SegmentView {
                    start: k * 100,
                    cursor: k * 100 + 50,
                    end: k * 100 + 100,
                })
                .collect(),
            connections: (0u32..)
                .zip(conns)
                .map(|(id, status)| ConnView {
                    id,
                    downloaded: 0,
                    status: status.clone(),
                    speed: 0,
                })
                .collect(),
            error: None,
            failure: None,
            events: Vec::new(),
            server: None,
        }
    }

    fn item(url: &str, status: Status) -> Item {
        let dest = std::path::PathBuf::from(r"D:\Downloads\Compressed\a.zip");
        let mut it = Item::new(7, url.parse().unwrap(), dest, Category::Compressed);
        it.status = status;
        it
    }

    #[test]
    fn status_line_table() {
        let cases = [
            (Status::Stopped, Some(0.63), "Stopped, 63.00%"),
            (Status::Downloading, Some(0.1338), "Downloading, 13.38%"),
            (Status::Queued, Some(0.5), "Waiting, 50.00%"),
            (Status::Error, Some(0.25), "Error, 25.00%"),
            // Complete never adds its share; no share known, the word alone.
            (Status::Complete, Some(1.0), "Complete"),
            (Status::Stopped, None, "Stopped"),
            (Status::Queued, None, "Waiting"),
            (Status::Downloading, None, "Downloading"),
        ];
        for (status, fraction, want) in cases {
            assert_eq!(
                status_line(status, fraction),
                want,
                "{status:?} {fraction:?}"
            );
        }
    }

    #[test]
    fn connections_label_table() {
        let working = [
            ConnStatus::Receiving,
            ConnStatus::Receiving,
            ConnStatus::Connecting,
            ConnStatus::Finished,
        ];
        let cases = [
            // Running: the connections working now (a finished one is not).
            (true, Some(snap(8, &working)), Some("3 connections")),
            (
                true,
                Some(snap(1, &[ConnStatus::Receiving])),
                Some("1 connection"),
            ),
            // Running without engine connections (a video, a torrent): no label.
            (true, Some(snap(1, &[])), None),
            // Stopped with this session's view: its segments.
            (
                false,
                Some(snap(8, &[ConnStatus::Finished])),
                Some("8 connections"),
            ),
            (false, Some(snap(0, &[])), None),
            // After a restart there is no view to count.
            (false, None, None),
            (true, None, None),
        ];
        for (running, snap, want) in cases {
            assert_eq!(
                connections_label(running, snap.as_ref()).as_deref(),
                want,
                "running {running}"
            );
        }
    }

    #[test]
    fn bytes_label_with_and_without_size() {
        assert_eq!(
            bytes_label(60_775_000, Some(92 * MB)).as_deref(),
            Some("57.96 MB of 92.00 MB")
        );
        assert_eq!(
            bytes_label(0, Some(512)).as_deref(),
            Some("0 Bytes of 512 Bytes")
        );
        assert_eq!(bytes_label(60_775_000, None), None);
    }

    #[test]
    fn selection_shows_empty_one_or_many() {
        let sizes = [(1, Some(10)), (2, None), (3, Some(5))];
        assert_eq!(shown(&[], &sizes), Shown::Empty);
        assert_eq!(shown(&[2], &sizes), Shown::One(2));
        // The one selected download is gone.
        assert_eq!(shown(&[9], &sizes), Shown::Empty);
        // Several: their count, the sum of the sizes known (none for 2, 9 is gone).
        assert_eq!(
            shown(&[1, 2, 3, 9], &sizes),
            Shown::Many {
                count: 4,
                total: 15
            }
        );
    }

    #[test]
    fn the_address_never_shows_its_password() {
        let url = url::Url::parse("ftp://user:s3cret@example.org/a.zip").unwrap();
        let text = address_text(&url);
        assert!(!text.contains("s3cret"), "{text}");
        assert!(text.contains("ftp://user@example.org/a.zip"), "{text}");
        // A magnet link (no host) is left as it is.
        let magnet = url::Url::parse("magnet:?xt=urn:btih:abc").unwrap();
        assert!(address_text(&magnet).contains("magnet:?xt=urn:btih:abc"));
    }

    #[test]
    fn rows_come_in_order_and_empty_ones_are_left_out() {
        let mut it = item("https://user:s3cret@example.org/a.zip", Status::Stopped);
        it.size = Some(100 * MB);
        it.downloaded = 63 * MB;
        it.resumable = Some(true);
        it.last_try = Some("Oct 28 21:44:12 2026".into());
        // "Use authorization"'s login is never shown.
        it.login = Some(("someone".into(), "hunter2".into()));
        let one = one_download(&it, None, Some("Main download queue"));
        let labels: Vec<&str> = one.rows.iter().map(|(l, _)| *l).collect();
        assert_eq!(
            labels,
            [
                "Status",
                "Resume capability",
                "Save to",
                "Address",
                "Queue",
                "Last Try Date"
            ]
        );
        let value = |label: &str| {
            one.rows
                .iter()
                .find(|(l, _)| *l == label)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(value("Status"), "Stopped");
        assert_eq!(value("Resume capability"), "Yes");
        assert!(value("Save to").contains(r"D:\Downloads\Compressed"));
        assert_eq!(value("Queue"), "Main download queue");
        for (_, v) in &one.rows {
            assert!(!v.contains("s3cret") && !v.contains("hunter2"), "{v}");
        }
        assert_eq!(one.status_line, "Stopped, 63.00%");
        assert_eq!(one.bytes.as_deref(), Some("63.00 MB of 100.00 MB"));
        assert_eq!(one.icon_ext, "zip");
        assert_eq!(one.kind, StatusKind::Stopped);
        assert_eq!(one.primary, Some(Primary::Resume));
        assert!(one.can_show_progress);
        // No view of the engine's: the cells from 0..downloaded.
        assert_eq!(one.cells[..5], [1.0; 5]);
        assert_eq!(one.cells[7], 0.0);
        assert_eq!(one.connections, None);
    }

    #[test]
    fn a_running_download_shows_its_rate_and_time_left_and_stop() {
        let mut it = item("https://example.org/a.zip", Status::Downloading);
        it.size = Some(100 * MB);
        it.downloaded = 50 * MB;
        let one = one_download(&it, Some(MB as f64), None);
        let labels: Vec<&str> = one.rows.iter().map(|(l, _)| *l).collect();
        assert_eq!(
            labels,
            ["Status", "Transfer rate", "Time left", "Save to", "Address"]
        );
        assert_eq!(one.rows[1].1, model::rate(MB as f64));
        assert_eq!(one.rows[2].1, model::time(50.0));
        assert_eq!(one.primary, Some(Primary::Stop));
        // Running with no data coming yet: no time left to say.
        let one = one_download(&it, Some(0.0), None);
        assert!(one.rows.iter().all(|(l, _)| *l != "Time left"));
    }

    #[test]
    fn a_finished_download_has_no_primary_and_no_progress_window() {
        let mut it = item("https://example.org/a.zip", Status::Complete);
        it.size = Some(10);
        it.downloaded = 10;
        let one = one_download(&it, None, None);
        assert_eq!(one.status_line, "Complete");
        assert_eq!(one.primary, None);
        assert!(!one.can_show_progress);
        assert_eq!(one.cells, [1.0; 8]);
    }

    #[test]
    fn done_ranges_read_the_snapshot_else_what_has_come() {
        let mut it = item("https://example.org/a.zip", Status::Stopped);
        it.downloaded = 42;
        assert_eq!(done_ranges(&it), [(0, 42)]);
        it.snap = Some(snap(2, &[]));
        assert_eq!(done_ranges(&it), [(0, 50), (100, 150)]);
        it.snap = Some(snap(0, &[]));
        assert_eq!(done_ranges(&it), [(0, 42)]);
    }

    #[test]
    fn the_address_and_save_to_are_cut_in_the_middle() {
        assert!(cuts_in_middle("Address"));
        assert!(cuts_in_middle("Save to"));
        assert!(!cuts_in_middle("Status"));
    }

    /// A connection's state from a number: the first three work (connecting, receiving,
    /// waiting to retry), the others do not.
    fn conn_status(k: u8) -> ConnStatus {
        use mdm_engine::download::Error;
        match k % 6 {
            0 => ConnStatus::Connecting,
            1 => ConnStatus::Receiving,
            2 => ConnStatus::Retrying(Error::BotCheck),
            3 => ConnStatus::Finished,
            4 => ConnStatus::Failed(Error::BotCheck),
            _ => ConnStatus::Retired {
                host: "example.org".into(),
                cap: 2,
            },
        }
    }

    const STATUSES: [Status; 5] = [
        Status::Queued,
        Status::Downloading,
        Status::Stopped,
        Status::Complete,
        Status::Error,
    ];

    /// A live rate: None (not running), at most 1 byte/s (no time left to say) or more.
    fn live_rate() -> impl proptest::strategy::Strategy<Value = Option<f64>> {
        use proptest::prelude::*;
        prop_oneof![
            Just(None),
            Just(Some(0.0)),
            (0.0f64..=1.0).prop_map(Some),
            (1.0f64..1e9).prop_map(Some),
        ]
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

        /// Feature: paste-bar-details-pane, Property 11: Details selection. Nothing selected is
        /// `Empty`; one is that download while it is there, else `Empty`; several are their
        /// count and the sum of their known sizes.
        /// **Validates: Requirements 14.1, 14.2, 14.3**
        #[test]
        fn details_selection_kind(
            there in proptest::collection::btree_map(
                0u64..24,
                proptest::option::of(0u64..1 << 40),
                0..12,
            ),
            picked in proptest::collection::btree_set(0u64..24, 0..8),
        ) {
            let ids: Vec<u64> = picked.into_iter().collect();
            let sizes: Vec<(u64, Option<u64>)> = there.iter().map(|(&i, &s)| (i, s)).collect();
            let want = match ids.as_slice() {
                [] => Shown::Empty,
                [id] if there.contains_key(id) => Shown::One(*id),
                [_] => Shown::Empty,
                _ => Shown::Many {
                    count: ids.len(),
                    total: ids.iter().filter_map(|i| there.get(i).copied().flatten()).sum(),
                },
            };
            proptest::prop_assert_eq!(shown(&ids, &sizes), want);
        }

        /// Feature: paste-bar-details-pane, Property 11: Details selection. One download's rows
        /// keep the fixed order with no empty value, each conditional row is there iff its
        /// condition holds, no value holds the address's password, and the connections label
        /// is there iff it runs with a working connection or, stopped, its snapshot has a
        /// segment (D7).
        /// **Validates: Requirements 12.3, 12.4, 12.5, 12.6, 12.7, 12.8, 12.9, 13.4, 13.5, 13.6**
        #[test]
        fn details_rows_follow_their_conditions(
            id in 0u64..1000,
            status in proptest::sample::select(STATUSES.to_vec()),
            size in proptest::option::of(0u64..1 << 40),
            downloaded in 0u64..1 << 40,
            rate in live_rate(),
            queue in proptest::option::of("[A-Z][a-z]{0,10}( [a-z]{1,8}){0,2}"),
            scheme in proptest::sample::select(vec!["http", "https", "ftp"]),
            user in proptest::option::of("[a-z]{1,8}"),
            password in proptest::option::of("Pw9[A-Za-z0-9]{4,10}"),
            host in "[a-z]{3,10}\\.(org|com)",
            file in "[a-z]{1,8}\\.(zip|iso|mp4)",
            folder in "[A-Za-z]{1,10}",
            last_try in proptest::option::of(
                "(Jan|Feb|Oct) (0[1-9]|1[0-9]|2[0-8]) ([01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9] 20[0-9]{2}",
            ),
            resumable in proptest::option::of(proptest::prelude::any::<bool>()),
            snapshot in proptest::option::of((
                0u64..9,
                proptest::collection::vec(0u8..6, 0..6),
            )),
        ) {
            use proptest::{prop_assert, prop_assert_eq};
            let auth = match (&user, &password) {
                (None, None) => String::new(),
                (u, p) => format!(
                    "{}{}@",
                    u.as_deref().unwrap_or_default(),
                    p.as_deref().map(|p| format!(":{p}")).unwrap_or_default()
                ),
            };
            let url: url::Url = format!("{scheme}://{auth}{host}/{file}").parse().unwrap();
            let dest = std::path::PathBuf::from(format!(r"D:\Downloads\{folder}\{file}"));
            let mut it = Item::new(id, url, dest, Category::Compressed);
            it.status = status;
            it.size = size;
            it.downloaded = downloaded;
            it.resumable = resumable;
            it.last_try = last_try.clone();
            it.snap = snapshot.as_ref().map(|(segments, conns)| {
                let conns: Vec<ConnStatus> = conns.iter().map(|&k| conn_status(k)).collect();
                snap(*segments, &conns)
            });
            let one = one_download(&it, rate, queue.as_deref());
            prop_assert_eq!(one.id, id);

            // A subsequence of the fixed order, no value empty.
            let at: Vec<usize> = one
                .rows
                .iter()
                .map(|(l, _)| ROWS.iter().position(|r| r == l).unwrap())
                .collect();
            prop_assert!(at.windows(2).all(|w| w[0] < w[1]), "{:?}", one.rows);
            for (label, v) in &one.rows {
                prop_assert!(!v.trim().is_empty(), "{label} is empty");
            }

            let value = |label: &str| {
                one.rows.iter().find(|(l, _)| *l == label).map(|(_, v)| v.as_str())
            };
            let running = rate.is_some();
            prop_assert_eq!(value("Status"), Some(status_word(status)));
            prop_assert_eq!(value("Transfer rate").is_some(), running);
            prop_assert_eq!(
                value("Time left").is_some(),
                rate.is_some_and(|r| r > 1.0) && status == Status::Downloading && size.is_some()
            );
            prop_assert_eq!(
                value("Resume capability"),
                resumable.map(|r| if r { "Yes" } else { "No" })
            );
            prop_assert!(value("Save to").is_some_and(|v| v.contains(folder.as_str())));
            let address = value("Address");
            prop_assert!(address.is_some_and(|v| v.contains(host.as_str())), "{address:?}");
            prop_assert_eq!(value("Queue").is_some(), queue.is_some());
            prop_assert_eq!(value("Last Try Date").is_some(), last_try.is_some());
            if let Some(p) = &password {
                for (label, v) in &one.rows {
                    prop_assert!(!v.contains(p.as_str()), "{label}: {v}");
                }
            }

            // The connections label (D7).
            let count = snapshot.as_ref().and_then(|(segments, conns)| {
                let n = if running {
                    conns.iter().filter(|&&k| k % 6 < 3).count() as u64
                } else {
                    *segments
                };
                (n >= 1).then_some(n)
            });
            prop_assert_eq!(one.connections.is_some(), count.is_some(), "{:?}", one.connections);
            if let (Some(label), Some(n)) = (&one.connections, count) {
                let want = format!("{n} connection");
                prop_assert!(label.starts_with(&want), "{label}");
            }
        }
    }

    /// The pane drawn 276 × 600 at the trailing edge of a 1240-wide window: what it returned
    /// over `frames`, one frame for each event list.
    fn pane(view: &DetailsView, frames: Vec<Vec<egui::Event>>) -> Option<DetailsAction> {
        let ctx = egui::Context::default();
        let rc = Rect::from_min_size(pos2(964.0, 58.0), vec2(276.0, 600.0));
        let mut action = None;
        for events in frames {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1240.0, 700.0))),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let mut icons = crate::chrome::ShellIcons;
                action = action.or(details_pane(ui, rc, view, &mut icons));
            });
            out.textures_delta.clear();
        }
        action
    }

    /// A pointer click on the pane's primary button: over it, pressed, released.
    fn click_primary() -> Vec<Vec<egui::Event>> {
        let rc = Rect::from_min_size(pos2(964.0, 58.0), vec2(276.0, 600.0));
        let at = crate::winui::rect(details_buttons(rc)[0]).center();
        let button = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        vec![
            vec![egui::Event::PointerMoved(at)],
            vec![button(true)],
            vec![button(false)],
        ]
    }

    /// Smoke test (task 8.8): every state paints in a 276 × 600 pane, long texts cut, and a
    /// click on the primary button asks for what it shows (nothing for a finished download,
    /// whose primary is greyed).
    #[test]
    fn the_pane_paints_each_state_and_its_primary_button_answers() {
        let _fonts = crate::theme::font_tests();
        assert_eq!(pane(&DetailsView::Empty, click_primary()), None);
        let many = DetailsView::Many {
            count: 3,
            total: 92 * MB,
        };
        assert_eq!(pane(&many, click_primary()), None);
        let long = "https://example.org/a/very/long/path/to/the/archive/with/many/parts/\
                    rust-1.80.0-x86_64-pc-windows-msvc-with-a-long-name.zip";
        let cases = [
            (Status::Downloading, Some(1e6), Some(DetailsAction::Stop)),
            (Status::Stopped, None, Some(DetailsAction::Resume)),
            (Status::Complete, None, None),
        ];
        for (status, rate, want) in cases {
            let mut it = item(long, status);
            it.file_name = "rust-1.80.0-x86_64-pc-windows-msvc-with-a-long-name.zip".into();
            it.size = Some(92 * MB);
            it.downloaded = if status == Status::Complete {
                92 * MB
            } else {
                58 * MB
            };
            it.resumable = Some(true);
            it.snap = Some(snap(8, &vec![ConnStatus::Receiving; 8]));
            let one = one_download(&it, rate, Some("Main download queue"));
            assert_eq!(one.primary.is_some(), want.is_some(), "{status:?}");
            let view = DetailsView::One(Box::new(one));
            assert_eq!(pane(&view, click_primary()), want, "{status:?}");
        }
    }

    /// What a screen reader finds under the pane's nodes: a text, or a button by its name and
    /// whether it is greyed.
    #[derive(Debug, PartialEq)]
    enum Read {
        Text(String),
        Button(String, bool),
    }

    /// The pane drawn 276 × 600 as in [`pane`], AccessKit on, two frames (the second has every
    /// node the first made): what a screen reader reads inside the one group named "Details",
    /// in its order, through the containers AccessKit leaves out. Every text and button of
    /// the frame is in that group.
    fn details_group(view: &DetailsView) -> Vec<Read> {
        use egui::accesskit::{Node, NodeId, Role};
        use std::collections::HashMap;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let rc = Rect::from_min_size(pos2(964.0, 58.0), vec2(276.0, 600.0));
        let mut nodes: HashMap<NodeId, Node> = HashMap::new();
        for _ in 0..2 {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1240.0, 700.0))),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let mut icons = crate::chrome::ShellIcons;
                details_pane(ui, rc, view, &mut icons);
            });
            out.textures_delta.clear();
            let update = out
                .platform_output
                .accesskit_update
                .expect("AccessKit is on");
            nodes = update.nodes.into_iter().collect();
        }
        let groups: Vec<&Node> = nodes
            .values()
            .filter(|n| n.role() == Role::Group && n.label() == Some("Details"))
            .collect();
        assert_eq!(groups.len(), 1, "one group named Details");
        fn walk(nodes: &HashMap<NodeId, Node>, node: &Node, out: &mut Vec<Read>) {
            for child in node.children().iter().filter_map(|c| nodes.get(c)) {
                let name = || {
                    child
                        .label()
                        .or(child.value())
                        .unwrap_or_default()
                        .to_owned()
                };
                match child.role() {
                    Role::Label => out.push(Read::Text(name())),
                    Role::Button => out.push(Read::Button(name(), child.is_disabled())),
                    _ => {}
                }
                walk(nodes, child, out);
            }
        }
        let mut reads = Vec::new();
        walk(&nodes, groups[0], &mut reads);
        let outside = nodes
            .values()
            .filter(|n| matches!(n.role(), Role::Label | Role::Button))
            .count();
        assert_eq!(
            outside,
            reads.len(),
            "every text and button is in the group"
        );
        reads
    }

    fn text(s: &str) -> Read {
        Read::Text(s.to_owned())
    }

    /// One stopped download (Requirement 18.1, 18.3): the "Details" group holds its name,
    /// status line, connections and bytes, then each row's label read right before its value,
    /// whole even where the value is drawn cut; then Resume, Open folder and Show progress
    /// window as buttons by their labels. Running, the primary is Stop; complete, Resume and
    /// Show progress window are greyed and Open folder is not.
    #[test]
    fn the_details_group_reads_each_label_before_its_value_and_the_buttons() {
        let _fonts = crate::theme::font_tests();
        let long = "https://user:secret@example.org/a/very/long/path/to/the/archive/\
                    rust-1.80.0-x86_64-pc-windows-msvc.zip";
        let cases = [
            (Status::Stopped, None, ("Resume", false), false),
            (Status::Downloading, Some(1e6), ("Stop", false), false),
            (Status::Complete, None, ("Resume", true), true),
        ];
        for (status, rate, (primary, primary_greyed), progress_greyed) in cases {
            let mut it = item(long, status);
            it.file_name = "rust-1.80.0-x86_64-pc-windows-msvc.zip".into();
            it.size = Some(92 * MB);
            it.downloaded = 58 * MB;
            it.resumable = Some(true);
            it.last_try = Some("Oct 28 21:44:12 2026".into());
            it.snap = Some(snap(8, &vec![ConnStatus::Receiving; 8]));
            let one = one_download(&it, rate, Some("Main download queue"));
            let mut want = vec![text(&one.file_name), text(&one.status_line)];
            want.extend(one.bytes.as_deref().map(text));
            want.extend(one.connections.as_deref().map(text));
            for (label, value) in &one.rows {
                want.extend([text(label), text(value)]);
            }
            want.extend([
                Read::Button(primary.into(), primary_greyed),
                Read::Button("Open folder".into(), false),
                Read::Button("Show progress window".into(), progress_greyed),
            ]);
            let labels: Vec<&str> = one.rows.iter().map(|(l, _)| *l).collect();
            for wanted in ["Status", "Resume capability", "Save to", "Address", "Queue"] {
                assert!(labels.contains(&wanted), "{status:?}: {labels:?}");
            }
            let reads = details_group(&DetailsView::One(Box::new(one)));
            assert_eq!(reads, want, "{status:?}");
            let secret = reads.iter().any(|r| match r {
                Read::Text(s) | Read::Button(s, _) => s.contains("secret"),
            });
            assert!(!secret, "no password read out");
        }
    }

    /// Nothing selected, or several (Requirement 18.2): the group holds the state's text, the
    /// count and the Total size row, and no button.
    #[test]
    fn the_details_group_reads_the_empty_and_the_many_state() {
        let _fonts = crate::theme::font_tests();
        assert_eq!(
            details_group(&DetailsView::Empty),
            [text("Select a download to see its details")]
        );
        let many = DetailsView::Many {
            count: 3,
            total: 92 * MB,
        };
        let size = crate::i18n::ltr(&model::size(92 * MB));
        assert_eq!(
            details_group(&many),
            [
                text("3 downloads selected"),
                text("Total size"),
                text(&size)
            ]
        );
    }

    /// The toolbar's Stop and Resume for one selected download (`App::paint`): Stop while it
    /// runs (`live.is_some()`), Resume while it can start (`can_start()`, which is false while
    /// it runs). A test item has no live transfer, so "it runs" is the live rate given, as
    /// `App::one_download` passes it.
    fn toolbar_stop_resume(it: &Item, rate: Option<f64>) -> (bool, bool) {
        let running = rate.is_some();
        (running, !running && it.can_start())
    }

    /// The primary the toolbar's rules call for: Stop, else Resume, else none.
    fn wanted_primary(it: &Item, rate: Option<f64>) -> Option<Primary> {
        match toolbar_stop_resume(it, rate) {
            (true, _) => Some(Primary::Stop),
            (false, true) => Some(Primary::Resume),
            (false, false) => None,
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

        /// Feature: paste-bar-details-pane, Property 12: Primary button. Stop iff it runs and
        /// Resume iff it can start, as the toolbar's Stop and Resume for the same download;
        /// no primary when neither is enabled (finished and not a gallery, or "Downloading"
        /// with no live transfer); "Show progress window" greyed iff it is complete.
        /// **Validates: Requirements 15.1, 15.2, 15.4**
        #[test]
        fn primary_button_matches_the_toolbar(
            status in proptest::sample::select(STATUSES.to_vec()),
            rate in live_rate(),
            gallery in proptest::option::of(proptest::option::of(1u64..100)),
            size in proptest::option::of(0u64..1 << 30),
            downloaded in 0u64..1 << 30,
        ) {
            use proptest::prop_assert_eq;
            let mut it = item("https://example.org/a.zip", status);
            it.gallery = gallery.map(|limit| model::GallerySpec { limit });
            it.size = size;
            it.downloaded = downloaded;
            let one = one_download(&it, rate, None);
            let (stop, resume) = toolbar_stop_resume(&it, rate);
            prop_assert_eq!(one.primary == Some(Primary::Stop), stop);
            prop_assert_eq!(one.primary == Some(Primary::Resume), resume);
            prop_assert_eq!(one.primary.is_none(), !stop && !resume);
            if rate.is_none() {
                prop_assert_eq!(one.primary == Some(Primary::Resume), it.can_start());
            }
            prop_assert_eq!(one.can_show_progress, status != Status::Complete);
        }

        /// Feature: paste-bar-details-pane, Property 12: Primary button. The same download
        /// rebuilt after its status changes shows the new status word and the matching
        /// primary; Stop then Resume swap the button each time.
        /// **Validates: Requirements 15.1, 15.2, 15.4, 16.3**
        #[test]
        fn primary_button_follows_a_status_change(
            before in (proptest::sample::select(STATUSES.to_vec()), live_rate()),
            after in (proptest::sample::select(STATUSES.to_vec()), live_rate()),
            running_rate in 0.0f64..1e9,
        ) {
            use proptest::prop_assert_eq;
            let mut it = item("https://example.org/a.zip", before.0);
            let one = one_download(&it, before.1, None);
            prop_assert_eq!(one.primary, wanted_primary(&it, before.1));
            it.status = after.0;
            let again = one_download(&it, after.1, None);
            prop_assert_eq!(again.primary, wanted_primary(&it, after.1));
            prop_assert_eq!(again.rows[0].0, "Status");
            prop_assert_eq!(again.rows[0].1.as_str(), status_word(after.0));
            prop_assert_eq!(again.can_show_progress, after.0 != Status::Complete);

            // Running, then Stop (it ends Stopped), then Resume (it runs again).
            it.status = Status::Downloading;
            let running = one_download(&it, Some(running_rate), None);
            prop_assert_eq!(running.primary, Some(Primary::Stop));
            it.status = Status::Stopped;
            let stopped = one_download(&it, None, None);
            prop_assert_eq!(stopped.primary, Some(Primary::Resume));
            prop_assert_eq!(stopped.rows[0].1.as_str(), status_word(Status::Stopped));
            it.status = Status::Downloading;
            let resumed = one_download(&it, Some(running_rate), None);
            prop_assert_eq!(resumed.primary, Some(Primary::Stop));
            prop_assert_eq!(resumed.rows[0].1.as_str(), status_word(Status::Downloading));
        }
    }
}
