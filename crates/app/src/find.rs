//! Downloads → Find (Ctrl-F) / Find Next (F3): IDM's "Find file" dialog
//! (notes/ui-spec/scheduler-and-misc.md §4).

use egui::vec2;

use crate::dialogs::{Outcome, window};
use crate::model::Item;
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, LIGHT_BG, r};

/// What to look for, kept for Find Next.
#[derive(Clone, Debug)]
pub struct Query {
    pub text: String,
    pub name: bool,
    pub description: bool,
    pub site: bool,
    pub case: bool,
    pub whole: bool,
}

impl Default for Query {
    fn default() -> Self {
        Query {
            text: String::new(),
            name: true,
            description: false,
            site: false,
            case: false,
            whole: false,
        }
    }
}

impl Query {
    /// Whether the download matches in one of the chosen fields.
    pub fn matches(&self, item: &Item) -> bool {
        let wanted = self.text.trim();
        if wanted.is_empty() {
            return false;
        }
        let hit = |field: &str| {
            let (field, wanted) = if self.case {
                (field.to_owned(), wanted.to_owned())
            } else {
                (field.to_lowercase(), wanted.to_lowercase())
            };
            if self.whole {
                field == wanted
            } else {
                field.contains(&wanted)
            }
        };
        let site = || {
            let ctx = &item.context;
            [
                Some(item.url.as_str()),
                item.url.host_str(),
                ctx.referer.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(hit)
        };
        (self.name && hit(&item.file_name))
            || (self.description && hit(&item.description))
            || (self.site && site())
    }

    /// The next match after `after` in `ids` (wrapping round), else None.
    pub fn next(&self, ids: &[u64], items: &[Item], after: Option<u64>) -> Option<u64> {
        let start = after
            .and_then(|a| ids.iter().position(|id| *id == a))
            .map_or(0, |p| p + 1);
        (0..ids.len())
            .map(|k| ids[(start + k) % ids.len()])
            .find(|id| items.iter().any(|i| i.id == *id && self.matches(i)))
    }
}

pub struct Find {
    pub query: Query,
    focused: bool,
}

impl Find {
    pub fn new(last: Option<&Query>) -> Self {
        Find {
            query: last.cloned().unwrap_or_default(),
            focused: false,
        }
    }

    /// `Done` on Find.
    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<()> {
        // The mockup's layout: the Find row on the label column, the check boxes 27 apart, a
        // hairline before the two matching rules, the footer.
        const W: f32 = 480.0;
        const BODY: f32 = 210.0;
        window(
            ctx,
            "find",
            "Find file",
            [W, BODY + winui::FOOTER_H],
            false,
            LIGHT_BG(),
            |ui, o| {
                let p = ui.painter().clone();
                let q = &mut self.query;
                let e = winui::EDGE;
                let fx = winui::form(&p, e, &["Find:"], false);
                winui::form_label(&p, o, e, 16.0, winui::CONTROL_H, "Find:", true);
                let edit = winui::edit_light(
                    ui,
                    r(o, fx, 16.0, W - e - fx, winui::CONTROL_H),
                    &mut q.text,
                    true,
                    false,
                );
                if !self.focused {
                    edit.request_focus();
                    self.focused = true;
                }
                let checks: [(&mut bool, &str, f32); 3] = [
                    (&mut q.name, "File name or part of the name", 58.0),
                    (
                        &mut q.description,
                        "Description or part of the description",
                        85.0,
                    ),
                    (
                        &mut q.site,
                        "Site name/Download link/Parent web page/Referer",
                        112.0,
                    ),
                ];
                for (on, caption, y) in checks {
                    winui::checkbox_light(ui, o + vec2(e, y), on, caption, true);
                }
                winui::rule(&p, o, e, 140.0, W - 2.0 * e);
                winui::checkbox_light(ui, o + vec2(e, 152.0), &mut q.case, "Match case", true);
                winui::checkbox_light(
                    ui,
                    o + vec2(e, 179.0),
                    &mut q.whole,
                    "Match whole string only",
                    true,
                );
                let can = !q.text.trim().is_empty() && (q.name || q.description || q.site);
                // Enter finds (while it can), Escape cancels, as before.
                let pressed = winui::footer(
                    ui,
                    o,
                    W,
                    BODY,
                    &mut [
                        FooterItem::Button(
                            // i18n: verb: button that starts the search
                            FooterButton::new(
                                "Find",
                                if can {
                                    ButtonKind::Default
                                } else {
                                    ButtonKind::Disabled
                                },
                            )
                            .enter(true)
                            .tip(
                                !can,
                                // i18n: tooltip of the greyed Find button
                                "Type what to find, and tick at least one place to look",
                            ),
                        ),
                        FooterItem::Button(
                            FooterButton::new("Cancel", ButtonKind::Normal).escape(true),
                        ),
                    ],
                );
                match pressed {
                    Some(1) => Outcome::Closed,
                    Some(0) if can => Outcome::Done(()),
                    _ => Outcome::Open,
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64, name: &str, description: &str) -> Item {
        let mut i = Item::new(
            id,
            url::Url::parse(&format!("https://files.example.com/{name}")).unwrap(),
            std::path::PathBuf::from(name),
            crate::model::Category::General,
        );
        i.description = description.into();
        i
    }

    #[test]
    fn finds_in_the_chosen_fields_and_wraps_round() {
        let items = vec![
            item(1, "setup.exe", ""),
            item(2, "Movie.mkv", "holiday"),
            item(3, "setup2.exe", ""),
        ];
        let ids = [1, 2, 3];
        let q = Query {
            text: "setup".into(),
            ..Query::default()
        };
        assert_eq!(q.next(&ids, &items, None), Some(1));
        assert_eq!(q.next(&ids, &items, Some(1)), Some(3));
        assert_eq!(q.next(&ids, &items, Some(3)), Some(1), "wraps round");
        let q = Query {
            text: "HOLIDAY".into(),
            name: false,
            description: true,
            ..Query::default()
        };
        assert_eq!(q.next(&ids, &items, None), Some(2));
        let q = Query { case: true, ..q };
        assert_eq!(q.next(&ids, &items, None), None, "match case");
        let q = Query {
            text: "files.example.com".into(),
            name: false,
            site: true,
            whole: true,
            ..Query::default()
        };
        assert!(
            q.next(&ids, &items, None).is_some(),
            "the host as a whole string"
        );
    }
}
