//! The main window's chrome painted to IDM 6.42's pixels (notes/ui-spec/main-window.md,
//! notes/ui-spec/menus.md): menu bar, popup menus, toolbar, Categories tree, shell icons.
//!
//! Our client area also holds the menu bar (IDM's is non-client), so every main-window
//! spec y is shifted by the menu bar's height ([`Layout::menu_h`], 20). Main-window spec
//! "baseline" is origin + ascent (the row under the glyphs); `winui::text_font` takes the last
//! ink row, i.e. spec baseline − 1.

use egui::{Color32, Pos2, Rect, Stroke, Ui, pos2, vec2};

use crate::icons::Icon;
use crate::scrollbar;
use crate::winui::{self, text_font, text_width_font};

/// Where the main window's bands and panes are, worked out once a frame from the theme
/// (plan 05a): `App::paint` and the painters here read every place from it, none repeats a
/// number. Heights and x are in points; y are from the top of the client area under a drawn
/// title bar (the `o` every painter gets).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// A drawn title bar's height (the system's own title bar is outside the client area).
    pub caption_h: f32,
    /// The menu bar's row, its bottom line included (IDM's: non-client y −20…−1).
    pub menu_h: f32,
    /// The toolbar under it: its two etched lines and its band.
    pub toolbar_h: f32,
    /// A toolbar button's drop-down menu: its top, down from the toolbar's top.
    pub drop_y: f32,
    /// The status strip under the panes.
    pub strip_h: f32,
    /// The Categories pane's frame, and the list's left edge while the pane is hidden.
    pub tree_left: f32,
    /// The splitter between the pane and the list.
    pub splitter_w: f32,
    /// From the Categories pane's and the splitter's bottom to the strip (the list's frame
    /// ends one row lower: its exclusive edge is `pane_gap - 1` above the strip).
    pub pane_gap: f32,
    /// From the list's right edge to the window's.
    pub list_right: f32,
    /// From the list's bottom edge (exclusive) to the strip.
    pub list_gap: f32,
    /// The menus are in the drawn title bar (Graphite's, `menu_h` 0), not in a row of their own.
    pub menus_in_caption: bool,
    /// The paste bar under the toolbar: [`Layout::PASTE_H`] when shown, 0 when hidden (plan 06).
    pub paste_h: f32,
    /// The details pane at the trailing edge: its width when shown, 0 when hidden (plan 06).
    pub details_w: f32,
}

impl Layout {
    /// Graphite's (DESIGN.md, the mockup's main window): a 36-point title bar that holds the
    /// menus, a 58-point toolbar band, the panes edge to edge, a 28-point status bar.
    pub const GRAPHITE: Layout = Layout {
        caption_h: 36.0,
        menu_h: 0.0,
        toolbar_h: 58.0,
        drop_y: TB_BUTTON_TOP + TB_BUTTON_H + 1.0,
        strip_h: 28.0,
        tree_left: 0.0,
        splitter_w: 0.0,
        pane_gap: 0.0,
        list_right: 0.0,
        list_gap: 0.0,
        menus_in_caption: true,
        paste_h: 0.0,
        details_w: 0.0,
    };

    /// The paste bar's band, its bottom rule included (the mockup's `.paste`: 8 + 32 + 8).
    pub const PASTE_H: f32 = 48.0;

    /// The details pane's splitter grip: its first 4 points, inside its leading rule.
    pub const DETAILS_GRIP: f32 = 4.0;

    /// The menu row under the system's title bar, its bottom line included (IDM's: non-client
    /// y -20..-1), when the title bar isn't ours.
    pub const MENU_ROW_H: f32 = 20.0;

    /// The layout this frame. `drawn_caption`: the title bar is ours (else the system's, and
    /// a menu row stays under it).
    pub fn current(drawn_caption: bool) -> Layout {
        if drawn_caption {
            Layout::GRAPHITE
        } else {
            Layout {
                menu_h: Layout::MENU_ROW_H,
                menus_in_caption: false,
                ..Layout::GRAPHITE
            }
        }
    }

    /// This frame's panes on top of `self` (plan 06): the paste bar's height when `paste`, the
    /// details pane's width when given (0 for `None`); every other field as it is.
    /// `with_panes(false, None)` is `self`.
    pub fn with_panes(self, paste: bool, details_w: Option<f32>) -> Layout {
        Layout {
            paste_h: if paste { Layout::PASTE_H } else { 0.0 },
            details_w: details_w.unwrap_or(0.0),
            ..self
        }
    }

    /// The list's bottom edge (exclusive) in a client area `h` high.
    pub fn list_bottom(&self, h: f32) -> f32 {
        self.strip_top(h) - self.list_gap
    }

    /// The toolbar's top, down from the client area's.
    pub fn toolbar_top(&self) -> f32 {
        self.menu_h
    }

    /// The paste bar's top, down from the client area's: under the toolbar.
    pub fn paste_top(&self) -> f32 {
        self.toolbar_top() + self.toolbar_h
    }

    /// Where the panes (Categories, the list) start, down from the client area's top: under
    /// the toolbar and the paste bar.
    pub fn panes_top(&self) -> f32 {
        self.paste_top() + self.paste_h
    }

    /// The list's trailing edge in a client area `w` wide: before its margin and the details
    /// pane.
    pub fn list_end(&self, w: f32) -> f32 {
        w - self.list_right - self.details_w
    }

    /// The details pane's leading edge in a client area `w` wide.
    pub fn details_left(&self, w: f32) -> f32 {
        w - self.details_w
    }

    /// The strip's top in a client area `h` high.
    pub fn strip_top(&self, h: f32) -> f32 {
        h - self.strip_h
    }
}

/// Plan 05b's accessibility and hit tests, headless: what the main window's bars give screen
/// readers and the window's hit test, in IDM's layout and Graphite's.
#[cfg(test)]
mod a11y_tests {
    use super::*;
    use egui::accesskit::{Node, NodeId, Role, Toggled};

    /// One frame of `paint` in a 1240 × 700 window, AccessKit on; its nodes by id, and what
    /// `paint` returned. `events`: this frame's input.
    fn frame<R>(
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        paint: impl FnOnce(&mut Ui) -> R,
    ) -> (std::collections::HashMap<NodeId, Node>, R) {
        frame_wide(ctx, 1240.0, events, paint)
    }

    /// [`frame`] in a window `w` wide (700 high).
    fn frame_wide<R>(
        ctx: &egui::Context,
        w: f32,
        events: Vec<egui::Event>,
        paint: impl FnOnce(&mut Ui) -> R,
    ) -> (std::collections::HashMap<NodeId, Node>, R) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(w, 700.0))),
            events,
            ..Default::default()
        };
        let mut out = None;
        let mut paint = Some(paint);
        let mut full = ctx.run_ui(input, |ui| {
            if let Some(p) = paint.take() {
                out = Some(p(ui));
            }
        });
        full.textures_delta.clear();
        let update = full
            .platform_output
            .accesskit_update
            .expect("AccessKit is on");
        (update.nodes.into_iter().collect(), out.expect("painted"))
    }

    fn named(nodes: &std::collections::HashMap<NodeId, Node>, role: Role) -> Vec<(String, bool)> {
        let mut v: Vec<(String, bool)> = nodes
            .values()
            .filter(|n| n.role() == role)
            .map(|n| (n.label().unwrap_or_default().to_owned(), n.is_disabled()))
            .collect();
        v.sort();
        v
    }

    const BUTTONS: [(Icon, &str, bool); 11] = [
        (Icon::AddUrl, "Add URL", true),
        (Icon::AddTorrent, "Add Torrent", true),
        (Icon::Resume, "Resume", false),
        (Icon::Stop, "Stop", false),
        (Icon::StopAll, "Stop All", false),
        (Icon::Delete, "Delete", true),
        (Icon::DeleteCompleted, "Delete Completed", true),
        (Icon::Options, "Options", true),
        (Icon::Scheduler, "Scheduler", true),
        (Icon::StartQueue, "Start Queue", true),
        (Icon::StopQueue, "Stop Queue", false),
    ];

    /// The toolbar's 11 buttons are buttons for screen readers, by their labels, greyed as
    /// drawn.
    #[test]
    fn the_toolbar_exposes_its_eleven_buttons() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut nodes = Default::default();
        for _ in 0..2 {
            (nodes, _) = frame(&ctx, vec![], |ui| {
                toolbar(ui, &Layout::GRAPHITE, Pos2::ZERO, 1240.0, &BUTTONS, None)
            });
        }
        let mut want: Vec<(String, bool)> = BUTTONS
            .iter()
            .map(|(_, l, on)| ((*l).to_owned(), !on))
            .collect();
        want.sort();
        assert_eq!(named(&nodes, Role::Button), want);
    }

    /// 04's overflow (plans/04-typeface.md): at 200 % the window is about 710 points wide, and
    /// Graphite's toolbar keeps every button inside it, the labels cut.
    #[test]
    fn the_graphite_toolbar_fits_a_narrow_window() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut nodes = Default::default();
        for _ in 0..2 {
            (nodes, _) = frame(&ctx, vec![], |ui| {
                toolbar(ui, &Layout::GRAPHITE, Pos2::ZERO, 710.0, &BUTTONS, None)
            });
        }
        let rights: Vec<f64> = nodes
            .values()
            .filter(|n| n.role() == Role::Button)
            .filter_map(|n| n.bounds().map(|b| b.x1))
            .collect();
        assert_eq!(rights.len(), 11);
        assert!(rights.iter().all(|r| *r <= 710.0), "{rights:?}");
    }

    /// A toolbar button as a screen reader has it: name, greyed, toggled state, bounds.
    type ButtonNode = (String, bool, Option<Toggled>, Option<[f64; 4]>);

    /// The toolbar's buttons in their order across the band.
    fn toolbar_buttons(nodes: &std::collections::HashMap<NodeId, Node>) -> Vec<ButtonNode> {
        let mut v: Vec<ButtonNode> = nodes
            .values()
            .filter(|n| n.role() == Role::Button)
            .map(|n| {
                (
                    n.label().unwrap_or_default().to_owned(),
                    n.is_disabled(),
                    n.toggled(),
                    n.bounds().map(|b| [b.x0, b.y0, b.x1, b.y1]),
                )
            })
            .collect();
        v.sort_by(|a, b| {
            let x = |n: &ButtonNode| n.3.map_or(f64::MAX, |b| b[0]);
            x(a).total_cmp(&x(b))
        });
        v
    }

    /// The main window's Details toggle, on or off.
    fn details(on: bool) -> Option<ToolbarToggle<'static>> {
        Some(ToolbarToggle {
            icon: Icon::Details,
            label: "Details",
            on,
        })
    }

    /// The toolbar's nodes after two frames 1240 wide with `toggle`.
    fn toolbar_nodes(
        ctx: &egui::Context,
        toggle: impl Fn() -> Option<ToolbarToggle<'static>>,
    ) -> std::collections::HashMap<NodeId, Node> {
        let mut nodes = Default::default();
        for _ in 0..2 {
            (nodes, _) = frame(ctx, vec![], |ui| {
                toolbar(
                    ui,
                    &Layout::GRAPHITE,
                    Pos2::ZERO,
                    1240.0,
                    &BUTTONS,
                    toggle(),
                )
            });
        }
        nodes
    }

    /// A screen reader's Click on the node named `label`.
    fn click_on(nodes: &std::collections::HashMap<NodeId, Node>, label: &str) -> egui::Event {
        let (id, _) = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some(label))
            .unwrap_or_else(|| panic!("a {label} button"));
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: *id,
            data: None,
        })
    }

    /// With the Details toggle the toolbar has 12 buttons: the 11 as before, then "Details",
    /// usable, its toggled state the setting's (requirements 11.4).
    #[test]
    fn the_toolbar_exposes_the_details_toggle_as_its_twelfth_button() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        for on in [false, true] {
            let buttons = toolbar_buttons(&toolbar_nodes(&ctx, || details(on)));
            assert_eq!(buttons.len(), 12, "{buttons:?}");
            for ((label, disabled, toggled, _), (_, want, enabled)) in buttons.iter().zip(&BUTTONS)
            {
                assert_eq!(
                    (label.as_str(), *disabled, *toggled),
                    (*want, !enabled, None)
                );
            }
            let (label, disabled, toggled, _) = &buttons[11];
            let want = if on { Toggled::True } else { Toggled::False };
            assert_eq!(
                (label.as_str(), *disabled, *toggled),
                ("Details", false, Some(want))
            );
        }
    }

    /// A screen reader's Click on Details presses the toggle (requirements 11.5).
    #[test]
    fn a_screen_readers_click_presses_the_toolbars_details_toggle() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let nodes = toolbar_nodes(&ctx, || details(false));
        let click = click_on(&nodes, "Details");
        let (_, event) = frame(&ctx, vec![click], |ui| {
            toolbar(
                ui,
                &Layout::GRAPHITE,
                Pos2::ZERO,
                1240.0,
                &BUTTONS,
                details(false),
            )
        });
        assert!(matches!(event, Some(ToolbarEvent::Toggle)));
    }

    /// Without a toggle the toolbar is as before (requirements 4.10): its 11 buttons, none with
    /// a toggled state, where they are beside the toggle; a Click on one is still that button's.
    #[test]
    fn without_a_toggle_the_toolbar_is_as_before() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let nodes = toolbar_nodes(&ctx, || None);
        let plain = toolbar_buttons(&nodes);
        assert_eq!(plain.len(), 11, "{plain:?}");
        assert!(plain.iter().all(|b| b.2.is_none() && b.0 != "Details"));
        let with = toolbar_buttons(&toolbar_nodes(&ctx, || details(true)));
        assert_eq!(plain[..], with[..11]);
        let click = click_on(&nodes, "Add URL");
        let (_, event) = frame(&ctx, vec![click], |ui| {
            toolbar(ui, &Layout::GRAPHITE, Pos2::ZERO, 1240.0, &BUTTONS, None)
        });
        assert!(matches!(event, Some(ToolbarEvent::Click(0))));
    }

    /// Property 15, toolbar fit: for any width from 710 (04's 200 %) to 2400, drawn from any
    /// `o.x`, every button's and the Details toggle's right edge is within `o.x + width`, and
    /// the toggle starts past the last button. One context for every case (a frame's fonts).
    ///
    /// **Validates: Requirements 4.9**
    #[test]
    fn the_toolbar_and_its_details_toggle_fit_any_width_from_710() {
        use proptest::test_runner::{Config, TestCaseError, TestRunner};
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut runner = TestRunner::new(Config::with_cases(256));
        let widths = (0.0f32..=120.0, 710.0f32..=2400.0, proptest::bool::ANY);
        let result = runner.run(&widths, |(ox, width, on)| {
            let o = pos2(ox, 0.0);
            let mut nodes = Default::default();
            for _ in 0..2 {
                (nodes, _) = frame_wide(&ctx, ox + width, vec![], |ui| {
                    toolbar(ui, &Layout::GRAPHITE, o, width, &BUTTONS, details(on))
                });
            }
            let buttons = toolbar_buttons(&nodes);
            let end = f64::from(o.x + width);
            let bounds: Vec<[f64; 4]> = buttons.iter().filter_map(|b| b.3).collect();
            let fail = |why: &str| {
                Err(TestCaseError::fail(format!(
                    "{why}: o.x {ox}, width {width}, on {on}, {buttons:?}"
                )))
            };
            if buttons.len() != 12 || bounds.len() != 12 || buttons[11].0 != "Details" {
                return fail("12 buttons, Details last");
            }
            if !bounds.iter().all(|b| b[2] <= end) {
                return fail("a right edge past o.x + width");
            }
            if bounds[11][0] < bounds[10][2] {
                return fail("the toggle over the last button");
            }
            Ok(())
        });
        if let Err(e) = result {
            panic!("{e}");
        }
    }

    /// Graphite's title-bar menus: still a menu bar of five menu items for screen readers, and
    /// each title's rect (the one the window's hit test is given, App::publish_caption) finds
    /// that title again ([`menubar_title_at`]); a screen reader's Click on a title opens it.
    #[test]
    fn the_title_bar_menus_are_a_menu_bar() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let (nodes, (_, titles)) = frame(&ctx, vec![], |ui| {
            let rel = caption_titles(ui.painter(), 0.0, 0.0);
            caption_menu_bar(ui, &rel, 300.0, 17.0, None)
        });
        assert_eq!(named(&nodes, Role::MenuBar).len(), 1);
        let mut items: Vec<String> = named(&nodes, Role::MenuItem)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        items.sort();
        assert_eq!(items, ["Downloads", "File", "Help", "Tasks", "View"]);
        assert_eq!(titles.len(), 5);
        for (i, t) in titles.iter().enumerate() {
            assert_eq!(menubar_title_at(&titles, winui::rect(*t).center()), Some(i));
            assert!(
                t.top() >= 4.0 && t.bottom() <= 31.0,
                "inside the bar: {t:?}"
            );
        }
        // Between two titles: none (the window's caption, which drags it).
        let gap = pos2(titles[0].right() + 0.5, titles[0].center().y);
        assert_eq!(menubar_title_at(&titles, gap), None);
        // Click (a screen reader's or UI Automation's) on File: its menu opens.
        let file = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::MenuItem && n.label() == Some("File"))
            .map(|(id, _)| *id)
            .unwrap();
        let click = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Expand,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: file,
            data: None,
        });
        let (_, (event, _)) = frame(&ctx, vec![click], |ui| {
            let rel = caption_titles(ui.painter(), 0.0, 0.0);
            caption_menu_bar(ui, &rel, 300.0, 17.0, None)
        });
        assert_eq!(
            event,
            Some(MenuBarEvent::Title {
                i: 1,
                released: false
            })
        );
    }

    /// Graphite's Find button is a button named Find, and a screen reader's Click presses it
    /// (Cmd::Find, as Ctrl+F).
    #[test]
    fn the_find_button_is_a_button() {
        let _fonts = crate::theme::font_tests();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let rc = Rect::from_min_size(pos2(800.0, 5.0), vec2(120.0, 26.0));
        let (nodes, clicked) = frame(&ctx, vec![], |ui| find_button(ui, rc, "Ctrl+F"));
        assert!(!clicked);
        let (id, _) = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some("Find"))
            .expect("a Find button");
        let click = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: *id,
            data: None,
        });
        let (_, clicked) = frame(&ctx, vec![click], |ui| find_button(ui, rc, "Ctrl+F"));
        assert!(clicked);
    }

    /// Graphite's menus draw each item's icon by its command; a submenu's and a greyed
    /// placeholder's by their labels, which are the app's own.
    #[test]
    fn menu_icons_for_submenus_and_placeholders() {
        assert_eq!(menu_icon("Speed &Limiter"), Some("gx-gauge"));
        assert_eq!(menu_icon("E&xport"), Some("gx-upload"));
        assert_eq!(menu_icon("Delete"), Some("tb-delete"));
        // A label no menu has (a queue named so is a command's item): none.
        assert_eq!(menu_icon("My downloads"), None);
    }
}

