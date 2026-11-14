//! Options (Downloads → Options, the toolbar's Options): the configuration dialog, twelve pages
//! listed down its side (plan 08; IDM's two rows of tabs before, notes/ui-spec/options-dialog.md).
//! It edits a copy of the settings: OK applies it, the close box and Escape discard it.

use egui::{Pos2, Rect, Sense, Ui, pos2, vec2};

use crate::dialogs::{Outcome, window};
use crate::model::Category;
use crate::regional::DateOrder;
use crate::settings::{self, Settings};
// Pages are laid out in English pixels: `fill` makes each one where it is on screen, `text`
// starts at an English pen, and a widget given a screen rect (from `r`) turns it back into its
// English rect (`winui::rect` both ways) before laying out its inside.
use crate::winui::{
    self, ButtonKind, FooterButton, FooterItem, LIGHT_BG, LIGHT_DISABLED, LIGHT_PAGE, LIGHT_TEXT,
    fill_l as fill, r, text,
};

pub const DUPLICATE: [&str; 4] = [
    crate::i18n::mark("Show a dialog and ask what to do."),
    crate::i18n::mark("Add the duplicate with a numbered file name"),
    crate::i18n::mark("Add the duplicate and overwrite the existing file"),
    crate::i18n::mark(
        "If existing file is complete, show download complete dialog; otherwise resume it.",
    ),
];
const SOUND_EVENTS: [&str; 4] = [
    crate::i18n::mark("Download complete"),
    crate::i18n::mark("Download failed"),
    crate::i18n::mark("Queue processing started"),
    crate::i18n::mark("Queue processing stopped/finished"),
];

/// A small entry dialog opened from a page.
enum Sub {
    /// Sites Logins → New / Edit: (row being edited, site/path, user, password).
    Login(Option<usize>, String, String, String),
    /// Connection → New / Edit exception: (row, server, number).
    Exception(Option<usize>, String, String),
    /// General → Download panels → Edit...: (the button over videos on, the sites it stays off).
    Panels(bool, String, String),
    /// General → Keys...: (also take the files the browser would open).
    Keys(bool),
    /// General → Test browser integration...: the checks, as found when it opened.
    Test(Vec<crate::extension::Check>),
    /// File types → "Don't start downloading automatically from the following addresses" →
    /// Edit list...: one address pattern per line.
    Addresses(String),
    /// Connection → DNS → Settings...: how the DNS is asked, its servers, a test.
    Dns(DnsEdit),
    /// Accounts → Add...: the service, a pasted token, and the sign-in running (#44).
    Account(AccountEdit),
    /// Remote → Pair a device...: the code shown, until a device pairs or it is closed.
    Pair,
}

/// Options → Remote's page number (the numbers in `PAGES`).
const REMOTE_PAGE: usize = 11;

/// Connection → DNS's sub-dialog: a copy of the DNS fields, kept on OK.
struct DnsEdit {
    mode: u8,
    provider: String,
    servers: String,
    tls_name: String,
    doh_url: String,
    nextdns_id: String,
    fallback: bool,
    /// The Test button's answer (shared with its thread): None while it runs.
    test: Option<TestSlot>,
}

/// A test's answer line and whether it worked, filled in by its thread.
type TestSlot = std::sync::Arc<std::sync::Mutex<Option<(String, bool)>>>;

impl DnsEdit {
    fn from(s: &Settings) -> Self {
        DnsEdit {
            mode: if s.dns_mode == 0 { 3 } else { s.dns_mode },
            provider: s.dns_provider.clone(),
            servers: s.dns_servers.clone(),
            tls_name: s.dns_tls_name.clone(),
            doh_url: s.dns_doh_url.clone(),
            nextdns_id: s.dns_nextdns_id.clone(),
            fallback: s.dns_fallback,
            test: None,
        }
    }

    fn choice(&self) -> Result<mdm_engine::DnsChoice, String> {
        settings::dns_choice(
            self.mode,
            &self.provider,
            &self.servers,
            &self.tls_name,
            &self.doh_url,
            &self.nextdns_id,
            self.fallback,
        )
    }

    fn save(self, s: &mut Settings) {
        s.dns_mode = self.mode;
        s.dns_provider = self.provider;
        s.dns_servers = self.servers.trim().to_owned();
        s.dns_tls_name = self.tls_name.trim().to_owned();
        s.dns_doh_url = self.doh_url.trim().to_owned();
        s.dns_nextdns_id = self.nextdns_id.trim().to_lowercase();
        s.dns_fallback = self.fallback;
    }
}

/// Sites Logins → Cookies for video sites' browser at first (#229): the one chosen, else the
/// first browser whose extension reached MDM when yt-dlp reads it (MDM doesn't know the
/// system's default browser), else Firefox, which yt-dlp reads on every system.
fn cookie_browser_at(s: &Settings) -> usize {
    let browsers = mdm_tools::proc::cookie_browsers();
    let seen = mdm_ipc::extension::seen_browsers();
    let by_extension = seen.iter().find_map(|(name, _)| {
        let ytdlp = match name.as_str() {
            "Mozilla Firefox" => "firefox",
            "Google Chrome" => "chrome",
            "Microsoft Edge" => "edge",
            "Brave Browser" => "brave",
            "Opera" => "opera",
            _ => return None,
        };
        browsers.iter().position(|b| *b == ytdlp)
    });
    browsers
        .iter()
        .position(|b| *b == s.video_cookies)
        .or(by_extension)
        .unwrap_or(0)
}

/// Connection → DNS's list: the system's, the services by name, the user's own.
fn dns_names() -> Vec<String> {
    let mut out = vec![
        // i18n: Options → Connection → DNS: the operating system's own DNS (the default)
        crate::i18n::t("The system's DNS").to_owned(),
    ];
    out.extend(mdm_engine::dns::PRESETS.iter().map(|p| p.name.to_owned()));
    // i18n: Options → Connection → DNS: a DNS server the user types in
    out.push(crate::i18n::t("Another DNS...").to_owned());
    out
}

fn dns_index(s: &Settings) -> usize {
    if s.dns_mode == 0 {
        return 0;
    }
    let presets = mdm_engine::dns::PRESETS;
    presets
        .iter()
        .position(|p| p.id == s.dns_provider)
        .map_or(presets.len() + 1, |i| i + 1)
}

fn set_dns_index(s: &mut Settings, i: usize) {
    let presets = mdm_engine::dns::PRESETS;
    if i == 0 {
        s.dns_mode = 0;
        return;
    }
    s.dns_provider = presets
        .get(i - 1)
        .map_or("custom".to_owned(), |p| p.id.to_owned());
    if s.dns_mode == 0 {
        s.dns_mode = 3;
    }
    // A service that isn't offered this way (NextDNS plain) moves to DNS over HTTPS.
    if let (Some(p), Some(m)) = (presets.get(i - 1), settings::dns_mode(s.dns_mode))
        && !p.offers(m)
    {
        s.dns_mode = 3;
    }
}

pub struct OptionsDialog {
    s: Settings,
    /// The settings it opened with: OK applies only what changed from them.
    pub from: Settings,
    page: usize,
    /// The browsers check list's scroll and selection.
    browser_rows: winui::RowList,
    category: usize,
    cat_dir: String,
    /// Connection → "Default max. conn. number", as typed.
    conns: String,
    mb: String,
    /// File types → "Don't take files smaller than", as typed (KB).
    min_kb: String,
    hours: String,
    /// The exceptions' and site logins' lists: scroll and selection.
    exc_rows: winui::RowList,
    login_rows: winui::RowList,
    /// Accounts' list (#44).
    account_rows: winui::RowList,
    /// The Sounds list's scroll and selection.
    sound_rows: winui::RowList,
    sub: Option<Sub>,
    /// Save to → New / Edit...: the category editor (D-021).
    cat_edit: Option<crate::categories::CategoryDialog>,
    /// Save to → Restore the built-in categories: its question (#93).
    restore_ask: Option<crate::dialogs::Confirm>,
    /// Connection caps the engine learned, shown as "(auto)" rows under Exceptions.
    learned: Vec<(String, u32)>,
    /// 7-Zip's program (Archives: RAR goes through it), looked for when the window opens.
    seven_zip: Option<std::path::PathBuf>,
    /// Proxy / Socks → Network adapter (#35): the computer's adapters, read again every 2 s.
    adapters: Option<(std::time::Instant, Vec<mdm_engine::net::Adapter>)>,
    /// Remote (#141): the running remote control (its state, paired devices, pairing), the
    /// port as typed, and the paired devices list's scroll and selection.
    remote: Option<mdm_remote::Remote>,
    port: String,
    device_rows: winui::RowList,
    /// Test browser integration → Set it up's runs seen finished (crate::extension).
    set_up_seen: u64,
    /// Sites Logins → Cookies for video sites (#229): the browser in the list (an index into
    /// `mdm_tools::proc::cookie_browsers`), kept while Off is chosen.
    cookie_browser: usize,
}

impl OptionsDialog {
    /// Open on page `page` (the numbers in `PAGES`: 0 General … 4 Connection, 5 Proxy / Socks,
    /// 6 Sites Logins, 7 Accounts, 8 Sounds, 9 Regional, 10 Archives, 11 Remote); for
    /// `/show options:<page>`.
    pub fn on_page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// With the app's remote control (Remote: its state, devices and pairing).
    pub fn with_remote(mut self, remote: Option<mdm_remote::Remote>) -> Self {
        self.remote = remote;
        self
    }

    /// With Remote → Pair a device... open (`/show options:pair`).
    pub fn with_pairing(mut self) -> Self {
        self.page = REMOTE_PAGE;
        self.pair();
        self
    }

    /// With General → Download panels → Edit... open (`/show options:panels`).
    pub fn with_panels(mut self) -> Self {
        self.sub = Some(Sub::Panels(
            self.s.video_panel,
            self.s.video_panel_sites.clone(),
            String::new(),
        ));
        self
    }

    /// With General → Test browser integration's window over it, or Accounts → Add's (`/show
    /// options:test`, `options:account`: screenshots, #192).
    pub fn with_window(mut self, name: &str) -> Self {
        match name {
            "test" => self.sub = Some(Sub::Test(crate::extension::checks(&self.s))),
            "account" => self.sub = Some(Sub::Account(AccountEdit::default())),
            _ => {}
        }
        self
    }

    pub fn new(
        settings: &Settings,
        downloads: &std::path::Path,
        learned: Vec<(String, u32)>,
    ) -> Self {
        let s = settings.clone();
        OptionsDialog {
            from: s.clone(),
            browser_rows: Default::default(),
            cat_dir: display_dir(&s.category_dir(Category::General, downloads)),
            conns: s.max_connections.clamp(1, s.connection_limit()).to_string(),
            mb: s.limits_mb.to_string(),
            min_kb: s.capture_min_kb.to_string(),
            hours: s.limits_hours.to_string(),
            s,
            page: 0,
            category: 0,
            exc_rows: Default::default(),
            login_rows: Default::default(),
            account_rows: Default::default(),
            sound_rows: Default::default(),
            sub: None,
            set_up_seen: 0,
            cat_edit: None,
            restore_ask: None,
            learned,
            seven_zip: crate::extractq::seven_zip(),
            adapters: None,
            remote: None,
            port: settings.remote_port.to_string(),
            device_rows: Default::default(),
            cookie_browser: cookie_browser_at(settings),
        }
    }

    /// `Done(settings)` on OK.
    pub fn show(&mut self, ctx: &egui::Context, downloads: &std::path::Path) -> Outcome<Settings> {
        let modal = self.sub.is_some() || self.cat_edit.is_some() || self.restore_ask.is_some();
        let outcome = window(
            ctx,
            "options",
            "Mister Download Manager Configuration",
            [W, H],
            false,
            LIGHT_BG(),
            |ui, o| {
                if let Some(p) = side_list(ui, o, self.page) {
                    self.page = p;
                }
                self.page_area(ui, o, downloads);
                // No Cancel (PLAN.md): Escape and the close box discard the changes, as before.
                // Escape and Enter keep their old order: Escape first.
                let escape = winui::escape_pressed(ui);
                let pressed = winui::footer(
                    ui,
                    o,
                    W,
                    H - winui::FOOTER_H,
                    // Tab order as before: OK, then Help.
                    &mut [
                        FooterItem::Button(
                            FooterButton::new("OK", ButtonKind::Default).enter(!modal),
                        ),
                        // i18n: noun: button that opens the help
                        FooterItem::Button(FooterButton::new("Help", ButtonKind::Ghost).leading()),
                    ],
                );
                if pressed == Some(1) {
                    crate::help::open_page("options.md");
                }
                if modal {
                    Outcome::Open
                } else if escape {
                    Outcome::Closed
                } else if pressed == Some(0) {
                    Outcome::Done(())
                } else {
                    Outcome::Open
                }
            },
        );
        if self.sub.is_some() {
            self.sub_dialog(ctx);
        }
        if let Some(d) = &mut self.cat_edit {
            match d.show(ctx, &crate::categories::names()) {
                Outcome::Open => {}
                Outcome::Closed => self.cat_edit = None,
                Outcome::Done(e) => {
                    self.cat_edit = None;
                    let cat = crate::categories::apply(&mut self.s, e, downloads);
                    // Show the category just made or changed.
                    if let Some(i) = Category::all().iter().position(|c| *c == cat) {
                        self.category = i;
                    }
                    self.cat_dir = display_dir(&self.s.category_dir(cat, downloads));
                }
            }
        }
        if let Some(d) = &mut self.restore_ask {
            match d.show(ctx) {
                Outcome::Open => {}
                Outcome::Closed => self.restore_ask = None,
                Outcome::Done(()) => {
                    self.restore_ask = None;
                    // In this draft: Cancel undoes it, as it undoes the dialog's other edits.
                    crate::categories::restore(&mut self.s);
                    let cats = Category::all();
                    self.category = self.category.min(cats.len() - 1);
                    self.cat_dir =
                        display_dir(&self.s.category_dir(cats[self.category], downloads));
                }
            }
        }
        match outcome {
            Outcome::Done(()) => {
                let mut s = self.s.clone();
                s.max_connections = self
                    .conns
                    .parse()
                    .unwrap_or(8)
                    .clamp(1, s.connection_limit());
                s.limits_mb = self.mb.parse().unwrap_or(200).max(1);
                s.capture_min_kb = self.min_kb.parse().unwrap_or(0);
                s.limits_hours = self.hours.parse().unwrap_or(5).max(1);
                s.remote_port = self.remote_port();
                Outcome::Done(s)
            }
            Outcome::Closed => Outcome::Closed,
            Outcome::Open => Outcome::Open,
        }
    }

    /// The chosen page under its heading. The pages keep their English pixels from the old tab
    /// frame (x 6..421): moved as a whole into the area right of the list ([`PAGE_AT`]).
    fn page_area(&mut self, ui: &mut Ui, o: Pos2, downloads: &std::path::Path) {
        heading(ui, o, self.page);
        let o = o + PAGE_AT;
        match self.page {
            0 => self.general(ui, o),
            1 => self.file_types(ui, o),
            2 => self.save_to(ui, o, downloads),
            3 => self.downloads(ui, o),
            4 => self.connection(ui, o),
            5 => self.proxy(ui, o),
            6 => self.logins(ui, o),
            7 => self.accounts(ui, o),
            8 => self.sounds(ui, o),
            9 => self.regional(ui, o),
            10 => self.archives(ui, o),
            REMOTE_PAGE => self.remote_page(ui, o),
            _ => self.regional(ui, o),
        }
    }

    // ---- pages --------------------------------------------------------------------------

    fn general(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let portable = mdm_ipc::portable_dir().is_some();
        let s = &mut self.s;
        if portable {
            s.launch_on_startup = false;
        }
        // The mock's check boxes, 22 apart (closer than its 30, so the page fits its area).
        let checks: [(&mut bool, &str); 7] = [
            (
                &mut s.launch_on_startup,
                crate::i18n::mark("Launch Mister Download Manager on startup"),
            ),
            (
                &mut s.clipboard_autostart,
                crate::i18n::mark("Offer to download links copied to the clipboard"),
            ),
            (
                &mut s.mv2_extensions,
                crate::i18n::mark("Extend the availability of Manifest 2 Chrome extensions"),
            ),
            (
                &mut s.advanced_integration,
                crate::i18n::mark("Use advanced browser integration"),
            ),
            // MDM's: turns back on the delete confirmation that "Don't show this dialog again"
            // turned off.
            (
                &mut s.confirm_delete,
                crate::i18n::mark("Confirm before deleting downloads"),
            ),
            // MDM's (#63): Minimize hides the main window and progress windows in the tray too.
            (
                &mut s.minimize_to_tray,
                // i18n: checkbox: the Minimize button hides MDM's windows to the notification area (system tray) instead of the taskbar
                crate::i18n::mark("Minimize to the notification area"),
            ),
            // MDM's: the × hides the window in the notification area, or quits.
            (
                &mut s.close_to_tray,
                // i18n: checkbox: closing keeps MDM running, in the notification area when available or else the taskbar, instead of quitting
                crate::i18n::mark("Keep running when closing the window"),
            ),
        ];
        let (left, right, h) = (PAGE_LEFT, PAGE_RIGHT, winui::CONTROL_H);
        for (i, (v, label)) in checks.into_iter().enumerate() {
            // Portable: nothing is written to the registry, so no start with Windows. Manifest 2
            // extensions: an IDM-era Chrome policy with no meaning for MDM's extension (MV3):
            // shown as IDM has it, greyed (D-021).
            let enabled = !(portable && i == 0) && i != 2;
            let at = o + vec2(left, PAGE_TOP + 22.0 * i as f32);
            let check = winui::checkbox_light(ui, at, v, label, enabled);
            // Where a start with the session leaves the window (#226).
            if i == 0 && enabled && check.hovered() {
                check.on_hover_text(
                    // i18n: tooltip of "Launch Mister Download Manager on startup": started with the computer, MDM's window stays hidden, its icon in the notification area (system tray)
                    crate::i18n::t("Started with the computer, it waits in the notification area"),
                );
            } else if i == 6 && check.hovered() {
                // i18n: tooltip of the close-setting: checked keeps MDM running when the window closes, in the notification area when available or else the taskbar; downloads continue; unchecked closing quits
                check.on_hover_text(crate::i18n::t(
                    "Checked, closing the window keeps MDM running: it hides in the notification area when available, or minimizes to the taskbar otherwise, and downloads continue; unchecked, closing quits MDM",
                ));
            }
        }
        // The browsers under a hairline and their title (the group box was), Test and Add
        // browser at the list's end (one width, as wide as the longer label in this language,
        // #192).
        // The list's top: its rows out of view are there for screen readers (#111), the twelve
        // default browsers' last one 206 below it, still inside the page's area.
        let list_y = 267.0;
        winui::rule(&p, o, left, 240.0, right - left);
        winui::form_label(
            &p,
            o,
            left,
            246.0,
            16.0,
            "Capture downloads from the following browsers:",
            true,
        );
        let (col_x, col_w) = winui::buttons_to_right(
            &p,
            right,
            0.0,
            &[(
                winui::widest_label(&p, &["Test browser integration...", "Add browser..."]),
                73.0,
            )],
            260.0,
        )[0];
        // Check list: one row per browser in the shared row list (Space ticks the selected
        // one; the box and its name tick it too).
        let browsers = &mut s.browsers;
        // Named by its group's caption, which says what the list is (#102).
        self.browser_rows.name = crate::i18n::t("Capture downloads from the following browsers:");
        let ev = winui::row_list(
            ui,
            "browsers",
            r(o, left, list_y, col_x - 8.0 - left, 78.0),
            browsers.len(),
            17.0,
            &mut self.browser_rows,
            |ui, clip, i, row, look| {
                let (name, on) = &mut browsers[i];
                look.name = name.clone();
                winui::row_check(
                    ui,
                    clip,
                    pos2(row.left() + 5.0, row.top() + 2.0),
                    on,
                    name,
                    ("browser", i),
                );
                // Its name in the row's ink (white on the selection).
                text(clip, row.left() + 23.0, row.top() + 12.0, name, look.ink);
            },
        );
        if let Some(i) = ev.space {
            browsers[i].1 = !browsers[i].1;
        }
        if ev.focused {
            self.browser_rows
                .type_to(ui, "browsers", browsers.len(), |i| browsers[i].0.clone());
        }
        // Adding browsers and the browser menu editor are the extension's settings (M8): shown as
        // IDM has them, not yet available.
        let off = |ui: &mut Ui, rc: Rect, label: &str| {
            winui::button_light(ui, rc, label, ButtonKind::Disabled, false);
        };
        off(
            ui,
            r(o, col_x, list_y + 36.0, col_w, h),
            crate::i18n::mark("Add browser..."),
        );
        // MDM's: what stands between a browser and MDM, checked in order.
        if winui::button_light(
            ui,
            r(o, col_x, list_y, col_w, h),
            "Test browser integration...",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.sub = Some(Sub::Test(crate::extension::checks(&self.s)));
        }
        winui::rule(&p, o, left, 357.0, right - left);
        // Three rows under a hairline: a sentence at the leading edge, its button at the end
        // (one width, as wide as the longer label in this language, #192).
        // i18n: verb: edit the item
        let edit = "Edit...";
        let (end_x, end_w) = winui::buttons_to_right(
            &p,
            right,
            0.0,
            &[(winui::widest_label(&p, &["Keys...", edit]), 73.0)],
            160.0,
        )[0];
        let caption = |ui: &Ui, top: f32, label: &str, color| {
            let (rect, cut) = winui::text_fit(
                &p,
                o.x + left,
                o.y + winui::baseline_in(top, h),
                label,
                color,
                o.x + end_x - 8.0,
            );
            if cut {
                winui::full_on_hover(ui, rect, ("general-right", top as i32), label);
            }
        };
        let (keys_y, menu_y, panels_y) = (369.0, 404.0, 439.0);
        caption(
            ui,
            keys_y,
            crate::i18n::t("Customize keys to prevent or force downloading with MDM"),
            LIGHT_TEXT(),
        );
        if winui::button_light(
            ui,
            r(o, end_x, keys_y, end_w, h),
            "Keys...",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.sub = Some(Sub::Keys(self.s.capture_opened));
        }
        caption(
            ui,
            menu_y,
            crate::i18n::t("Customize MDM menu items in context menu of browsers"),
            LIGHT_DISABLED(),
        );
        off(ui, r(o, end_x, menu_y, end_w, h), edit);
        // The download button over videos in the browsers (our own panel, Sub::Panels).
        caption(
            ui,
            panels_y,
            crate::i18n::t("Customize MDM Download panels in browsers"),
            LIGHT_TEXT(),
        );
        if winui::button_light(
            ui,
            r(o, end_x, panels_y, end_w, h),
            // i18n: verb: edit the item
            "Edit...",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.sub = Some(Sub::Panels(
                self.s.video_panel,
                self.s.video_panel_sites.clone(),
                String::new(),
            ));
        }
    }

