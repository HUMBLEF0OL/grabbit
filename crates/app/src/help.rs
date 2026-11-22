//! The Help menu (D-020): the user guide and the home page on the website (docs/ made into
//! pages, in the interface language), problem reports on GitHub (pre-filled), update checks
//! against GitHub Releases (on demand, and at every start and once a day while running), About,
//! Keyboard shortcuts and Tell a friend.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use egui::{pos2, vec2};

use crate::dialogs::{Outcome, window};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, r};

/// The repository: problem reports, releases, source, licences. This fork's own: its users'
/// reports and its builds' updates come here, not to the original (anefzaoui/misterdownloadmanager).
pub const REPO: &str = "https://github.com/HUMBLEF0OL/grabbit";
/// Where Help → Support MDM and About's last line go: this fork's repository until its owner's
/// GitHub Sponsors page exists (then https://github.com/sponsors/HUMBLEF0OL, with the website's,
/// the extension popup's and .github/FUNDING.yml's).
pub const SPONSORS: &str = REPO;
/// This fork's releases (none yet: GitHub's 404 reads as up to date).
const LATEST: &str = "https://api.github.com/repos/HUMBLEF0OL/grabbit/releases/latest";
/// This fork's repository on GitHub (`owner/name`): its release pages when GitHub's API refuses
/// for its rate limit (#1337).
const REPO_PATH: &str = "HUMBLEF0OL/grabbit";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT: &str = env!("MDM_COMMIT");
/// While MDM runs, the automatic check comes again this often (it also checks at every start).
pub const CHECK_EVERY: i64 = 24 * 3600;
/// The check at start waits this long (the window, the list and the downloads first).
pub const CHECK_AFTER_START: std::time::Duration = std::time::Duration::from_secs(5);

/// The official website: the home page and the user guide, docs/*.md made into pages by
/// tools/website/build.py. GitHub (REPO) keeps what is GitHub's: issues, releases, the source.
pub const SITE: &str = "https://humblef0ol.github.io/grabbit";

/// The website's address of a user guide page: `doc` is the page's file in docs/ with an optional section (`"video.md"`,
/// `"README.md#tutorials"`; the guide's contents are README.md), `""` the home page. The mapping
/// is build.py's: docs/README.md is /help/, docs/<page>.md /help/<page>.html.
pub fn page_url(doc: &str) -> String {
    let (file, anchor) = match doc.split_once('#') {
        Some((file, anchor)) => (file, format!("#{anchor}")),
        None => (doc, String::new()),
    };
    let path = match file {
        "" => "/".to_owned(),
        "README.md" => "/help/".to_owned(),
        page => format!("/help/{}.html", page.strip_suffix(".md").unwrap_or(page)),
    };
    format!("{SITE}{path}{anchor}")
}

/// Open a page of the user guide on the website (`doc` as page_url takes it: `"video.md"`, `"README.md#tutorials"`, `""` the home page).
/// Never in an off-screen test run (MDM_OFFSCREEN), whose clicks come from scripts, not from a
/// person at the browser.
pub fn open_page(doc: &str) {
    if !crate::offscreen() {
        crate::launch::open(page_url(doc));
    }
}

/// Help → Support MDM and About's link: the Sponsors page in the browser. Asked for only: MDM
/// never brings it up by itself, and never in an off-screen test run (MDM_OFFSCREEN), whose
/// clicks come from scripts, not from a person at the browser.
pub fn support() {
    if !crate::offscreen() {
        crate::launch::open(SPONSORS);
    }
}

/// A new GitHub issue on the Bug report form (.github/ISSUE_TEMPLATE/bug.yml), filled with
/// what MDM knows: its version, the system and its version, how it was installed, the
/// language. The parameters are the form's field ids (all text fields: GitHub's page drops a
/// dropdown's value from the address).
pub fn report_problem() {
    // On a thread of its own: the fields ask the system, which the window's thread doesn't wait
    // for (#222).
    let _ = std::thread::Builder::new()
        .name("report-problem".into())
        .spawn(|| {
            let enc =
                |s: &str| url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>();
            let url = report_fields().iter().fold(
                format!("{REPO}/issues/new?template=bug.yml"),
                |url, (k, v)| format!("{url}&{k}={}", enc(v)),
            );
            let _ = open::that_detached(url);
        });
}

/// The Bug report form's fields MDM fills in: (field id, value).
fn report_fields() -> Vec<(&'static str, String)> {
    let os_version = crate::win32::windows_version();
    let os = if os_version.contains("Windows 10") {
        "Windows 10"
    } else {
        "Windows 11"
    };
    let mut fields = vec![
        ("os", os.to_owned()),
        ("os-version", os_version),
        ("version", format!("{VERSION} ({COMMIT})")),
    ];
    fields.push((
        "install",
        mdm_tools::update::Install::detect().label().to_owned(),
    ));
    fields
}

// ---------------------------------------------------------------------------------------------
// Updates

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    pub notes: String,
    /// The release's page (its files are there too).
    pub page: String,
    /// Its files: (name, download address); Update now takes this copy's from them.
    pub assets: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Found {
    Latest,
    Newer(Release),
}

pub type Slot = Arc<Mutex<Option<Result<Found, String>>>>;

/// Ask GitHub for the latest release in the background; the answer lands in the slot. Whatever
/// goes wrong, even a panic in the check, lands there as a failed check (#1123): the slot never
/// stays empty, which would leave Check for updates waiting for good.
pub fn check(rt: &tokio::runtime::Handle, ctx: &egui::Context) -> Slot {
    let slot: Slot = Arc::new(Mutex::new(None));
    let (out, ctx) = (slot.clone(), ctx.clone());
    let task = rt.spawn(latest());
    rt.spawn(async move {
        let res = settled(task.await);
        *out.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(res);
        ctx.request_repaint();
    });
    slot
}

/// The check's task ended: its answer, or a failed check when it panicked (or was cancelled).
fn settled(joined: Result<Result<Found, String>, tokio::task::JoinError>) -> Result<Found, String> {
    joined.unwrap_or_else(|e| {
        tracing::warn!("the update check stopped: {e}");
        Err(crate::i18n::t("Grabbit hit an internal error").to_owned())
    })
}

/// Where the check reads: GitHub's API (`feed`), else, when it refuses for its rate limit
/// (#1337), github.com's release pages (`web`) and the repository's files at the release's tag
/// (`raw`).
struct Sources {
    feed: String,
    web: String,
    raw: String,
}

