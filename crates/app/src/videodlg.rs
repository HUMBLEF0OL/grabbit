//! Tasks → Download video from a page… (M9, ours: IDM's video panel is fed by its browser
//! module, which comes with M8). Paste a page, look it up with yt-dlp, pick a quality, choose
//! where it goes. yt-dlp is installed (verified, D-011) from here on first use, and ffmpeg (the
//! LGPL build) when the best qualities need merging. It follows the app's mode, like Add torrent.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{Pos2, Ui, pos2};
use mdm_tools::resolve::{Choice, Looked, SubChoice, Video};
use mdm_tools::video::{ALL_UPLOADERS, Subs};

use crate::dialogs::{Outcome, window};
use crate::model::Category;
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, r, text};

/// Download video's width, and where its footer starts (the mockup's layout).
const VIDEO_W: f32 = 600.0;
const VIDEO_BODY: f32 = 382.0;
/// Its content's left and right edges (the playlist's too, playlistdlg.rs).
pub(crate) const LX: f32 = winui::EDGE;
pub(crate) const RIGHT: f32 = VIDEO_W - winui::EDGE;
/// The rows under the list: Save to, then File name (a playlist's Quality), then Subtitles.
const ROWS_Y: [f32; 3] = [256.0, 296.0, 336.0];

type Slot<T> = Arc<Mutex<Option<Result<T, String>>>>;

/// What Download Now or Later adds: a video, or a playlist's (#69).
pub enum VideoOut {
    One(VideoAdd),
    Many(crate::playlistdlg::PlaylistAdd),
}

/// What OK adds.
pub struct VideoAdd {
    pub page: url::Url,
    pub selector: String,
    pub dest: PathBuf,
    /// The category it is listed under: Music for a sound-only choice, else Video.
    pub category: Category,
    pub size: Option<u64>,
    pub start_now: bool,
    /// What the browser's request for the page carried (its button), for yt-dlp to ask the
    /// same; none for an address typed or edited here.
    pub context: mdm_ipc::Command,
    /// The subtitles to save with it (#68; none for a sound-only choice).
    pub subs: Subs,
    /// The Subtitles choice to remember for the next video (settings.video_subs); none when
    /// the page had no subtitles to choose from (the last real choice stays).
    pub remember: Option<Subs>,
}

/// Which helper is being installed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    YtDlp,
    Ffmpeg,
}

pub struct VideoDialog {
    page: String,
    /// The browser's request context for `page` as it came (VideoAdd::context).
    context: mdm_ipc::Command,
    from_browser: String,
    /// Save to.
    folder: String,
    /// The Video and Music categories' folders MDM puts in Save to (upstream issue 611): Music's
    /// while a sound-only choice is chosen, as long as Save to holds one of them (a folder typed
    /// or picked stays). No Music folder: the Music category was deleted, all goes as video.
    video_dir: String,
    music_dir: Option<String>,
    /// Save to was last set for a sound-only choice.
    sound: bool,
    name: String,
    lookup: Option<Slot<Looked>>,
    /// The yt-dlp and the browser's context the look-up ran with (a list is listed with them).
    looking: Option<(PathBuf, mdm_tools::proc::FromBrowser)>,
    /// The page is a playlist or a channel: its videos (#69).
    list: Option<crate::playlistdlg::PlaylistView>,
    /// Adding that many was answered Yes: Download Now (true) or Later goes ahead.
    confirmed: Option<bool>,
    install: Option<(Tool, Slot<mdm_tools::Installed>)>,
    /// The helpers found (read again when an install ends, not every frame: it reads files).
    tools: Option<Tools>,
    /// The address the choices were looked up for (an edited one needs Look up again).
    looked_up: String,
    video: Option<(Video, Vec<Choice>)>,
    chosen: usize,
    /// First choice shown (the list scrolls by whole rows).
    /// The choices list's scroll and selection.
    rows: winui::RowList,
    error: Option<String>,
    focused: bool,
    /// Download Now (true) or Later, once chosen.
    start_now: Option<bool>,
    /// Look the page up as soon as yt-dlp is there (opened from Add URL).
    auto_look: bool,
    /// Asking to create a missing folder.
    asking: crate::dialogs::FolderGate<()>,
    /// Helpers...: where yt-dlp and ffmpeg come from (#51).
    helpers: Option<crate::helpers::Programs>,
    /// The looked-up video's subtitle languages (#68) and the one chosen in the Subtitles list
    /// (0: none, then "All the uploader's" when offered, then `subs` in order).
    subs: Vec<SubChoice>,
    sub_pick: usize,
    /// Put the subtitles into the video rather than beside it.
    embed: bool,
    /// The last video's choice (settings.video_subs), applied to each video looked up.
    pref: Subs,
    /// The clipboard's page address, read off the window's thread (#661).
    clip: crate::clipboard::Prefill,
}

/// What is installed in the tools folder: (folder, yt-dlp, ffmpeg).
type Tools = (
    Option<PathBuf>,
    Option<mdm_tools::Installed>,
    Option<mdm_tools::Installed>,
);

/// Keep yt-dlp current: video sites change often and an old yt-dlp is the usual reason a page
/// stops working. Once per session, when the video window opens and yt-dlp is installed, the
/// newest release is fetched in the background (mdm_tools::install_latest: signed by yt-dlp's
/// developers, never older, switched atomically, the previous version kept). Quiet: a failure
/// leaves the installed version in use. A system copy is left to its package manager.
pub fn refresh_ytdlp(rt: &tokio::runtime::Handle) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    let Some(dir) = mdm_tools::tools_dir() else {
        return;
    };
    if !mdm_tools::refreshable(&dir) || DONE.swap(true, Ordering::SeqCst) {
        return;
    }
    rt.spawn(async move {
        if let Err(e) = mdm_tools::install_latest(&dir, &crate::settings::own_net(), None).await {
            eprintln!("yt-dlp update check: {e}"); // not translated: a log line
        }
    });
}

fn tools() -> Tools {
    let dir = mdm_tools::tools_dir();
    let ytdlp = dir.as_deref().and_then(mdm_tools::installed);
    let ffmpeg = dir.as_deref().and_then(mdm_tools::ffmpeg::installed);
    (dir, ytdlp, ffmpeg)
}

impl VideoDialog {
    /// Opened from the menu: a video page's address on the clipboard is taken (read off the
    /// window's thread, put in the box on the frame it comes).
    pub fn new(folder: PathBuf) -> Self {
        VideoDialog {
            clip: crate::clipboard::Prefill::ask(),
            ..Self::blank(folder)
        }
    }

    /// The clipboard's text as the page, when it is a web address and the box is still empty
    /// (nothing typed meanwhile).
    fn fill_from_clipboard(&mut self, clip: &str) {
        let clip = clip.trim();
        if self.page.is_empty()
            && url::Url::parse(clip).is_ok_and(|u| matches!(u.scheme(), "http" | "https"))
        {
            self.page = clip.to_owned();
        }
    }

    /// No address yet, saving into `folder` (the Video category's).
    fn blank(folder: PathBuf) -> Self {
        let folder = folder.display().to_string();
        VideoDialog {
            page: String::new(),
            context: Default::default(),
            from_browser: String::new(),
            video_dir: folder.clone(),
            music_dir: None,
            sound: false,
            folder,
            name: String::new(),
            lookup: None,
            looking: None,
            list: None,
            confirmed: None,
            install: None,
            tools: None,
            looked_up: String::new(),
            video: None,
            chosen: 0,
            rows: Default::default(),
            error: None,
            focused: false,
            start_now: None,
            auto_look: false,
            asking: Default::default(),
            helpers: None,
            subs: Vec::new(),
            sub_pick: 0,
            embed: false,
            pref: Subs::default(),
            clip: Default::default(),
        }
    }

    /// Offer `pref` (the last video's subtitles, settings.video_subs) for the videos looked up.
    pub fn with_subs(mut self, pref: Subs) -> Self {
        self.embed = pref.embed;
        self.pref = pref;
        self
    }

