//! The User-Agent MDM sends for the links it is given by hand (upstream issue 668): the one the
//! default browser of this computer sends today, at the version installed, so a site sees the
//! browser its user has rather than a program or a browser from years ago. The app finds that
//! browser once, off the interface's thread (`win32::default_browser`), and hands it here
//! (`set_automatic`); until then, or when none is found, MDM speaks as a current Chrome on
//! Windows (`fallback`).
//!
//! A User-Agent typed in Options, or the one a browser hands over with its download (the
//! extension's), always wins over this one: the download's `Options::automatic_agent` says which.

use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// MDM's User-Agent before upstream issue 668: Internet Explorer 11's, as IDM 6.42 sends it (lab
/// S1, D-007). Still asked with once when a site turns a browser's User-Agent away on MDM's
/// connection (#372: a bot check that sees a browser's name on a connection that isn't a
/// browser's lets this one through), and a setting saved with it means "automatic" (it was the
/// default).
pub const IE11_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.1; Trident/7.0; rv:11.0) like Gecko";

/// A browser whose User-Agent MDM knows how to write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Browser {
    Chrome,
    Edge,
    Brave,
    Vivaldi,
    Opera,
    /// Chromium itself and the Chromium browsers that send its User-Agent unchanged.
    Chromium,
    Firefox,
    /// LibreWolf counts as Firefox does; Waterfox, Floorp and Zen count their own versions but
    /// send Firefox's User-Agent (at the Firefox they are built on: MDM's estimate of today's).
    FirefoxFork,
}

impl Browser {
    /// The name people know it by (a product name: not translated).
    pub fn name(self) -> &'static str {
        match self {
            Browser::Chrome => "Google Chrome",
            Browser::Edge => "Microsoft Edge",
            Browser::Brave => "Brave",
            Browser::Vivaldi => "Vivaldi",
            Browser::Opera => "Opera",
            Browser::Chromium => "Chromium",
            Browser::Firefox | Browser::FirefoxFork => "Firefox",
        }
    }

    /// The browser `name` is: an executable's file name without its extension (`msedge`), or
    /// the ProgId Windows opens https links with (`ChromeHTML`, `MSEdgeHTM`); case ignored. None
    /// for anything else (an unknown browser: MDM then speaks as Chrome).
    pub fn named(name: &str) -> Option<Browser> {
        let n = name.to_ascii_lowercase();
        let has = |w: &str| n.contains(w);
        // The order matters: "chromium" holds "chrom".
        Some(if has("msedge") || n == "edge" {
            Browser::Edge
        } else if has("brave") {
            Browser::Brave
        } else if has("vivaldi") {
            Browser::Vivaldi
        } else if has("opera") {
            Browser::Opera
        } else if has("chromium") {
            Browser::Chromium
        } else if has("chrome") {
            Browser::Chrome
        } else if has("librewolf") || has("firefox") {
            Browser::Firefox
        } else if has("waterfox") || has("floorp") || n == "zen" {
            Browser::FirefoxFork
        } else {
            return None;
        })
    }

    fn chromium_based(self) -> bool {
        !matches!(self, Browser::Firefox | Browser::FirefoxFork)
    }
}

/// The default browser found on this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub browser: Browser,
    /// Its version as installed ("157.0.8103.39", Brave's "157.1.99.55", Opera's "122.0.5643.24",
    /// Firefox's "160.0"); None when it couldn't be read (MDM's estimate then).
    pub version: Option<String>,
}

impl Found {
    /// "Brave 157", "Firefox 160": the browser as Options shows it.
    pub fn label(&self) -> String {
        match (self.browser, self.version.as_deref()) {
            (Browser::FirefoxFork, _) | (_, None) => self.browser.name().to_owned(),
            (b, Some(v)) => match leading_number(v) {
                Some(major) => format!("{} {major}", b.name()),
                None => b.name().to_owned(),
            },
        }
    }
}

/// The day Chrome 157 became stable (2026-11-10), and Chrome's and Firefox's rhythm of a new
/// major version every 4 weeks: `chrome_major` counts from it.
const CHROME_KNOWN: (u32, u64) = (157, 1_794_268_800);
/// Firefox 160, the same day.
const FIREFOX_KNOWN: (u32, u64) = (160, 1_794_268_800);
const FOUR_WEEKS: u64 = 28 * 24 * 3600;
const A_YEAR: u64 = 365 * 24 * 3600;

