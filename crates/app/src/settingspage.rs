//! Options in the main window (plan 11a): the page that takes the panes' place under the
//! toolbar and the paste bar, the status strip staying below it. A header with "← Downloads"
//! (Back, also Escape and Alt+Left) over Options' side list, the chosen page right of it in its
//! own English layout (scrolled where the window is smaller), and a save bar under the page:
//! Help, then Discard and Save, greyed while nothing differs. Back with changes asks first (D3).
//! The page's small dialogs (New login, DNS settings, Pair a device...) stay windows over the
//! main window (D5); the app keeps the settings, merges a Save and applies it (app.rs).

use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::dialogs::{Confirm, Outcome};
use crate::options::{OptionsDialog, SIDE_W};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem};

/// The band over the side list that holds "← Downloads".
pub const HEADER_H: f32 = 40.0;

/// What the page asks of the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageEvent {
    /// Save: merge the draft into the settings and apply it; the page stays, on a fresh draft.
    Save,
    /// Discard: drop the draft (undo what it did live); the page stays, on a fresh draft.
    Discard,
    /// Back to the downloads, the draft dropped (it had no changes, or Yes was answered).
    Close,
}

pub struct SettingsPage {
    pub options: OptionsDialog,
    /// Back with unsaved changes: "Discard your changes to Options?" (D3).
    ask: Option<Confirm>,
}

impl SettingsPage {
    pub fn new(options: OptionsDialog) -> Self {
        SettingsPage { options, ask: None }
    }

    /// A small dialog over the page waits for an answer: the page's buttons and keys wait.
    pub fn modal(&self) -> bool {
        self.options.modal() || self.ask.is_some()
    }

    /// Whether Save would change anything.
    pub fn dirty(&self) -> bool {
        self.options.dirty()
    }

    /// Back, as "← Downloads", Escape and Alt+Left ask for it: closes at once without changes,
    /// else asks first. What the app is to do now, if anything.
    pub fn back(&mut self) -> Option<PageEvent> {
        if self.modal() {
            return None;
        }
        if self.dirty() {
            self.ask = Some(Confirm::new(
                // i18n: the title of the question asked when going back from Options with changes
                crate::i18n::mark("Options"),
                // i18n: Options in the main window: Back (or Escape) with changes not saved yet
                crate::i18n::t("Discard your changes to Options?").to_owned(),
            ));
            None
        } else {
            Some(PageEvent::Close)
        }
    }

