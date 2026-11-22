//! The browser extension's offer: at each start, until a browser's extension has reached the
//! app (the native host notes it: mdm_ipc::extension::seen), a window lists the browsers
//! installed and, for the one picked, opens the tutorial's section that loads the copy in
//! Grabbit's folder (Grabbit has no store listing yet; once the Chrome Web Store has it, Chrome,
//! Edge, Brave, Opera and Vivaldi get its page, and registering the host names it to Chrome,
//! crates/nmhost). "Don't ask again" stops the offer; `/show extension` brings it back.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use egui::{Pos2, Ui};

use crate::dialogs::{Outcome, window};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, r};

const W: f32 = 520.0;
const BANNER: f32 = 84.0;
const ROW: f32 = 34.0;

/// Whether to offer the extension at this start: browser integration on and registered (not a
/// portable copy, which registers nothing), no browser's extension seen yet, not refused.
pub fn worth_asking(settings: &crate::settings::Settings) -> bool {
    !settings.extension_never_ask
        && settings.advanced_integration
        && mdm_ipc::portable_dir().is_none()
        && !mdm_ipc::extension::seen()
}

pub struct Extension {
    /// The browsers installed: name and program. Empty: one row for the system's browser.
    browsers: Vec<(String, Option<PathBuf>)>,
    /// A browser's extension has reached the app since the window opened.
    connected: bool,
    /// The browsers whose extension has (Options' names, mdm_ipc::extension::seen_browsers).
    seen: Vec<String>,
    /// The browsers can find the app (the native host is registered).
    registered: bool,
    /// When the status files were read last (once a second, not every frame).
    looked: std::time::Instant,
    /// The read under way beside the window, one at a time ([`Reading`]).
    reading: Option<Reading<Looked>>,
    /// Set it up's runs seen finished ([`set_up_finished`]).
    set_up_seen: u64,
}

/// What the window shows of the extension's state, as read from the status files, the
/// browsers' folders and the registry.
struct Looked {
    connected: bool,
    seen: Vec<String>,
    registered: bool,
}

impl Looked {
    fn now() -> Looked {
        Looked {
            connected: mdm_ipc::extension::seen(),
            seen: seen_names(),
            registered: registered(),
        }
    }
}

/// A read made on a thread of its own, its answer taken once it came: a browser's folder slow
/// to answer (a sleeping disk, a profile on the network) held the window's thread each second
/// the window was open (upstream issue 1218).
struct Reading<T> {
    answer: std::sync::mpsc::Receiver<T>,
}

impl<T: Send + 'static> Reading<T> {
    /// Start `read` beside the window; the window repaints when it's done. None when no thread
    /// could be made (the window keeps what it showed).
    fn start(ctx: &egui::Context, read: impl FnOnce() -> T + Send + 'static) -> Option<Self> {
        let (tx, answer) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("extension status".into())
            .spawn(move || {
                let _ = tx.send(read());
                ctx.request_repaint();
            })
            .ok()?;
        Some(Reading { answer })
    }

    /// The answer, once it came (`Some(None)` when the read ended without one); None while it
    /// runs.
    fn take(&self) -> Option<Option<T>> {
        match self.answer.try_recv() {
            Ok(v) => Some(Some(v)),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(None),
        }
    }
}

/// Whether `row` (a browser as the window lists it) is among the browsers seen (as Options
/// names them). Vivaldi, Chromium, Helium, Thorium and Arc say they are Chrome (no brand of
/// their own, or Chrome's on purpose): they share Chrome's status. Firefox's forks give their
/// own name (`capture::browser_named`).
fn is_seen(seen: &[String], row: &str) -> bool {
    let named = match row {
        "Brave" => "Brave Browser",
        "Vivaldi" | "Chromium" | "Helium" | "Thorium" | "Arc" => "Google Chrome",
        other => other,
    };
    seen.iter().any(|b| b == named)
}

/// A row for a browser of Firefox's family (Firefox, Zen, LibreWolf …): it gets the tutorial's
/// Firefox section, not the Chrome Web Store.
fn is_gecko(row: &str) -> bool {
    mdm_ipc::browsers::named(row).is_some_and(|b| b.family == mdm_ipc::browsers::Family::Gecko)
}

