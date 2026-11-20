//! Add: the toolbar's one window for adding anything (it took Add URL's place). Addresses,
//! magnet links, video pages and .torrent files, one per line, typed, pasted or dropped; each
//! line read by `crate::addkind` and listed with what comes next for it. Add hands every item
//! to the flow it already had (Download File Info, Add torrent, the video window, Batch
//! download: `App::add_items`).
//!
//! Kept from Add URL: the clipboard's links filled in as it opens, "Use authorization" with its
//! login and password (and Options → Site logins shown there, #511), and the addresses go into
//! the Address history as before (the paste bar's drop list and Up / Down step through them).

use std::path::Path;

use egui::{Pos2, Ui, pos2, vec2};
use url::Url;

use crate::addkind::{self, Found, Kind};
use crate::dialogs::{Outcome, window};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, r};

/// The window's width and where its footer starts.
const W: f32 = 560.0;
const BODY: f32 = 410.0;
const FIELD_Y: f32 = 40.0;
const FIELD_H: f32 = 96.0;
const FOUND_Y: f32 = 160.0;
const LIST_Y: f32 = 168.0;
const ROW_H: f32 = 30.0;
/// Rows shown; past them the last row says how many more.
const ROWS: usize = 4;
const DROP_Y: f32 = LIST_Y + ROW_H * ROWS as f32 + 10.0;
const DROP_H: f32 = 44.0;
const AUTH_Y: f32 = DROP_Y + DROP_H + 14.0;

/// What Add (or Later) hands on: the items in order, the login typed under "Use
/// authorization", whether each asks where it goes ("Ask where to save each one") and whether
/// its links wait in a queue (Later).
#[derive(Debug, PartialEq)]
pub struct AddOut {
    pub items: Vec<Kind>,
    pub login: Option<(String, String)>,
    pub ask: bool,
    pub later: bool,
}

#[derive(Default)]
pub struct AddWindow {
    /// The field's text: one address a line.
    pub address: String,
    pub auth: bool,
    pub login: String,
    pub password: String,
    /// "Ask where to save each one" (on as it opens).
    pub ask: bool,
    /// The address Options → Site logins was last looked up for, and the login it gave
    /// (#511): "Use authorization" shows it while the box is left as it was filled.
    looked_up: String,
    filled: Option<(String, String)>,
    /// The clipboard's links, read off the window's thread (#661).
    clip: crate::clipboard::Prefill,
    /// The field had the keys in the last frame: Enter there adds (Shift+Enter a new line).
    field_focused: bool,
    /// The box's id's number: a new box once the clipboard filled it.
    field_gen: u32,
    focused: bool,
    /// The lines read, for the text they were read from (the disk is asked once per edit).
    found: Option<(String, Vec<Found>)>,
    /// Links and files dropped on this window.
    drops: crate::droptarget::Drops,
}

/// The window's title (its drop target is found by it): "Add".
const TITLE: &str = "Add";

impl AddWindow {
    /// Opened with nothing in it.
    pub fn new() -> Self {
        AddWindow {
            ask: true,
            ..Default::default()
        }
    }

    /// Opened from Add (the toolbar, Ctrl+N, the menus): the clipboard's links filled in, read
    /// off the window's thread and put in the box on the frame they come.
    pub fn from_clipboard() -> Self {
        AddWindow {
            clip: crate::clipboard::Prefill::ask(),
            ..Self::new()
        }
    }

    /// Opened with `text` in it (a drop on the main window, the paste bar's several links).
    pub fn with_text(text: &str) -> Self {
        let mut w = Self::new();
        w.append(text);
        w
    }

    /// `text` added after what the box holds, on lines of its own.
    pub fn append(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if !self.address.trim().is_empty() && !self.address.ends_with('\n') {
            self.address.push('\n');
        }
        self.address.push_str(text);
    }

    /// The clipboard's addresses in the box, one a line, while it is still empty (nothing
    /// typed meanwhile); nothing when the clipboard holds none.
    pub(crate) fn fill_from_clipboard(&mut self, clip: &str) {
        if !self.address.is_empty() {
            return;
        }
        let found = addkind::scan(clip, &|_| false);
        let lines: Vec<&str> = found
            .iter()
            .filter(|f| f.kind.is_ok())
            .map(|f| f.text.as_str())
            .collect();
        self.address = lines.join("\n");
        if !self.address.is_empty() {
            // A box of its own, focused again: its caret after the text, as in a window that
            // opened with it (the old box's caret stayed at its start).
            self.field_gen += 1;
            self.focused = false;
        }
    }

