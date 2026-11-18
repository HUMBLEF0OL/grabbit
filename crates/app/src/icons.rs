//! MDM's own icons by role, as an icon set draws them (`crate::themeicons`): the toolbar's from
//! the theme's set, else Graphite's; the others (helpers, the drag handle)
//! Graphite's in every theme (plans 05.5a and 05.5c). IDM's own images are never used (D-001).

use egui::Image;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    AddUrl,
    /// Add Torrent (ours, M15).
    AddTorrent,
    Resume,
    Stop,
    StopAll,
    Delete,
    DeleteCompleted,
    Options,
    Scheduler,
    StartQueue,
    StopQueue,
    /// The first-start helpers window (ours): yt-dlp finds a page's video,
    HelperVideo,
    /// ffmpeg joins its picture and sound,
    HelperMerge,
    /// and both are checked before they run.
    HelperChecked,
    /// The toolbar's Details toggle (plan 06): shows or hides the details pane.
    Details,
}

impl Icon {
    /// The icon set's name for this icon (`crate::themeicons`): a toolbar need, or the Graphite
    /// icon its place draws (plan 05.5a; the mockup names each).
    fn need(self) -> &'static str {
        match self {
            Icon::AddUrl => "tb-add-url",
            Icon::AddTorrent => "tb-add-torrent",
            Icon::Resume => "tb-resume",
            Icon::Stop => "tb-stop",
            Icon::StopAll => "tb-stop-all",
            Icon::Delete => "tb-delete",
            Icon::DeleteCompleted => "tb-delete-completed",
            Icon::Options => "tb-options",
            Icon::Scheduler => "tb-scheduler",
            Icon::StartQueue => "tb-start-queue",
            Icon::StopQueue => "tb-stop-queue",
            Icon::HelperVideo => "gx-search",
            Icon::HelperMerge => "gx-merge",
            Icon::HelperChecked => "gx-shield",
            // Not a tb- need, so Graphite's in every theme.
            Icon::Details => "gx-sidebar",
        }
    }

    /// A toolbar icon: the theme's icon set may replace it; the others are Graphite's always.
    fn toolbar(self) -> bool {
        self.need().starts_with("tb-")
    }

    /// The icon, enabled or greyed, a one-colour icon in the text colour: a toolbar icon the
    /// theme's own where its icon set has one, else Graphite's; any other Graphite's.
    pub fn image(self, enabled: bool) -> Image<'static> {
        let ink = crate::themeicons::ink(enabled);
        let made = self
            .toolbar()
            .then(|| crate::themeicons::source(self.need(), 32.0, ink, enabled))
            .flatten()
            .or_else(|| crate::themeicons::graphite(self.need(), 32.0, ink, enabled));
        match made {
            Some((src, tint)) => Image::new(src).tint(tint),
            // ponytail: never drawn (every_icon_has_a_graphite_need); an empty drawing, not a panic.
            None => Image::new(egui::ImageSource::Bytes {
                uri: "bytes://mdm/icon/none.svg".into(),
                bytes: egui::load::Bytes::Static(
                    br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#,
                ),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Icon;

    /// Every icon's need is in Graphite's set at 16 and 24 (plan 05.5a).
    #[test]
    fn every_icon_has_a_graphite_need() {
        use Icon::*;
        let all = [
            AddUrl,
            AddTorrent,
            Resume,
            Stop,
            StopAll,
            Delete,
            DeleteCompleted,
            Options,
            Scheduler,
            StartQueue,
            StopQueue,
            HelperVideo,
            HelperMerge,
            HelperChecked,
            Details,
        ];
        // A new Icon breaks this match: add it to `all` above too.
        for icon in all {
            match icon {
                AddUrl | AddTorrent | Resume | Stop | StopAll | Delete | DeleteCompleted
                | Options | Scheduler | StartQueue | StopQueue | HelperVideo | HelperMerge
                | HelperChecked | Details => {}
            }
        }
        for icon in all {
            let need = icon.need();
            assert!(
                crate::themeicons::graphite_has(need, 16)
                    && crate::themeicons::graphite_has(need, 24),
                "{icon:?}: Graphite's set lacks {need}"
            );
        }
    }

    /// The toolbar's Details toggle (plan 06) draws Graphite's `gx-sidebar` in every theme.
    #[test]
    fn details_is_graphites_sidebar() {
        assert_eq!(Icon::Details.need(), "gx-sidebar");
        assert!(!Icon::Details.toolbar());
        assert!(crate::themeicons::graphite_has("gx-sidebar", 16));
        assert!(crate::themeicons::graphite_has("gx-sidebar", 24));
    }
}