    /// Save to switches to `dir`, the Music category's folder, while a sound-only choice is
    /// chosen (upstream issue 611); None (the Music category was deleted): sound goes as video
    /// does.
    pub fn with_music_folder(mut self, dir: Option<PathBuf>) -> Self {
        self.music_dir = dir.map(|d| d.display().to_string());
        self
    }

    /// Opened for `page` (Add URL was given a video's page, or the browser's button was
    /// clicked with `context`): looks it up at once.
    pub fn for_page(folder: PathBuf, page: &str, mut context: mdm_ipc::Command) -> Self {
        let own = mdm_tools::pages::video_address(page);
        // An embedded player taken to the video's own page (#359): the site that embedded it
        // isn't that page's referer, and Dailymotion refuses its own page asked with one (403).
        if own != page {
            context.referer = None;
        }
        let page = own;
        VideoDialog {
            page: page.clone(),
            from_browser: page,
            context,
            auto_look: true,
            ..Self::blank(folder)
        }
    }

    /// The browser's context, while the address is still the one it came with.
    fn context(&self) -> mdm_ipc::Command {
        if self.page.trim() == self.from_browser {
            self.context.clone()
        } else {
            mdm_ipc::Command::default()
        }
    }

    /// The choices for a looked-up video: merged qualities when ffmpeg is there.
    fn set_video(&mut self, mut v: Video) {
        // No page reader for this site: the page's own title rather than a hash (#345), without
        // a media extension when the page is titled with the file's name (upstream 622).
        if let Some(title) = self
            .context
            .video_title
            .as_deref()
            .map(|t| without_media_extension(t.trim()))
            && !title.is_empty()
            && self.page.trim() == self.from_browser
            && mdm_tools::resolve::named_after_address(&v.title, &self.page)
        {
            v.title = title.to_owned();
        }
        let merge = self.tools.get_or_insert_with(tools).2.is_some();
        let choices = mdm_tools::resolve::choices(&v, merge);
        self.chosen = 0;
        self.rows = Default::default();
        self.error = choices
            .is_empty()
            .then(|| crate::i18n::t("No video this can download on that page").into());
        self.subs = mdm_tools::resolve::sub_choices(&v, "en");
        self.sub_pick = pick_for(&self.subs, &self.pref);
        self.video = Some((v, choices));
        self.name_from_choice();
    }

    /// The Subtitles list's choice as yt-dlp is asked for it.
    fn chosen_subs(&self) -> Subs {
        let embed = self.embed;
        let all = all_offered(&self.subs);
        match self.sub_pick {
            0 => Subs {
                embed,
                ..Default::default()
            },
            1 if all => Subs {
                langs: vec![ALL_UPLOADERS.into()],
                automatic: false,
                embed,
            },
            i => match self.subs.get(i - 1 - usize::from(all)) {
                Some(c) => Subs {
                    langs: vec![c.code.clone()],
                    automatic: c.automatic,
                    embed,
                },
                None => Subs {
                    embed,
                    ..Default::default()
                },
            },
        }
    }

