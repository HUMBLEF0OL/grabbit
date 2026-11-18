//! The browsers the extension runs in: one table for the app (which lists the browsers and
//! checks the manifests are there), #41, and the native host.

use std::path::{Path, PathBuf};

/// Which manifest a browser reads: Chromium's (`allowed_origins`) or Firefox's
/// (`allowed_extensions`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Chromium,
    Gecko,
}

/// A browser MDM knows by name.
#[derive(Debug)]
pub struct Browser {
    /// Its name in the app (the offer window, Options).
    pub name: &'static str,
    pub family: Family,
}

const fn chromium(name: &'static str) -> Browser {
    Browser {
        name,
        family: Family::Chromium,
    }
}

const fn gecko(name: &'static str) -> Browser {
    Browser {
        name,
        family: Family::Gecko,
    }
}

/// Every browser MDM names, Chromium's first.
pub const BROWSERS: &[Browser] = &[
    chromium("Google Chrome"),
    chromium("Chromium"),
    chromium("Microsoft Edge"),
    chromium("Brave"),
    chromium("Vivaldi"),
    chromium("Opera"),
    chromium("Helium"),
    chromium("Thorium"),
    chromium("Arc"),
    gecko("Mozilla Firefox"),
    gecko("Zen Browser"),
    gecko("LibreWolf"),
    gecko("Floorp"),
    gecko("Waterfox"),
];

/// The browser named `name` in the table.
pub fn named(name: &str) -> Option<&'static Browser> {
    BROWSERS.iter().find(|b| b.name == name)
}