    fn file_types(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let (left, right, h) = (PAGE_LEFT, PAGE_RIGHT, winui::CONTROL_H);
        // A sentence at the leading edge, centred on a row `top`..`top + height`.
        let line = |top: f32, height: f32, s: &str| {
            text(
                &p,
                o.x + left,
                o.y + winui::baseline_in(top, height),
                s,
                LIGHT_TEXT(),
            )
        };
        line(
            88.0,
            16.0,
            crate::i18n::t("Automatically start downloading the following file types:"),
        );
        multiline(
            ui,
            r(o, left, 108.0, right - left, 72.0),
            &mut self.s.auto_types,
            "auto-types",
            true,
        );
        // Default as wide as its label in this language (#192), at the end.
        let (default_x, default_w) =
            winui::buttons_to_right(&p, right, 0.0, &[("Default", 73.0)], 200.0)[0];
        if winui::button_light(
            ui,
            r(o, default_x, 186.0, default_w, h),
            // i18n: button: put back the default list
            "Default",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.s.auto_types = settings::DEFAULT_AUTO_TYPES.into();
        }
        line(
            224.0,
            16.0,
            crate::i18n::t("Don't start downloading automatically from the following sites:"),
        );
        multiline(
            ui,
            r(o, left, 244.0, right - left, 44.0),
            &mut self.s.excluded_sites,
            "excluded-sites",
            true,
        );
        // The note, and Default at the end of its row (the mock's line).
        line(294.0, h, crate::i18n::t("(separate names by spaces)"));
        if winui::button_light(
            ui,
            r(o, default_x, 294.0, default_w, h),
            // i18n: button: put back the default list
            "Default",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.s.excluded_sites = settings::DEFAULT_EXCLUDED_SITES.into();
        }
        // The last two rows as the mock's lines: a sentence, its control right after it (the
        // control kept inside the page; a longer translation cut with "...", whole on hover).
        let after = |s: &str, w: f32| (left + winui::text_width(&p, s) + 8.0).min(right - w);
        let lead = |ui: &Ui, top: f32, s: &str, end: f32| {
            let (rect, cut) = winui::text_fit(
                &p,
                o.x + left,
                o.y + winui::baseline_in(top, h),
                s,
                LIGHT_TEXT(),
                o.x + end - 8.0,
            );
            if cut {
                winui::full_on_hover(ui, rect, ("types-lead", top as i32), s);
            }
        };
        let label =
            crate::i18n::t("Don't start downloading automatically from the following addresses:");
        let list_w = winui::button_fit_width(&p, "Edit list ...", 73.0, 260.0);
        let list_x = after(label, list_w);
        lead(ui, 336.0, label, list_x);
        if winui::button_light(
            ui,
            r(o, list_x, 336.0, list_w, h),
            "Edit list ...",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.sub = Some(Sub::Addresses(self.s.excluded_addresses.join("\n")));
        }
        // MDM's: a receipt or an icon isn't worth a download window.
        let label = crate::i18n::t("Don't take files smaller than (KB, 0 = any size):");
        let min_x = after(label, 73.0);
        lead(ui, 374.0, label, min_x);
        winui::number_field(
            ui,
            r(o, min_x, 374.0, 73.0, h),
            &mut self.min_kb,
            0,
            10_000_000,
            true,
            LIGHT_PAGE(),
        );
    }

    fn save_to(&mut self, ui: &mut Ui, o: Pos2, downloads: &std::path::Path) {
        let p = ui.painter().clone();
        let (left, right, h) = (PAGE_LEFT, PAGE_RIGHT, winui::CONTROL_H);
        // IDM's, Torrents and the user's (the registry follows this dialog's edits).
        let cats = Category::all();
        self.category = self.category.min(cats.len() - 1);
        let cat = cats[self.category];
        let label = cat.shown();
        let with_category = |template: &str| crate::i18n::fmt(template, &[("category", &label)]);
        let dir_line = with_category(crate::i18n::t(
            "Default download directory for \"{category}\" category",
        ));
        // The mock's wide form (240): Category, the folder, the temporary directory.
        let fx = winui::form(
            &p,
            left,
            &["Category", &dir_line, "Temporary directory"],
            true,
        );
        // The group box's caption as a title over its rows.
        winui::form_label(&p, o, left, 88.0, 16.0, "Save To...", true);
        // i18n: noun: label before the list of categories
        winui::form_label(&p, o, left, 110.0, h, "Category", true);
        // New / Edit... (D-021): a category of the user's; a built-in one's folder. One width for
        // both, as wide as the longer label in this language (#192), after the list.
        let col_w = winui::button_fit_width(
            &p,
            winui::widest_label(&p, &["New", "Edit..."]),
            73.0,
            142.0,
        );
        let (edit_x, new_x) = (right - col_w, right - 2.0 * col_w - 8.0);
        let names: Vec<String> = cats.iter().map(|c| c.label().into_owned()).collect();
        let labels: Vec<&str> = names.iter().map(String::as_str).collect();
        if let Some(i) = winui::combo_light(
            ui,
            r(o, fx, 110.0, new_x - 8.0 - fx, h),
            &labels,
            self.category,
            true,
            "opt-category",
        ) {
            // Keep the folder typed for the category being left.
            self.commit_cat_dir(downloads);
            self.category = i;
            self.cat_dir = display_dir(&self.s.category_dir(cats[i], downloads));
        }
        if winui::button_light(
            ui,
            r(o, new_x, 110.0, col_w, h),
            // i18n: button: create a new item
            "New",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.commit_cat_dir(downloads);
            self.cat_edit = Some(crate::categories::CategoryDialog::new());
        }
        if winui::button_light(
            ui,
            r(o, edit_x, 110.0, col_w, h),
            // i18n: verb: edit the item
            "Edit...",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.commit_cat_dir(downloads);
            let cat = cats[self.category];
            let folder = self.s.category_dir(cat, downloads);
            self.cat_edit = Some(crate::categories::CategoryDialog::edit(cat, &folder));
        }
        // The category chosen this frame names the rows after it.
        let cat = cats[self.category];
        let label = cat.shown();
        let with_category = |template: &str| crate::i18n::fmt(template, &[("category", &label)]);
        let line = with_category(crate::i18n::t(
            "Automatically put in \"{category}\" category the following file types:",
        ));
        winui::text_in_window(
            ui,
            &p,
            o.x + left,
            o.y + winui::baseline_in(150.0, 16.0),
            &line,
            LIGHT_TEXT(),
        );
        let types = cat.types_text();
        readonly(ui, r(o, left, 170.0, right - left, h), &types);
        let dir_line = with_category(crate::i18n::t(
            "Default download directory for \"{category}\" category",
        ));
        winui::form_label(&p, o, left, 210.0, h, &dir_line, true);
        // Browse as wide as its label in this language (#192); the box ends 8 px before it.
        let (browse_x, browse_w) =
            winui::buttons_to_right(&p, right, 0.0, &[("Browse", 73.0)], 160.0)[0];
        winui::edit_light_on(
            ui,
            r(o, fx, 210.0, browse_x - 8.0 - fx, h),
            &mut self.cat_dir,
            true,
            false,
            LIGHT_PAGE(),
        );
        if winui::button_light(
            ui,
            r(o, browse_x, 210.0, browse_w, h),
            // i18n: verb: choose a folder
            "Browse",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            let (title, from) = (dir_line.clone(), self.cat_dir.clone());
            winui::start_pick(ui.ctx(), "options-category-folder", move || {
                rfd::FileDialog::new()
                    .set_title(title)
                    .set_directory(from)
                    .pick_folder()
            });
        }
        if let Some(dir) = winui::picked(ui.ctx(), "options-category-folder") {
            self.cat_dir = display_dir(&dir);
        }
        self.commit_cat_dir(downloads);
        let line = with_category(crate::i18n::t(
            "Change folder for \"{category}\" category on last selected",
        ));
        winui::checkbox_light(
            ui,
            o + vec2(left, 252.0),
            &mut self.s.change_folder_on_last,
            &line,
            true,
        );
        winui::checkbox_light(
            ui,
            o + vec2(left, 274.0),
            &mut self.s.set_creation_date,
            "Set file creation date as provided by the server",
            true,
        );
        // The temporary directory under a hairline (its group box was), a row of the form.
        winui::rule(&p, o, left, 297.0, right - left);
        winui::form_label(&p, o, left, 305.0, h, "Temporary directory", true);
        // D-004: no temporary directory — parts are written next to the file, so there is
        // nothing to assemble at the end.
        let none = crate::i18n::t("Not used: file parts are written next to the file");
        readonly(ui, r(o, fx, 305.0, browse_x - 8.0 - fx, h), none);
        winui::button_light(
            ui,
            r(o, browse_x, 305.0, browse_w, h),
            // i18n: verb: choose a folder
            "Browse",
            ButtonKind::Disabled,
            false,
        );
        winui::paragraph(
            &p,
            o.x + left,
            o.y + 355.0,
            crate::i18n::t(
                "Mister Download Manager writes every connection straight into the final\nfile (a sparse part file next to it), so no temporary directory and no\nassembling step are needed.",
            ),
            LIGHT_TEXT(),
            o.x + right,
            13.0,
        );
        // #93: the built-in categories by file type as shipped again (on this page only: at
        // its foot, since the footer is the window's).
        let restore_w = winui::button_fit_width(&p, "Restore the built-in categories", 73.0, 400.0);
        if winui::button_light(
            ui,
            r(o, left, PAGE_BOTTOM - h, restore_w, h),
            "Restore the built-in categories",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            self.commit_cat_dir(downloads);
            let text = crate::i18n::fmt(
                crate::i18n::t(
                    "Bring back {names} as they were shipped: their names, file types and folders? Your own categories, General and Torrents stay as they are. Downloads already moved to General stay there.",
                ),
                &[("names", &crate::categories::restored_names())],
            );
            self.restore_ask = Some(crate::dialogs::Confirm::new(
                crate::i18n::mark("Restore the built-in categories"),
                text,
            ));
        }
    }

    /// The typed folder for the selected category becomes its own (or the default again).
    fn commit_cat_dir(&mut self, downloads: &std::path::Path) {
        let cats = Category::all();
        let Some(&cat) = cats.get(self.category) else {
            return;
        };
        let base = if cat == Category::General {
            std::borrow::Cow::Borrowed(downloads)
        } else {
            crate::model::default_downloads(downloads, &self.s.category_dirs)
        };
        let default = display_dir(&cat.default_folder(&base));
        let typed = self.cat_dir.trim();
        let own = (!typed.is_empty()
            && typed.trim_end_matches(['\\', '/']) != default.trim_end_matches(['\\', '/']))
        .then(|| typed.to_owned());
        self.s.set_category_dir(cat, own);
    }

    fn downloads(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let s = &mut self.s;
        let (left, right, h) = (PAGE_LEFT, PAGE_RIGHT, winui::CONTROL_H);
        let w = right - left;
        // Edit... as wide as its label in this language (#192) at the end of its row, its
        // sentence at the leading edge (greyed, as the button: neither is built yet).
        let (edit_x, edit_w) =
            winui::buttons_to_right(&p, right, 0.0, &[("Edit...", 73.0)], 160.0)[0];
        let greyed = |top: f32, s: &str| {
            winui::text_fit(
                &p,
                o.x + left,
                o.y + winui::baseline_in(top, h),
                s,
                LIGHT_DISABLED(),
                o.x + edit_x - 8.0,
            )
        };
        greyed(
            88.0,
            crate::i18n::t("Customize \"Download progress\" dialog"),
        );
        // The progress dialog customisation dialog is not built yet.
        winui::button_light(
            ui,
            r(o, edit_x, 88.0, edit_w, h),
            // i18n: verb: edit the item
            "Edit...",
            ButtonKind::Disabled,
            false,
        );
        // The mock's check boxes, 22 apart (closer than its 30, so the page fits its area).
        winui::checkbox_light(
            ui,
            o + vec2(left, 126.0),
            &mut s.show_start_dialog,
            "Show start download dialog",
            true,
        );
        let on = s.show_start_dialog;
        winui::checkbox_light(
            ui,
            o + vec2(left + 24.0, 148.0),
            &mut s.only_add_to_queue,
            "Do not start downloading, only add files to the queue",
            !on,
        );
        winui::checkbox_light(
            ui,
            o + vec2(left, 170.0),
            &mut s.show_complete,
            "Show download complete dialog",
            true,
        );
        text(
            &p,
            o.x + left,
            o.y + winui::baseline_in(190.0, 16.0),
            crate::i18n::t("Note: These settings don't relate to queue processing"),
            LIGHT_TEXT(),
        );
        winui::rule(&p, o, left, 214.0, w);
        let checks: [(&mut bool, &str); 4] = [
            (
                &mut s.start_immediately,
                "Start downloading immediately while displaying \"Download File Info\" dialog",
            ),
            (
                &mut s.queue_panel_later,
                "Show queue selection panel on pressing \"Download Later\" button",
            ),
            (
                &mut s.queue_panel_batch,
                "Show queue selection panel on closing batch download dialogs",
            ),
            (
                &mut s.ignore_mtime,
                "Ignore file modification time changes when resuming a download",
            ),
        ];
        for (i, (v, label)) in checks.into_iter().enumerate() {
            winui::checkbox_light(ui, o + vec2(left, 222.0 + 22.0 * i as f32), v, label, true);
        }
        winui::rule(&p, o, left, 311.0, w);
        // The mock's wide form (240): the duplicate's choice and the User-Agent.
        let dup = "If a duplicate download link is added:";
        let agent = "User-Agent for manually added downloads:";
        let fx = winui::form(&p, left, &[dup, agent], true);
        winui::form_label(&p, o, left, 319.0, h, dup, true);
        if let Some(i) = winui::combo_light(
            ui,
            r(o, fx, 319.0, right - fx, h),
            &DUPLICATE,
            s.duplicate_action as usize,
            true,
            "opt-duplicate",
        ) {
            s.duplicate_action = i as u8;
        }
        winui::form_label(&p, o, left, 359.0, h, agent, true);
        winui::edit_light(
            ui,
            r(o, fx, 359.0, right - fx, h),
            &mut s.user_agent,
            true,
            false,
        );
        winui::rule(&p, o, left, 399.0, w);
        greyed(407.0, crate::i18n::t("Virus checking settings"));
        winui::button_light(
            ui,
            r(o, edit_x, 407.0, edit_w, h),
            // i18n: verb: edit the item
            "Edit...",
            ButtonKind::Disabled,
            false,
        );
    }