#[cfg(test)]
mod layout_tests {
    use super::Layout;

    /// Graphite's (plan 05b): the menus in a 36-point title bar, so the toolbar starts at the
    /// client's top; its band 58; the panes edge to edge down to a 28-point status bar; a
    /// drop-down's menu just under its button.
    #[test]
    fn the_graphite_layout_moves_the_panes_up() {
        let l = Layout::GRAPHITE;
        assert_eq!((l.caption_h, l.menu_h, l.toolbar_top()), (36.0, 0.0, 0.0));
        assert_eq!(l.panes_top(), 58.0);
        assert_eq!(l.drop_y, super::TB_BUTTON_TOP + super::TB_BUTTON_H + 1.0);
        assert!(l.drop_y < l.toolbar_h);
        assert_eq!(l.strip_top(600.0), 572.0);
        assert_eq!(l.list_bottom(600.0), 572.0);
        assert_eq!((l.tree_left, l.list_right), (0.0, 0.0));
        assert!(l.menus_in_caption);
        // Under the system's title bar, a menu row of its own; the rest as above.
        let s = Layout::current(false);
        assert_eq!((s.menu_h, s.toolbar_top()), (20.0, 20.0));
        assert_eq!(s.panes_top(), 78.0);
        assert!(!s.menus_in_caption);
        assert_eq!(Layout::current(true).panes_top(), 58.0);
    }

    /// Plan 06: with the paste bar and the details pane both off, the layout is today's for
    /// either title bar (panes at 58 under Graphite's, 78 under the system's).
    #[test]
    fn both_panes_off_is_todays_layout() {
        for drawn in [true, false] {
            let l = Layout::current(drawn).with_panes(false, None);
            assert_eq!(l, Layout::current(drawn));
            assert_eq!((l.paste_h, l.details_w), (0.0, 0.0));
            assert_eq!(l.list_end(1240.0), 1240.0);
        }
        let (drawn, system) = (Layout::current(true), Layout::current(false));
        assert_eq!(drawn.with_panes(false, None).panes_top(), 58.0);
        assert_eq!(system.with_panes(false, None).panes_top(), 78.0);
    }

    /// The paste bar's 48 points go between the toolbar and the panes: its top is the
    /// toolbar's bottom, the panes start under it (106 under Graphite's title bar, 126 under
    /// the system's).
    #[test]
    fn the_paste_bar_moves_the_panes_down() {
        let l = Layout::current(true).with_panes(true, None);
        assert_eq!(l.paste_h, Layout::PASTE_H);
        assert_eq!(l.paste_top(), 58.0);
        assert_eq!(l.panes_top(), 106.0);
        let s = Layout::current(false).with_panes(true, None);
        assert_eq!(s.paste_top(), 78.0);
        assert_eq!(s.panes_top(), 126.0);
        // The paste bar leaves the list's trailing edge alone.
        assert_eq!(l.list_end(1240.0), 1240.0);
    }

    /// The details pane's width comes off the list's trailing edge, after the list's margin;
    /// the pane starts where the list ends when there is no margin.
    #[test]
    fn the_details_pane_ends_the_list_before_it() {
        let l = Layout::current(true).with_panes(false, Some(276.0));
        assert_eq!(l.details_w, 276.0);
        assert_eq!(l.panes_top(), 58.0);
        assert_eq!(l.list_end(1240.0), 964.0);
        assert_eq!(l.details_left(1240.0), 964.0);
        // A layout with a right margin: the margin and the pane both come off.
        let m = Layout {
            list_right: 2.0,
            ..l
        };
        assert_eq!(m.list_end(1240.0), 962.0);
        // Both on: the paste bar and the pane together.
        let both = Layout::current(false).with_panes(true, Some(300.0));
        assert_eq!((both.panes_top(), both.list_end(1000.0)), (126.0, 700.0));
    }

    // Feature: paste-bar-details-pane, Property 6: Layout neutrality.
    // **Validates: Requirements 5.5**
    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]
        /// Both panes off is today's layout for either title bar, and `with_panes` changes no
        /// field but the paste bar's height and the details pane's width.
        #[test]
        fn with_panes_changes_only_the_panes(
            drawn in proptest::prelude::any::<bool>(),
            paste in proptest::prelude::any::<bool>(),
            details in proptest::option::of(0.0f32..=100_000.0),
        ) {
            let base = Layout::current(drawn);
            proptest::prop_assert_eq!(base.with_panes(false, None), base);
            let l = base.with_panes(paste, details);
            proptest::prop_assert_eq!(
                Layout {
                    paste_h: base.paste_h,
                    details_w: base.details_w,
                    ..l
                },
                base
            );
        }
    }

    // Feature: paste-bar-details-pane, Property 7: Layout arithmetic.
    // **Validates: Requirements 5.1, 5.2, 5.3, 5.4**
    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]
        /// The paste bar is 48 high when shown and 0 when hidden, right under the toolbar; the
        /// panes start under both; the details pane's width comes off the list's trailing edge
        /// after its margin.
        #[test]
        fn the_panes_add_up(
            drawn in proptest::prelude::any::<bool>(),
            paste in proptest::prelude::any::<bool>(),
            d in 0.0f32..=100_000.0,
            w in 0.0f32..=100_000.0,
        ) {
            let l = Layout::current(drawn).with_panes(paste, Some(d));
            let paste_h = if paste { 48.0 } else { 0.0 };
            proptest::prop_assert_eq!(l.paste_h, paste_h);
            proptest::prop_assert_eq!(l.paste_top(), l.toolbar_top() + l.toolbar_h);
            proptest::prop_assert_eq!(l.panes_top(), l.toolbar_top() + l.toolbar_h + paste_h);
            proptest::prop_assert_eq!(l.details_w, d);
            proptest::prop_assert_eq!(l.list_end(w), w - l.list_right - d);
        }
    }
}

use crate::theme::themed;
// The active theme's colours (crate::palette, #45).
themed! {
    pub WINDOW_BG => window;
    MENUBAR_BG => menubar;
    /// The bar's last row (light: #f2f2f2).
    MENUBAR_BOTTOM => menubar_bottom;
    /// The open menu's item. Light: #f9f9f9, the popup's own background (captured 2026-10-30).
    MENUBAR_OPEN => menubar_open;
    /// An item under the pointer, no menu open (captured 2026-10-30): dark #505050 from 1 px
    /// below the bar's top; light #f5f5f5, 1 px inside the bar's top and bottom.
    MENUBAR_HOT => menubar_hot;
    MENUBAR_LINE => menubar_line;
    POPUP_BG => menu;
    // The native menu window's frame (Windows draws menus in their own window), and every
    // in-window popup's (popup_surface).
    POPUP_BORDER => menu_border;
    POPUP_SEP => menu_sep;
    /// Light: #f0f0f0, rounded, captured 2026-10-30.
    POPUP_HOVER => menu_hot;
    POPUP_ITEM => menu_item;
    TEXT => menu_text;
    /// Bullets and submenu arrows (light: dark cores #1a1a1a / #282828).
    GLYPH => menu_glyph;
    TEXT_DISABLED_MENU => menu_text_disabled;
    TB_DISABLED => toolbar_text_disabled;
    /// The tree's chevrons.
    TREE_EXPANDER => tree_expander;
}

use crate::winui::{fill, px};

// ---- shell icons ---------------------------------------------------------------------------

/// What a 16 × 16 icon shows: a file type (by extension) or a folder.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IconKey {
    Ext(String),
    /// 32 × 32 (Download File Info).
    ExtLarge(String),
    /// The theme icon set's icon for a need (`crate::themeicons`), else Graphite's.
    Themed(&'static str),
    /// Graphite's icon for a need, whatever the theme (`themeicons::graphite`).
    Graphite(&'static str),
}

/// The small icons drawn by key: the icon set's (the Categories tree), Graphite's (the
/// Scheduler's queues) and the file types', which fileicons.rs draws.
#[derive(Default)]
pub struct ShellIcons;

impl ShellIcons {
    /// Paint the icon for `key` with its top-left at English point `min` (16 × 16, the large ones
    /// 32): placed where that is on screen, not mirrored. A generic page if unavailable.
    pub fn paint(&mut self, ui: &Ui, key: IconKey, min: Pos2) {
        self.paint_in(ui, ui.painter(), key, min);
    }

    /// `paint` through `painter`, so the icon keeps to its clip (a list scrolled sideways must
    /// not draw its icons over the pane beside it).
    pub fn paint_in(&mut self, ui: &Ui, painter: &egui::Painter, key: IconKey, min: Pos2) {
        let side = if matches!(key, IconKey::ExtLarge(_)) {
            32.0
        } else {
            16.0
        };
        let rc = winui::rect(Rect::from_min_size(min, vec2(side, side)));
        match &key {
            IconKey::Themed(need) => {
                if !crate::themeicons::paint(ui, painter, need, rc, true) {
                    crate::themeicons::paint_graphite(ui, painter, need, rc, true);
                }
            }
            IconKey::Graphite(need) => {
                crate::themeicons::paint_graphite(ui, painter, need, rc, true);
            }
            IconKey::Ext(e) | IconKey::ExtLarge(e) => {
                // The key holds the extension alone (empty when there is none).
                let name = if e.is_empty() {
                    "file".to_string()
                } else {
                    format!("x.{e}")
                };
                crate::fileicons::paint(ui, painter, crate::fileicons::Kind::File(&name), rc);
            }
        }
    }
}

// ---- menu bar --------------------------------------------------------------------------------

/// Top-level menus: label, item rect x and width (measured, "Ta" kerned by the font): the
/// standard Win32 bar (text at item.x + 7, width = text + 14; light-main.md), in both modes
/// (user decision 2026-11-01: dark mode only changes colours, never sizes or places; IDM's dark
/// bar was wider).
const MENUS: [(&str, f32, f32); 5] = [
    ("&Tasks", 0.0, 42.0),
    ("&File", 42.0, 32.0),
    ("&Downloads", 74.0, 73.0),
    ("&View", 147.0, 39.0),
    ("&Help", 186.0, 39.0),
];

/// The menu bar: the measured rects.
pub fn menus() -> Vec<(&'static str, f32, f32)> {
    MENUS.to_vec()
}

/// Paint the menu bar in its own row; returns what happened to its titles this frame, for
/// [`menubar_next`] (a title clicked, or opened and shut by a screen reader, or the pointer over
/// one, which only moves to it from another open menu), and the titles' rects (English).
pub fn menu_bar(
    ui: &mut Ui,
    layout: &Layout,
    o: Pos2,
    width: f32,
    open: Option<usize>,
) -> (Option<MenuBarEvent>, Vec<Rect>) {
    let p = ui.painter().clone();
    let menu_h = layout.menu_h;
    fill(
        &p,
        o.x,
        o.y,
        o.x + width - 1.0,
        o.y + menu_h - 2.0,
        MENUBAR_BG(),
    );
    // The bar's last row (light: #f2f2f2; dark: the bar's own).
    fill(
        &p,
        o.x,
        o.y + menu_h - 2.0,
        o.x + width - 1.0,
        o.y + menu_h - 2.0,
        MENUBAR_BOTTOM(),
    );
    fill(
        &p,
        o.x,
        o.y + menu_h - 1.0,
        o.x + width - 1.0,
        o.y + menu_h - 1.0,
        MENUBAR_LINE(),
    );
    // From the leading edge: the first menu at the right in RTL.
    let titles: Vec<Rect> = menus()
        .iter()
        .map(|(_, x, w)| Rect::from_min_size(o + vec2(*x, 0.0), vec2(*w, menu_h)))
        .collect();
    let bar = Rect::from_min_size(o, vec2(width, menu_h));
    let hit = bar_titles(ui, bar, &titles, open, false);
    (hit, titles)
}

/// Graphite's menu titles, in its title bar after the window's title (plan 05b, the mockup's
/// `.menu`): from English x `x`, each its text with 8 either side, 1 apart, 26 high, centred
/// on the bar's middle `mid`.
pub fn caption_titles(p: &egui::Painter, x: f32, mid: f32) -> Vec<Rect> {
    let mut x = x;
    menus()
        .iter()
        .map(|(label, ..)| {
            let w = text_width_font(p, winui::segoe(), &plain(label).0).ceil() + 16.0;
            let rc = Rect::from_min_size(pos2(x, (mid - 13.0).round()), vec2(w, 26.0));
            x += w + 1.0;
            rc
        })
        .collect()
}

/// The room Graphite's menu titles take after the window's title: the gap before them and the
/// titles themselves (`rel`: [`caption_titles`] at 0, 0).
pub fn caption_menus_width(rel: &[Rect]) -> f32 {
    CAPTION_MENUS_GAP + rel.last().map_or(0.0, |r| r.right())
}

/// From the window's title to the first menu title (the mockup's 10 + 8).
pub const CAPTION_MENUS_GAP: f32 = 18.0;

/// Paint Graphite's menu titles in the title bar (`rel`: [`caption_titles`] at 0, 0, placed from
/// English x `x`, centred on `mid`); what happened to them, as [`menu_bar`], and their rects
/// (English).
pub fn caption_menu_bar(
    ui: &mut Ui,
    rel: &[Rect],
    x: f32,
    mid: f32,
    open: Option<usize>,
) -> (Option<MenuBarEvent>, Vec<Rect>) {
    let by = vec2(x, (mid - 13.0).round() + 13.0);
    let titles: Vec<Rect> = rel.iter().map(|r| r.translate(by)).collect();
    let bar = titles.iter().fold(Rect::NOTHING, |a, r| a.union(*r));
    let hit = bar_titles(ui, bar, &titles, open, true);
    (hit, titles)
}

/// Graphite's Find button, in the title bar between the menus and the window's buttons (plan
/// 05b; 00's accepted exception: a new way to Downloads → Find): its size for the label and
/// `key` ("Ctrl+F").
pub fn find_button_size(p: &egui::Painter, key: &str) -> egui::Vec2 {
    let label = text_width_font(p, winui::segoe(), crate::i18n::t("Find")).ceil();
    let key = text_width_font(p, key_font(), key).ceil();
    vec2(9.0 + 14.0 + 7.0 + label + 7.0 + key + 9.0, 26.0)
}

/// The font of a Graphite menu's shortcut column and of the Find button's key.
fn key_font() -> egui::FontId {
    egui::FontId::proportional(crate::theme::metrics().font_small)
}

/// Paint the Find button in English rect `rc` ([`find_button_size`]): the window's face, the
/// buttons' border, radius 6, a magnifier, "Find" and its key in the shortcuts' ink. Exposed as
/// a button named Find. True when clicked (or invoked by a screen reader).
pub fn find_button(ui: &mut Ui, rc: Rect, key: &str) -> bool {
    let p = ui.painter().clone();
    let screen = winui::rect(rc);
    let resp = ui.interact(
        screen,
        egui::Id::unique("caption-find"),
        winui::pointer_only(),
    );
    // i18n: the title bar's button that opens Find file (Ctrl+F)
    let label = crate::i18n::t("Find");
    winui::expose(&resp, egui::Role::Button, label, true, None);
    let c = crate::theme::colors();
    let face = if resp.is_pointer_button_down_on() {
        c.button_pressed
    } else if resp.hovered() {
        c.button_hot
    } else {
        c.window
    };
    let radius = crate::theme::metrics().part_radius;
    p.rect_filled(screen, radius, face);
    p.rect_stroke(
        screen,
        radius,
        Stroke::new(1.0, c.button_border),
        egui::StrokeKind::Inside,
    );
    let mid = rc.center().y;
    let icon = Rect::from_min_size(pos2(rc.left() + 9.0, (mid - 7.0).round()), vec2(14.0, 14.0));
    // The mockup's: the icon and the label in the secondary ink, the key in the shortcuts'.
    let ink = c.text_secondary;
    crate::themeicons::paint_in(ui, &p, "gx-search", winui::rect(icon), ink, true);
    let baseline = rc.top() + 17.0;
    let drawn = text_font(&p, winui::segoe(), icon.right() + 7.0, baseline, label, ink);
    text_font(
        &p,
        key_font(),
        winui::rect(drawn).right() + 7.0,
        baseline,
        key,
        c.menu_shortcut,
    );
    resp.clicked()
}

/// The menu bar's titles in `titles` (English rects; `bar` around them all), drawn IDM's way
/// or Graphite's (`graphite`), with what happened to them this frame.
fn bar_titles(
    ui: &mut Ui,
    bar: Rect,
    titles: &[Rect],
    open: Option<usize>,
    graphite: bool,
) -> Option<MenuBarEvent> {
    let p = ui.painter().clone();
    let mut hit = None;
    // For screen readers and UI Automation (#113): a menu bar whose titles are menu items that
    // open their menus (Click, as with the mouse).
    let bar = ui.interact(
        winui::rect(bar),
        egui::Id::unique("menubar-node"),
        egui::Sense::hover(),
    );
    winui::expose(&bar, egui::Role::MenuBar, "", true, None);
    let titles_ui = ui.new_child(egui::UiBuilder::new().accessibility_parent(bar.id));
    let released = ui.input(|x| x.pointer.any_released());
    for (i, ((label, ..), english)) in menus().iter().zip(titles).enumerate() {
        let rc = winui::rect(*english);
        let resp = titles_ui.interact(rc, egui::Id::unique(("menubar", i)), winui::pointer_only());
        winui::expose(&resp, egui::Role::MenuItem, &plain(label).0, true, None);
        ui.ctx().accesskit_node_builder(resp.id, |n| {
            n.set_has_popup(egui::accesskit::HasPopup::Menu);
            n.set_expanded(open == Some(i));
        });
        // The title's text start and its last ink row.
        let (tx, baseline) = if graphite {
            // Open: the selection's fill; under the pointer, the caption buttons' tint.
            let radius = crate::theme::metrics().menu_item_radius;
            if open == Some(i) {
                p.rect_filled(rc, radius, winui::SELECTION());
            } else if open.is_none() && resp.hovered() {
                p.rect_filled(rc, radius, crate::theme::colors().caption_hot);
            }
            (english.left() + 8.0, english.top() + 17.0)
        } else {
            if open == Some(i) {
                p.rect_filled(rc, 0.0, MENUBAR_OPEN());
            } else if open.is_none() && resp.hovered() {
                let hot = Rect::from_min_max(rc.min + vec2(0.0, 1.0), rc.max - vec2(0.0, 1.0));
                p.rect_filled(hot, 0.0, MENUBAR_HOT());
            }
            (english.left() + 7.0, english.top() + 13.0)
        };
        // Graphite's titles in the secondary ink, the open one in the caption's.
        let ink = if graphite && open == Some(i) {
            winui::caption::CAPTION_INK()
        } else if graphite {
            crate::theme::colors().text_secondary
        } else {
            TEXT()
        };
        // The title without its "&", its access letter underlined (each language marks its
        // own: French Tâches and Téléchargements both had T).
        let (shown, letter) = plain(label);
        let drawn = text_font(&p, winui::segoe(), tx, baseline, &shown, ink);
        // Dark mode always underlines mnemonics (1 px under the letter's advance); light mode
        // follows the Windows rule: once the keyboard is in use (`cues`).
        if cues(ui.ctx()) {
            let (x0, x1) =
                winui::char_span(&p, &winui::segoe(), &shown, letter.unwrap_or(0), drawn);
            fill(&p, x0, baseline + 2.0, x1, baseline + 2.0, ink);
        }
        // A screen reader or UI Automation opens a title's menu with Expand (or Click) and
        // shuts it with Collapse.
        let (expand, collapse) = ui.input(|x| {
            (
                x.has_accesskit_action_request(resp.id, egui::accesskit::Action::Expand),
                x.has_accesskit_action_request(resp.id, egui::accesskit::Action::Collapse),
            )
        });
        if collapse && open == Some(i) {
            hit = Some(MenuBarEvent::Title { i, released: true });
        } else if resp.clicked() || (expand && open != Some(i)) {
            hit = Some(MenuBarEvent::Title { i, released });
        } else if resp.hovered() && open.is_some_and(|o| o != i) {
            hit = Some(MenuBarEvent::Hover(i));
        }
        if resp.clicked() || expand || collapse {
            // No pointer moves to bring the next frame: the menu is drawn (or shut) on it.
            ui.ctx().request_repaint();
        }
    }
    hit
}

/// The menu-bar title under `pos` (on screen), of `titles` as [`menu_bar`] or
/// [`caption_menu_bar`] laid them out this frame (English): the first menu at the right in RTL.
pub fn menubar_title_at(titles: &[Rect], pos: Pos2) -> Option<usize> {
    titles.iter().position(|r| winui::rect(*r).contains(pos))
}

/// What happened to the menu bar this frame, for [`menubar_next`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuBarEvent {
    /// Title `i` clicked (`released`: the click's release, or a screen reader's Collapse), or
    /// opened by a screen reader's Expand.
    Title { i: usize, released: bool },
    /// The pointer over title `i`, no click: while another menu is open, that one's menu opens
    /// instead (Windows' menu bars, and IDM's); otherwise nothing.
    Hover(usize),
    /// The open menu shut itself (Escape, a press outside it, the app losing the focus);
    /// `press_on_title`: the press that shut it went down on a title of the bar.
    Dismissed { press_on_title: bool },
    /// An item was chosen.
    Chosen,
    /// Left or Right, or the keyboard (Alt, F10, an access letter), opened menu `i`.
    Open(usize),
}