    /// A looked-up video with subtitles in four languages, without yt-dlp or the network
    /// (`/show video-subs`: the window's check in every language, its screenshots).
    pub fn show_sample(&mut self) {
        const SAMPLE: &str = r#"{"title": "A conference talk about download managers, recorded in Lisbon",
            "duration": 1830, "language": "en", "formats": [
            {"format_id": "140", "ext": "m4a", "vcodec": "none", "acodec": "mp4a", "filesize": 29000000},
            {"format_id": "18", "ext": "mp4", "height": 360, "vcodec": "avc1", "acodec": "mp4a", "filesize": 61000000},
            {"format_id": "137", "ext": "mp4", "height": 1080, "vcodec": "avc1", "acodec": "none", "filesize": 412000000}],
            "subtitles": {"de": [{"name": "German"}], "pt-PT": [{"name": "Portuguese (Portugal)"}],
                "ar": [{"name": "Arabic"}]},
            "automatic_captions": {"en-orig": [{"name": "English (Original)"}], "en": [{"name": "English"}]}}"#;
        if let Some(v) = mdm_tools::resolve::parse(SAMPLE.as_bytes()) {
            self.page = "https://video.example/talk".into();
            self.looked_up = self.page.clone();
            self.auto_look = false;
            self.set_video(v);
        }
    }

    /// A page like YouTube's when yt-dlp has no JavaScript runtime (#351): videos and sound
    /// apart, nothing carrying both (`/show video-adaptive`).
    pub fn show_sample_adaptive(&mut self) {
        const SAMPLE: &str = r#"{"title": "A conference talk about download managers, recorded in Lisbon",
            "duration": 1830, "language": "en", "formats": [
            {"format_id": "140", "ext": "m4a", "vcodec": "none", "acodec": "mp4a", "filesize": 29000000},
            {"format_id": "137", "ext": "mp4", "height": 1080, "vcodec": "avc1", "acodec": "none", "filesize": 412000000},
            {"format_id": "136", "ext": "mp4", "height": 720, "vcodec": "avc1", "acodec": "none", "filesize": 201000000}]}"#;
        if let Some(v) = mdm_tools::resolve::parse(SAMPLE.as_bytes()) {
            self.page = "https://video.example/talk".into();
            self.looked_up = self.page.clone();
            self.auto_look = false;
            self.set_video(v);
        }
    }

    /// A listed playlist, as [`crate::playlistdlg::PlaylistView::sample`] makes it, its quality
    /// the one at position `pick` (`/show video-playlist:7`).
    pub fn show_sample_list(&mut self, pick: &str) {
        self.page = "https://video.example/playlist".into();
        self.looked_up = self.page.clone();
        self.auto_look = false;
        self.list = Some(crate::playlistdlg::PlaylistView::sample(pick));
    }

    /// The sample's choice whose label starts with `prefix` (any case), chosen and scrolled into
    /// view (`/show video-subs:flac`: a screenshot of a choice below the list's first rows).
    pub fn pick(&mut self, prefix: &str) {
        let prefix = prefix.to_lowercase();
        let found = self.video.as_ref().and_then(|(_, choices)| {
            choices.iter().position(|c| {
                let label = c.label.to_lowercase();
                label.starts_with(&prefix)
                    || label
                        .strip_prefix("audio only ")
                        .is_some_and(|l| l.starts_with(&prefix))
            })
        });
        if let Some(i) = found {
            self.chosen = i;
            // The list shows 8 rows.
            self.rows.show_row(i, 8);
            self.name_from_choice();
        }
    }

    /// Helpers..., as its button does (`/show video-helpers`).
    pub fn open_helpers(&mut self) {
        if self.helpers.is_none() {
            self.helpers = Some(crate::helpers::Programs::new());
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, rt: &tokio::runtime::Handle) -> Outcome<VideoOut> {
        if let Some(clip) = self.clip.take() {
            self.fill_from_clipboard(&clip);
        }
        // Results from the background work.
        let looked = self.lookup.as_ref().and_then(|s| s.lock().unwrap().take());
        if let Some(res) = looked {
            self.lookup = None;
            match res {
                Ok(Looked::Video(v)) => self.set_video(v),
                Ok(Looked::List(l)) => {
                    if let Some((exe, browser)) = self.looking.take() {
                        self.list = Some(crate::playlistdlg::PlaylistView::start(
                            &exe,
                            &self.looked_up,
                            browser,
                            l,
                        ));
                    }
                }
                Err(e) => {
                    // Whose cookies the look-up had (#229): the extension's, or (none came) the
                    // browser's Options names, or none.
                    let sent = self.looking.as_ref().is_some_and(|(_, b)| {
                        b.cookies.as_deref().is_some_and(|c| !c.trim().is_empty())
                    });
                    let store = (!sent).then(mdm_tools::proc::browser_cookies).flatten();
                    self.error = Some(lookup_error(&e, sent, store.as_ref()));
                }
            }
        }
        let installed = self
            .install
            .as_ref()
            .and_then(|(t, s)| s.lock().unwrap().take().map(|r| (*t, r)));
        if let Some((tool, res)) = installed {
            self.install = None;
            self.tools = None;
            match res {
                Err(e) => {
                    let what = if tool == Tool::YtDlp {
                        "yt-dlp"
                    } else {
                        "ffmpeg"
                    };
                    self.error = Some(crate::i18n::fmt(
                        // i18n: an error: {what} = a helper program (yt-dlp, ffmpeg), {error} = why
                        crate::i18n::t("{what} could not be installed: {error}"),
                        &[("what", what), ("error", &e)],
                    ));
                }
                // With ffmpeg the qualities that need merging appear.
                Ok(_) if tool == Tool::Ffmpeg => {
                    if let Some((v, _)) = self.video.take() {
                        self.set_video(v);
                    }
                }
                Ok(_) => {}
            }
        }
        // A playlist's question before adding many (#69): Yes goes ahead in the window's frame.
        if let Some(list) = &mut self.list
            && let Some(now) = list.answer(ctx)
        {
            self.confirmed = Some(now);
            ctx.request_repaint();
        }
        let out = window(
            ctx,
            "video-page",
            "Download video from a page",
            [VIDEO_W, VIDEO_BODY + winui::FOOTER_H],
            false,
            winui::BG(),
            |ui, o| self.body(ui, o, rt),
        );
        // A playlist's quality chosen, a list or a video looked up: Save to follows.
        self.follow_sound();
        // Helpers... open: on Close, the helpers are read again (another program, or MDM's own).
        if let Some(h) = &mut self.helpers
            && !matches!(h.show(ctx), Outcome::Open)
        {
            self.helpers = None;
            self.tools = None;
            if let Some((v, _)) = self.video.take() {
                self.set_video(v);
            }
        }
        let folder = crate::dialogs::resolve_path(&self.folder);
        let out = crate::dialogs::ensure_folder(ctx, &mut self.asking, out, &folder);
        match out {
            Outcome::Open => Outcome::Open,
            Outcome::Closed => Outcome::Closed,
            Outcome::Done(()) => self.added().map_or(Outcome::Closed, Outcome::Done),
        }
    }

    /// What Download Now or Later adds: a playlist's videos, each its own download, or the
    /// video looked up (none when there is nothing to add).
    fn added(&self) -> Option<VideoOut> {
        let folder = self.save_folder();
        let start_now = self.start_now.unwrap_or(true);
        if let Some(list) = &self.list {
            let merge = self.tools.as_ref().is_some_and(|t| t.2.is_some());
            return Some(VideoOut::Many(list.add(
                &folder,
                merge,
                start_now,
                &self.context(),
                self.category(),
            )));
        }
        let (Some((_, choices)), Ok(page)) = (&self.video, url::Url::parse(self.page.trim()))
        else {
            return None;
        };
        let c = choices.get(self.chosen.min(choices.len().checked_sub(1)?))?;
        let name = mdm_engine::naming::sanitize(self.name.trim());
        let chosen = self.chosen_subs();
        Some(VideoOut::One(VideoAdd {
            page,
            selector: c.selector.clone(),
            dest: folder.join(name),
            category: self.category(),
            size: c.size,
            start_now,
            context: self.context(),
            // Into the video only where its file type holds them (and with ffmpeg, which
            // VideoJob checks).
            subs: if mdm_tools::resolve::is_audio(c) {
                Subs::default()
            } else {
                Subs {
                    embed: chosen.embed && mdm_tools::resolve::holds_subtitles(c),
                    ..chosen.clone()
                }
            },
            remember: (!self.subs.is_empty()).then_some(chosen),
        }))
    }

    /// The choice (or the playlist's quality) saves only the sound; None while there is none.
    fn sound_only(&self) -> Option<bool> {
        if let Some(list) = &self.list {
            return Some(list.sound_only());
        }
        let (_, choices) = self.video.as_ref()?;
        choices.get(self.chosen).map(mdm_tools::resolve::is_audio)
    }

    /// The category the download is listed under, and whose folder "Change folder for this
    /// category on last selected" keeps: Music for a sound-only choice, else Video.
    pub fn category(&self) -> Category {
        if self.music_dir.is_some() && self.sound_only() == Some(true) {
            Category::Music
        } else {
            Category::Video
        }
    }

    /// Save to goes to the Music category's folder when the choice turns sound-only, back to
    /// Video's when it turns video again, but only while it holds the folder MDM put there: a
    /// folder the user typed or picked is theirs.
    fn follow_sound(&mut self) {
        let Some(sound) = self.sound_only() else {
            return;
        };
        if sound == self.sound {
            return;
        }
        self.sound = sound;
        let Some(music) = &self.music_dir else {
            return;
        };
        let (from, to) = if sound {
            (&self.video_dir, music)
        } else {
            (music, &self.video_dir)
        };
        if crate::dialogs::resolve_path(&self.folder) == crate::dialogs::resolve_path(from) {
            self.folder = to.clone();
        }
    }

    /// The Save to folder: the one a playlist's own folder goes under, too.
    pub fn save_folder(&self) -> PathBuf {
        crate::dialogs::resolve_path(&self.folder)
    }

    /// The file name, and Save to, follow the choice.
    fn name_from_choice(&mut self) {
        if let Some((v, choices)) = &self.video
            && let Some(c) = choices.get(self.chosen)
        {
            let stem = mdm_engine::naming::sanitize(&v.title);
            self.name = format!("{stem}.{}", c.ext);
        }
        self.follow_sound();
    }

    /// Install a helper in the background.
    fn install(
        &mut self,
        tool: Tool,
        dir: PathBuf,
        rt: &tokio::runtime::Handle,
        ctx: &egui::Context,
    ) {
        let slot: Slot<mdm_tools::Installed> = Arc::new(Mutex::new(None));
        let (out, ctx) = (slot.clone(), ctx.clone());
        let net = crate::settings::own_net();
        rt.spawn(async move {
            let (res, name) = match tool {
                Tool::YtDlp => (mdm_tools::install_latest(&dir, &net, None).await, "yt-dlp"),
                Tool::Ffmpeg => (
                    mdm_tools::ffmpeg::install_latest(&dir, &net, None).await,
                    "ffmpeg",
                ),
            };
            // What failed and what to do (upstream issue 816).
            *out.lock().unwrap() = Some(res.map_err(|e| crate::errors::tools_and_advice(&e, name)));
            ctx.request_repaint();
        });
        self.install = Some((tool, slot));
        self.error = None;
    }

    fn body(&mut self, ui: &mut Ui, o: Pos2, rt: &tokio::runtime::Handle) -> Outcome<()> {
        let p = ui.painter().clone();
        let (dir, ytdlp, ffmpeg) = self.tools.get_or_insert_with(tools).clone();
        // The labels' column (the rows under the list share it).
        let bx = winui::form(
            &p,
            LX,
            &[
                "Page address",
                "Save to",
                "File name",
                // i18n: label before the quality chosen for all of a playlist's videos
                "Quality",
                // i18n: label before the list of a video's subtitle languages
                "Subtitles",
            ],
            false,
        );
        let h = winui::CONTROL_H;
        winui::form_label(&p, o, LX, 16.0, h, "Page address", true);
        // Look up is as wide as its label in this language (#172); the address ends 8 px before.
        let look_w = winui::footer_button_width(&p, "Look up", true);
        let look_x = RIGHT - look_w;
        let edit = winui::edit(
            ui,
            r(o, bx, 16.0, look_x - 8.0 - bx, h),
            &mut self.page,
            true,
            false,
        );
        if !self.focused {
            edit.request_focus();
            self.focused = true;
        }
        let busy = self.lookup.is_some() || self.install.is_some();
        let page_ok =
            url::Url::parse(self.page.trim()).is_ok_and(|u| matches!(u.scheme(), "http" | "https"));
        let can_look = page_ok && !busy && ytdlp.is_some();
        let look = winui::button_icon(
            ui,
            r(o, look_x, 16.0, look_w, h),
            Some("gx-search"),
            "Look up",
            if can_look {
                ButtonKind::Default
            } else {
                ButtonKind::Disabled
            },
            false,
            None,
        )
        .clicked()
            || (edit.has_focus()
                && can_look
                && winui::enter_pressed(ui)
                && winui::take_key(ui, egui::Key::Enter));
        // Opened for a page: look it up as soon as yt-dlp is there.
        let look = look || (can_look && std::mem::take(&mut self.auto_look));
        if look
            && can_look
            && let Some(tool) = &ytdlp
        {
            // A typed or pasted address too (#359); the box shows what is looked up.
            self.page = mdm_tools::pages::video_address(self.page.trim());
            let slot: Slot<Looked> = Arc::new(Mutex::new(None));
            let (out, exe, page, ctx, rt) = (
                slot.clone(),
                tool.exe.clone(),
                self.page.trim().to_owned(),
                ui.ctx().clone(),
                rt.clone(),
            );
            let browser = from_browser(&self.context());
            self.looking = Some((exe.clone(), browser.clone()));
            std::thread::spawn(move || {
                let mut res =
                    mdm_tools::resolve::look_up(&exe, &page, &browser).map_err(|e| e.to_string());
                // A plain file: yt-dlp doesn't say its size, its server does (#320).
                if let Ok(Looked::Video(v)) = &mut res
                    && v.needs_size()
                    && let Some(size) = direct_size(&rt, &page, &browser)
                {
                    v.set_size(size);
                }
                *out.lock().unwrap() = Some(res);
                ctx.request_repaint();
            });
            self.lookup = Some(slot);
            self.looked_up = self.page.trim().to_owned();
            self.video = None;
            // A list being listed stops (dropped).
            self.list = None;
            self.error = None;
        }

        // The status line: a helper missing or installing / looking up / the title.
        let line_y = o.y + 70.0;
        let installing = self.install.as_ref().map(|(t, _)| *t);
        let offer = match (&ytdlp, &ffmpeg) {
            (None, _) => Some((
                Tool::YtDlp,
                // i18n: button: download and install the yt-dlp program
                crate::i18n::mark("Install yt-dlp"),
                crate::i18n::t(
                    "Video lookups use yt-dlp (about 23 MB, checked against its signature).",
                ),
            )),
            (Some(_), None) => Some((
                Tool::Ffmpeg,
                // i18n: button: download and install the ffmpeg program
                crate::i18n::mark("Install ffmpeg"),
                crate::i18n::t("The best qualities, MP3, Opus and FLAC need ffmpeg (about 80 MB)."),
            )),
            _ => None,
        };
        // The Install button, as wide as its label in this language, and the status line up to
        // 9 px before it (to the window's edge without one).
        let install_at = offer
            .as_ref()
            .filter(|(_, label, _)| !label.is_empty() && installing.is_none())
            .map(|(_, label, _)| {
                winui::buttons_to_right(&p, RIGHT, 8.0, &[(label, 90.0)], 200.0)[0]
            });
        let status_right = install_at.map_or(RIGHT, |(x, _)| x - 9.0);
        // #351: a page whose better videos need ffmpeg says so in place of its title (still in
        // File name): YouTube without a JavaScript runtime leaves nothing but the sound to offer
        // here, and the list alone gives no reason.
        let merge_note = self
            .video
            .as_ref()
            .filter(|(v, _)| ffmpeg.is_none() && mdm_tools::resolve::needs_merging(v))
            .and(offer.as_ref())
            .map(|(_, _, msg)| *msg);
        if let Some(t) = installing {
            crate::torrentdlg::spinner(ui, pos2(o.x + LX + 7.0, line_y - 4.0), 6.0);
            let msg = if t == Tool::YtDlp {
                crate::i18n::t("Downloading and verifying yt-dlp...")
            } else {
                crate::i18n::t("Downloading and verifying ffmpeg (about 80 MB)...")
            };
            text(&p, o.x + LX + 20.0, line_y, msg, winui::TEXT());
        } else if self.lookup.is_some() {
            crate::torrentdlg::spinner(ui, pos2(o.x + LX + 7.0, line_y - 4.0), 6.0);
            text(
                &p,
                o.x + LX + 20.0,
                line_y,
                crate::i18n::t("Looking the page up..."),
                winui::TEXT(),
            );
        } else if self.list.is_some() {
            // The list draws its own (PlaylistView::middle).
        } else if let Some(msg) = merge_note {
            winui::fit(
                ui,
                &p,
                o.x + LX,
                line_y,
                msg,
                winui::MUTED(),
                o.x + status_right,
                "video-offer",
            );
        } else if let Some((v, _)) = &self.video {
            let (rect, cut) = winui::text_fit(
                &p,
                o.x + LX,
                line_y,
                &v.title,
                winui::TEXT(),
                o.x + status_right,
            );
            if cut {
                winui::full_on_hover(ui, rect, "video-title", &v.title);
            }
        } else if let Some((_, _, msg)) = offer {
            winui::fit(
                ui,
                &p,
                o.x + LX,
                line_y,
                msg,
                winui::MUTED(),
                o.x + status_right,
                "video-offer",
            );
        }
        if let Some((tool, label, _)) = offer
            && let Some((x, w)) = install_at
            && winui::button(
                ui,
                r(o, x, 57.0, w, 21.0),
                label,
                ButtonKind::Normal,
                false,
                None,
            )
            .clicked()
            && let Some(dir) = dir.clone()
        {
            self.install(tool, dir, rt, ui.ctx());
        }
        // An error too long for its line, with no choices below it: read whole in their place.
        let error_in_list = self.video.is_none()
            && self.list.is_none()
            && self
                .error
                .as_ref()
                .is_some_and(|e| winui::text_width(&p, e) > RIGHT - LX);
        if let Some(e) = self.error.as_ref().filter(|_| !error_in_list) {
            let (rect, cut) =
                winui::text_fit(&p, o.x + LX, o.y + 86.0, e, winui::ERROR(), o.x + RIGHT);
            if cut {
                winui::full_on_hover(ui, rect, "video-error", e);
            }
        } else if self
            .video
            .as_ref()
            .and_then(|(_, c)| c.get(self.chosen))
            .is_some_and(|c| c.selector == mdm_tools::resolve::FLAC)
        {
            // The honest word on FLAC (#70): the site's sound is compressed with losses already.
            winui::fit(
                ui,
                &p,
                o.x + LX,
                o.y + 86.0,
                // i18n: a note in Download video while FLAC is chosen: FLAC keeps the site's (already lossy) sound exactly, it can't make it better, the file is only bigger
                crate::i18n::t(
                    "FLAC keeps the sound exactly, but it can't add quality the site's sound doesn't have: the file is only bigger.",
                ),
                winui::MUTED(),
                o.x + RIGHT,
                "video-flac",
            );
        }

        // A playlist's or a channel's videos (#69), or the video's choices.
        if let Some(list) = &mut self.list {
            // The status line stops short of an Install button beside it.
            list.middle(ui, o, installing.is_none().then_some(status_right));
        } else {
            // The choices: the shared row list.
            let list = r(o, LX, 94.0, RIGHT - LX, 150.0);
            let mut picked = None;
            let choices = self.video.as_ref().map_or(&[][..], |(_, c)| c.as_slice());
            self.rows.selected = Some(self.chosen).filter(|c| *c < choices.len());
            // i18n: what a screen reader calls the list of a video's formats (1080p, 720p, audio only…)
            self.rows.name = crate::i18n::t("Quality");
            let ev = winui::row_list(
                ui,
                "video-choices",
                list,
                choices.len(),
                17.0,
                &mut self.rows,
                |ui, clip, i, row, look| {
                    let c = &choices[i];
                    // The icon of the file this choice gives, then the format ("1080p mp4, 60
                    // fps", left to right); its size at the trailing edge.
                    let file = format!("video.{}", c.ext);
                    let kind = crate::fileicons::Kind::File(&file);
                    crate::fileicons::paint_at(
                        ui,
                        clip,
                        kind,
                        pos2(row.left() + 4.0, row.top()),
                        16.0,
                    );
                    let label = choice_label(&c.label);
                    let x = row.left() + 4.0 + crate::fileicons::INDENT;
                    text(clip, x, row.top() + 12.0, &label, look.ink);
                    let size = c
                        .size
                        // i18n: the video's file size is not known
                        .map_or(
                            crate::i18n::t("size unknown").into(),
                            crate::dialogs::dialog_size,
                        );
                    // Read as the format and its size (#95: the choices were blank to a screen reader
                    // and alike to UI Automation), in the user's language, as drawn.
                    look.name = format!("{label}, {size}");
                    winui::text_right(clip, row.right() - 6.0, row.top() + 12.0, &size, look.ink);
                },
            );
            if ev.focused
                && self.rows.type_to(ui, "video-choices", choices.len(), |i| {
                    choices[i].label.clone()
                })
            {
                picked = self.rows.selected;
            }
            if let Some(i) = ev.picked {
                picked = Some(i);
            }
            if let Some(i) = picked {
                self.chosen = i;
                self.name_from_choice();
            }
            // Wrapped in the empty list (#317).
            if let Some(e) = self.error.as_ref().filter(|_| error_in_list) {
                winui::paragraph(
                    &p.with_clip_rect(list),
                    list.left() + 8.0,
                    list.top() + 18.0,
                    e,
                    winui::ERROR(),
                    list.right() - 8.0,
                    16.0,
                );
            }
        }

        // The rows on the labels' column (`bx`, from the page address's row).
        let [save_y, name_y, subs_y] = ROWS_Y;
        winui::form_label(&p, o, LX, save_y, h, "Save to", true);
        winui::edit(
            ui,
            r(o, bx, save_y, RIGHT - h - 8.0 - bx, h),
            &mut self.folder,
            true,
            false,
        );
        // i18n: tooltip of the "..." after Save to: choose the folder
        if winui::icon_only_button(ui, r(o, RIGHT - h, save_y, h, h), "gx-dots", "Browse", true)
            .clicked()
        {
            let from = self.folder.clone();
            winui::start_pick(ui.ctx(), "video-folder", move || {
                rfd::FileDialog::new().set_directory(from).pick_folder()
            });
        }
        if let Some(d) = winui::picked(ui.ctx(), "video-folder") {
            self.folder = d.display().to_string();
        }
        if let Some(list) = &mut self.list {
            // One quality for every video; their names are their titles (no subtitles).
            winui::form_label(&p, o, LX, name_y, h, "Quality", true);
            list.quality_row(ui, r(o, bx, name_y, RIGHT - bx, h), ffmpeg.is_some());
        } else {
            winui::form_label(&p, o, LX, name_y, h, "File name", true);
            winui::edit(
                ui,
                r(o, bx, name_y, RIGHT - bx, h),
                &mut self.name,
                true,
                false,
            );
            let on = self.subtitles_row(ui, o, bx, subs_y, ffmpeg.is_some());
            winui::form_label(&p, o, LX, subs_y, h, "Subtitles", on);
        }

        // The choices belong to the address looked up: an edited address is looked up first
        // (it downloaded the new page with the old one's format and name).
        let stale =
            (self.video.is_some() || self.list.is_some()) && self.page.trim() != self.looked_up;
        let listing = self.list.as_ref().is_some_and(|l| l.listing());
        let chosen = match &self.list {
            Some(l) => l.chosen() > 0 && !listing,
            None => {
                self.video.as_ref().is_some_and(|(_, c)| !c.is_empty())
                    && !self.name.trim().is_empty()
            }
        };
        let ready = chosen && !busy && !stale;
        let kind = |default: bool| match (ready, default) {
            (false, _) => ButtonKind::Disabled,
            (true, true) => ButtonKind::Default,
            (true, false) => ButtonKind::Normal,
        };
        // Why Download Now is greyed: each reason that holds, one a line (as both showed before).
        // English keys here: the footer's tip translates what it is given, once.
        let why = [
            // i18n: why Download Now is greyed: the page address was changed after the look-up
            (stale, crate::i18n::mark("Look up the new address first")),
            // i18n: why Download Now is greyed: a playlist's videos are still being listed
            (
                listing,
                crate::i18n::mark("Wait for the list, or stop listing"),
            ),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, s)| *s)
        .collect::<Vec<_>>()
        .join("\n");
        // Escape answers the playlist's question first, when it's asked.
        let asking = self.list.as_ref().is_some_and(|l| l.asking());
        // The footer: Helpers at its start; Enter Download Now (while it can), Escape Cancel.
        let pressed = winui::footer(
            ui,
            o,
            VIDEO_W,
            VIDEO_BODY,
            &mut [
                // Where yt-dlp and ffmpeg come from: MDM's own copies or the person's (#51).
                // i18n: button in Download video: where MDM takes yt-dlp and ffmpeg from
                FooterItem::Button(FooterButton::new("Helpers...", ButtonKind::Normal).leading()),
                FooterItem::Button(
                    FooterButton::new("Download Now", kind(true))
                        .primary()
                        .enter(ready)
                        .tip(!why.is_empty(), &why),
                ),
                FooterItem::Button(
                    // i18n: button: add it now, start downloading later
                    FooterButton::new("Later", kind(false)),
                ),
                FooterItem::Button(FooterButton::new("Cancel", ButtonKind::Normal).escape(!asking)),
            ],
        );
        if pressed == Some(0) {
            self.open_helpers();
        }
        if pressed == Some(3) {
            return Outcome::Closed;
        }
        let (now, later) = (pressed == Some(1), pressed == Some(2));
        if ready && (now || later) {
            // A playlist of many asks first; Yes comes back as `confirmed`.
            if let Some(list) = &mut self.list
                && list.ask(now)
            {
                return Outcome::Open;
            }
            self.start_now = Some(now);
            return Outcome::Done(());
        }
        if let Some(now) = self.confirmed.take()
            && ready
        {
            self.start_now = Some(now);
            return Outcome::Done(());
        }
        Outcome::Open
    }
}

