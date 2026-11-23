//! Find (Ctrl+F) / Find Next (F3), in the main window (1.0.1): the title bar's search field
//! filters the list as you type, and its funnel's dropdown chooses where to look (the file name,
//! the description, the site) and how (match case, whole word). Was IDM's "Find file" dialog
//! (notes/ui-spec/scheduler-and-misc.md §4).

use crate::model::Item;

/// What to look for: the search field's text and the dropdown's choices.
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub text: String,
    pub name: bool,
    pub description: bool,
    pub site: bool,
    pub case: bool,
    pub whole: bool,
}

impl Default for Query {
    fn default() -> Self {
        Query {
            text: String::new(),
            name: true,
            description: false,
            site: false,
            case: false,
            whole: false,
        }
    }
}

impl Query {
    /// Is there anything to look for? (An empty field filters nothing.)
    pub fn active(&self) -> bool {
        !self.text.trim().is_empty()
    }

    /// Whether the download matches in one of the chosen fields. Nothing allocated: the list
    /// asks it of every download each frame while a search is on.
    pub fn matches(&self, item: &Item) -> bool {
        let wanted = self.text.trim();
        if wanted.is_empty() {
            return false;
        }
        let hit = |field: &str| found(field, wanted, self.case, self.whole);
        let site = || {
            let ctx = &item.context;
            [
                Some(item.url.as_str()),
                item.url.host_str(),
                ctx.referer.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(hit)
        };
        (self.name && hit(&item.file_name))
            || (self.description && hit(&item.description))
            || (self.site && site())
    }

    /// The next match after `after` in `ids` (wrapping round), else None; the one before it
    /// when `back` (Shift+F3).
    pub fn step(&self, ids: &[u64], items: &[Item], after: Option<u64>, back: bool) -> Option<u64> {
        let n = ids.len();
        if n == 0 {
            return None;
        }
        let at = after.and_then(|a| ids.iter().position(|id| *id == a));
        // The matches once (one pass over the downloads), not a scan of them per id.
        let hits: std::collections::HashSet<u64> = items
            .iter()
            .filter(|i| self.matches(i))
            .map(|i| i.id)
            .collect();
        let hit = |id: &u64| hits.contains(id);
        if back {
            let start = at.unwrap_or(0);
            (1..=n).map(|k| ids[(start + n - k) % n]).find(hit)
        } else {
            let start = at.map_or(0, |p| p + 1);
            (0..n).map(|k| ids[(start + k) % n]).find(hit)
        }
    }
}

/// Is `wanted` in `field`? Case-insensitive unless `case`; as a whole word (not inside a longer
/// one: letters and digits on neither side) when `whole`.
pub fn found(field: &str, wanted: &str, case: bool, whole: bool) -> bool {
    if wanted.is_empty() {
        return false;
    }
    let same = |a: char, b: char| {
        if case {
            a == b
        } else {
            a == b || a.to_lowercase().eq(b.to_lowercase())
        }
    };
    let word = |c: Option<char>| c.is_some_and(char::is_alphanumeric);
    for (start, _) in field.char_indices() {
        let mut rest = field[start..].chars();
        let mut end = start;
        let mut all = true;
        for w in wanted.chars() {
            match rest.next() {
                Some(c) if same(c, w) => end += c.len_utf8(),
                _ => {
                    all = false;
                    break;
                }
            }
        }
        if all
            && (!whole
                || (!word(field[..start].chars().next_back())
                    && !word(field[end..].chars().next())))
        {
            return true;
        }
    }
    false
}

// ---- the title bar's search field and its dropdown -----------------------------------------

use egui::{Rect, Stroke, Ui, pos2, vec2};

use crate::list::{self, ListState};
use crate::winui;

/// The search field's text box (Ctrl+F gives it the keys).
pub fn field_id() -> egui::Id {
    egui::Id::unique("find-field")
}

/// The search field's state beside its query: the dropdown, and Ctrl+F's request.
#[derive(Default)]
pub struct Bar {
    /// The funnel's dropdown is open.
    pub open: bool,
    /// Give the field the keys, its text selected, on the next frame (Ctrl+F).
    pub focus: bool,
    /// The open dropdown's first and last controls, as last drawn: Tab goes round them.
    pub ends: Option<(egui::Id, egui::Id)>,
    /// The field had the keys at the end of the last frame (egui drops them as Escape comes in,
    /// before the field sees it).
    focused: bool,
    /// The field was drawn last frame (the title bar had room for it).
    shown: bool,
}

impl Bar {
    /// Ctrl+F (or F3 with nothing to find): give the field the keys on the next frame, its text
    /// selected. False, and nothing asked, while the field isn't drawn (a title bar too narrow).
    pub fn ask_focus(&mut self) -> bool {
        self.focus = self.shown;
        self.shown
    }