/// The menu bar's open menu after `event` (`open`: the one open before it).
///
/// A menu that shut itself is closed here too, unless the press that shut it went down on a
/// title: that press's release is the title's click, which shuts the menu (its own title) or
/// opens another one. Escape, or the app losing the focus, with the pointer resting on the bar
/// (where the click that opened the menu left it) used to leave the menu marked open with
/// nothing shown, so the next click on its title only "shut" it, and nothing opened.
///
/// The pointer alone never opens a menu: it only moves from an open one to another (#277). Once
/// a menu closes in any way (a command chosen, Escape, a click elsewhere, another window taking
/// the focus), the bar waits for a click.
pub fn menubar_next(open: Option<usize>, event: MenuBarEvent) -> Option<usize> {
    match event {
        MenuBarEvent::Hover(i) => open.map(|_| i),
        MenuBarEvent::Title { i, released } => {
            if open == Some(i) && released {
                None
            } else {
                Some(i)
            }
        }
        MenuBarEvent::Dismissed {
            press_on_title: true,
        } => open,
        MenuBarEvent::Dismissed {
            press_on_title: false,
        }
        | MenuBarEvent::Chosen => None,
        MenuBarEvent::Open(i) => Some(i),
    }
}

// ---- popup menus ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum MenuEntry<C> {
    /// Label (with `&` before the mnemonic letter), command, enabled, checked.
    Item(&'static str, C, bool, bool),
    /// Like `Item`, but checked shows a bullet (Win32 radio-check items).
    Radio(&'static str, C, bool, bool),
    /// Label, enabled, children.
    Sub(&'static str, bool, Vec<MenuEntry<C>>),
    Sep,
}

/// A menu label made at run time (a queue's name): each distinct text is kept once for the
/// life of the app, so menus rebuilt every frame don't allocate or leak more than that.
pub fn label(text: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static LABELS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut set = LABELS.get_or_init(Default::default).lock().unwrap();
    if let Some(l) = set.get(text) {
        return l;
    }
    let l: &'static str = Box::leak(text.to_owned().into_boxed_str());
    set.insert(l);
    l
}

const SLOT: f32 = 22.0;
const SEP: f32 = 9.0;

/// A menu title's access letter, lower case ("&Tasks": "t"; none marked: its first letter).
pub fn menu_letter(label: &str) -> String {
    let (shown, at) = plain(label);
    shown
        .chars()
        .nth(at.unwrap_or(0))
        .map(|c| c.to_lowercase().to_string())
        .unwrap_or_default()
}

fn plain(label: &str) -> (String, Option<usize>) {
    let mut out = String::new();
    let mut mnemonic = None;
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'&') {
            // "&&": an "&" in the text (a queue named "Tom & Jerry"), no access key.
            out.push(chars.next().unwrap());
        } else if c == '&' && chars.peek().is_some() {
            mnemonic = Some(out.chars().count());
            out.push(chars.next().unwrap());
        } else {
            out.push(c);
        }
    }
    (out, mnemonic)
}

pub enum PopupResult<C> {
    Open,
    Chosen(C),
    Dismissed,
    /// Left or Right on a menu of the bar: open the menu before (-1) or after (+1) it.
    Neighbor(i32),
}

/// A key or key combination as written in the shortcuts list ("Ctrl+N", "Shift+Delete"): the
/// translated key names.
pub fn key_label(spec: &'static str) -> String {
    crate::i18n::tc("key", spec).to_owned()
}

/// A menu label without the shortcut IDM writes into some labels ("Find (Ctrl-F)"), when the
/// menu shows the shortcut in its own column.
fn without_shortcut(text: &str) -> &str {
    let t = text.trim_end();
    if let Some(open) = t.rfind(" (")
        && t.ends_with(')')
    {
        let inner = &t[open + 2..t.len() - 1];
        let keyish = inner.len() <= 12
            && !inner.contains(' ')
            && (inner.contains('-') || inner.contains('+') || inner.starts_with('F'));
        if keyish {
            return &t[..open];
        }
    }
    t
}

/// Open a menu from the keyboard: its first item is highlighted.
pub fn start_keyboard(ctx: &egui::Context, id: &str) {
    show_cues(ctx);
    let kb = egui::Id::unique(("popup-sub", id)).with("kb");
    ctx.data_mut(|d| d.insert_temp::<(Option<usize>, Option<usize>)>(kb, (Some(usize::MAX), None)));
}

/// Keyboard cues on: the user reached for the keyboard (Alt, F10, a menu opened by keys), so
/// light mode underlines access letters from now on, the Windows rule (dark mode always does).
pub fn show_cues(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("keyboard-cues"), true));
}

/// Whether access letters are underlined: always in dark mode, after [`show_cues`] in light.
pub fn cues(ctx: &egui::Context) -> bool {
    crate::theme::colors().access_keys_always
        || ctx.data(|d| {
            d.get_temp::<bool>(egui::Id::unique("keyboard-cues"))
                .unwrap_or(false)
        })
}

/// The next item of `level` from `from` in direction `dir`, skipping separators (wraps).
fn step<C>(level: &[MenuEntry<C>], from: Option<usize>, dir: i32) -> Option<usize> {
    let n = level.len() as i32;
    if n == 0 {
        return None;
    }
    let mut i = match from {
        Some(f) if f < level.len() => f as i32,
        _ if dir > 0 => -1,
        _ => n,
    };
    for _ in 0..n {
        i = (i + dir).rem_euclid(n);
        if !matches!(level[i as usize], MenuEntry::Sep) {
            return Some(i as usize);
        }
    }
    None
}

fn entry_label<C>(e: &MenuEntry<C>) -> Option<(&'static str, bool)> {
    match e {
        MenuEntry::Item(l, _, en, _) | MenuEntry::Radio(l, _, en, _) | MenuEntry::Sub(l, en, _) => {
            Some((l, *en))
        }
        MenuEntry::Sep => None,
    }
}

/// The keys an open menu takes from `i` (without Ctrl or Command: those are shortcuts).
fn menu_keys(i: &egui::InputState) -> Vec<egui::Key> {
    i.events
        .iter()
        .filter_map(|e| match e {
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } if !modifiers.command && !modifiers.ctrl => Some(*key),
            _ => None,
        })
        .collect()
}

/// Keys for an open menu (pressed in the main window or in the menu's own windows): Up and Down
/// move the highlight (Home / End to the first / last), Right opens a submenu or the next menu, Left closes a submenu or opens
/// the previous menu, Enter or an item's underlined letter runs it, Escape closes a submenu.
fn keyboard<C: Clone>(
    ctx: &egui::Context,
    entries: &[MenuEntry<C>],
    sub_key: egui::Id,
) -> Option<PopupResult<C>> {
    let kb_key = sub_key.with("kb");
    let mut keys = ctx.input(menu_keys);
    // Pressed in the menu's own windows: popup_level noted them as those windows drew.
    keys.extend(
        ctx.data_mut(|d| d.remove_temp::<Vec<egui::Key>>(sub_key.with("window-keys")))
            .unwrap_or_default(),
    );
    let (mut hl0, mut hl1): (Option<usize>, Option<usize>) =
        ctx.data(|d| d.get_temp(kb_key)).unwrap_or_default();
    if hl0 == Some(usize::MAX) {
        hl0 = step(entries, None, 1);
        ctx.data_mut(|d| d.insert_temp(kb_key, (hl0, hl1)));
    }
    if keys.is_empty() {
        return None;
    }
    // The keys are the menu's: the list underneath doesn't move too.
    ctx.input_mut(|i| {
        for k in &keys {
            i.consume_key(egui::Modifiers::NONE, *k);
        }
    });
    let mut open_sub = ctx.data(|d| d.get_temp::<usize>(sub_key));
    let mut out = None;
    for key in keys {
        let children: Option<&[MenuEntry<C>]> = open_sub.and_then(|i| match entries.get(i) {
            Some(MenuEntry::Sub(_, _, c)) => Some(c.as_slice()),
            _ => None,
        });
        let in_sub = children.is_some() && hl1.is_some();
        let level = if in_sub {
            children.unwrap_or(&[])
        } else {
            entries
        };
        let hl = if in_sub { hl1 } else { hl0 };
        let mut target = None;
        match key {
            egui::Key::ArrowDown | egui::Key::ArrowUp => {
                let n = step(level, hl, if key == egui::Key::ArrowDown { 1 } else { -1 });
                if in_sub { hl1 = n } else { hl0 = n }
            }
            // Home / End: the first / last item, as Windows' menus have them.
            egui::Key::Home | egui::Key::End => {
                let n = step(level, None, if key == egui::Key::Home { 1 } else { -1 });
                if in_sub { hl1 = n } else { hl0 = n }
            }
            egui::Key::Escape if in_sub => {
                open_sub = None;
                hl1 = None;
            }
            egui::Key::Escape => out = Some(PopupResult::Dismissed),
            egui::Key::ArrowLeft if in_sub => {
                open_sub = None;
                hl1 = None;
            }
            egui::Key::ArrowLeft => out = Some(PopupResult::Neighbor(-1)),
            egui::Key::ArrowRight => match hl.and_then(|h| level.get(h)) {
                Some(MenuEntry::Sub(_, true, c)) if !in_sub => {
                    open_sub = hl;
                    hl1 = step(c, None, 1);
                }
                _ => out = Some(PopupResult::Neighbor(1)),
            },
            egui::Key::Enter | egui::Key::Space => target = hl,
            other => {
                let name = other.name().to_lowercase();
                if name.chars().count() == 1 {
                    target = level.iter().position(|e| {
                        entry_label(e).is_some_and(|(l, en)| {
                            let (t, m) = plain(crate::i18n::tc("menu", l));
                            en && m
                                .and_then(|m| t.chars().nth(m))
                                .is_some_and(|c| c.to_lowercase().to_string() == name)
                        })
                    });
                }
            }
        }
        if let Some(t) = target {
            if in_sub {
                hl1 = Some(t)
            } else {
                hl0 = Some(t)
            }
            match level.get(t) {
                Some(MenuEntry::Item(_, c, true, _) | MenuEntry::Radio(_, c, true, _)) => {
                    out = Some(PopupResult::Chosen(c.clone()));
                }
                Some(MenuEntry::Sub(_, true, c)) if !in_sub => {
                    open_sub = Some(t);
                    hl1 = step(c, None, 1);
                }
                _ => {}
            }
        }
        if out.is_some() {
            break;
        }
    }
    ctx.data_mut(|d| {
        d.insert_temp(kb_key, (hl0, hl1));
        match open_sub {
            Some(i) => {
                d.insert_temp(sub_key, i);
            }
            None => {
                d.remove::<usize>(sub_key);
            }
        }
    });
    out
}

