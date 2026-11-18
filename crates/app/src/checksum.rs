//! Right-click → Verify checksum… (beyond IDM, D-015): paste the published hash; a finished file
//! is checked at once, an unfinished one when its download ends (the engine verifies it before
//! the final rename). The dialog is ours; it uses the light dialog style.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mdm_engine::verify::Hash;

use crate::dialogs::{Outcome, window};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, LIGHT_BG, LIGHT_TEXT, r, text};

/// A check of the file: its digest once read, or why it couldn't be.
type Check = Arc<Mutex<Option<Result<Vec<u8>, String>>>>;

pub struct ChecksumDialog {
    pub id: u64,
    text: String,
    file: Option<PathBuf>,
    check: Option<Check>,
    focused: bool,
    /// The window closed (a check still running tells its verdict through the app).
    gone: Arc<std::sync::atomic::AtomicBool>,
}

impl ChecksumDialog {
    /// `file`: the finished file (checked now); None for a download still to finish.
    pub fn new(id: u64, known: Option<&Hash>, file: Option<PathBuf>) -> Self {
        ChecksumDialog {
            id,
            text: known.map(Hash::display).unwrap_or_default(),
            file,
            check: None,
            focused: false,
            gone: Default::default(),
        }
    }

    /// The file's own checksum when the check found it differs (shown, to copy, under the
    /// verdict).
    fn mismatch(&self) -> Option<Hash> {
        let want = Hash::parse(&self.text)?;
        let check = self.check.as_ref()?.lock().unwrap();
        match &*check {
            Some(Ok(got)) if *got != want.value => Some(Hash {
                algo: want.algo,
                value: got.clone(),
            }),
            _ => None,
        }
    }