/// Where a row's button goes: the store's page (`store`, None while Grabbit isn't listed), else
/// the tutorial's section for that browser (a docs/ path, for help::page_url). Firefox's family
/// has no listing: always its section.
fn row_target<'a>(row: &str, store: Option<&'a str>) -> (Option<&'a str>, &'static str) {
    if is_gecko(row) {
        (None, mdm_ipc::extension::GUIDE_FIREFOX)
    } else {
        (store, mdm_ipc::extension::GUIDE_CHROMIUM)
    }
}

impl Extension {
    pub fn new() -> Self {
        let found = crate::win32::browsers();
        let mut browsers: Vec<_> = found.into_iter().map(|(n, p)| (n, Some(p))).collect();
        // The table's order (Chrome first, its button the accent one), others after.
        browsers.sort_by_key(|(n, _)| {
            mdm_ipc::browsers::BROWSERS
                .iter()
                .position(|b| b.name == n.as_str())
                .unwrap_or(usize::MAX)
        });
        if browsers.is_empty() {
            browsers.push((String::new(), None));
        }
        Extension {
            browsers,
            connected: mdm_ipc::extension::seen(),
            seen: seen_names(),
            registered: registered(),
            looked: std::time::Instant::now(),
            reading: None,
            set_up_seen: 0,
        }
    }

    /// The host marks the extension seen when a browser starts it: look again now and then
    /// (each browser's own, as each adds it), beside the window, one read at a time; what was
    /// read last shows meanwhile. Every frame the window or the page's section is drawn.
    fn refresh(&mut self, ctx: &egui::Context) {
        if let Some(answer) = self.reading.as_ref().and_then(Reading::take) {
            self.reading = None;
            if let Some(l) = answer {
                self.connected = l.connected;
                self.seen = l.seen;
                self.registered = l.registered;
            }
        }
        if self.reading.is_none()
            && (self.looked.elapsed() >= Duration::from_secs(1)
                || set_up_finished(&mut self.set_up_seen))
        {
            self.looked = std::time::Instant::now();
            self.reading = Reading::start(ctx, Looked::now);
        }
        ctx.request_repaint_after(Duration::from_secs(1));
    }

    /// The rows' height with the window's footer band (`body`'s `h`).
    fn height(&self) -> f32 {
        BANNER + 16.0 + self.browsers.len() as f32 * ROW + 104.0
    }

    /// The section's layout size (plan 11b2): the window's client without its footer, a long
    /// translation's banner included (`winui::banner`).
    pub fn size(&self, ctx: &egui::Context) -> [f32; 2] {
        let extra = crate::dialogs::grow_extra(ctx, "extension-banner");
        [W, self.height() - 32.0 + extra]
    }

    /// The browser extension as a section of Options' page (plan 11b2): the window's rows at
    /// `o`, [`Extension::size`] points, without its footer (nothing to save; the offer at a
    /// start, with Don't ask again, stays the window).
    pub fn section(&mut self, ui: &mut Ui, o: Pos2) {
        self.refresh(ui.ctx());
        let h = self.height();
        let size = self.size(ui.ctx());
        crate::dialogs::inline(ui, o, size, winui::BG(), |ui, o| {
            self.body(ui, o, h, None);
        });
    }

    /// Open until closed; Done when "Don't ask again" was pressed.
    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<()> {
        self.refresh(ctx);
        let h = self.height();
        // A long translation's subline makes the banner taller (winui::banner).
        let extra = crate::dialogs::grow_extra(ctx, "extension-banner");
        let mut answer = None;
        window(
            ctx,
            "extension",
            // i18n: title of the window that offers the browser extension
            "Add Grabbit to your browser",
            [W, h - 32.0 + winui::FOOTER_H + extra],
            false,
            winui::BG(),
            |ui, o| {
                self.body(ui, o, h, Some(&mut answer));
                match answer.take() {
                    Some(true) => Outcome::Done(()),
                    Some(false) => Outcome::Closed,
                    None if winui::escape_pressed(ui) => Outcome::Closed,
                    None => Outcome::Open,
                }
            },
        )
    }

