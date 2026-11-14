//! Download a gallery (#58): a gallery's page (Wikimedia Commons, Flickr, DeviantArt…) given to
//! Add URL opens this window instead of downloading the page as a file. gallery-dl (an optional
//! helper, installed and verified as yt-dlp is, or the person's own) checks it can read the
//! address, offline, and downloads its files into a folder of their own; at most so many files
//! when asked. Started again, it fetches only the files not there yet.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{Pos2, Ui, pos2};

use crate::dialogs::{Outcome, window};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, r, text};

/// Download a gallery's width, and where its footer starts (the mockup's layout).
const GALLERY_W: f32 = 520.0;
const GALLERY_BODY: f32 = 232.0;
/// Its content's left and right edges.
const LX: f32 = winui::EDGE;
const RIGHT: f32 = GALLERY_W - winui::EDGE;

type Slot<T> = Arc<Mutex<Option<Result<T, String>>>>;

/// What Download Now or Later adds.
pub struct GalleryAdd {
    pub page: url::Url,
    /// The gallery's own folder (Save to, then its name).
    pub folder: PathBuf,
    /// Its name in the list.
    pub name: String,
    pub limit: Option<u64>,
    pub start_now: bool,
}

pub struct GalleryDialog {
    page: String,
    folder: String,
    limit: String,
    install: Option<Slot<mdm_tools::Installed>>,
    /// gallery-dl's answer for the address (`check`), and the address it was for.
    check: Option<Slot<bool>>,
    checked: Option<(String, bool)>,
    error: Option<String>,
    start_now: Option<bool>,
    asking: crate::dialogs::FolderGate<()>,
    helpers: Option<crate::helpers::Programs>,
    /// The helper as found (read again after an install or Helpers, not every frame).
    tool: Option<Option<mdm_tools::Installed>>,
}

fn tool() -> Option<mdm_tools::Installed> {
    mdm_tools::tools_dir()
        .as_deref()
        .and_then(mdm_tools::gallery::installed)
}