    /// The field isn't drawn this frame: nothing stays asked or open, and the search, which
    /// nothing would show or clear, ends (the list shows every download again). True when the
    /// field had the keys: the caller gives them back to the list.
    pub fn hidden(&mut self, q: &mut Query) -> bool {
        let had = std::mem::take(&mut self.focused);
        self.shown = false;
        self.open = false;
        self.focus = false;
        self.ends = None;
        if !q.text.is_empty() {
            q.text.clear();
        }
        had
    }
}

/// What the field asks of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Enter: select the next match.
    Next,
    /// Shift+Enter: select the match before.
    Previous,
    /// Escape: the text cleared and the field left; the list takes the keys.
    Leave,
}

/// The field's narrowest and widest (English px).
pub const FIELD_MIN: f32 = 140.0;
pub const FIELD_MAX: f32 = 220.0;
/// The funnel's and the clear button's zones at the field's end.
const FUNNEL_W: f32 = 26.0;
const CLEAR_W: f32 = 20.0;
const DROP_W: f32 = 268.0;

/// Are the dropdown's Match in / case / whole word choices other than their defaults? With a
/// column filter on (`list::any_filter`), it lights the funnel.
pub fn options_changed(q: &Query) -> bool {
    let d = Query::default();
    (q.name, q.description, q.site, q.case, q.whole)
        != (d.name, d.description, d.site, d.case, d.whole)
}

