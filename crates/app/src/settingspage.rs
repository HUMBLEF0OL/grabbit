//! Options in the main window (plan 11a): the page that takes the panes' place under the
//! toolbar and the paste bar, the status strip staying below it. A header with "← Downloads"
//! (Back, also Escape and Alt+Left) over Options' side list, the chosen page right of it in its
//! own English layout (scrolled where the window is smaller), and a save bar under the page:
//! Help, then Discard and Save, greyed while nothing differs. Back with changes asks first (D3).
//! The page's small dialogs (New login, DNS settings, Pair a device...) stay windows over the
//! main window (D5); the app keeps the settings, merges a Save and applies it (app.rs).

//!
//! Under Options' pages, after "Tools", the windows that moved into the page (plan 11b): the
//! Scheduler (11b1), Speed Limiter settings, Torrent settings and the browser extension (11b2),
//! each draft made on its first visit and saved by the same Save, in that order, before
//! Options' changes (the extension has nothing to save).

use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::chrome::ShellIcons;
use crate::dialogs::{Confirm, Outcome, SpeedSettings};
use crate::extension::Extension;
use crate::model::Item;
use crate::options::{
    EXTENSION_PAGE, OptionsDialog, SCHEDULER_PAGE, SIDE_W, SPEED_PAGE, TORRENTS_PAGE,
};
use crate::queue::Queue;
use crate::scheduler::{Action, Scheduler};
use crate::settings::Settings;
use crate::torrentdlg::TorrentSettings;
use crate::winui::{self, ButtonKind, FooterButton, FooterItem};

/// The band over the side list that holds "← Downloads".
pub const HEADER_H: f32 = 40.0;

/// Where a moved window's layout starts in its section: under the section's heading (its tile
/// 16 from the top, 40 tall), as Options' pages' controls start under theirs.
const SECTION_TOP: f32 = 56.0;

/// What the sections that were windows need from the app every frame they are drawn.
pub struct Tools<'a> {
    pub icons: &'a mut ShellIcons,
    pub items: &'a [Item],
    /// The app's queues as they are now (what runs, their downloads).
    pub queues: &'a [Queue],
    pub settings: &'a Settings,
    /// Scheduler → "Wake the computer to run this queue": why it could not be set, if so.
    pub wake_error: Option<&'a str>,
    /// What the Scheduler's Start now and Stop ask of the app, at once.
    pub actions: &'a mut Vec<Action>,
}

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
    /// The Scheduler's section (plan 11b1): its draft, made on its first visit.
    pub scheduler: Option<Scheduler>,
    /// The Speed Limiter's and Torrents' sections (plan 11b2): their drafts, made on their
    /// first visit.
    pub speed: Option<SpeedSettings>,
    pub torrents: Option<TorrentSettings>,
    /// The browser extension's section (11b2): nothing to save, its browsers read on the
    /// first visit.
    extension: Option<Extension>,
    /// Back with unsaved changes: "Discard your changes to Options?" (D3).
    ask: Option<Confirm>,
    /// The page whose row the side list last brought into view (a section opened by a command,
    /// `/show scheduler`, is shown in the list too, in a window too short for every row).
    listed: Option<usize>,
}

impl SettingsPage {
    pub fn new(options: OptionsDialog) -> Self {
        SettingsPage {
            options,
            scheduler: None,
            speed: None,
            torrents: None,
            extension: None,
            ask: None,
            listed: None,
        }
    }

    /// The page on section `page` (a moved window's, plan 11b: its commands open it there).
    pub fn show_section(&mut self, page: usize) {
        self.options.show_page(page);
    }

    /// A small dialog over the page waits for an answer: the page's buttons and keys wait.
    pub fn modal(&self) -> bool {
        self.options.modal()
            || self.ask.is_some()
            || self.scheduler.as_ref().is_some_and(Scheduler::modal)
    }