impl VideoDialog {
    /// Subtitles (#68): a language the video has, from the uploader or the site's automatic
    /// captions, saved beside the video (`<name>.<language>.srt`) or put into it (ffmpeg).
    /// The list starts at `bx`, after the labels' column, on the row at `y`; true when it can be
    /// chosen from (its label is greyed otherwise).
    fn subtitles_row(&mut self, ui: &mut Ui, o: Pos2, bx: f32, y: f32, ffmpeg: bool) -> bool {
        let choice = self
            .video
            .as_ref()
            .and_then(|(_, c)| c.get(self.chosen))
            .cloned();
        let audio = choice.as_ref().is_some_and(mdm_tools::resolve::is_audio);
        let on = !self.subs.is_empty() && !audio;
        let items = sub_items(&self.subs, self.video.is_some());
        let shown: Vec<&str> = items.iter().map(String::as_str).collect();
        let pick = if on { self.sub_pick } else { 0 };
        if let Some(i) = winui::combo_light(
            ui,
            r(o, bx, y, 200.0, winui::CONTROL_H),
            &shown,
            pick.min(shown.len().saturating_sub(1)),
            on,
            "video-subs",
        ) {
            self.sub_pick = i;
        }
        let chosen = on && self.sub_pick > 0;
        let holds = choice
            .as_ref()
            .is_some_and(mdm_tools::resolve::holds_subtitles);
        let mut embed = self.embed && chosen && ffmpeg && holds;
        let resp = winui::checkbox_until(
            ui,
            pos2(
                o.x + bx + 212.0,
                o.y + y + ((winui::CONTROL_H - 13.0) / 2.0).round(),
            ),
            &mut embed,
            // i18n: checkbox in Download video: subtitles go inside the video file instead of a separate .srt file beside it
            "Put them into the video",
            chosen && ffmpeg && holds,
            Some(o.x + RIGHT),
        );
        let resp = winui::why_disabled(
            resp,
            chosen && !ffmpeg,
            // i18n: why "Put them into the video" is greyed: ffmpeg (a helper program) is not installed
            "This needs ffmpeg",
        );
        winui::why_disabled(
            resp,
            chosen && ffmpeg && !holds,
            // i18n: why "Put them into the video" is greyed: the chosen file type can't hold subtitles
            "Only MP4 and MKV videos can hold subtitles",
        );
        if chosen && ffmpeg && holds {
            self.embed = embed;
        }
        on
    }
}