    /// Its rows from `o`, `h` tall with the window's footer; `answer` is the window's (None in
    /// the page's section: no footer).
    fn body(&mut self, ui: &mut Ui, o: Pos2, h: f32, answer: Option<&mut Option<bool>>) {
        let p = ui.painter().clone();
        let (ink, muted) = (winui::TEXT(), winui::MUTED());
        // The banner: what the extension does, in one line, and how.
        let extra = winui::banner(
            ui,
            &p,
            o,
            W,
            "extension-banner",
            // i18n: the headline of the window that offers the browser extension
            crate::i18n::t("Catch downloads from your browser"),
            crate::i18n::t(
                "Add the free extension: files you download in your browser come here, and videos get a Download button.",
            ),
        );
        // Everything below it moves down with a taller banner.
        let o = o + egui::vec2(0.0, extra);

        // One row per browser: its name, and the button that opens the extension's page in it:
        // the Chrome Web Store's once Grabbit is listed there; until then (and always for
        // Firefox's family, which has no listing) the tutorial's section for that browser.
        let store = mdm_ipc::extension::store_page();
        for (k, (name, program)) in self.browsers.iter().enumerate() {
            let y = BANNER + 16.0 + k as f32 * ROW;
            let (page, guide) = row_target(name, store.as_deref());
            winui::text_font(
                &p,
                winui::tahoma_bold(),
                o.x + 22.0,
                o.y + y + 16.0,
                if name.is_empty() {
                    // i18n: a row of the browser extension's window when no browser was found by name
                    crate::i18n::t("Your browser")
                } else {
                    name.as_str()
                },
                ink,
            );
            let label = if page.is_none() {
                // i18n: button in the browser extension's window: open the tutorial that adds the extension
                "Show me how"
            } else {
                // i18n: button in the browser extension's window: open the extension's page in that browser
                "Add the extension"
            };
            // As wide as its label in this language (#192).
            let (add_x, add_w) =
                winui::buttons_to_right(&p, W - 18.0, 8.0, &[(label, 150.0)], 220.0)[0];
            // This browser's extension is there already: said, before its button.
            if is_seen(&self.seen, name) {
                winui::text_right(
                    &p,
                    o.x + add_x - 10.0,
                    o.y + y + 16.0,
                    &format!(
                        "\u{2714} {}",
                        // i18n: in the browser extension's window, beside a browser whose extension reached MDM
                        crate::i18n::t("Connected")
                    ),
                    winui::SUCCESS(),
                );
            }
            let add = winui::button(
                ui,
                r(o, add_x, y, add_w, 25.0),
                label,
                if k == 0 {
                    ButtonKind::Accent
                } else {
                    ButtonKind::Normal
                },
                false,
                None,
            );
            // Every row's button reads the same: a screen reader hears which browser it is for.
            let browser = if name.is_empty() {
                crate::i18n::t("Your browser")
            } else {
                name.as_str()
            };
            let spoken = crate::i18n::fmt(
                // i18n: what a screen reader calls a button that one row of a window has for each thing it lists; {label} is the button's text, {what} the row's browser or program
                crate::i18n::t("{label} for {what}"),
                &[
                    ("label", crate::i18n::t(label)),
                    ("what", &crate::i18n::ltr(browser)),
                ],
            );
            ui.ctx()
                .accesskit_node_builder(add.id, |n| n.set_label(spoken));
            if add.clicked() {
                // No store page: the tutorial on the website, at this browser's section.
                let url = page.map_or_else(|| crate::help::page_url(guide), str::to_owned);
                let url = url.as_str();
                match program {
                    Some(program) => crate::win32::open_in(program, url),
                    None => {
                        crate::launch::open(url);
                    }
                }
            }
        }

        // Where it stands, then the answer. Not set up at all (the setup's box unticked): no
        // extension could reach the app, so that comes first, with the fix.
        let y = h - 84.0;
        // A copy Set it up can never register (the native host refuses it): why, and what to
        // do, in place of a button that would do nothing.
        let never = if !self.registered && mdm_ipc::portable_dir().is_some() {
            // i18n: in the browser extension's window, a portable copy (it writes nothing outside its folder)
            Some(
                "This portable copy can't be connected to a browser: it writes nothing outside its folder. Install Grabbit to use the extension.",
            )
        } else if !self.registered && temporary_copy() {
            // i18n: in the browser extension's window, Grabbit started from inside a zip (Windows deletes that folder once it quits)
            Some(
                "Grabbit runs from inside a zip, a folder Windows deletes once it quits. Extract it to a folder of its own and start it from there.",
            )
        } else {
            None
        };
        if let Some(never) = never {
            winui::paragraph(
                &p,
                o.x + 18.0,
                o.y + y,
                crate::i18n::t(never),
                winui::ERROR(),
                o.x + W - 18.0,
                crate::theme::metrics().line_paragraph,
            );
        } else if !self.registered {
            // As wide as its label in this language (#192); the paragraph ends 10 px before it.
            let (fix_x, fix_w) =
                winui::buttons_to_right(&p, W - 18.0, 8.0, &[("Set it up", 110.0)], 180.0)[0];
            // While it runs, then why it failed if it did (the full reason on the button's
            // hover; Options → General → Test browser integration lists it too).
            let failed = SET_UP_ERROR.lock().ok().and_then(|e| e.clone());
            let said = if SETTING_UP.load(Ordering::Acquire) {
                // i18n: in the browser extension's window, while Set it up runs
                crate::i18n::t("Setting up browser integration...")
            } else if failed.is_some() {
                // i18n: in the browser extension's window, after Set it up failed; pointing at the button shows the reason
                crate::i18n::t(
                    "Set it up didn't work. Point at the button to see why, or try it again.",
                )
            } else {
                // i18n: in the browser extension's window, beside Set it up: the native host isn't registered with the browsers
                crate::i18n::t(
                    "Browser integration isn't set up on this computer, so no browser can reach Grabbit yet.",
                )
            };
            winui::paragraph(
                &p,
                o.x + 18.0,
                o.y + y,
                said,
                winui::ERROR(),
                o.x + fix_x - 10.0,
                crate::theme::metrics().line_paragraph,
            );
            let mut fix = winui::button(
                ui,
                r(o, fix_x, y - 4.0, fix_w, 25.0),
                // i18n: button in the browser extension's window: register MDM's helper with the browsers
                "Set it up",
                ButtonKind::Accent,
                false,
                None,
            );
            if let Some(why) = failed {
                fix = fix.on_hover_text(why);
            }
            if fix.clicked() {
                set_up(ui.ctx());
            }
        }
        let (line, color) = if !self.registered {
            (String::new(), muted)
        } else if self.connected {
            (
                crate::i18n::t("Connected: downloads from your browser now come here.").to_owned(),
                winui::SUCCESS(),
            )
        } else if store.is_some() {
            (
                crate::i18n::t(
                    "Once it is added, this window says so. Chrome may also ask by itself at its next start.",
                )
                .to_owned(),
                muted,
            )
        } else {
            (
                // i18n: in the browser extension's window, under the browsers, while it has no store listing
                crate::i18n::t(
                    "The extension comes with Grabbit: Show me how loads it from Grabbit's folder. Once it is added, this window says so.",
                )
                .to_owned(),
                muted,
            )
        };
        if !line.is_empty() {
            let lh = crate::theme::metrics().line_paragraph;
            winui::paragraph(&p, o.x + 18.0, o.y + y, &line, color, o.x + W - 18.0, lh);
        }
        // The fallback, when a store can't be used (a managed browser, a store turned off): the
        // tutorial, which adds it by hand.
        if store.is_some()
            && winui::link(
                ui,
                &p,
                o.x + 18.0,
                o.y + h - 48.0,
                // i18n: link in the browser extension's window: the tutorial that adds it without the browser's store
                "Install it by hand",
                "by-hand",
            )
        {
            crate::help::open_page(mdm_ipc::extension::GUIDE_DOC);
        }
        // As a section of Options' page (plan 11b2) there is no footer: the page has its own
        // bar, and the offer's answers are the window's.
        let Some(answer) = answer else {
            return;
        };
        // The footer: Don't ask again (quiet, at its start) only while no extension has reached
        // MDM, and Close, which Enter presses, as before (Escape closes it too, in `show`).
        let mut items = vec![FooterItem::Button(
            FooterButton::new("Close", ButtonKind::Default).enter(true),
        )];
        if !self.connected {
            // i18n: button in the browser extension's window: stop offering it at each start
            items.push(FooterItem::Button(
                FooterButton::new("Don't ask again", ButtonKind::Ghost).leading(),
            ));
        }
        match winui::footer(ui, o, W, h - 32.0, &mut items) {
            Some(0) => *answer = Some(false),
            Some(_) => *answer = Some(true),
            None => {}
        }
    }
}