/// A browser's current stable major version at `now` (seconds since 1970), counted from one
/// known release in its rhythm: what MDM says when it can't read the installed version. Never
/// before the known one, never more than two years past it (a clock far off).
fn estimate(known: (u32, u64), every: u64, now: u64) -> u32 {
    let steps = now.saturating_sub(known.1) / every;
    known.0 + steps.min(2 * A_YEAR / every) as u32
}

/// Chrome's current stable major version (`estimate`).
pub fn chrome_major(now: u64) -> u32 {
    estimate(CHROME_KNOWN, FOUR_WEEKS, now)
}

fn firefox_major(now: u64) -> u32 {
    estimate(FIREFOX_KNOWN, FOUR_WEEKS, now)
}

/// The number `s` starts with ("157" of "157.0.8103.39", "140" of "140.4.0esr").
fn leading_number(s: &str) -> Option<u32> {
    let digits: String = s.trim().chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok().filter(|n| *n > 0)
}

/// The User-Agent `found` sends on Windows at `now` (seconds since 1970, for the versions MDM
/// estimates): the reduced User-Agents browsers send today, whose minor version numbers are
/// zeros. Nothing found, or a browser MDM doesn't know: a current Chrome's.
pub fn user_agent(found: Option<&Found>, now: u64) -> String {
    const PLATFORM: &str = "Windows NT 10.0; Win64; x64";
    let browser = found.map_or(Browser::Chrome, |f| f.browser);
    let major = found
        .and_then(|f| f.version.as_deref())
        .and_then(leading_number);
    if browser.chromium_based() {
        // Opera and Vivaldi count their own versions, not their Chromium's.
        let chrome = match browser {
            Browser::Opera | Browser::Vivaldi => None,
            _ => major,
        }
        .unwrap_or_else(|| chrome_major(now));
        let base = format!(
            "Mozilla/5.0 ({PLATFORM}) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/{chrome}.0.0.0 Safari/537.36"
        );
        return match (browser, major) {
            (Browser::Edge, _) => format!("{base} Edg/{chrome}.0.0.0"),
            (Browser::Opera, Some(opera)) => format!("{base} OPR/{opera}.0.0.0"),
            _ => base,
        };
    }
    let firefox = match browser {
        Browser::FirefoxFork => None,
        _ => major,
    }
    .unwrap_or_else(|| firefox_major(now));
    format!("Mozilla/5.0 ({PLATFORM}; rv:{firefox}.0) Gecko/20100101 Firefox/{firefox}.0")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// What the app found (`set_automatic`): the browser's label and its User-Agent. None: not
/// looked for yet.
static AUTOMATIC: RwLock<Option<(Option<String>, String)>> = RwLock::new(None);

/// The default browser the app found (None: none, or one MDM doesn't know): from now on the
/// automatic User-Agent is the one it sends.
pub fn set_automatic(found: Option<Found>) {
    let agent = user_agent(found.as_ref(), now());
    *AUTOMATIC.write().unwrap() = Some((found.map(|f| f.label()), agent));
}

/// The User-Agent MDM sends for a link given by hand when the user set none: the default
/// browser's (`set_automatic`), else a current Chrome's (`fallback`).
pub fn automatic() -> String {
    match &*AUTOMATIC.read().unwrap() {
        Some((_, agent)) => agent.clone(),
        None => fallback(),
    }
}

/// The browser `automatic` speaks as ("Brave 157"); None until it is found, or when none was.
pub fn automatic_browser() -> Option<String> {
    AUTOMATIC
        .read()
        .unwrap()
        .as_ref()
        .and_then(|(b, _)| b.clone())
}

/// Whether the default browser has been looked for (`set_automatic`).
pub fn looked_for() -> bool {
    AUTOMATIC.read().unwrap().is_some()
}

/// A current Chrome's User-Agent on Windows: when no browser is found.
pub fn fallback() -> String {
    user_agent(None, now())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-11-11, the day these were written upstream.
    const TODAY: u64 = 1_794_355_200;

    fn ua(browser: Browser, version: Option<&str>) -> String {
        let found = Found {
            browser,
            version: version.map(str::to_owned),
        };
        user_agent(Some(&found), TODAY)
    }

    #[test]
    fn chromium_browsers_send_the_reduced_user_agent() {
        use Browser::*;
        assert_eq!(
            ua(Chrome, Some("157.0.8103.39")),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/157.0.0.0 Safari/537.36"
        );
        // Brave's version is its Chromium's major, then its own; it sends Chrome's agent.
        assert_eq!(
            ua(Brave, Some("157.1.99.55")),
            ua(Chrome, Some("157.0.8103.39"))
        );
        assert_eq!(
            ua(Chromium, Some("156.0.7000.1")),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/156.0.0.0 Safari/537.36"
        );
        assert_eq!(
            ua(Edge, Some("157.0.4341.42")),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/157.0.0.0 Safari/537.36 Edg/157.0.0.0"
        );
        // Opera counts its own version: OPR/ is it, Chrome/ today's Chromium (estimated).
        assert_eq!(
            ua(Opera, Some("122.0.5643.24")),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/157.0.0.0 Safari/537.36 OPR/122.0.0.0"
        );
        // Opera without a version, and Vivaldi: Chrome's.
        assert_eq!(ua(Opera, None), ua(Chrome, Some("157.0.1.2")));
        assert_eq!(
            ua(Vivaldi, Some("7.6.3797.52")),
            ua(Chrome, Some("157.0.1.2"))
        );
    }

    #[test]
    fn firefox_sends_its_own() {
        use Browser::*;
        assert_eq!(
            ua(Firefox, Some("160.0")),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:160.0) Gecko/20100101 Firefox/160.0"
        );
        assert_eq!(
            ua(Firefox, Some("140.4.0esr")),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:140.0) Gecko/20100101 Firefox/140.0"
        );
        // A fork counts its own version: Firefox's of today instead.
        assert_eq!(ua(FirefoxFork, Some("6.6.4")), ua(Firefox, Some("160.0")));
    }

    #[test]
    fn nothing_found_is_a_current_chrome() {
        assert_eq!(
            user_agent(None, TODAY),
            ua(Browser::Chrome, Some("157.0.0.0"))
        );
        // A browser without a version: today's.
        assert_eq!(
            ua(Browser::Edge, None),
            ua(Browser::Edge, Some("157.0.0.0"))
        );
        assert!(ua(Browser::Firefox, None).contains("Firefox/160.0"));
        // Chrome's rhythm, never before the known release nor far past it.
        assert_eq!(chrome_major(0), 157);
        assert_eq!(chrome_major(CHROME_KNOWN.1 + 28 * 86_400), 158);
        assert_eq!(chrome_major(CHROME_KNOWN.1 + 100 * A_YEAR), 157 + 26);
    }

    #[test]
    fn browsers_by_their_programs_and_prog_ids() {
        use Browser::*;
        for (name, b) in [
            ("chrome", Chrome),
            ("msedge", Edge),
            ("brave", Brave),
            ("vivaldi", Vivaldi),
            ("opera", Opera),
            ("Opera GX", Opera),
            ("firefox", Firefox),
            ("librewolf", Firefox),
            ("waterfox", FirefoxFork),
            ("zen", FirefoxFork),
            ("ChromeHTML", Chrome),
            ("MSEdgeHTM", Edge),
            ("BraveHTML", Brave),
            ("FirefoxURL-308046B0AF4A39CB", Firefox),
            ("OperaStable", Opera),
        ] {
            assert_eq!(Browser::named(name), Some(b), "{name}");
        }
        for other in ["iexplore", "thunderbird", "IE.HTTPS", ""] {
            assert_eq!(Browser::named(other), None, "{other}");
        }
    }

    #[test]
    fn labels_name_the_browser_and_its_major_version() {
        let found = |browser, version: Option<&str>| Found {
            browser,
            version: version.map(str::to_owned),
        };
        assert_eq!(
            found(Browser::Brave, Some("157.1.99.55")).label(),
            "Brave 157"
        );
        assert_eq!(
            found(Browser::Opera, Some("122.0.1.2")).label(),
            "Opera 122"
        );
        assert_eq!(found(Browser::Firefox, None).label(), "Firefox");
    }
}