    fn connection(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let (left, right, h) = (PAGE_LEFT, PAGE_RIGHT, winui::CONTROL_H);
        // The group boxes' captions as titles, their groups between hairlines (the mock's).
        let max = "Default max. conn. number";
        // i18n: label before the choice of DNS (the service that turns names into addresses)
        let dns = "DNS:";
        let fx = winui::form(&p, left, &[max, dns], false);
        winui::form_label(&p, o, left, 88.0, 16.0, "Max. connections number", true);
        winui::form_label(&p, o, left, 108.0, h, max, true);
        // Typed, or stepped with the arrows (IDM's drop-down offered 1 to 32; issue #12).
        winui::spin_box(
            ui,
            r(o, fx, 108.0, 90.0, h),
            &mut self.conns,
            1,
            u64::from(self.s.connection_limit()),
            true,
            "opt-conns",
        );
        let locked = mdm_engine::download::MAX_CONNECTIONS;
        let line = crate::i18n::fmt(
            // i18n: checkbox under the number of connections per download; {n} is 32
            crate::i18n::t("Allow more than {n} connections"),
            &[("n", &crate::i18n::integer(locked as u64))],
        );
        let was = self.s.many_connections;
        winui::checkbox_light(
            ui,
            o + vec2(left, 146.0),
            &mut self.s.many_connections,
            &line,
            true,
        );
        // Locked again: what went beyond comes back to the ceiling.
        if was && !self.s.many_connections {
            if self.conns.parse::<u32>().is_ok_and(|n| n > locked) {
                self.conns = locked.to_string();
            }
            for (_, n) in &mut self.s.conn_exceptions {
                *n = (*n).min(locked);
            }
        }
        // i18n: plural noun: label before a list of exceptions
        winui::form_label(&p, o, left, 167.0, 16.0, "Exceptions:", true);
        // The user's exceptions, then the caps the engine learned (read-only "(auto)" rows).
        let rows: Vec<Vec<String>> = self
            .s
            .conn_exceptions
            .iter()
            .map(|(server, n)| vec![server.clone(), n.to_string()])
            .chain(
                self.learned
                    .iter()
                    .filter(|(h, _)| {
                        !self
                            .s
                            .conn_exceptions
                            .iter()
                            .any(|(s, _)| s.eq_ignore_ascii_case(h))
                    })
                    .map(|(h, n)| {
                        // i18n: a number of connections the app learned by itself for a server
                        let auto = crate::i18n::t("{n} (auto)");
                        vec![
                            h.clone(),
                            crate::i18n::fmt(auto, &[("n", &crate::i18n::integer(u64::from(*n)))]),
                        ]
                    }),
            )
            .collect();
        // New, Delete and Edit: one width, as wide as the longest label in this language (#192),
        // the list ending 8 px before them (its Number column 20 short of the list's end).
        let (col_x, col_w) = winui::buttons_to_right(
            &p,
            right,
            0.0,
            &[(winui::widest_label(&p, &["New", "Delete", "Edit"]), 73.0)],
            92.0,
        )[0];
        let list_end = col_x - 8.0;
        let (dbl, del_key) = report(
            ui,
            o,
            r(o, left, 187.0, list_end - left, 98.0),
            // i18n: column titles: the server, and the number of connections allowed to it
            &[
                (crate::i18n::mark("Server"), list_end - 90.0),
                (crate::i18n::mark("Number"), list_end - 20.0),
            ],
            &rows,
            &mut self.exc_rows,
            true,
            "opt-exceptions",
        );
        // Learned rows can't be edited or deleted (they fade by themselves).
        let sel = self
            .exc_rows
            .selected
            .filter(|i| *i < self.s.conn_exceptions.len());
        let new = winui::button_light(
            ui,
            r(o, col_x, 187.0, col_w, h),
            // i18n: button: create a new item
            "New",
            ButtonKind::Normal,
            false,
        )
        .clicked();
        let kind = if sel.is_some() {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        // i18n: verb: delete the item
        let del = winui::button_light(ui, r(o, col_x, 221.0, col_w, h), "Delete", kind, false)
            .clicked()
            || del_key;
        let edit =
            // i18n: verb: edit the item
            winui::button_light(ui, r(o, col_x, 255.0, col_w, h), "Edit", kind, false)
                .clicked();
        if new {
            self.sub = Some(Sub::Exception(None, String::new(), "1".into()));
        }
        if let Some(i) = sel {
            if del {
                self.s.conn_exceptions.remove(i);
                self.exc_rows.selected = None;
            } else if edit || dbl {
                let (server, n) = self.s.conn_exceptions[i].clone();
                self.sub = Some(Sub::Exception(Some(i), server, n.to_string()));
            }
        }
        // Download limits (shared with the Scheduler's page): its check box leads the group,
        // the sentence and the warning indented under it (the mock's).
        winui::rule(&p, o, left, 295.0, right - left);
        winui::checkbox_light(
            ui,
            o + vec2(left, 303.0),
            &mut self.s.limits_on,
            "Download limits",
            true,
        );
        let on = self.s.limits_on;
        // Each line's words on its box's text baseline; the labels end at one column, 8 before
        // the boxes.
        let line = |y: f32, gap: f32| {
            let baseline = o.y + winui::baseline_in(y, 24.0);
            winui::SentenceLine {
                lead: winui::Lead::Right(o.x + left + 150.0),
                lead_baseline: baseline,
                boxes: o + vec2(left + 158.0, y),
                gap,
                baseline,
            }
        };
        let boxes = winui::sentence(
            ui,
            &p,
            crate::i18n::t(crate::quota::LIMITS_SENTENCE),
            &[line(322.0, 8.0), line(350.0, 11.0)],
            &[("mb", vec2(64.0, 24.0)), ("hours", vec2(64.0, 24.0))],
            |p, x, baseline, s| {
                winui::label(p, x, baseline, s, on);
            },
        );
        winui::number_field(ui, boxes[0], &mut self.mb, 1, 999_999, on, LIGHT_PAGE());
        winui::number_field(ui, boxes[1], &mut self.hours, 1, 999, on, LIGHT_PAGE());
        winui::checkbox_light(
            ui,
            o + vec2(left + 24.0, 382.0),
            &mut self.s.limits_warn,
            "Show warning before stopping downloads",
            on,
        );
        // DNS (#36): who answers the names; the details in Settings....
        winui::rule(&p, o, left, 405.0, right - left);
        winui::form_label(&p, o, left, 413.0, h, dns, true);
        // Settings... as wide as its label in this language (#192) at the row's end.
        let (set_x, set_w) =
            winui::buttons_to_right(&p, right, 0.0, &[("Settings...", 73.0)], 160.0)[0];
        let names = dns_names();
        let labels: Vec<&str> = names.iter().map(String::as_str).collect();
        if let Some(i) = winui::combo_light(
            ui,
            r(o, fx, 413.0, set_x - 8.0 - fx, h),
            &labels,
            dns_index(&self.s),
            true,
            "opt-dns",
        ) {
            set_dns_index(&mut self.s, i);
            if i > 0 {
                self.sub = Some(Sub::Dns(DnsEdit::from(&self.s)));
            }
        }
        let kind = if self.s.dns_mode == 0 {
            ButtonKind::Disabled
        } else {
            ButtonKind::Normal
        };
        // i18n: button: opens the DNS's details (how it's asked, its servers, a test)
        if winui::button_light(ui, r(o, set_x, 413.0, set_w, h), "Settings...", kind, false)
            .clicked()
        {
            self.sub = Some(Sub::Dns(DnsEdit::from(&self.s)));
        }
    }

    fn proxy(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        // Get System as wide as its label (#192), its right end kept, on the heading's row
        // (the heading is the window's, `heading`).
        let (get_x, get_w) = winui::button_row(
            &p,
            &[("Get System", PROXY_GET_X, 93.0)],
            0.0,
            PROXY_GET_X,
            160.0,
        )[0];
        // On whole pixels: a half-pixel button's edges blur.
        let get_y = HEAD_TOP + ((HEAD_TILE - 21.0) / 2.0).floor() - PAGE_AT.y;
        let s = &mut self.s;
        if winui::button_light(
            ui,
            r(o, get_x, get_y, get_w, 21.0),
            "Get System",
            ButtonKind::Normal,
            false,
        )
        .clicked()
            && let Some((host, port)) = system_proxy()
        {
            s.proxy_mode = 3;
            s.proxy_host = host;
            s.proxy_port = port;
            s.proxy_http = true;
            s.proxy_https = true;
        }
        winui::checkbox_light(
            ui,
            o + vec2(30.0, 100.0),
            &mut s.proxy_from_browser,
            "",
            true,
        );
        // The checkbox's caption, in two lines as IDM has it (clicking it ticks the box).
        let lines = winui::paragraph(
            &p,
            o.x + 46.0,
            o.y + 104.0,
            crate::i18n::t(
                "Use proxy/socks from a browser's request if there are download errors /\nfor the downloads intercepted from the browser",
            ),
            LIGHT_TEXT(),
            o.x + 426.0,
            13.0,
        );
        winui::caption_click(
            ui,
            r(o, 46.0, 99.0, 380.0, 13.0 * lines as f32 + 2.0),
            "proxy-from-browser",
            &mut s.proxy_from_browser,
            true,
        );
        winui::etched_h(&p, o.x + 22.0, o.y + 128.0, 380.0);
        let modes = [
            (21.0, 137.0, crate::i18n::mark("No proxy/socks")),
            (21.0, 167.0, crate::i18n::mark("Use system settings")),
            (
                21.0,
                194.0,
                crate::i18n::mark("Use automatic configuration script"),
            ),
            (
                22.0,
                250.0,
                crate::i18n::mark("Manual proxy/socks configuration"),
            ),
        ];
        let mut group = Vec::new();
        for (i, (x, y, label)) in modes.iter().enumerate() {
            let resp = winui::radio_light(
                ui,
                o + vec2(*x, *y),
                s.proxy_mode as usize == i,
                label,
                true,
            );
            if resp.clicked() {
                s.proxy_mode = i as u8;
            }
            group.push(resp);
        }
        if let Some(i) = winui::radio_keys(ui, &group) {
            s.proxy_mode = i as u8;
        }
        winui::etched_h(&p, o.x + 22.0, o.y + 159.0, 380.0);
        winui::etched_h(&p, o.x + 21.0, o.y + 187.0, 380.0);
        winui::etched_h(&p, o.x + 21.0, o.y + 242.0, 380.0);
        // The script's address (http, https or file): MDM fetches and runs it (pacfetch.rs,
        // mdm_engine::pac).
        let script = s.proxy_mode == 2;
        // i18n: noun: the web address (URL)
        disabled_label(&p, o.x + 30.0, o.y + 228.0, "Address", script, false);
        winui::edit_light_on(
            ui,
            r(o, 90.0, 213.0, 305.0, 23.0),
            &mut s.proxy_script,
            script,
            false,
            LIGHT_PAGE(),
        );
        let manual = s.proxy_mode == 3;
        disabled_label(
            &p,
            o.x + 30.0,
            o.y + 280.0,
            "Proxy server address",
            manual,
            false,
        );
        disabled_label(&p, o.x + 172.0, o.y + 278.0, "Port", manual, false);
        disabled_label(&p, o.x + 229.0, o.y + 278.0, "UserName", manual, false);
        disabled_label(&p, o.x + 313.0, o.y + 278.0, "Password", manual, false);
        winui::edit_light_on(
            ui,
            r(o, 30.0, 286.0, 137.0, 24.0),
            &mut s.proxy_host,
            manual,
            false,
            LIGHT_PAGE(),
        );
        winui::number_field(
            ui,
            r(o, 172.0, 286.0, 51.0, 24.0),
            &mut s.proxy_port,
            1,
            65535,
            manual,
            LIGHT_PAGE(),
        );
        winui::edit_light_on(
            ui,
            r(o, 229.0, 286.0, 78.0, 24.0),
            &mut s.proxy_user,
            manual,
            false,
            LIGHT_PAGE(),
        );
        password(
            ui,
            r(o, 312.0, 286.0, 83.0, 24.0),
            &mut s.proxy_pass,
            manual,
        );
        disabled_label(
            &p,
            o.x + 30.0,
            o.y + 326.0,
            "Use this proxy for the following protocols:",
            manual,
            false,
        );
        winui::checkbox_light(ui, o + vec2(30.0, 334.0), &mut s.proxy_http, "http", manual);
        winui::checkbox_light(
            ui,
            o + vec2(97.0, 334.0),
            &mut s.proxy_https,
            "https",
            manual,
        );
        // FTP connects to its servers directly, never through an HTTP proxy: the ftp box
        // stays greyed (D-021).
        winui::checkbox_light(ui, o + vec2(174.0, 334.0), &mut s.proxy_ftp, "ftp", false);
        // Socks settings: with the proxy work in M10.
        winui::button_light(
            ui,
            r(o, 235.0, 339.0, 159.0, 21.0),
            "Advanced / Socks...",
            ButtonKind::Disabled,
            false,
        );
        winui::etched_h(&p, o.x + 21.0, o.y + 447.0, 381.0);
        winui::checkbox_light(
            ui,
            o + vec2(27.0, 453.0),
            &mut s.ftp_passive,
            "Use FTP in PASV mode",
            true,
        );
        self.adapter_row(ui, o);
    }

    /// Proxy / Socks → Network adapter (#35): the adapter every connection goes through (a VPN's),
    /// and what it is doing now. The list is the computer's adapters, read every 2 s; the one
    /// chosen stays in it while it's gone, "(not connected)".
    fn adapter_row(&mut self, ui: &mut Ui, o: Pos2) {
        use mdm_engine::Route;
        let stale = self
            .adapters
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() >= std::time::Duration::from_secs(2));
        if stale {
            self.adapters = Some((std::time::Instant::now(), crate::netwatch::adapters()));
        }
        let adapters = self
            .adapters
            .as_ref()
            .map(|(_, a)| a.clone())
            .unwrap_or_default();
        let p = ui.painter().clone();
        winui::etched_h(&p, o.x + 21.0, o.y + 372.0, 381.0);
        // i18n: label before the choice of network adapter (Ethernet, Wi-Fi, a VPN's) every connection must go through
        text(
            &p,
            o.x + 27.0,
            o.y + 395.0,
            crate::i18n::t("Network adapter:"),
            LIGHT_TEXT(),
        );
        // The choices: any, then each adapter with its address (the ones with none too: a VPN
        // switched off), the chosen one kept when it's gone.
        let chosen = self.s.net_adapter.trim().to_owned();
        let mut values: Vec<String> = vec![String::new()];
        let mut labels: Vec<String> =
            // i18n: Options → Network adapter: the first choice: no adapter is chosen, the system picks one for each connection
            vec![crate::i18n::t("Any (the system chooses)").to_owned()];
        for a in &adapters {
            let ip =
                a.v4.first()
                    .map(|v| v.to_string())
                    .or(a.v6.first().map(|v| v.to_string()));
            values.push(a.name.clone());
            labels.push(match ip {
                Some(ip) => format!("{} \u{2014} {ip}", a.name),
                None => crate::i18n::fmt(
                    // i18n: Options → Network adapter: an adapter with no address now (a VPN switched off, a cable out); {adapter} its name
                    crate::i18n::t("{adapter} (not connected)"),
                    &[("adapter", &a.name)],
                ),
            });
        }
        if !chosen.is_empty() && !values.iter().any(|v| v.eq_ignore_ascii_case(&chosen)) {
            values.push(chosen.clone());
            labels.push(crate::i18n::fmt(
                crate::i18n::t("{adapter} (not connected)"),
                &[("adapter", &chosen)],
            ));
        }
        let at = values
            .iter()
            .position(|v| v.eq_ignore_ascii_case(&chosen))
            .unwrap_or(0);
        let shown: Vec<&str> = labels.iter().map(String::as_str).collect();
        if let Some(i) = winui::combo_light(
            ui,
            r(o, 130.0, 380.0, 265.0, 21.0),
            &shown,
            at,
            true,
            "opt-adapter",
        ) {
            self.s.net_adapter = values[i].clone();
        }
        // What it does now.
        let (line, ok) = match mdm_engine::net::route_for(&self.s.net_adapter, &adapters) {
            Route::Any => (
                // i18n: Options → Network adapter: what "Any" means
                crate::i18n::t("Each connection takes the system's route.").to_owned(),
                true,
            ),
            Route::Down { label } => (
                crate::i18n::fmt(
                    // i18n: Options → Network adapter: the chosen adapter has no address now; nothing is sent until it's back; {adapter} its name
                    crate::i18n::t(
                        "{adapter} is not connected: nothing is downloaded until it is back.",
                    ),
                    &[("adapter", &label)],
                ),
                false,
            ),
            Route::Bound(b) => {
                let ip = b.v4.map(|v| v.to_string()).or(b.v6.map(|v| v.to_string()));
                let line = crate::i18n::fmt(
                    // i18n: Options → Network adapter: every connection goes out through the chosen adapter, from this address (none goes another way); {ip} its address
                    crate::i18n::t("Every connection goes through it, from {ip}."),
                    &[("ip", &crate::i18n::ltr(&ip.unwrap_or_default()))],
                );
                // Downloads, torrents (#143), video pages and galleries (through the helpers'
                // relay, #35) all go through it on every system: nothing more to say.
                (line, true)
            }
        };
        let color = if ok {
            LIGHT_TEXT()
        } else {
            crate::theme::colors().error_strong
        };
        // Up to two lines, each cut with "..." (whole on hover) when a translation is longer.
        for (row, part) in line.lines().take(2).enumerate() {
            let (rect, cut) = winui::label_into(
                &p,
                o.x + 27.0,
                o.y + 422.0 + 15.0 * row as f32,
                part,
                color,
                o.x + 400.0,
                o.x + 27.0,
            );
            if cut {
                winui::full_on_hover(ui, rect, ("opt-adapter-state", row), part);
            }
        }
    }