/// The browsers whose extension reached the app, by Options' names.
fn seen_names() -> Vec<String> {
    mdm_ipc::extension::seen_browsers()
        .into_iter()
        .map(|(b, _)| b)
        .collect()
}

/// Where browser integration's registration stands on this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Registration {
    /// The browsers find the native host's manifest, and the helper program it names is there.
    Ready,
    /// A browser's folder (or the registry) has no manifest.
    Missing,
    /// A manifest names a helper program that isn't there or can't be run (#253).
    HelperMissing(PathBuf),
    /// The browsers are pointed at a manifest that is gone, with the copy that wrote it: one run
    /// from inside a zip, whose temporary folder was deleted (upstream issue 818).
    Gone(PathBuf),
}

/// Where browser integration's registration stands: the native host's manifests where the
/// browsers look, and the helper they name.
pub fn registration() -> Registration {
    crate::win32::host_registration()
}

/// Is browser integration registered on this computer, ready for the browsers?
pub fn registered() -> bool {
    registration() == Registration::Ready
}

/// Why the last registration failed (at a start, or Set it up's), in the native host's words;
/// None once one succeeded.
static REGISTER_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Why the last registration failed, if it did (Test browser integration says it).
fn register_error() -> Option<String> {
    REGISTER_ERROR.lock().ok().and_then(|e| e.clone())
}

