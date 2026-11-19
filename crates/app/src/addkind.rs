//! What a line typed, pasted or dropped into Add (the toolbar's Add window, the paste bar, a
//! drop on the main window) is, and so where it goes next: the one place that tells a file's
//! address from a magnet link, a .torrent, a video's page, an image gallery or a batch pattern.
//! Pure (the disk is asked through `is_file`), so every rule is tested here.
//!
//! - [`Route`], [`route_of`]: where an accepted address goes (Add URL's order, plan 06).
//! - [`Kind`], [`Found`], [`scan`]: each line of a text, what it is or why it is not one.
//! - [`Bar`], [`bar_kind`]: the paste bar's one-line reading of the same rules.

use std::path::Path;

use url::Url;

/// Where an accepted address goes, in Add URL's order (plan 06): `route_of` picks it,
/// `App::route_new_url` opens it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// A magnet link: Add torrent.
    Magnet,
    /// A .torrent's address (#39): Add torrent fetches it.
    TorrentUrl,
    /// A video's page (YouTube, Vimeo…): the video window.
    VideoPage,
    /// An image gallery's page (#58): gallery-dl.
    GalleryPage,
    /// Anything else: the duplicate check, then Download File Info.
    File,
}

/// The route for an accepted address: the first check that holds wins, in this order.
pub fn route_of(url: &Url) -> Route {
    if url.scheme() == "magnet" {
        Route::Magnet
    } else if crate::torrents::is_torrent_url(url, None, None) {
        Route::TorrentUrl
    } else if mdm_tools::pages::is_video_page(url) {
        Route::VideoPage
    } else if mdm_tools::pages::is_gallery_page(url) {
        Route::GalleryPage
    } else {
        Route::File
    }
}

/// What one line is, and so the flow it goes into.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A file's address (http, https, ftp): Download File Info.
    Link(Url),
    /// A magnet link: Add torrent, which asks the peers what it holds.
    Magnet(Url),
    /// A .torrent's web address: Add torrent fetches it.
    TorrentUrl(Url),
    /// A .torrent file on this computer: Add torrent reads it.
    TorrentFile(String),
    /// A video's page: the video window looks it up.
    Video(Url),
    /// An image gallery's page: gallery-dl.
    Gallery(Url),
    /// An address with a `*` where numbers or letters go: Batch download.
    Batch(String),
}

/// One line read: as written (trimmed), and what it is or why it is not one (English, marked,
/// shown through `crate::i18n::t`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub text: String,
    pub kind: Result<Kind, &'static str>,
}

impl Kind {
    /// The address an item of this kind goes on with (None: a file on this computer, a
    /// pattern).
    #[cfg(test)]
    pub fn url(&self) -> Option<&Url> {
        match self {
            Kind::Link(u)
            | Kind::Magnet(u)
            | Kind::TorrentUrl(u)
            | Kind::Video(u)
            | Kind::Gallery(u) => Some(u),
            Kind::TorrentFile(_) | Kind::Batch(_) => None,
        }
    }