/// The search field in English rect `rc`: a magnifier, the text box (its placeholder "Find"
/// and Ctrl+F's key while empty), a clear button while there is text, and the funnel that
/// opens the dropdown. What it asks of the window, if anything.
pub fn field(
    ui: &mut Ui,
    rc: Rect,
    key: &str,
    bar: &mut Bar,
    q: &mut Query,
    list: &mut ListState,
) -> Option<Event> {
    let ctx = ui.ctx().clone();
    let id = field_id();
    let p = ui.painter().clone();
    let c = crate::theme::colors();
    let screen = winui::rect(rc);
    bar.shown = true;
    let had = bar.focused;
    // Escape closes the dropdown first (and leaves the field as it was, with the keys); then it
    // clears the field and leaves it.
    let mut event = None;
    if bar.open && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        bar.open = false;
        if had {
            ctx.memory_mut(|m| m.request_focus(id));
        }
    } else if had && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        // Taken: Options' page under the field (plan 11a) mustn't go back on it too.
        q.text.clear();
        event = Some(Event::Leave);
    }
    if std::mem::take(&mut bar.focus) {
        winui::select_all_in(&ctx, id, q.text.chars().count());
    }
    // Like every text box: the field roles, and the theme's focus look (plan 12 G4).
    winui::field_frame(&p, screen, winui::FieldLook::Editable);
    if had {
        winui::focus_bottom(&p, screen);
    }
    let mid = rc.center().y;
    let icon = Rect::from_min_size(pos2(rc.left() + 9.0, (mid - 7.0).round()), vec2(14.0, 14.0));
    crate::themeicons::paint_in(
        ui,
        &p,
        "gx-search",
        winui::rect(icon),
        c.text_secondary,
        true,
    );
    let funnel = Rect::from_min_max(pos2(rc.right() - FUNNEL_W, rc.top()), rc.max);
    let clear = (!q.text.is_empty()).then(|| {
        Rect::from_min_max(
            pos2(funnel.left() - CLEAR_W, rc.top()),
            pos2(funnel.left(), rc.bottom()),
        )
    });
    let text_end = clear.map_or(funnel.left(), |r| r.left()) - 2.0;
    let text_rc = Rect::from_min_max(
        pos2(icon.right() + 5.0, rc.top()),
        pos2(text_end, rc.bottom()),
    );
    if q.text.is_empty() {
        let baseline = rc.top() + (rc.height() / 2.0 + 4.0).round();
        // i18n: the title bar's search field while it is empty (Ctrl+F)
        let label = crate::i18n::t("Find");
        let drawn = winui::text_font(
            &p,
            winui::segoe(),
            text_rc.left() + 2.0,
            baseline,
            label,
            c.text_secondary,
        );
        let key_x = winui::rect(drawn).right() + 7.0;
        if key_x + 30.0 < text_rc.right() {
            winui::text_font(
                &p,
                egui::FontId::proportional(crate::theme::metrics().font_small),
                key_x,
                baseline,
                key,
                c.menu_shortcut,
            );
        }
    }
    let resp = winui::bare_edit(ui, winui::rect(text_rc), &mut q.text, id);
    // i18n: the title bar's search field, to screen readers: it filters the list as you type
    let name = crate::i18n::t("Find downloads");
    ctx.accesskit_node_builder(id, |n| n.set_label(name));
    // Enter in the field (it had the keys as the frame began; a one-line box may give them up
    // on Enter): the next match (Shift+Enter: the one before), and the field keeps the keys.
    // The press's own modifiers (Shift held at that key, not at the frame's end).
    let enter = ctx.input(|i| {
        i.events.iter().find_map(|e| match e {
            egui::Event::Key {
                key: egui::Key::Enter,
                pressed: true,
                modifiers,
                ..
            } => Some(modifiers.shift),
            _ => None,
        })
    });
    if let Some(shift) = enter.filter(|_| had || resp.has_focus()) {
        ctx.memory_mut(|m| m.request_focus(id));
        event = Some(if shift { Event::Previous } else { Event::Next });
    }
    if resp.changed() {
        ctx.request_repaint();
    }
    if let Some(x) = clear {
        let r = ui.interact(
            winui::rect(x),
            egui::Id::unique("find-clear"),
            winui::pointer_only(),
        );
        // i18n: the button in the search field that empties it
        winui::expose(
            &r,
            egui::Role::Button,
            crate::i18n::t("Clear search"),
            true,
            None,
        );
        let ink = if r.hovered() {
            c.text
        } else {
            c.text_secondary
        };
        let m = winui::rect(x).center();
        for (a, b) in [
            (vec2(-3.5, -3.5), vec2(3.5, 3.5)),
            (vec2(-3.5, 3.5), vec2(3.5, -3.5)),
        ] {
            p.line_segment([m + a, m + b], Stroke::new(1.5, ink));
        }
        if r.clicked() {
            q.text.clear();
            ctx.memory_mut(|m| m.request_focus(id));
        }
    }
    let f = ui.interact(
        winui::rect(funnel),
        egui::Id::unique("find-funnel"),
        winui::pointer_only(),
    );
    let active = list::any_filter(list) || options_changed(q);
    // i18n: the button at the end of the search field: where to look and which downloads to show
    let label = crate::i18n::t("Search options");
    winui::expose(&f, egui::Role::Button, label, true, None);
    ctx.accesskit_node_builder(f.id, |n| n.set_expanded(bar.open));
    if f.hovered() || bar.open {
        let radius = crate::theme::metrics().part_radius;
        p.rect_filled(winui::rect(funnel.shrink(3.0)), radius, c.button_hot);
    }
    let ink = if active {
        c.button_accent
    } else {
        c.text_secondary
    };
    crate::themeicons::paint_in(
        ui,
        &p,
        "gx-filter",
        winui::rect(Rect::from_center_size(funnel.center(), vec2(14.0, 14.0))),
        ink,
        true,
    );
    if active {
        // A dot on the funnel: something narrows the list.
        p.circle_filled(
            winui::rect(funnel).right_top() + vec2(-6.0, 6.0),
            2.5,
            c.button_accent,
        );
    }
    let f = f.on_hover_text(label);
    if f.clicked() {
        bar.open = !bar.open;
    }
    if bar.open {
        dropdown(ui, funnel, bar, q, list);
    } else {
        bar.ends = None;
    }
    bar.focused = ctx.memory(|m| m.has_focus(id));
    event
}