    /// Whether Save would change anything, in any section.
    pub fn dirty(&self) -> bool {
        self.options.dirty()
            || self.scheduler.as_ref().is_some_and(Scheduler::dirty)
            || self.speed.as_ref().is_some_and(SpeedSettings::dirty)
            || self.torrents.as_ref().is_some_and(TorrentSettings::dirty)
    }

    /// The page on the Scheduler's section, its draft made from `queues` and `settings` if it
    /// has none yet: the Scheduler, to be shown on a queue or asked a new one's name.
    pub fn show_scheduler(&mut self, queues: &[Queue], settings: &Settings) -> &mut Scheduler {
        self.options.show_page(SCHEDULER_PAGE);
        self.scheduler
            .get_or_insert_with(|| Scheduler::new(queues, settings))
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
        tools: &mut Tools,
    ) -> Option<PageEvent> {
        crate::crash::window(crash_name(self.options.page()));
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
        let side_id = egui::Id::unique("settings-pages");
        // A page newly shown (opened on a section, or picked): its row into view, the list
        // moved no more than that. `scale::pane` keeps the list's offset under `id.with("pane")`,
        // which outlives this page: Options opened anew starts from the list's top.
        let page = self.options.page();
        if self.listed != Some(page) {
            let fresh = self.listed.is_none();
            self.listed = Some(page);
            if list_scrolls {
                ui.ctx().data_mut(|d| {
                    let off = d.get_temp_mut_or_default::<egui::Vec2>(side_id.with("pane"));
                    let from = if fresh { 0.0 } else { off.y };
                    off.y = row_into_view(from, OptionsDialog::page_row(page), list_room);
                });
            }
            // A row picked this pass (a key, a click) is scrolled to next pass: draw it now.
            ui.ctx().request_repaint();
        }
        let mut side = ui.new_child(egui::UiBuilder::new().max_rect(view));
        side.set_clip_rect(view.intersect(ui.clip_rect()));
        crate::scale::pane(&mut side, side_id, vec2(SIDE_W + 1.0, list_h), |ui, at| {
            // While a small dialog waits for an answer the page stays where it is.
            let before = self.options.page();
            self.options.pages(ui, at, list_h);
            if modal {
                self.options.show_page(before);
            }
        });

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
        // The side list may just have picked another section: what it shows now.
        let shown = self.options.page();
        if shown >= SCHEDULER_PAGE {
            self.tool_pane(ui, page_view, shown, tools);
        } else {
            self.options.page_pane(ui, page_view, downloads);
        }

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
            crate::help::open_page(help_page(self.options.page()));
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

    /// Back's question answered No, as its window's No does (tests).
    #[cfg(test)]
    pub(crate) fn answer_no(&mut self) {
        self.ask = None;
    }

    /// The windows over the page (a page's small dialogs, the discard question): every frame
    /// the page is open, from the app's `dialogs`. `Close` once Yes drops the changes.
    pub fn overlays(
        &mut self,
        ctx: &egui::Context,
        downloads: &std::path::Path,
    ) -> Option<PageEvent> {
        self.options.overlays(ctx, downloads);
        if let Some(s) = &mut self.scheduler {
            s.overlays(ctx);
        }
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

    /// The section of a moved window (`page`, not one of Options' own) in `view`, its draft
    /// made on the first visit.
    fn tool_pane(&mut self, ui: &mut Ui, view: Rect, page: usize, tools: &mut Tools) {
        match page {
            SCHEDULER_PAGE => {
                let s = self
                    .scheduler
                    .get_or_insert_with(|| Scheduler::new(tools.queues, tools.settings));
                section_pane(ui, view, page, crate::scheduler::SIZE, |ui, at| {
                    s.section(
                        ui,
                        at,
                        tools.icons,
                        tools.items,
                        tools.queues,
                        tools.wake_error,
                        tools.actions,
                    );
                });
            }
            SPEED_PAGE => {
                let s = self
                    .speed
                    .get_or_insert_with(|| SpeedSettings::new(tools.settings));
                section_pane(ui, view, page, s.size(), |ui, at| s.section(ui, at));
            }
            TORRENTS_PAGE => {
                let s = self
                    .torrents
                    .get_or_insert_with(|| TorrentSettings::new(tools.settings));
                section_pane(
                    ui,
                    view,
                    page,
                    crate::torrentdlg::SETTINGS_SIZE,
                    |ui, at| s.section(ui, at),
                );
            }
            _ => {
                let s = self.extension.get_or_insert_with(Extension::new);
                let size = s.size(ui.ctx());
                section_pane(ui, view, page, size, |ui, at| s.section(ui, at));
            }
        }
    }
}

/// A moved window's section in `view` (plan 11b): its heading, then the window's layout
/// (`size`, its client without the footer) under it, scrolled where `view` is smaller
/// (`scale::pane`); each section keeps its own scroll while the page is open.
fn section_pane(
    ui: &mut Ui,
    view: Rect,
    page: usize,
    size: [f32; 2],
    body: impl FnOnce(&mut Ui, egui::Pos2),
) {
    let mut pane = ui.new_child(egui::UiBuilder::new().max_rect(view));
    pane.set_clip_rect(view.intersect(ui.clip_rect()));
    let id = egui::Id::unique(("tool-section", page));
    let size = vec2(size[0], SECTION_TOP + size[1]);
    crate::scale::pane(&mut pane, id, size, |ui, at| {
        let _surface = winui::Surface::enter(at.x, at.x + size.x);
        // Options' heading is laid out from the side list's left edge.
        crate::options::heading(ui, at - vec2(SIDE_W + 1.0, 0.0), page);
        body(ui, at + vec2(0.0, SECTION_TOP));
    });
}

/// Each section's help page (Options' pages: options.md).
fn help_page(page: usize) -> &'static str {
    match page {
        SCHEDULER_PAGE => "queues-and-scheduler.md#the-scheduler",
        SPEED_PAGE => "connections-and-speed.md#speed-limiter",
        TORRENTS_PAGE => "torrents.md#torrent-settings",
        EXTENSION_PAGE => "browser-integration.md#set-it-up",
        _ => "options.md",
    }
}

/// Each section's name for crash reports (its window's, as before plan 11).
fn crash_name(page: usize) -> &'static str {
    match page {
        SCHEDULER_PAGE => "scheduler",
        SPEED_PAGE => "speed-settings",
        TORRENTS_PAGE => "torrent-settings",
        EXTENSION_PAGE => "extension",
        _ => "options",
    }
}