/// What a popup menu is: how its window says so to the window system (X11's window types) and,
/// on X11, whether it shares its window with others ([`menu_viewport`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuKind {
    /// A menu of the menu bar, and its submenus.
    Bar,
    /// A context menu: a right click, the Menu key, a toolbar button's drop-down.
    Context,
    /// A drop-down list's choices: a combo box's, a text box's history.
    List,
}

/// Show a popup menu with its top-left at `pos` (main-window client coordinates, on screen) —
/// its top-right in a right-to-left interface, where menus open to the left.
///
/// Like a Win32 menu, each level is its own borderless top-level window (an immediate egui
/// viewport): it can extend past the main window, flips to stay on the monitor, never takes
/// activation (so Escape still reaches the main window), and clicks on it never reach the
/// widgets underneath. Call every frame while open, from the main window's `ui`.
pub fn popup<C: Clone>(
    ctx: &egui::Context,
    kind: MenuKind,
    id: &str,
    pos: Pos2,
    entries: &[MenuEntry<C>],
    hint: &dyn Fn(&C) -> Option<&'static str>,
) -> PopupResult<C> {
    popup_with_icons(ctx, kind, id, pos, entries, hint, &|_, _| None)
}

/// [popup], with the icon a Graphite menu draws at each item's start: `icon` gives it for an
/// item from its command and its label (a submenu's from its label, [menu_icon]).
#[allow(clippy::too_many_arguments)]
pub fn popup_with_icons<C: Clone>(
    ctx: &egui::Context,
    kind: MenuKind,
    id: &str,
    pos: Pos2,
    entries: &[MenuEntry<C>],
    hint: &dyn Fn(&C) -> Option<&'static str>,
    icon: &dyn Fn(&C, &str) -> Option<&'static str>,
) -> PopupResult<C> {
    let sub_key = egui::Id::unique(("popup-sub", id));
    // The control that has the keys while the menu is open (a drop-down list, a text box): the
    // menu's windows never take them, so Escape reached its window, and egui let them go there
    // (Escape drops the focus). Escape closing the menu gives them back.
    let opener = sub_key.with("opener");
    let focused_now = ctx.memory(|m| m.focused());
    if let Some(f) = focused_now {
        ctx.data_mut(|d| d.insert_temp(opener, f));
    }
    let give_back = |ctx: &egui::Context| {
        let had = ctx.data_mut(|d| {
            let had = d.get_temp::<egui::Id>(opener);
            d.remove::<egui::Id>(opener);
            had
        });
        if let Some(had) = had
            && focused_now.is_none()
            && ctx.input(|i| i.key_pressed(egui::Key::Escape))
        {
            ctx.memory_mut(|m| m.request_focus(had));
        }
    };
    if let Some(r) = keyboard(ctx, entries, sub_key) {
        ctx.data_mut(|d| {
            d.remove::<usize>(sub_key);
            d.remove::<(Option<usize>, Option<usize>)>(sub_key.with("kb"));
        });
        give_back(ctx);
        return r;
    }
    let origin = ctx
        .input(|i| i.viewport().inner_rect)
        .map_or(Pos2::ZERO, |r| r.min);
    let mut result = None;
    // Set by the menu's windows as they draw (popup_level) when one of them has the focus.
    let window_focus = sub_key.with("window-focus");
    ctx.data_mut(|d| d.insert_temp(window_focus, false));
    popup_level(
        ctx,
        kind,
        id,
        origin + pos.to_vec2(),
        None,
        entries,
        sub_key,
        &mut result,
        0,
        hint,
        icon,
    );
    if let Some(c) = result {
        ctx.data_mut(|d| {
            d.remove::<usize>(sub_key);
            d.remove::<(Option<usize>, Option<usize>)>(sub_key.with("kb"));
        });
        give_back(ctx);
        return PopupResult::Chosen(c);
    }
    // The menus are other windows: any press that reaches the main window, Escape, or the
    // app losing focus closes them (as Win32 menus do) — except on the frame the menu opened,
    // whose press may be the very click that opened it.
    let pass = ctx.cumulative_pass_nr();
    let shown = sub_key.with("shown");
    // Focus in the menu's own windows is still ours: a click in a menu window may give that
    // window the focus, and must not count as clicking away.
    let focused = ctx.input(|i| i.viewport().focused) == Some(true)
        || ctx.data(|d| d.get_temp::<bool>(window_focus).unwrap_or(false));
    // Away from our windows, Windows says who has it now: another program (upstream issue 859).
    let elsewhere = !focused && crate::win32::foreground_is_another_program();
    // (first pass, last pass, the focus): focus is only *lost* if it was had, and then at once
    // when another program has it, else when no window of ours gets it back within FOCUS_GRACE
    // (#250).
    let now = ctx.input(|i| i.time);
    let (first, verdict) = ctx.data_mut(|d| {
        let (first, last, seen) = d.get_temp::<(u64, u64, MenuFocus)>(shown).unwrap_or((
            pass,
            pass,
            MenuFocus::default(),
        ));
        let (first, mut seen) = if last + 1 < pass {
            (pass, MenuFocus::default())
        } else {
            (first, seen)
        };
        let verdict = seen.pass(focused, elsewhere, now);
        d.insert_temp(shown, (first, pass, seen));
        (first == pass, verdict)
    });
    if let FocusVerdict::Away { wait } = verdict {
        // No event may come to look again (the pointer resting on the menu).
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait));
    }
    let lost_focus = verdict == FocusVerdict::Left;
    if !first
        && (lost_focus
            || ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.pointer.any_pressed()))
    {
        // Closed by a press: where it was, so the control that opened it doesn't take that
        // press's click (on release) as opening it again (winui::opener_click). Read before
        // the data lock (asking the context inside it deadlocks).
        let press = ctx.input(|i| i.pointer.press_origin().filter(|_| i.pointer.any_pressed()));
        let dismissed = dismissed_at_key(ctx);
        ctx.data_mut(|d| {
            d.remove::<usize>(sub_key);
            d.remove::<(Option<usize>, Option<usize>)>(sub_key.with("kb"));
            if let Some(at) = press {
                d.insert_temp(dismissed, at);
            }
        });
        give_back(ctx);
        return PopupResult::Dismissed;
    }
    PopupResult::Open
}

/// How long the focus may be away from every window of ours before an open menu counts as left,
/// when Windows doesn't say that another program has it
/// ([`crate::win32::foreground_is_another_program`]). A menu's window may get the focus as it opens. When a submenu closes (the pointer back on
/// another item of its parent), its window passes the focus on, and for a frame or more no
/// window of ours has it: counted as the app losing the focus, it closed the whole menu (#250).
const FOCUS_GRACE: f64 = 0.4;

/// Whether an open menu still has the focus: the main window's or one of the menu's windows'.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MenuFocus {
    /// One of ours has had the focus since the menu opened.
    had: bool,
    /// Since when none of ours has had it (the app's clock, in seconds).
    away_since: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FocusVerdict {
    /// Ours, or never had (a menu opened while another app was in front).
    Here,
    /// Gone for less than FOCUS_GRACE: look again in `wait` seconds.
    Away { wait: f64 },
    /// Gone for FOCUS_GRACE: another app has it, the menu closes.
    Left,
}

impl MenuFocus {
    /// After a pass at `now`: `focused` is whether one of our windows had the focus in it, and
    /// `elsewhere` whether Windows said another program has it.
    fn pass(&mut self, focused: bool, elsewhere: bool, now: f64) -> FocusVerdict {
        if focused {
            self.had = true;
            self.away_since = None;
            return FocusVerdict::Here;
        }
        if !self.had {
            return FocusVerdict::Here;
        }
        // Another program has it: no window of ours is about to take it back (upstream issue 859).
        if elsewhere {
            return FocusVerdict::Left;
        }
        let since = *self.away_since.get_or_insert(now);
        let away = now - since;
        if away >= FOCUS_GRACE {
            FocusVerdict::Left
        } else {
            FocusVerdict::Away {
                wait: FOCUS_GRACE - away,
            }
        }
    }
}

/// Where the last press that closed a popup went down, in this window.
pub fn dismissed_at_key(ctx: &egui::Context) -> egui::Id {
    egui::Id::unique(("popup-dismissed-at", ctx.viewport_id()))
}

/// Open a menu from the keyboard with item `i` highlighted (a drop-down list on its choice).
pub fn start_at(ctx: &egui::Context, id: &str, i: usize) {
    show_cues(ctx);
    let kb = egui::Id::unique(("popup-sub", id)).with("kb");
    ctx.data_mut(|d| d.insert_temp::<(Option<usize>, Option<usize>)>(kb, (Some(i), None)));
}

/// Open a menu with its entry `i`'s submenu open and the submenu's first entry highlighted, as
/// the keyboard leaves it (`/show menu:torrent`, for screenshots).
pub fn open_sub_at(ctx: &egui::Context, id: &str, i: usize) {
    show_cues(ctx);
    let sub_key = egui::Id::unique(("popup-sub", id));
    ctx.data_mut(|d| {
        d.insert_temp(sub_key, i);
        d.insert_temp::<(Option<usize>, Option<usize>)>(sub_key.with("kb"), (Some(i), Some(0)));
    });
}

/// The surface of a popup drawn inside a window (the date picker's calendar, a column's filter
/// box): the menus' colours and border, rounded as Windows 11's flyouts (they had their own).
pub fn popup_surface(p: &egui::Painter, rc: egui::Rect) {
    let radius = crate::theme::metrics().flyout_radius;
    p.rect_filled(rc, radius, POPUP_BG());
    p.rect_stroke(
        rc,
        radius,
        egui::Stroke::new(1.0, POPUP_BORDER()),
        egui::StrokeKind::Inside,
    );
}

/// A menu label's width as Win32 sizes menus: GDI's whole-pixel width (egui's advances differ
/// by up to a pixel either way: IDM's menus are 236, 291 wide where egui gives 237, 290), or the
/// width it is drawn at when that is wider by more (a script drawn in a fallback font, such as
/// Japanese).
fn menu_width(p: &egui::Painter, label: &str) -> f32 {
    let drawn = text_width_font(p, winui::segoe(), label).ceil();
    {
        let gdi = crate::win32::menu_text_width(label) as f32;
        if drawn > gdi + 1.0 { drawn } else { gdi }
    }
}

/// How a menu's window is laid out: Graphite's (plan 05b, the mockup's `.menu-pop`) for the
/// menu bar's and the right-click menus, Win32's for drop-down lists until the dialogs get
/// theirs (plan 07).
#[derive(Clone, Copy, Debug, PartialEq)]
struct MenuLook {
    graphite: bool,
    /// Above the first slot and below the last; a slot's inset from the sides.
    pad: f32,
    /// An item's height, and a separator's.
    slot: f32,
    sep: f32,
    /// Where an item's label starts.
    text_x: f32,
    /// The window's least width.
    min_w: f32,
}

impl MenuLook {
    fn of(kind: MenuKind) -> MenuLook {
        match kind {
            MenuKind::Bar | MenuKind::Context => MenuLook {
                graphite: true,
                pad: 5.0,
                slot: 28.0,
                sep: 11.0,
                text_x: 39.0,
                min_w: if kind == MenuKind::Bar { 258.0 } else { 240.0 },
            },
            _ => MenuLook {
                graphite: false,
                pad: 3.0,
                slot: SLOT,
                sep: SEP,
                text_x: 38.0,
                min_w: 0.0,
            },
        }
    }
}

/// The icon a Graphite menu draws at a submenu's start, or at a greyed placeholder's (an item
/// with no command, which [`popup_with_icons`]'s caller can't tell by its command), by its
/// label: the English text the menus are built with, all the app's own (a queue's name, a
/// user's word, never reaches here). The mockup's (`MENUS` and the right-click menus in
/// `design/graphite/source/graphite.html`). None for the rest.
pub fn menu_icon(label: &str) -> Option<&'static str> {
    Some(match label {
        // Submenus.
        "E&xport" => "gx-upload",
        "&Import" => "gx-download",
        "Start &queue" | "Start now" => "tb-start-queue",
        "S&top queue" | "Stop" => "tb-stop-queue",
        "Speed &Limiter" => "gx-gauge",
        "Add to queue" | "Move to queue" => "cat-queues",
        "On Double click" => "gx-pointer",
        // A category's and a queue's greyed items.
        "Browse" => "cat-folder-open",
        "Properties" => "gx-info",
        "Delete" => "tb-delete",
        "Edit queue" => "gx-edit",
        "Schedule" => "tb-scheduler",
        _ => return None,
    })
}

/// Size of a menu window. IDM's: the check column (38), the widest label, then 30 to the edge
/// (48 when an item opens a submenu: its arrow), slots of 22 (separators 9), 3 px top and
/// bottom. Graphite's: the icon column (39), the widest label with its shortcut (18 before it)
/// or its submenu's chevron, 14 to the edge, at least 240 wide (258 for the menu bar's); slots
/// of 28 (separators 11), 5 top and bottom. Fitted to its texts, so they don't float in a wide
/// window.
fn menu_size<C>(
    ctx: &egui::Context,
    look: MenuLook,
    entries: &[MenuEntry<C>],
    hint: &dyn Fn(&C) -> Option<&'static str>,
) -> egui::Vec2 {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let width = |t: &str| menu_width(&painter, t);
    if look.graphite {
        let widest = entries
            .iter()
            .filter_map(|e| match e {
                MenuEntry::Item(l, c, ..) | MenuEntry::Radio(l, c, ..) => Some(match hint(c) {
                    Some(k) => {
                        width(without_shortcut(&plain(crate::i18n::tc("menu", l)).0))
                            + 18.0
                            + text_width_font(&painter, key_font(), &key_label(k)).ceil()
                    }
                    None => width(&plain(crate::i18n::tc("menu", l)).0),
                }),
                MenuEntry::Sub(l, ..) => {
                    Some(width(&plain(crate::i18n::tc("menu", l)).0) + 18.0 + 13.0)
                }
                MenuEntry::Sep => None,
            })
            .fold(0.0, f32::max);
        let h: f32 = entries
            .iter()
            .map(|e| {
                if matches!(e, MenuEntry::Sep) {
                    look.sep
                } else {
                    look.slot
                }
            })
            .sum();
        return vec2(
            (look.text_x + widest + 14.0).max(look.min_w),
            h + 2.0 * look.pad,
        );
    }
    let widest = entries
        .iter()
        .filter_map(|e| match e {
            MenuEntry::Item(l, c, ..) | MenuEntry::Radio(l, c, ..) => Some(match hint(c) {
                Some(k) => {
                    width(without_shortcut(&plain(crate::i18n::tc("menu", l)).0))
                        + 28.0
                        + text_width_font(&painter, winui::segoe(), &key_label(k)).ceil()
                }
                None => width(&plain(crate::i18n::tc("menu", l)).0),
            }),
            MenuEntry::Sub(l, ..) => Some(width(&plain(crate::i18n::tc("menu", l)).0)),
            MenuEntry::Sep => None,
        })
        .fold(0.0, f32::max);
    let h: f32 = entries
        .iter()
        .map(|e| {
            if matches!(e, MenuEntry::Sep) {
                SEP
            } else {
                SLOT
            }
        })
        .sum();
    let has_sub = entries.iter().any(|e| matches!(e, MenuEntry::Sub(..)));
    vec2(38.0 + widest + if has_sub { 48.0 } else { 30.0 }, h + 6.0)
}

/// How a borderless window of ours looks and takes the pointer ([`style_popup`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PopupLook {
    /// A menu: never activated (the main window keeps the focus), small rounded corners and
    /// the menu's border, no open animation.
    Menu,
    /// A small floating window (the drop target): activated as any window, rounded corners.
    Floating,
}

/// The builder of every borderless window of ours (a menu, the drop target): no title bar, not
/// resizable, no taskbar button, always on top. On Windows it is created hidden: its first pass
/// makes it a plain popup ([`style_popup`]) and [`show_popup`] shows it then.
pub fn popup_viewport(title: &str) -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title(title)
        .with_decorations(false)
        .with_resizable(false)
        .with_taskbar(false)
        .with_always_on_top()
        // Hidden until styled, then shown (show_popup).
        .with_visible(false)
}