    /// The chip's word (English, marked): what was recognised.
    pub fn chip(&self) -> &'static str {
        match self {
            // i18n: Add: the chip of a line that is a file's web address
            Kind::Link(_) => crate::i18n::mark("Link"),
            // i18n: Add: the chip of a magnet link
            Kind::Magnet(_) => crate::i18n::mark("Magnet"),
            // i18n: Add: the chip of a .torrent file (on the web or on this computer)
            Kind::TorrentUrl(_) | Kind::TorrentFile(_) => crate::i18n::mark("Torrent file"),
            // i18n: Add: the chip of a video's page
            Kind::Video(_) => crate::i18n::mark("Video"),
            // i18n: Add: the chip of an image gallery's page
            Kind::Gallery(_) => crate::i18n::mark("Gallery"),
            // i18n: Add: the chip of an address with a * (a batch download)
            Kind::Batch(_) => crate::i18n::mark("Batch"),
        }
    }

    /// The chip's icon (a Graphite need).
    pub fn icon(&self) -> &'static str {
        match self {
            Kind::Link(_) => "gx-link",
            Kind::Magnet(_) => "tb-add-torrent",
            Kind::TorrentUrl(_) | Kind::TorrentFile(_) => "ft-torrent",
            Kind::Video(_) => "cat-video",
            Kind::Gallery(_) => "ft-image",
            Kind::Batch(_) => "gx-list-numbers",
        }
    }

    /// A link's chip is the web chip's colour (the download list's Web); the others the
    /// torrent chip's.
    pub fn web(&self) -> bool {
        matches!(self, Kind::Link(_))
    }

    /// What comes next for it (English, marked), shown at the row's end; a link's is its
    /// category, which `next_link` writes.
    pub fn next(&self) -> &'static str {
        match self {
            Kind::Link(_) => "",
            // i18n: Add: what comes after Add for a magnet link or a .torrent
            Kind::Magnet(_) | Kind::TorrentUrl(_) | Kind::TorrentFile(_) => {
                crate::i18n::mark("Choose its files next")
            }
            // i18n: Add: what comes after Add for a video's page
            Kind::Video(_) => crate::i18n::mark("Choose its quality next"),
            // i18n: Add: what comes after Add for an image gallery's page
            Kind::Gallery(_) => crate::i18n::mark("Choose its pictures next"),
            // i18n: Add: what comes after Add for an address with a *
            Kind::Batch(_) => crate::i18n::mark("Set its range next"),
        }
    }

    /// The name a row shows: a link's file name, a magnet's `dn`, a file's name, else the
    /// address as written.
    pub fn name(&self, text: &str) -> String {
        match self {
            Kind::Link(u) | Kind::TorrentUrl(u) => mdm_engine::naming::file_name(None, u),
            Kind::Magnet(u) => u
                .query_pairs()
                .find(|(k, _)| k == "dn")
                .map_or_else(|| text.to_owned(), |(_, v)| v.into_owned()),
            Kind::TorrentFile(p) => Path::new(p)
                .file_name()
                .map_or_else(|| p.clone(), |n| n.to_string_lossy().into_owned()),
            _ => text.to_owned(),
        }
    }
}

/// The host an address names, without `www.` (a chip's "Link · example.org").
pub fn site(url: &Url) -> String {
    let host = url.host_str().unwrap_or_default();
    host.strip_prefix("www.").unwrap_or(host).to_owned()
}

/// A Windows path to a .torrent (`C:\…`, `\\server\…`), or a `file:` address of one.
fn torrent_path(line: &str) -> Option<String> {
    let line = line.trim_matches('"');
    if !line.to_ascii_lowercase().ends_with(".torrent") {
        return None;
    }
    if let Ok(u) = Url::parse(line)
        && u.scheme() == "file"
    {
        return u.to_file_path().ok().map(|p| p.display().to_string());
    }
    let b = line.as_bytes();
    let drive =
        b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && matches!(b[2], b'\\' | b'/');
    (drive || line.starts_with("\\\\")).then(|| line.to_owned())
}

/// One whole line: what it is, or why not.
fn read_line(line: &str, is_file: &dyn Fn(&Path) -> bool) -> Result<Kind, &'static str> {
    if let Some(path) = torrent_path(line) {
        return if is_file(Path::new(&path)) {
            Ok(Kind::TorrentFile(path))
        } else {
            // i18n: Add: a line that names a .torrent file that is not there
            Err(crate::i18n::mark("No such file on this computer"))
        };
    }
    if line.contains('*') {
        // The pattern with its * as a digit: an address Batch download takes.
        return if crate::dialogs::accept_address(&line.replacen('*', "1", 1))
            .is_some_and(|u| u.scheme() != "magnet")
        {
            Ok(Kind::Batch(line.to_owned()))
        } else {
            Err(crate::i18n::mark("Not an address"))
        };
    }
    let Some(url) = crate::dialogs::accept_address(line) else {
        let scheme = line.split_once(':').map(|(s, _)| s.to_ascii_lowercase());
        return Err(match scheme.as_deref() {
            // i18n: Add: a line with an address of a kind Grabbit doesn't download (mailto:, file:…)
            Some(s)
                if !s.is_empty()
                    && s.chars().all(|c| c.is_ascii_alphanumeric())
                    && !line.contains(' ') =>
            {
                crate::i18n::mark("Grabbit doesn't download this kind of address")
            }
            // i18n: Add: a line that is not a web address, a magnet link or a .torrent file
            _ => crate::i18n::mark("Not an address"),
        });
    };
    Ok(match route_of(&url) {
        Route::Magnet => Kind::Magnet(url),
        Route::TorrentUrl => Kind::TorrentUrl(url),
        Route::VideoPage => Kind::Video(url),
        Route::GalleryPage => Kind::Gallery(url),
        Route::File => Kind::Link(url),
    })
}

