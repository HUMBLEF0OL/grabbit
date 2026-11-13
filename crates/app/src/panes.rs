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

use egui::{Pos2, Rect, Ui, pos2, vec2};

use crate::chrome::Layout;
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
}