    /// Accounts (#44, ours): debrid service accounts that download file-host links. The list is
    /// asked in its order; a link goes through the first account whose service takes its host.
    fn accounts(&mut self, ui: &mut Ui, o: Pos2) {
        let vault = crate::secrets::vault();
        let rows: Vec<Vec<String>> = self
            .s
            .debrid_accounts
            .iter()
            .map(|a| {
                vec![
                    a.service.name().to_owned(),
                    a.user.clone(),
                    if a.account().is_none() {
                        if vault.unread(&format!("debrid/{}", a.id))
                            && vault.kept() == crate::secrets::Kept::Locked
                        {
                            // i18n: Options → Accounts: the account's sign-in is in the system's keyring, which is locked now; MDM never unlocks it
                            crate::i18n::t("in the locked keyring").to_owned()
                        } else {
                            // i18n: Options → Accounts: the account's sign-in can't be read (signed out, or the settings came from another computer)
                            crate::i18n::t("signed out: sign in again").to_owned()
                        }
                    } else {
                        a.state.clone()
                    },
                ]
            })
            .collect();
        let (_, del_key) = report(
            ui,
            o,
            r(o, 21.0, 93.0, 383.0, 124.0),
            &[
                // i18n: column title: the debrid service (Real-Debrid…)
                (crate::i18n::mark("Service"), 110.0),
                // i18n: column title: the account's name at the service
                (crate::i18n::mark("Account"), 205.0),
                // i18n: column title: the account's state (premium, days left)
                (crate::i18n::mark("State"), 387.0),
            ],
            &rows,
            &mut self.account_rows,
            false,
            "opt-accounts",
        );
        let sel = self.account_rows.selected.filter(|i| *i < rows.len());
        // i18n: button: add a debrid service account (opens the sign-in)
        let add = winui::button_light(
            ui,
            r(o, 21.0, 225.0, 90.0, 21.0),
            "Add...",
            ButtonKind::Normal,
            false,
        )
        .clicked();
        let kind = |on: bool| {
            if on {
                ButtonKind::Normal
            } else {
                ButtonKind::Disabled
            }
        };
        let remove = winui::button_light(
            ui,
            r(o, 117.0, 225.0, 90.0, 21.0),
            "Remove",
            kind(sel.is_some()),
            false,
        )
        .clicked()
            || del_key;
        // i18n: button (short): move the account up the list (accounts are asked in the list's order)
        let up = winui::button_light(
            ui,
            r(o, 213.0, 225.0, 90.0, 21.0),
            "Up",
            kind(sel.is_some_and(|i| i > 0)),
            false,
        )
        .clicked();
        // i18n: button (short): move the account down the list
        let down = winui::button_light(
            ui,
            r(o, 309.0, 225.0, 95.0, 21.0),
            "Down",
            kind(sel.is_some_and(|i| i + 1 < rows.len())),
            false,
        )
        .clicked();
        if add {
            self.sub = Some(Sub::Account(AccountEdit::default()));
        }
        if let Some(i) = sel {
            let list = &mut self.s.debrid_accounts;
            if remove {
                list.remove(i);
                self.account_rows.selected = None;
            } else if up && i > 0 {
                list.swap(i, i - 1);
                self.account_rows.selected = Some(i - 1);
            } else if down && i + 1 < list.len() {
                list.swap(i, i + 1);
                self.account_rows.selected = Some(i + 1);
            }
        }
        // Experimental until each service is confirmed with a real account (#44).
        let p = ui.painter().clone();
        let note = crate::i18n::t(
            // i18n: Options → Accounts: a note that the debrid services' support (Real-Debrid, AllDebrid, Premiumize) is new and not yet tried with real paid accounts
            "Experimental: built from the services' public API documentation, not yet confirmed with real accounts.",
        );
        winui::paragraph(
            &p,
            o.x + 21.0,
            o.y + 256.0,
            note,
            crate::theme::colors().warning,
            o.x + 404.0,
            14.0,
        );
        winui::checkbox_light(
            ui,
            o + vec2(21.0, 302.0),
            &mut self.s.debrid_on,
            // i18n: Options → Accounts: checkbox: links of the file hosts these services support are downloaded through the accounts
            "Download supported file-host links through these accounts",
            true,
        );
        winui::checkbox_light(
            ui,
            o + vec2(21.0, 324.0),
            &mut self.s.debrid_torrents,
            // i18n: Options → Accounts: checkbox: magnet links and .torrent files go to the first account's service, which downloads them; MDM then downloads the files from it (no connections to peers)
            "Send torrents and magnet links to the first account instead of to peers",
            self.s.debrid_on,
        );
        let p = ui.painter().clone();
        let lines = winui::paragraph(
            &p,
            o.x + 21.0,
            o.y + 358.0,
            // i18n: Options → Accounts: what these accounts are, and the services' rule against sharing
            crate::i18n::t(
                "A debrid service downloads a file host's link with your own paid account and gives MDM a direct link. Use only your own account: sharing an account or its links breaks the services' terms and can get it suspended. Downloads the browser already started stay as they are.",
            ),
            LIGHT_TEXT(),
            o.x + 404.0,
            14.0,
        );
        // Where the sign-ins are kept (#44, secrets.rs); the list is two rows shorter for it.
        let kept = vault.kept();
        if kept == crate::secrets::Kept::Looking {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(200));
        }
        if let Some(text) = kept_text(kept) {
            winui::paragraph(
                &p,
                o.x + 21.0,
                o.y + 358.0 + lines as f32 * 14.0 + 6.0,
                crate::i18n::t(text),
                LIGHT_TEXT(),
                o.x + 404.0,
                14.0,
            );
        }
    }

    fn logins(&mut self, ui: &mut Ui, o: Pos2) {
        let rows: Vec<Vec<String>> = self
            .s
            .site_logins
            .iter()
            .map(|(site, user, pass)| {
                vec![site.clone(), user.clone(), "*".repeat(pass.chars().count())]
            })
            .collect();
        // The list leaves the page's foot to Cookies for video sites (#229).
        let (dbl, del_key) = report(
            ui,
            o,
            r(o, 21.0, 93.0, 383.0, 140.0),
            &[
                // i18n: column title: the site (and path) a login is for
                (crate::i18n::mark("Site/path"), 262.0),
                // i18n: column title: the user name
                (crate::i18n::mark("User"), 332.0),
                (crate::i18n::mark("Password"), 387.0),
            ],
            &rows,
            &mut self.login_rows,
            false,
            "opt-logins",
        );
        let sel = self.login_rows.selected.filter(|i| *i < rows.len());
        // As wide as their labels in this language (#192), under the list (it ends at 404).
        let [new_at, edit_at, remove_at] = winui::button_row_from_left(
            &ui.painter().clone(),
            &[
                ("New", 31.0, 73.0),
                ("Edit", 125.0, 73.0),
                ("Remove", 220.0, 73.0),
            ],
            21.0,
            404.0,
            140.0,
        )[..] else {
            unreachable!("three buttons")
        };
        let new = winui::button_light(
            ui,
            r(o, new_at.0, 241.0, new_at.1, 21.0),
            // i18n: button: create a new item
            "New",
            ButtonKind::Normal,
            false,
        )
        .clicked();
        let kind = if sel.is_some() {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        let edit =
            // i18n: verb: edit the item
            winui::button_light(ui, r(o, edit_at.0, 241.0, edit_at.1, 21.0), "Edit", kind, false).clicked();
        // i18n: verb: remove the item
        let remove = winui::button_light(
            ui,
            r(o, remove_at.0, 241.0, remove_at.1, 21.0),
            "Remove",
            kind,
            false,
        )
        .clicked()
            || del_key;
        if new {
            self.sub = Some(Sub::Login(
                None,
                String::new(),
                String::new(),
                String::new(),
            ));
        }
        if let Some(i) = sel {
            if remove {
                self.s.site_logins.remove(i);
                self.login_rows.selected = None;
            } else if edit || dbl {
                let (site, user, pass) = self.s.site_logins[i].clone();
                self.sub = Some(Sub::Login(Some(i), site, user, pass));
            }
        }
        self.video_cookies(ui, o);
    }

    /// Sites Logins → Cookies for video sites (#229): Off, or the browser whose cookies yt-dlp
    /// reads on this computer (`--cookies-from-browser`) when the extension passed none, and its
    /// profile (a name, or a folder: a Chromium fork's).
    fn video_cookies(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        // i18n: Options → Sites Logins: group box title: yt-dlp signs in to video sites with a browser's cookies
        group(ui, o, 16.0, 270.0, 392.0, 130.0, "Cookies for video sites");
        let browsers = mdm_tools::proc::cookie_browsers();
        let at = self.cookie_browser.min(browsers.len() - 1);
        let on = !self.s.video_cookies.is_empty();
        // i18n: Options → Sites Logins → Cookies for video sites: the tooltip of its choices
        let tip = crate::i18n::t(
            "yt-dlp signs in to video sites with this browser's cookies, read from its files on this computer only. Used when the browser's extension didn't pass the page's own: a pasted address, a playlist.",
        );
        // i18n: Options → Sites Logins → Cookies for video sites: don't read any browser's cookies (the default)
        let off =
            winui::radio_light(ui, o + vec2(30.0, 288.0), !on, "Off", true).on_hover_text(tip);
        let from = winui::radio_light_until(
            ui,
            o + vec2(30.0, 314.0),
            on,
            // i18n: Options → Sites Logins → Cookies for video sites: read the cookies of the browser chosen in the list after it
            "From browser:",
            true,
            // Before the browser list (`until` is a window x, not the page's).
            Some(o.x + 194.0),
        )
        .on_hover_text(tip);
        let choose = |s: &mut Settings, i: usize| s.video_cookies = browsers[i].to_owned();
        match winui::radio_keys(ui, &[off.clone(), from.clone()]) {
            Some(0) => self.s.video_cookies.clear(),
            Some(_) => choose(&mut self.s, at),
            None if off.clicked() => self.s.video_cookies.clear(),
            None if from.clicked() => choose(&mut self.s, at),
            None => {}
        }
        let on = !self.s.video_cookies.is_empty();
        let names: Vec<&str> = browsers
            .iter()
            .map(|b| crate::videodlg::browser_label(b))
            .collect();
        if let Some(i) = winui::combo_light(
            ui,
            r(o, 200.0, 311.0, 192.0, 21.0),
            &names,
            at,
            on,
            "opt-cookie-browser",
        ) {
            self.cookie_browser = i;
            choose(&mut self.s, i);
        }
        // The profile: a name (Firefox's, "Profile 1") or a folder; empty, the last one used.
        let ink = if on { LIGHT_TEXT() } else { LIGHT_DISABLED() };
        // i18n: Options → Sites Logins → Cookies for video sites: label before the browser profile's name or folder (may stay empty)
        let label = crate::i18n::t("Profile (optional):");
        let (rect, cut) =
            winui::text_right_within(&p, o.x + 194.0, o.y + 354.0, label, ink, o.x + 30.0);
        if cut {
            winui::full_on_hover(ui, rect, "cookie-profile", label);
        }
        winui::edit_light_on(
            ui,
            r(o, 200.0, 338.0, 160.0, 23.0),
            &mut self.s.video_cookies_profile,
            on,
            false,
            LIGHT_PAGE(),
        )
        .on_hover_text(crate::i18n::t(
            // i18n: Options → Sites Logins → Cookies for video sites: the tooltip of the profile box
            "A profile's name or its folder. Empty: the profile used last. For a browser built on Chromium that isn't listed (Helium, Thorium...), choose Chromium and its profile's folder.",
        ));
        // i18n: Options → Sites Logins → Cookies for video sites: the title of the window, and the tooltip of the button, that pick a browser profile's folder
        let pick_title = crate::i18n::t("Choose the profile's folder");
        let kind = if on {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        if winui::button_light(ui, r(o, 365.0, 339.0, 27.0, 21.0), "...", kind, false)
            .on_hover_text(pick_title)
            .clicked()
        {
            let title = pick_title.to_owned();
            winui::start_pick(ui.ctx(), "options-cookie-profile", move || {
                rfd::FileDialog::new().set_title(title).pick_folder()
            });
        }
        if let Some(dir) = winui::picked(ui.ctx(), "options-cookie-profile") {
            self.s.video_cookies_profile = dir.display().to_string();
        }
        // What to expect: Chrome-family browsers on Windows lock their cookies (Chrome 127 and
        // later's app-bound encryption); yt-dlp says so when it meets it, Options says so first.
        let chromium = on && self.s.video_cookies != "firefox";
        let (note, ink) = if chromium {
            (
                // i18n: Options → Sites Logins → Cookies for video sites, a Chrome-like browser chosen on Windows
                crate::i18n::t(
                    "On Windows, Chrome, Edge and Brave may lock their cookies. Firefox works.",
                ),
                winui::ERROR(),
            )
        } else {
            (
                // i18n: Options → Sites Logins → Cookies for video sites: a note under the choices
                crate::i18n::t("Read on this computer only, and sent only to the video's site."),
                LIGHT_DISABLED(),
            )
        };
        // Two lines at most: a longer language wraps rather than being cut.
        winui::paragraph(&p, o.x + 30.0, o.y + 380.0, note, ink, o.x + 400.0, 13.0);
    }

    /// Archives (ours, #37): extracting archives when their download completes, what becomes
    /// of them, and Link grabber's ways.
    fn archives(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let s = &mut self.s;
        // i18n: Options → Archives: group box title
        group(ui, o, 16.0, 92.0, 392.0, 196.0, "Archives");
        winui::checkbox_light(
            ui,
            o + vec2(30.0, 110.0),
            &mut s.extract_auto,
            // i18n: Options → Archives check box (a Link grabber package has its own "Extract when done")
            "Extract archives when their download completes",
            true,
        );
        winui::checkbox_light(
            ui,
            o + vec2(30.0, 134.0),
            &mut s.extract_delete,
            // i18n: Options → Archives check box
            "Then move the archives to the Recycle Bin",
            true,
        );
        text(
            &p,
            o.x + 30.0,
            o.y + 174.0,
            // i18n: Options → Archives: a label before the list of where extracted files go
            crate::i18n::t("What an archive holds goes:"),
            LIGHT_TEXT(),
        );
        const INTO: [&str; 3] = [
            // i18n: Options → Archives: where extracted files go
            crate::i18n::mark("Into a folder named after it, when it holds several things"),
            // i18n: Options → Archives: where extracted files go
            crate::i18n::mark("Always into a folder named after it"),
            // i18n: Options → Archives: where extracted files go
            crate::i18n::mark("Straight into the archive's folder"),
        ];
        if let Some(i) = winui::combo_light(
            ui,
            r(o, 30.0, 181.0, 364.0, 21.0),
            &INTO,
            s.extract_subfolder.min(2) as usize,
            true,
            "opt-extract-into",
        ) {
            s.extract_subfolder = i as u8;
        }
        winui::paragraph(
            &p,
            o.x + 30.0,
            o.y + 226.0,
            crate::i18n::t(
                "ZIP, 7Z, RAR, TAR, GZ and XZ; a split archive (.part1.rar, .7z.001) once\nevery volume is in. A file there already is kept: the new one is numbered.",
            ),
            LIGHT_TEXT(),
            o.x + 400.0,
            13.0,
        );
        let (rar, ink) = match &self.seven_zip {
            Some(path) => (
                crate::i18n::fmt(
                    // i18n: Options → Archives: {path} is where 7-Zip's program is
                    crate::i18n::t("7z and RAR archives are extracted by 7-Zip ({path})."),
                    &[("path", &crate::i18n::ltr(&path.display().to_string()))],
                ),
                LIGHT_TEXT(),
            ),
            None => (
                // i18n: Options → Archives, when 7-Zip isn't installed
                crate::i18n::t(
                    "7z and RAR archives need 7-Zip (7-zip.org), which isn't installed.",
                )
                .to_owned(),
                winui::ERROR(),
            ),
        };
        winui::fit(
            ui,
            &p,
            o.x + 30.0,
            o.y + 272.0,
            &rar,
            ink,
            o.x + 400.0,
            "opt-rar-tool",
        );
        // i18n: Options → Archives: group box title
        group(ui, o, 16.0, 300.0, 392.0, 100.0, "Link grabber");
        winui::checkbox_light(
            ui,
            o + vec2(30.0, 318.0),
            &mut s.grabber_check,
            // i18n: Options → Archives → Link grabber check box (IDM's Download all links looks every link up)
            "Look links up when they arrive (name, size, online)",
            true,
        );
        winui::checkbox_light(
            ui,
            o + vec2(30.0, 342.0),
            &mut s.grabber_hide_pages,
            "Hide web pages",
            true,
        );
        winui::paragraph(
            &p,
            o.x + 30.0,
            o.y + 380.0,
            crate::i18n::t(
                "Tasks → Link grabber, or the browser's \"Download all links\": a split\narchive's volumes are one package, the other links of a page another.",
            ),
            LIGHT_DISABLED(),
            o.x + 400.0,
            13.0,
        );
    }

    /// Regional (ours): how dates and times are written in the list and the Scheduler, as the
    /// system's region writes them or the user's own way.
    fn regional(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let reg = &mut self.s.regional;
        group(ui, o, 16.0, 92.0, 392.0, 250.0, "Write dates and times");
        let system = winui::radio_light(
            ui,
            o + vec2(30.0, 112.0),
            !reg.custom,
            "As the system's region writes them",
            true,
        );
        let own = winui::radio_light(ui, o + vec2(30.0, 140.0), reg.custom, "My own way:", true);
        match winui::radio_keys(ui, &[system.clone(), own.clone()]) {
            Some(i) => reg.custom = i == 1,
            None if system.clicked() => reg.custom = false,
            None if own.clicked() => reg.custom = true,
            None => {}
        }
        let on = reg.custom;
        // Each choice: its label right-aligned before its box.
        let row = |ui: &mut Ui, y: f32, label: &str, items: &[&str], at: usize, id: &str| {
            let label = crate::i18n::t(label);
            let color = if on { LIGHT_TEXT() } else { LIGHT_DISABLED() };
            let (rect, cut) =
                winui::text_right_within(&p, o.x + 186.0, o.y + y + 15.0, label, color, o.x + 48.0);
            if cut {
                winui::full_on_hover(ui, rect, ("regional-label", y as i32), label);
            }
            winui::combo_light(ui, r(o, 192.0, y, 200.0, 21.0), items, at, on, id)
        };
        let orders = [
            // i18n: a date's order: day first (30/10/2026)
            crate::i18n::mark("Day, month, year"),
            // i18n: a date's order: month first (10/30/2026)
            crate::i18n::mark("Month, day, year"),
            // i18n: a date's order: year first (2026-10-30)
            crate::i18n::mark("Year, month, day"),
        ];
        let order_at = match reg.order {
            DateOrder::Dmy => 0,
            DateOrder::Mdy => 1,
            DateOrder::Ymd => 2,
        };
        // i18n: label before the choice of a date's order
        if let Some(i) = row(
            ui,
            168.0,
            crate::i18n::mark("Date order:"),
            &orders,
            order_at,
            "regional-order",
        ) {
            reg.order = [DateOrder::Dmy, DateOrder::Mdy, DateOrder::Ymd][i];
        }
        let clocks = [
            // i18n: a 12-hour clock, with an example
            crate::i18n::mark("12-hour (9:41 PM)"),
            // i18n: a 24-hour clock, with an example
            crate::i18n::mark("24-hour (21:41)"),
        ];
        // i18n: label before the choice of a 12- or 24-hour clock
        if let Some(i) = row(
            ui,
            198.0,
            crate::i18n::mark("Clock:"),
            &clocks,
            reg.clock24 as usize,
            "regional-clock",
        ) {
            reg.clock24 = i == 1;
        }
        let days = [
            crate::i18n::mark("Sunday"),
            crate::i18n::mark("Monday"),
            crate::i18n::mark("Saturday"),
        ];
        let day_at = match reg.week_start {
            1 => 1,
            6 => 2,
            _ => 0,
        };
        // i18n: label before the choice of the week's first day (in the Scheduler's calendar)
        if let Some(i) = row(
            ui,
            228.0,
            crate::i18n::mark("First day of the week:"),
            &days,
            day_at,
            "regional-week",
        ) {
            reg.week_start = [0, 1, 6][i];
        }
        // What it looks like, now.
        let now = crate::clock::now().naive_local();
        let sample = crate::regional::format_with(crate::regional::effective(reg), now);
        // i18n: label before an example date and time written the chosen way
        let label = crate::i18n::t("Example:");
        text(&p, o.x + 48.0, o.y + 290.0, label, LIGHT_TEXT());
        let lw = winui::text_width(&p, label);
        winui::text_in_window(
            ui,
            &p,
            o.x + 48.0 + lw + 8.0,
            o.y + 290.0,
            &crate::i18n::ltr(&sample),
            LIGHT_TEXT(),
        );
        // i18n: a note under the date and time choices
        winui::text_in_window(
            ui,
            &p,
            o.x + 30.0,
            o.y + 320.0,
            crate::i18n::t("Used by the list of downloads and the Scheduler."),
            LIGHT_DISABLED(),
        );
    }

    fn sounds(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        group(
            ui,
            o,
            25.0,
            102.0,
            383.0,
            299.0,
            "Select sounds for Mister Download Manager events",
        );
        let list_on_screen = r(o, 33.0, 112.0, 366.0, 250.0);
        list_frame(&p, list_on_screen);
        let list = winui::rect(list_on_screen);
        let head_y = list.top() + 1.0;
        // i18n: column title: the event a sound plays for
        text(
            &p,
            list.left() + 8.0,
            head_y + 16.0,
            crate::i18n::t("Event"),
            LIGHT_TEXT(),
        );
        fill(
            &p,
            o.x + 224.0,
            head_y + 1.0,
            o.x + 224.0,
            head_y + 23.0,
            winui::HEADER_LINE(),
        );
        text(
            &p,
            o.x + 231.0,
            head_y + 16.0,
            crate::i18n::t("Sound file"),
            LIGHT_TEXT(),
        );
        // The events: the shared row list under the header (its frame on the list's). Space
        // ticks one, a double-click or Enter chooses its file, Delete clears it.
        let body = Rect::from_min_max(
            pos2(list.left() + 1.0, head_y + 25.0),
            pos2(list.right() - 1.0, list.bottom() - 1.0),
        );
        let grid = winui::GRID();
        let sounds = &mut self.s.sounds;
        self.sound_rows.name = crate::i18n::t("Sounds");
        let ev = winui::row_list(
            ui,
            "sounds",
            winui::rect(body.expand(1.0)),
            SOUND_EVENTS.len(),
            17.0,
            &mut self.sound_rows,
            |ui, clip, i, row, look| {
                let top = row.top();
                fill(clip, row.left(), top + 16.0, row.right(), top + 16.0, grid);
                fill(clip, o.x + 225.0, top, o.x + 225.0, top + 16.0, grid);
                let (on, file) = &mut sounds[i];
                let event = crate::i18n::t(SOUND_EVENTS[i]);
                look.name = event.to_owned();
                winui::row_check(
                    ui,
                    clip,
                    pos2(o.x + 39.0, top + 2.0),
                    on,
                    event,
                    ("sound", i),
                );
                winui::fit(
                    ui,
                    clip,
                    o.x + 57.0,
                    top + 12.0,
                    event,
                    look.ink,
                    o.x + 221.0,
                    ("sound-event", i),
                );
                let name = std::path::Path::new(file.as_str())
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let drawn = winui::fit(
                    ui,
                    clip,
                    o.x + 231.0,
                    top + 12.0,
                    &name,
                    look.ink,
                    row.right() - 4.0,
                    ("sound-file", i),
                );
                // The whole path on hover (the name alone says little).
                if !file.is_empty() {
                    winui::copyable(ui, drawn, ("sound-path", i), file, true);
                }
            },
        );
        if let Some(i) = ev.space {
            sounds[i].0 = !sounds[i].0;
        }
        if ev.focused {
            self.sound_rows
                .type_to(ui, "sounds", SOUND_EVENTS.len(), |i| {
                    crate::i18n::t(SOUND_EVENTS[i]).to_owned()
                });
        }
        if let Some(i) = ev.delete {
            sounds[i] = (false, String::new());
        }
        let pick_file = ev.double_clicked.or(ev.entered);
        if pick_file.is_some() {
            winui::take_key(ui, egui::Key::Enter);
        }
        let sel = self.sound_rows.selected;
        let kind = if sel.is_some() {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        // Both as wide as their labels in this language (#192), each around its English centre.
        let browse_at = winui::button_around(&p, "Browse...", 106.0, 73.0, 21.0, 233.0);
        let play_at = winui::button_around(
            &p,
            "Play",
            241.0,
            73.0,
            browse_at.0 + browse_at.1 + 8.0,
            404.0,
        );
        let browse = winui::button_light(
            ui,
            r(o, browse_at.0, 371.0, browse_at.1, 21.0),
            "Browse...",
            kind,
            false,
        )
        .clicked();
        let has_file = sel.is_some_and(|i| !self.s.sounds[i].1.is_empty());
        let play_kind = if has_file {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        let play =
            // i18n: verb: plays the sound
            winui::button_light(ui, r(o, play_at.0, 371.0, play_at.1, 21.0), "Play", play_kind, false)
                .clicked();
        if let Some(i) = sel {
            if browse || pick_file == Some(i) {
                let title = crate::i18n::t(SOUND_EVENTS[i]).to_owned();
                // i18n: the kind of file the Open dialog lists (sound files)
                let kind = crate::i18n::t("Sounds").to_owned();
                winui::start_pick(ui.ctx(), &format!("sound-{i}"), move || {
                    rfd::FileDialog::new()
                        .set_title(title)
                        .add_filter(kind, &["wav"])
                        .pick_file()
                });
            }
            if let Some(f) = winui::picked(ui.ctx(), &format!("sound-{i}")) {
                self.s.sounds[i] = (true, f.display().to_string());
            }
            if play && has_file {
                play_sound(&self.s.sounds[i].1);
            }
        }
    }

    // ---- entry dialogs --------------------------------------------------------------------

    /// The computer's adapters, read again every 2 s (Proxy / Socks' adapter, Remote's address).
    fn adapters_now(&mut self) -> Vec<mdm_engine::net::Adapter> {
        let stale = self
            .adapters
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() >= std::time::Duration::from_secs(2));
        if stale {
            self.adapters = Some((std::time::Instant::now(), crate::netwatch::adapters()));
        }
        self.adapters
            .as_ref()
            .map(|(_, a)| a.clone())
            .unwrap_or_default()
    }

    /// Remote's port as typed, or the one saved when it isn't a port MDM takes.
    fn remote_port(&self) -> u16 {
        self.port
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|p| *p >= 1024)
            .unwrap_or(self.from.remote_port)
    }

    /// The remote control as this draft says (on, address, port).
    fn remote_config(&self) -> mdm_remote::Config {
        let mut s = self.s.clone();
        s.remote_port = self.remote_port();
        crate::remote::config(&s)
    }

    /// Remote → Pair a device...: the remote control starts as the page says (Cancel puts it
    /// back as it was), a new code is shown.
    fn pair(&mut self) {
        let config = self.remote_config();
        if let Some(r) = &self.remote {
            r.apply(config);
            r.start_pairing();
        }
        self.sub = Some(Sub::Pair);
    }

    /// Remote (#141, ours): phones and computers on the local network add and control
    /// downloads from a page MDM serves, once paired with a code shown here.
    fn remote_page(&mut self, ui: &mut Ui, o: Pos2) {
        let p = ui.painter().clone();
        let adapters = self.adapters_now();
        winui::checkbox_light(
            ui,
            o + vec2(21.0, 92.0),
            &mut self.s.remote_on,
            // i18n: Options → Remote check box (off by default)
            "Let phones and computers on the local network control MDM",
            true,
        );
        let on = self.s.remote_on;
        let color = if on { LIGHT_TEXT() } else { LIGHT_DISABLED() };
        text(
            &p,
            o.x + 21.0,
            o.y + 131.0,
            // i18n: Options → Remote: label before the choice of the address MDM listens on
            crate::i18n::t("Listen on:"),
            color,
        );
        let chosen = self.s.remote_listen.trim().to_owned();
        let mut values = vec![String::new(), mdm_remote::addrs::THIS_COMPUTER.to_owned()];
        let mut labels = vec![
            // i18n: Options → Remote → Listen on: the default: each of the computer's private local network addresses (never one the internet reaches)
            crate::i18n::t("Every local network address").to_owned(),
            // i18n: Options → Remote → Listen on: only programs on this computer can connect (127.0.0.1)
            crate::i18n::t("This computer only (127.0.0.1)").to_owned(),
        ];
        for a in &adapters {
            for ip in
                a.v4.iter()
                    .map(|v| v.to_string())
                    .chain(a.v6.iter().map(|v| v.to_string()))
            {
                labels.push(format!("{} \u{2014} {ip}", a.name));
                values.push(ip);
            }
        }
        if !values.contains(&chosen) {
            labels.push(crate::i18n::fmt(
                crate::i18n::t("{adapter} (not connected)"),
                &[("adapter", &chosen)],
            ));
            values.push(chosen.clone());
        }
        let at = values.iter().position(|v| *v == chosen).unwrap_or(0);
        let shown: Vec<&str> = labels.iter().map(String::as_str).collect();
        if let Some(i) = winui::combo_light(
            ui,
            r(o, 130.0, 118.0, 278.0, 21.0),
            &shown,
            at,
            on,
            "opt-remote-at",
        ) {
            self.s.remote_listen = values[i].clone();
        }
        // i18n: Options → Remote: label before the port number box
        text(&p, o.x + 21.0, o.y + 159.0, crate::i18n::t("Port:"), color);
        winui::spin_box(
            ui,
            r(o, 130.0, 146.0, 80.0, 21.0),
            &mut self.port,
            1024,
            65535,
            on,
            "opt-remote-port",
        );
        // What it does now: as saved, or what OK changes.
        group(ui, o, 16.0, 178.0, 392.0, 96.0, "Status");
        let running = self.remote.as_ref().map(|r| r.status());
        let urls = self.remote.as_ref().map(|r| r.urls()).unwrap_or_default();
        let changed = self.remote_config() != crate::remote::config(&self.from);
        use mdm_remote::Status;
        let (line, ok) = match &running {
            _ if changed => (
                // i18n: Options → Remote → Status: the settings were changed; they apply on OK
                crate::i18n::t("The changes apply when you click OK.").to_owned(),
                true,
            ),
            None | Some(Status::Off) => (
                // i18n: Options → Remote → Status: the remote control is off
                crate::i18n::t("Off: nothing on the network can reach MDM.").to_owned(),
                true,
            ),
            Some(Status::Listening(_)) => (
                // i18n: Options → Remote → Status: on; the address follows in a box below
                crate::i18n::t("On. On the phone, open this address:").to_owned(),
                true,
            ),
            Some(Status::NoAddress) => (
                // i18n: Options → Remote → Status: on, but the computer has no local network address now
                crate::i18n::t("No local network now: it starts when the computer joins one.")
                    .to_owned(),
                false,
            ),
            Some(Status::PortInUse(port)) => (
                crate::i18n::fmt(
                    // i18n: Options → Remote → Status: another program uses the port; {port} = its number
                    crate::i18n::t("Port {port} is used by another program: choose another."),
                    &[("port", &port.to_string())],
                ),
                false,
            ),
            Some(Status::Failed(e)) => (
                crate::i18n::fmt(
                    // i18n: Options → Remote → Status: it could not start; {error} = the system's reason
                    crate::i18n::t("It could not start: {error}"),
                    &[("error", e)],
                ),
                false,
            ),
        };
        let ink = if ok {
            LIGHT_TEXT()
        } else {
            crate::theme::colors().error_strong
        };
        let (rect, cut) = winui::label_into(
            &p,
            o.x + 30.0,
            o.y + 207.0,
            &line,
            ink,
            o.x + 400.0,
            o.x + 30.0,
        );
        if cut {
            winui::full_on_hover(ui, rect, "opt-remote-state", &line);
        }
        if !changed && let Some(first) = urls.first() {
            readonly_ltr(ui, r(o, 30.0, 215.0, 364.0, 21.0), first);
        }
        if let Some(fp) = self.remote.as_ref().and_then(|r| r.fingerprint()) {
            let label = crate::i18n::fmt(
                // i18n: Options → Remote → Status: the certificate's fingerprint, which the phone's browser shows when it warns about it; {fingerprint} = hex digits
                crate::i18n::t("Certificate (SHA-256): {fingerprint}"),
                &[("fingerprint", &crate::i18n::ltr(&fp))],
            );
            let (rect, cut) = winui::label_into(
                &p,
                o.x + 30.0,
                o.y + 260.0,
                &label,
                LIGHT_TEXT(),
                o.x + 400.0,
                o.x + 30.0,
            );
            if cut {
                winui::full_on_hover(ui, rect, "opt-remote-cert", &label);
            }
        }
        // The paired devices.
        text(
            &p,
            o.x + 21.0,
            o.y + 296.0,
            // i18n: Options → Remote: label above the list of paired phones and computers
            crate::i18n::t("Paired devices:"),
            LIGHT_TEXT(),
        );
        let devices = self
            .remote
            .as_ref()
            .map(|r| r.devices())
            .unwrap_or_default();
        let when = |secs: i64| {
            crate::clock::local_at(secs)
                .map(|t| crate::regional::short_date_time(t.naive_local()))
                .unwrap_or_default()
        };
        let rows: Vec<Vec<String>> = devices
            .iter()
            .map(|d| vec![d.name.clone(), when(d.added), when(d.seen)])
            .collect();
        let (_, del_key) = report(
            ui,
            o,
            r(o, 16.0, 302.0, 392.0, 88.0),
            &[
                // i18n: column title: a paired phone's or computer's name
                (crate::i18n::mark("Device"), 120.0),
                // i18n: column title: when the device was paired
                (crate::i18n::mark("Paired"), 256.0),
                // i18n: column title: when the device last reached MDM
                (crate::i18n::mark("Last seen"), 392.0),
            ],
            &rows,
            &mut self.device_rows,
            false,
            "opt-remote-devices",
        );
        let sel = self.device_rows.selected.filter(|i| *i < rows.len());
        let pair_kind = if on {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        let pair = winui::why_disabled(
            winui::button_light(
                ui,
                r(o, 16.0, 398.0, 130.0, 21.0),
                // i18n: Options → Remote button: shows a code (and a QR code) that pairs a phone
                "Pair a device...",
                pair_kind,
                false,
            ),
            !on,
            // i18n: why Options → Remote → Pair a device... is greyed
            "Turn on the remote control first.",
        )
        .clicked();
        let remove_kind = if sel.is_some() {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        };
        let remove = winui::button_light(
            ui,
            r(o, 154.0, 398.0, 90.0, 21.0),
            "Remove",
            remove_kind,
            false,
        )
        .clicked()
            || del_key;
        if pair && on {
            self.pair();
        }
        // Unpaired at once (not on OK): a lost phone is refused from its next request on.
        if remove
            && let (Some(i), Some(r)) = (sel, &self.remote)
            && let Some(d) = devices.get(i)
        {
            r.remove(&d.id);
            self.device_rows.selected = None;
        }
        winui::paragraph(
            &p,
            o.x + 21.0,
            o.y + 438.0,
            // i18n: Options → Remote, at the bottom of the page
            crate::i18n::t(
                "Only paired devices see or change anything. The first time, the phone's browser warns about MDM's own certificate: its fingerprint is the one above.",
            ),
            LIGHT_TEXT(),
            o.x + 406.0,
            13.0,
        );
    }

    /// Remote → Pair a device...: the address (and a QR code of it), the code, and the device
    /// once it paired. False when it closes.
    fn pair_window(&mut self, ctx: &egui::Context) -> bool {
        let Some(remote) = self.remote.clone() else {
            return false;
        };
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        // The mockup's layout: the sentence, the QR code, the address, the code and how long it
        // works, the footer.
        const W: f32 = 380.0;
        const BODY: f32 = 380.0;
        let out = window(
            ctx,
            "opt-pair",
            // i18n: window title: Options → Remote → Pair a device...
            "Pair a device",
            [W, BODY + winui::FOOTER_H],
            false,
            LIGHT_BG(),
            |ui, o| {
                let p = ui.painter().clone();
                let e = winui::EDGE;
                let urls = remote.urls();
                let (shown, paired) = remote.pairing();
                let close_label = if paired.is_some() {
                    // i18n: button: close the window (the device is paired)
                    "Done"
                } else {
                    "Close"
                };
                // First in Tab order, as before. Escape closes; Enter too once a device paired
                // (while MDM can be reached).
                let close = winui::footer(
                    ui,
                    o,
                    W,
                    BODY,
                    &mut [FooterItem::Button(
                        FooterButton::new(close_label, ButtonKind::Default)
                            .enter(paired.is_some() && !urls.is_empty())
                            .escape(true),
                    )],
                )
                .is_some();
                let Some(url) = urls.first() else {
                    winui::paragraph(
                        &p,
                        o.x + e,
                        o.y + 29.0,
                        // i18n: Options → Remote → Pair a device: the remote control isn't listening (no network, port taken): the Remote page says why
                        crate::i18n::t(
                            "MDM can't be reached on the network now: Options → Remote says why.",
                        ),
                        crate::theme::colors().error_strong,
                        o.x + W - e,
                        LINE,
                    );
                    return if close {
                        Outcome::Closed
                    } else {
                        Outcome::Open
                    };
                };
                if let Some(name) = &paired {
                    let line = crate::i18n::fmt(
                        // i18n: Options → Remote → Pair a device: done; {device} = the name it was given
                        crate::i18n::t("{device} is paired. It can control MDM now."),
                        &[("device", &crate::i18n::isolate(name))],
                    );
                    winui::paragraph(
                        &p,
                        o.x + e,
                        o.y + 29.0,
                        &line,
                        LIGHT_TEXT(),
                        o.x + W - e,
                        LINE,
                    );
                    return if close {
                        Outcome::Done(())
                    } else {
                        Outcome::Open
                    };
                }
                winui::paragraph(
                    &p,
                    o.x + e,
                    o.y + 29.0,
                    // i18n: Options → Remote → Pair a device: how to pair
                    crate::i18n::t(
                        "On the phone, scan this code with the camera, or open the address and enter the code.",
                    ),
                    LIGHT_TEXT(),
                    o.x + W - e,
                    LINE,
                );
                let link = match &shown {
                    Some((code, _)) => format!("{url}#pair={code}"),
                    None => url.clone(),
                };
                crate::remote::paint_qr(&p, r(o, (W - 180.0) / 2.0, 62.0, 180.0, 180.0), &link);
                readonly_ltr(ui, r(o, e, 256.0, W - 2.0 * e, winui::CONTROL_H), url);
                match &shown {
                    Some((code, left)) => {
                        // i18n: Options → Remote → Pair a device: label before the 8-digit code
                        let label = crate::i18n::t("Code:");
                        let baseline = o.y + winui::baseline_in(300.0, winui::CONTROL_H);
                        text(&p, o.x + e, baseline, label, LIGHT_TEXT());
                        let big = egui::FontId::new(26.0, winui::tahoma_bold().family);
                        winui::text_font(
                            &p,
                            big,
                            o.x + e + winui::text_width(&p, label) + 10.0,
                            baseline + 4.0,
                            &crate::i18n::ltr(&format!("{} {}", &code[..4], &code[4..])),
                            LIGHT_TEXT(),
                        );
                        let secs = left.as_secs();
                        let line = crate::i18n::fmt(
                            // i18n: Options → Remote → Pair a device: how long the code stays valid; {time} = minutes:seconds ("9:58")
                            crate::i18n::t("Waiting for the device. The code works for {time}."),
                            &[("time", &format!("{}:{:02}", secs / 60, secs % 60))],
                        );
                        hint(&p, o, e, 347.0, &line, W - e);
                    }
                    None => {
                        winui::paragraph(
                            &p,
                            o.x + e,
                            o.y + 313.0,
                            // i18n: Options → Remote → Pair a device: the code ran out (10 minutes, or 5 wrong tries)
                            crate::i18n::t("This code can't be used any more."),
                            crate::theme::colors().error_strong,
                            o.x + W - e,
                            LINE,
                        );
                        let w = winui::footer_button_width(&p, "New code", false);
                        if winui::button_light(
                            ui,
                            r(o, e, 330.0, w, winui::CONTROL_H),
                            // i18n: Options → Remote → Pair a device: button: show a new code
                            "New code",
                            ButtonKind::Normal,
                            false,
                        )
                        .clicked()
                        {
                            remote.start_pairing();
                        }
                    }
                }
                if close {
                    Outcome::Closed
                } else {
                    Outcome::Open
                }
            },
        );
        matches!(out, Outcome::Open)
    }

    fn sub_dialog(&mut self, ctx: &egui::Context) {
        let Some(sub) = self.sub.take() else { return };
        match sub {
            Sub::Pair => {
                if self.pair_window(ctx) {
                    self.sub = Some(Sub::Pair);
                } else if let Some(r) = &self.remote {
                    r.stop_pairing();
                }
            }
            Sub::Login(row, mut site, mut user, mut pass) => {
                let title = if row.is_some() {
                    crate::i18n::mark("Edit login")
                } else {
                    crate::i18n::mark("New login")
                };
                // The mockup's layout: three form rows, the footer.
                const W: f32 = 340.0;
                const BODY: f32 = 142.0;
                let out = window(
                    ctx,
                    "opt-login",
                    title,
                    [W, BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| {
                        let p = ui.painter().clone();
                        let (e, h) = (winui::EDGE, winui::CONTROL_H);
                        // i18n: Site/path: label before a web address (site and folder); User: noun, label before the user name
                        let labels = ["Site/path:", "User:", "Password:"];
                        let fx = winui::form(&p, e, &labels, false);
                        let fw = W - e - fx;
                        winui::form_label(&p, o, e, 16.0, h, labels[0], true);
                        winui::edit_light(ui, r(o, fx, 16.0, fw, h), &mut site, true, false);
                        winui::form_label(&p, o, e, 56.0, h, labels[1], true);
                        winui::edit_light(ui, r(o, fx, 56.0, fw, h), &mut user, true, false);
                        winui::form_label(&p, o, e, 96.0, h, labels[2], true);
                        winui::password_light(ui, r(o, fx, 96.0, fw, h), &mut pass, true);
                        ok_cancel(ui, o, W, BODY, !site.trim().is_empty())
                    },
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Login(row, site, user, pass)),
                    Outcome::Done(()) => {
                        let entry = (site.trim().to_owned(), user, pass);
                        match row {
                            Some(i) => self.s.site_logins[i] = entry,
                            None => self.s.site_logins.push(entry),
                        }
                    }
                    Outcome::Closed => {}
                }
            }
            Sub::Exception(row, mut server, mut number) => {
                let limit = self.s.connection_limit();
                let title = if row.is_some() {
                    crate::i18n::mark("Edit exception")
                } else {
                    crate::i18n::mark("New exception")
                };
                // The mockup's layout: two form rows, the footer.
                const W: f32 = 340.0;
                const BODY: f32 = 102.0;
                let out = window(
                    ctx,
                    "opt-exception",
                    title,
                    [W, BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| {
                        let p = ui.painter().clone();
                        let (e, h) = (winui::EDGE, winui::CONTROL_H);
                        // i18n: Server: noun, label before a server name; Connections: label before the number of connections allowed
                        let labels = ["Server:", "Connections:"];
                        let fx = winui::form(&p, e, &labels, false);
                        winui::form_label(&p, o, e, 16.0, h, labels[0], true);
                        winui::edit_light(
                            ui,
                            r(o, fx, 16.0, W - e - fx, h),
                            &mut server,
                            true,
                            false,
                        );
                        winui::form_label(&p, o, e, 56.0, h, labels[1], true);
                        winui::spin_box(
                            ui,
                            r(o, fx, 56.0, 90.0, h),
                            &mut number,
                            1,
                            u64::from(limit),
                            true,
                            "opt-exception-number",
                        );
                        let in_range = number
                            .parse::<u32>()
                            .is_ok_and(|n| (1..=limit).contains(&n));
                        ok_cancel(ui, o, W, BODY, !server.trim().is_empty() && in_range)
                    },
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Exception(row, server, number)),
                    Outcome::Done(()) => {
                        let entry = (
                            server.trim().to_owned(),
                            number.parse().unwrap_or(1).clamp(1, limit),
                        );
                        match row {
                            Some(i) => self.s.conn_exceptions[i] = entry,
                            None => self.s.conn_exceptions.push(entry),
                        }
                    }
                    Outcome::Closed => {}
                }
            }
            Sub::Panels(mut on, mut sites, mut said) => {
                // The mockup's layout: the check box, the sites, Export and Import with what
                // they did, the footer.
                const W: f32 = 400.0;
                const BODY: f32 = 196.0;
                let out = window(
                    ctx,
                    "opt-panels",
                    // i18n: window title: the download button the extension shows over videos
                    "Download panels",
                    [W, BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| {
                        let p = ui.painter().clone();
                        let e = winui::EDGE;
                        winui::checkbox_light(
                            ui,
                            o + vec2(e, 16.0),
                            &mut on,
                            "Show a download button over videos in browsers",
                            true,
                        );
                        winui::text_in_window(
                            ui,
                            &p,
                            o.x + e,
                            o.y + 57.0,
                            crate::i18n::t("Never show it on these sites:"),
                            if on { LIGHT_TEXT() } else { LIGHT_DISABLED() },
                        );
                        multiline(
                            ui,
                            r(o, e, 66.0, W - 2.0 * e, 56.0),
                            &mut sites,
                            "panel-sites",
                            on,
                        );
                        hint(
                            &p,
                            o,
                            e,
                            139.0,
                            crate::i18n::t("(separate names by spaces)"),
                            W - e,
                        );
                        // The list as a file, to keep or carry to a new installation (#424).
                        let export_w = winui::footer_button_width(&p, "Export...", false);
                        let import_w = winui::footer_button_width(&p, "Import...", false);
                        let export_at = (e, export_w);
                        let import_at = (e + export_w + 8.0, import_w);
                        let row = 150.0;
                        if winui::button_light(
                            ui,
                            r(o, export_at.0, row, export_at.1, winui::CONTROL_H),
                            "Export...",
                            ButtonKind::Normal,
                            false,
                        )
                        .clicked()
                        {
                            let title = crate::i18n::t("Export").to_owned();
                            winui::start_pick(ui.ctx(), "opt-panels-export", move || {
                                rfd::FileDialog::new()
                                    .set_title(title)
                                    .add_filter("TXT", &["txt"])
                                    .set_file_name("sites.txt")
                                    .save_file()
                            });
                        }
                        if winui::button_light(
                            ui,
                            r(o, import_at.0, row, import_at.1, winui::CONTROL_H),
                            "Import...",
                            ButtonKind::Normal,
                            false,
                        )
                        .clicked()
                        {
                            let title = crate::i18n::t("Import").to_owned();
                            winui::start_pick(ui.ctx(), "opt-panels-import", move || {
                                rfd::FileDialog::new()
                                    .set_title(title)
                                    .add_filter("TXT", &["txt"])
                                    .pick_file()
                            });
                        }
                        if let Some(path) = winui::picked(ui.ctx(), "opt-panels-export") {
                            said = match std::fs::write(
                                &path,
                                mdm_ipc::capture::sites_to_text(&sites),
                            ) {
                                Ok(()) => crate::i18n::tn(
                                    sites.split_whitespace().count() as u64,
                                    "{n} site saved to the file.",
                                    "{n} sites saved to the file.",
                                ),
                                Err(e) => crate::i18n::fmt(
                                    crate::i18n::t("The list could not be saved: {error}"),
                                    &[("error", &e.to_string())],
                                ),
                            };
                        }
                        if let Some(path) = winui::picked(ui.ctx(), "opt-panels-import") {
                            said = match read_sites_file(&path) {
                                Ok(text) => {
                                    let (merged, added) =
                                        mdm_ipc::capture::merge_sites(&sites, &text);
                                    sites = merged;
                                    crate::i18n::tn(
                                        added as u64,
                                        "{n} site added from the file.",
                                        "{n} sites added from the file.",
                                    )
                                }
                                Err(e) => crate::i18n::fmt(
                                    crate::i18n::t("The file could not be read: {error}"),
                                    &[("error", &e.to_string())],
                                ),
                            };
                        }
                        if !said.is_empty() {
                            // After the buttons, as the mockup's hint.
                            let (rect, cut) = winui::text_fit(
                                &p,
                                o.x + import_at.0 + import_at.1 + 12.0,
                                o.y + winui::baseline_in(row, winui::CONTROL_H),
                                &said,
                                crate::theme::colors().muted,
                                o.x + W - e,
                            );
                            if cut {
                                winui::full_on_hover(ui, rect, "opt-panels-said", &said);
                            }
                        }
                        ok_cancel(ui, o, W, BODY, true)
                    },
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Panels(on, sites, said)),
                    Outcome::Done(()) => {
                        self.s.video_panel = on;
                        self.s.video_panel_sites =
                            sites.split_whitespace().collect::<Vec<_>>().join(" ");
                    }
                    Outcome::Closed => {}
                }
            }
            Sub::Dns(mut e) => {
                let out = window(
                    ctx,
                    "opt-dns",
                    // i18n: window title: the settings of the DNS chosen in Options → Connection
                    "DNS settings",
                    [DNS_W, DNS_BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| dns_dialog(ui, o, &mut e),
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Dns(e)),
                    Outcome::Done(()) => e.save(&mut self.s),
                    Outcome::Closed => {}
                }
            }
            Sub::Account(mut e) => {
                let out = window(
                    ctx,
                    "opt-account",
                    // i18n: window title: adding a debrid service account (Real-Debrid…)
                    "Add an account",
                    [ACCOUNT_W, ACCOUNT_BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| account_dialog(ui, o, &mut e),
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Account(e)),
                    Outcome::Done(account) => {
                        let mut account = account;
                        account.id = self
                            .s
                            .debrid_accounts
                            .iter()
                            .map(|a| a.id)
                            .max()
                            .unwrap_or(0)
                            + 1;
                        self.s.debrid_accounts.push(account);
                    }
                    Outcome::Closed => e
                        .signin
                        .stop
                        .store(true, std::sync::atomic::Ordering::Relaxed),
                }
            }
            Sub::Addresses(mut list) => {
                // The mockup's layout: the sentence, the list, the footer.
                const W: f32 = 420.0;
                const BODY: f32 = 200.0;
                let out = window(
                    ctx,
                    "opt-addresses",
                    // i18n: window title: the addresses whose downloads stay with the browser
                    "Addresses left to the browser",
                    [W, BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| {
                        let p = ui.painter().clone();
                        let e = winui::EDGE;
                        let lines = winui::paragraph(
                            &p,
                            o.x + e,
                            o.y + 29.0,
                            crate::i18n::t(
                                "One address per line; * stands for any text: example.com/private/*",
                            ),
                            LIGHT_TEXT(),
                            o.x + W - e,
                            LINE,
                        );
                        // Under the sentence (one or two lines), down to the footer's room.
                        let top = 16.0 + lines as f32 * LINE + 10.0;
                        multiline(
                            ui,
                            r(o, e, top, W - 2.0 * e, BODY - e - top),
                            &mut list,
                            "addresses",
                            true,
                        );
                        ok_cancel(ui, o, W, BODY, true)
                    },
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Addresses(list)),
                    Outcome::Done(()) => {
                        self.s.excluded_addresses = list
                            .lines()
                            .map(str::trim)
                            .filter(|l| !l.is_empty())
                            .map(str::to_owned)
                            .collect();
                    }
                    Outcome::Closed => {}
                }
            }
            Sub::Keys(mut opened) => {
                // The mockup's layout: the sentence, the check box, the footer.
                const W: f32 = 400.0;
                const BODY: f32 = 96.0;
                let out = window(
                    ctx,
                    "opt-keys",
                    // i18n: window title: the keys that force or prevent downloading with MDM
                    "Keys",
                    [W, BODY + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| {
                        let p = ui.painter().clone();
                        let e = winui::EDGE;
                        let lines = winui::paragraph(
                            &p,
                            o.x + e,
                            o.y + 29.0,
                            crate::i18n::t(
                                "Hold Alt while clicking a link to let the browser download it instead of Mister Download Manager.",
                            ),
                            LIGHT_TEXT(),
                            o.x + W - e,
                            LINE,
                        );
                        winui::checkbox_light(
                            ui,
                            o + vec2(e, 16.0 + lines.max(2) as f32 * LINE + 16.0),
                            &mut opened,
                            "Also take files the browser would open (PDF, music, video)",
                            true,
                        );
                        ok_cancel(ui, o, W, BODY, true)
                    },
                );
                match out {
                    Outcome::Open => self.sub = Some(Sub::Keys(opened)),
                    Outcome::Done(()) => self.s.capture_opened = opened,
                    Outcome::Closed => {}
                }
            }
            Sub::Test(mut checks) => {
                // The mockup's layout: a status icon and a line per check, the footer with the
                // fixes at its start.
                const W: f32 = 440.0;
                const ROW: f32 = 26.0;
                let body = 2.0 * winui::EDGE + checks.len() as f32 * ROW - 6.0;
                let mut set_up = false;
                let out = window(
                    ctx,
                    "opt-test",
                    // i18n: window title: Options → General → Test browser integration
                    "Test browser integration",
                    [W, body + winui::FOOTER_H],
                    false,
                    LIGHT_BG(),
                    |ui, o| {
                        let p = ui.painter().clone();
                        let e = winui::EDGE;
                        for (k, c) in checks.iter().enumerate() {
                            let top = e + k as f32 * ROW;
                            let (need, color) = if c.ok {
                                ("st-complete", winui::SUCCESS())
                            } else {
                                ("st-error", winui::ERROR())
                            };
                            crate::themeicons::paint_in(
                                ui,
                                &p,
                                need,
                                r(o, e, top + 2.0, 16.0, 16.0),
                                color,
                                true,
                            );
                            winui::text_in_window(
                                ui,
                                &p,
                                o.x + e + 24.0,
                                o.y + winui::baseline_in(top, 20.0),
                                &c.text,
                                LIGHT_TEXT(),
                            );
                        }
                        let first = checks.iter().find(|c| !c.ok);
                        let fix = first.and_then(|c| c.why);
                        let can_set_up = first.is_some_and(|c| c.why == Some("not-installed"))
                            && mdm_ipc::portable_dir().is_none();
                        // Tab and the footer as before: How to fix it, Set it up, Close; Enter
                        // and Escape both close.
                        let mut items = Vec::new();
                        if fix.is_some() {
                            items.push(FooterItem::Button(
                                // i18n: button in Test browser integration: open the help page's section for the problem
                                FooterButton::new("How to fix it", ButtonKind::Ghost).leading(),
                            ));
                        }
                        if can_set_up {
                            items.push(FooterItem::Button(
                                // i18n: button in Test browser integration: register MDM's helper with the browsers
                                FooterButton::new("Set it up", ButtonKind::Normal).leading(),
                            ));
                        }
                        let close = items.len();
                        items.push(FooterItem::Button(
                            FooterButton::new("Close", ButtonKind::Default)
                                .enter(true)
                                .escape(true),
                        ));
                        match (winui::footer(ui, o, W, body, &mut items), fix) {
                            (Some(i), _) if i == close => return Outcome::<()>::Closed,
                            (Some(0), Some(why)) => {
                                crate::help::open_page(&mdm_ipc::extension::help_doc(why));
                            }
                            (Some(_), _) => set_up = true,
                            (None, _) => {}
                        }
                        Outcome::Open
                    },
                );
                // Set it up runs on a thread of its own; the checks are read again once it's done.
                if set_up {
                    crate::extension::set_up(ctx);
                }
                if crate::extension::set_up_finished(&mut self.set_up_seen) {
                    checks = crate::extension::checks(&self.s);
                }
                if matches!(out, Outcome::Open) {
                    self.sub = Some(Sub::Test(checks));
                }
            }
        }
    }
}

// ---- pieces -----------------------------------------------------------------------------

/// Proxy / Socks → Get System: Windows' own proxy (host, port), if one is set.
fn system_proxy() -> Option<(String, String)> {
    crate::win32::system_proxy()
}

pub fn play_sound(file: &str) {
    if crate::offscreen() {
        return;
    }
    crate::win32::play_sound(file);
}

/// A folder as the dialogs show it: the full path, as IDM's Options shows it, ending in the
/// system's separator (a backslash on Windows).
fn display_dir(dir: &std::path::Path) -> String {
    let s = dir.display().to_string();
    if s.ends_with(std::path::MAIN_SEPARATOR) {
        s
    } else {
        format!("{s}{}", std::path::MAIN_SEPARATOR)
    }
}

/// OK / Cancel at (x, y), OK only when `valid`.
/// Connection → DNS → Settings...: how it's asked, its servers, the fallback, a test.
fn dns_dialog(ui: &mut Ui, o: Pos2, e: &mut DnsEdit) -> Outcome<()> {
    use mdm_engine::dns::{Mode, Preset};
    let p = ui.painter().clone();
    let (edge, h, w) = (winui::EDGE, winui::CONTROL_H, DNS_W);
    let preset = Preset::by_id(&e.provider);
    // i18n: Options → DNS settings: heading over the ways of asking the DNS
    text(
        &p,
        o.x + edge,
        o.y + 29.0,
        crate::i18n::t("Ask it with:"),
        LIGHT_TEXT(),
    );
    let modes = [
        (1u8, Mode::Plain, "plain DNS"),
        (2, Mode::Tls, "DNS over TLS"),
        (3, Mode::Https, "DNS over HTTPS"),
    ];
    // The three in a line, as the mockup's.
    let mut clicked = Vec::new();
    let mut x = edge;
    for (n, m, caption) in &modes {
        let offered = preset.is_none_or(|p| p.offers(*m));
        let r = winui::radio_light(ui, o + vec2(x, 48.0), e.mode == *n, caption, offered);
        if r.clicked() && offered {
            e.mode = *n;
        }
        clicked.push(r);
        x = winui::checkbox_caption_end(&p, x, caption) + 16.0;
    }
    if let Some(i) = winui::radio_keys(ui, &clicked) {
        let (n, m, _) = modes[i];
        if preset.is_none_or(|p| p.offers(m)) {
            e.mode = n;
        }
    }
    // One label column for every mode's rows, so it doesn't move when the mode changes.
    let fx = winui::form(
        &p,
        edge,
        &[
            "Server:",
            "Configuration ID:",
            "Servers:",
            "Certificate name:",
            "Address:",
            "IP addresses (optional):",
        ],
        false,
    );
    let (row1, row2) = (DNS_ROWS, DNS_ROWS + h + 10.0);
    let field = |ui: &mut Ui, y: f32, label: &str, value: &mut String, enabled: bool| {
        winui::form_label(&p, o, edge, y, h, label, enabled);
        winui::edit_light(ui, r(o, fx, y, w - edge - fx, h), value, enabled, false);
    };
    let mode = settings::dns_mode(e.mode).unwrap_or(Mode::Https);
    match preset {
        // A service: its servers as they are, NextDNS's configuration ID to type.
        Some(pr) => {
            let shown = match mode {
                Mode::Plain => pr.plain.join(", "),
                Mode::Tls => pr.tls_name.to_owned(),
                Mode::Https => pr.doh.to_owned(),
            };
            // i18n: Options → DNS settings: label before the chosen service's server (an address)
            winui::form_label(&p, o, edge, row1, h, "Server:", true);
            let (rect, cut) = winui::label_into(
                &p,
                o.x + fx,
                o.y + winui::baseline_in(row1, h),
                &shown,
                LIGHT_DISABLED(),
                o.x + w - edge,
                o.x + fx,
            );
            if cut {
                winui::full_on_hover(ui, rect, "dns-server", &shown);
            }
            if pr.id == "nextdns" {
                // i18n: Options → DNS settings: NextDNS's own configuration ID (optional)
                field(
                    ui,
                    row2,
                    "Configuration ID:",
                    &mut e.nextdns_id,
                    mode != Mode::Plain,
                );
            }
        }
        // The user's own DNS.
        None => match mode {
            // i18n: Options → DNS settings: label before the DNS servers' addresses, as IP addresses
            Mode::Plain => field(ui, row1, "Servers:", &mut e.servers, true),
            Mode::Tls => {
                field(ui, row1, "Servers:", &mut e.servers, true);
                // i18n: Options → DNS over TLS: label before the name the server's certificate shows
                field(ui, row2, "Certificate name:", &mut e.tls_name, true);
            }
            Mode::Https => {
                // i18n: Options → DNS over HTTPS: label before the server's https:// address
                field(ui, row1, "Address:", &mut e.doh_url, true);
                // i18n: Options → DNS over HTTPS: label before the server's IP addresses, which may be left empty
                field(ui, row2, "IP addresses (optional):", &mut e.servers, true);
            }
        },
    }
    let check_y = row2 + h + 14.0;
    winui::checkbox_light(
        ui,
        o + vec2(edge, check_y),
        &mut e.fallback,
        // i18n: Options → DNS settings: checkbox: when the chosen DNS can't be reached, use the system's instead
        "If it can't be reached, use the system's DNS",
        true,
    );
    hint(
        &p,
        o,
        edge,
        check_y + 13.0 + 14.0 + 13.0,
        // i18n: Options → DNS settings: what the DNS choice covers and what it doesn't
        crate::i18n::t(
            "Names in your hosts file and local names (single words, .local, .lan) stay with the system's DNS. Video pages and torrent trackers use the system's DNS for now.",
        ),
        w - edge,
    );
    let choice = e.choice();
    // Test: asks github.com through the choice, without falling back, on a thread of its own.
    let running = e.test.as_ref().is_some_and(|t| t.lock().unwrap().is_none());
    let kind = if choice.is_ok() && !running {
        ButtonKind::Normal
    } else {
        ButtonKind::Disabled
    };
    // The Test line: the button, then its answer (or why the choice can't be used).
    let test_y = DNS_BODY - edge - h;
    let test_w = winui::footer_button_width(&p, "Test", false);
    // i18n: button: asks the chosen DNS for an address, to see it works
    if winui::button_light(ui, r(o, edge, test_y, test_w, h), "Test", kind, false).clicked()
        && let Ok(c) = &choice
    {
        let slot = std::sync::Arc::new(std::sync::Mutex::new(None));
        e.test = Some(slot.clone());
        let (c, ctx) = (c.clone(), ui.ctx().clone());
        // Through the network adapter downloads use (#35).
        let route = crate::settings::own_route();
        std::thread::spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())
                .and_then(|rt| {
                    rt.block_on(mdm_engine::dns::test(&c, "github.com", &[], &route))
                        .map_err(|e| e.to_string())
                });
            let line = match result {
                Ok((took, ips)) => (
                    crate::i18n::fmt(
                        // i18n: Options → DNS settings: the test's answer; {ms} milliseconds, {ip} the address found
                        crate::i18n::t("Answered in {ms} ms: {ip}"),
                        &[
                            ("ms", &crate::i18n::integer(took.as_millis() as u64)),
                            (
                                "ip",
                                &ips.first().map(|i| i.to_string()).unwrap_or_default(),
                            ),
                        ],
                    ),
                    true,
                ),
                Err(why) => (why, false),
            };
            *slot.lock().unwrap() = Some(line);
            ctx.request_repaint();
        });
    }
    let status = match (&choice, &e.test) {
        (Err(why), _) => Some((why.clone(), false)),
        (Ok(_), Some(t)) => Some(t.lock().unwrap().clone().unwrap_or_else(|| {
            // i18n: Options → DNS settings: the test is running
            (crate::i18n::t("Asking...").to_owned(), true)
        })),
        _ => None,
    };
    if let Some((line, ok)) = status {
        let color = if ok {
            LIGHT_TEXT()
        } else {
            crate::theme::colors().error_strong
        };
        let x = o.x + edge + test_w + 8.0;
        let (rect, cut) = winui::label_into(
            &p,
            x,
            o.y + winui::baseline_in(test_y, h),
            &line,
            color,
            o.x + w - edge,
            x,
        );
        if cut {
            winui::full_on_hover(ui, rect, "dns-test", &line);
        }
    }
    ok_cancel(ui, o, w, DNS_BODY, choice.is_ok())
}