/// Note how a registration went: logged, and its failure kept for Test browser integration.
pub(crate) fn registered_as(result: &Result<String, String>) {
    match result {
        Ok(said) => tracing::info!("browser integration set up: {said}"),
        Err(e) => tracing::warn!("browser integration not set up: {e}"),
    }
    if let Ok(mut last) = REGISTER_ERROR.lock() {
        *last = result.as_ref().err().cloned();
    }
}

/// The native host beside this program run with `arg` (`--register`, `--repair`), waiting up
/// to `limit`: what it says it did, or why it failed.
pub(crate) fn run_host(arg: &str, limit: Duration) -> Result<String, String> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    let host = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .with_file_name("grabbit-nmhost.exe");
    let mut cmd = Command::new(&host);
    cmd.arg(arg)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::win32::no_console(&mut cmd);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{}: {e}", host.display()))?;
    let status = crate::launch::wait_within(&mut child, limit);
    // A few lines at most: they fit the pipes, so the helper never waits on them.
    let mut said = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut said);
    }
    if let Some(mut err) = child.stderr.take() {
        said.push('\n');
        let _ = err.read_to_string(&mut said);
    }
    let said = said.split_whitespace().collect::<Vec<_>>().join(" ");
    match status {
        Some(s) if s.success() => Ok(said),
        Some(s) => Err(format!("{} {arg}: {s}: {said}", host.display())),
        None => Err(format!("{} {arg}: no answer in {limit:?}", host.display())),
    }
}

/// This copy runs from a folder that is deleted later (upstream issue 818): an archive run
/// without extracting it first (WinRAR, 7-Zip and Windows Explorer extract it to a temporary
/// folder, deleted once it quits), or anything under the temporary folder. It registers nothing
/// with the browsers, which would be left with a helper that is gone.
pub fn temporary_copy() -> bool {
    static TEMPORARY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TEMPORARY.get_or_init(|| {
        std::env::current_exe().is_ok_and(|p| mdm_ipc::browsers::in_temporary_folder(&p))
    })
}

/// At each start, on a thread of its own (not in a portable copy, which registers nothing, nor
/// an off-screen test run, whose browsers are the person's): a browser's registration that leads
/// nowhere is repaired for this copy, one that works is left alone, whichever copy it names
/// (upstream issue 818).
pub fn repair_at_start() {
    if mdm_ipc::portable_dir().is_some() || crate::offscreen() {
        return;
    }
    if temporary_copy() {
        tracing::warn!(
            "browser integration: not set up, this copy runs from a temporary folder ({}), \
             deleted once it quits: extract it to a folder of its own",
            std::env::current_exe().unwrap_or_default().display()
        );
        return;
    }
    crate::win32::repair_browsers();
}

/// Set it up's runs finished so far: a window that shows the checks reads them again when this
/// changes ([`set_up_finished`]).
static SET_UP_DONE: AtomicU64 = AtomicU64::new(0);
/// A Set it up is running (a second press waits for it).
static SETTING_UP: AtomicBool = AtomicBool::new(false);
/// Why the last Set it up failed (None: none failed). Its own: a start's `--repair` that failed
/// isn't the button's failure (REGISTER_ERROR keeps either, for Test browser integration).
static SET_UP_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
/// How long the native host's `--register` has (it writes a few files, or registry keys).
pub(crate) const SET_UP_LIMIT: Duration = Duration::from_secs(20);

