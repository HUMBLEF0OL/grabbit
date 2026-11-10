//! The remote control's side in the app (#141; the server is crates/remote): starting it as
//! Options says, the page's texts, and the torrents a paired device sends while their files are
//! looked up.

use std::sync::{Arc, Mutex};

use crate::model::Category;
use crate::settings::Settings;

/// The remote control for `settings`, started (or not) as they say. None without a data folder.
pub fn start(
    rt: &tokio::runtime::Handle,
    sink: mdm_ipc::Sink,
    settings: &Settings,
) -> Option<mdm_remote::Remote> {
    let dir = mdm_ipc::data_dir()?.join("remote");
    let texts: mdm_remote::Texts = Arc::new(|_: &str| page_texts());
    let remote = mdm_remote::Remote::new(rt.clone(), dir, sink, texts);
    remote.apply(config(settings));
    Some(remote)
}

/// The page's address as people open it (`MDM_REMOTE_URL`, #157): a container's own address
/// isn't the server's.
pub fn public_url() -> Option<String> {
    let url = std::env::var("MDM_REMOTE_URL").ok()?;
    let url = url.trim();
    url.starts_with("https://").then(|| url.to_owned())
}

/// The names `--hosts` gave a headless run (#157), over `MDM_REMOTE_HOSTS`.
static HOSTS: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Use `hosts` (names separated by commas) as [`config`]'s extra names.
pub fn set_hosts(hosts: Option<&str>) {
    if let Some(h) = hosts {
        let _ = HOSTS.set(h.to_owned());
    }
}

/// Where and whether to listen, from `settings`; `MDM_REMOTE_HOSTS` (names separated by commas)
/// adds the names a request may use besides the addresses (a server's, a reverse proxy's).
pub fn config(settings: &Settings) -> mdm_remote::Config {
    let hosts = HOSTS
        .get()
        .cloned()
        .or_else(|| std::env::var("MDM_REMOTE_HOSTS").ok())
        .unwrap_or_default()
        .split(',')
        .map(|h| h.trim().to_ascii_lowercase())
        .filter(|h| !h.is_empty())
        .collect();
    mdm_remote::Config {
        on: settings.remote_on,
        port: settings.remote_port,
        listen: settings.remote_listen.clone(),
        hosts,
    }
}

/// The page's texts: English, as the page writes them (no translations to send).
// ponytail: /api/texts stays so the page's protocol is unchanged; drop it with crates/remote/web's T().
pub fn page_texts() -> mdm_remote::PageTexts {
    mdm_remote::PageTexts {
        lang: "en".to_owned(),
        rtl: false,
        texts: Default::default(),
    }
}

/// A QR code of `text` (black on white, whatever the theme: phones' cameras read it so) in the
/// middle of `rect`, with its quiet zone, in whole pixels.
pub fn paint_qr(p: &egui::Painter, rect: egui::Rect, text: &str) {
    use egui::{Color32, Rect, vec2};
    let Ok(qr) = qrcodegen::QrCode::encode_text(text, qrcodegen::QrCodeEcc::Medium) else {
        return;
    };
    let n = qr.size();
    let quiet = 3;
    let cells = (n + 2 * quiet) as f32;
    let cell = (rect.width().min(rect.height()) / cells).floor().max(1.0);
    let side = cell * cells;
    let origin = (rect.center() - vec2(side / 2.0, side / 2.0)).round();
    p.rect_filled(
        Rect::from_min_size(origin, vec2(side, side)),
        0.0,
        Color32::WHITE,
    );
    for y in 0..n {
        for x in 0..n {
            if qr.get_module(x, y) {
                let at = origin + vec2((x + quiet) as f32 * cell, (y + quiet) as f32 * cell);
                p.rect_filled(
                    Rect::from_min_size(at, vec2(cell, cell)),
                    0.0,
                    Color32::BLACK,
                );
            }
        }
    }
}

/// A QR code of `text` for a terminal: two rows of modules per line, with half blocks. Light
/// modules are drawn (full blocks) and dark ones left as the terminal's background, so it reads
/// on a dark terminal, the usual one; its quiet zone is light.
pub fn qr_text(text: &str) -> String {
    let Ok(qr) = qrcodegen::QrCode::encode_text(text, qrcodegen::QrCodeEcc::Low) else {
        return String::new();
    };
    let (n, quiet) = (qr.size(), 2);
    let dark = |x: i32, y: i32| x >= 0 && y >= 0 && x < n && y < n && qr.get_module(x, y);
    let mut out = String::new();
    let mut y = -quiet;
    while y < n + quiet {
        for x in -quiet..n + quiet {
            out.push(match (dark(x, y), dark(x, y + 1)) {
                (false, false) => '\u{2588}',
                (false, true) => '\u{2580}',
                (true, false) => '\u{2584}',
                (true, true) => ' ',
            });
        }
        out.push('\n');
        y += 2;
    }
    out
}

/// A torrent's files being looked up: filled in on the runtime when they are known.
pub type Slot = Arc<Mutex<Option<Result<mdm_torrent::Meta, String>>>>;

/// A torrent a paired device sent: its files looked up on the runtime, then added as Add
/// torrent would with everything chosen.
pub struct Lookup {
    pub slot: Slot,
    pub category: Option<Category>,
    /// The queue it waits in (by index), or None to start now.
    pub queue: Option<usize>,
    pub from_magnet: bool,
}