    /// The lines read, again only when the text changed.
    fn found(&mut self) -> &[Found] {
        if self.found.as_ref().is_none_or(|(t, _)| *t != self.address) {
            let found = addkind::scan(&self.address, &is_file);
            self.found = Some((self.address.clone(), found));
        }
        self.found.as_ref().map_or(&[], |(_, f)| f.as_slice())
    }

    /// The first link's address (what Site logins is looked up for).
    fn first_link(&mut self) -> Option<Url> {
        self.found().iter().find_map(|f| match &f.kind {
            Ok(Kind::Link(u) | Kind::TorrentUrl(u)) => Some(u.clone()),
            Ok(Kind::Batch(s)) => crate::model::parse_address(&s.replacen('*', "1", 1)).ok(),
            _ => None,
        })
    }

    /// "Use authorization" shows the Site logins entry the first link will use (#511):
    /// `site_login` is that same match (`Settings::site_login`). A login typed or changed in
    /// the box is left alone; one filled from Options follows the address.
    fn follow_site_login(&mut self, site_login: impl Fn(&Url) -> Option<(String, String)>) {
        let first = self.first_link();
        let key = first.as_ref().map_or_else(String::new, Url::to_string);
        if self.looked_up == key {
            return;
        }
        self.looked_up = key;
        let found = first.and_then(|u| site_login(&u));
        if found == self.filled {
            return;
        }
        let as_filled = match &self.filled {
            Some((user, pass)) => self.auth && self.login == *user && self.password == *pass,
            None => !self.auth && self.login.is_empty() && self.password.is_empty(),
        };
        if as_filled {
            self.auth = found.is_some();
            (self.login, self.password) = found.clone().unwrap_or_default();
        }
        self.filled = found;
    }

