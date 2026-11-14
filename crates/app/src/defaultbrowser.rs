//! The default browser of this computer and its version (upstream issue 668), found once when
//! MDM starts, on a thread of its own: its User-Agent becomes the one MDM sends for links added
//! by hand (`mdm_engine::agent`). Where to look is Windows' (`win32::default_browser`); reading
//! the command line it finds is here.

use std::sync::mpsc;
use std::time::Duration;

/// Look for the default browser on a thread of its own; the automatic User-Agent is a current
/// Chrome's until it is found. Returns a receiver told when it is done (`wait`).
pub fn start() -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("default-browser".into())
        .spawn(move || {
            let started = std::time::Instant::now();
            let found = crate::win32::default_browser();
            let browser = found.as_ref().map_or("none MDM knows".to_owned(), |f| {
                format!("{:?} {}", f.browser, f.version.as_deref().unwrap_or("?"))
            });
            mdm_engine::agent::set_automatic(found);
            tracing::info!(
                "default browser: {browser} ({} ms); links added by hand go as {:?}",
                started.elapsed().as_millis(),
                mdm_engine::agent::automatic()
            );
            let _ = tx.send(());
        });
    if spawned.is_err() {
        mdm_engine::agent::set_automatic(None);
    }
    rx
}

/// Wait at most `limit` for `start`'s search (the command line's downloads begin at once).
pub fn wait(done: &mpsc::Receiver<()>, limit: Duration) {
    let _ = done.recv_timeout(limit);
}

/// The program a Windows `shell\open\command` runs: `"C:\…\brave.exe" --single-argument %1`
/// (quoted), or `C:\Program Files\Mozilla Firefox\firefox.exe -osint -url "%1"` (not quoted: up
/// to its `.exe`).
pub fn windows_program(command: &str) -> Option<String> {
    let c = command.trim();
    let program = match c.strip_prefix('"') {
        Some(rest) => rest.split('"').next()?,
        None => {
            let end = c.to_ascii_lowercase().find(".exe")? + 4;
            &c[..end]
        }
    };
    let program = program.trim();
    (!program.is_empty()).then(|| program.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_commands_name_their_program() {
        assert_eq!(
            windows_program(
                r#""C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe" --single-argument %1"#
            )
            .as_deref(),
            Some(r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe")
        );
        assert_eq!(
            windows_program(r#"C:\Program Files\Mozilla Firefox\firefox.EXE -osint -url "%1""#)
                .as_deref(),
            Some(r"C:\Program Files\Mozilla Firefox\firefox.EXE")
        );
        assert_eq!(windows_program(""), None);
        assert_eq!(windows_program("rundll32 url.dll"), None);
    }
}