/// "All the uploader's" is offered when they gave two languages or more.
fn all_offered(subs: &[SubChoice]) -> bool {
    subs.iter().filter(|s| !s.automatic).count() >= 2
}

/// The Subtitles list (English where it is MDM's words; combo_light shows them translated):
/// None, All the uploader's, then each language, the automatic ones marked.
fn sub_items(subs: &[SubChoice], looked_up: bool) -> Vec<String> {
    let mut out = vec![if looked_up && subs.is_empty() {
        // i18n: in the Subtitles list of Download video: the video has no subtitles
        crate::i18n::mark("None on this page").to_owned()
    } else {
        // i18n: in the Subtitles list of Download video: download no subtitles
        crate::i18n::mark("None").to_owned()
    }];
    if all_offered(subs) {
        // i18n: in the Subtitles list of Download video: every language the video's uploader gave
        out.push(crate::i18n::mark("All the uploader's").to_owned());
    }
    for s in subs {
        out.push(if s.automatic {
            crate::i18n::fmt(
                // i18n: in the Subtitles list of Download video: {language} = a language's name; the site made these subtitles automatically (speech recognition)
                crate::i18n::t("{language}, automatic"),
                &[("language", &s.name)],
            )
        } else {
            s.name.clone()
        });
    }
    out
}