    /// `Done(hash)`: keep it with the download. Close and the window's × both keep a hash that
    /// was checked.
    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<Hash> {
        let id = format!("checksum-{}", self.id);
        let got = self.mismatch();
        // The mockup's layout: the file's card (a finished file), the box, the verdict in a
        // banner (or a line while there is none), the file's own checksum under a mismatch.
        const W: f32 = 520.0;
        let e = winui::EDGE;
        let y0 = if self.file.is_some() { 96.0 } else { 16.0 };
        let verdict = self
            .check
            .as_ref()
            .is_some_and(|c| c.lock().unwrap().is_some());
        let body = y0
            + 64.0
            + if verdict {
                winui::status_banner_height(0)
            } else {
                20.0
            }
            + if got.is_some() { 40.0 } else { 0.0 }
            + 16.0;
        let out = window(
            ctx,
            &id,
            "Verify checksum",
            [W, body + winui::FOOTER_H],
            false,
            LIGHT_BG(),
            |ui, o| {
                let p = ui.painter().clone();
                if let Some(file) = &self.file {
                    let name = file
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let sub = Hash::parse(&self.text)
                        .map(|h| {
                            crate::i18n::fmt(
                                // i18n: Verify checksum, the file card's line under its name: which checksum was given; {algorithm} = SHA-256, MD5…
                                crate::i18n::t("{algorithm} checksum"),
                                &[("algorithm", h.algo.name())],
                            )
                        })
                        .unwrap_or_default();
                    winui::file_card(
                        ui,
                        r(o, e, 16.0, W - 2.0 * e, 66.0),
                        crate::fileicons::Kind::File(&name),
                        &crate::i18n::isolate(&name),
                        &sub,
                        None,
                    );
                }
                text(
                    &p,
                    o.x + e,
                    o.y + y0 + 13.0,
                    crate::i18n::t("Expected checksum (MD5, SHA-1, SHA-256, SHA-512 or CRC32):"),
                    LIGHT_TEXT(),
                );
                let edit = winui::edit_light(
                    ui,
                    r(o, e, y0 + 20.0, W - 2.0 * e, winui::CONTROL_H),
                    &mut self.text,
                    true,
                    false,
                );
                if !self.focused {
                    edit.request_focus();
                    self.focused = true;
                }
                let hash = Hash::parse(&self.text);
                if edit.changed() {
                    self.check = None;
                }
                // The verdict: in a banner (info: it matches; error: it doesn't, or can't be
                // read), with its status icon.
                let banner = match (&hash, &self.check) {
                    (Some(h), Some(c)) => match &*c.lock().unwrap() {
                        Some(Ok(v)) if *v == h.value => Some((
                            winui::BannerKind::Info,
                            "st-complete",
                            crate::i18n::fmt(
                                crate::i18n::t("The file matches ({algorithm})"),
                                &[("algorithm", h.algo.name())],
                            ),
                        )),
                        Some(Ok(_)) => Some((
                            winui::BannerKind::Error,
                            "st-error",
                            crate::i18n::fmt(
                                crate::i18n::t("The file does NOT match ({algorithm})"),
                                &[("algorithm", h.algo.name())],
                            ),
                        )),
                        Some(Err(err)) => Some((
                            winui::BannerKind::Error,
                            "st-error",
                            crate::i18n::fmt(
                                crate::i18n::t("Cannot read the file: {error}"),
                                &[("error", &err.to_string())],
                            ),
                        )),
                        None => None,
                    },
                    _ => None,
                };
                let status_y = y0 + 64.0;
                if let Some((kind, icon, title)) = &banner {
                    let rc = r(o, e, status_y, W - 2.0 * e, winui::status_banner_height(0));
                    // A long reason is cut, and shown whole on hover by the banner itself.
                    winui::status_banner(ui, rc, *kind, icon, title, "");
                }
                let (line, color) = match (&hash, &self.check) {
                    _ if banner.is_some() => (String::new(), LIGHT_TEXT()),
                    (None, _) if !self.text.trim().is_empty() => (
                        crate::i18n::t("Not a checksum this can read").to_owned(),
                        winui::ERROR(),
                    ),
                    (None, _) => (String::new(), LIGHT_TEXT()),
                    (Some(_), Some(_)) => (
                        crate::i18n::t("Checking...").to_owned(),
                        winui::LIGHT_DISABLED(),
                    ),
                    (Some(h), None) if self.file.is_none() => (
                        crate::i18n::fmt(
                            crate::i18n::t("{algorithm}: checked when the download finishes"),
                            &[("algorithm", h.algo.name())],
                        ),
                        LIGHT_TEXT(),
                    ),
                    (Some(h), None) => (
                        crate::i18n::fmt(
                            crate::i18n::t("{algorithm} checksum"),
                            &[("algorithm", h.algo.name())],
                        ),
                        LIGHT_TEXT(),
                    ),
                };
                if !line.is_empty() {
                    winui::fit(
                        ui,
                        &p,
                        o.x + e,
                        o.y + status_y + 14.0,
                        &line,
                        color,
                        o.x + W - e,
                        "checksum-status",
                    );
                }
                if let Some(got) = &got {
                    winui::readonly_field(
                        ui,
                        r(
                            o,
                            e,
                            status_y + winui::status_banner_height(0) + 10.0,
                            W - 2.0 * e,
                            winui::CONTROL_H,
                        ),
                        &got.display(),
                        LIGHT_BG(),
                        true,
                        "checksum-got",
                    )
                    .on_hover_text(crate::i18n::t(
                        // i18n: tooltip of the box showing the checksum the file really has
                        "The file's own checksum",
                    ));
                }
                // Verify waits while a check runs.
                let checking = self
                    .check
                    .as_ref()
                    .is_some_and(|c| c.lock().unwrap().is_none());
                let valid = hash.is_some() && !checking;
                let kind = if valid {
                    ButtonKind::Default
                } else {
                    ButtonKind::Disabled
                };
                let (primary, secondary) = if self.file.is_some() {
                    // i18n: both verbs: Verify = check the file now; Close = close the window
                    (crate::i18n::mark("Verify"), crate::i18n::mark("Close"))
                } else {
                    (crate::i18n::mark("OK"), crate::i18n::mark("Cancel"))
                };
                // Enter verifies (while it can), Escape closes, as before.
                let pressed = winui::footer(
                    ui,
                    o,
                    W,
                    body,
                    &mut [
                        FooterItem::Button(
                            FooterButton::new(primary, kind).primary().enter(true).tip(
                                !valid,
                                if checking {
                                    // i18n: why Verify is greyed: the file is being checked
                                    "Checking the file..."
                                } else {
                                    // i18n: why Verify / OK is greyed in Verify checksum
                                    "Paste the checksum the download's page gives"
                                },
                            ),
                        ),
                        FooterItem::Button(
                            FooterButton::new(secondary, ButtonKind::Normal).escape(true),
                        ),
                    ],
                );
                let (go, close) = (pressed == Some(0), pressed == Some(1));
                if close {
                    // A checked file keeps its hash with the download.
                    match (hash, &self.check) {
                        (Some(h), Some(_)) => Outcome::Done(h),
                        _ => Outcome::Closed,
                    }
                } else if let (Some(h), true) = (hash, go && valid) {
                    match &self.file {
                        None => Outcome::Done(h),
                        Some(file) => {
                            let slot: Check = Arc::new(Mutex::new(None));
                            let (out, file, ctx) = (slot.clone(), file.clone(), ui.ctx().clone());
                            let gone = self.gone.clone();
                            std::thread::spawn(move || {
                                let r = mdm_engine::verify::digest(&file, h.algo)
                                    .map_err(|e| e.to_string());
                                // Closed while it checked: the verdict comes anyway (a large
                                // file's was lost).
                                if gone.load(std::sync::atomic::Ordering::Relaxed) {
                                    let name = crate::i18n::isolate(
                                        &file.file_name().unwrap_or_default().to_string_lossy(),
                                    );
                                    let said = match &r {
                                        Ok(v) if *v == h.value => crate::i18n::t(
                                            "{file}: the file matches its checksum ({algorithm})",
                                        ),
                                        Ok(_) => crate::i18n::t(
                                            "{file}: the file does NOT match its checksum ({algorithm})",
                                        ),
                                        Err(_) => {
                                            crate::i18n::t("{file} could not be read to check it")
                                        }
                                    };
                                    crate::dialogs::tell(
                                        crate::i18n::mark("Verify checksum"),
                                        crate::i18n::fmt(
                                            said,
                                            &[("file", &name), ("algorithm", h.algo.name())],
                                        ),
                                    );
                                }
                                *out.lock().unwrap() = Some(r);
                                ctx.request_repaint();
                            });
                            self.check = Some(slot);
                            Outcome::Open
                        }
                    }
                } else {
                    Outcome::Open
                }
            },
        );
        // Closed while a check runs: its thread tells the verdict when it comes.
        if !matches!(out, Outcome::Open)
            && self
                .check
                .as_ref()
                .is_some_and(|c| c.lock().unwrap().is_none())
        {
            self.gone.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // The window's ×: as Close.
        match (out, Hash::parse(&self.text), &self.check) {
            (Outcome::Closed, Some(h), Some(_)) => Outcome::Done(h),
            (out, ..) => out,
        }
    }
}