async fn latest() -> Result<Found, String> {
    // MDM_UPDATE_FEED: another address for the latest release (tests of Update now, as
    // GRABBIT_DATA_DIR is for a separate list); MDM_UPDATE_WEB and MDM_UPDATE_RAW: other
    // addresses for GitHub's pages and files (tests of the way around the API's rate limit: the
    // lab server's fake GitHub, `/gh/web` and `/gh/raw`).
    let var = |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.into());
    let sources = Sources {
        feed: var("MDM_UPDATE_FEED", LATEST),
        web: var("MDM_UPDATE_WEB", mdm_tools::github::WEB),
        raw: var("MDM_UPDATE_RAW", mdm_tools::github::RAW),
    };
    let net = crate::settings::own_net();
    // The chosen DNS and network adapter, as downloads have them (#36, #35).
    let builder = |url: &str| {
        mdm_engine::net::client_builder(&net.dns, &net.route, url)
            .map(|b| {
                b.user_agent(concat!("Grabbit/", env!("CARGO_PKG_VERSION")))
                    .timeout(Duration::from_secs(20))
            })
            .map_err(|e| e.to_string())
    };
    let client = builder(&sources.feed)?.build().map_err(|e| e.to_string())?;
    // GitHub's pages are read without following their redirects: where `releases/latest` sends
    // Grabbit is the answer (mdm_tools::github::latest_tag).
    let pages = |url: &str| {
        builder(url)?
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| e.to_string())
    };
    let install = mdm_tools::update::Install::detect();
    fetch(&client, &pages, &sources, VERSION, install).await
}

/// The latest release against the running `current` version: from GitHub's API at
/// `sources.feed` (`client`), else, when it refuses for its rate limit, from its pages with
/// `pages`' clients (one for each server, following no redirect), the files this copy updates
/// from (`install`) as the release has them. When the pages fail too, the check fails as GitHub
/// refusing too many requests from this network.
async fn fetch(
    client: &reqwest::Client,
    pages: &(dyn Fn(&str) -> Result<reqwest::Client, String> + Sync),
    sources: &Sources,
    current: &str,
    install: mdm_tools::update::Install,
) -> Result<Found, String> {
    use mdm_tools::github::Api;
    let body = match mdm_tools::github::api_get(client, &sources.feed).await {
        Ok(Api::Answer(body)) => body,
        Ok(Api::RateLimited(refusal)) => {
            return match from_pages(pages, sources, current, install).await {
                Ok(found) => Ok(found),
                Err(e) => {
                    tracing::warn!("the update check on GitHub's pages failed too: {e}");
                    // Worded as the helpers' downloads word it: "api.github.com refused the
                    // request (403 Forbidden): too many requests from your network for now".
                    Err(crate::errors::tools(&refusal))
                }
            };
        }
        // No release published yet: nothing newer than this.
        Err(mdm_tools::Error::Http(e)) if e.status() == Some(reqwest::StatusCode::NOT_FOUND) => {
            return Ok(Found::Latest);
        }
        Err(mdm_tools::Error::Http(e)) => return Err(e.to_string()),
        Err(e) => return Err(e.to_string()),
    };
    parse_release(&String::from_utf8_lossy(&body), current)
}

/// The latest release from GitHub's pages, without its API (#1337): its tag from where
/// `releases/latest` redirects, its date and notes from its section of CHANGELOG.md at that tag
/// (publish.ps1's source for them; none when it can't be read), and of its files the signed
/// checksum list and the ones this copy updates from (`install`), those the release has (asked
/// without downloading them). Update now checks them as it checks the API's.
async fn from_pages(
    pages: &(dyn Fn(&str) -> Result<reqwest::Client, String> + Sync),
    sources: &Sources,
    current: &str,
    install: mdm_tools::update::Install,
) -> Result<Found, String> {
    use mdm_tools::github;
    let client = pages(&github::latest_page(&sources.web, REPO_PATH))?;
    let tag = github::latest_tag(&client, &sources.web, REPO_PATH)
        .await
        .map_err(|e| e.to_string())?;
    tracing::info!("the latest release is {tag}, from GitHub's pages");
    let version = version_of(&tag)?;
    if !newer(version, current) {
        return Ok(Found::Latest);
    }
    let log = github::file_at(&sources.raw, REPO_PATH, &tag, "CHANGELOG.md");
    let (date, notes) = match changelog(pages, &log).await {
        Ok(text) => changelog_section(&text, version).unwrap_or_default(),
        Err(e) => {
            tracing::warn!("{log}: {e}");
            Default::default()
        }
    };
    let mut names = vec!["SHA256SUMS.txt".to_owned(), "SHA256SUMS.txt.sig".to_owned()];
    names.extend(install.assets(version));
    let mut assets = Vec::new();
    for name in names {
        let url = github::download_url(&sources.web, REPO_PATH, &tag, &name);
        let there = client.head(&url).send().await.is_ok_and(|r| {
            let s = r.status();
            s.is_success() || s.is_redirection()
        });
        if there {
            assets.push((name, url));
        }
    }
    Ok(Found::Newer(Release {
        version: version.to_owned(),
        date,
        notes,
        page: github::release_page(&sources.web, REPO_PATH, &tag),
        assets,
    }))
}