/// Whether a word in running text is an address written whole (its scheme there): the words
/// a dropped or pasted text's links are picked from (as a drop's text is, `win32::ole`).
fn is_written_address(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    ["http://", "https://", "ftp://", "ftps://", "magnet:"]
        .iter()
        .any(|s| w.starts_with(s))
}

/// Every line of `text` read, in order, blank lines left out and a line already found once
/// (the same address) not again. A line that isn't one address whole may be text with links
/// in it: its written addresses are taken one by one, and it is "not an address" only when it
/// has none.
pub fn scan(text: &str, is_file: &dyn Fn(&Path) -> bool) -> Vec<Found> {
    let mut out: Vec<Found> = Vec::new();
    let mut push = |text: &str, kind: Result<Kind, &'static str>| {
        let same = |f: &Found| match (&f.kind, &kind) {
            (Ok(a), Ok(b)) => a == b,
            _ => f.text == text,
        };
        if !out.iter().any(same) {
            out.push(Found {
                text: text.to_owned(),
                kind,
            });
        }
    };
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let whole = read_line(line, is_file);
        // A word of running text may open with a bracket or a quote: "(https://…)".
        let words: Vec<&str> = line
            .split_whitespace()
            .map(|w| w.trim_start_matches(['(', '[', '{', '<', '"', '\'']))
            .filter(|w| is_written_address(w))
            .collect();
        // One address whole (a path with spaces too), unless the line holds several written
        // ones (an address reads spaces after it as part of its path).
        if !line.contains(char::is_whitespace) || (whole.is_ok() && words.len() < 2) {
            push(line, whole);
            continue;
        }
        if words.is_empty() {
            push(line, whole);
        }
        for w in words {
            let w = trim_prose(w);
            push(w, read_line(w, is_file));
        }
    }
    out
}

/// An address picked out of running text, without the sentence's punctuation after it
/// ("…/f.zip," → "…/f.zip"): `, . ; : ! ? ' " >` at its end, and a closing bracket the
/// address doesn't open itself (a wiki's `Foo_(bar)` keeps its own).
fn trim_prose(word: &str) -> &str {
    let mut w = word;
    loop {
        let Some(last) = w.chars().last() else {
            return w;
        };
        let open = match last {
            ')' => Some('('),
            ']' => Some('['),
            '}' => Some('{'),
            _ => None,
        };
        let strip = match open {
            Some(o) => w.matches(o).count() < w.matches(last).count(),
            None => matches!(last, ',' | '.' | ';' | ':' | '!' | '?' | '\'' | '"' | '>'),
        };
        if !strip {
            return w;
        }
        w = &w[..w.len() - last.len_utf8()];
    }
}

/// The items of a scan that can be added.
pub fn valid(found: &[Found]) -> Vec<Kind> {
    found.iter().filter_map(|f| f.kind.clone().ok()).collect()
}

/// What the paste bar's one line holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bar {
    /// Nothing typed.
    Empty,
    /// One address: its kind.
    One(Kind),
    /// Several (pasted text with links): how many; the Add window lists them.
    Many(usize),
    /// Not an address, and no address in it.
    Invalid,
}

/// The paste bar's text read by [`scan`]'s rules: one line, which holds one address (a path
/// with spaces too) or several written one after another.
pub fn bar_kind(text: &str, is_file: &dyn Fn(&Path) -> bool) -> Bar {
    if text.trim().is_empty() {
        return Bar::Empty;
    }
    let mut found = valid(&scan(text, is_file));
    match found.len() {
        0 => Bar::Invalid,
        1 => Bar::One(found.remove(0)),
        n => Bar::Many(n),
    }
}