/// Register it now, for a copy installed without it (the setup's box unticked), or to take the
/// browsers back from another copy: the native host's own `--register`, for this user, on a
/// thread of its own (a program waited for on the window's thread freezes it for as long as the
/// program takes, #222). The window repaints when it's done, and [`set_up_finished`] tells the
/// checks to read the result, and why it failed if it did.
pub fn set_up(ctx: &egui::Context) {
    if SETTING_UP.swap(true, Ordering::AcqRel) {
        return;
    }
    let ctx = ctx.clone();
    let started = std::thread::Builder::new()
        .name("set-up".into())
        .spawn(move || {
            let result = run_host("--register", SET_UP_LIMIT);
            registered_as(&result);
            if let Ok(mut last) = SET_UP_ERROR.lock() {
                *last = result.err();
            }
            SETTING_UP.store(false, Ordering::Release);
            SET_UP_DONE.fetch_add(1, Ordering::AcqRel);
            ctx.request_repaint();
        });
    if started.is_err() {
        SETTING_UP.store(false, Ordering::Release);
    }
}

/// Whether a Set it up finished since `seen` was last given here (it is then brought up to
/// date): the caller reads the checks again.
pub fn set_up_finished(seen: &mut u64) -> bool {
    let done = SET_UP_DONE.load(Ordering::Acquire);
    std::mem::replace(seen, done) != done
}

/// One line of Options → General → Test browser integration: what was checked, whether it
/// holds, and the help page's section when it doesn't.
pub struct Check {
    pub ok: bool,
    pub text: String,
    pub why: Option<&'static str>,
    /// A path the check names, on a line of its own under it: shortened in the middle when it
    /// doesn't fit, whole on hover.
    pub detail: Option<String>,
}