/// Call first in the pass of a borderless window titled `title` (made with [`popup_viewport`]),
/// before anything is drawn in it. It becomes a plain popup window, its client the
/// whole window (win32::popup_window): winit's borderless windows keep a caption style, which
/// on a Windows 11 Insider build (22635) put a black band a title bar tall across the top of
/// every menu and of the drop target, their picture moved down by as much (#208). Styled in
/// the window's own pass: its frame changes, and a picture drawn before that was lost with the
/// old frame (black until something made it draw again). True on the pass that styled it: show
/// it then ([`show_popup`]).
pub fn style_popup(title: &str, look: PopupLook) -> bool {
    use crate::win32::Popup;
    let kind = match look {
        PopupLook::Menu => Popup::Menu {
            border: u32::from_le_bytes([
                POPUP_BORDER().r(),
                POPUP_BORDER().g(),
                POPUP_BORDER().b(),
                0,
            ]),
        },
        PopupLook::Floating => Popup::Floating,
    };
    crate::win32::popup_window(title, kind)
}

/// Show the borderless window `vid` [`style_popup`] just styled (from its parent's pass).
pub fn show_popup(ctx: &egui::Context, vid: egui::ViewportId) {
    ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Visible(true));
    // The command is carried out on the window's next frame: a menu opened from the keyboard
    // (Shift+F10, the Menu key) or a screen reader moves no pointer to bring one, and stayed
    // hidden (#113).
    ctx.request_repaint_of(vid);
    ctx.request_repaint();
}

/// Place a menu of `size` at screen point `at`, flipped to stay inside the monitor's work area:
/// up (or left) of the point, or for a submenu (`parent`: the parent menu's screen rect) to the
/// parent's left side. All in points: the
/// work area too (it was taken in pixels, which differ at a display or interface scale other
/// than 100 %, #262).
fn place_menu(ctx: &egui::Context, at: Pos2, size: egui::Vec2, parent: Option<Rect>) -> Pos2 {
    // An off-screen test copy (MDM_OFFSCREEN) keeps its menus beside its window, off the
    // screens too (scale::work_area has none there): the nearest monitor's work area brought
    // them onto the screen, over the user's work.
    if let Some(area) = crate::scale::work_area(ctx, at) {
        let (l, t, r, b) = (area.min.x, area.min.y, area.max.x, area.max.y);
        let mut p = at;
        if p.x + size.x > r {
            p.x = match parent {
                Some(pr) => pr.left() - size.x + 4.0,
                None => at.x - size.x,
            };
        }
        if p.y + size.y > b {
            p.y = match parent {
                Some(_) => b - size.y,
                None => at.y - size.y,
            };
        }
        return pos2(p.x.max(l), p.y.max(t));
    }
    let _ = parent;
    at
}

/// The window menu `id`'s level `depth` is drawn in. With `bar_shared`
/// every menu of the bar is drawn in one window per level, moved and resized as the pointer or
/// the arrow keys go from title to title, as a submenu's window already is from item to item.
/// A window of its own for each menu was mapped and unmapped at every switch, and a compositor
/// that animates new windows (Hyprland, picom with fading) faded each one in, and the last one
/// out, every time (#277). Other menus: a window of their own, as before.
fn menu_viewport(kind: MenuKind, id: &str, depth: usize, bar_shared: bool) -> egui::ViewportId {
    let key = if bar_shared && kind == MenuKind::Bar {
        "menu-bar"
    } else {
        id
    };
    egui::ViewportId::from_hash_of(("menu", key, depth))
}

/// The entry under `p` in a menu drawn at `o`, `w` wide: its index, none on a separator or the
/// padding (the slots as `popup_level` lays them out, in the menu's `look`: Graphite's rows are
/// taller and inset by its padding, the other looks' 22 high, 6 in).
fn item_at<C>(
    entries: &[MenuEntry<C>],
    look: &MenuLook,
    o: Pos2,
    w: f32,
    p: Pos2,
) -> Option<usize> {
    let inset = if look.graphite { look.pad } else { 6.0 };
    let mut y = o.y + look.pad;
    for (i, e) in entries.iter().enumerate() {
        if matches!(e, MenuEntry::Sep) {
            y += look.sep;
            continue;
        }
        let slot = Rect::from_min_max(pos2(o.x + inset, y), pos2(o.x + w - inset, y + look.slot));
        if slot.contains(p) {
            return Some(i);
        }
        y += look.slot;
    }
    None
}

/// Whether the pointer at `p`, having left an open submenu's entry at `from`, is on its way into
/// the submenu at `sub` (#579): inside the triangle from `from` to the submenu's near edge, with
/// a pixel's slack. A pointer going up or down the menu instead is not.
fn heading_into(from: Pos2, p: Pos2, sub: Rect) -> bool {
    let near = if sub.center().x >= from.x {
        sub.left()
    } else {
        sub.right()
    };
    let (a, b, c) = (from, pos2(near, sub.top()), pos2(near, sub.bottom()));
    let side = |p1: Pos2, p2: Pos2| (p2.x - p1.x) * (p.y - p1.y) - (p2.y - p1.y) * (p.x - p1.x);
    let (d1, d2, d3) = (side(a, b), side(b, c), side(c, a));
    let slack = 1.0 * (b - a).length().max(1.0);
    let neg = d1 < -slack || d2 < -slack || d3 < -slack;
    let pos = d1 > slack || d2 > slack || d3 > slack;
    !(neg && pos)
}

#[allow(clippy::too_many_arguments)]
fn popup_level<C: Clone>(
    ctx: &egui::Context,
    kind: MenuKind,
    id: &str,
    at: Pos2,
    parent: Option<Rect>,
    entries: &[MenuEntry<C>],
    sub_key: egui::Id,
    result: &mut Option<C>,
    depth: usize,
    hint: &dyn Fn(&C) -> Option<&'static str>,
    icon: &dyn Fn(&C, &str) -> Option<&'static str>,
) {
    let look = MenuLook::of(kind);
    let g = look.graphite;
    let size = menu_size(ctx, look, entries, hint);
    let kb_key = sub_key.with("kb");
    let kb: Option<usize> = ctx
        .data(|d| d.get_temp::<(Option<usize>, Option<usize>)>(kb_key))
        .and_then(|(a, b)| if depth == 0 { a } else { b });
    let screen = Rect::from_min_size(place_menu(ctx, at, size, parent), size);
    if depth == 1 {
        // Where the submenu is, for its menu's next pass (`heading_into`, #579).
        ctx.data_mut(|d| d.insert_temp(sub_key.with("sub-rect"), screen));
    }
    let title = format!("misterdownloadmanager menu {depth}"); // not translated: an id
    let builder = popup_viewport(&title)
        .with_active(false)
        .with_position(screen.min)
        .with_inner_size(size);
    let open_sub = if depth == 0 {
        ctx.data(|d| d.get_temp::<usize>(sub_key))
    } else {
        None
    };
    // (index, slot top) of the submenu to show beside this menu.
    let mut sub_at: Option<(usize, f32)> = None;
    // One window per menu (`bar_shared` off).
    let vid = menu_viewport(kind, id, depth, false);
    let parent_vid = ctx.viewport_id();
    // Set when this pass made the new (hidden) window a menu window: shown below. (Elsewhere a
    // menu's window is created visible.)
    let mut styled = false;
    ctx.show_viewport_immediate(vid, builder, |ui, _| {
        // Made a menu window in its own pass, before anything is drawn in it (style_popup).
        styled = style_popup(&title, PopupLook::Menu);
        // What happens in this window, for popup() and keyboard(), which run in the parent's
        // pass: they must not ask egui about a menu window by its id, which creates one when it
        // isn't open (a submenu, mostly): an untitled copy of the main window, opened and closed
        // every frame (issues #4 and #5: fading windows; on a full-screen Mac, a new space).
        let (focused, keys) = ui.input(|i| (i.viewport().focused == Some(true), menu_keys(i)));
        ui.ctx().data_mut(|d| {
            if focused {
                d.insert_temp(sub_key.with("window-focus"), true);
            }
            if !keys.is_empty() {
                d.get_temp_mut_or_default::<Vec<egui::Key>>(sub_key.with("window-keys"))
                    .extend(keys.iter().copied());
            }
        });
        if !keys.is_empty() {
            // keyboard() takes them on the parent's next pass.
            ui.ctx().request_repaint_of(parent_vid);
        }
        let w = size.x;
        let painter = ui.painter().clone();
        let o = ui.max_rect().min;
        // Where the items are, for the tests that point at them.
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(sub_key.with(("origin", depth)), o));
        // Right to left: text right-aligned, check marks and bullets on the right, the submenu
        // arrow on the left pointing left.
        let _surface = winui::Surface::enter(o.x, o.x + w);
        let rtl = winui::rtl();
        // Body; Windows 11 rounds the window and draws the border and shadow (style_popup).
        fill(
            &painter,
            o.x,
            o.y,
            o.x + w - 1.0,
            o.y + size.y - 1.0,
            POPUP_BG(),
        );
        // Where the pointer is, or was on this pass: a tap on a touch screen ends with the pointer
        // gone (egui-winit lets go of it as the finger lifts, so nothing stays lit), in the very
        // pass that releases it, where the hover position is none and no item was chosen (#733).
        let pointer = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
        // The item under the pointer (none on a separator or the padding): one item is lit at a
        // time, so an open submenu's entry gives up its light to it (#579).
        let under = pointer.and_then(|p| item_at(entries, &look, o, w, p));
        // An open submenu stays open while the pointer is on another item only as long as it
        // heads into the submenu (#579): from where it left the submenu's entry, inside the
        // triangle to the submenu's near edge. Its screen rect, from the pass that drew it, in
        // this window's points.
        let anchor_key = sub_key.with("anchor");
        let sub_rect = (depth == 0)
            .then(|| {
                ui.ctx()
                    .data(|d| d.get_temp::<Rect>(sub_key.with("sub-rect")))
            })
            .flatten()
            .map(|r| r.translate(o.to_vec2() - screen.min.to_vec2()));
        let clicked = ui.input(|i| i.pointer.primary_released());
        // The pointer moving over an item takes the highlight from the keyboard.
        let moving = ui.input(|i| i.pointer.is_moving());
        // For screen readers and UI Automation (#113): the window is a menu, its entries its
        // items (made in a child whose accessibility parent is the menu); invoking one chooses
        // it, as a click does.
        let menu_node = ui.interact(
            Rect::from_min_size(o, size),
            egui::Id::unique(("menu-node", id, depth)),
            egui::Sense::hover(),
        );
        winui::expose(&menu_node, egui::Role::Menu, "", true, None);
        let items_ui = ui.new_child(egui::UiBuilder::new().accessibility_parent(menu_node.id));
        let mut y = o.y + look.pad;
        // Graphite's rows are 6 taller: what is centred in a row moves down by half of it.
        let (dy, inset) = if g { (3.0, look.pad) } else { (0.0, 6.0) };
        for (i, e) in entries.iter().enumerate() {
            if matches!(e, MenuEntry::Sep) {
                if g {
                    // The mockup's `hr`: the menu's border colour, 4 in from the slots.
                    winui::fill_l(
                        &painter,
                        o.x + inset + 4.0,
                        y + 5.0,
                        o.x + w - inset - 5.0,
                        y + 5.0,
                        POPUP_BORDER(),
                    );
                } else {
                    winui::fill_l(
                        &painter,
                        o.x + 9.0,
                        y + 4.0,
                        o.x + w - 10.0,
                        y + 4.0,
                        POPUP_SEP(),
                    );
                }
                y += look.sep;
                continue;
            }
            let (label, enabled, checked) = match e {
                MenuEntry::Item(l, _, en, ch) | MenuEntry::Radio(l, _, en, ch) => (*l, *en, *ch),
                MenuEntry::Sub(l, en, _) => (*l, *en, false),
                MenuEntry::Sep => unreachable!(),
            };
            let slot =
                Rect::from_min_max(pos2(o.x + inset, y), pos2(o.x + w - inset, y + look.slot));
            let hovered = pointer.is_some_and(|p| slot.contains(p));
            if hovered && moving && kb.is_some() && kb != Some(i) {
                ui.ctx().data_mut(|d| {
                    let (a, b) = d
                        .get_temp::<(Option<usize>, Option<usize>)>(kb_key)
                        .unwrap_or_default();
                    d.insert_temp(
                        kb_key,
                        if depth == 0 {
                            (Some(i), b)
                        } else {
                            (a, Some(i))
                        },
                    );
                });
            }
            let lit = match kb {
                Some(k) => k == i,
                None => hovered,
            };
            // An open submenu's entry stays lit while the pointer is on it, in the submenu or
            // off this menu; not while it is on another item, which is the lit one then (#579).
            let opened = open_sub == Some(i) && under.is_none_or(|u| u == i);
            if lit || opened {
                painter.rect_filled(
                    slot,
                    crate::theme::metrics().menu_item_radius,
                    POPUP_HOVER(),
                );
            } else if !g {
                // Spec: an item's normal background is one level lighter than the padding.
                fill(
                    &painter,
                    slot.left(),
                    y,
                    slot.right() - 1.0,
                    y + SLOT - 1.0,
                    POPUP_ITEM(),
                );
            }
            let color = if enabled {
                TEXT()
            } else {
                TEXT_DISABLED_MENU()
            };
            // Bullets, check marks and arrows (light mode: dark cores, not pure black).
            let glyph = if enabled {
                GLYPH()
            } else {
                TEXT_DISABLED_MENU()
            };
            let (t, mn) = plain(crate::i18n::tc("menu", label));
            let keys = match e {
                MenuEntry::Item(_, c, ..) | MenuEntry::Radio(_, c, ..) => hint(c),
                _ => None,
            };
            let t = if keys.is_some() {
                without_shortcut(&t).to_owned()
            } else {
                t
            };
            // The entry's node: a menu item (a check or radio item when it is one), its name,
            // whether it can be used, and for a submenu that it opens one.
            let item = items_ui.interact(
                slot,
                egui::Id::unique(("menu-item", id, depth, i)),
                winui::pointer_only(),
            );
            let (role, on) = match e {
                MenuEntry::Radio(..) => (egui::Role::MenuItemRadio, Some(checked)),
                MenuEntry::Item(..) if checked => (egui::Role::MenuItemCheckBox, Some(true)),
                _ => (egui::Role::MenuItem, None),
            };
            winui::expose(&item, role, &t, enabled, on);
            if let MenuEntry::Sub(..) = e {
                let open = open_sub == Some(i);
                ui.ctx().accesskit_node_builder(item.id, |n| {
                    n.set_has_popup(egui::accesskit::HasPopup::Menu);
                    n.set_expanded(open);
                });
            }
            // Invoked by a screen reader or UI Automation (a real click is the pointer's,
            // below): an item is chosen, a submenu opens.
            // (A submenu's entry is opened with Expand too, its pattern being Expand / Collapse.)
            let expand_sub = matches!(e, MenuEntry::Sub(..))
                && ui.input(|x| {
                    x.has_accesskit_action_request(item.id, egui::accesskit::Action::Expand)
                });
            let invoked = (item.clicked() && !item.clicked_by(egui::PointerButton::Primary)
                || expand_sub)
                && enabled;
            match e {
                MenuEntry::Item(_, c, ..) | MenuEntry::Radio(_, c, ..) if invoked => {
                    *result = Some(c.clone());
                    ui.ctx().request_repaint_of(parent_vid);
                }
                MenuEntry::Sub(..) if invoked && depth == 0 => {
                    ui.ctx().data_mut(|d| d.insert_temp(sub_key, i));
                    sub_at = Some((i, y - o.y));
                    ui.ctx().request_repaint_of(parent_vid);
                }
                _ => {}
            }
            let baseline = y + 14.0 + if g { 4.0 } else { 0.0 };
            let drawn = text_font(
                &painter,
                winui::segoe(),
                o.x + look.text_x,
                baseline,
                &t,
                color,
            );
            // The shortcut at the trailing edge, in the menu's quieter ink (Graphite's own
            // role for it, `menu.shortcut`, in its smaller size).
            if let Some(k) = keys {
                let key = key_label(k);
                if g {
                    let ink = crate::theme::colors().menu_shortcut;
                    let x = o.x + w - 14.0 - text_width_font(&painter, key_font(), &key);
                    text_font(&painter, key_font(), x, baseline, &key, ink);
                } else {
                    winui::text_right(
                        &painter,
                        o.x + w - 24.0,
                        y + 14.0,
                        &key,
                        TEXT_DISABLED_MENU(),
                    );
                }
            }
            // Underlined mnemonics: always in dark mode, once the keyboard is in use in light.
            if let Some(m) = mn
                && cues(ctx)
            {
                let (x0, x1) = winui::char_span(&painter, &winui::segoe(), &t, m, drawn);
                fill(&painter, x0, baseline + 2.0, x1, baseline + 2.0, color);
            }
            // Graphite: the item's icon at its start (16, the icon column 14..30), where it
            // has one and no check mark takes the column.
            let marked = checked || matches!(e, MenuEntry::Radio(..));
            if g && !marked
                && let Some(need) = match e {
                    MenuEntry::Item(l, c, ..) | MenuEntry::Radio(l, c, ..) => icon(c, l),
                    MenuEntry::Sub(l, ..) => menu_icon(l),
                    MenuEntry::Sep => None,
                }
            {
                let ink = if enabled {
                    crate::theme::colors().text_secondary
                } else {
                    TEXT_DISABLED_MENU()
                };
                let rc = winui::rect(Rect::from_min_size(
                    pos2(o.x + 14.0, y + 6.0),
                    vec2(16.0, 16.0),
                ));
                crate::themeicons::paint_in(ui, &painter, need, rc, ink, enabled);
            }
            // Graphite's check marks in the accent (the mockup's), IDM's in the glyphs' ink.
            let glyph = if g && enabled {
                crate::theme::colors().accent
            } else {
                glyph
            };
            // The check box column: x 12..21 from the leading edge (the glyphs keep their
            // shape); Graphite's icon column, 14..30.
            let gx = |x: f32| {
                let x = if g { x + 6.0 } else { x };
                if rtl { x + w - 33.0 } else { x }
            };
            let gy = y + dy;
            if checked && matches!(e, MenuEntry::Radio(..)) {
                // Measured 4×4 bullet at rel x 15..18, slot rows 9..12 (ink over #2c).
                const BULLET: [[u8; 4]; 4] = [
                    [0x54, 0xca, 0xd7, 0x54],
                    [0xd7, 0xff, 0xff, 0xca],
                    [0xca, 0xff, 0xff, 0xd7],
                    [0x54, 0xd7, 0xca, 0x54],
                ];
                for (dy, row) in BULLET.iter().enumerate() {
                    for (dx, v) in row.iter().enumerate() {
                        let a = ((*v as f32 - 44.0) / (255.0 - 44.0) * 255.0).round() as u8;
                        let c = Color32::from_rgba_unmultiplied(glyph.r(), glyph.g(), glyph.b(), a);
                        px(
                            &painter,
                            gx(o.x + 15.0 + dx as f32),
                            gy + 9.0 + dy as f32,
                            c,
                        );
                    }
                }
            } else if checked {
                painter.line(
                    vec![
                        pos2(gx(o.x + 12.5), gy + 11.5),
                        pos2(gx(o.x + 15.5), gy + 14.5),
                        pos2(gx(o.x + 20.5), gy + 8.5),
                    ],
                    Stroke::new(1.3, glyph),
                );
            }
            match e {
                MenuEntry::Sub(..) => {
                    // At the trailing edge, pointing where the submenu opens.
                    painter.line(
                        vec![
                            pos2(winui::mx(o.x + w - 20.5), gy + 7.5),
                            pos2(winui::mx(o.x + w - 16.5), gy + 11.5),
                            pos2(winui::mx(o.x + w - 20.5), gy + 15.5),
                        ],
                        Stroke::new(
                            1.2,
                            if g {
                                crate::theme::colors().menu_shortcut
                            } else {
                                glyph
                            },
                        ),
                    );
                    if depth == 0 && enabled && hovered && (kb.is_none() || moving) {
                        ui.ctx().data_mut(|d| d.insert_temp(sub_key, i));
                    }
                    if depth == 0
                        && hovered
                        && let Some(p) = pointer
                    {
                        ui.ctx().data_mut(|d| d.insert_temp(anchor_key, p));
                    }
                    if depth == 0 && (open_sub == Some(i) || (enabled && hovered && kb.is_none())) {
                        sub_at = Some((i, y - o.y));
                    }
                }
                MenuEntry::Item(_, c, ..) | MenuEntry::Radio(_, c, ..) => {
                    // The pointer on another item of the root closes an open submenu at once
                    // (#579), unless it is crossing the item on its way into the submenu
                    // (diagonally): then after Windows' menu delay, if it rests there.
                    let since_key = sub_key.with(("leave-since", i));
                    if depth == 0 && hovered && open_sub.is_some() {
                        let now = ui.input(|inp| inp.time);
                        let since = ui
                            .ctx()
                            .data_mut(|d| *d.get_temp_mut_or_insert_with(since_key, || now));
                        let anchor = ui.ctx().data(|d| d.get_temp::<Pos2>(anchor_key));
                        let heading = match (anchor, pointer, sub_rect) {
                            (Some(a), Some(p), Some(r)) => heading_into(a, p, r),
                            _ => false,
                        };
                        if !heading || now - since >= 0.35 {
                            ui.ctx().data_mut(|d| {
                                d.remove::<usize>(sub_key);
                                d.remove::<f64>(since_key);
                                d.remove::<Pos2>(anchor_key);
                                d.remove::<Rect>(sub_key.with("sub-rect"));
                            });
                            sub_at = None;
                        } else {
                            // The window that draws the menus (this one is drawn in its pass):
                            // a pointer resting on the item brings no event to close it by.
                            ui.ctx().request_repaint_after_for(
                                std::time::Duration::from_millis(60),
                                parent_vid,
                            );
                        }
                    } else if depth == 0 {
                        ui.ctx().data_mut(|d| d.remove::<f64>(since_key));
                    }
                    if hovered && clicked && enabled {
                        *result = Some(c.clone());
                    }
                }
                MenuEntry::Sep => {}
            }
            y += look.slot;
        }
    });
    if styled {
        show_popup(ctx, vid);
    }
    if let Some((i, slot_y)) = sub_at
        && let MenuEntry::Sub(_, _, children) = &entries[i]
    {
        // The submenu's top-left: parent right − 6, slot top less the padding (3, as measured;
        // Graphite's 5).
        let x = screen.max.x - 6.0;
        let at = pos2(x, screen.min.y + slot_y - look.pad);
        popup_level(
            ctx,
            kind,
            id,
            at,
            Some(screen),
            children,
            sub_key,
            result,
            depth + 1,
            hint,
            icon,
        );
    }
}