    /// The page in `area` (the panes' place in the main window). `keys`: the window's keys are
    /// the page's (no menu or popup open, no text field of the bars above focused): Enter
    /// saves, Escape and Alt+Left go back. `typing`: a text box on the page has the keys, so
    /// Alt+Left is its own (a word back). What the app is to do, if anything.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        area: Rect,
        keys: bool,
        typing: bool,
        downloads: &std::path::Path,
    ) -> Option<PageEvent> {
        crate::crash::window("options");
        let c = crate::theme::colors();
        let o = area.min;
        let _surface = winui::Surface::enter(area.min.x, area.max.x);
        // For screen readers and the screenshots' crop: one pane, "Options", holding the rest.
        let pane = ui.interact(area, egui::Id::unique("settings-page"), Sense::hover());
        // i18n: Options in the main window, to screen readers: the page that holds its sections
        winui::expose(
            &pane,
            egui::Role::Pane,
            crate::i18n::t("Options"),
            true,
            None,
        );
        let mut inner = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(area)
                .accessibility_parent(pane.id),
        );
        let ui = &mut inner;
        ui.set_clip_rect(area.intersect(ui.clip_rect()));
        let p = ui.painter().clone();
        p.rect_filled(area, 0.0, winui::LIGHT_BG());
        let modal = self.modal();

        // The side list under the header, scrolled where the window is too short for it.
        let list_top = o.y + HEADER_H;
        let list_room = (area.max.y - list_top).max(0.0);
        let rows_h = OptionsDialog::pages_height();
        let ppp = ui.ctx().pixels_per_point();
        let list_scrolls = ((rows_h - list_room) * ppp).round() > 1.0;
        let side_w = SIDE_W
            + 1.0
            + if list_scrolls {
                crate::scrollbar::THICK
            } else {
                0.0
            };
        p.rect_filled(
            Rect::from_min_max(o, pos2(o.x + side_w, area.max.y)),
            0.0,
            c.panel,
        );
        let view = Rect::from_min_max(
            pos2(o.x, list_top),
            pos2(o.x + side_w, area.max.y.max(list_top)),
        );
        let list_h = if list_scrolls { rows_h } else { list_room };
        let mut side = ui.new_child(egui::UiBuilder::new().max_rect(view));
        side.set_clip_rect(view.intersect(ui.clip_rect()));
        crate::scale::pane(
            &mut side,
            egui::Id::unique("settings-pages"),
            vec2(SIDE_W + 1.0, list_h),
            |ui, at| self.options.pages(ui, at, list_h),
        );

        // The header: "← Downloads" over the list, the list's border down its side: right of
        // its scroll bar while it scrolls (the list's own border, left of the bar, hidden).
        let border = o.x + side_w - 1.0;
        if list_scrolls {
            winui::fill(
                &p,
                o.x + SIDE_W,
                list_top,
                o.x + SIDE_W,
                area.max.y - 1.0,
                c.panel,
            );
        }
        winui::fill(
            &p,
            border,
            o.y,
            border,
            if list_scrolls {
                area.max.y - 1.0
            } else {
                list_top - 1.0
            },
            c.menubar_line,
        );
        let back_rc = winui::r(o, 8.0, 5.0, SIDE_W - 16.0, winui::CONTROL_H);
        let back = winui::button_icon(
            ui,
            back_rc,
            Some("gx-arrow-left"),
            // i18n: Options in the main window: the button that goes back to the downloads list
            "Downloads",
            ButtonKind::Ghost,
            false,
            None,
        );
        // i18n: Options in the main window, to screen readers: the "← Downloads" button
        let name = crate::i18n::t("Back to downloads");
        ui.ctx()
            .accesskit_node_builder(back.id, |n| n.set_label(name));

        // The section column: the page, then the save bar under it.
        let col = o.x + side_w;
        let col_w = (area.max.x - col).max(0.0);
        let foot = (area.height() - winui::FOOTER_H).max(0.0);
        let page_view = Rect::from_min_max(pos2(col, o.y), pos2(area.max.x, o.y + foot));
        self.options.page_pane(ui, page_view, downloads);

        let dirty = self.dirty();
        let state = if dirty {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        let save_kind = if dirty {
            ButtonKind::Default
        } else {
            ButtonKind::Disabled
        };
        let fo = pos2(col, o.y);
        let pressed = winui::footer(
            ui,
            fo,
            col_w,
            foot,
            // Tab order: Save, Discard, then Help.
            &mut [
                FooterItem::Button(
                    // i18n: Options in the main window: keep the changes made on its pages
                    FooterButton::new("Save", save_kind)
                        .primary()
                        .enter(keys && !modal),
                ),
                // i18n: Options in the main window: drop the changes made on its pages
                FooterItem::Button(FooterButton::new("Discard", state)),
                // i18n: noun: button that opens the help
                FooterItem::Button(FooterButton::new("Help", ButtonKind::Ghost).leading()),
            ],
        );
        if dirty {
            // After Help, where the trailing buttons leave room.
            let help_w = winui::footer_button_width(&p, "Help", false);
            let x = col + winui::EDGE + help_w + 12.0;
            let end = col
                + winui::footer_trailing_left(
                    col_w,
                    &[
                        winui::footer_button_width(&p, "Discard", false),
                        winui::footer_button_width(&p, "Save", false),
                    ],
                );
            // i18n: Options in the main window: the save bar's note while a page has changes
            let note = crate::i18n::t("Unsaved changes");
            let shown = winui::ellipsize(&p, &winui::tahoma(), note, end - x);
            if !shown.is_empty() && end > x {
                winui::text(
                    &p,
                    x,
                    o.y + foot + winui::baseline_in(1.0, winui::FOOTER_H - 1.0),
                    &shown,
                    c.muted,
                );
            }
        }
        if pressed == Some(2) {
            crate::help::open_page("options.md");
        }
        let back_key = keys
            && ui.input_mut(|i| {
                i.key_pressed(egui::Key::Escape)
                    || (!typing && i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowLeft))
            });
        if modal {
            None
        } else if back.clicked() || back_key {
            self.back()
        } else if pressed == Some(0) && dirty {
            Some(PageEvent::Save)
        } else if pressed == Some(1) && dirty {
            Some(PageEvent::Discard)
        } else {
            None
        }
    }

    /// The windows over the page (a page's small dialogs, the discard question): every frame
    /// the page is open, from the app's `dialogs`. `Close` once Yes drops the changes.
    pub fn overlays(
        &mut self,
        ctx: &egui::Context,
        downloads: &std::path::Path,
    ) -> Option<PageEvent> {
        self.options.overlays(ctx, downloads);
        let d = self.ask.as_mut()?;
        match d.show(ctx) {
            Outcome::Open => None,
            // No: the page stays, changes and all.
            Outcome::Closed => {
                self.ask = None;
                None
            }
            Outcome::Done(()) => {
                self.ask = None;
                Some(PageEvent::Close)
            }
        }
    }

    /// The page shown (Options' page numbers), kept by a fresh draft after Save or Discard.
    pub fn page(&self) -> usize {
        self.options.page()
    }
}