/// The checks in the order a connection needs them; the first failing one is what to fix.
pub fn checks(settings: &crate::settings::Settings) -> Vec<Check> {
    let check = |ok: bool, text: &str, why: Option<&'static str>| Check {
        ok,
        text: crate::i18n::t(text).to_owned(),
        why: (!ok).then_some(why).flatten(),
        detail: None,
    };
    let mut out = vec![
        check(
            settings.advanced_integration,
            // i18n: a check of Test browser integration (Options → General)
            "Advanced browser integration is on",
            Some("options"),
        ),
        check(
            mdm_ipc::portable_dir().is_none(),
            // i18n: a check of Test browser integration: a portable copy can't be connected
            "This is the installed copy, not a portable one",
            Some("not-installed"),
        ),
    ];
    if temporary_copy() {
        out.push(check(
            false,
            // i18n: a check of Test browser integration: Grabbit was started from inside a zip (or from the temporary folder), which is deleted after it quits, so it can't be connected to the browsers
            "This copy runs from a temporary folder that will be deleted",
            Some("not-installed"),
        ));
    }
    let registration = registration();
    out.push(match &registration {
        Registration::Ready | Registration::Missing => check(
            registration == Registration::Ready,
            // i18n: a check of Test browser integration: the browsers find MDM's helper program
            "The browsers can find Grabbit",
            Some("not-installed"),
        ),
        Registration::HelperMissing(path) => {
            tracing::warn!("browser integration: {} is missing", path.display());
            check(
                false,
                // i18n: a check of Test browser integration: the browsers are set up, but the small helper program they start (grabbit-nmhost) is gone
                "The helper program the browsers start is missing",
                Some("not-installed"),
            )
        }
        Registration::Gone(manifest) => {
            tracing::warn!("browser integration: {} is gone", manifest.display());
            // The copy's folder; the user's own manifest (in the data folder, no copy's) as it is.
            let folder = if mdm_ipc::browsers::is_user_manifest(manifest) {
                manifest
            } else {
                manifest.parent().unwrap_or(manifest)
            };
            Check {
                ok: false,
                // i18n: a check of Test browser integration: the browsers are pointed at a copy of Grabbit that was deleted (often one started from inside a zip); that copy's folder shows on the line under it
                text: crate::i18n::t("The browsers are set up for a copy that is gone:").to_owned(),
                why: Some("not-installed"),
                detail: Some(crate::i18n::ltr(&folder.display().to_string())),
            }
        }
    });
    if registration != Registration::Ready
        && let Some(error) = register_error()
    {
        out.push(Check {
            ok: false,
            text: crate::i18n::fmt(
                // i18n: a check of Test browser integration: setting it up failed; {error} is the helper program's own message, in English
                crate::i18n::t("Setting it up failed: {error}"),
                &[("error", &crate::i18n::ltr(&error))],
            ),
            why: Some("not-installed"),
            detail: None,
        });
    }
    let seen = mdm_ipc::extension::seen_browsers();
    if seen.is_empty() {
        out.push(check(
            false,
            // i18n: a check of Test browser integration: no browser's extension has reached MDM
            "No browser's extension has connected yet",
            None,
        ));
    }
    for (browser, version) in seen {
        let name = if browser.is_empty() {
            crate::i18n::t("Your browser").to_owned()
        } else {
            browser
        };
        let text = if version.is_empty() {
            crate::i18n::fmt(
                // i18n: a check of Test browser integration: {browser} is a browser's name
                crate::i18n::t("{browser}: the extension connected"),
                &[("browser", &name)],
            )
        } else {
            crate::i18n::fmt(
                // i18n: a check of Test browser integration: {browser} is a browser's name, {version} the extension's version number
                crate::i18n::t("{browser}: the extension {version} connected"),
                &[("browser", &name), ("version", &version)],
            )
        };
        out.push(Check {
            ok: true,
            text,
            why: None,
            detail: None,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Upstream issue 1218: a read slow to answer (a browser's folder on a sleeping disk) doesn't
    /// hold the caller: it starts at once, nothing is there while it runs, and its answer is
    /// taken once it comes.
    #[test]
    fn a_slow_read_is_made_beside_the_window() {
        let ctx = egui::Context::default();
        let (go, wait) = std::sync::mpsc::channel::<()>();
        let started = std::time::Instant::now();
        let reading = Reading::start(&ctx, move || {
            let _ = wait.recv_timeout(Duration::from_secs(10));
            7
        })
        .expect("a thread");
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(reading.take(), None, "nothing while it reads");
        go.send(()).unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(10);
        let answer = loop {
            if let Some(a) = reading.take() {
                break a;
            }
            assert!(std::time::Instant::now() < until, "the answer never came");
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(answer, Some(7));
        // A read that ends without an answer (its code panicked) is over too.
        let gone = Reading::<u8>::start(&ctx, || panic!("a read's code panics")).unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(10);
        while gone.take().is_none() {
            assert!(std::time::Instant::now() < until);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// #41: Helium says it is Chrome; Zen gives its own name; each row reads its own status.
    #[test]
    fn rows_match_the_names_browsers_report() {
        let seen = vec!["Google Chrome".to_owned(), "Zen Browser".to_owned()];
        assert!(is_seen(&seen, "Helium") && is_seen(&seen, "Google Chrome"));
        assert!(is_seen(&seen, "Zen Browser"));
        assert!(!is_seen(&seen, "Mozilla Firefox") && !is_seen(&seen, "Brave"));
        assert!(is_seen(&["Brave Browser".to_owned()], "Brave"));
    }

    /// Firefox's family gets the tutorial, whatever the store has.
    #[test]
    fn gecko_rows_are_firefoxs_family() {
        for b in [
            "Mozilla Firefox",
            "Zen Browser",
            "LibreWolf",
            "Floorp",
            "Waterfox",
        ] {
            assert!(is_gecko(b), "{b}");
        }
        for b in ["Google Chrome", "Helium", "Brave", "Your browser", ""] {
            assert!(!is_gecko(b), "{b}");
        }
    }

    /// Grabbit has no store listing: every row opens the tutorial at its browser's section
    /// (never the original project's listings); a listing, once there, is Chromium's only.
    #[test]
    fn rows_open_the_tutorial_until_listed() {
        let store = mdm_ipc::extension::store_page();
        assert_eq!(store, None);
        for b in ["Google Chrome", "Brave", "Microsoft Edge", ""] {
            assert_eq!(
                row_target(b, store.as_deref()),
                (None, mdm_ipc::extension::GUIDE_CHROMIUM),
                "{b}"
            );
        }
        assert_eq!(
            row_target("Mozilla Firefox", Some("https://store/x")),
            (None, mdm_ipc::extension::GUIDE_FIREFOX)
        );
        assert_eq!(
            row_target("Brave", Some("https://store/x")).0,
            Some("https://store/x")
        );
        assert!(
            crate::help::page_url(mdm_ipc::extension::GUIDE_CHROMIUM)
                .ends_with("/help/guide-catch-browser-downloads.html#chromium")
        );
    }
}