// ---- toolbar -------------------------------------------------------------------------------

pub enum ToolbarEvent {
    Click(usize),
    /// A drop-down button's arrow: its menu opens at this point (the button's left edge,
    /// [`Layout::drop_y`] under the toolbar's top).
    Drop(usize, Pos2),
    /// The toggle at the toolbar's end (Details) was pressed.
    Toggle,
}

/// A toggle at the toolbar's end: its icon, label and whether it is on (filled with
/// `toolbar.checked`, exposed with its toggled state).
pub struct ToolbarToggle<'a> {
    pub icon: Icon,
    pub label: &'a str,
    pub on: bool,
}

/// Graphite's toolbar buttons: their top inside the band and their height (DESIGN.md: the
/// band 58, its rule included).
pub const TB_BUTTON_TOP: f32 = 5.0;
pub const TB_BUTTON_H: f32 = 47.0;

/// The buttons that end a group of Graphite's toolbar (a rule after each): [Add URL, Add
/// Torrent] [Resume, Stop, Stop All] [Delete, Delete Completed] [Options, Scheduler] [Start
/// Queue, Stop Queue], IDM's order.
fn ends_group(icon: Icon) -> bool {
    matches!(
        icon,
        Icon::AddTorrent | Icon::StopAll | Icon::DeleteCompleted | Icon::Scheduler
    )
}

/// Graphite's toolbar (plan 05b, the mockup's `.tools`): the band on `panel` with a rule
/// under it, 20-point icons over 11.5-point labels, buttons at least 60 wide (8 either side of
/// the label), 2 apart in a group and a 1-point rule between groups; under the pointer the
/// toolbar's hot face rounded 8; a drop-down's arrow inside its button's end, which is its own
/// part (a split button, as IDM's layout has it). Down to where the panes start
/// ([`Layout::panes_top`]). `toggle`: a toggle button at the band's trailing end (the
/// mockup's Details), at least `RULE_GAP` after the last button.
pub fn toolbar(
    ui: &mut Ui,
    layout: &Layout,
    o: Pos2,
    width: f32,
    buttons: &[(Icon, &str, bool)],
    toggle: Option<ToolbarToggle<'_>>,
) -> Option<ToolbarEvent> {
    let p = ui.painter().clone();
    let c = crate::theme::colors();
    let top = o.y + layout.toolbar_top();
    let band = Rect::from_min_size(pos2(o.x, top), vec2(width, layout.toolbar_h));
    p.rect_filled(winui::rect(band), 0.0, c.panel);
    fill(
        &p,
        o.x,
        band.bottom() - 1.0,
        o.x + width - 1.0,
        band.bottom() - 1.0,
        c.menubar_line,
    );
    let font = egui::FontId::proportional(crate::theme::metrics().font_small);
    const MIN_W: f32 = 60.0;
    const PAD: f32 = 16.0;
    // A drop-down's arrow part, at the button's end.
    const ARROW_W: f32 = 14.0;
    // In a group 2 apart; a group ends 6 before its rule, the next starts 6 after it.
    const GAP: f32 = 2.0;
    const RULE_GAP: f32 = 6.0;
    const SIDE: f32 = 10.0;
    let is_dropdown =
        |icon: &Icon| matches!(icon, Icon::StartQueue | Icon::StopQueue | Icon::AddTorrent);
    let fixed: f32 = 2.0 * SIDE
        + buttons
            .iter()
            .enumerate()
            .map(|(i, (icon, ..))| {
                if i + 1 == buttons.len() {
                    0.0
                } else if ends_group(*icon) {
                    2.0 * RULE_GAP + 1.0
                } else {
                    GAP
                }
            })
            .sum::<f32>()
        + if toggle.is_some() { RULE_GAP } else { 0.0 };
    // A toolbar wider than the window (long labels, 200 %): the longest labels give way first.
    // The toggle's label among them, by the same rules (no arrow).
    let natural: Vec<(f32, f32)> = buttons
        .iter()
        .map(|(icon, label, _)| (label, is_dropdown(icon)))
        .chain(toggle.as_ref().map(|t| (&t.label, false)))
        .map(|(label, dropdown)| {
            let ext = text_width_font(&p, font.clone(), crate::i18n::tc("toolbar", label)).ceil();
            // label_cap pads a label by 12: Graphite's by PAD. A drop-down's arrow is a part of
            // its own at the button's end, past the label.
            (ext + PAD - 12.0, if dropdown { ARROW_W } else { 0.0 })
        })
        .collect();
    // Narrower than 60 only in a window too small for them all (200 %, a narrow window): down
    // to the icon's room, the labels cut with "..." and whole on hover.
    let room = (width - fixed).max(0.0);
    let arrows: f32 = natural.iter().map(|n| n.1).sum();
    let min_w = MIN_W
        .min(((room - arrows) / natural.len().max(1) as f32).floor())
        .max(36.0);
    let cap = label_cap(&natural, min_w, room);
    let mut x = o.x + SIDE;
    let mut event = None;
    let pointer_down = ui.input(|i| i.pointer.primary_down());
    for (i, (icon, label, enabled)) in buttons.iter().enumerate() {
        let full = crate::i18n::tc("toolbar", label);
        let dropdown = is_dropdown(icon);
        let full_ext = text_width_font(&p, font.clone(), full).ceil();
        // The icon and the label in the main part; a drop-down's arrow part after it.
        let main_w = min_w.max((full_ext + PAD).min(cap));
        let bw = main_w + if dropdown { ARROW_W } else { 0.0 };
        let cut = full_ext + PAD > main_w;
        let shown = if cut {
            winui::ellipsize(&p, &font, full, (main_w - PAD).max(0.0))
        } else {
            full.to_owned()
        };
        let english = Rect::from_min_size(pos2(x, top + TB_BUTTON_TOP), vec2(bw, TB_BUTTON_H));
        let rc = winui::rect(english);
        let resp = ui.interact(rc, egui::Id::unique(("tb", i)), winui::pointer_only());
        winui::expose(&resp, egui::Role::Button, full, *enabled, None);
        let resp = if cut { resp.on_hover_text(full) } else { resp };
        let past = |q: Pos2| winui::mx(q.x) >= english.right() - ARROW_W;
        let on_arrow = dropdown && resp.hover_pos().is_some_and(past);
        let radius = crate::theme::metrics().button_radius;
        if (*enabled || (dropdown && on_arrow)) && resp.hovered() {
            let down = resp.is_pointer_button_down_on() && pointer_down;
            let face = if down {
                c.toolbar_pressed
            } else {
                c.toolbar_hot
            };
            if dropdown {
                // The part under the pointer lit, the other faintly, as IDM's layout does.
                let split = english.right() - ARROW_W;
                let main = Rect::from_min_max(english.min, pos2(split, english.bottom()));
                let arrow = Rect::from_min_max(pos2(split, english.top()), english.max);
                // A greyed button's arrow still opens its menu: only the arrow part lights.
                let (main_c, arrow_c) = if on_arrow && !*enabled {
                    (Color32::TRANSPARENT, face)
                } else if on_arrow {
                    (c.toolbar_hot_faint, face)
                } else {
                    (face, c.toolbar_hot_faint)
                };
                let r = radius as u8;
                let left = egui::CornerRadius {
                    nw: r,
                    sw: r,
                    ne: 0,
                    se: 0,
                };
                let right = egui::CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: r,
                    se: r,
                };
                let (lc, rc2) = if winui::rtl() {
                    (right, left)
                } else {
                    (left, right)
                };
                p.rect_filled(winui::rect(main), lc, main_c);
                p.rect_filled(winui::rect(arrow), rc2, arrow_c);
            } else {
                p.rect_filled(rc, radius, face);
            }
        }
        // The icon, 20, 6 under the button's top, centred (a drop-down's in what is left of
        // its arrow's part).
        let cx = english.left() + main_w / 2.0;
        let icon_rc = Rect::from_min_size(
            pos2((cx - 10.0).round(), english.top() + 6.0),
            vec2(20.0, 20.0),
        );
        icon.image(*enabled).paint_at(ui, winui::rect(icon_rc));
        let ext = text_width_font(&p, font.clone(), &shown).ceil();
        let ink = if *enabled {
            c.text_secondary
        } else {
            TB_DISABLED()
        };
        text_font(
            &p,
            font.clone(),
            (cx - ext / 2.0).round(),
            english.top() + 39.0,
            &shown,
            ink,
        );
        if dropdown {
            // A chevron, 10 across at the icon's height, inside the button's end.
            let ax = english.right() - 7.0;
            let ay = english.top() + 14.0;
            let s = Stroke::new(1.5, winui::MUTED());
            p.line(
                vec![
                    pos2(winui::mx(ax - 3.0), ay - 1.5),
                    pos2(winui::mx(ax), ay + 1.5),
                    pos2(winui::mx(ax + 3.0), ay - 1.5),
                ],
                s,
            );
        }
        let on_arrow = dropdown && resp.interact_pointer_pos().is_some_and(past);
        if resp.clicked() && (*enabled || on_arrow) {
            // The drop-down menu's top-left: the button's start, just under it.
            event = Some(if on_arrow {
                ToolbarEvent::Drop(i, pos2(winui::mx(english.left()), top + layout.drop_y))
            } else {
                ToolbarEvent::Click(i)
            });
        }
        x += bw;
        if i + 1 < buttons.len() {
            if ends_group(*icon) {
                let rx = x + RULE_GAP;
                fill(
                    &p,
                    rx,
                    top + TB_BUTTON_TOP + 4.0,
                    rx,
                    top + TB_BUTTON_TOP + TB_BUTTON_H - 5.0,
                    c.menubar_line,
                );
                x = rx + 1.0 + RULE_GAP;
            } else {
                x += GAP;
            }
        }
    }
    if let Some(t) = toggle {
        // At the band's trailing end (the mockup's `.tg.end`), never nearer the last button
        // than RULE_GAP; filled with `toolbar.checked` while on.
        let full = crate::i18n::tc("toolbar", t.label);
        let full_ext = text_width_font(&p, font.clone(), full).ceil();
        let bw = min_w.max((full_ext + PAD).min(cap));
        let cut = full_ext + PAD > bw;
        let shown = if cut {
            winui::ellipsize(&p, &font, full, (bw - PAD).max(0.0))
        } else {
            full.to_owned()
        };
        let left = (o.x + width - SIDE - bw).max(x + RULE_GAP);
        let english = Rect::from_min_size(pos2(left, top + TB_BUTTON_TOP), vec2(bw, TB_BUTTON_H));
        let rc = winui::rect(english);
        let resp = ui.interact(
            rc,
            egui::Id::unique(("tb", "toggle")),
            winui::pointer_only(),
        );
        winui::expose(&resp, egui::Role::Button, full, true, Some(t.on));
        let resp = if cut { resp.on_hover_text(full) } else { resp };
        let radius = crate::theme::metrics().button_radius;
        let face = if resp.hovered() {
            Some(if resp.is_pointer_button_down_on() && pointer_down {
                c.toolbar_pressed
            } else {
                c.toolbar_hot
            })
        } else if t.on {
            Some(c.toolbar_checked)
        } else {
            None
        };
        if let Some(face) = face {
            p.rect_filled(rc, radius, face);
        }
        let cx = english.left() + bw / 2.0;
        let icon_rc = Rect::from_min_size(
            pos2((cx - 10.0).round(), english.top() + 6.0),
            vec2(20.0, 20.0),
        );
        t.icon.image(true).paint_at(ui, winui::rect(icon_rc));
        let ext = text_width_font(&p, font.clone(), &shown).ceil();
        text_font(
            &p,
            font.clone(),
            (cx - ext / 2.0).round(),
            english.top() + 39.0,
            &shown,
            c.text_secondary,
        );
        if resp.clicked() {
            event = Some(ToolbarEvent::Toggle);
        }
    }
    event
}