/// The page's area in the smallest main window (600 x 300, plan 11), its paste bar shown: the
/// panes' place in Graphite's layout.
#[cfg(test)]
fn min_area() -> Rect {
    let layout = crate::chrome::Layout::GRAPHITE.with_panes(true, None);
    Rect::from_min_max(
        pos2(0.0, layout.panes_top()),
        pos2(600.0, layout.strip_top(300.0)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::PAGE_SIZE;
    use crate::settings::Settings;
    use egui::Pos2;
    use egui::accesskit::{NodeId, Role, TreeUpdate};

    fn input(size: egui::Vec2, events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        }
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

    fn button(at: Pos2, down: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(at),
            egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: down,
                modifiers: Default::default(),
            },
        ]
    }

    fn page(s: &Settings) -> SettingsPage {
        SettingsPage::new(OptionsDialog::new(s, &std::env::temp_dir(), vec![]))
    }

    /// One pass of the page in `area`: what it asked for, and the AccessKit tree.
    fn pass(
        ctx: &egui::Context,
        d: &mut SettingsPage,
        area: Rect,
        events: Vec<egui::Event>,
    ) -> (Option<PageEvent>, TreeUpdate) {
        let dir = std::env::temp_dir();
        let mut ev = None;
        let mut out = ctx.run_ui(input(area.max.to_vec2(), events), |ui| {
            ev = d.show(ui, area, true, false, &dir);
        });
        out.textures_delta.clear();
        (
            ev,
            out.platform_output
                .accesskit_update
                .expect("AccessKit is on"),
        )
    }

    fn bounds(n: &egui::accesskit::Node) -> Option<Rect> {
        n.bounds().map(|b| {
            Rect::from_min_max(
                pos2(b.x0 as f32, b.y0 as f32),
                pos2(b.x1 as f32, b.y1 as f32),
            )
        })
    }

    fn find<'a>(
        tree: &'a TreeUpdate,
        role: Role,
        label: &str,
    ) -> Option<(NodeId, &'a egui::accesskit::Node)> {
        tree.nodes
            .iter()
            .find(|(_, n)| n.role() == role && n.label() == Some(label))
            .map(|(id, n)| (*id, n))
    }

    fn click(ctx: &egui::Context, d: &mut SettingsPage, area: Rect, at: Pos2) -> Option<PageEvent> {
        pass(ctx, d, area, vec![egui::Event::PointerMoved(at)]);
        pass(ctx, d, area, button(at, true));
        pass(ctx, d, area, button(at, false)).0
    }

    /// Plan 11a: a page frame has the "Options" pane holding the "Pages" list and the Back
    /// button, and the save bar's buttons; nothing of the downloads list or tree.
    #[test]
    fn the_page_is_one_pane_with_its_list_back_and_save_bar() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = Rect::from_min_size(pos2(0.0, 100.0), vec2(1080.0, 400.0));
        let mut d = page(&Settings::default());
        pass(&ctx, &mut d, area, vec![]);
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        let nodes: std::collections::HashMap<NodeId, &egui::accesskit::Node> =
            tree.nodes.iter().map(|(id, n)| (*id, n)).collect();
        let (pane, _) = find(&tree, Role::Pane, "Options").expect("the Options pane");
        let mut under = vec![pane];
        let mut i = 0;
        while i < under.len() {
            if let Some(n) = nodes.get(&under[i]) {
                under.extend_from_slice(n.children());
            }
            i += 1;
        }
        for (role, label) in [
            (Role::List, "Pages"),
            (Role::Button, "Back to downloads"),
            (Role::Button, "Save"),
            (Role::Button, "Discard"),
            (Role::Button, "Help"),
        ] {
            let (id, _) = find(&tree, role, label).unwrap_or_else(|| panic!("no {label}"));
            assert!(under.contains(&id), "{label} under the pane");
        }
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, n)| matches!(n.role(), Role::Tree | Role::Table | Role::Grid)),
            "no list or tree"
        );
        // Nothing changed: Save and Discard greyed.
        for label in ["Save", "Discard"] {
            assert!(
                find(&tree, Role::Button, label).unwrap().1.is_disabled(),
                "{label}"
            );
        }
    }

    /// Plan 11a (replaces options_fits_a_short_screen): in a 600 x 300 main window the page's
    /// area is short and narrow; the side list and the page scroll in it, their parts drawn
    /// inside the area, the save bar whole at its bottom.
    #[test]
    fn the_page_scrolls_in_a_small_window() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = min_area();
        let mut d = page(&Settings::default());
        pass(&ctx, &mut d, area, vec![]);
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        for label in ["Save", "Discard", "Help", "Back to downloads"] {
            let rc = bounds(find(&tree, Role::Button, label).expect(label).1).unwrap();
            assert!(area.expand(0.5).contains_rect(rc), "{label}: {rc:?}");
        }
        let save = bounds(find(&tree, Role::Button, "Save").unwrap().1).unwrap();
        assert!(save.bottom() <= area.bottom() && save.top() >= area.bottom() - winui::FOOTER_H);
        // Both scroll: the list's last row and the page's lowest controls lie beyond what is
        // shown of them (the area's bottom, the save bar's top), and the wheel over the page
        // moves its controls up.
        let rows: Vec<Rect> = tree
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::ListItem)
            .filter_map(|(_, n)| bounds(n))
            .collect();
        assert_eq!(rows.len(), 12, "every page's row");
        let last = rows.iter().map(|r| r.bottom()).fold(f32::MIN, f32::max);
        assert!(
            last > area.bottom(),
            "the list's last row below the area: {last}"
        );
        assert!(PAGE_SIZE.x > area.width() - SIDE_W - 1.0);
        let lowest = |tree: &TreeUpdate| {
            tree.nodes
                .iter()
                .filter(|(_, n)| {
                    matches!(n.role(), Role::CheckBox | Role::TextInput | Role::ComboBox)
                })
                .filter_map(|(_, n)| bounds(n))
                .map(|r| r.bottom())
                .fold(f32::MIN, f32::max)
        };
        let before = lowest(&tree);
        assert!(
            before > area.bottom() - winui::FOOTER_H,
            "the page's lowest control under the save bar's top: {before}"
        );
        let over = pos2(area.right() - 60.0, area.top() + 30.0);
        pass(&ctx, &mut d, area, vec![egui::Event::PointerMoved(over)]);
        let wheel = egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: vec2(0.0, -3.0),
            modifiers: Default::default(),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Wheel,
        };
        pass(&ctx, &mut d, area, vec![wheel]);
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        assert!(lowest(&tree) < before, "the wheel scrolled the page");
    }

    /// Plan 11a (D2, D3): a change makes Save and Discard live; Save and Discard say so; Back
    /// with changes asks and stays, Back without changes closes at once (Escape, Alt+Left and
    /// "← Downloads" alike).
    #[test]
    fn back_with_changes_asks_back_without_changes_closes() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = Rect::from_min_size(pos2(0.0, 100.0), vec2(1080.0, 600.0));
        let s = Settings::default();
        // Without changes.
        for k in [
            key(egui::Key::Escape, egui::Modifiers::NONE),
            key(egui::Key::ArrowLeft, egui::Modifiers::ALT),
        ] {
            let mut d = page(&s);
            pass(&ctx, &mut d, area, vec![]);
            assert_eq!(pass(&ctx, &mut d, area, vec![k]).0, Some(PageEvent::Close));
        }
        let mut d = page(&s);
        pass(&ctx, &mut d, area, vec![]);
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        let back = bounds(find(&tree, Role::Button, "Back to downloads").unwrap().1).unwrap();
        assert_eq!(
            click(&ctx, &mut d, area, back.center()),
            Some(PageEvent::Close)
        );
        // With a change: Back asks (nothing to do yet), the question is open, the page modal.
        assert!(!page(&s).dirty(), "a draft as opened is clean");
        let mut d = changed(&s);
        assert!(d.dirty());
        pass(&ctx, &mut d, area, vec![]);
        let (ev, _) = pass(
            &ctx,
            &mut d,
            area,
            vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert_eq!(ev, None, "asked first");
        assert!(d.ask.is_some() && d.modal());
        // Answered No: the page stays with its changes.
        d.ask = None;
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        for label in ["Save", "Discard"] {
            assert!(
                !find(&tree, Role::Button, label).unwrap().1.is_disabled(),
                "{label}"
            );
        }
        let save = bounds(find(&tree, Role::Button, "Save").unwrap().1).unwrap();
        assert_eq!(
            click(&ctx, &mut d, area, save.center()),
            Some(PageEvent::Save)
        );
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        let discard = bounds(find(&tree, Role::Button, "Discard").unwrap().1).unwrap();
        assert_eq!(
            click(&ctx, &mut d, area, discard.center()),
            Some(PageEvent::Discard)
        );
        // Enter is Save, but not while the window's keys are another's (`keys` false). The
        // click left Discard focused, whose own Enter would press it: nothing focused now.
        ctx.memory_mut(|m| {
            if let Some(f) = m.focused() {
                m.surrender_focus(f);
            }
        });
        let dir = std::env::temp_dir();
        let mut ev = None;
        let mut out = ctx.run_ui(
            input(
                area.max.to_vec2(),
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            ),
            |ui| ev = d.show(ui, area, false, false, &dir),
        );
        out.textures_delta.clear();
        assert_eq!(ev, None);
        assert_eq!(
            pass(
                &ctx,
                &mut d,
                area,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)]
            )
            .0,
            Some(PageEvent::Save)
        );
    }

    /// A page whose draft differs from what it opened with (General's "Launch on startup"
    /// flipped, as its check box does).
    fn changed(s: &Settings) -> SettingsPage {
        let mut d = page(s);
        let mut t = d.options.result();
        t.launch_on_startup = !t.launch_on_startup;
        d.options.set_draft(t);
        d
    }
}