/// DNS settings (the mockup's): its width, its body above the footer, its form rows' top.
const DNS_W: f32 = 420.0;
const DNS_BODY: f32 = 302.0;
const DNS_ROWS: f32 = 84.0;

/// Accounts → Add...: the service, a token typed in, and the sign-in on its thread (#44).
#[derive(Default)]
struct AccountEdit {
    service: mdm_debrid::ServiceKind,
    token: String,
    signin: crate::debrid::SignIn,
}

/// Accounts → Add...: sign in by code in the browser (Real-Debrid's device flow, AllDebrid's PIN),
/// or paste the API token (Premiumize: its API key only); the account is checked before it is kept.
fn account_dialog(ui: &mut Ui, o: Pos2, e: &mut AccountEdit) -> Outcome<settings::DebridAccount> {
    use crate::debrid::SignInState as S;
    let p = ui.painter().clone();
    let ctx = ui.ctx().clone();
    let state = e.signin.get();
    let busy = matches!(state, S::Starting | S::Waiting { .. } | S::Checking);
    // The mockup's layout: the service, Sign in in the browser, the code's box, the token's
    // line, Find my token with how the sign-in went, the footer.
    let (edge, h, w) = (winui::EDGE, winui::CONTROL_H, ACCOUNT_W);
    // i18n: Options → Add an account: label before the choice of debrid service
    let fx = winui::form(&p, edge, &["Service:"], false);
    winui::form_label(&p, o, edge, 16.0, h, "Service:", true);
    let all = mdm_debrid::ServiceKind::ALL;
    let names: Vec<&str> = all.iter().map(|k| k.name()).collect();
    let at = all.iter().position(|k| *k == e.service).unwrap_or(0);
    if let Some(i) = winui::combo_light(
        ui,
        r(o, fx, 16.0, w - edge - fx, h),
        &names,
        at,
        !busy,
        "opt-account-service",
    ) && all[i] != e.service
    {
        // Another service: what was signed in to (or typed) belongs to the one before.
        e.service = all[i];
        e.token.clear();
        *e.signin.state.lock().unwrap() = S::Idle;
    }
    let by_code = e.service.signs_in_by_code();
    let kind = |on: bool| {
        if on {
            ButtonKind::Normal
        } else {
            ButtonKind::Disabled
        }
    };
    // i18n: button: sign in to the debrid service in the browser, by typing a code there
    if winui::button_light(
        ui,
        r(
            o,
            edge,
            60.0,
            winui::footer_button_width(&p, "Sign in in the browser", false),
            h,
        ),
        "Sign in in the browser",
        kind(!busy && by_code),
        false,
    )
    .clicked()
    {
        e.signin
            .stop
            .store(false, std::sync::atomic::Ordering::Relaxed);
        e.signin.by_code(e.service, &ctx);
    }
    if let S::Waiting { code, page, until } = &state {
        // In a box, as the mockup's: the code, large, to type at the service's page, its
        // buttons beside it, and what to do with it.
        let c = crate::theme::colors();
        let rc = r(o, edge, 104.0, w - 2.0 * edge, 94.0);
        p.rect_filled(rc, 6.0, c.panel);
        p.rect_stroke(
            rc,
            6.0,
            egui::Stroke::new(1.0, c.menubar_line),
            egui::StrokeKind::Inside,
        );
        let (x, row) = (edge + 12.0, 116.0);
        let code_font = egui::FontId::monospace(crate::theme::metrics().font_code);
        p.text(
            o + vec2(x, row + h / 2.0),
            egui::Align2::LEFT_CENTER,
            code,
            code_font.clone(),
            LIGHT_TEXT(),
        );
        // The buttons as wide as their labels in this language (#192), short of the code.
        let code_end = x + winui::text_width_font(&p, code_font, code);
        let [copy_at, open_at] = winui::button_row(
            &p,
            &[("Copy code", 200.0, 90.0), ("Open the page", 298.0, 110.0)],
            8.0,
            (code_end + 12.0).min(200.0),
            200.0,
        )[..] else {
            unreachable!("two buttons")
        };
        // i18n: button: copy the sign-in code
        if winui::button_light(
            ui,
            r(o, copy_at.0, row, copy_at.1, h),
            "Copy code",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            crate::copy_text(&ctx, code.clone());
        }
        // i18n: button: open the service's page where the code is typed
        if winui::button_light(
            ui,
            r(o, open_at.0, row, open_at.1, h),
            "Open the page",
            ButtonKind::Normal,
            false,
        )
        .clicked()
        {
            open_page(page);
        }
        let left = until
            .saturating_duration_since(std::time::Instant::now())
            .as_secs();
        hint(
            &p,
            o,
            x,
            row + h + 8.0 + 13.0,
            &crate::i18n::fmt(
                // i18n: Options → Add an account: waiting while the user types the code at the service's page; {page} its address, {time} minutes:seconds left
                crate::i18n::t(
                    "Type this code at {page} and allow MDM; waiting for you (the code expires in {time}).",
                ),
                &[
                    ("page", &crate::i18n::ltr(page)),
                    ("time", &format!("{}:{:02}", left / 60, left % 60)),
                ],
            ),
            w - edge - 12.0,
        );
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
    let label = if by_code {
        // i18n: Options → Add an account: label before the field for the service's API token (the other way to sign in)
        crate::i18n::t("Or paste your API token:")
    } else {
        // i18n: Options → Add an account: label before the field for the service's API key (Premiumize: the only way to sign in; the key is on the service's Account page)
        crate::i18n::t("Paste your API key (from the service's Account page):")
    };
    text(&p, o.x + edge, o.y + 225.0, label, LIGHT_TEXT());
    // As wide as its label in this language (#192); the box ends 8 before it.
    let use_w = winui::footer_button_width(&p, "Use the token", false);
    let use_x = w - edge - use_w;
    winui::password_light(
        ui,
        r(o, edge, 234.0, use_x - 8.0 - edge, h),
        &mut e.token,
        !busy,
    );
    // i18n: button: sign in with the token typed in
    if winui::button_light(
        ui,
        r(o, use_x, 234.0, use_w, h),
        "Use the token",
        kind(!busy && !e.token.trim().is_empty()),
        false,
    )
    .clicked()
    {
        e.signin.by_token(e.service, e.token.clone(), &ctx);
    }
    // i18n: button: open the service's page that shows the user's API token
    let find_w = winui::footer_button_width(&p, "Find my token", false);
    if winui::button_light(
        ui,
        r(o, edge, 278.0, find_w, h),
        "Find my token",
        ButtonKind::Ghost,
        false,
    )
    .clicked()
    {
        open_page(e.service.token_page());
    }
    let (line, ok) = match &state {
        // i18n: Options → Add an account: the sign-in is starting or being checked
        S::Starting | S::Checking => (crate::i18n::t("Asking the service...").to_owned(), true),
        S::Done { user, state, .. } => (
            crate::i18n::fmt(
                // i18n: Options → Add an account: signed in; {user} the account's name, {state} premium or not
                crate::i18n::t("Signed in as {user} ({state})"),
                &[("user", &crate::i18n::ltr(user)), ("state", state)],
            ),
            true,
        ),
        S::Failed(why) => (why.clone(), false),
        _ => (String::new(), true),
    };
    let color = if ok {
        LIGHT_TEXT()
    } else {
        crate::theme::colors().error_strong
    };
    // After Find my token, on its line.
    let x = o.x + edge + find_w + 12.0;
    let (rect, cut) = winui::label_into(
        &p,
        x,
        o.y + winui::baseline_in(278.0, h),
        &line,
        color,
        o.x + w - edge,
        x,
    );
    if cut {
        winui::full_on_hover(ui, rect, "opt-account-state", &line);
    }
    let done = matches!(state, S::Done { .. });
    match ok_cancel(ui, o, w, ACCOUNT_BODY, done) {
        Outcome::Done(()) => match state {
            S::Done {
                user,
                state,
                credential,
            } => Outcome::Done(settings::DebridAccount {
                id: 0,
                service: e.service,
                enabled: true,
                user,
                state,
                secret: credential,
            }),
            _ => Outcome::Open,
        },
        Outcome::Closed => Outcome::Closed,
        Outcome::Open => Outcome::Open,
    }
}

/// Add an account (the mockup's): its width and its body above the footer.
const ACCOUNT_W: f32 = 440.0;
const ACCOUNT_BODY: f32 = 324.0;

/// Open a service's web page in the browser; never in an off-screen test run (MDM_OFFSCREEN),
/// whose clicks come from scripts, not from a person at the browser.
fn open_page(url: &str) {
    if !crate::offscreen() {
        crate::launch::open(url);
    }
}

/// A list of sites from a file (Download panels' Import, #424): text, and no more than a list
/// could be.
fn read_sites_file(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read as _;
    let mut text = Vec::new();
    std::fs::File::open(path)?
        .take(1 << 20)
        .read_to_end(&mut text)?;
    String::from_utf8(text).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// A sub-dialog's footer at English `y` across `w`: Cancel, OK* (OK greyed until `valid`).
/// Enter is OK while `valid`, Escape is Cancel, as before; Tab: OK, then Cancel.
fn ok_cancel(ui: &mut Ui, o: Pos2, w: f32, y: f32, valid: bool) -> Outcome<()> {
    let kind = if valid {
        ButtonKind::Default
    } else {
        ButtonKind::Disabled
    };
    let pressed = winui::footer(
        ui,
        o,
        w,
        y,
        &mut [
            FooterItem::Button(FooterButton::new("OK", kind).primary().enter(true)),
            FooterItem::Button(FooterButton::new("Cancel", ButtonKind::Normal).escape(true)),
        ],
    );
    match pressed {
        Some(1) => Outcome::Closed,
        Some(_) if valid => Outcome::Done(()),
        _ => Outcome::Open,
    }
}

/// A sub-dialog's hint (the mockup's `.hint`): `s` wrapped from English `x` at `baseline` to
/// `right` (window-relative to `o`), in the muted colour. Its line count.
fn hint(p: &egui::Painter, o: Pos2, x: f32, baseline: f32, s: &str, right: f32) -> usize {
    winui::paragraph(
        p,
        o.x + x,
        o.y + baseline,
        s,
        crate::theme::colors().muted,
        o.x + right,
        LINE,
    )
}

/// A sub-dialog's line height for wrapped text.
const LINE: f32 = 16.0;

/// The window: the side list ([`SIDE_W`]), the page area, the footer ([`winui::footer`]).
/// The mock's 820 x 520 (plan 08b).
const W: f32 = 820.0;
const H: f32 = 520.0;
/// The side list's width (its right border at this x).
const SIDE_W: f32 = 190.0;
/// A side list row: its height and the gap after it, from the list's top `SIDE_TOP`.
const ROW_H: f32 = 30.0;
const ROW_GAP: f32 = 1.0;
const SIDE_TOP: f32 = 10.0;
/// Where a page's English pixels (laid out in the old tab frame, x 6..421, its header at y 54..80)
/// go in the window: their left edge 16 after the list, the content under the page's heading.
const PAGE_AT: egui::Vec2 = vec2(SIDE_W, -16.0);
/// The page heading's icon tile: 40, 16 from the page area's top and leading edge.
const HEAD_TOP: f32 = 16.0;
const HEAD_TILE: f32 = 40.0;
/// Where a page's controls go, in its English pixels: under its heading, 16 in from the list
/// and the window's edge, 8 above the footer. The test holds every page to it; 08b's pages lay
/// out on it (only the test reads the first three until then).
#[cfg_attr(not(test), allow(dead_code))]
const PAGE_LEFT: f32 = winui::EDGE;
#[cfg_attr(not(test), allow(dead_code))]
const PAGE_RIGHT: f32 = W - SIDE_W - winui::EDGE;
#[cfg_attr(not(test), allow(dead_code))]
const PAGE_TOP: f32 = HEAD_TOP + HEAD_TILE + winui::EDGE - PAGE_AT.y;
const PAGE_BOTTOM: f32 = H - winui::FOOTER_H - 8.0 - PAGE_AT.y;
/// Proxy's Get System, on its heading's row: its English x, where the heading's title ends.
const PROXY_GET_X: f32 = 307.0;

/// The pages in the list's order, which is the old tabs' (Remote, ours, after Connection): page
/// number (as `on_page` and `/show options:<n>` take it), its name in the list, its Graphite
/// icon, and its heading (the old page title).
const PAGES: [(usize, &str, &str, &str); 12] = [
    // i18n: Options page name: general settings
    (
        0,
        crate::i18n::mark("General"),
        "gx-adjust",
        crate::i18n::mark("Browser/System Integration"),
    ),
    // i18n: Options page name
    (
        1,
        crate::i18n::mark("File types"),
        "ft-generic",
        crate::i18n::mark("Downloaded file types"),
    ),
    (
        2,
        // i18n: Options page name: where downloads are saved
        crate::i18n::mark("Save to"),
        "cat-folder",
        crate::i18n::mark("Categories, file types, folders"),
    ),
    (
        3,
        // i18n: Options page name: download settings
        crate::i18n::mark("Downloads"),
        "gx-download",
        crate::i18n::mark("Default download settings"),
    ),
    (
        4,
        // i18n: Options page name: connection settings
        crate::i18n::mark("Connection"),
        "gx-network",
        crate::i18n::mark("Connections and Limits"),
    ),
    (
        REMOTE_PAGE,
        // i18n: Options page name: control MDM from a phone or another computer on the local network
        crate::i18n::mark("Remote"),
        "gx-device",
        // i18n: Options page title: control MDM from a phone or another computer on the local network
        crate::i18n::mark("Remote control"),
    ),
    (
        5,
        // i18n: Options page name
        crate::i18n::mark("Proxy / Socks"),
        "gx-shield",
        crate::i18n::mark("Proxy / socks configuration"),
    ),
    (
        6,
        // i18n: Options page name: user names and passwords for sites
        crate::i18n::mark("Sites Logins"),
        "gx-key",
        crate::i18n::mark("User names and passwords for servers/sites"),
    ),
    (
        7,
        // i18n: Options page name: the user's debrid service accounts (Real-Debrid…)
        crate::i18n::mark("Accounts"),
        "gx-user",
        crate::i18n::mark("Debrid service accounts"),
    ),
    // i18n: Options page name: sound settings
    (
        8,
        crate::i18n::mark("Sounds"),
        "gx-volume",
        crate::i18n::mark("Sound settings"),
    ),
    (
        9,
        // i18n: Options page name: how dates and times are written (date order, 12- or 24-hour clock)
        crate::i18n::mark("Regional"),
        "gx-world",
        crate::i18n::mark("Dates and times"),
    ),
    (
        10,
        // i18n: Options page name: extracting downloaded archives, and Link grabber
        crate::i18n::mark("Archives"),
        "ft-archive",
        // i18n: Options page title: extracting downloaded archives, and Link grabber
        crate::i18n::mark("Archives and Link grabber"),
    ),
];

/// The page's place in [`PAGES`] (0 for a number that is none).
fn page_at(page: usize) -> usize {
    PAGES.iter().position(|p| p.0 == page).unwrap_or(0)
}

/// Side list row `k`'s rect (window-relative to `o`).
fn side_row(o: Pos2, k: usize) -> Rect {
    r(
        o,
        8.0,
        SIDE_TOP + k as f32 * (ROW_H + ROW_GAP),
        SIDE_W - 16.0,
        ROW_H,
    )
}

/// The pages down the side (plan 08a): one Tab stop, the chosen row; Up / Down and Left / Right
/// move through them, Home / End go to the ends, Ctrl+Tab / Ctrl+Shift+Tab (and Ctrl+Page Down
/// / Up) wrap round from anywhere in the window, a click picks one. The page picked.
fn side_list(ui: &mut Ui, o: Pos2, page: usize) -> Option<usize> {
    let c = crate::theme::colors();
    let p = ui.painter().clone();
    let at = page_at(page);
    let foot = H - winui::FOOTER_H;
    p.rect_filled(r(o, 0.0, 0.0, SIDE_W, foot), 0.0, c.panel);
    fill(
        &p,
        o.x + SIDE_W,
        o.y,
        o.x + SIDE_W,
        o.y + foot - 1.0,
        c.menubar_line,
    );
    let mut picked = winui::tab_keys(ui, PAGES.len(), at);
    for (k, (_, label, icon, _)) in PAGES.iter().enumerate() {
        let sel = k == at;
        let rc = side_row(o, k);
        let sense = if sel {
            Sense::click()
        } else {
            winui::pointer_only()
        };
        let resp = ui.interact(rc, egui::Id::unique(("opt-page", k)), sense);
        let label = crate::i18n::t(label);
        winui::expose(&resp, egui::Role::ListItem, label, true, Some(sel));
        let resp = winui::enter_is_not_a_click(ui, resp);
        if resp.clicked() && !sel {
            picked = Some(k);
        }
        if sel {
            winui::keep_keys(ui, &resp, true, true, false);
            if resp.has_focus() {
                let last = PAGES.len() - 1;
                let to = if winui::take_key(ui, egui::Key::ArrowDown)
                    || winui::take_key(ui, egui::Key::ArrowRight)
                {
                    Some((k + 1).min(last))
                } else if winui::take_key(ui, egui::Key::ArrowUp)
                    || winui::take_key(ui, egui::Key::ArrowLeft)
                {
                    Some(k.saturating_sub(1))
                } else if winui::take_key(ui, egui::Key::Home) {
                    Some(0)
                } else if winui::take_key(ui, egui::Key::End) {
                    Some(last)
                } else {
                    None
                };
                if let Some(to) = to {
                    picked = Some(to);
                    // The keys stay on the list: the new row takes the focus.
                    ui.memory_mut(|m| m.request_focus(egui::Id::unique(("opt-page", to))));
                }
            }
        }
        let face = if sel {
            Some(c.selection)
        } else if resp.hovered() {
            Some(c.subtle_hot)
        } else {
            None
        };
        if let Some(face) = face {
            p.rect_filled(rc, winui::control_radius(), face);
        }
        let (ink, glyph) = if sel {
            (c.selection_text, c.accent)
        } else {
            (c.text_secondary, c.text_secondary)
        };
        crate::themeicons::paint_in(
            ui,
            &p,
            icon,
            Rect::from_min_size(pos2(rc.left() + 9.0, rc.center().y - 8.0), vec2(16.0, 16.0)),
            glyph,
            true,
        );
        winui::fit(
            ui,
            &p,
            rc.left() + 34.0,
            winui::baseline_in(rc.top(), rc.height()),
            label,
            ink,
            rc.right() - 6.0,
            ("opt-page-label", k),
        );
        winui::show_focus(ui, &resp, rc);
    }
    if let Some(k) = picked
        && k != at
    {
        Some(PAGES[k].0)
    } else {
        None
    }
}

/// The page's heading above it: its icon on a tile and its title.
fn heading(ui: &Ui, o: Pos2, page: usize) {
    let (_, _, icon, title) = PAGES[page_at(page)];
    let x = SIDE_W + winui::EDGE;
    winui::icon_tile(
        ui,
        r(o, x, HEAD_TOP, HEAD_TILE, HEAD_TILE),
        icon,
        crate::theme::colors().accent,
        8.0,
    );
    let p = ui.painter();
    let left = o.x + x + HEAD_TILE + 12.0;
    let full = crate::i18n::t(title);
    // Proxy's title ends before its Get System, on the same row.
    let end = if page == 5 {
        PAGE_AT.x + PROXY_GET_X - 8.0
    } else {
        W - winui::EDGE
    };
    let shown = winui::ellipsize(p, &winui::tahoma_title(), full, o.x + end - left);
    let drawn = winui::text_at(
        p,
        winui::tahoma_title(),
        left,
        o.y + winui::baseline_in(HEAD_TOP, HEAD_TILE),
        &shown,
        crate::theme::colors().text,
    );
    if shown != full {
        winui::full_on_hover(ui, drawn, ("opt-heading", page), full);
    }
}

/// Group box (`#dcdcdc` frame) with its caption (translated here) at x + 9.
fn group(ui: &Ui, o: Pos2, x: f32, y: f32, w: f32, h: f32, caption: &str) {
    let p = ui.painter();
    let full = crate::i18n::t(caption);
    // A caption longer than the box is cut with "..." before its right edge (whole on hover).
    let caption = winui::ellipsize(p, &winui::tahoma(), full, w - 18.0);
    let cw = winui::text_width(p, &caption);
    winui::group_box(
        p,
        r(o, x, y, w, h),
        Some((o.x + x + 7.0, o.x + x + 9.0 + cw + 2.0)),
    );
    let rc = text(p, o.x + x + 9.0, o.y + y + 4.0, &caption, LIGHT_TEXT());
    if caption != full {
        winui::full_on_hover(ui, rc, ("group", caption.as_str()), full);
    }
}

/// A label (translated here) that IDM greys out with the classic embossed look when its
/// controls are off.
fn disabled_label(p: &egui::Painter, x: f32, base: f32, s: &str, enabled: bool, right: bool) {
    let s = crate::i18n::t(s);
    let x = if right {
        x - winui::text_width(p, s)
    } else {
        x
    };
    winui::label(p, x, base, s, enabled);
}

pub(crate) fn list_frame(p: &egui::Painter, rc: Rect) {
    p.rect_filled(rc, 0.0, winui::FIELD());
    p.rect_stroke(
        rc,
        0.0,
        egui::Stroke::new(1.0, winui::FIELD_EDGE()),
        egui::StrokeKind::Inside,
    );
}

/// A report list (header 25 px, rows 17 px): `cols` are (title, right edge in client x).
/// Returns true on a double click of a row.
#[allow(clippy::too_many_arguments)]
fn report(
    ui: &mut Ui,
    o: Pos2,
    rc: Rect,
    cols: &[(&str, f32)],
    rows: &[Vec<String>],
    st: &mut winui::RowList,
    grid: bool,
    id: &str,
) -> (bool, bool) {
    let p = ui.painter().clone();
    list_frame(&p, rc);
    // Laid out in English, as `cols` are; each part made on screen.
    let rc = winui::rect(rc);
    let left = rc.left();
    let head = rc.top() + 1.0;
    let divider = winui::HEADER_LINE();
    fill(
        &p,
        left + 1.0,
        head,
        rc.right() - 2.0,
        head + 23.0,
        winui::LIST_HEADER(),
    );
    let mut x0 = left;
    for (i, (title, right)) in cols.iter().enumerate() {
        let title = crate::i18n::t(title);
        let right = o.x + right;
        let tx = if i == 0 { x0 + 8.0 } else { x0 + 7.0 };
        let (rect, cut) = winui::text_fit(&p, tx, head + 16.0, title, LIGHT_TEXT(), right - 3.0);
        if cut {
            winui::full_on_hover(ui, rect, (id, "head", i), title);
        }
        fill(&p, right, head + 1.0, right, head + 23.0, divider);
        x0 = right;
    }
    // The rows: the shared row list under the header (its frame on the list's).
    let body = Rect::from_min_max(
        pos2(rc.left() + 1.0, head + 25.0),
        pos2(rc.right() - 1.0, rc.bottom() - 1.0),
    );
    let gridc = winui::GRID();
    let ev = winui::row_list(
        ui,
        (id, "rows"),
        winui::rect(body.expand(1.0)),
        rows.len(),
        17.0,
        st,
        |ui, clip, k, line, look| {
            let top = line.top();
            // Read as its cells, joined.
            look.name = rows[k]
                .iter()
                .filter(|c| !c.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            if grid {
                fill(
                    clip,
                    line.left(),
                    top + 16.0,
                    line.right(),
                    top + 16.0,
                    gridc,
                );
            }
            let mut cx = left;
            for (i, (_, right)) in cols.iter().enumerate() {
                let right = o.x + right;
                if grid {
                    fill(clip, right + 1.0, top, right + 1.0, top + 16.0, gridc);
                }
                if let Some(cell) = rows[k].get(i) {
                    winui::fit(
                        ui,
                        clip,
                        cx + 6.0,
                        top + 12.0,
                        cell,
                        look.ink,
                        right - 3.0,
                        (id, "cell", k, i),
                    );
                }
                cx = right;
            }
        },
    );
    if ev.focused {
        st.type_to(ui, id, rows.len(), |i| {
            rows[i].first().cloned().unwrap_or_default()
        });
    }
    // A double-click or Enter edits (Enter taken: not the dialog's OK too); Delete removes.
    if ev.entered.is_some() {
        winui::take_key(ui, egui::Key::Enter);
    }
    (
        ev.double_clicked.or(ev.entered).is_some(),
        ev.delete.is_some(),
    )
}

/// A multi-line edit that wraps its text (the shared one).
fn multiline(ui: &mut Ui, rc: Rect, value: &mut String, id: &str, enabled: bool) {
    winui::multiline_light(ui, rc, value, id, enabled);
}

/// A read-only field (the page colour, as IDM's): the shared one, so it can be copied.
fn readonly(ui: &mut Ui, rc: Rect, value: &str) {
    winui::readonly_field(
        ui,
        rc,
        value,
        LIGHT_PAGE(),
        false,
        ("options", rc.left() as i32, rc.top() as i32),
    );
}

/// [`readonly`] for an address: left to right in every language.
fn readonly_ltr(ui: &mut Ui, rc: Rect, value: &str) {
    winui::readonly_field(
        ui,
        rc,
        value,
        LIGHT_PAGE(),
        true,
        ("options", rc.left() as i32, rc.top() as i32),
    );
}

/// A password edit (characters shown as bullets): the shared one, on the page.
fn password(ui: &mut Ui, rc: Rect, value: &mut String, enabled: bool) {
    winui::password_light_on(ui, rc, value, enabled, LIGHT_PAGE());
}

/// Options → Accounts: where the sign-ins (and Sites Logins' passwords, the proxy's) are kept,
/// as `secrets::Vault::kept` found it. None while it is still looking.
fn kept_text(kept: crate::secrets::Kept) -> Option<&'static str> {
    use crate::secrets::Kept;
    Some(match kept {
        Kept::Looking => return None,
        Kept::Keychain => {
            // i18n: Options → Accounts: where the debrid accounts' sign-ins and the other passwords are kept (Windows' own Credential Manager, in Control Panel)
            crate::i18n::mark("Sign-ins and passwords are kept in Windows Credential Manager.")
        }
        // i18n: Options → Accounts: a portable copy (it writes nothing outside its folder) keeps the sign-ins in its own settings, encrypted so that only this Windows account can read them
        Kept::Portable => crate::i18n::mark(
            "This portable copy keeps sign-ins and passwords in its own settings, encrypted for your Windows account.",
        ),
        // i18n: Options → Accounts: the system keyring is locked (MDM never unlocks it: that would ask for its password), so the sign-ins stay in MDM's settings folder, which only the user's account may read
        Kept::Locked => crate::i18n::mark(
            "The system keyring is locked and MDM doesn't unlock it, so sign-ins and passwords are kept in MDM's settings, which only you can read.",
        ),
        // i18n: Options → Accounts: Windows Credential Manager didn't answer, so the sign-ins stay in MDM's settings, encrypted so that only this Windows account can read them
        Kept::Missing => crate::i18n::mark(
            "Windows Credential Manager didn't answer, so sign-ins and passwords are kept in MDM's settings, encrypted for your Windows account.",
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::accesskit::{NodeId, Role, TreeUpdate};

    fn input(events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(W, H))),
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

    /// A press (`down`) or release of the left button at `at`.
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

    /// One pass of the side list on `page`: the page it picked.
    fn list_pass(ctx: &egui::Context, page: usize, events: Vec<egui::Event>) -> Option<usize> {
        let mut picked = None;
        let mut out = ctx.run_ui(input(events), |ui| picked = side_list(ui, Pos2::ZERO, page));
        out.textures_delta.clear();
        picked
    }

    /// Plan 08 tests: every page is picked by a click on its row, Up / Down walk the list in
    /// its order (Home / End to its ends), and Ctrl+Tab / Ctrl+Shift+Tab wrap round it.
    #[test]
    fn every_page_is_reached_by_click_arrows_and_ctrl_tab() {
        let ctx = egui::Context::default();
        let order: Vec<usize> = PAGES.iter().map(|p| p.0).collect();
        assert_eq!(
            order,
            [0, 1, 2, 3, 4, 11, 5, 6, 7, 8, 9, 10],
            "the old tabs' order"
        );
        for (_, label, need, _) in PAGES {
            assert!(
                crate::themeicons::graphite_has(need, 16)
                    && crate::themeicons::graphite_has(need, 24),
                "{label}: Graphite's set lacks {need}"
            );
        }
        for (k, &want) in order.iter().enumerate() {
            let from = if want == 0 { 1 } else { 0 };
            let at = side_row(Pos2::ZERO, k).center();
            list_pass(&ctx, from, vec![egui::Event::PointerMoved(at)]);
            list_pass(&ctx, from, button(at, true));
            assert_eq!(
                list_pass(&ctx, from, button(at, false)),
                Some(want),
                "click row {k}"
            );
        }
        // A key on `page`'s row, focused as Tab leaves it.
        let press = |page: usize, k: egui::Key| {
            list_pass(&ctx, page, vec![]);
            ctx.memory_mut(|m| m.request_focus(egui::Id::unique(("opt-page", page_at(page)))));
            list_pass(&ctx, page, vec![key(k, Default::default())])
        };
        for w in order.windows(2) {
            assert_eq!(
                press(w[0], egui::Key::ArrowDown),
                Some(w[1]),
                "Down from {}",
                w[0]
            );
            assert_eq!(
                press(w[1], egui::Key::ArrowUp),
                Some(w[0]),
                "Up from {}",
                w[1]
            );
        }
        assert_eq!(press(10, egui::Key::ArrowDown), None, "the end");
        assert_eq!(press(0, egui::Key::ArrowUp), None, "the start");
        assert_eq!(press(4, egui::Key::Home), Some(0));
        assert_eq!(press(4, egui::Key::End), Some(10));
        // Ctrl+Tab from anywhere: no row focused.
        ctx.memory_mut(|m| m.surrender_focus(egui::Id::unique(("opt-page", 0))));
        let ctrl = egui::Modifiers::COMMAND;
        let mut page = order[0];
        for &want in order.iter().cycle().skip(1).take(order.len()) {
            page = list_pass(&ctx, page, vec![key(egui::Key::Tab, ctrl)]).expect("Ctrl+Tab");
            assert_eq!(page, want);
        }
        let back = list_pass(
            &ctx,
            0,
            vec![key(egui::Key::Tab, ctrl | egui::Modifiers::SHIFT)],
        );
        assert_eq!(back, Some(10), "Ctrl+Shift+Tab wraps back to Archives");
    }

    /// Plan 08b3: a sub-dialog's footer keeps its keys: Enter is OK only while the entry is
    /// valid, Escape is Cancel either way.
    #[test]
    fn sub_dialog_enter_is_ok_only_when_valid_escape_cancels() {
        let ctx = egui::Context::default();
        let pass = |k: egui::Key, valid: bool| {
            let mut out = Outcome::Open;
            let mut o = ctx.run_ui(input(vec![key(k, Default::default())]), |ui| {
                out = ok_cancel(ui, Pos2::ZERO, 340.0, 100.0, valid);
            });
            o.textures_delta.clear();
            out
        };
        assert!(matches!(pass(egui::Key::Enter, true), Outcome::Done(())));
        assert!(matches!(pass(egui::Key::Enter, false), Outcome::Open));
        assert!(matches!(pass(egui::Key::Escape, true), Outcome::Closed));
        assert!(matches!(pass(egui::Key::Escape, false), Outcome::Closed));
    }

    /// One pass of the window's page area on `page`, AccessKit on: its tree.
    fn page_pass(
        ctx: &egui::Context,
        d: &mut OptionsDialog,
        events: Vec<egui::Event>,
    ) -> TreeUpdate {
        let dir = std::env::temp_dir();
        let mut out = ctx.run_ui(input(events), |ui| d.page_area(ui, Pos2::ZERO, &dir));
        out.textures_delta.clear();
        out.platform_output
            .accesskit_update
            .expect("AccessKit is on")
    }

    /// Plan 08 tests: at its new place every page's controls lie in the page area (right of the
    /// list, above the footer), and its first control is hit where it is drawn: a click on it
    /// gives it the keys.
    #[test]
    fn every_page_lies_in_its_area_and_is_hit_where_drawn() {
        let area = Rect::from_min_max(pos2(PAGE_LEFT, PAGE_TOP), pos2(PAGE_RIGHT, PAGE_BOTTOM))
            .translate(PAGE_AT)
            // Proxy's Get System sits on the heading's row.
            .with_min_y(HEAD_TOP)
            // A check box's click area runs on to the window's edge after its caption.
            .with_max_x(W);
        for (page, label, ..) in PAGES {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let s = Settings::default();
            let mut d = OptionsDialog::new(&s, &std::env::temp_dir(), vec![]).on_page(page);
            page_pass(&ctx, &mut d, vec![]);
            let tree = page_pass(&ctx, &mut d, vec![]);
            let controls: Vec<(NodeId, Rect, u8)> = tree
                .nodes
                .iter()
                .filter(|(_, n)| {
                    matches!(
                        n.role(),
                        Role::Button
                            | Role::CheckBox
                            | Role::RadioButton
                            | Role::TextInput
                            | Role::ComboBox
                            | Role::ListBox
                            | Role::SpinButton
                    )
                })
                .filter_map(|(id, n)| {
                    let b = n.bounds()?;
                    let rc = Rect::from_min_max(
                        pos2(b.x0 as f32, b.y0 as f32),
                        pos2(b.x1 as f32, b.y1 as f32),
                    );
                    // A Tab stop (0): not greyed, nor pointer-only (`winui::pointer_only`, spin
                    // arrows). A list (1) only where there is nothing else (Sounds, its buttons
                    // greyed until a row is chosen): its first row is clicked, its blank area
                    // takes no click.
                    let focusable =
                        !n.is_disabled() && n.supports_action(egui::accesskit::Action::Focus);
                    let rank = match (focusable, n.role() == Role::ListBox) {
                        (true, false) => 0,
                        (true, true) => 1,
                        _ => 2,
                    };
                    Some((*id, rc, rank))
                })
                .collect();
            assert!(!controls.is_empty(), "{label}: no controls");
            // A caption's limit is a window x: Sites Logins' "From browser:" was cut down to
            // "..." when given as a page x.
            if page == 6 {
                let b = tree
                    .nodes
                    .iter()
                    .find(|(_, n)| n.label() == Some("From browser:"))
                    .and_then(|(_, n)| n.bounds())
                    .expect("From browser:");
                assert!(b.x1 - b.x0 > 60.0, "\"From browser:\" cut: {b:?}");
            }
            for (_, rc, _) in &controls {
                assert!(
                    area.expand(0.5).contains_rect(*rc),
                    "{label}: {rc:?} outside {area:?}"
                );
            }
            // The page's first control (a Tab stop, topmost, then leftmost), clicked.
            let (first, rc, rank) = *controls
                .iter()
                .filter(|c| c.2 < 2)
                .min_by(|a, b| {
                    (a.2, a.1.top(), a.1.left())
                        .partial_cmp(&(b.2, b.1.top(), b.1.left()))
                        .unwrap()
                })
                .unwrap_or_else(|| panic!("{label}: no Tab stop among {controls:?}"));
            let at = if rank == 1 {
                pos2(rc.center().x, rc.top() + 8.0)
            } else {
                rc.center()
            };
            page_pass(&ctx, &mut d, button(at, true));
            page_pass(&ctx, &mut d, button(at, false));
            let tree = page_pass(&ctx, &mut d, vec![]);
            assert_eq!(
                tree.focus, first,
                "{label}: the click at {rc:?} reached its control"
            );
        }
    }

    /// #424: the list written and read back through a real file: nothing lost, nothing doubled.
    #[test]
    fn sites_round_trip_through_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sites.txt");
        let sites = "a.test *.b.test";
        std::fs::write(&file, mdm_ipc::capture::sites_to_text(sites)).unwrap();
        let text = read_sites_file(&file).unwrap();
        assert_eq!(text, "a.test\n*.b.test\n");
        let (back, added) = mdm_ipc::capture::merge_sites("", &text);
        assert_eq!((back.as_str(), added), (sites, 2));
        assert_eq!(mdm_ipc::capture::merge_sites(sites, &text).1, 0);
    }
}