/// The widest a toolbar button may grow for its label so that all of them fit in `width`:
/// unlimited when they fit whole; else the largest cap at which they do (never under `base_w`).
/// `buttons`: each label's width and its drop-down arrow's.
fn label_cap(buttons: &[(f32, f32)], base_w: f32, width: f32) -> f32 {
    let total = |cap: f32| -> f32 {
        buttons
            .iter()
            .map(|(ext, drop)| base_w.max((ext + 12.0).min(cap)) + drop)
            .sum()
    };
    if total(f32::INFINITY) <= width {
        return f32::INFINITY;
    }
    let (mut lo, mut hi) = (
        base_w,
        buttons.iter().map(|b| b.0 + 12.0).fold(base_w, f32::max),
    );
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if total(mid) <= width {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo.floor()
}

#[cfg(test)]
mod label_cap_tests {
    #[test]
    fn labels_give_way_only_when_the_toolbar_is_too_wide() {
        let b = [(40.0, 0.0), (200.0, 15.0), (300.0, 0.0)];
        // Room for all: no cap.
        assert!(super::label_cap(&b, 70.0, 1000.0).is_infinite());
        // Too wide: the cap brings it inside, and the short label keeps its button.
        let cap = super::label_cap(&b, 70.0, 450.0);
        let total: f32 = b
            .iter()
            .map(|(e, d)| 70f32.max((e + 12.0).min(cap)) + d)
            .sum();
        assert!(total <= 450.0 && cap >= 70.0, "{cap} {total}");
    }
}

// ---- Categories pane -----------------------------------------------------------------------

#[derive(Clone)]
pub struct TreeRow<'a> {
    pub level: usize,
    pub label: std::borrow::Cow<'a, str>,
    /// None: leaf; Some(open): has an expander.
    pub expander: Option<bool>,
    pub selected: bool,
    pub icon: IconKey,
    /// How many downloads the node holds, shown after its label (none when empty).
    pub count: Option<usize>,
}

/// A tree node's count, after its label: "(7)".
fn count_text(n: usize) -> String {
    format!("({})", crate::i18n::integer(n as u64))
}

pub enum TreeEvent {
    Select(usize),
    Toggle(usize),
    /// Right-click: the row whose menu opens (the selection does not move) and where.
    Context(usize, Pos2),
}

/// One Categories row's events and its node: a tree item with
/// its name (and count), selected, open or shut, its level; Click selects it (on its expander,
/// English x `chevron`, toggles it), a double-click toggles it, a right-click and Show context
/// menu open its menu, Expand / Collapse open and shut it. `hit`: the row on screen; `tip`: its
/// whole label, when cut.
#[allow(clippy::too_many_arguments)]
fn tree_row(
    ui: &Ui,
    rows_ui: &Ui,
    r: &TreeRow<'_>,
    k: usize,
    hit: Rect,
    chevron: (f32, f32),
    tip: Option<&str>,
    event: &mut Option<TreeEvent>,
    selected_at: &mut Option<(usize, Pos2)>,
) {
    use egui::accesskit::Action;
    let resp = rows_ui.interact(
        hit,
        egui::Id::unique(("tree-row", k)),
        winui::pointer_only(),
    );
    let resp = match tip {
        Some(t) => resp.on_hover_text(t),
        None => resp,
    };
    let label = crate::i18n::t(&r.label);
    let name = match r.count {
        Some(n) => format!("{label} {}", count_text(n)),
        None => label.to_owned(),
    };
    winui::expose(&resp, egui::Role::TreeItem, &name, true, None);
    let (expand, collapse, menu) = ui.input(|i| {
        (
            i.has_accesskit_action_request(resp.id, Action::Expand),
            i.has_accesskit_action_request(resp.id, Action::Collapse),
            i.has_accesskit_action_request(resp.id, Action::ShowContextMenu),
        )
    });
    let (level, selected, open) = (r.level, r.selected, r.expander);
    ui.ctx().accesskit_node_builder(resp.id, |n| {
        n.set_selected(selected);
        n.set_level(level + 1);
        n.add_action(Action::ShowContextMenu);
        if let Some(open) = open {
            n.set_expanded(open);
            n.add_action(if open {
                Action::Collapse
            } else {
                Action::Expand
            });
        }
    });
    let under = resp.rect.left_bottom() + vec2(24.0, 0.0);
    if r.selected {
        *selected_at = Some((k, under));
    }
    if (expand && r.expander == Some(false)) || (collapse && r.expander == Some(true)) {
        *event = Some(TreeEvent::Toggle(k));
    }
    if menu {
        *event = Some(TreeEvent::Context(k, under));
    }
    // An assistive action moves no pointer: the frame showing its result is asked for.
    if expand || collapse || menu {
        ui.ctx().request_repaint();
    }
    if resp.clicked() {
        let on_expander = resp.interact_pointer_pos().is_some_and(|q| {
            let x = winui::mx(q.x);
            x >= chevron.0 && x <= chevron.1
        });
        *event = Some(if on_expander && r.expander.is_some() {
            TreeEvent::Toggle(k)
        } else {
            TreeEvent::Select(k)
        });
    }
    if resp.double_clicked() && r.expander.is_some() {
        *event = Some(TreeEvent::Toggle(k));
    }
    if resp.secondary_clicked()
        && let Some(at) = resp.interact_pointer_pos()
    {
        *event = Some(TreeEvent::Context(k, at));
    }
}
/// Graphite's Categories pane (plan 05b, the mockup's `aside.cats`): on `panel` from the
/// window's start to `right`, a rule at its end; no caption bar (View → Hide categories closes
/// it); rows 28 high, 1 apart, 10 under the pane's top and 8 in from its sides, each a chevron
/// (branches), a 16-point icon, the label and, on the top rows, the count muted; children 20
/// further in. The chosen row is filled with the selection, rounded 6, its icon in the accent.
/// The same events, keys and accessibility tree as IDM's.
#[allow(clippy::too_many_arguments)]
pub fn categories(
    ui: &mut Ui,
    icons: &mut ShellIcons,
    layout: &Layout,
    o: Pos2,
    bottom: f32,
    right: f32,
    rows: &[TreeRow<'_>],
    keys: bool,
) -> Option<TreeEvent> {
    const ROW: f32 = 28.0;
    const STRIDE: f32 = 29.0;
    let p = ui.painter().clone();
    let c = crate::theme::colors();
    let top = o.y + layout.panes_top();
    let left = o.x + layout.tree_left;
    let pane = Rect::from_min_max(pos2(left, top), pos2(right, bottom));
    p.rect_filled(winui::rect(pane), 0.0, c.panel);
    fill(
        &p,
        right - 1.0,
        top,
        right - 1.0,
        bottom - 1.0,
        c.menubar_line,
    );
    let area = Rect::from_min_max(pos2(left, top), pos2(right - 1.0, bottom));
    let inner_top = top + 10.0;
    let content_h = rows.len() as f32 * STRIDE + 10.0;
    let need_v = content_h > area.height();
    let rows_right = area.right() - 8.0 - if need_v { scrollbar::THICK + 6.0 } else { 0.0 };
    let scroll_id = egui::Id::unique("tree-scroll");
    let mut vpos = ui
        .ctx()
        .data(|d| d.get_temp::<(f32, f32)>(scroll_id))
        .unwrap_or_default()
        .0;
    let page_rows = ((area.height() - 10.0) / STRIDE).floor().max(1.0);
    if ui.rect_contains_pointer(winui::rect(area)) {
        vpos += winui::wheel_rows(ui, STRIDE, page_rows);
    }
    vpos = vpos
        .clamp(0.0, (rows.len() as f32 - page_rows).max(0.0))
        .round();
    let row_top = |k: usize| inner_top + STRIDE * (k as f32 - vpos);
    let tp = p.with_clip_rect(winui::rect(area));
    let mut event = None;
    let blank = ui.interact(
        winui::rect(area),
        egui::Id::unique("tree-blank"),
        winui::pointer_only(),
    );
    if blank.secondary_clicked()
        && let Some(k) = rows.iter().position(|r| r.selected)
        && let Some(at) = blank.interact_pointer_pos()
    {
        event = Some(TreeEvent::Context(k, at));
    }
    winui::expose(
        &blank,
        egui::Role::Tree,
        crate::i18n::t("Categories"),
        true,
        None,
    );
    let rows_ui = ui.new_child(egui::UiBuilder::new().accessibility_parent(blank.id));
    let mut selected_at: Option<(usize, Pos2)> = None;
    let font = winui::segoe();
    let medium = egui::FontId::new(font.size, egui::FontFamily::Name("geist-medium".into()));
    let radius = crate::theme::metrics().part_radius;
    for (k, r) in rows.iter().enumerate() {
        let y = row_top(k);
        if y + ROW <= area.top() || y >= area.bottom() {
            continue;
        }
        let row = Rect::from_min_max(pos2(left + 8.0, y), pos2(rows_right, y + ROW));
        if r.selected {
            tp.rect_filled(winui::rect(row), radius, winui::SELECTION());
        }
        let indent = if r.level == 0 { 0.0 } else { 20.0 };
        let chev_x = row.left() + 6.0 + indent;
        let mid = y + ROW / 2.0;
        if let Some(open) = r.expander {
            // The mockup's chevron-down / -right, 13 across, in the expander's ink.
            let s = Stroke::new(1.5, TREE_EXPANDER());
            let cx = chev_x + 6.5;
            let pts = if open {
                vec![
                    pos2(cx - 3.5, mid - 1.75),
                    pos2(cx, mid + 1.75),
                    pos2(cx + 3.5, mid - 1.75),
                ]
            } else {
                vec![
                    pos2(cx - 1.75, mid - 3.5),
                    pos2(cx + 1.75, mid),
                    pos2(cx - 1.75, mid + 3.5),
                ]
            };
            tp.line(
                pts.into_iter().map(|q| pos2(winui::mx(q.x), q.y)).collect(),
                s,
            );
        }
        let icon_x = chev_x + 13.0 + 7.0;
        let icon_rc = Rect::from_min_size(pos2(icon_x, (mid - 8.0).round()), vec2(16.0, 16.0));
        // The chosen row's icon in the accent, its label in the text's ink; the others in the
        // secondary ink (the mockup's).
        let ink = if r.selected {
            crate::theme::colors().accent
        } else {
            crate::theme::colors().text_secondary
        };
        let label_ink = if r.selected {
            TEXT()
        } else {
            crate::theme::colors().text_secondary
        };
        let drawn_themed = match &r.icon {
            IconKey::Themed(need) => {
                crate::themeicons::paint_in(ui, &tp, need, winui::rect(icon_rc), ink, true)
            }
            _ => false,
        };
        if !drawn_themed {
            icons.paint_in(ui, &tp, r.icon.clone(), icon_rc.min);
        }
        let text_x = icon_rc.right() + 7.0;
        let label = crate::i18n::t(&r.label);
        let count = r.count.map(count_text);
        let count_w = count
            .as_ref()
            .map_or(0.0, |s| 5.0 + text_width_font(&p, font.clone(), s).ceil());
        let row_font = if r.selected {
            medium.clone()
        } else {
            font.clone()
        };
        let shown = winui::ellipsize(
            &p,
            &row_font,
            label,
            (row.right() - 6.0 - text_x - count_w).max(0.0),
        );
        let baseline = (mid + 4.0).round();
        let drawn = text_font(&tp, row_font, text_x, baseline, &shown, label_ink);
        if let Some(s) = &count {
            text_font(
                &tp,
                font.clone(),
                winui::rect(drawn).right() + 5.0,
                baseline,
                s,
                winui::MUTED(),
            );
        }
        if keys && r.selected {
            winui::focus_ring(&tp, winui::rect(row));
        }
        let hit = winui::rect(row.intersect(area));
        let tip = (shown != label).then_some(label);
        tree_row(
            ui,
            &rows_ui,
            r,
            k,
            hit,
            (chev_x - 4.0, chev_x + 17.0),
            tip,
            &mut event,
            &mut selected_at,
        );
    }
    if keys
        && let Some((k, at)) = selected_at
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::SHIFT, egui::Key::F10)
                || i.consume_key(egui::Modifiers::NONE, egui::Key::Menu)
        })
    {
        event = Some(TreeEvent::Context(k, at));
    }
    if need_v {
        // Clear of the splitter's grip, the pane's last 6 points (App::paint).
        let track = winui::rect(Rect::from_min_max(
            pos2(area.right() - 6.0 - scrollbar::THICK, area.top()),
            pos2(area.right() - 6.0, area.bottom()),
        ));
        if scrollbar::bar(
            ui,
            scroll_id.with("v"),
            track,
            true,
            rows.len() as f32 + 10.0 / STRIDE,
            page_rows,
            1.0,
            &mut vpos,
        ) {
            vpos = vpos.round();
            ui.ctx().request_repaint();
        }
    }
    ui.ctx().data_mut(|d| d.insert_temp(scroll_id, (vpos, 0.0)));
    event
}

#[cfg(test)]
mod tests {
    use super::{
        FOCUS_GRACE, FocusVerdict, MenuBarEvent, MenuEntry, MenuFocus, MenuKind, MenuLook, SEP,
        SLOT, heading_into, item_at, menu_viewport, menubar_next,
    };
    use egui::{Rect, pos2};

    /// #277: on X11 the menu bar's menus share one window per level, so going from title to
    /// title moves it (a compositor has no new window to fade in); elsewhere, and for every
    /// other menu, a window of their own as before.
    #[test]
    fn the_bar_menus_share_one_window_on_x11() {
        let bar = |i: usize, depth, shared| {
            menu_viewport(MenuKind::Bar, &format!("menu-{i}"), depth, shared)
        };
        // Tasks, File, Downloads, View, Help: one window (and one for their submenus).
        for i in 1..5 {
            assert_eq!(bar(0, 0, true), bar(i, 0, true));
            assert_eq!(bar(0, 1, true), bar(i, 1, true));
        }
        assert_ne!(bar(0, 0, true), bar(0, 1, true));
        // Without sharing: one per menu, as before.
        assert_ne!(bar(0, 0, false), bar(1, 0, false));
        assert_eq!(
            bar(2, 0, false),
            egui::ViewportId::from_hash_of(("menu", "menu-2", 0usize))
        );
        // A context menu and a drop-down list keep their own window, shared or not.
        for shared in [true, false] {
            let ctx = menu_viewport(MenuKind::Context, "popup-1", 0, shared);
            let list = menu_viewport(MenuKind::List, "drop-x", 0, shared);
            assert_ne!(ctx, bar(0, 0, shared));
            assert_ne!(list, bar(0, 0, shared));
            assert_ne!(ctx, list);
            assert_eq!(
                ctx,
                egui::ViewportId::from_hash_of(("menu", "popup-1", 0usize))
            );
        }
    }

    /// Runs `events` from no menu open; the open menu after each.
    fn run(events: &[MenuBarEvent]) -> Vec<Option<usize>> {
        let mut open = None;
        events
            .iter()
            .map(|e| {
                open = menubar_next(open, *e);
                open
            })
            .collect()
    }

    #[test]
    fn a_title_clicked_after_escape_opens_its_menu_again() {
        // Click Tasks, Escape (the pointer still on Tasks: no press), click Tasks.
        let steps = run(&[
            MenuBarEvent::Title {
                i: 0,
                released: true,
            },
            MenuBarEvent::Dismissed {
                press_on_title: false,
            },
            MenuBarEvent::Title {
                i: 0,
                released: true,
            },
        ]);
        assert_eq!(steps, [Some(0), None, Some(0)]);
    }

    #[test]
    fn clicking_the_open_menus_title_shuts_it() {
        // The press shuts the menu's window; its release, on the same title, shuts the menu.
        let steps = run(&[
            MenuBarEvent::Title {
                i: 1,
                released: true,
            },
            MenuBarEvent::Dismissed {
                press_on_title: true,
            },
            MenuBarEvent::Title {
                i: 1,
                released: true,
            },
            MenuBarEvent::Title {
                i: 1,
                released: true,
            },
        ]);
        assert_eq!(steps, [Some(1), Some(1), None, Some(1)]);
    }

    #[test]
    fn another_title_opens_its_menu_from_an_open_one() {
        let steps = run(&[
            MenuBarEvent::Title {
                i: 0,
                released: true,
            },
            // Hovered while Tasks is open: File opens.
            MenuBarEvent::Hover(1),
            // Pressed and released on Downloads.
            MenuBarEvent::Dismissed {
                press_on_title: true,
            },
            MenuBarEvent::Title {
                i: 2,
                released: true,
            },
        ]);
        assert_eq!(steps, [Some(0), Some(1), Some(1), Some(2)]);
    }