/// Where a chip's word starts: after the check mark's room, so on and off are as wide.
const CHIP_TEXT: f32 = 22.0;

/// How a Status or Source chip is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChipLook {
    face: egui::Color32,
    edge: egui::Color32,
    edge_w: f32,
    ink: egui::Color32,
}

/// On: filled with the selection tint, the accent's border, the text's ink (and a check mark).
/// Off: a plain outline, the field's face (the hover face under the pointer), the buttons'
/// border, the secondary ink.
fn chip_look(on: bool, hot: bool) -> ChipLook {
    let c = crate::theme::colors();
    if on {
        ChipLook {
            face: c.selection,
            edge: c.accent,
            edge_w: 1.5,
            ink: c.text,
        }
    } else {
        ChipLook {
            face: if hot { c.button_hot } else { c.window },
            edge: c.button_border,
            edge_w: 1.0,
            ink: c.text_secondary,
        }
    }
}

/// The funnel's dropdown under it (`funnel`, English), in the main window (an egui Area, no
/// window of its own): Status and Source as toggle chips (the column filters themselves), where
/// to look, match case, whole word, and Clear all. A press outside it closes it.
fn dropdown(ui: &mut Ui, funnel: Rect, bar: &mut Bar, q: &mut Query, list: &mut ListState) {
    let ctx = ui.ctx().clone();
    let font = winui::segoe();
    let p = ui.painter().clone();
    // The chips' rows, wrapped in the dropdown's width: (column, value, English rect from 0, 0).
    let pad = 12.0;
    let mut y = 10.0;
    let mut chips: Vec<(usize, &'static str, Rect)> = Vec::new();
    let mut labels: Vec<(f32, &'static str)> = Vec::new();
    for (col, title) in [(3, "Status"), (list::SOURCE, "Source")] {
        labels.push((y + 13.0, title));
        y += 20.0;
        let mut x = pad;
        for v in list::filter_values(col).unwrap_or(&[]) {
            let w = winui::text_width_font(&p, font.clone(), crate::i18n::t(v)).ceil()
                + CHIP_TEXT
                + 10.0;
            if x + w > DROP_W - pad && x > pad {
                x = pad;
                y += 28.0;
            }
            chips.push((col, v, Rect::from_min_size(pos2(x, y), vec2(w, 24.0))));
            x += w + 6.0;
        }
        y += 34.0;
    }
    let rule_y = y;
    let match_y = y + 10.0;
    let h = match_y + 20.0 + 5.0 * 24.0 + 40.0;
    let size = vec2(DROP_W, h);
    // Under the funnel, its trailing edge at the funnel's.
    let at = pos2(funnel.right() - DROP_W, funnel.bottom() + 6.0);
    let rect = Rect::from_min_size(at, size);
    let shown = winui::rect(rect);
    let funnel_on_screen = winui::rect(funnel);
    if ctx.input(|i| i.pointer.any_pressed())
        && ctx
            .pointer_interact_pos()
            .is_some_and(|pp| !shown.contains(pp) && !funnel_on_screen.contains(pp))
    {
        bar.open = false;
    }
    egui::Area::new(egui::Id::unique("find-dropdown"))
        .order(egui::Order::Foreground)
        .fixed_pos(shown.min)
        .show(&ctx, |ui| {
            let _ = ui.allocate_exact_size(size, egui::Sense::hover());
            let p = ui.painter().clone();
            let c = crate::theme::colors();
            crate::chrome::popup_surface(&p, shown);
            let o = rect.min;
            let at = |r: Rect| winui::rect(r.translate(o.to_vec2()));
            let mut first = None;
            for (baseline, title) in &labels {
                winui::text(
                    &p,
                    o.x + pad,
                    o.y + baseline,
                    crate::i18n::t(title),
                    c.text_secondary,
                );
            }
            for (k, (col, v, r)) in chips.iter().enumerate() {
                let on = list::value_on(list, *col, v);
                let resp = ui.interact(
                    at(*r),
                    egui::Id::unique(("find-chip", k)),
                    egui::Sense::click(),
                );
                winui::expose(
                    &resp,
                    egui::Role::CheckBox,
                    crate::i18n::t(v),
                    true,
                    Some(on),
                );
                first = first.or(Some(resp.id));
                let radius = r.height() / 2.0;
                let look = chip_look(on, resp.hovered());
                p.rect_filled(at(*r), radius, look.face);
                p.rect_stroke(
                    at(*r),
                    radius,
                    Stroke::new(look.edge_w, look.edge),
                    egui::StrokeKind::Inside,
                );
                if on {
                    // A check mark before the word: on isn't told by colour alone.
                    let m = at(*r).left_center() + vec2(12.0, 0.0);
                    p.add(egui::Shape::line(
                        vec![
                            m + vec2(-3.5, 0.0),
                            m + vec2(-1.0, 2.5),
                            m + vec2(3.5, -2.5),
                        ],
                        Stroke::new(crate::theme::metrics().tick_width, look.ink),
                    ));
                }
                // Only from the keyboard, as every control's ring (plan 12 G3).
                if resp.has_focus() {
                    winui::focus_ring_round(&p, at(r.expand(2.0)), radius + 2.0);
                }
                winui::text_font(
                    &p,
                    winui::segoe(),
                    o.x + r.left() + CHIP_TEXT,
                    o.y + r.top() + 16.0,
                    crate::i18n::t(v),
                    look.ink,
                );
                if resp.clicked() {
                    list::set_value(list, *col, v, !on);
                }
            }
            winui::rule(&p, o, pad, rule_y, DROP_W - 2.0 * pad);
            // i18n: the search options' heading: the fields the search looks in
            winui::text(
                &p,
                o.x + pad,
                o.y + match_y + 13.0,
                crate::i18n::t("Match in"),
                c.text_secondary,
            );
            let boxes: [(&mut bool, &str); 5] = [
                // i18n: a search option: look in the file's name
                (&mut q.name, "File name"),
                // i18n: a search option: look in the download's description
                (&mut q.description, "Description"),
                // i18n: a search option: look in the download's address, its site and the page it came from
                (&mut q.site, "Site and address"),
                (&mut q.case, "Match case"),
                // i18n: a search option: only where the text is a word of its own
                (&mut q.whole, "Match whole word only"),
            ];
            for (k, (on, caption)) in boxes.into_iter().enumerate() {
                let yy = o.y + match_y + 22.0 + 24.0 * k as f32 + if k >= 3 { 6.0 } else { 0.0 };
                let check = winui::checkbox(ui, pos2(o.x + pad, yy), on, caption, true);
                first = first.or(Some(check.id));
            }
            let clear = winui::button(
                ui,
                winui::rect(Rect::from_min_size(
                    pos2(o.x + pad, o.y + h - 32.0),
                    vec2(96.0, 23.0),
                )),
                // i18n: the search options' button: no search, no filter, the options as they first were
                "Clear all",
                winui::ButtonKind::Normal,
                false,
                None,
            );
            if clear.clicked() {
                *q = Query::default();
                list.filters = vec![None; list::COLS];
            }
            bar.ends = first.map(|f| (f, clear.id));
        });
}

/// The search field's rect in Graphite's title bar, between the menus' `end` and the window's
/// buttons (`free` px), centred on `mid`: as wide as the room allows ([`FIELD_MIN`] to
/// [`FIELD_MAX`]); None when it doesn't fit.
pub fn caption_place(end: f32, free: f32, mid: f32) -> Option<Rect> {
    let room = free - 16.0;
    (room >= FIELD_MIN).then(|| {
        let fw = room.min(FIELD_MAX);
        let x = (end + (free - fw) / 2.0).round();
        Rect::from_min_size(pos2(x, (mid - 13.0).round()), vec2(fw, 26.0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64, name: &str, description: &str) -> Item {
        let mut i = Item::new(
            id,
            url::Url::parse(&format!("https://files.example.com/{name}")).unwrap(),
            std::path::PathBuf::from(name),
            crate::model::Category::General,
        );
        i.description = description.into();
        i
    }

    #[test]
    fn finds_in_the_chosen_fields_and_wraps_round() {
        let items = vec![
            item(1, "setup.exe", ""),
            item(2, "Movie.mkv", "holiday"),
            item(3, "setup2.exe", ""),
        ];
        let ids = [1, 2, 3];
        let q = Query {
            text: "setup".into(),
            ..Query::default()
        };
        assert_eq!(q.step(&ids, &items, None, false), Some(1));
        assert_eq!(q.step(&ids, &items, Some(1), false), Some(3));
        assert_eq!(q.step(&ids, &items, Some(3), false), Some(1), "wraps round");
        // Shift+F3: the one before, wrapping round the other way.
        assert_eq!(q.step(&ids, &items, Some(3), true), Some(1));
        assert_eq!(
            q.step(&ids, &items, Some(1), true),
            Some(3),
            "wraps round back"
        );
        assert_eq!(q.step(&ids, &items, None, true), Some(3));
        assert_eq!(q.step(&[], &items, None, false), None);
        let q = Query {
            text: "HOLIDAY".into(),
            name: false,
            description: true,
            ..Query::default()
        };
        assert_eq!(q.step(&ids, &items, None, false), Some(2));
        let q = Query { case: true, ..q };
        assert_eq!(q.step(&ids, &items, None, false), None, "match case");
        let q = Query {
            text: "files.example.com".into(),
            name: false,
            site: true,
            whole: true,
            ..Query::default()
        };
        assert!(
            q.step(&ids, &items, None, false).is_some(),
            "the host, whole"
        );
        assert!(
            !Query::default().matches(&items[0]),
            "an empty field matches nothing"
        );
    }

    /// The dropdown's chips and the column titles' filter boxes are one filter: what one sets the
    /// other shows, every value on is no filter, and the funnel's look follows.
    #[test]
    fn the_dropdown_and_the_headers_share_the_filters() {
        let mut st = ListState::default();
        assert!(!list::any_filter(&st));
        // A chip hides Complete: the Status column's filter, as its header's box writes it.
        list::set_value(&mut st, 3, "Complete", false);
        assert_eq!(
            st.filters[3],
            Some(list::Filter::Values(vec![
                "Downloading",
                "Stopped",
                "Waiting",
                "Error"
            ]))
        );
        assert!(list::any_filter(&st) && !list::value_on(&st, 3, "Complete"));
        assert!(list::value_on(&st, 3, "Error"));
        // A header's box keeping Torrent only: the dropdown's Web chip reads it off.
        st.filters[list::SOURCE] = Some(list::Filter::Values(vec!["Torrent"]));
        assert!(!list::value_on(&st, list::SOURCE, "Web"));
        // Back on: no filter left on that column.
        list::set_value(&mut st, 3, "Complete", true);
        assert_eq!(st.filters[3], None);
        list::set_value(&mut st, list::SOURCE, "Web", true);
        assert!(!list::any_filter(&st));
        // The options other than their defaults light the funnel too.
        assert!(!options_changed(&Query::default()));
        assert!(options_changed(&Query {
            case: true,
            ..Query::default()
        }));
        assert!(!options_changed(&Query {
            text: "x".into(),
            ..Query::default()
        }));
    }

    /// One frame of the field in a 1240 × 700 window, with `events`.
    fn frame(
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        bar: &mut Bar,
        q: &mut Query,
        st: &mut ListState,
    ) -> Option<Event> {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1240.0, 700.0))),
            events,
            ..Default::default()
        };
        let mut out = None;
        let mut full = ctx.run_ui(input, |ui| {
            let rc = Rect::from_min_size(pos2(700.0, 5.0), vec2(FIELD_MAX, 26.0));
            out = field(ui, rc, "Ctrl+F", bar, q, st);
        });
        full.textures_delta.clear();
        out
    }

    fn escape() -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }

    /// Escape closes the open dropdown first, keeping the text; then it clears the field and
    /// leaves it (the list takes the keys). Ctrl+F's request gives the field the keys.
    #[test]
    fn escape_closes_the_dropdown_then_clears_the_field() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        let (mut bar, mut st) = (Bar::default(), ListState::default());
        let mut q = Query {
            text: "setup".into(),
            ..Query::default()
        };
        bar.focus = true;
        frame(&ctx, vec![], &mut bar, &mut q, &mut st);
        frame(&ctx, vec![], &mut bar, &mut q, &mut st);
        assert!(
            ctx.memory(|m| m.has_focus(field_id())),
            "Ctrl+F gives it the keys"
        );
        bar.open = true;
        frame(&ctx, vec![], &mut bar, &mut q, &mut st);
        assert_eq!(frame(&ctx, vec![escape()], &mut bar, &mut q, &mut st), None);
        assert!(!bar.open, "the dropdown closed");
        assert_eq!(q.text, "setup", "the text kept");
        assert_eq!(
            frame(&ctx, vec![escape()], &mut bar, &mut q, &mut st),
            Some(Event::Leave)
        );
        assert!(q.text.is_empty(), "the text cleared");
    }

    /// A chip that's on doesn't look like one that's off, hovered or not: another fill, border
    /// and ink.
    #[test]
    fn chips_on_and_off_look_different() {
        for hot in [false, true] {
            let (on, off) = (chip_look(true, hot), chip_look(false, hot));
            assert_ne!(on.face, off.face);
            assert_ne!(on.edge, off.edge);
            assert_ne!(on.ink, off.ink);
            assert!(on.edge_w > off.edge_w);
        }
    }

    /// Audit 69-2: Ctrl+F asks nothing while the field isn't drawn, and a search ends with it.
    #[test]
    fn no_field_no_focus_and_no_search() {
        let mut bar = Bar::default();
        let mut q = Query {
            text: "setup".into(),
            ..Query::default()
        };
        assert!(!bar.ask_focus() && !bar.focus, "never drawn: nothing asked");
        bar.shown = true;
        assert!(bar.ask_focus() && bar.focus);
        bar.open = true;
        bar.hidden(&mut q);
        assert!(!bar.focus && !bar.open && q.text.is_empty());
        assert!(!bar.ask_focus(), "hidden: nothing asked");
        // Audit 69-r2-1: hidden while it had the keys says so (the list takes them back), once.
        bar.focused = true;
        assert!(bar.hidden(&mut q));
        assert!(!bar.hidden(&mut q));
    }

    /// Enter in the field asks for the next match, and the field keeps the keys.
    #[test]
    fn enter_in_the_field_selects_the_next_match() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        let (mut bar, mut st) = (Bar::default(), ListState::default());
        let mut q = Query {
            text: "setup".into(),
            ..Query::default()
        };
        bar.focus = true;
        frame(&ctx, vec![], &mut bar, &mut q, &mut st);
        frame(&ctx, vec![], &mut bar, &mut q, &mut st);
        let enter = egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        assert_eq!(
            frame(&ctx, vec![enter], &mut bar, &mut q, &mut st),
            Some(Event::Next)
        );
        frame(&ctx, vec![], &mut bar, &mut q, &mut st);
        assert!(
            ctx.memory(|m| m.has_focus(field_id())),
            "the field keeps the keys"
        );
        assert_eq!(q.text, "setup");
        // Shift+Enter: the match before.
        let shift_enter = egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        };
        assert_eq!(
            frame(&ctx, vec![shift_enter], &mut bar, &mut q, &mut st),
            Some(Event::Previous)
        );
    }

    #[test]
    fn case_and_whole_words() {
        assert!(found("Setup.EXE", "setup", false, false));
        assert!(!found("Setup.EXE", "setup", true, false));
        assert!(
            found("my setup.exe", "setup", false, true),
            "a word between a space and a dot"
        );
        assert!(
            !found("setup2.exe", "setup", false, true),
            "inside a longer word"
        );
        assert!(!found("presetup.exe", "setup", false, true));
        assert!(found("ÉTÉ.mp3", "été", false, false), "beyond ASCII");
        assert!(found("a", "a", false, true));
        assert!(!found("abc", "", false, false));
        assert!(!found("ab", "abc", false, false));
    }
}
