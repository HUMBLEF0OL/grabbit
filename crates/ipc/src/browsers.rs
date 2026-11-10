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
}