/// The text at `url` (CHANGELOG.md at a tag).
async fn changelog(
    pages: &(dyn Fn(&str) -> Result<reqwest::Client, String> + Sync),
    url: &str,
) -> Result<String, String> {
    pages(url)?
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

/// `version`'s date and notes in CHANGELOG.md: its section (`## [X.Y.Z] - YYYY-MM-DD` up to the
/// next `## `), as publish.ps1 makes a release's notes from it.
fn changelog_section(text: &str, version: &str) -> Option<(String, String)> {
    let head = format!("## [{version}]");
    let mut lines = text.lines();
    let title = lines.by_ref().find(|l| l.starts_with(&head))?;
    let date = title[head.len()..]
        .trim_start_matches([' ', '-', '–'])
        .chars()
        .take(10)
        .collect();
    let notes: Vec<&str> = lines.take_while(|l| !l.starts_with("## ")).collect();
    Some((date, notes.join("\n").trim().to_owned()))
}

/// The version a release's tag names (`v0.5.2` → `0.5.2`), or why there is none.
fn version_of(tag: &str) -> Result<&str, String> {
    let version = tag.trim().trim_start_matches(['v', 'V']);
    // A number MDM can't read ("next", "1.x") would compare as 0 and pass for older: no answer
    // is better than a wrong one.
    let numeric = version
        .split_once('-')
        .map_or(version, |(main, _)| main)
        .split('.')
        .all(|p| p.parse::<u64>().is_ok());
    if version.is_empty() || !numeric {
        // i18n: why updates could not be checked: the newest release on the site has no number
        return Err(crate::i18n::t("the release has no version").to_owned());
    }
    Ok(version)
}

/// GitHub's latest-release JSON against the running `current` version.
fn parse_release(json: &str, current: &str) -> Result<Found, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let version = version_of(v["tag_name"].as_str().unwrap_or(""))?;
    if !newer(version, current) {
        return Ok(Found::Latest);
    }
    let s = |k: &str| v[k].as_str().unwrap_or("").to_owned();
    Ok(Found::Newer(Release {
        version: version.to_owned(),
        date: s("published_at").chars().take(10).collect(),
        notes: s("body"),
        page: s("html_url"),
        assets: v["assets"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| {
                        Some((
                            x["name"].as_str()?.to_owned(),
                            x["browser_download_url"].as_str()?.to_owned(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }))
}

/// `a` is a later version than `b` (numeric parts; a pre-release `-…` counts before its
/// release).
pub fn newer(a: &str, b: &str) -> bool {
    fn key(v: &str) -> (Vec<u64>, bool) {
        let (main, pre) = v.split_once('-').map_or((v, false), |(m, _)| (m, true));
        (
            main.split('.')
                .map(|p| p.trim().parse().unwrap_or(0))
                .collect(),
            !pre,
        )
    }
    let (mut ka, mut kb) = (key(a), key(b));
    let n = ka.0.len().max(kb.0.len());
    ka.0.resize(n, 0);
    kb.0.resize(n, 0);
    ka > kb
}

/// Help → Check for updates... (and the automatic check when it finds one).
pub struct Updates {
    slot: Option<Slot>,
    result: Option<Result<Found, String>>,
    /// Update now: the release file on its way (crate::update).
    job: Option<crate::update::Job>,
    /// What Update now came to, when it didn't close MDM.
    status: Option<UpdateStatus>,
    /// Update now pressed by a test (`/show update-now`, feature `test-update`).
    press: bool,
}

#[derive(Clone, Debug)]
enum UpdateStatus {
    Failed(String),
}

impl Updates {
    /// Update now, as if pressed (`/show update-now`: Update now's end-to-end tests).
    #[cfg(feature = "test-update")]
    pub fn press_update_now(&mut self) {
        self.press = true;
    }

    pub fn check(rt: &tokio::runtime::Handle, ctx: &egui::Context) -> Self {
        Updates {
            slot: Some(check(rt, ctx)),
            result: None,
            job: None,
            status: None,
            press: false,
        }
    }

    /// The automatic check found `release`.
    pub fn found(release: Release) -> Self {
        Updates {
            slot: None,
            result: Some(Ok(Found::Newer(release))),
            job: None,
            status: None,
            press: false,
        }
    }

    /// `weekly`: the automatic check (its switch is here, under the answer). `Done(true)`:
    /// the update is going in place, MDM closes for it (and starts again).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        rt: &tokio::runtime::Handle,
        weekly: &mut bool,
    ) -> Outcome<bool> {
        if let Some(r) = self.slot.as_ref().and_then(|s| s.lock().unwrap().take()) {
            self.result = Some(r);
            self.slot = None;
        }
        let install = mdm_tools::update::Install::detect();
        // The release file arrived (checked) and is in place (on the job's thread): MDM closes
        // once the new copy (or the installer) has started.
        if let Some(res) = self.job.as_ref().and_then(crate::update::Job::result) {
            self.job = None;
            match res {
                Ok(crate::update::Next::Restart) => return Outcome::Done(true),
                Err(e) => self.status = Some(UpdateStatus::Failed(e)),
            }
        }
        if self.job.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        let result = self.result.clone();
        // Compact unless there are release notes to show (then a window of its own size).
        let notes = matches!(result, Some(Ok(Found::Newer(_))));
        // One window that grows when notes arrive (a second id made a new window flash up).
        // The mockup's layout: an icon tile, the answer beside it, the notes, the status row, the
        // footer.
        const W: f32 = 480.0;
        let (id, body) = ("updates", if notes { 277.0 } else { 105.0 });
        let bottom = body - 16.0 - 21.0;
        window(
            ctx,
            id,
            "Check for updates",
            [W, body + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| {
                let p = ui.painter().clone();
                let ink = winui::TEXT();
                let e = winui::EDGE;
                let right = o.x + W - e;
                let tx = o.x + e + 40.0 + 12.0;
                winui::icon_tile(
                    ui,
                    r(o, e, 16.0, 40.0, 40.0),
                    "gx-download",
                    crate::theme::colors().accent,
                    8.0,
                );
                let mut download = None;
                match &result {
                    None => {
                        winui::text(
                            &p,
                            tx,
                            o.y + 31.0,
                            crate::i18n::t("Checking for updates..."),
                            ink,
                        );
                    }
                    Some(Ok(Found::Latest)) => {
                        let line = crate::i18n::fmt(
                            crate::i18n::t("You have the latest version of Grabbit ({version})."),
                            &[("version", VERSION)],
                        );
                        winui::paragraph(&p, tx, o.y + 31.0, &line, ink, right, 13.0);
                    }
                    Some(Err(e)) => {
                        let line = crate::i18n::fmt(
                            crate::i18n::t("Updates could not be checked: {error}"),
                            &[("error", e)],
                        );
                        let (rect, cut) = winui::text_fit(&p, tx, o.y + 31.0, &line, ink, right);
                        if cut {
                            winui::full_on_hover(ui, rect, "update-error", &line);
                        }
                    }
                    Some(Ok(Found::Newer(rel))) => {
                        let line = crate::i18n::fmt(
                            crate::i18n::t("Grabbit {version} is available (released {date})."),
                            &[("version", &rel.version), ("date", &rel.date)],
                        );
                        winui::fit(ui, &p, tx, o.y + 31.0, &line, ink, right, "update-line");
                        let have = crate::i18n::fmt(
                            crate::i18n::t("You have version {version}."),
                            &[("version", VERSION)],
                        );
                        winui::fit(
                            ui,
                            &p,
                            tx,
                            o.y + 49.0,
                            &have,
                            winui::MUTED(),
                            right,
                            "update-have",
                        );
                        winui::text(&p, o.x + e, o.y + 80.0, crate::i18n::t("What's new:"), ink);
                        let notes = egui::Rect::from_min_max(
                            pos2(o.x + e, o.y + 88.0),
                            pos2(right, o.y + 228.0),
                        );
                        // English, as the paragraph is laid out; made where it is on screen.
                        // Read-only text: the read-only field's look (it drew a square box of its own).
                        winui::field_frame(&p, winui::rect(notes), winui::FieldLook::ReadOnly);
                        // Release notes run longer than the box: it scrolls (issue #5).
                        winui::scroll_text(
                            ui,
                            egui::Id::unique("update-notes"),
                            notes,
                            &plain_notes(&rel.notes),
                            ink,
                            13.0,
                            true,
                        );
                        download = Some(rel.clone());
                    }
                }
                // Update now when this copy's file is in the signed release (and it hasn't just
                // failed); else Download opens the release's page.
                let asset = download.as_ref().and_then(|rel| {
                    crate::update::asset_for(install, &rel.version, &rel.assets)
                        .filter(|_| !matches!(self.status, Some(UpdateStatus::Failed(_))))
                });
                let close_label = if self.job.is_some() {
                    // i18n: button: stop downloading the update
                    crate::i18n::mark("Cancel")
                } else if download.is_some() {
                    // i18n: button: not now (download the update later)
                    crate::i18n::mark("Later")
                } else {
                    crate::i18n::mark("OK")
                };
                // The first button: Retry after a failed check, Update now or Download for a new
                // version.
                let first = if matches!(result, Some(Err(_))) {
                    // i18n: button: check for updates again
                    Some("Retry")
                } else if download.is_some() {
                    Some(if asset.is_some() {
                        // i18n: button: download and install the new version, then restart
                        "Update now"
                    } else {
                        "Download"
                    })
                } else {
                    None
                };
                // Under the answer: how Update now is going, else the automatic check's switch,
                // across the window (the buttons are in the footer). In English, as the text pens
                // take it.
                let status_rc =
                    egui::Rect::from_min_size(o + vec2(e, bottom), vec2(W - 2.0 * e, 21.0));
                if let Some(job) = &self.job {
                    let installing = job.installing();
                    // Put in place: a bar that moves, no percent.
                    let fraction = job.progress.fraction().filter(|_| !installing);
                    let line = if installing {
                        // i18n: Check for updates, after Update now downloaded it: unpacking and putting the new version in place
                        crate::i18n::t("Installing the update...").to_owned()
                    } else {
                        crate::i18n::fmt(
                            // i18n: Check for updates, while Update now downloads; {percent} like "42%"
                            crate::i18n::t("Downloading the update... {percent}"),
                            &[(
                                "percent",
                                &fraction.map_or(String::new(), |f| {
                                    crate::i18n::percent(f64::from(f), 0)
                                }),
                            )],
                        )
                    };
                    winui::text(&p, status_rc.left(), status_rc.top() + 9.0, &line, ink);
                    winui::progress_bar(
                        &p,
                        winui::rect(egui::Rect::from_min_size(
                            pos2(status_rc.left(), status_rc.top() + 15.0),
                            vec2(status_rc.width(), 4.0),
                        )),
                        fraction,
                        ui.input(|i| i.time),
                    );
                } else if let Some(status) = &self.status {
                    let line = match status {
                        UpdateStatus::Failed(e) => crate::i18n::fmt(
                            crate::i18n::t("The update could not be installed: {error}"),
                            &[("error", e)],
                        ),
                    };
                    let (rect, cut) = winui::text_fit(
                        &p,
                        status_rc.left(),
                        status_rc.top() + 14.0,
                        &line,
                        ink,
                        status_rc.right(),
                    );
                    if cut {
                        winui::full_on_hover(ui, rect, "update-status", &line);
                    }
                } else {
                    // Its caption stops before the buttons (a longer translation ends in "...").
                    winui::checkbox_until(
                        ui,
                        o + vec2(e, bottom + 4.0),
                        weekly,
                        // i18n: check box in the update window: look for a new version every time MDM starts
                        "Check for updates at every start",
                        true,
                        Some(right),
                    );
                }
                let idle = self.job.is_none();
                // Being put in place: nothing can stop that, so no Cancel (nor Esc) meanwhile.
                let installing = self
                    .job
                    .as_ref()
                    .is_some_and(crate::update::Job::installing);
                // The first button is the default (Enter); without one, Close is. Escape is
                // Close's while nothing downloads (Cancel then stops the download on a click
                // only; Escape closes the window, as before).
                let first_kind = if first == Some("Update now") && !idle {
                    ButtonKind::Disabled
                } else {
                    ButtonKind::Default
                };
                let close_kind = if installing {
                    ButtonKind::Disabled
                } else if first.is_some() {
                    ButtonKind::Normal
                } else {
                    ButtonKind::Default
                };
                let mut items: Vec<FooterItem> = first
                    .map(|label| {
                        FooterItem::Button(
                            FooterButton::new(label, first_kind).primary().enter(true),
                        )
                    })
                    .into_iter()
                    .collect();
                items.push(FooterItem::Button(
                    FooterButton::new(close_label, close_kind)
                        .enter(first.is_none())
                        .escape(idle),
                ));
                let last = items.len() - 1;
                let pressed = winui::footer(ui, o, W, body, &mut items);
                let first_hit = first.is_some() && pressed == Some(0);
                let close = pressed == Some(last);
                // A check that failed can be tried again.
                if matches!(result, Some(Err(_))) && first_hit {
                    self.slot = Some(check(rt, ui.ctx()));
                    self.result = None;
                }
                if let Some(rel) = download.as_ref() {
                    if let Some(asset) = asset {
                        if (first_hit && idle) || (idle && std::mem::take(&mut self.press)) {
                            self.status = None;
                            self.job = Some(crate::update::Job::start(
                                rt,
                                ui.ctx(),
                                install,
                                rel.assets.clone(),
                                asset,
                            ));
                        }
                    } else if first_hit {
                        crate::launch::open(&rel.page);
                        return Outcome::Done(false);
                    }
                }
                if installing {
                    Outcome::Open
                } else if close && self.job.is_some() {
                    // Cancel: the download stops, the window stays (Update now again, Later).
                    self.job = None;
                    Outcome::Open
                } else if close || (!idle && winui::escape_pressed(ui)) {
                    Outcome::Closed
                } else {
                    Outcome::Open
                }
            },
        )
    }
}

/// Release notes (Markdown) as plain lines: headings, bullets, emphasis marks and link
/// addresses dropped (a link keeps its text). A paragraph or bullet wrapped over several lines
/// in the Markdown is one line again (the box wraps it to its own width).
fn plain_notes(md: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut joinable = false;
    for raw in md.lines() {
        let l = raw.trim();
        if l.is_empty() {
            out.push(String::new());
            joinable = false;
            continue;
        }
        let heading = l.starts_with('#');
        let bullet = l.strip_prefix("- ").or_else(|| l.strip_prefix("* "));
        let text = link_texts(l.trim_start_matches('#').trim_start())
            .replace("**", "")
            .replace('`', "");
        match (bullet, out.last_mut()) {
            (None, Some(last)) if joinable && !heading => {
                last.push(' ');
                last.push_str(&text);
            }
            (Some(b), _) => {
                let b = link_texts(b).replace("**", "").replace('`', "");
                out.push(format!("• {b}"));
            }
            _ => out.push(text),
        }
        joinable = !heading;
    }
    out.join("\n")
}

/// `[text](address)` becomes `text`.
fn link_texts(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        let Some(close) = after.find("](") else { break };
        let Some(end) = after[close + 2..].find(')') else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str(&after[..close]);
        rest = &after[close + 2 + end + 1..];
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------------------------
// About

pub struct About;

impl About {
    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<()> {
        // A long translation pushes the links and OK down; the window grows with them.
        let extra = crate::dialogs::grow_extra(ctx, "about");
        const W: f32 = 420.0;
        const BODY: f32 = 248.0;
        let body = BODY + extra;
        window(
            ctx,
            "about",
            "About Grabbit",
            [W, body + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| {
                let p = ui.painter().clone();
                let ink = winui::TEXT();
                // The mockup's About: everything centred under the app's mark at 56, from the
                // icon's PNG just above that size on screen (the 48 one stretched blurred at 150
                // and 200 %; Direct3D has no mipmaps, so a far larger one would shimmer at 100).
                let cx = o.x + W / 2.0;
                let mark = if 56.0 * ui.ctx().pixels_per_point() <= 64.0 {
                    egui::Image::from_bytes(
                        "bytes://mdm-64.png",
                        include_bytes!("../assets/icons/app/mdm-64.png"),
                    )
                } else {
                    egui::Image::from_bytes(
                        "bytes://mdm-128.png",
                        include_bytes!("../assets/icons/app/mdm-128.png"),
                    )
                };
                mark.paint_at(ui, winui::r(o, W / 2.0 - 28.0, 24.0, 56.0, 56.0));
                let centred = |font: egui::FontId, baseline: f32, s: &str, color| {
                    let w = winui::text_width_font(&p, font.clone(), s);
                    winui::text_at(&p, font, (cx - w / 2.0).round(), baseline, s, color)
                };
                centred(
                    winui::tahoma_title(),
                    o.y + 104.0,
                    crate::i18n::t("Grabbit"),
                    ink,
                );
                let version = crate::i18n::fmt(
                    crate::i18n::t("Version {version} ({commit})"),
                    &[("version", VERSION), ("commit", COMMIT)],
                );
                // Copyable: bug reports ask for it.
                let rect = centred(winui::tahoma(), o.y + 124.0, &version, winui::MUTED());
                winui::copyable(ui, rect, "about-version", &version, false);
                let about = winui::wrap_lines(
                    &p,
                    crate::i18n::t(
                        "The download manager you already know, rebuilt to be faster. Free and open source under the GNU GPL, version 3 or later. Videos are found by yt-dlp; torrents run on librqbit.",
                    ),
                    W - 2.0 * 32.0,
                );
                let mut y = o.y + 150.0;
                for line in &about {
                    centred(winui::tahoma(), y, line, ink);
                    y += 16.0;
                }
                // The links in centred rows, 18 px apart: one that would pass the room (a long
                // translation) starts the next row.
                let links = ["Home page", "Source code", "Licenses and credits"];
                let width = |l: &str| winui::text_width(&p, crate::i18n::t(l));
                let mut rows: Vec<Vec<&str>> = vec![Vec::new()];
                let mut row_w = 0.0;
                for l in links {
                    let w = width(l);
                    let last = rows.last_mut().expect("a row");
                    if !last.is_empty() && row_w + 18.0 + w > W - 2.0 * 32.0 {
                        rows.push(vec![l]);
                        row_w = w;
                    } else {
                        row_w += if last.is_empty() { w } else { 18.0 + w };
                        last.push(l);
                    }
                }
                y += 8.0;
                for row in &rows {
                    let total: f32 =
                        row.iter().map(|l| width(l)).sum::<f32>() + 18.0 * (row.len() - 1) as f32;
                    let mut x = (cx - total / 2.0).round();
                    for l in row {
                        if winui::link(ui, &p, x, y, l, l) {
                            match *l {
                                "Home page" => open_page(""),
                                "Source code" => crate::launch::open(REPO),
                                _ => {
                                    crate::launch::open(format!("{REPO}/blob/main/ATTRIBUTION.md"))
                                }
                            }
                        }
                        x += width(l) + 18.0;
                    }
                    y += 16.0;
                }
                // Its own row under them: SPONSORS (Help → Support MDM).
                y += 6.0;
                let support_label = "Support Grabbit on GitHub";
                let sw = width(support_label);
                if winui::link(ui, &p, (cx - sw / 2.0).round(), y, support_label, "support") {
                    support();
                }
                // The last row ends 16 above the footer; the window grows with a long text.
                crate::dialogs::grow_px(ui.ctx(), "about", y + 20.0 - (o.y + BODY));
                // Enter and Escape both close it, as before.
                let ok = winui::footer(
                    ui,
                    o,
                    W,
                    body,
                    &mut [FooterItem::Button(
                        FooterButton::new("OK", ButtonKind::Default)
                            .enter(true)
                            .escape(true),
                    )],
                );
                if ok.is_some() {
                    Outcome::Closed
                } else {
                    Outcome::Open
                }
            },
        )
    }
}

// ---------------------------------------------------------------------------------------------
// Keyboard shortcuts

/// Every shortcut the main window answers (keep in step with `app.rs` and `list.rs`).
const SHORTCUTS: [(&str, &str); 29] = [
    (
        "Ctrl+N",
        "Add links, magnets or video pages (the Add window)",
    ),
    ("Ctrl+O", "Open a .torrent file"),
    (
        "Ctrl+V",
        "Paste into the paste bar (several links: the Add window)",
    ),
    (
        "Ctrl+F",
        "Go to the Find field in the title bar, its text selected",
    ),
    (
        "F3 / Shift+F3",
        "Next / previous match (in the field: Enter / Shift+Enter)",
    ),
    (
        "Esc in the Find field",
        "Clear it and go back to the list (or close its menu)",
    ),
    ("Enter", "Open the selected download"),
    ("Alt+Enter", "Properties of the selected download"),
    ("Ctrl+M / F2", "Move or rename the selected download"),
    ("Delete", "Delete the selected downloads (asks first)"),
    (
        "Shift+Delete",
        "Delete the selected downloads and their files (asks first)",
    ),
    ("Ctrl+C", "Copy the selected downloads' addresses"),
    ("Ctrl+A", "Select all downloads"),
    ("Ctrl+click", "Select or unselect one more download"),
    ("Shift+click", "Select every download up to this one"),
    ("Drag on blank space", "Select the downloads in the box"),
    ("Up / Down", "Select the previous / next download"),
    (
        "Shift+Up / Shift+Down",
        "Make the selection longer or shorter",
    ),
    ("Ctrl+Up / Ctrl+Down", "Move without changing the selection"),
    (
        "Ctrl+Space",
        "Select or unselect the download with the dotted outline",
    ),
    ("Page Up / Page Down", "Move a page up / down"),
    ("Home / End", "Select the first / last download"),
    (
        "Type a name",
        "Jump to the download whose name starts with it",
    ),
    ("Shift+F10", "Open the selected download's menu"),
    (
        "Tab",
        "Move between the paste bar, the categories and the download list",
    ),
    (
        "Shift+F10 / Menu",
        "In the categories: the selected category's menu",
    ),
    ("Esc", "Close a menu or dialog"),
    // View → Interface size (#262), in every window.
    (
        "Ctrl++ / Ctrl+-",
        "Make everything larger / smaller (View → Interface size)",
    ),
    ("Ctrl+0", "Everything at its normal size (100%)"),
];

pub struct Shortcuts;

impl Shortcuts {
    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<()> {
        let rows = SHORTCUTS.len() as f32;
        const W: f32 = 520.0;
        let body = 24.0 + 18.0 * rows + 8.0;
        window(
            ctx,
            "shortcuts",
            "Keyboard shortcuts",
            [W, body + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| {
                let p = ui.painter().clone();
                let ink = winui::TEXT();
                for (i, (keys, what)) in SHORTCUTS.iter().enumerate() {
                    let y = o.y + 24.0 + 18.0 * i as f32;
                    // The translated key names.
                    let keys = crate::chrome::key_label(keys);
                    winui::fit(
                        ui,
                        &p,
                        o.x + 16.0,
                        y,
                        &keys,
                        ink,
                        o.x + 174.0,
                        ("keys", y as i32),
                    );
                    let what = crate::i18n::t(what);
                    winui::fit(
                        ui,
                        &p,
                        o.x + 184.0,
                        y,
                        what,
                        ink,
                        o.x + 504.0,
                        ("what", y as i32),
                    );
                }
                // Enter and Escape both close it (with a modifier held too, as before).
                let any_key = ui
                    .input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Enter));
                let ok = winui::footer(
                    ui,
                    o,
                    W,
                    body,
                    &mut [FooterItem::Button(FooterButton::new(
                        "OK",
                        ButtonKind::Default,
                    ))],
                );
                if ok.is_some() || any_key {
                    Outcome::Closed
                } else {
                    Outcome::Open
                }
            },
        )
    }
}

// ---------------------------------------------------------------------------------------------
// Tell a friend

pub struct TellFriend {
    /// What became of Copy or Email: the line under the message, and whether it is a failure.
    said: Option<(&'static str, bool)>,
}

impl TellFriend {
    pub fn new() -> Self {
        TellFriend { said: None }
    }

    fn message() -> String {
        crate::i18n::fmt(
            crate::i18n::t(
                "I download with Grabbit: fast, free, and it looks like the download manager you already know. {link}",
            ),
            &[("link", SITE)],
        )
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<()> {
        let message = Self::message();
        // The message's box grows with a long translation, the window with it.
        let extra = crate::dialogs::grow_extra(ctx, "tell-a-friend");
        const W: f32 = 440.0;
        let body = 154.0 + extra;
        window(
            ctx,
            "tell-a-friend",
            "Tell a friend",
            [W, body + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| {
                let p = ui.painter().clone();
                let ink = winui::TEXT();
                let e = winui::EDGE;
                winui::text(
                    &p,
                    o.x + e,
                    o.y + 29.0,
                    crate::i18n::t("Send this to a friend:"),
                    ink,
                );
                let rect = egui::Rect::from_min_max(
                    pos2(o.x + e, o.y + 40.0),
                    pos2(o.x + W - e, o.y + 116.0 + extra),
                );
                // English, as the paragraph is laid out; made where it is on screen.
                // Read-only text: the read-only field's look (it drew a square box of its own).
                winui::field_frame(&p, winui::rect(rect), winui::FieldLook::ReadOnly);
                let lines = winui::paragraph(
                    &p,
                    rect.left() + 6.0,
                    rect.top() + 17.0,
                    &message,
                    ink,
                    rect.right() - 6.0,
                    13.0,
                );
                crate::dialogs::grow_to(ui.ctx(), "tell-a-friend", lines, 5);
                if let Some((said, failed)) = self.said {
                    // Under the message (a long translation: "...", whole on hover).
                    let c = if failed {
                        winui::ERROR()
                    } else {
                        winui::MUTED()
                    };
                    winui::fit(
                        ui,
                        &p,
                        o.x + e,
                        o.y + 136.0 + extra,
                        crate::i18n::t(said),
                        c,
                        o.x + W - e,
                        "copied",
                    );
                }
                // Enter copies, Escape closes, as before.
                let pressed = winui::footer(
                    ui,
                    o,
                    W,
                    body,
                    &mut [
                        FooterItem::Button(
                            FooterButton::new("Copy", ButtonKind::Default).enter(true),
                        ),
                        FooterItem::Button(FooterButton::new("Email", ButtonKind::Normal)),
                        FooterItem::Button(
                            FooterButton::new("Close", ButtonKind::Normal).escape(true),
                        ),
                    ],
                );
                let (copy, email, close) =
                    (pressed == Some(0), pressed == Some(1), pressed == Some(2));
                // The user asked for it: the message goes to the clipboard (a busy clipboard
                // says so; it showed nothing).
                let to_clipboard = crate::set_clipboard;
                if copy {
                    self.said = Some(if to_clipboard(&message) {
                        (crate::i18n::mark("Copied to the clipboard."), false)
                    } else {
                        // i18n: Tell a friend: Copy failed (another program holds the clipboard)
                        (crate::i18n::mark("The clipboard is busy: try again."), true)
                    });
                }
                if email {
                    let enc = |s: &str| {
                        url::form_urlencoded::byte_serialize(s.as_bytes())
                            .collect::<String>()
                            .replace('+', "%20")
                    };
                    let subject = crate::i18n::t("Grabbit");
                    let opened = open::that_detached(format!(
                        "mailto:?subject={}&body={}",
                        enc(subject),
                        enc(&message)
                    ));
                    // No mail program: the message is copied instead, and said so.
                    if opened.is_err() {
                        self.said = Some(if to_clipboard(&message) {
                            // i18n: Tell a friend: Email found no mail program, so the message was copied
                            (
                                crate::i18n::mark(
                                    "No mail app: the message is copied, paste it in your mail.",
                                ),
                                false,
                            )
                        } else {
                            // i18n: Tell a friend: Email found no mail program
                            (crate::i18n::mark("No mail app opened."), true)
                        });
                    }
                }
                if close {
                    Outcome::Closed
                } else {
                    Outcome::Open
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number() {
        assert!(newer("0.2.0", "0.1.0"));
        assert!(newer("0.10.0", "0.9.9"));
        assert!(newer("1.0", "0.9.9"));
        assert!(
            newer("1.0.0", "1.0.0-rc.1"),
            "a release is newer than its pre-release"
        );
        assert!(!newer("0.1.0", "0.1.0"));
        assert!(!newer("0.1.0-beta", "0.1.0"));
        assert!(!newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn a_release_is_read_from_github() {
        let json = r###"{"tag_name":"v0.2.0","published_at":"2026-11-05T10:00:00Z",
            "html_url":"https://github.com/x/y/releases/tag/v0.2.0","body":"## New\n- Faster",
            "assets":[{"name":"SHA256SUMS.txt","browser_download_url":"https://x/SHA256SUMS.txt"}]}"###;
        assert_eq!(
            parse_release(json, "0.1.0").unwrap(),
            Found::Newer(Release {
                version: "0.2.0".into(),
                date: "2026-11-05".into(),
                notes: "## New\n- Faster".into(),
                page: "https://github.com/x/y/releases/tag/v0.2.0".into(),
                assets: vec![("SHA256SUMS.txt".into(), "https://x/SHA256SUMS.txt".into())],
            })
        );
        assert_eq!(parse_release(json, "0.2.0").unwrap(), Found::Latest);
        assert!(parse_release("{}", "0.1.0").is_err());
    }

    // #1123: whatever GitHub (or a proxy, or a captive portal) answers, the check fails with a
    // reason and never panics.
    #[test]
    fn a_bad_answer_is_a_failed_check() {
        let bad = [
            // Malformed or cut-off JSON, or not JSON at all.
            r#"{"tag_name":"v0.6.0","assets":["#,
            "",
            "<html><body>Sign in to the Wi-Fi</body></html>",
            // An empty release list, or no release at all.
            "[]",
            "null",
            r#"{"tag_name":""}"#,
            r#"{"tag_name":"v"}"#,
            // Version strings that don't parse.
            r#"{"tag_name":"next"}"#,
            r#"{"tag_name":"v1.x"}"#,
            r#"{"tag_name":"v١.٢"}"#,
            r#"{"tag_name":"v0..6"}"#,
            r#"{"tag_name":"v99999999999999999999999.0"}"#,
            r#"{"tag_name":"v0.6.-rc"}"#,
            r#"{"tag_name":42}"#,
        ];
        for json in bad {
            assert!(parse_release(json, VERSION).is_err(), "{json:?} was taken");
        }
        // Odd but readable answers still work: a pre-release, missing fields, a short date,
        // and text that isn't ASCII where the date is cut.
        let odd = r#"{"tag_name":"V9.0.0-rc.1","published_at":"2026-ü","assets":{"x":1}}"#;
        match parse_release(odd, "0.1.0").unwrap() {
            Found::Newer(r) => {
                assert_eq!(r.version, "9.0.0-rc.1");
                assert_eq!(r.date, "2026-ü");
                assert!(r.assets.is_empty() && r.notes.is_empty() && r.page.is_empty());
            }
            f => panic!("{f:?}"),
        }
    }

    #[test]
    fn a_check_that_panics_is_a_failed_check() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let joined = rt.block_on(async {
            tokio::spawn(async { panic!("a test panic in the update check") }).await
        });
        assert!(settled(joined).is_err());
        let fine = rt.block_on(async { tokio::spawn(async { Ok(Found::Latest) }).await });
        assert_eq!(settled(fine), Ok(Found::Latest));
    }

    /// A local server that answers every request with `status` and `body` (a fixture: the
    /// tests never ask GitHub).
    async fn feed(status: &'static str, body: &'static str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut s, _)) = listener.accept().await {
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf).await;
                let reply = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = s.write_all(reply.as_bytes()).await;
                let _ = s.shutdown().await;
            }
        });
        format!("http://{addr}/repos/x/y/releases/latest")
    }

    /// A client as the check's, without the system's proxy; for GitHub's pages, following no
    /// redirect.
    fn test_client(redirects: bool) -> reqwest::Client {
        let policy = if redirects {
            reqwest::redirect::Policy::default()
        } else {
            reqwest::redirect::Policy::none()
        };
        reqwest::Client::builder()
            .no_proxy()
            .redirect(policy)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
    }

    fn pages(_: &str) -> Result<reqwest::Client, String> {
        Ok(test_client(false))
    }

    /// The API at `feed`; GitHub's pages on a port nothing listens on.
    fn api_only(feed: &str) -> Sources {
        let closed = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap()
        };
        Sources {
            feed: feed.to_owned(),
            web: format!("http://{closed}/web"),
            raw: format!("http://{closed}/raw"),
        }
    }

    #[test]
    fn the_check_reads_the_feed_and_fails_softly() {
        use mdm_tools::update::Install;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let client = test_client(true);
            let check = |feed: String| {
                let client = client.clone();
                async move {
                    fetch(&client, &pages, &api_only(&feed), "0.5.2", Install::Source).await
                }
            };
            let newer = feed(
                "200 OK",
                r#"{"tag_name":"v9.0.0","published_at":"2026-12-01T00:00:00Z"}"#,
            )
            .await;
            assert!(matches!(
                check(newer).await,
                Ok(Found::Newer(r)) if r.version == "9.0.0" && r.date == "2026-12-01"
            ));
            let none = feed("404 Not Found", r#"{"message":"Not Found"}"#).await;
            assert_eq!(check(none).await, Ok(Found::Latest));
            for (status, body) in [
                // GitHub's rate limit, and its pages not answering either.
                ("403 Forbidden", r#"{"message":"API rate limit exceeded"}"#),
                ("403 Forbidden", r#"{"message":"Repository access blocked"}"#),
                ("500 Internal Server Error", ""),
                ("200 OK", r#"{"tag_name":"v9.0.0","#),
                ("200 OK", "[]"),
                ("200 OK", r#"{"tag_name":"latest"}"#),
            ] {
                let url = feed(status, body).await;
                assert!(check(url).await.is_err(), "{status} {body}");
            }
            // Nothing listening: a network error.
            let closed = {
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                l.local_addr().unwrap()
            };
            assert!(check(format!("http://{closed}/latest")).await.is_err());
        });
    }

    /// #1337: GitHub's API refuses for its rate limit (the lab server's fake GitHub). The check
    /// finds the latest release on the pages: its version from where `releases/latest`
    /// redirects, its date and notes from CHANGELOG.md at its tag, and of its files the signed
    /// list and this copy's, those the release has (the plain zip, not the setup it hasn't);
    /// Update now takes it. When the pages fail too, the check says GitHub refused too many
    /// requests from this network.
    #[test]
    fn a_rate_limited_check_reads_the_release_pages() {
        use mdm_tools::update::Install;
        let files = tempfile::tempdir().unwrap();
        let repo = files.path().join("gh").join(REPO_PATH);
        let release = repo.join("v9.9.9");
        std::fs::create_dir_all(&release).unwrap();
        std::fs::write(repo.join("LATEST"), "v9.9.9").unwrap();
        let zip = Install::PlainZip.assets("9.9.9");
        let mut published = vec!["SHA256SUMS.txt", "SHA256SUMS.txt.sig"];
        published.extend(zip.first().map(String::as_str));
        for name in &published {
            std::fs::write(release.join(name), name).unwrap();
        }
        std::fs::write(
            release.join("CHANGELOG.md"),
            "# Changelog\n\n## [Unreleased]\n\n## [9.9.9] - 2026-12-24\n\n### Fixed\n\n- **Check for updates** works behind a shared address.\n\n## [9.9.8] - 2026-12-01\n\n- Older.\n",
        )
        .unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let lab = mdm_labserver::LabServer::start(mdm_labserver::Config {
                files: Some(files.path().to_owned()),
                ..Default::default()
            })
            .await
            .unwrap();
            let client = test_client(true);
            let sources = Sources {
                feed: lab.url(&format!("/gh/api/repos/{REPO_PATH}/releases/latest")),
                web: lab.url("/gh/web"),
                raw: lab.url("/gh/raw"),
            };
            let web = sources.web.clone();
            let found = fetch(&client, &pages, &sources, "0.5.2", Install::PlainZip).await;
            let Ok(Found::Newer(rel)) = found else {
                panic!("{found:?}")
            };
            assert_eq!(rel.version, "9.9.9");
            assert_eq!(rel.date, "2026-12-24");
            assert_eq!(
                plain_notes(&rel.notes),
                "Fixed\n\n• Check for updates works behind a shared address."
            );
            assert_eq!(rel.page, format!("{web}/{REPO_PATH}/releases/tag/v9.9.9"));
            let want: Vec<(String, String)> = published
                .iter()
                .map(|n| {
                    (
                        (*n).to_owned(),
                        format!("{web}/{REPO_PATH}/releases/download/v9.9.9/{n}"),
                    )
                })
                .collect();
            assert_eq!(rel.assets, want);
            assert_eq!(
                crate::update::asset_for(Install::PlainZip, &rel.version, &rel.assets),
                zip.first().cloned()
            );
            // Not newer than this copy: the latest.
            assert_eq!(
                fetch(&client, &pages, &sources, "9.9.9", Install::PlainZip).await,
                Ok(Found::Latest)
            );
            // No CHANGELOG.md at the tag: no date nor notes, the files all the same.
            std::fs::remove_file(release.join("CHANGELOG.md")).unwrap();
            let found = fetch(&client, &pages, &sources, "0.5.2", Install::PlainZip).await;
            assert!(
                matches!(&found, Ok(Found::Newer(r)) if r.notes.is_empty() && r.assets == want),
                "{found:?}"
            );
            // No release on the pages either: GitHub refused too many requests.
            std::fs::remove_file(repo.join("LATEST")).unwrap();
            let err = fetch(&client, &pages, &sources, "0.5.2", Install::PlainZip)
                .await
                .unwrap_err();
            assert!(
                err.contains("127.0.0.1")
                    && err.contains("403")
                    && err.contains("too many requests"),
                "{err}"
            );
        });
    }

    #[test]
    fn notes_become_plain_lines() {
        assert_eq!(
            plain_notes("## New\n- **Faster** `x`\n* b"),
            "New\n• Faster x\n• b"
        );
        assert_eq!(
            plain_notes("Tell us in an [issue](https://x/issues), or [the guide](docs/a.md#b)."),
            "Tell us in an issue, or the guide."
        );
        assert_eq!(plain_notes("[not a link] and [x]"), "[not a link] and [x]");
        // Hard-wrapped Markdown: one line per paragraph and per bullet.
        assert_eq!(
            plain_notes("Fixes for\nthe Mac.\n\n### Fixed\n\n- **Menus** did\n  nothing.\n- Paste"),
            "Fixes for the Mac.\n\nFixed\n\n• Menus did nothing.\n• Paste"
        );
    }

    #[test]
    fn the_report_fills_fields_the_bug_form_has() {
        let form = include_str!("../../../.github/ISSUE_TEMPLATE/bug.yml").replace("\r\n", "\n");
        let has = |line: String| form.contains(&format!("{line}\n"));
        let fields = report_fields();
        for (id, _) in &fields {
            assert!(has(format!("id: {id}")), "no field {id} in bug.yml");
            // Filled as text: GitHub's page put a dropdown's value from the address back to none.
            let field = form
                .split(&format!(
                    "id: {id}
"
                ))
                .next()
                .unwrap();
            assert!(
                field.ends_with(
                    "- type: input
    "
                ),
                "{id} isn't a text field"
            );
        }
        assert!(fields.iter().any(|(id, _)| *id == "install"));
        // Every value Install::label() and report_fields() can give, on any system, is one the
        // field's description names.
        let named = |v: &str| {
            [format!("{v}, "), format!("{v} or "), format!(" or {v}.")]
                .iter()
                .any(|s| form.contains(s.as_str()))
        };
        for install in mdm_tools::update::Install::ALL {
            assert!(named(install.label()), "{}", install.label());
        }
        for option in ["Windows 11", "Windows 10", "Other"] {
            assert!(named(option), "{option}");
        }
    }

    #[test]
    fn help_pages_are_the_websites() {
        assert_eq!(
            page_url("README.md"),
            "https://humblef0ol.github.io/grabbit/help/"
        );
        assert_eq!(
            page_url("README.md#tutorials"),
            "https://humblef0ol.github.io/grabbit/help/#tutorials"
        );
        assert_eq!(
            page_url("video.md"),
            "https://humblef0ol.github.io/grabbit/help/video.html"
        );
        assert_eq!(page_url(""), "https://humblef0ol.github.io/grabbit/");
    }

    /// The repository's root.
    fn root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Every user guide page the program links to (`doc` as page_url takes it): the literal
    /// pages given to open_page and the Help menu (`Cmd::HelpPage`) in the app's source, the
    /// extension window's tutorial and help sections, and every website help address written in
    /// a crate or in the browser extension.
    fn linked_docs() -> Vec<String> {
        let mut docs = vec![mdm_ipc::extension::GUIDE_DOC.to_owned()];
        // The sections Options' Test browser integration opens (extension::checks' reasons).
        for why in ["options", "not-installed"] {
            docs.push(mdm_ipc::extension::help_doc(why));
        }
        let calls = regex::Regex::new(r#"(?:open_page|HelpPage)\(\s*"([^"]+)""#).unwrap();
        let site =
            regex::Regex::new(r"https://humblef0ol\.github\.io/grabbit/help/([\w.-]*)(#[\w-]+)?")
                .unwrap();
        let mut files = vec![];
        let mut dirs = vec![root().join("crates"), root().join("extension")];
        while let Some(dir) = dirs.pop() {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let path = e.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|x| x == "rs" || x == "js") {
                    files.push(path);
                }
            }
        }
        let app = root().join("crates").join("app");
        for f in files {
            let text = std::fs::read_to_string(&f).unwrap();
            if f.starts_with(&app) {
                docs.extend(calls.captures_iter(&text).map(|c| c[1].to_owned()));
            }
            for c in site.captures_iter(&text) {
                let page = match &c[1] {
                    "" => "README.md".to_owned(),
                    p => format!("{}.md", p.strip_suffix(".html").unwrap_or(p)),
                };
                docs.push(format!("{page}{}", c.get(2).map_or("", |a| a.as_str())));
            }
        }
        docs.sort();
        docs.dedup();
        docs
    }

    /// A heading's section id as the website's Markdown (Python-Markdown's toc) makes it: its
    /// own `{#id}`, else its text in lower case, letters, digits, `_` and `-` kept, spaces `-`.
    fn heading_id(line: &str) -> Option<String> {
        let text = line.trim_start_matches('#');
        if text.len() == line.len() || !text.starts_with(' ') {
            return None;
        }
        if let Some((_, id)) = text.split_once("{#") {
            return Some(id.trim_end().trim_end_matches('}').to_owned());
        }
        let link = regex::Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap();
        let kept: String = link
            .replace_all(text, "$1")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ' '))
            .collect::<String>()
            .trim()
            .to_lowercase();
        let dashes = regex::Regex::new(r"[-\s]+").unwrap();
        Some(dashes.replace_all(&kept, "-").into())
    }

    #[test]
    fn every_linked_help_page_and_section_is_in_the_guide() {
        let docs = linked_docs();
        assert!(docs.len() >= 8, "{docs:?}");
        for doc in &docs {
            let (file, anchor) = doc.split_once('#').unwrap_or((doc, ""));
            let Ok(text) = std::fs::read_to_string(root().join("docs").join(file)) else {
                panic!("{doc}: docs/{file} isn't there: the website has no such page");
            };
            if !anchor.is_empty() {
                let ids: Vec<String> = text.lines().filter_map(heading_id).collect();
                assert!(
                    ids.iter().any(|id| id == anchor),
                    "{doc}: no section #{anchor} in docs/{file} ({ids:?})"
                );
            }
        }
    }

    /// The same pages on a built website: `python tools/website/build.py` (MDM_SITE_PREVIEW=1
    /// is enough, English), then `MDM_SITE_DIR=target/site cargo test -p grabbit-app --
    /// --ignored linked_help_pages_are_on_the_built_site`: each page is a file there, with its
    /// section's id.
    #[test]
    #[ignore = "needs the website built, in MDM_SITE_DIR"]
    fn linked_help_pages_are_on_the_built_site() {
        let out = std::path::PathBuf::from(std::env::var("MDM_SITE_DIR").expect("MDM_SITE_DIR"));
        for doc in linked_docs() {
            let url = page_url(&doc);
            let path = url.strip_prefix(SITE).unwrap();
            let (path, anchor) = path.split_once('#').unwrap_or((path, ""));
            let mut file = out.join(path.trim_start_matches('/'));
            if path.ends_with('/') {
                file.push("index.html");
            }
            let Ok(html) = std::fs::read_to_string(&file) else {
                panic!("{doc}: {url} isn't built ({})", file.display());
            };
            if !anchor.is_empty() {
                let id = format!("id=\"{anchor}\"");
                assert!(html.contains(&id), "{doc}: {url} has no #{anchor}");
            }
            println!("{doc} -> {url}: ok");
        }
    }
}