impl Bar {
    /// The paste bar's button (English, marked): what Enter or a click does.
    pub fn button(&self) -> &'static str {
        match self {
            // i18n: the paste bar's button for a torrent (a magnet link, a .torrent)
            Bar::One(Kind::Magnet(_) | Kind::TorrentUrl(_) | Kind::TorrentFile(_)) => {
                crate::i18n::mark("Add torrent")
            }
            // i18n: the paste bar's button for a video's page: the video window, to choose a quality
            Bar::One(Kind::Video(_)) => crate::i18n::mark("Choose quality"),
            // i18n: the paste bar's button for an image gallery's page
            Bar::One(Kind::Gallery(_)) => crate::i18n::mark("Choose pictures"),
            // i18n: the paste bar's button for an address with a *: Batch download
            Bar::One(Kind::Batch(_)) => crate::i18n::mark("Set range"),
            // i18n: the paste bar's button: add the typed address as a download
            _ => crate::i18n::mark("Download"),
        }
    }

    /// Whether the button can be pressed.
    pub fn enabled(&self) -> bool {
        matches!(self, Bar::One(_) | Bar::Many(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "0123456789abcdef0123456789abcdef01234567";

    fn none(_: &Path) -> bool {
        false
    }

    fn any(_: &Path) -> bool {
        true
    }

    fn kinds(text: &str) -> Vec<Result<Kind, &'static str>> {
        scan(text, &none).into_iter().map(|f| f.kind).collect()
    }

    fn one(text: &str) -> Result<Kind, &'static str> {
        let mut k = scan(text, &any);
        assert_eq!(k.len(), 1, "{text}: one line");
        k.remove(0).kind
    }

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn web_and_ftp_addresses_are_links() {
        for s in [
            "http://example.org/a.zip",
            "https://files.example.org/report-2026.pdf",
            "ftp://ftp.example.org/pub/f.iso",
            "ftps://ftp.example.org/f.iso",
        ] {
            assert_eq!(one(s), Ok(Kind::Link(url(s))), "{s}");
        }
        // Without its scheme: https, as Add URL read it.
        assert_eq!(
            one("www.example.org/a.zip"),
            Ok(Kind::Link(url("https://www.example.org/a.zip")))
        );
    }

    #[test]
    fn a_magnet_is_a_magnet() {
        let m = format!("magnet:?xt=urn:btih:{HASH}&dn=ubuntu.iso");
        assert_eq!(one(&m), Ok(Kind::Magnet(url(&m))));
        let k = one(&m).unwrap();
        assert_eq!(k.name(&m), "ubuntu.iso");
        assert_eq!(k.next(), "Choose its files next");
        assert!(!k.web());
    }

    #[test]
    fn a_torrent_address_and_a_torrent_file() {
        let s = "https://releases.example.org/ubuntu-26.04.torrent";
        assert_eq!(one(s), Ok(Kind::TorrentUrl(url(s))));
        assert_eq!(
            one(r"C:\Downloads\ubuntu.torrent"),
            Ok(Kind::TorrentFile(r"C:\Downloads\ubuntu.torrent".into()))
        );
        // Quoted (Explorer's Copy as path), on a share, with spaces.
        assert_eq!(
            one(r#""D:\My files\a b.TORRENT""#),
            Ok(Kind::TorrentFile(r"D:\My files\a b.TORRENT".into()))
        );
        assert_eq!(
            one(r"\\server\share\x.torrent"),
            Ok(Kind::TorrentFile(r"\\server\share\x.torrent".into()))
        );
        // A file that is not there says so.
        assert_eq!(
            kinds(r"C:\gone\x.torrent"),
            vec![Err("No such file on this computer")]
        );
        assert_eq!(one(r"C:\a\x.torrent").unwrap().name(""), "x.torrent");
    }

    #[test]
    fn video_and_gallery_pages_the_app_knows() {
        let v = "https://www.youtube.com/watch?v=Tg6byXUo8RU";
        assert_eq!(one(v), Ok(Kind::Video(url(v))));
        assert_eq!(site(&url(v)), "youtube.com");
        assert_eq!(one(v).unwrap().next(), "Choose its quality next");
        let g = "https://commons.wikimedia.org/wiki/Category:Featured_pictures_of_birds";
        assert_eq!(one(g), Ok(Kind::Gallery(url(g))));
    }

    #[test]
    fn route_of_matches_the_kinds() {
        let magnet = format!("magnet:?xt=urn:btih:{HASH}");
        for (s, r) in [
            ("https://example.org/a.zip", Route::File),
            (magnet.as_str(), Route::Magnet),
            ("https://example.org/a.torrent", Route::TorrentUrl),
            ("https://www.youtube.com/watch?v=x", Route::VideoPage),
        ] {
            assert_eq!(route_of(&url(s)), r, "{s}");
        }
    }

    #[test]
    fn an_address_with_a_star_is_a_batch() {
        let s = "https://example.org/img*.jpg";
        assert_eq!(one(s), Ok(Kind::Batch(s.into())));
        assert_eq!(kinds("not * an address"), vec![Err("Not an address")]);
    }

    #[test]
    fn junk_says_why() {
        assert_eq!(
            kinds("meeting notes for monday"),
            vec![Err("Not an address")]
        );
        assert_eq!(kinds("hello"), vec![Err("Not an address")]);
        assert_eq!(
            kinds("mailto:a@b.example"),
            vec![Err("Grabbit doesn't download this kind of address")]
        );
        assert_eq!(
            kinds("file:///C:/x.zip"),
            vec![Err("Grabbit doesn't download this kind of address")]
        );
    }

    #[test]
    fn several_lines_blank_ones_and_duplicates() {
        let m = format!("magnet:?xt=urn:btih:{HASH}");
        let text = format!(
            "\n  https://a.example/1.zip  \n\n\t\n{m}\nhttps://a.example/1.zip\njunk words\n{m}\n"
        );
        let found = scan(&text, &none);
        let texts: Vec<&str> = found.iter().map(|f| f.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["https://a.example/1.zip", m.as_str(), "junk words"]
        );
        assert_eq!(valid(&found).len(), 2);
        assert!(scan("", &none).is_empty());
        assert!(scan(" \n\t \r\n", &none).is_empty());
    }

    #[test]
    fn text_with_links_gives_its_links() {
        let found = scan(
            "see https://a.example/1.zip and https://www.youtube.com/watch?v=x, thanks",
            &none,
        );
        assert_eq!(found.len(), 2);
        assert!(matches!(found[0].kind, Ok(Kind::Link(_))));
        assert!(matches!(found[1].kind, Ok(Kind::Video(_))));
        // The sentence's comma isn't part of the address (PR 76's audit).
        assert_eq!(found[0].text, "https://a.example/1.zip");
        assert_eq!(found[1].text, "https://www.youtube.com/watch?v=x");
        let found = scan(
            "get https://a.example/f.zip. Also (https://b.example/g.zip) and \
             https://en.wikipedia.org/wiki/Foo_(bar)!",
            &none,
        );
        let texts: Vec<&str> = found.iter().map(|f| f.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "https://a.example/f.zip",
                "https://b.example/g.zip",
                "https://en.wikipedia.org/wiki/Foo_(bar)"
            ]
        );
    }

    #[test]
    fn the_paste_bar_reads_one_line() {
        assert_eq!(bar_kind("", &none), Bar::Empty);
        assert_eq!(bar_kind("   ", &none), Bar::Empty);
        assert_eq!(bar_kind("meeting notes", &none), Bar::Invalid);
        assert!(!Bar::Invalid.enabled() && !Bar::Empty.enabled());
        let cases = [
            ("https://files.example.org/a.pdf".to_owned(), "Download"),
            (format!("magnet:?xt=urn:btih:{HASH}"), "Add torrent"),
            ("https://example.org/u.torrent".to_owned(), "Add torrent"),
            (
                "https://www.youtube.com/watch?v=x".to_owned(),
                "Choose quality",
            ),
            ("https://example.org/p*.jpg".to_owned(), "Set range"),
        ];
        for (text, button) in cases {
            let b = bar_kind(&text, &none);
            assert!(matches!(b, Bar::One(_)), "{text}");
            assert!(b.enabled());
            assert_eq!(b.button(), button, "{text}");
        }
        let many = bar_kind(
            &format!("https://a.example/1.zip magnet:?xt=urn:btih:{HASH} https://b.example/2.zip"),
            &none,
        );
        assert_eq!(many, Bar::Many(3));
        // A path with spaces is one file.
        assert_eq!(
            bar_kind(r"C:\My files\a b.torrent", &any),
            Bar::One(Kind::TorrentFile(r"C:\My files\a b.torrent".into()))
        );
    }
}