    /// The login Add hands on: none while the box shows the Site logins entry as it was
    /// filled, so the download keeps following Options, as it does without one.
    pub(crate) fn typed_login(&self) -> Option<(String, String)> {
        let own = (self.login.clone(), self.password.clone());
        (self.auth && self.filled.as_ref() != Some(&own)).then_some(own)
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        site_login: impl Fn(&Url) -> Option<(String, String)>,
    ) -> Outcome<AddOut> {
        if let Some(clip) = self.clip.take() {
            self.fill_from_clipboard(&clip);
        }
        if let Some(path) = winui::picked(ctx, "add-torrent-file") {
            self.append(&path.display().to_string());
        }
        self.follow_site_login(site_login);
        window(
            ctx,
            "add",
            TITLE,
            [W, BODY + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| self.body(ui, o),
        )
    }

    fn body(&mut self, ui: &mut Ui, o: Pos2) -> Outcome<AddOut> {
        let ctx = ui.ctx().clone();
        for dropped in self.drops.poll(&ctx, crate::i18n::t(TITLE)) {
            self.append(&dropped);
        }
        let p = ui.painter().clone();
        let c = crate::theme::colors();
        let valid = self.found().iter().filter(|f| f.kind.is_ok()).count();

        // i18n: Add: the label over its box
        let label = crate::i18n::t("Addresses, magnet links or video pages, one per line");
        winui::fit(
            ui,
            &p,
            o.x + winui::EDGE,
            o.y + 29.0,
            label,
            c.text_secondary,
            o.x + W - winui::EDGE,
            ("add", 1),
        );
        // Enter in the box adds what it found (Add's default), Shift+Enter is a new line; with
        // nothing found yet Enter is a new line too.
        let mut submit = false;
        if self.field_focused
            && valid > 0
            // Only a plain Enter: `consume_key` lets Shift through.
            && ui.input_mut(|i| {
                let plain = i.events.iter().any(|e| {
                    matches!(e, egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers, .. }
                        if modifiers.is_none())
                });
                plain && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
            })
        {
            submit = true;
        }
        let field_rc = r(o, winui::EDGE, FIELD_Y, W - 2.0 * winui::EDGE, FIELD_H);
        let field_id = format!("add-field-{}", self.field_gen);
        let field = winui::multiline_light(ui, field_rc, &mut self.address, &field_id, true);
        winui::expose(&field, egui::Role::MultilineTextInput, label, true, None);
        if !std::mem::replace(&mut self.focused, true) {
            field.request_focus();
        }
        self.field_focused = field.has_focus();

        // "Found N" and the rows.
        // Only the rows shown are copied out of the cached scan (a long paste stays cheap).
        let total = self.found().len();
        let found: Vec<Found> = self.found().iter().take(ROWS).cloned().collect();
        let small = egui::FontId::proportional(crate::theme::metrics().font_small);
        // i18n: Add: over the list of what was recognised; {n} is how many can be added
        let found_text = crate::i18n::fmt(
            crate::i18n::t("Found {n}"),
            &[("n", &crate::i18n::integer(valid as u64))],
        );
        let drawn = winui::text_font(
            &p,
            small.clone(),
            o.x + winui::EDGE,
            o.y + FOUND_Y,
            &found_text,
            c.muted,
        );
        // Read by screen readers (and found by name by the help's shot list).
        winui::readable(ui, drawn, "add-found", &found_text);
        let list = r(
            o,
            winui::EDGE,
            LIST_Y,
            W - 2.0 * winui::EDGE,
            ROW_H * ROWS as f32,
        );
        p.rect_stroke(
            list,
            6.0,
            egui::Stroke::new(1.0, c.menubar_line),
            egui::StrokeKind::Inside,
        );
        if found.is_empty() {
            // i18n: Add: the empty list under the box
            let hint = crate::i18n::t("Type or paste addresses above, or drop them here.");
            let drawn = winui::text(&p, list.left() + 11.0, list.top() + 19.0, hint, c.muted);
            winui::readable(ui, drawn, "add-empty", hint);
        }
        let more = total.saturating_sub(ROWS);
        for (k, f) in found.iter().enumerate() {
            let top = list.top() + ROW_H * k as f32;
            if more > 0 && k + 1 == ROWS {
                // i18n: Add: the list's last row when more lines were found than it shows
                let rest =
                    crate::i18n::tn((more + 1) as u64, "and {n} more line", "and {n} more lines");
                let drawn = winui::text(&p, list.left() + 11.0, top + 19.0, &rest, c.muted);
                winui::readable(ui, drawn, "add-more", &rest);
                break;
            }
            if k > 0 {
                p.hline(
                    list.left() + 1.0..=list.right() - 1.0,
                    top,
                    egui::Stroke::new(1.0, c.menubar_line),
                );
            }
            row(ui, &p, f, list.left(), list.right(), top, k);
        }

        // The drop area and "Open a .torrent file…".
        let drop = r(o, winui::EDGE, DROP_Y, W - 2.0 * winui::EDGE, DROP_H);
        let edge = if self.drops.over() {
            winui::accent()
        } else {
            c.menubar_line
        };
        let (a, b) = (drop.left_top(), drop.right_bottom());
        for side in [
            [a, pos2(b.x, a.y)],
            [pos2(b.x, a.y), b],
            [b, pos2(a.x, b.y)],
            [pos2(a.x, b.y), a],
        ] {
            p.extend(egui::Shape::dashed_line(
                &side,
                egui::Stroke::new(1.0, edge),
                4.0,
                3.0,
            ));
        }
        let lead = winui::rect(drop).left();
        crate::themeicons::paint_in(
            ui,
            &p,
            "gx-drag-drop",
            winui::rect(egui::Rect::from_min_size(
                pos2(lead + 11.0, drop.center().y - 9.0),
                vec2(18.0, 18.0),
            )),
            c.muted,
            true,
        );
        // i18n: Add: its open .torrent button
        let open_label = "Open a .torrent file...";
        let open_w = winui::footer_button_width(&p, open_label, false);
        let open_rc = r(
            o,
            W - winui::EDGE - 8.0 - open_w,
            DROP_Y + (DROP_H - winui::CONTROL_H) / 2.0,
            open_w,
            winui::CONTROL_H,
        );
        // i18n: Add: the drop area's line
        let drop_line = crate::i18n::t("Drop a .torrent file or a text with links here");
        let drawn = winui::fit(
            ui,
            &p,
            lead + 38.0,
            drop.center().y + 5.0,
            drop_line,
            c.muted,
            winui::rect(open_rc).left() - 8.0,
            ("add", 2),
        );
        winui::readable(ui, drawn, "add-drop-line", drop_line);
        if winui::button(ui, open_rc, open_label, ButtonKind::Normal, true, None).clicked() {
            let kind = crate::i18n::t("Torrent files").to_owned();
            let title = crate::i18n::t("Open torrent file").to_owned();
            winui::start_pick(&ctx, "add-torrent-file", move || {
                rfd::FileDialog::new()
                    .add_filter(kind, &["torrent"])
                    .set_title(title)
                    .pick_file()
            });
        }

        // Use authorization: its login and password beside it.
        winui::checkbox(
            ui,
            o + vec2(winui::EDGE, AUTH_Y + 8.0),
            &mut self.auth,
            "Use authorization",
            true,
        );
        let fx = winui::checkbox_caption_end(&p, winui::EDGE, "Use authorization") + 20.0;
        let fw = ((W - winui::EDGE - fx - 12.0) / 2.0).floor();
        let login = winui::edit(
            ui,
            r(o, fx, AUTH_Y, fw, winui::CONTROL_H),
            &mut self.login,
            self.auth,
            false,
        );
        winui::expose(
            &login,
            egui::Role::TextInput,
            crate::i18n::t("Login"),
            self.auth,
            None,
        );
        let pass = winui::edit(
            ui,
            r(o, fx + fw + 12.0, AUTH_Y, fw, winui::CONTROL_H),
            &mut self.password,
            self.auth,
            true,
        );
        winui::expose(
            &pass,
            egui::Role::PasswordInput,
            crate::i18n::t("Password"),
            self.auth,
            None,
        );
        // Their names inside them while empty (no label column in this window).
        for (x, value, name) in [
            (fx, &self.login, "Login"),
            (fx + fw + 12.0, &self.password, "Password"),
        ] {
            if value.is_empty() {
                winui::text(
                    &p,
                    winui::rect(egui::Rect::from_min_size(
                        o + vec2(x + 6.0, AUTH_Y),
                        vec2(1.0, 1.0),
                    ))
                    .left(),
                    o.y + winui::baseline_in(AUTH_Y, winui::CONTROL_H),
                    crate::i18n::t(name),
                    winui::MUTED(),
                );
            }
        }

        // Footer: Ask where to save each one, Later, Add N.
        let add_label = if valid > 0 {
            // i18n: Add: its default button; {n} is how many items it adds
            crate::i18n::fmt(
                crate::i18n::t("Add {n}"),
                &[("n", &crate::i18n::integer(valid as u64))],
            )
        } else {
            crate::i18n::t("Add").to_owned()
        };
        let kind = if valid > 0 {
            ButtonKind::Default
        } else {
            ButtonKind::Disabled
        };
        let later_kind = if valid > 0 {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        let pressed = winui::footer(
            ui,
            o,
            W,
            BODY,
            &mut [
                // i18n: Add: off, links are added in their category's folder without asking
                FooterItem::Check("Ask where to save each one", &mut self.ask, true),
                // i18n: Add: add without starting (links wait in a queue)
                FooterItem::Button(FooterButton::new("Later", later_kind)),
                FooterItem::Button(FooterButton::new(&add_label, kind).enter(true).primary()),
            ],
        );
        if winui::escape_pressed(ui) {
            return Outcome::Closed;
        }
        let later = pressed == Some(1);
        if (submit || pressed == Some(2) || later) && valid > 0 {
            // Every line found, not only the rows shown.
            let items = addkind::valid(self.found());
            return Outcome::Done(AddOut {
                items,
                login: self.typed_login(),
                ask: self.ask,
                later,
            });
        }
        Outcome::Open
    }
}

/// The disk asked whether a .torrent file is there (a share isn't waited for: it shows as
/// there, and Add torrent says if it isn't).
fn is_file(p: &Path) -> bool {
    p.to_string_lossy().starts_with("\\\\") || p.is_file()
}

/// A chip: `label` in a rounded pill, its `icon` before it; `fill` None draws its outline in
/// `ink` (the error's). Returns where it ends.
pub fn chip(
    ui: &Ui,
    p: &egui::Painter,
    (x, mid): (f32, f32),
    label: &str,
    icon: Option<&str>,
    (fill, ink): (Option<egui::Color32>, egui::Color32),
) -> f32 {
    let font = egui::FontId::new(
        crate::theme::metrics().font_small,
        egui::FontFamily::Name("geist-medium".into()),
    );
    let w = winui::text_width_font(p, font.clone(), label).round();
    let icon_w = if icon.is_some() { 17.0 } else { 0.0 };
    let pill = egui::Rect::from_min_size(pos2(x, mid - 10.0), vec2(w + icon_w + 16.0, 20.0));
    let on = winui::rect(pill);
    match fill {
        Some(f) => p.rect_filled(on, 10.0, f),
        None => p.rect_stroke(
            on,
            10.0,
            egui::Stroke::new(1.0, ink),
            egui::StrokeKind::Inside,
        ),
    };
    if let Some(need) = icon {
        let at = egui::Rect::from_min_size(pos2(x + 8.0, mid - 6.0), vec2(12.0, 12.0));
        crate::themeicons::paint_in(ui, p, need, winui::rect(at), ink, true);
    }
    winui::text_font(p, font, x + 8.0 + icon_w, mid + 4.0, label, ink);
    pill.right()
}

/// One row of the list: the chip, the name, and what comes next (a link's category), or why
/// the line isn't added. Exposed to screen readers as one line.
fn row(ui: &mut Ui, p: &egui::Painter, f: &Found, left: f32, right: f32, top: f32, k: usize) {
    let c = crate::theme::colors();
    let mid = top + ROW_H / 2.0;
    let (chip_label, icon, fill, ink, name, next) = match &f.kind {
        Ok(kind) => {
            let name = kind.name(&f.text);
            let next = match kind {
                Kind::Link(_) => crate::model::Category::of(&name).label().into_owned(),
                other => crate::i18n::t(other.next()).to_owned(),
            };
            let fill = if kind.web() {
                c.chip_web
            } else {
                c.chip_torrent
            };
            (
                crate::i18n::t(kind.chip()).to_owned(),
                Some(kind.icon()),
                Some(fill),
                c.chip_text,
                name,
                next,
            )
        }
        Err(why) => (
            // i18n: Add: the chip of a line that is left out
            crate::i18n::t("Skipped").to_owned(),
            None,
            None,
            c.error,
            f.text.clone(),
            crate::i18n::t(why).to_owned(),
        ),
    };
    let end = chip(ui, p, (left + 10.0, mid), &chip_label, icon, (fill, ink));
    let font = winui::tahoma();
    let next_w = winui::text_width_font(p, font.clone(), &next).min((right - left) * 0.4);
    let next_shown = winui::ellipsize(p, &font, &next, next_w);
    let next_ink = if f.kind.is_ok() { c.muted } else { c.error };
    winui::text_font(
        p,
        font.clone(),
        right - 11.0 - next_w,
        mid + 4.0,
        &next_shown,
        next_ink,
    );
    let room = (right - 11.0 - next_w - 12.0 - (end + 10.0)).max(0.0);
    let shown = winui::ellipsize(p, &font, &crate::i18n::isolate(&name), room);
    winui::text_font(
        p,
        font,
        end + 10.0,
        mid + 4.0,
        &shown,
        crate::theme::colors().text,
    );
    let resp = ui.interact(
        egui::Rect::from_min_max(pos2(left, top), pos2(right, top + ROW_H)),
        egui::Id::unique(("add-row", k)),
        egui::Sense::hover(),
    );
    // i18n: Add: a found row to screen readers: {kind} is "Link", "Magnet"…, {name} the line, {next} what happens to it
    let said = crate::i18n::t("{kind}: {name}, {next}");
    let said = crate::i18n::fmt(
        said,
        &[
            ("kind", &chip_label),
            ("name", &crate::i18n::isolate(&name)),
            ("next", &next),
        ],
    );
    // Not `winui::expose`: it reads "&" as an access key's mark, and an address or a file name
    // ("watch?v=x&list=y", "Tom & Jerry.mp4") keeps every "&".
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ListItem, true, &said));
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "0123456789abcdef0123456789abcdef01234567";

    /// Frames of the window with `events` in the last one, as `App` handles its outcome.
    fn frames(dlg: &mut AddWindow, events: Vec<Vec<egui::Event>>) -> Option<Outcome<AddOut>> {
        let ctx = egui::Context::default();
        let mut last = None;
        for ev in events {
            // The keyboard's state as the key events say (Shift held for Shift+Enter).
            let modifiers = ev
                .iter()
                .find_map(|e| match e {
                    egui::Event::Key { modifiers, .. } => Some(*modifiers),
                    _ => None,
                })
                .unwrap_or_default();
            let mut events = vec![egui::Event::ModifiersChanged(modifiers)];
            events.extend(ev);
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                last = Some(dlg.show(ui.ctx(), |_| None));
            });
            out.textures_delta.clear();
            if matches!(last, Some(Outcome::Done(_) | Outcome::Closed)) {
                break;
            }
        }
        last
    }

    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    #[test]
    fn enter_adds_every_item_with_ask_on() {
        let m = format!("magnet:?xt=urn:btih:{HASH}&dn=a.iso");
        let mut dlg = AddWindow::with_text(&format!(
            "https://a.example/f.zip\n{m}\nhttps://www.youtube.com/watch?v=x\nnotes"
        ));
        let out = frames(
            &mut dlg,
            vec![
                vec![],
                vec![],
                vec![key(egui::Key::Enter, Default::default())],
            ],
        );
        let Some(Outcome::Done(out)) = out else {
            panic!("Enter adds");
        };
        assert!(out.ask && !out.later && out.login.is_none());
        assert_eq!(out.items.len(), 3, "the junk line is left out");
        assert!(matches!(out.items[0], Kind::Link(_)));
        assert!(matches!(out.items[1], Kind::Magnet(_)));
        assert!(matches!(out.items[2], Kind::Video(_)));
    }

    #[test]
    fn escape_closes_and_enter_with_nothing_found_adds_nothing() {
        let mut dlg = AddWindow::with_text("just words");
        let out = frames(
            &mut dlg,
            vec![
                vec![],
                vec![key(egui::Key::Enter, Default::default())],
                vec![],
            ],
        );
        assert!(matches!(out, Some(Outcome::Open)), "nothing to add");
        let out = frames(
            &mut dlg,
            vec![vec![], vec![key(egui::Key::Escape, Default::default())]],
        );
        assert!(matches!(out, Some(Outcome::Closed)));
    }

    #[test]
    fn shift_enter_is_a_new_line() {
        let mut dlg = AddWindow::with_text("https://a.example/f.zip");
        let out = frames(
            &mut dlg,
            vec![
                vec![],
                vec![],
                vec![key(egui::Key::Enter, egui::Modifiers::SHIFT)],
            ],
        );
        assert!(matches!(out, Some(Outcome::Open)));
        assert!(dlg.address.contains('\n'), "{:?}", dlg.address);
    }

    #[test]
    fn the_clipboard_fills_its_links_only() {
        let fill = |clip: &str| {
            let mut d = AddWindow::new();
            d.fill_from_clipboard(clip);
            d.address
        };
        assert_eq!(
            fill(" https://a.example/f.zip \n"),
            "https://a.example/f.zip"
        );
        assert_eq!(fill("magnet:?xt=urn:btih:ab"), "magnet:?xt=urn:btih:ab");
        assert_eq!(fill("some words"), "");
        assert_eq!(fill("mailto:a@b.example"), "");
        assert_eq!(
            fill("see https://a.example/1.zip and https://b.example/2.zip"),
            "https://a.example/1.zip\nhttps://b.example/2.zip"
        );
        // Never over what was typed.
        let mut d = AddWindow::with_text("https://typed.example/b.zip");
        d.fill_from_clipboard("https://a.example/f.zip");
        assert_eq!(d.address, "https://typed.example/b.zip");
    }

    /// #661, kept from Add URL: the window asks for the clipboard as it opens and draws its
    /// frames without waiting for the answer, even from a clipboard owner that never answers;
    /// the link fills the box on the frame it comes, the caret after it, never a box typed in.
    #[test]
    fn it_never_waits_for_the_clipboard() {
        use std::time::{Duration, Instant};
        let (release, stuck) = std::sync::mpsc::channel::<()>();
        let stuck = std::sync::Mutex::new(stuck);
        let reader: &'static crate::clipboard::Reader =
            Box::leak(Box::new(crate::clipboard::Reader::new(
                move || {
                    let _ = stuck.lock().unwrap().recv();
                    Some(" https://late.example/a.zip\n".to_owned())
                },
                || {},
            )));
        let ctx = egui::Context::default();
        let frame = |dlg: &mut AddWindow, events: Vec<egui::Event>| {
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let _ = dlg.show(ui.ctx(), |_| None);
            });
            out.textures_delta.clear();
        };
        let mut dlg = AddWindow {
            clip: crate::clipboard::Prefill::ask_of(reader),
            ..AddWindow::new()
        };
        let mut typed = AddWindow {
            clip: crate::clipboard::Prefill::ask_of(reader),
            ..AddWindow::new()
        };
        let started = Instant::now();
        for _ in 0..10 {
            frame(&mut dlg, vec![]);
        }
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(dlg.address.is_empty(), "nothing yet");
        typed.address = "https://typed.example/b.zip".into();
        drop(release);
        let until = Instant::now() + Duration::from_secs(5);
        while dlg.address.is_empty() && Instant::now() < until {
            frame(&mut dlg, vec![]);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(dlg.address, "https://late.example/a.zip");
        frame(&mut dlg, vec![]);
        frame(&mut dlg, vec![egui::Event::Text("x".into())]);
        assert_eq!(dlg.address, "https://late.example/a.zipx");
        while typed.clip.pending() && Instant::now() < until {
            frame(&mut typed, vec![]);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(typed.address, "https://typed.example/b.zip", "typed: kept");
    }

    /// #511, kept from Add URL: the first link's Site logins entry shows in "Use
    /// authorization", follows the address while left as filled, and a typed login stays.
    #[test]
    fn it_shows_the_sites_logins_entry_for_its_first_link() {
        const PASS: &str = "dummy-pass";
        let settings = crate::settings::Settings {
            site_logins: vec![(
                "files.example.com/private".into(),
                "demo".into(),
                PASS.into(),
            )],
            ..Default::default()
        };
        let mut dlg = AddWindow::with_text("https://files.example.com/private/x.zip");
        let look = |d: &mut AddWindow| d.follow_site_login(|u| settings.site_login(u));
        look(&mut dlg);
        assert!(dlg.auth && dlg.login == "demo" && dlg.password == PASS);
        assert!(dlg.typed_login().is_none(), "the download follows Options");
        dlg.address = "https://files.example.com/public/x.zip".into();
        look(&mut dlg);
        assert!(!dlg.auth && dlg.login.is_empty());
        dlg.address = "https://files.example.com/private/y.zip".into();
        look(&mut dlg);
        dlg.login = "someone".into();
        dlg.address = "https://files.example.com/public/y.zip".into();
        look(&mut dlg);
        assert_eq!(dlg.typed_login().map(|l| l.0).as_deref(), Some("someone"));
    }

    /// Screen readers find the box, the rows and the buttons by their (i18n) names.
    #[test]
    fn its_parts_have_names() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut dlg = AddWindow::with_text(
            "https://a.example/f.zip\nhttps://www.youtube.com/watch?v=x&list=y",
        );
        let mut names = Vec::new();
        for _ in 0..3 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                let _ = dlg.show(ui.ctx(), |_| None);
            });
            out.textures_delta.clear();
            if let Some(update) = out.platform_output.accesskit_update {
                // A label's text is its value (egui's Label role), a control's its label.
                names = update
                    .nodes
                    .iter()
                    .filter_map(|(_, n)| n.label().or(n.value()).map(str::to_owned))
                    .collect();
            }
        }
        for want in [
            "Addresses, magnet links or video pages, one per line",
            "Add 2",
            "Later",
            "Open a .torrent file...",
            "Ask where to save each one",
            // Painted lines read too (PR 76's audit; the help's shot finds "Found" by name).
            "Found 2",
            "Drop a .torrent file or a text with links here",
        ] {
            assert!(names.iter().any(|n| n == want), "{want} in {names:?}");
        }
        assert!(names.iter().any(|n| n.starts_with("Link: ")), "{names:?}");
        // A row's address keeps its "&" (no access-key reading of it).
        assert!(
            names
                .iter()
                .any(|n| n.starts_with("Video: ") && n.contains("v=x&list=y")),
            "{names:?}"
        );
    }
}
