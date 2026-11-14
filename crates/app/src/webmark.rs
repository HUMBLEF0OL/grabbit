//! The mark a downloaded file carries to say it came from the internet (upstream issue 813):
//! Windows' Mark of the Web, the `Zone.Identifier` stream SmartScreen and Office's Protected View
//! read. Every download MDM makes is marked here (the engine's files as they are created,
//! videos, galleries, torrents), as Options → Save to → "Mark downloaded files as coming from the
//! internet" says and as Windows' own policy for attachments says: browsers save their downloads
//! through Windows' attachment services, which leave the mark out under "Do not preserve zone
//! information in file attachments".

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static ON: AtomicBool = AtomicBool::new(true);

/// Options → Save to → "Mark downloaded files as coming from the internet", as it is now (MDM's
/// start, Options → OK).
pub fn set(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// Whether Windows keeps the mark on downloads, from its Group Policy "Do not preserve zone
/// information in file attachments" (`SaveZoneInformation`, `win32::zone_policy`): the
/// computer's value, where it is set, wins over the user's; 1 keeps none, 2 or none set keeps it.
pub fn keeps_zones(machine: Option<u32>, user: Option<u32>) -> bool {
    machine.or(user) != Some(1)
}

/// Whether a downloaded file is marked: MDM's option, and Windows' policy.
pub fn wanted(option: bool, system_keeps: bool) -> bool {
    option && system_keeps
}

/// Mark `path`, a file being downloaded (or just downloaded) from `url` (`referrer`'s page), as
/// coming from the internet, unless MDM's option or Windows' policy says not to. The policy is
/// read now (two registry reads: for a thread other than the window's).
pub fn mark(path: &Path, url: &str, referrer: Option<&str>) {
    let (machine, user) = crate::win32::zone_policy();
    if wanted(ON.load(Ordering::Relaxed), keeps_zones(machine, user)) {
        crate::win32::mark_downloaded(path, url, referrer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_computers_policy_wins_and_only_1_drops_the_mark() {
        // Not set anywhere: kept, as Windows keeps it.
        assert!(keeps_zones(None, None));
        // The user's own policy.
        assert!(!keeps_zones(None, Some(1)));
        assert!(keeps_zones(None, Some(2)));
        // The computer's, over the user's either way.
        assert!(!keeps_zones(Some(1), None));
        assert!(!keeps_zones(Some(1), Some(2)));
        assert!(keeps_zones(Some(2), Some(1)));
        // A value Windows doesn't define keeps it.
        assert!(keeps_zones(Some(0), None));
    }

    #[test]
    fn a_file_is_marked_only_with_the_option_on_and_the_policy_keeping_it() {
        for (option, policy, marked) in [
            (true, None, true),
            (true, Some(2), true),
            (true, Some(1), false),
            (false, None, false),
            (false, Some(2), false),
            (false, Some(1), false),
        ] {
            assert_eq!(
                wanted(option, keeps_zones(None, policy)),
                marked,
                "option {option}, policy {policy:?}"
            );
        }
    }
}