/// The entry of the Subtitles list that `pref` (the last video's choice) stands for in this
/// video's: the same language (the uploader's before the automatic one, then any variant of
/// it: `pt` for `pt-BR`), every uploader's language again, or none.
fn pick_for(subs: &[SubChoice], pref: &Subs) -> usize {
    let all = all_offered(subs);
    let first = 1 + usize::from(all);
    let Some(want) = pref.langs.first() else {
        return 0;
    };
    if want == ALL_UPLOADERS {
        return if all {
            1
        } else {
            subs.iter()
                .position(|s| !s.automatic)
                .map_or(0, |i| i + first)
        };
    }
    let base = |c: &str| c.split(['-', '_']).next().unwrap_or(c).to_ascii_lowercase();
    subs.iter()
        .position(|s| s.code == *want && s.automatic == pref.automatic)
        .or_else(|| subs.iter().position(|s| s.code == *want))
        .or_else(|| subs.iter().position(|s| base(&s.code) == base(want)))
        .map_or(0, |i| i + first)
}

/// yt-dlp's view of the browser's request context.
pub fn from_browser(c: &mdm_ipc::Command) -> mdm_tools::proc::FromBrowser {
    mdm_tools::proc::FromBrowser {
        referer: c.referer.clone(),
        origin: c.origin.clone(),
        user_agent: c.user_agent.clone(),
        cookies: c.cookies.clone(),
        jar: c.cookie_jar.clone(),
    }
}

/// The size the server says for a plain file at `page`, asked as the browser would (its Referer,
/// its Origin, its cookies, its user agent). None when it won't say: the window then says "size
/// unknown", as it always did. A cookie jar (a whole site's cookies, for yt-dlp) isn't used: the
/// header is.
fn direct_size(
    rt: &tokio::runtime::Handle,
    page: &str,
    browser: &mdm_tools::proc::FromBrowser,
) -> Option<u64> {
    use reqwest::header::{COOKIE, HeaderMap, HeaderValue, ORIGIN, REFERER};
    let url = url::Url::parse(page).ok()?;
    let mut headers = HeaderMap::new();
    for (name, value) in [
        (REFERER, &browser.referer),
        (ORIGIN, &browser.origin),
        (COOKIE, &browser.cookies),
    ] {
        if let Some(v) = value.as_deref().and_then(|v| HeaderValue::from_str(v).ok()) {
            headers.insert(name, v);
        }
    }
    let mut opts = crate::settings::own_net();
    if let Some(agent) = &browser.user_agent {
        opts.speak_as(agent.clone());
    }
    rt.block_on(mdm_engine::probe(url, headers, &opts))
        .ok()?
        .size
}

/// A browser's name as people know it, from yt-dlp's (`chrome`: Chrome).
pub fn browser_label(name: &str) -> &'static str {
    match name {
        "firefox" => "Firefox",
        "chrome" => "Chrome",
        "edge" => "Edge",
        "brave" => "Brave",
        "opera" => "Opera",
        "vivaldi" => "Vivaldi",
        "chromium" => "Chromium",
        "whale" => "Whale",
        _ => "?",
    }
}

/// yt-dlp's reason in plain words where it is one users meet often; otherwise its own text.
/// `sent`: the browser's extension passed the page's cookies; `store`: else, the browser whose
/// cookies yt-dlp read (Options → Sites Logins → Cookies for video sites, #229).
fn lookup_error(e: &str, sent: bool, store: Option<&mdm_tools::proc::BrowserCookies>) -> String {
    let lower = e.to_ascii_lowercase();
    let browser = store.map_or("?", |b| browser_label(&b.browser));
    let named = |text: &str| crate::i18n::fmt(crate::i18n::t(text), &[("browser", browser)]);
    // The browser's cookies yt-dlp couldn't read come first: the site's "sign in" that follows
    // is only their result.
    if store.is_some() && mdm_tools::proc::cookie_trouble(&lower) {
        return if lower.contains("could not copy chrome cookie database") {
            // i18n: the Download video window: yt-dlp couldn't read the cookies of the browser chosen in Options (Chrome, Edge...), which holds them while it runs; {browser} = its name
            named(
                "yt-dlp can't read {browser}'s cookies while {browser} is open. Close {browser} and look the page up again, or choose Firefox in Options → Sites Logins.",
            )
        } else if lower.contains("could not find") {
            // i18n: the Download video window: the browser chosen in Options has no cookies on this computer (not installed, or another profile); {browser} = its name
            named(
                "yt-dlp found no cookies of {browser} on this computer. Check the browser and its profile in Options → Sites Logins.",
            )
        } else if lower.contains("decrypt") || lower.contains("unknown cookie version") {
            // i18n: the Download video window: Chrome 127 and later (and Edge, Brave) on Windows encrypt their cookies so only they can read them; {browser} = its name
            named(
                "{browser} keeps its cookies locked to itself on Windows, so yt-dlp can't read them. Choose Firefox in Options → Sites Logins, or use the Download video button over the video in {browser}.",
            )
        } else {
            crate::i18n::fmt(
                // i18n: the Download video window: yt-dlp couldn't read the cookies of the browser chosen in Options; {browser} = its name, {error} = yt-dlp's words
                crate::i18n::t("yt-dlp couldn't read {browser}'s cookies: {error}"),
                &[("browser", browser), ("error", e)],
            )
        };
    }
    // A site's bot check turned yt-dlp away (#317).
    let plain = if lower.contains("anti-bot") {
        if store.is_some() {
            // i18n: the Download video window: the site's bot check (Cloudflare) turned yt-dlp away though it had the cookies of the browser chosen in Options; {browser} = its name
            return named(
                "The site's bot check (Cloudflare) refused yt-dlp, even with {browser}'s cookies. Use the Download video button over the video in {browser}, or try the page again later.",
            );
        }
        if sent {
            // i18n: the Download video window: the site's bot check (Cloudflare) turned yt-dlp away though the browser's extension passed the page's cookies
            "The site's bot check (Cloudflare) refused yt-dlp, even with your browser's cookies for this page. Try again later, or from another browser."
        } else {
            // i18n: the Download video window: the site's bot check (Cloudflare) turned yt-dlp away and no browser cookies were used
            "The site's bot check (Cloudflare) refused yt-dlp. If the page opens in your browser, choose that browser in Options → Sites Logins → Cookies for video sites and look the page up again, or use the Download video button over the video."
        }
    } else if lower.contains("drm") {
        // i18n: the Download video window: the page's video is copy-protected
        "This video is protected (DRM): no tool can save it."
    } else if lower.contains("unsupported url") {
        // i18n: the Download video window: yt-dlp found no video on the page
        "No video found on this page. Play the video, then use the Download video button over it."
    } else if lower.contains("sign in")
        || lower.contains("login")
        || lower.contains("private video")
    {
        if store.is_some() {
            // i18n: the Download video window: the site asked for a sign-in although yt-dlp had the cookies of the browser chosen in Options; {browser} = its name
            return named(
                "The site wants you signed in, even with {browser}'s cookies. Sign in to it in {browser}, then look the page up again.",
            );
        }
        if sent {
            // i18n: the Download video window: the site asked for a sign-in although the browser's extension passed the page's cookies
            "The site wants you signed in, even with your browser's cookies for this page. Sign in to it in your browser, then try again."
        } else {
            // i18n: the Download video window: the video needs the user to be signed in
            "This video needs you to be signed in. Use the Download video button over it in your browser, which passes your sign-in on, or choose your browser in Options → Sites Logins → Cookies for video sites."
        }
    } else {
        return e.to_owned();
    };
    crate::i18n::t(plain).to_owned()
}