/// The list's offset `off` moved the least that shows `row` in `room` points of it.
fn row_into_view(off: f32, row: egui::Rangef, room: f32) -> f32 {
    let lo = (row.max - room).max(0.0);
    off.clamp(lo, row.min.max(lo))
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

    /// The two built-in queues, as a new list has them.
    fn queues() -> Vec<Queue> {
        vec![
            Queue::new("Main download queue", false),
            Queue::new("Synchronization queue", true),
        ]
    }

    /// `f` with what the sections need from an app with [`queues`] and default settings; what
    /// the Scheduler asked of the app, after.
    fn with_tools<R>(f: impl FnOnce(Tools) -> R) -> (R, Vec<Action>) {
        let (queues, settings) = (queues(), Settings::default());
        let mut actions = Vec::new();
        let out = f(Tools {
            icons: &mut ShellIcons,
            items: &[],
            queues: &queues,
            settings: &settings,
            wake_error: None,
            actions: &mut actions,
        });
        (out, actions)
    }

    /// One pass of the page in `area`: what it asked for, and the AccessKit tree.
    fn pass(
        ctx: &egui::Context,
        d: &mut SettingsPage,
        area: Rect,
        events: Vec<egui::Event>,
    ) -> (Option<PageEvent>, TreeUpdate) {
        pass_keys(ctx, d, area, events, true).0
    }

    /// [`pass`] with the window's keys the page's or not (`keys`); what the Scheduler asked of
    /// the app too.
    fn pass_keys(
        ctx: &egui::Context,
        d: &mut SettingsPage,
        area: Rect,
        events: Vec<egui::Event>,
        keys: bool,
    ) -> ((Option<PageEvent>, TreeUpdate), Vec<Action>) {
        let dir = std::env::temp_dir();
        let mut ev = None;
        let (mut out, actions) = with_tools(|mut tools| {
            ctx.run_ui(input(area.max.to_vec2(), events), |ui| {
                ev = d.show(ui, area, keys, false, &dir, &mut tools);
            })
        });
        out.textures_delta.clear();
        (
            (
                ev,
                out.platform_output
                    .accesskit_update
                    .expect("AccessKit is on"),
            ),
            actions,
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

    #[test]
    fn row_into_view_moves_the_least() {
        let row = egui::Rangef::new(400.0, 430.0);
        // Below the view: scrolled until its bottom shows; above: until its top does.
        assert_eq!(row_into_view(0.0, row, 200.0), 230.0);
        assert_eq!(row_into_view(500.0, row, 200.0), 400.0);
        // Already in view: the list stays.
        assert_eq!(row_into_view(300.0, row, 200.0), 300.0);
        // A row near the top in a short list: never above the list's start.
        assert_eq!(
            row_into_view(50.0, egui::Rangef::new(10.0, 40.0), 200.0),
            10.0
        );
    }

    /// Plan 11, PR 82's audit: a section opened by a command (`/show scheduler`, the toolbar's
    /// Scheduler) in a window too short for every row shows its row in the side list.
    #[test]
    fn a_section_opened_by_a_command_is_in_view_in_the_list() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = min_area();
        for (section, label) in [
            (SCHEDULER_PAGE, "Scheduler"),
            (EXTENSION_PAGE, "Browser extension"),
        ] {
            let mut d = page(&Settings::default());
            d.show_section(section);
            pass(&ctx, &mut d, area, vec![]);
            let (_, tree) = pass(&ctx, &mut d, area, vec![]);
            let (_, list) = find(&tree, Role::List, "Pages").expect("the side list");
            let list = bounds(list).expect("the list's bounds");
            let (_, row) = find(&tree, Role::ListItem, label).expect("the section's row");
            let row = bounds(row).expect("the row's bounds");
            assert!(
                row.top() >= area.top() + HEADER_H - 0.5 && row.bottom() <= area.bottom() + 0.5,
                "section {section}: its row {row:?} is out of the page's view {area:?} (list {list:?})"
            );
        }
    }

    /// PR 82's audit r2: the list's offset outlives a page (egui's temp data); Options opened
    /// anew on General after a visit to a section low in the list shows the list from its top.
    #[test]
    fn options_opened_anew_shows_the_list_from_its_top() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = min_area();
        let mut d = page(&Settings::default());
        d.show_section(EXTENSION_PAGE);
        pass(&ctx, &mut d, area, vec![]);
        pass(&ctx, &mut d, area, vec![]);
        // The list was scrolled well past General's row (else this test proves nothing).
        let key = egui::Id::unique("settings-pages").with("pane");
        let off = ctx
            .data(|m| m.get_temp::<egui::Vec2>(key))
            .unwrap_or_default();
        assert!(
            off.y > OptionsDialog::page_row(0).max,
            "the Extension's visit scrolled the list: {off:?}"
        );
        let mut d = page(&Settings::default());
        pass(&ctx, &mut d, area, vec![]);
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        let (_, row) = find(&tree, Role::ListItem, "General").expect("General's row");
        let row = bounds(row).expect("the row's bounds");
        let top = area.top() + HEADER_H + OptionsDialog::page_row(0).min;
        assert!(
            (row.top() - top).abs() < 0.5,
            "General's row {row:?} should be at the list's top, y {top}"
        );
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
        assert_eq!(rows.len(), 16, "every page's row, and the tools'");
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
        let ((ev, _), _) = pass_keys(
            &ctx,
            &mut d,
            area,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            false,
        );
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

    /// Plan 11b1: the Scheduler is a section of the page, after "Tools" in the side list: a
    /// click there shows its queue list, its tabs, Start now and Stop in the "Options" pane,
    /// above the page's save bar; its draft is made then, and a change in it is the page's
    /// (Save live, Back asks).
    #[test]
    fn the_scheduler_is_a_section_of_the_page() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = Rect::from_min_size(pos2(0.0, 100.0), vec2(1080.0, 600.0));
        let mut d = page(&Settings::default());
        pass(&ctx, &mut d, area, vec![]);
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        let row = bounds(find(&tree, Role::ListItem, "Scheduler").expect("its row").1).unwrap();
        assert!(d.scheduler.is_none(), "made on the first visit");
        click(&ctx, &mut d, area, row.center());
        assert_eq!(d.page(), SCHEDULER_PAGE);
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
            (Role::Tab, "Schedule"),
            (Role::Tab, "Files in the queue"),
            (Role::Button, "Start now"),
            (Role::Button, "Stop"),
            (Role::Button, "New queue"),
            (Role::Button, "Save"),
        ] {
            let (id, n) = tree
                .nodes
                .iter()
                .find(|(_, n)| {
                    n.role() == role
                        && n.label().map(|l| l.replace('&', "")).as_deref() == Some(label)
                })
                .unwrap_or_else(|| panic!("no {label}"));
            assert!(under.contains(id), "{label} under the pane");
            // The section above the save bar.
            if label != "Save" {
                let rc = bounds(n).unwrap();
                assert!(
                    rc.bottom() <= area.bottom() - winui::FOOTER_H,
                    "{label}: {rc:?}"
                );
            }
        }
        assert!(
            find(&tree, Role::Button, "Save").unwrap().1.is_disabled(),
            "nothing changed yet"
        );
        d.scheduler
            .as_mut()
            .expect("its draft")
            .set_limits(true, 321);
        assert!(d.dirty());
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        assert!(!find(&tree, Role::Button, "Save").unwrap().1.is_disabled());
        let (ev, _) = pass(
            &ctx,
            &mut d,
            area,
            vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert_eq!(ev, None, "Back asks first");
        assert!(d.ask.is_some());
        // Audit r1: while the question waits, a click on another page's row leaves the page
        // on the Scheduler.
        let (_, tree) = pass(&ctx, &mut d, area, vec![]);
        let general = bounds(find(&tree, Role::ListItem, "General").unwrap().1).unwrap();
        click(&ctx, &mut d, area, general.center());
        assert_eq!(d.page(), SCHEDULER_PAGE, "held while modal");
        d.ask = None;
        click(&ctx, &mut d, area, general.center());
        assert_eq!(d.page(), 0, "picked once answered");
    }

    /// Plan 11b2: Speed Limiter settings, Torrent settings and the browser extension are
    /// sections of the page, after the Scheduler: each shows its window's rows in the "Options"
    /// pane, without its window's buttons (OK, Cancel, Close, Don't ask again: the save bar is
    /// the page's). A change in the Speed Limiter or Torrents is the page's; the extension has
    /// nothing to save.
    #[test]
    fn the_moved_windows_are_sections_of_the_page() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let area = Rect::from_min_size(pos2(0.0, 100.0), vec2(1080.0, 700.0));
        let mut d = page(&Settings::default());
        for (section, row, control) in [
            (
                SPEED_PAGE,
                "Speed Limiter",
                &["Always turn on Speed Limiter on Grabbit startup"][..],
            ),
            (
                TORRENTS_PAGE,
                "Torrents",
                &["Keep sharing torrents after they finish"][..],
            ),
            (
                EXTENSION_PAGE,
                "Browser extension",
                // A row's button ("Show me how for Chrome"; "Add the extension…" once listed in
                // a store), whatever browsers this computer has.
                &["Show me how", "Add the extension"][..],
            ),
        ] {
            pass(&ctx, &mut d, area, vec![]);
            let (_, tree) = pass(&ctx, &mut d, area, vec![]);
            let at = bounds(find(&tree, Role::ListItem, row).expect(row).1).unwrap();
            click(&ctx, &mut d, area, at.center());
            assert_eq!(d.page(), section, "{row}");
            pass(&ctx, &mut d, area, vec![]);
            let (_, tree) = pass(&ctx, &mut d, area, vec![]);
            assert!(
                tree.nodes.iter().any(|(_, n)| n
                    .label()
                    .is_some_and(|l| control.iter().any(|c| l.starts_with(c)))),
                "{row}: {control:?}"
            );
            // Audit r1: a caption that ends before its box ends where the section has it, not
            // at the window's x (Torrents' were cut to "Stop shari..." right of the side list).
            if section == TORRENTS_PAGE {
                let caption = tree
                    .nodes
                    .iter()
                    .find(|(_, n)| {
                        n.role() == Role::CheckBox
                            && n.label() == Some("Stop sharing at a share ratio of")
                    })
                    .and_then(|(_, n)| bounds(n))
                    .expect("its check box");
                // Its click area holds the whole caption, in the font it is drawn in.
                let mut text = 0.0;
                let mut out = ctx.run_ui(input(area.max.to_vec2(), vec![]), |ui| {
                    text = winui::text_width(ui.painter(), "Stop sharing at a share ratio of");
                });
                out.textures_delta.clear();
                assert!(
                    text > 50.0 && caption.width() >= text,
                    "cut: {caption:?}, the caption {text} wide"
                );
            }
            for gone in ["OK", "Cancel", "Close", "Don't ask again"] {
                assert!(
                    find(&tree, Role::Button, gone).is_none(),
                    "{row}: no {gone}"
                );
            }
            assert!(!d.dirty(), "{row}: only looked at");
        }
        assert!(d.speed.is_some() && d.torrents.is_some());
        d.speed.as_mut().unwrap().on_startup ^= true;
        assert!(d.dirty(), "the Speed Limiter's change is the page's");
        d.speed = Some(SpeedSettings::new(&Settings::default()));
        let t = d.torrents.as_mut().unwrap();
        let share = t.choice().share;
        t.set_share(!share);
        assert!(d.dirty(), "Torrents' change is the page's");
    }

    /// Plan 11b2: the Speed Limiter's and Torrents' drafts read the settings back as they are
    /// (nothing to save when opened); what Save gives is what they hold.
    #[test]
    fn speed_and_torrent_choices_round_trip() {
        let s = Settings {
            speed_limit_kb: 123,
            limiter_on_startup: true,
            global_limit_on: true,
            global_limit_kb: 4567,
            global_only_active: true,
            torrent_keep_sharing: true,
            torrent_port: 51413,
            torrent_upload_on: true,
            torrent_upload_kb: 99,
            torrent_stop_ratio_on: true,
            torrent_stop_ratio: 150,
            ..Settings::default()
        };
        let speed = SpeedSettings::new(&s);
        assert!(!speed.dirty());
        let c = speed.choice();
        assert_eq!(
            (c.kb, c.on_startup, c.global_on, c.global_kb, c.only_active),
            (123, true, true, 4567, true)
        );
        let torrents = TorrentSettings::new(&s);
        assert!(!torrents.dirty());
        let c = torrents.choice();
        assert!(c.share && c.upload == (true, 99) && c.net.port == 51413);
        assert!(c.seeding.ratio_on && c.seeding.ratio == 150);
        assert_eq!(c.net, crate::torrentdlg::Net::of(&s));
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