impl GalleryDialog {
    pub fn new(folder: PathBuf, page: &str) -> Self {
        GalleryDialog {
            page: page.to_owned(),
            folder: folder.display().to_string(),
            limit: String::new(),
            install: None,
            check: None,
            checked: None,
            error: None,
            start_now: None,
            asking: Default::default(),
            helpers: None,
            tool: None,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &tokio::runtime::Handle,
    ) -> Outcome<GalleryAdd> {
        let installed = self.install.as_ref().and_then(|s| s.lock().unwrap().take());
        if let Some(res) = installed {
            self.install = None;
            self.tool = None;
            if let Err(e) = res {
                self.error = Some(crate::i18n::fmt(
                    crate::i18n::t("{what} could not be installed: {error}"),
                    &[("what", "gallery-dl"), ("error", &e)],
                ));
            }
        }
        let answered = self.check.as_ref().and_then(|s| s.lock().unwrap().take());
        if let Some(res) = answered {
            self.check = None;
            match res {
                Ok(ok) => self.checked = Some((self.page.trim().to_owned(), ok)),
                Err(e) => self.error = Some(e),
            }
        }
        let out = window(
            ctx,
            "gallery-page",
            // i18n: window title: download an image gallery's files (gallery-dl)
            "Download a gallery",
            [GALLERY_W, GALLERY_BODY + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| self.body(ui, o, rt),
        );
        if let Some(h) = &mut self.helpers
            && !matches!(h.show(ctx), Outcome::Open)
        {
            self.helpers = None;
            self.tool = None;
            self.checked = None;
        }
        let folder = crate::dialogs::resolve_path(&self.folder);
        match crate::dialogs::ensure_folder(ctx, &mut self.asking, out, &folder) {
            Outcome::Open => Outcome::Open,
            Outcome::Closed => Outcome::Closed,
            Outcome::Done(()) => {
                let Ok(page) = url::Url::parse(self.page.trim()) else {
                    return Outcome::Closed;
                };
                let name = mdm_tools::gallery::name_of(&page);
                Outcome::Done(GalleryAdd {
                    folder: folder.join(&name),
                    name,
                    page,
                    limit: self.limit.trim().parse().ok().filter(|n| *n > 0),
                    start_now: self.start_now.unwrap_or(true),
                })
            }
        }
    }

    fn body(&mut self, ui: &mut Ui, o: Pos2, rt: &tokio::runtime::Handle) -> Outcome<()> {
        let p = ui.painter().clone();
        let tool = self.tool.get_or_insert_with(tool).clone();
        let h = winui::CONTROL_H;
        let fx = winui::form(
            &p,
            LX,
            &[
                "Page address",
                "Save to",
                // i18n: label before the box for how many of a gallery's files to download at most
                "Files at most",
            ],
            false,
        );
        winui::form_label(&p, o, LX, 16.0, h, "Page address", true);
        winui::edit(
            ui,
            r(o, fx, 16.0, RIGHT - fx, h),
            &mut self.page,
            true,
            false,
        );
        let page = self.page.trim().to_owned();
        let page_ok = url::Url::parse(&page).is_ok_and(|u| matches!(u.scheme(), "http" | "https"));
        // The address checked as soon as gallery-dl is there, and again when it changes.
        if let Some(t) = &tool
            && page_ok
            && self.check.is_none()
            && self.checked.as_ref().is_none_or(|(p, _)| *p != page)
        {
            let slot: Slot<bool> = Arc::new(Mutex::new(None));
            let (out, tool, url, ctx) = (slot.clone(), t.clone(), page.clone(), ui.ctx().clone());
            std::thread::spawn(move || {
                let res = mdm_tools::gallery::handles(&tool, &url).map_err(|e| e.to_string());
                *out.lock().unwrap() = Some(res);
                ctx.request_repaint();
            });
            self.check = Some(slot);
            self.error = None;
        }

        // The status: gallery-dl missing or installing / checking (a line), what it said (a
        // banner), in the band under the address.
        let band = (60.0, winui::status_banner_height(0));
        let line_y = o.y + winui::baseline_in(band.0, band.1);
        let can_install = mdm_tools::gallery::asset().is_some();
        let readable = self
            .checked
            .as_ref()
            .is_some_and(|(p, ok)| *p == page && *ok);
        let mut status_right = RIGHT;
        if self.install.is_some() {
            crate::torrentdlg::spinner(ui, pos2(o.x + LX + 7.0, line_y - 4.0), 6.0);
            let msg = crate::i18n::t("Downloading and verifying gallery-dl...");
            text(&p, o.x + LX + 20.0, line_y, msg, winui::TEXT());
        } else if tool.is_none() {
            // i18n: in Download a gallery, before gallery-dl is installed
            let msg = crate::i18n::t(
                "Galleries use gallery-dl (about 23 MB, checked against its signature).",
            );
            // The Install button, as wide as its label in this language (#192), and the line
            // up to 9 px before it.
            let install_at = can_install.then(|| {
                let w = winui::footer_button_width(&p, "Install gallery-dl", false);
                (RIGHT - w, w)
            });
            if let Some((x, _)) = install_at {
                status_right = x - 9.0;
            }
            let rc = winui::fit(
                ui,
                &p,
                o.x + LX,
                line_y,
                msg,
                winui::MUTED(),
                o.x + status_right,
                "gallery-offer",
            );
            winui::readable(ui, rc, "gallery-status", msg);
            if let Some((x, w)) = install_at
                && winui::button(
                    ui,
                    r(o, x, band.0 + ((band.1 - h) / 2.0).round(), w, h),
                    // i18n: button: download and install the gallery-dl program
                    "Install gallery-dl",
                    ButtonKind::Normal,
                    false,
                    None,
                )
                .clicked()
                && let Some(dir) = mdm_tools::tools_dir()
            {
                let slot: Slot<mdm_tools::Installed> = Arc::new(Mutex::new(None));
                let (out, ctx) = (slot.clone(), ui.ctx().clone());
                let net = crate::settings::own_net();
                rt.spawn(async move {
                    let res = mdm_tools::gallery::install_latest(&dir, &net, None).await;
                    // What failed and what to do (upstream issue 816).
                    *out.lock().unwrap() =
                        Some(res.map_err(|e| crate::errors::tools_and_advice(&e, "gallery-dl")));
                    ctx.request_repaint();
                });
                self.install = Some(slot);
                self.error = None;
            }
        } else if self.check.is_some() {
            crate::torrentdlg::spinner(ui, pos2(o.x + LX + 7.0, line_y - 4.0), 6.0);
            // i18n: in Download a gallery, while gallery-dl checks the address (offline)
            let msg = crate::i18n::t("Checking the address with gallery-dl...");
            text(&p, o.x + LX + 20.0, line_y, msg, winui::TEXT());
        } else if let Some((_, ok)) = self.checked.as_ref().filter(|(p, _)| *p == page) {
            let (msg, kind, icon) = if *ok {
                // i18n: in Download a gallery: gallery-dl can read the address
                (
                    crate::i18n::t("gallery-dl can download this gallery's files."),
                    winui::BannerKind::Info,
                    "st-complete",
                )
            } else {
                // i18n: in Download a gallery: gallery-dl has no way to read the address
                (
                    crate::i18n::t(
                        "gallery-dl can't read this address. Add it as a file instead (Add URL).",
                    ),
                    winui::BannerKind::Error,
                    "st-error",
                )
            };
            // The answer in a banner (the mockup's), read out as before.
            let rc = r(o, LX, band.0, RIGHT - LX, band.1);
            winui::status_banner(ui, rc, kind, icon, msg, "");
            winui::readable(ui, rc, "gallery-status", msg);
        }
        if let Some(e) = &self.error {
            let rc = winui::fit(
                ui,
                &p,
                o.x + LX,
                o.y + 112.0,
                e,
                winui::ERROR(),
                o.x + RIGHT,
                "gallery-error",
            );
            winui::readable(ui, rc, "gallery-error", e);
        }

        // Save to, and how many files at most, on the labels' column.
        let (save_y, limit_y) = (124.0, 164.0);
        winui::form_label(&p, o, LX, save_y, h, "Save to", true);
        winui::edit(
            ui,
            r(o, fx, save_y, RIGHT - h - 8.0 - fx, h),
            &mut self.folder,
            true,
            false,
        );
        // i18n: tooltip of the "..." after Save to: choose the folder
        if winui::icon_only_button(ui, r(o, RIGHT - h, save_y, h, h), "gx-dots", "Browse", true)
            .clicked()
        {
            let from = self.folder.clone();
            winui::start_pick(ui.ctx(), "gallery-folder", move || {
                rfd::FileDialog::new().set_directory(from).pick_folder()
            });
        }
        if let Some(d) = winui::picked(ui.ctx(), "gallery-folder") {
            self.folder = d.display().to_string();
        }
        self.limit.retain(|c| c.is_ascii_digit());
        winui::form_label(&p, o, LX, limit_y, h, "Files at most", true);
        winui::edit(
            ui,
            r(o, fx, limit_y, 110.0, h),
            &mut self.limit,
            true,
            false,
        );
        // The hint on its own line under the box (the mockup's).
        winui::fit(
            ui,
            &p,
            o.x + fx,
            o.y + 212.0,
            // i18n: in Download a gallery, after the "Files at most" box
            crate::i18n::t("Empty: all of them. On most sites the newest come first."),
            winui::MUTED(),
            o.x + RIGHT,
            "gallery-limit-hint",
        );

        let ready = readable && self.install.is_none() && self.check.is_none();
        let kind = |default: bool| match (ready, default) {
            (false, _) => ButtonKind::Disabled,
            (true, true) => ButtonKind::Default,
            (true, false) => ButtonKind::Normal,
        };
        // The footer: Helpers at its start; Enter Download Now (while it can), Escape Cancel.
        let pressed = winui::footer(
            ui,
            o,
            GALLERY_W,
            GALLERY_BODY,
            &mut [
                FooterItem::Button(FooterButton::new("Helpers...", ButtonKind::Normal).leading()),
                FooterItem::Button(
                    FooterButton::new("Download Now", kind(true))
                        .primary()
                        .enter(ready),
                ),
                FooterItem::Button(FooterButton::new("Later", kind(false))),
                FooterItem::Button(FooterButton::new("Cancel", ButtonKind::Normal).escape(true)),
            ],
        );
        match pressed {
            Some(0) => {
                if self.helpers.is_none() {
                    self.helpers = Some(crate::helpers::Programs::new());
                }
                Outcome::Open
            }
            Some(3) => Outcome::Closed,
            Some(k) if ready => {
                self.start_now = Some(k == 1);
                Outcome::Done(())
            }
            _ => Outcome::Open,
        }
    }
}