/// A choice's label in the user's language: the audio choice's words translated, the rest (a
/// resolution, a type) as they are, kept left to right.
pub(crate) fn choice_label(label: &str) -> String {
    if let Some(kind) = label.strip_prefix(mdm_tools::resolve::AUDIO_ORIGINAL) {
        return crate::i18n::fmt(
            // i18n: a choice in the Download video window: only the sound, the site's own file as it is (not converted); {type} is WEBM, M4A…
            crate::i18n::t("Audio only, original {type}"),
            &[("type", &crate::i18n::ltr(kind.trim()))],
        );
    }
    match label.strip_prefix(mdm_tools::resolve::AUDIO_ONLY) {
        Some(kind) => crate::i18n::fmt(
            // i18n: a choice in the Download video window: only the sound; {type} is MP3, M4A…
            crate::i18n::t("Audio only {type}"),
            &[("type", &crate::i18n::ltr(kind.trim()))],
        ),
        None => crate::i18n::ltr(label),
    }
}

/// `title` without a video or sound file's extension at its end (`Episode 31.mp4` → `Episode
/// 31`): a page that shows one file (Dropbox's, a server's listing) is often titled with the
/// file's name, and the choice adds its own type (it was `Episode 31.mp4.mp4`, upstream 622).
/// As it is otherwise, or when nothing would be left.
fn without_media_extension(title: &str) -> &str {
    match title.rsplit_once('.') {
        Some((stem, ext))
            if !stem.trim().is_empty()
                && mdm_tools::video::MEDIA
                    .iter()
                    .any(|m| m.eq_ignore_ascii_case(ext)) =>
        {
            stem.trim_end()
        }
        _ => title,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Upstream 622: a Dropbox page is titled with the file's name, extension and all; the video
    /// named by it got the choice's extension once more (`Episode 31.mp4.mp4`).
    #[test]
    fn a_page_titled_with_the_file_name_names_it_once() {
        for (title, want) in [
            ("Episode 31.mp4", "Episode 31"),
            ("Episode 31.MKV", "Episode 31"),
            ("Episode 31.m4a", "Episode 31"),
            // Not a media file's extension: a title as it is.
            ("Episode 31. The end", "Episode 31. The end"),
            ("Report v2.0", "Report v2.0"),
            (".mp4", ".mp4"),
        ] {
            assert_eq!(without_media_extension(title), want, "{title}");
        }
    }

    /// #359: a Dailymotion player embedded in another site is looked up at the video's own page,
    /// without that site as its Referer (Dailymotion answers 403); any other page keeps it.
    #[test]
    fn an_embedded_player_taken_to_its_page_drops_the_embedding_sites_referer() {
        let from = |page: &str| {
            let context = mdm_ipc::Command {
                referer: Some("https://news.test/article".into()),
                ..Default::default()
            };
            VideoDialog::for_page(PathBuf::from("v"), page, context)
        };
        let embed = from("https://geo.dailymotion.com/player/x1abc.html?video=x9xyz12");
        assert_eq!(embed.page, "https://www.dailymotion.com/video/x9xyz12");
        assert_eq!(embed.context().referer, None);
        let plain = from("https://player.vimeo.test/video/1");
        assert_eq!(
            plain.context().referer.as_deref(),
            Some("https://news.test/article")
        );
    }

    /// #317: Cloudflare's check in plain words, saying what to try for each kind of cookies had.
    #[test]
    fn a_bot_check_says_what_to_try() {
        let check = "yt-dlp failed: [generic] Got HTTP Error 403 caused by Cloudflare anti-bot challenge; try again with --extractor-args \"generic:impersonate\"";
        let none = lookup_error(check, false, None);
        assert!(none.contains("bot check"), "{none}");
        assert!(none.contains("Cookies for video sites"), "{none}");
        let sent = lookup_error(check, true, None);
        assert!(sent.contains("even with your browser's cookies"), "{sent}");
        let firefox = mdm_tools::proc::BrowserCookies::new("firefox", "").unwrap();
        let chrome_had = lookup_error(check, false, Some(&firefox));
        assert!(
            chrome_had.contains("even with Firefox's cookies"),
            "{chrome_had}"
        );
        assert_eq!(
            lookup_error("yt-dlp failed: Video unavailable", false, None),
            "yt-dlp failed: Video unavailable"
        );
    }

    /// #229: yt-dlp's refusals and its trouble with a browser's cookies, in MDM's words, saying
    /// which cookies it had.
    #[test]
    fn cookie_and_sign_in_errors_in_plain_words() {
        let bot = "yt-dlp failed: [youtube] A1WG3x-Ebpk: Sign in to confirm you're not a bot. Use --cookies-from-browser or --cookies for the authentication.";
        assert!(lookup_error(bot, false, None).contains("Cookies for video sites"));
        assert!(lookup_error(bot, true, None).contains("even with your browser's cookies"));
        let firefox = mdm_tools::proc::BrowserCookies::new("firefox", "").unwrap();
        let chrome = mdm_tools::proc::BrowserCookies::new("chrome", "").unwrap();
        assert_eq!(
            lookup_error(bot, false, Some(&firefox)),
            "The site wants you signed in, even with Firefox's cookies. Sign in to it in Firefox, then look the page up again."
        );
        // Chrome's app-bound cookies: only a warning, kept on the error (proc::with_cookie_trouble).
        let v20 = format!("{bot} (unknown cookie version: \"b'v20'\")");
        let said = lookup_error(&v20, false, Some(&chrome));
        assert!(
            said.starts_with("Chrome keeps its cookies locked"),
            "{said}"
        );

        // A key that couldn't be read: the same.
        let v11 = format!("{bot} (cannot decrypt v11 cookies: no key found)");
        let said = lookup_error(&v11, false, Some(&chrome));
        assert!(
            said.starts_with("Chrome keeps its cookies locked"),
            "{said}"
        );

        assert!(
            lookup_error(
                "yt-dlp failed: failed to load cookies (Could not copy Chrome cookie database. See  https://github.com/yt-dlp/yt-dlp/issues/7271  for more info)",
                false,
                Some(&chrome)
            )
            .starts_with("yt-dlp can't read Chrome's cookies while Chrome is open")
        );
        assert!(
            lookup_error(
                "yt-dlp failed: failed to load cookies (could not find firefox cookies database in 'C:\\p')",
                false,
                Some(&firefox)
            )
            .starts_with("yt-dlp found no cookies of Firefox")
        );
        // Anything else stays yt-dlp's own words.
        assert_eq!(
            lookup_error("yt-dlp failed: Video unavailable", false, Some(&firefox)),
            "yt-dlp failed: Video unavailable"
        );
    }

    fn sub(code: &str, automatic: bool) -> SubChoice {
        SubChoice {
            code: code.into(),
            name: code.into(),
            automatic,
        }
    }

    fn want(langs: &[&str], automatic: bool) -> Subs {
        Subs {
            langs: langs.iter().map(|l| l.to_string()).collect(),
            automatic,
            embed: false,
        }
    }

    /// The Subtitles list (#68): None, All the uploader's when they gave two or more, then each
    /// language, the automatic ones marked.
    #[test]
    fn subtitle_list_entries() {
        let subs = [sub("de", false), sub("fr", false), sub("en", true)];
        assert_eq!(
            sub_items(&subs, true),
            ["None", "All the uploader's", "de", "fr", "en, automatic"]
        );
        assert_eq!(sub_items(&subs[2..], true), ["None", "en, automatic"]);
        assert_eq!(sub_items(&[], true), ["None on this page"]);
        assert_eq!(sub_items(&[], false), ["None"]);
    }

    /// The last video's choice finds its place in the next video's list: the same language
    /// (the uploader's first), a variant of it, every uploader's again, or none.
    #[test]
    fn the_last_choice_is_offered_again() {
        let subs = [sub("de", false), sub("pt-BR", false), sub("en", true)];
        assert_eq!(pick_for(&subs, &Subs::default()), 0);
        assert_eq!(pick_for(&subs, &want(&[ALL_UPLOADERS], false)), 1);
        assert_eq!(pick_for(&subs, &want(&["de"], false)), 2);
        assert_eq!(pick_for(&subs, &want(&["pt"], false)), 3, "pt for pt-BR");
        assert_eq!(
            pick_for(&subs, &want(&["en"], false)),
            4,
            "the automatic one"
        );
        assert_eq!(pick_for(&subs, &want(&["ja"], false)), 0);
        // One uploader's language only: "all" stands for it.
        let one = [sub("fr", false), sub("en", true)];
        assert_eq!(pick_for(&one, &want(&[ALL_UPLOADERS], false)), 1);
    }

    /// Each entry stands for what yt-dlp is asked.
    #[test]
    fn entries_ask_for_their_subtitles() {
        let mut d = VideoDialog::blank(PathBuf::from("."));
        d.subs = vec![sub("de", false), sub("fr", false), sub("en-orig", true)];
        d.embed = true;
        let mut at = |i: usize| {
            d.sub_pick = i;
            d.chosen_subs()
        };
        assert!(at(0).langs.is_empty());
        assert_eq!(at(1).langs, [ALL_UPLOADERS]);
        assert_eq!(at(3).langs, ["fr"]);
        let auto = at(4);
        assert_eq!(auto.langs, ["en-orig"]);
        assert!(auto.automatic && auto.embed);
    }

    /// A page's address from the clipboard comes a frame or more after the window opens: it
    /// fills an empty box only, and only with a web address.
    #[test]
    fn the_clipboard_fills_only_an_empty_page_box() {
        let video = std::env::temp_dir().join("mdm-688").join("Video");
        let mut d = VideoDialog::blank(video.clone());
        d.fill_from_clipboard(
            " https://video.example/talk 
",
        );
        assert_eq!(d.page, "https://video.example/talk");
        d.fill_from_clipboard("https://video.example/other");
        assert_eq!(
            d.page, "https://video.example/talk",
            "not over what is there"
        );
        let mut d = VideoDialog::blank(video);
        d.fill_from_clipboard("ftp://files.example/a.mp4");
        assert_eq!(d.page, "");
    }

    /// The Video and Music categories' folders (not made: only their names are compared).
    fn dirs() -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join("mdm-611");
        (base.join("Video"), base.join("Music"))
    }

    /// Download video as the app opens it (Save to: Video's folder, Music's offered), a video
    /// with a sound-only choice and video ones looked up, without yt-dlp or ffmpeg.
    fn looked_up(music: bool) -> VideoDialog {
        const SAMPLE: &str = r#"{"title": "A talk", "duration": 60, "formats": [
            {"format_id": "140", "ext": "m4a", "vcodec": "none", "acodec": "mp4a", "filesize": 2900},
            {"format_id": "18", "ext": "mp4", "height": 360, "vcodec": "avc1", "acodec": "mp4a", "filesize": 6100}]}"#;
        let (video, music_dir) = dirs();
        let mut d = VideoDialog::for_page(video, "https://video.example/talk", Default::default())
            .with_music_folder(music.then_some(music_dir));
        // No helpers looked for on this computer: no merged qualities.
        d.tools = Some((None, None, None));
        d.looked_up = d.page.clone();
        d.set_video(mdm_tools::resolve::parse(SAMPLE.as_bytes()).unwrap());
        d
    }

    /// The choices' rows: (a sound-only one, a video one).
    fn rows(d: &VideoDialog) -> (usize, usize) {
        let choices = &d.video.as_ref().unwrap().1;
        let sound = choices.iter().position(mdm_tools::resolve::is_audio);
        let video = choices
            .iter()
            .position(|c| !mdm_tools::resolve::is_audio(c));
        (sound.unwrap(), video.unwrap())
    }

    /// A row clicked in the choices list, as the list does it.
    fn choose(d: &mut VideoDialog, row: usize) {
        d.chosen = row;
        d.name_from_choice();
    }

    /// What Download Now adds for a single video: (its folder, its category).
    fn one(d: &VideoDialog) -> (PathBuf, Category) {
        match d.added() {
            Some(VideoOut::One(add)) => (add.dest.parent().unwrap().to_owned(), add.category),
            _ => panic!("a video is added"),
        }
    }

    /// Upstream issue 611: a sound-only choice saves in the Music category's folder and is
    /// listed under Music; a video choice again goes back to Video's.
    #[test]
    fn a_sound_only_choice_saves_in_music_and_back() {
        let (video, music) = dirs();
        let mut d = looked_up(true);
        let (sound, film) = rows(&d);
        choose(&mut d, film);
        assert_eq!(d.save_folder(), video);
        assert_eq!(one(&d), (video.clone(), Category::Video));
        choose(&mut d, sound);
        assert_eq!(
            d.save_folder(),
            music,
            "Save to follows a sound-only choice"
        );
        assert_eq!(one(&d), (music.clone(), Category::Music));
        assert_eq!(d.category(), Category::Music, "the folder kept is Music's");
        choose(&mut d, film);
        assert_eq!(d.save_folder(), video, "and goes back for a video");
        assert_eq!(one(&d), (video, Category::Video));
    }

    /// Upstream issue 611: a folder the user typed or picked stays whatever is chosen; a
    /// sound-only download into it is still listed under Music.
    #[test]
    fn a_folder_the_user_chose_stays() {
        let mut d = looked_up(true);
        let (sound, film) = rows(&d);
        let own = std::env::temp_dir().join("mdm-611").join("Talks");
        d.folder = own.display().to_string();
        choose(&mut d, sound);
        assert_eq!(d.save_folder(), own);
        assert_eq!(one(&d), (own.clone(), Category::Music));
        choose(&mut d, film);
        assert_eq!(d.save_folder(), own);
        assert_eq!(one(&d), (own, Category::Video));
    }

    /// The Music category deleted: sound-only goes where video does, as before.
    #[test]
    fn without_a_music_category_sound_goes_as_video() {
        let (video, _) = dirs();
        let mut d = looked_up(false);
        let (sound, _) = rows(&d);
        choose(&mut d, sound);
        assert_eq!(d.save_folder(), video);
        assert_eq!(one(&d), (video, Category::Video));
    }

    /// Upstream issue 611: a playlist downloaded as sound only goes into the Music category's
    /// folder (then the list's own folder) and each of its videos is listed under Music; a
    /// video quality, Video's.
    #[test]
    fn a_playlist_as_sound_saves_in_music() {
        let (video, music) = dirs();
        let many = |d: &VideoDialog| match d.added() {
            Some(VideoOut::Many(add)) => (add.folder.parent().unwrap().to_owned(), add.category),
            _ => panic!("a playlist is added"),
        };
        // "Audio only, original" (7), then "Up to 1080p" (3).
        for (pick, folder, category) in
            [("7", music, Category::Music), ("3", video, Category::Video)]
        {
            let mut d = looked_up(true);
            d.show_sample_list(pick);
            // As each frame of the window does.
            d.follow_sound();
            assert_eq!(d.save_folder(), folder, "quality {pick}");
            assert_eq!(many(&d), (folder, category), "quality {pick}");
        }
    }
}