    /// The focus over a menu's passes (`(focused, time)` each), as `MenuFocus` judges it, no
    /// other program in front.
    fn focus_run(passes: &[(bool, f64)]) -> Vec<FocusVerdict> {
        let mut f = MenuFocus::default();
        passes.iter().map(|&(on, t)| f.pass(on, false, t)).collect()
    }

    /// Upstream issue 859: a click on the desktop, the taskbar or another program's window while
    /// a menu is open. Windows says another program is in front: the menu closes on the very
    /// pass that learns the focus left, not FOCUS_GRACE later.
    #[test]
    fn a_click_in_another_program_closes_the_menu_at_once() {
        let mut f = MenuFocus::default();
        assert_eq!(f.pass(true, false, 0.0), FocusVerdict::Here);
        assert_eq!(f.pass(true, false, 0.5), FocusVerdict::Here);
        assert_eq!(f.pass(false, true, 1.0), FocusVerdict::Left);
        // Away from our windows without another program in front (focus passing between our
        // own windows): the grace.
        let mut f = MenuFocus::default();
        f.pass(true, false, 0.0);
        assert_eq!(
            f.pass(false, false, 1.0),
            FocusVerdict::Away { wait: FOCUS_GRACE }
        );
        assert_eq!(f.pass(false, true, 1.01), FocusVerdict::Left);
        // A menu opened while another program was in front (a tray menu): nothing to lose.
        let mut f = MenuFocus::default();
        assert_eq!(f.pass(false, true, 0.0), FocusVerdict::Here);
        assert_eq!(f.pass(false, true, 5.0), FocusVerdict::Here);
    }

    /// #250: Tasks open, Import's submenu open with the focus (KWin, sway), the pointer back
    /// onto another item: the submenu's window closes and hands the focus on, and for a few
    /// frames no window of ours has it. The menu stays.
    #[test]
    fn a_submenu_closing_with_the_focus_leaves_its_menu_open() {
        let v = focus_run(&[
            (true, 0.0),   // the main window, then the menu's window
            (true, 0.5),   // the submenu's window took it
            (false, 1.0),  // the submenu closed: nobody has it
            (false, 1.02), // the next frame
            (true, 1.05),  // the main window (or the menu's) got it back
            (true, 1.6),
        ]);
        assert!(v.iter().all(|x| *x != FocusVerdict::Left), "{v:?}");
        assert_eq!(v[2], FocusVerdict::Away { wait: FOCUS_GRACE });
        // Gone again later: counted afresh.
        let v = focus_run(&[
            (true, 0.0),
            (false, 1.0),
            (true, 1.3),
            (false, 5.0),
            (false, 5.3),
        ]);
        assert!(v.iter().all(|x| *x != FocusVerdict::Left), "{v:?}");
    }

    /// Another app takes the focus (a click on its window, Alt+Tab): the menu closes once the
    /// focus has stayed away, and asks to be looked at again until then.
    #[test]
    fn the_focus_gone_to_another_app_closes_the_menu() {
        let v = focus_run(&[(true, 0.0), (false, 2.0), (false, 2.1), (false, 2.5)]);
        assert_eq!(v[0], FocusVerdict::Here);
        assert!(matches!(v[1], FocusVerdict::Away { wait } if (wait - FOCUS_GRACE).abs() < 1e-9));
        assert!(
            matches!(v[2], FocusVerdict::Away { wait } if (wait - (FOCUS_GRACE - 0.1)).abs() < 1e-9)
        );
        assert_eq!(v[3], FocusVerdict::Left);
        // A menu opened while another app was in front (a tray menu): no focus to lose.
        let v = focus_run(&[(false, 0.0), (false, 1.0), (false, 9.0)]);
        assert!(v.iter().all(|x| *x == FocusVerdict::Here), "{v:?}");
    }

    /// #277: Help → About clicked, About closed, then the pointer over the titles: nothing opens
    /// until a click. The bar is as inactive after a command as before the first click.
    #[test]
    fn after_a_command_the_pointer_alone_opens_nothing() {
        use MenuBarEvent::{Chosen, Dismissed, Hover, Title};
        let steps = run(&[
            // Before any click: the pointer over the titles opens nothing.
            Hover(0),
            Hover(4),
            // Help clicked; the pointer on its own title keeps it.
            Title {
                i: 4,
                released: true,
            },
            Hover(4),
            // About chosen: the menu closes.
            Chosen,
            // About opens and takes the focus, is closed, the main window has it again: no
            // event of the bar's (its menu is closed). A stray dismissal changes nothing.
            Dismissed {
                press_on_title: false,
            },
            // The pointer over every title: still nothing.
            Hover(0),
            Hover(1),
            Hover(2),
            Hover(3),
            Hover(4),
            // A click opens a menu again.
            Title {
                i: 1,
                released: true,
            },
        ]);
        assert_eq!(
            steps,
            [
                None,
                None,
                Some(4),
                Some(4),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(1)
            ]
        );
        // The same after a command chosen from the keyboard (Alt+H, A), and after Escape or a
        // click elsewhere.
        for end in [
            Chosen,
            Dismissed {
                press_on_title: false,
            },
        ] {
            assert_eq!(
                run(&[MenuBarEvent::Open(4), end, Hover(0), Hover(3)]),
                [Some(4), None, None, None]
            );
        }
    }

    /// #579: Tasks with Export's submenu open to the right (menu 200 wide, Export's slot at
    /// y 190..212). The pointer going on down or up the menu is not heading into the submenu, so
    /// the next item closes it at once; crossing the item below diagonally towards the submenu
    /// keeps it, as does going there on the left side of a right-to-left menu.
    #[test]
    fn a_submenu_stays_open_only_while_the_pointer_heads_into_it() {
        let sub = Rect::from_min_max(pos2(194.0, 187.0), pos2(330.0, 236.0));
        let left_export = pos2(120.0, 205.0);
        // Down to Import, up to Show drop target: straight, not towards the submenu.
        assert!(!heading_into(left_export, pos2(118.0, 216.0), sub));
        assert!(!heading_into(left_export, pos2(121.0, 170.0), sub));
        assert!(!heading_into(left_export, pos2(60.0, 225.0), sub));
        // Diagonally across Import, towards the submenu.
        assert!(heading_into(left_export, pos2(150.0, 214.0), sub));
        assert!(heading_into(left_export, pos2(185.0, 230.0), sub));
        // Right to left: the submenu on the left.
        let rtl = Rect::from_min_max(pos2(-130.0, 187.0), pos2(6.0, 236.0));
        assert!(heading_into(pos2(80.0, 205.0), pos2(50.0, 214.0), rtl));
        assert!(!heading_into(pos2(80.0, 205.0), pos2(82.0, 216.0), rtl));
    }

    /// #579, the menu itself (egui without a window system: each level drawn in the main
    /// window): Tasks-like, Import's submenu open, the pointer going on up the menu closes it at
    /// once and lights only the item it is on, where before the submenu stayed open (and its
    /// entry lit) until the pointer had rested 0.35 s on one item, so never while it moved on.
    /// Crossing an item on the way into the submenu keeps it.
    #[test]
    fn the_pointer_on_another_item_closes_the_submenu_at_once() {
        // Labels through names, so tools/i18n/extract.py doesn't take them for texts to translate.
        let [rss, show_drop, import, file, exit] = [
            "RSS feeds...",
            "Show drop target",
            "Import",
            "From a file",
            "Exit",
        ];
        let entries: Vec<MenuEntry<u8>> = vec![
            MenuEntry::Item(rss, 0, true, false),
            MenuEntry::Item(show_drop, 1, true, false),
            MenuEntry::Sep,
            MenuEntry::Sub(import, true, vec![MenuEntry::Item(file, 2, true, false)]),
            MenuEntry::Item(exit, 3, true, false),
        ];
        let ctx = egui::Context::default();
        let sub_key = egui::Id::unique(("popup-sub", "t579"));
        let mut t = 0.0;
        let mut frame = |pointer: Option<egui::Pos2>| {
            t += 0.016;
            let mut raw = egui::RawInput {
                time: Some(t),
                screen_rect: Some(Rect::from_min_size(
                    pos2(0.0, 0.0),
                    egui::vec2(1200.0, 800.0),
                )),
                ..Default::default()
            };
            if let Some(p) = pointer {
                raw.events.push(egui::Event::PointerMoved(p));
            }
            let mut out = ctx.run_ui(raw, |ui| {
                let mut result = None;
                super::popup_level(
                    ui.ctx(),
                    MenuKind::Bar,
                    "t579",
                    pos2(100.0, 100.0),
                    None,
                    &entries,
                    sub_key,
                    &mut result,
                    0,
                    &|_| None,
                    // 05b's menu icons: none here (the test is about the pointer, not the look).
                    &|_, _| None,
                );
            });
            out.textures_delta.clear();
        };
        frame(None);
        let o = ctx
            .data(|d| d.get_temp::<egui::Pos2>(sub_key.with(("origin", 0usize))))
            .expect("drawn");
        // The menu bar's (Graphite's) slots: 5..33, 33..61, separator, Import 72..100, Exit
        // 100..128.
        let at = |x: f32, y: f32| Some(o + egui::vec2(x, y));
        let open = |ctx: &egui::Context| ctx.data(|d| d.get_temp::<usize>(sub_key));
        for _ in 0..3 {
            frame(at(60.0, 86.0));
        }
        assert_eq!(open(&ctx), Some(3), "Import's submenu opens");
        // Straight up to Show drop target: closed in that pass.
        frame(at(60.0, 47.0));
        frame(at(60.0, 45.0));
        assert_eq!(open(&ctx), None, "closed at once");

        // Again, then diagonally across Exit towards the submenu: kept.
        for _ in 0..3 {
            frame(at(60.0, 86.0));
        }
        assert_eq!(open(&ctx), Some(3));
        let right = ctx
            .data(|d| d.get_temp::<egui::Pos2>(sub_key.with(("origin", 1usize))))
            .expect("submenu drawn")
            .x;
        // Most of the way to the submenu, a little lower: over Exit.
        let x = o.x + (right - o.x) * 0.9 - o.x;
        frame(at(x, 102.0));
        assert_eq!(open(&ctx), Some(3), "kept on the way into it");
    }

    /// #733: a tap on a touch screen chooses the item under the finger. egui-winit makes the
    /// tap a press and a release of the pointer, then lets go of it (PointerGone) as the finger
    /// lifts, in the same pass as the release (in one pass with the press too, for a quick tap):
    /// the item was looked for under a pointer that was gone, and none was chosen.
    #[test]
    fn a_tap_chooses_the_item_under_the_finger() {
        let [a, b, c] = ["a", "b", "c"]; // not texts to translate (tools/i18n/extract.py)
        let entries: Vec<MenuEntry<u8>> = vec![
            MenuEntry::Item(a, 0, true, false),
            MenuEntry::Sep,
            MenuEntry::Item(b, 1, true, false),
            MenuEntry::Item(c, 2, true, false),
        ];
        // Each pass's events; what the menu chose.
        let run = |passes: &[&dyn Fn(egui::Pos2) -> Vec<egui::Event>]| {
            let ctx = egui::Context::default();
            let sub_key = egui::Id::unique(("popup-sub", "t733"));
            let mut t = 0.0;
            let mut chosen = None;
            let mut pass = |events: Vec<egui::Event>| {
                t += 0.016;
                let raw = egui::RawInput {
                    time: Some(t),
                    screen_rect: Some(Rect::from_min_size(
                        pos2(0.0, 0.0),
                        egui::vec2(1200.0, 800.0),
                    )),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(raw, |ui| {
                    let mut result = None;
                    super::popup_level(
                        ui.ctx(),
                        MenuKind::Context,
                        "t733",
                        pos2(100.0, 100.0),
                        None,
                        &entries,
                        sub_key,
                        &mut result,
                        0,
                        &|_| None,
                        &|_, _| None, // the fork's menu icons (plan 05b): none in this test
                    );
                    if result.is_some() {
                        chosen = result;
                    }
                });
                out.textures_delta.clear();
            };
            pass(Vec::new());
            let o = ctx
                .data(|d| d.get_temp::<egui::Pos2>(sub_key.with(("origin", 0usize))))
                .expect("drawn");
            // "b": below "a" (3..25) and the separator (25..34), at 34..56.
            let on_b = o + egui::vec2(60.0, 45.0);
            for p in passes {
                pass(p(on_b));
            }
            chosen
        };
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let down = |p: egui::Pos2| vec![egui::Event::PointerMoved(p), button(p, true)];
        let up_gone = |p: egui::Pos2| vec![button(p, false), egui::Event::PointerGone];
        let tap = |p: egui::Pos2| {
            vec![
                egui::Event::PointerMoved(p),
                button(p, true),
                button(p, false),
                egui::Event::PointerGone,
            ]
        };
        let up = |p: egui::Pos2| vec![button(p, false)];
        // A tap: the finger down in one pass, up (and the pointer gone) in the next.
        assert_eq!(run(&[&down, &up_gone]), Some(1));
        // A quick tap, all in one pass.
        assert_eq!(run(&[&tap]), Some(1));
        // The mouse, as before.
        assert_eq!(run(&[&down, &up]), Some(1));
    }

    /// #579: the entry under the pointer, separators and padding none.
    #[test]
    fn the_item_under_the_pointer() {
        let [a, b, c] = ["a", "b", "c"]; // not texts to translate (tools/i18n/extract.py)
        let entries: Vec<MenuEntry<u8>> = vec![
            MenuEntry::Item(a, 0, true, false),
            MenuEntry::Sep,
            MenuEntry::Sub(b, true, vec![]),
            MenuEntry::Item(c, 1, true, false),
        ];
        let o = pos2(0.0, 0.0);
        let classic = MenuLook {
            graphite: false,
            pad: 3.0,
            slot: SLOT,
            sep: SEP,
            text_x: 38.0,
            min_w: 0.0,
        };
        assert_eq!(
            item_at(&entries, &classic, o, 200.0, pos2(50.0, 10.0)),
            Some(0)
        );
        // The separator: 25..34.
        assert_eq!(
            item_at(&entries, &classic, o, 200.0, pos2(50.0, 29.0)),
            None
        );
        assert_eq!(
            item_at(&entries, &classic, o, 200.0, pos2(50.0, 40.0)),
            Some(2)
        );
        assert_eq!(
            item_at(&entries, &classic, o, 200.0, pos2(50.0, 60.0)),
            Some(3)
        );
        // The padding at the sides.
        assert_eq!(item_at(&entries, &classic, o, 200.0, pos2(3.0, 40.0)), None);
        // Graphite's taller rows (05b): a at 5..33, the separator 33..44, b 44..72, c 72..100,
        // inset 5 from the sides: the rows the menu draws, not the classic ones.
        let graphite = MenuLook {
            graphite: true,
            pad: 5.0,
            slot: 28.0,
            sep: 11.0,
            text_x: 39.0,
            min_w: 240.0,
        };
        assert_eq!(
            item_at(&entries, &graphite, o, 240.0, pos2(50.0, 30.0)),
            Some(0)
        );
        assert_eq!(
            item_at(&entries, &graphite, o, 240.0, pos2(50.0, 38.0)),
            None
        );
        assert_eq!(
            item_at(&entries, &graphite, o, 240.0, pos2(50.0, 70.0)),
            Some(2)
        );
        assert_eq!(
            item_at(&entries, &graphite, o, 240.0, pos2(50.0, 90.0)),
            Some(3)
        );
        assert_eq!(
            item_at(&entries, &graphite, o, 240.0, pos2(4.0, 50.0)),
            None
        );
    }

    /// With a menu open, the pointer moves it from title to title, as in Windows' menu bars
    /// (and IDM's); on its own title it stays.
    #[test]
    fn with_a_menu_open_the_pointer_switches_menus() {
        use MenuBarEvent::{Hover, Title};
        let steps = run(&[
            Title {
                i: 0,
                released: true,
            },
            Hover(1),
            Hover(1),
            Hover(3),
            Hover(0),
        ]);
        assert_eq!(steps, [Some(0), Some(1), Some(1), Some(3), Some(0)]);
    }

    #[test]
    fn a_press_elsewhere_a_choice_or_the_focus_lost_closes_the_menu() {
        for end in [
            MenuBarEvent::Dismissed {
                press_on_title: false,
            },
            MenuBarEvent::Chosen,
        ] {
            assert_eq!(run(&[MenuBarEvent::Open(3), end]), [Some(3), None]);
        }
        // Left / Right from the keyboard move to the neighbour.
        assert_eq!(
            run(&[MenuBarEvent::Open(0), MenuBarEvent::Open(4)]),
            [Some(0), Some(4)]
        );
    }
}