/// Windows: the programs of the browsers in the table, with the folder that tells apart those
/// that share a program's name (Chrome, Chromium and Helium all install a `chrome.exe`).
pub const WINDOWS_PROGRAMS: &[(&str, Option<&str>, &str)] = &[
    ("chrome.exe", Some(r"\google\chrome\"), "Google Chrome"),
    ("chrome.exe", Some(r"\imput\helium\"), "Helium"),
    ("chrome.exe", Some(r"\chromium\"), "Chromium"),
    ("msedge.exe", None, "Microsoft Edge"),
    ("brave.exe", None, "Brave"),
    ("vivaldi.exe", None, "Vivaldi"),
    ("opera.exe", None, "Opera"),
    ("thorium.exe", None, "Thorium"),
    ("firefox.exe", None, "Mozilla Firefox"),
    ("zen.exe", None, "Zen Browser"),
    ("librewolf.exe", None, "LibreWolf"),
    ("floorp.exe", None, "Floorp"),
    ("waterfox.exe", None, "Waterfox"),
];

/// Windows: the browser `program` (a full path) is, by its file name and folder; none for a
/// program not in the table (or a `chrome.exe` in another folder).
pub fn windows_named(program: &str) -> Option<&'static str> {
    let lower = program.to_lowercase();
    let file = lower.rsplit(['\\', '/']).next().unwrap_or(&lower);
    WINDOWS_PROGRAMS
        .iter()
        .find(|(exe, folder, _)| *exe == file && folder.is_none_or(|f| lower.contains(f)))
        .map(|(_, _, name)| *name)
}

/// Windows: the program a `shell\open\command` value starts (`"C:\…\zen.exe" -osint -url "%1"`
/// or unquoted `C:\…\x.exe %1`).
pub fn command_program(command: &str) -> Option<String> {
    let c = command.trim();
    let program = if let Some(rest) = c.strip_prefix('"') {
        rest.split('"').next()?
    } else {
        let end = c.to_lowercase().find(".exe").map(|i| i + 4)?;
        &c[..end]
    };
    let program = program.trim();
    (!program.is_empty()).then(|| program.to_owned())
}

/// Windows lists Internet Explorer as a browser; it can't run the extension.
pub fn windows_can_run_extension(program: &str) -> bool {
    !program.to_lowercase().ends_with("iexplore.exe")
}

/// The program the host manifest at `manifest` starts (its `path`; a relative one, which
/// Windows allows, is beside the manifest), when the manifest can be read.
pub fn manifest_program(manifest: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(manifest).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let path = PathBuf::from(json.get("path")?.as_str()?.trim());
    if path.as_os_str().is_empty() {
        return None;
    }
    Some(match manifest.parent() {
        Some(dir) if path.is_relative() => dir.join(path),
        _ => path,
    })
}

/// The native host's name, which its manifests and registry entries carry.
pub const HOST_NAME: &str = "com.grabbit.host";

/// The registry keys the browsers look the native host up under (`<key>\HOST_NAME`'s default
/// value: the manifest's path), under HKCU for one user or HKLM for everyone, each with the
/// family whose manifest it names. Chromium-branded builds (Chromium, Helium, Thorium, Brave and
/// Brave Origin with every channel) read Chromium's first, then Chrome's, as most other forks do
/// (Chromium's `launch_context_win.cc`); Brave's own key is kept for builds that read it.
/// Firefox and its forks read Mozilla's.
pub const WINDOWS_HOST_KEYS: [(&str, Family); 5] = [
    (
        r"Software\Google\Chrome\NativeMessagingHosts",
        Family::Chromium,
    ),
    (r"Software\Chromium\NativeMessagingHosts", Family::Chromium),
    (
        r"Software\Microsoft\Edge\NativeMessagingHosts",
        Family::Chromium,
    ),
    (
        r"Software\BraveSoftware\Brave-Browser\NativeMessagingHosts",
        Family::Chromium,
    ),
    (r"Software\Mozilla\NativeMessagingHosts", Family::Gecko),
];

/// The manifest a registry entry's value names (quotes and spaces around it dropped).
pub fn registered_manifest(value: &str) -> PathBuf {
    PathBuf::from(value.trim().trim_matches('"'))
}

/// What a browser finds where it looks for the native host (upstream issue 818).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Holder {
    /// A manifest naming a helper program that is there (this one).
    Alive(PathBuf),
    /// The manifest isn't there (or can't be read): the copy that wrote it is gone, such as one
    /// run from inside a zip, whose temporary folder the archive tool deleted afterwards.
    ManifestGone(PathBuf),
    /// The manifest names a helper program that isn't there (this path).
    ProgramGone(PathBuf),
}

impl Holder {
    /// What the manifest at `manifest` leads to.
    pub fn of(manifest: &Path) -> Holder {
        match manifest_program(manifest) {
            None => Holder::ManifestGone(manifest.to_owned()),
            Some(program) if !program.is_file() => Holder::ProgramGone(program),
            Some(program) => Holder::Alive(program),
        }
    }

    /// The helper it leads to is gone: no browser can start it.
    pub fn dead(&self) -> bool {
        !matches!(self, Holder::Alive(_))
    }
}

/// What a copy of Grabbit does at its start with one browser's registry entries (upstream
/// issue 818).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repair {
    /// Nothing: they work (whichever copy they name: two copies are both legitimate, and
    /// neither takes the other's), or there are none (the setup's box was unticked).
    Keep,
    /// Point the user's entry at this copy: the one there leads nowhere, or there is none and
    /// the computer's leads nowhere.
    Take,
    /// Remove the user's entry, which leads nowhere: the computer's works, and is read once the
    /// user's is gone.
    Clear,
}

/// The [`Repair`] for one browser's entries, the user's (HKCU) and the computer's (HKLM). The
/// browsers read the user's first, and the computer's only when there is no user's: a user's
/// that leads nowhere hides a computer's that works (Chromium's `launch_context_win.cc`).
pub fn windows_repair(user: Option<&Holder>, machine: Option<&Holder>) -> Repair {
    match (user, machine) {
        (Some(Holder::Alive(_)), _) => Repair::Keep,
        (Some(_), Some(Holder::Alive(_))) => Repair::Clear,
        (Some(_), _) => Repair::Take,
        (None, Some(m)) if m.dead() => Repair::Take,
        (None, _) => Repair::Keep,
    }
}

/// Whether `program` runs from a folder that is deleted soon after it quits (upstream issue
/// 818), where a registration would lead the browsers nowhere once it is gone: under the
/// temporary folder (where WinRAR, 7-Zip and Windows Explorer put a program run from inside an
/// archive), or in an archive tool's own extraction folder wherever its temporary folder was set.
pub fn in_temporary_folder(program: &Path) -> bool {
    temporary(program, &temporary_folders())
}

/// The temporary folders: this process's, the ones TMP and TEMP name, and Windows' own.
fn temporary_folders() -> Vec<PathBuf> {
    let mut dirs = vec![std::env::temp_dir()];
    for var in ["TMP", "TEMP"] {
        if let Some(d) = std::env::var_os(var).filter(|d| !d.is_empty()) {
            dirs.push(PathBuf::from(d));
        }
    }
    if let Some(root) = std::env::var_os("SystemRoot") {
        dirs.push(PathBuf::from(root).join("Temp"));
    }
    dirs.retain(|d| d.is_absolute());
    dirs
}

/// [`in_temporary_folder`] with the temporary folders given.
fn temporary(program: &Path, temps: &[PathBuf]) -> bool {
    // Windows' paths ignore case.
    let lower = |p: &Path| PathBuf::from(p.to_string_lossy().to_lowercase());
    let program = lower(&full_path(program));
    let under = |dir: &PathBuf| program.starts_with(lower(&full_path(dir)));
    let extracted = program
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|c| archive_folder(&c.as_os_str().to_string_lossy()));
    extracted || temps.iter().any(under)
}

/// A folder an archive tool extracts a program into to run it from inside the archive, and
/// deletes afterwards: WinRAR's `Rar$EXa…` (`Rar$DIa…` for a drag), 7-Zip's `7zO…` / `7zE…`
/// followed by hex digits, Windows Explorer's `Temp1_<name>.zip`.
fn archive_folder(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let seven = lower.strip_prefix("7z").is_some_and(|rest| {
        rest.len() >= 5
            && rest.starts_with(['o', 'e', 's'])
            && rest[1..].chars().all(|c| c.is_ascii_hexdigit())
    });
    let explorer = lower.strip_prefix("temp").is_some_and(|rest| {
        rest.ends_with(".zip")
            && rest
                .split_once('_')
                .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    });
    lower.starts_with("rar$") || seven || explorer
}

/// `path` with its links and short names resolved (`C:\Users\MEGAPC~1` is `C:\Users\Mega PC`),
/// without the `\\?\` before a drive. A path that isn't there (yet) is resolved as far as it is,
/// the rest kept as given, so it compares with a folder that is; as given when none of it can be
/// read.
fn full_path(path: &Path) -> PathBuf {
    let mut there = path;
    let mut rest = Vec::new();
    let full = loop {
        if let Ok(full) = std::fs::canonicalize(there) {
            break rest
                .iter()
                .rev()
                .fold(full, |p: PathBuf, name| p.join(name));
        }
        match (there.parent(), there.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name);
                there = parent;
            }
            _ => break path.to_owned(),
        }
    };
    let text = full.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => full,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #41: Helium installs a chrome.exe; only Chrome's folder makes it Google Chrome.
    #[test]
    fn windows_programs_by_file_and_folder() {
        let named = |p: &str| windows_named(p);
        assert_eq!(
            named(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            Some("Google Chrome")
        );
        assert_eq!(
            named(r"C:\Users\a\AppData\Local\imput\Helium\Application\chrome.exe"),
            Some("Helium")
        );
        assert_eq!(
            named(r"C:\Users\a\AppData\Local\Chromium\Application\chrome.exe"),
            Some("Chromium")
        );
        assert_eq!(
            named(r"D:\Tools\chrome.exe"),
            None,
            "a chrome.exe elsewhere"
        );
        assert_eq!(
            named(r"C:\Program Files\Zen Browser\ZEN.EXE"),
            Some("Zen Browser")
        );
        assert_eq!(named(r"C:\x\unknown.exe"), None);
        for (_, _, name) in WINDOWS_PROGRAMS {
            assert!(super::named(name).is_some(), "{name} is in the table");
        }
    }

    #[test]
    fn programs_from_open_commands() {
        assert_eq!(
            command_program(r#""C:\Program Files\Zen Browser\zen.exe" -osint -url "%1""#)
                .as_deref(),
            Some(r"C:\Program Files\Zen Browser\zen.exe")
        );
        assert_eq!(
            command_program(r"C:\Browsers\x.EXE %1").as_deref(),
            Some(r"C:\Browsers\x.EXE")
        );
        assert_eq!(command_program(r#""""#), None);
        assert_eq!(command_program("rundll32"), None);
        assert!(!windows_can_run_extension(
            r"C:\Program Files\Internet Explorer\iexplore.exe"
        ));
        assert!(windows_can_run_extension(r"C:\x\zen.exe"));
    }

    #[test]
    fn names_are_unique_and_found() {
        for b in BROWSERS {
            assert_eq!(named(b.name).map(|x| x.name), Some(b.name));
        }
        assert!(named("Netscape").is_none());
    }

    #[test]
    fn a_manifests_program() {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("host.json");
        let abs = d.path().join("elsewhere").join("host");
        let json = serde_json::json!({ "name": "x", "path": abs, "type": "stdio" });
        std::fs::write(&m, json.to_string()).unwrap();
        assert_eq!(manifest_program(&m), Some(abs));
        std::fs::write(&m, r#"{"name":"x","path":"host.exe"}"#).unwrap();
        assert_eq!(manifest_program(&m), Some(d.path().join("host.exe")));
        for bad in [r#"{"name":"x"}"#, r#"{"path":""}"#, "not json"] {
            std::fs::write(&m, bad).unwrap();
            assert_eq!(manifest_program(&m), None, "{bad}");
        }
        assert_eq!(manifest_program(&d.path().join("none.json")), None);
    }

    /// Upstream issue 818: a copy run from inside an archive, or from anywhere under the
    /// temporary folder, is in a folder that is deleted later; a copy in a folder of its own
    /// isn't.
    #[test]
    fn temporary_folders_are_found() {
        let names = [
            ("Rar$EXa34720.17029.rartemp", true),
            ("Rar$DIa1234.5678", true),
            ("7zO8A3B1C2D", true),
            ("7zE0F12AB34", true),
            ("Temp1_Grabbit-1.0.0-x64.zip", true),
            ("Temp12_x.zip", true),
            ("7-Zip", false),
            ("7zip", false),
            ("7zOops", false),
            ("Templates", false),
            ("Temp_x.zip", false),
            ("Grabbit-1.0.0-x64", false),
            ("Rar", false),
        ];
        for (name, archive) in names {
            assert_eq!(archive_folder(name), archive, "{name}");
        }
        let (temp, home) = (r"C:\Users\x\AppData\Local\Temp", r"C:\Users\x");
        let temps = [PathBuf::from(temp)];
        let at = |p: &str| Path::new(p).join("grabbit.exe");
        let rar = format!(r"{temp}\Rar$EXa34720.17029.rartemp\Grabbit-1.0.0-x64");
        assert!(temporary(&at(&rar), &temps));
        assert!(temporary(&at(&format!(r"{temp}\grabbit")), &temps));
        assert!(temporary(&at(temp), &temps));
        // An archive tool's folder outside the temporary folder (WinRAR's own setting).
        assert!(temporary(&at(&format!(r"{home}\rt\Rar$EXa1.2")), &temps));
        assert!(!temporary(&at(&format!(r"{home}\Grabbit")), &temps));
        assert!(!temporary(
            &at(&format!(r"{home}\Temporary\Grabbit")),
            &temps
        ));
        assert!(!temporary(&at(&format!(r"{temp}x\Grabbit")), &temps));
        let upper = r"C:\USERS\X\APPDATA\LOCAL\TEMP\grabbit";
        assert!(temporary(&at(upper), &temps), "Windows' paths ignore case");
        // The folder itself, read through its short names, as the system gives it.
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("copy")).unwrap();
        let exe = d.path().join("copy").join("grabbit.exe");
        std::fs::write(&exe, b"").unwrap();
        assert!(in_temporary_folder(&exe));
    }

    fn manifest_naming(dir: &Path, program: &Path) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let m = dir.join(format!("{HOST_NAME}.json"));
        let json = serde_json::json!({ "name": HOST_NAME, "path": program, "type": "stdio" });
        std::fs::write(&m, json.to_string()).unwrap();
        m
    }

    /// Upstream issue 818: what a registration leads to: a helper that is there, or nothing
    /// (the manifest gone with its temporary folder, or the helper it names gone).
    #[test]
    fn a_registration_leads_to_a_helper_or_nowhere() {
        let d = tempfile::tempdir().unwrap();
        let helper = d.path().join("grabbit").join("grabbit-nmhost.exe");
        let m = manifest_naming(&d.path().join("grabbit"), &helper);
        std::fs::write(&helper, b"").unwrap();
        assert_eq!(Holder::of(&m), Holder::Alive(helper.clone()));
        assert!(!Holder::of(&m).dead());
        let gone = d
            .path()
            .join("Rar$EXa1.2")
            .join(format!("{HOST_NAME}.json"));
        assert_eq!(Holder::of(&gone), Holder::ManifestGone(gone.clone()));
        let missing = d.path().join("old").join("grabbit-nmhost.exe");
        let m = manifest_naming(d.path(), &missing);
        assert_eq!(Holder::of(&m), Holder::ProgramGone(missing));
        assert!(Holder::of(&m).dead());
    }

    /// Upstream issue 818: at a start, only an entry that leads nowhere is repaired; one that
    /// works is never taken, whichever copy it names; a user's that leads nowhere is removed
    /// when the computer's works (the browsers read the user's first).
    #[test]
    fn only_a_dead_registration_is_repaired() {
        let alive = Holder::Alive(PathBuf::from(r"C:\Other\grabbit-nmhost.exe"));
        let gone = Holder::ManifestGone(PathBuf::from(r"C:\Temp\Rar$EXa1\x.json"));
        let helper = Holder::ProgramGone(PathBuf::from(r"C:\Old\grabbit-nmhost.exe"));
        use Repair::*;
        for (user, machine, repair) in [
            (None, None, Keep),
            (Some(&alive), None, Keep),
            (Some(&alive), Some(&gone), Keep),
            (Some(&alive), Some(&alive), Keep),
            (None, Some(&alive), Keep),
            (Some(&gone), None, Take),
            (Some(&helper), None, Take),
            (Some(&gone), Some(&helper), Take),
            (None, Some(&gone), Take),
            (Some(&gone), Some(&alive), Clear),
            (Some(&helper), Some(&alive), Clear),
        ] {
            assert_eq!(
                windows_repair(user, machine),
                repair,
                "{user:?} {machine:?}"
            );
        }
    }
}
