//! File → Add batch download (notes/ui-spec/scheduler-and-misc.md §3): an address with an
//! asterisk, replaced by numbers or letters over a range, previewed before it is added.

use egui::{Pos2, Ui, vec2};

use crate::dialogs::{Outcome, window};
use crate::winui::{self, ButtonKind, FooterButton, FooterItem, LIGHT_BG, LIGHT_TEXT, r, text};

/// The window's width and where its footer starts (the mockup's layout).
const W: f32 = 540.0;
const BODY: f32 = 448.0;
/// Its left edge, and its rows' tops: 30 high, 10 apart, under the note.
const LX: f32 = winui::EDGE;
const ADDRESS_Y: f32 = 96.0;
const REPLACE_Y: f32 = 136.0;
const FROM_Y: f32 = 176.0;
const AUTH_Y: f32 = 216.0;
const LOGIN_Y: f32 = 246.0;
/// The first preview row's top (the second 26 under it, the last 74).
const PREVIEW_Y: f32 = 336.0;

pub struct Batch {
    address: String,
    letters: bool,
    from: String,
    to: String,
    size: String,
    auth: bool,
    login: String,
    password: String,
    focused: bool,
    /// The pattern's addresses (or why none), and the inputs they were made from.
    cached: Option<(PatternKey, Result<Vec<url::Url>, &'static str>)>,
    /// The clipboard's address, read off the window's thread (#661).
    clip: crate::clipboard::Prefill,
}

/// What a pattern's addresses are made from: address, letters, from, to, size.
type PatternKey = (String, bool, String, String, String);

/// What OK adds: the addresses, and the login if one was given.
pub struct Added {
    pub urls: Vec<url::Url>,
    pub login: Option<(String, String)>,
}

impl Batch {
    /// Pre-filled from the clipboard when it holds an http(s) or ftp(s) address, as IDM's: read
    /// off the window's thread, the address put in the box on the frame it comes.
    pub fn new(from_clipboard: bool) -> Self {
        Batch {
            address: String::new(),
            letters: false,
            from: "0".into(),
            to: "10".into(),
            size: "1".into(),
            auth: false,
            login: String::new(),
            password: String::new(),
            focused: false,
            cached: None,
            clip: if from_clipboard {
                crate::clipboard::Prefill::ask()
            } else {
                Default::default()
            },
        }
    }

    /// The clipboard's text as the address, when it is one MDM downloads and the box is still
    /// empty (nothing typed meanwhile).
    fn fill_from_clipboard(&mut self, clip: &str) {
        let clip = clip.trim();
        if self.address.is_empty() && url::Url::parse(clip).is_ok_and(|u| mdm_engine::supported(&u))
        {
            self.address = clip.to_owned();
        }
    }

    /// The addresses the pattern stands for (empty while it isn't valid).
    #[cfg(test)]
    fn expand(&self) -> Vec<url::Url> {
        self.pattern().unwrap_or_default()
    }

    /// The addresses the pattern stands for, or why there are none (English, marked).
    fn pattern(&self) -> Result<Vec<url::Url>, &'static str> {
        let Some((head, tail)) = self.address.trim().split_once('*') else {
            // i18n: why Batch download's OK is greyed
            return Err(crate::i18n::mark(
                "Put a * in the address where the numbers or letters go",
            ));
        };
        if tail.contains('*') {
            // i18n: why Batch download's OK is greyed
            return Err(crate::i18n::mark("Use one * only"));
        }
        let width: usize = self.size.parse().unwrap_or(1).clamp(1, 10);
        let parts: Vec<String> = if self.letters {
            let (Some(a), Some(b)) = (self.from.chars().next(), self.to.chars().next()) else {
                // i18n: why Batch download's OK is greyed
                return Err(crate::i18n::mark("Type a letter in From and in To"));
            };
            if !a.is_ascii_alphabetic() || !b.is_ascii_alphabetic() {
                return Err(crate::i18n::mark("Type a letter in From and in To"));
            }
            // In From's case (Z to a would run through [\]^_` between the cases).
            let b = if a.is_ascii_uppercase() {
                b.to_ascii_uppercase()
            } else {
                b.to_ascii_lowercase()
            };
            if a > b {
                // i18n: why Batch download's OK is greyed
                return Err(crate::i18n::mark("From must come before To"));
            }
            // One letter per file; the wildcard size applies to numbers.
            (a..=b).map(|c| c.to_string()).collect()
        } else {
            let (Ok(a), Ok(b)) = (self.from.parse::<u64>(), self.to.parse::<u64>()) else {
                // i18n: why Batch download's OK is greyed
                return Err(crate::i18n::mark("Type a number in From and in To"));
            };
            if a > b {
                return Err(crate::i18n::mark("From must come before To"));
            }
            if b - a > 100_000 {
                // i18n: why Batch download's OK is greyed
                return Err(crate::i18n::mark(
                    "That is more than 100,000 files: make the range smaller",
                ));
            }
            (a..=b).map(|n| format!("{n:0width$}")).collect()
        };
        let urls: Vec<url::Url> = parts
            .iter()
            .filter_map(|p| crate::model::parse_address(&format!("{head}{p}{tail}")).ok())
            .filter(mdm_engine::supported)
            .collect();
        if urls.is_empty() {
            // i18n: why Batch download's OK is greyed
            return Err(crate::i18n::mark(
                "The address must start with http://, https:// or ftp://",
            ));
        }
        Ok(urls)
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Outcome<Added> {
        if let Some(clip) = self.clip.take() {
            self.fill_from_clipboard(&clip);
        }
        // Worked out again only when the pattern changes (up to 100,000 addresses).
        let key = (
            self.address.clone(),
            self.letters,
            self.from.clone(),
            self.to.clone(),
            self.size.clone(),
        );
        if self.cached.as_ref().is_none_or(|(k, _)| *k != key) {
            let made = self.pattern();
            self.cached = Some((key, made));
        }
        // Lent to the body (not copied each frame), put back after.
        let cached = self.cached.take();
        let (urls, why): (&[url::Url], Option<&'static str>) = match cached.as_ref().map(|(_, r)| r)
        {
            Some(Ok(u)) => (u, None),
            Some(Err(e)) => (&[], Some(*e)),
            None => (&[], None),
        };
        let outcome = window(
            ctx,
            "batch",
            "Batch download",
            [W, BODY + winui::FOOTER_H],
            false,
            LIGHT_BG(),
            |ui, o| self.body(ui, o, urls, why),
        );
        let out = match outcome {
            Outcome::Done(()) => Outcome::Done(Added {
                urls: urls.to_vec(),
                login: self
                    .auth
                    .then(|| (self.login.clone(), self.password.clone())),
            }),
            Outcome::Closed => Outcome::Closed,
            Outcome::Open => Outcome::Open,
        };
        self.cached = cached;
        out
    }

    /// `why`: why the pattern makes no addresses (OK's tooltip).
    fn body(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        urls: &[url::Url],
        why: Option<&'static str>,
    ) -> Outcome<()> {
        let p = ui.painter().clone();
        let h = winui::CONTROL_H;
        let right = W - LX;
        // The note in muted text, then the rows on the labels' column (the mockup's).
        let lines = winui::paragraph(
            &p,
            o.x + LX,
            o.y + 29.0,
            crate::i18n::t(
                "It's possible to add a group of sequential file names like img001.jpg, img002.jpg, etc.,\nimg100.jpg to MDM download queue. Use the asterisk wildcard for the file name pattern.",
            ),
            winui::MUTED(),
            o.x + right,
            15.0,
        );
        text(
            &p,
            o.x + LX,
            o.y + 29.0 + 15.0 * lines as f32,
            &crate::i18n::fmt(
                // i18n: {address} = an example web address with a * in the file name
                crate::i18n::t("For example:  {address}"),
                &[(
                    "address",
                    &crate::i18n::ltr("http://www.example.com/pictures/img*.jpg"),
                )],
            ),
            winui::MUTED(),
        );
        let fx = winui::form(
            &p,
            LX,
            &[
                // i18n: noun: the web address (URL) pattern
                "Address",
                // i18n: label: what the * in the address is replaced with (numbers or letters)
                "Replace asterisk to",
                // i18n: label before the first value of the range (an input box follows)
                "From:",
                // i18n: noun: the user name for the server (an input box follows)
                "Login",
                "Password",
                // i18n: the first file name the pattern gives (its name follows)
                "First file:",
                "Second file:",
                "Last file:",
            ],
            false,
        );
        winui::form_label(&p, o, LX, ADDRESS_Y, h, "Address", true);
        let edit = winui::edit_light(
            ui,
            r(o, fx, ADDRESS_Y, right - fx, h),
            &mut self.address,
            true,
            false,
        );
        if !self.focused {
            edit.request_focus();
            self.focused = true;
        }

        // Replace asterisk to: Numbers / Letters, then From, To, Wildcard size.
        winui::form_label(&p, o, LX, REPLACE_Y, h, "Replace asterisk to", true);
        let radio_y = REPLACE_Y + ((h - 13.0) / 2.0).round();
        // i18n: the * in the address runs through numbers
        let numbers = winui::radio_light(ui, o + vec2(fx, radio_y), !self.letters, "Numbers", true);
        // i18n: the * in the address runs through letters
        let letters = winui::radio_light(
            ui,
            o + vec2(fx + 115.0, radio_y),
            self.letters,
            "Letters",
            true,
        );
        let key = winui::radio_keys(ui, &[numbers.clone(), letters.clone()]);
        if (numbers.clicked() || key == Some(0)) && self.letters {
            self.letters = false;
            self.from = "0".into();
            self.to = "10".into();
        }
        if (letters.clicked() || key == Some(1)) && !self.letters {
            self.letters = true;
            self.from = "a".into();
            self.to = "z".into();
        }
        // From, then To and Wildcard size on the same row, each label 12 after the box before it.
        winui::form_label(&p, o, LX, FROM_Y, h, "From:", true);
        let base = o.y + winui::baseline_in(FROM_Y, h);
        self.spin(ui, o, fx, 70.0, "from");
        // i18n: label before the last value of the range (an input box follows)
        let to_label = crate::i18n::t("To:");
        let to_label_x = fx + 70.0 + 12.0;
        text(&p, o.x + to_label_x, base, to_label, LIGHT_TEXT());
        let to_x = (to_label_x + winui::text_width(&p, to_label) + 8.0).ceil();
        self.spin(ui, o, to_x, 70.0, "to");
        // The size box at the row's end; a longer label is cut with "..." before it, whole on
        // hover.
        let size_x = right - 60.0;
        let size_label = crate::i18n::t("Wildcard size");
        let (rect, cut) = winui::label_into(
            &p,
            o.x + to_x + 70.0 + 12.0,
            base,
            size_label,
            LIGHT_TEXT(),
            o.x + size_x,
            o.x + to_x + 70.0 + 12.0,
        );
        if cut {
            winui::full_on_hover(ui, rect, "batch-size", size_label);
        }
        self.spin(ui, o, size_x, 60.0, "size");

        // Use authorization, and its two rows (greyed while it is off).
        winui::checkbox_light(
            ui,
            o + vec2(fx, AUTH_Y + 3.0),
            &mut self.auth,
            "Use authorization",
            true,
        );
        let on = self.auth;
        winui::form_label(&p, o, LX, LOGIN_Y, h, "Login", on);
        winui::edit_light(
            ui,
            r(o, fx, LOGIN_Y, right - fx, h),
            &mut self.login,
            on,
            false,
        );
        winui::form_label(&p, o, LX, LOGIN_Y + 40.0, h, "Password", on);
        winui::password_light(
            ui,
            r(o, fx, LOGIN_Y + 40.0, right - fx, h),
            &mut self.password,
            on,
        );

        // Preview: first, second and last file.
        let name = |i: Option<&url::Url>| {
            i.map(|u| mdm_engine::naming::file_name(None, u))
                .unwrap_or_default()
        };
        // Under a rule (it was its own box): the first, second and last file, and how many.
        winui::rule(&p, o, LX, PREVIEW_Y - 10.0, right - LX);
        let rows = [
            ("First file:", PREVIEW_Y, name(urls.first())),
            ("Second file:", PREVIEW_Y + 26.0, name(urls.get(1))),
            (
                "Last file:",
                PREVIEW_Y + 74.0,
                if urls.len() > 1 {
                    name(urls.last())
                } else {
                    String::new()
                },
            ),
        ];
        for (label, y, value) in rows {
            let rc = r(o, fx, y, right - fx, 22.0);
            winui::form_label(&p, o, LX, y, 22.0, label, true);
            // The shared read-only field: the address can be selected and copied, and screen
            // readers read it.
            winui::readonly_file(ui, rc, &value, ("batch-preview", y as i32));
        }
        let gap = o.y + winui::baseline_in(PREVIEW_Y + 52.0, 18.0);
        text(&p, o.x + LX + 12.0, gap, "...", LIGHT_TEXT());
        if urls.len() > 2 {
            // i18n: how many files the pattern gives in all
            let more = crate::i18n::tn(urls.len() as u64, "{n} file", "{n} files");
            text(&p, o.x + fx, gap, &more, LIGHT_TEXT());
        }

        let valid = !urls.is_empty();
        let ok_kind = if valid {
            ButtonKind::Default
        } else {
            ButtonKind::Disabled
        };
        // Enter OK (a greyed OK ignores it), Escape Cancel, as before.
        match winui::footer(
            ui,
            o,
            W,
            BODY,
            &mut [
                FooterItem::Button(
                    FooterButton::new("OK", ok_kind)
                        .primary()
                        .enter(true)
                        .tip(!valid, why.unwrap_or("")),
                ),
                FooterItem::Button(FooterButton::new("Cancel", ButtonKind::Normal).escape(true)),
            ],
        ) {
            Some(1) => Outcome::Closed,
            Some(_) if valid => Outcome::Done(()),
            _ => Outcome::Open,
        }
    }

    /// A spin box (its arrows inside) at English `x`, `w` wide, on the From row.
    fn spin(&mut self, ui: &mut Ui, o: Pos2, x: f32, w: f32, which: &str) {
        let letters = self.letters && which != "size";
        let value = match which {
            "from" => &mut self.from,
            "to" => &mut self.to,
            _ => &mut self.size,
        };
        let rc = r(o, x, FROM_Y, w, winui::CONTROL_H);
        let id = format!("batch-{which}");
        // The wildcard size runs 1 to 10 (what expand() makes); from and to, any number.
        let (min, max) = if which == "size" {
            (1, 10)
        } else {
            (0, 999_999_999)
        };
        if !letters {
            winui::spin_box(ui, rc, value, min, max, true, &id);
            return;
        }
        let (_, step) = winui::spin_frame(ui, rc, true, &id, |ui, text| {
            winui::field_text(ui, text, value, true)
        });
        value.retain(|c| c.is_ascii_alphabetic());
        value.truncate(1);
        if step != 0 {
            *value = {
                let c = value.chars().next().unwrap_or('a');
                // Within the letter's own case.
                let (lo, hi) = if c.is_ascii_uppercase() {
                    (b'A', b'Z')
                } else {
                    (b'a', b'z')
                };
                let n = (c as u8 as i32 + step).clamp(lo as i32, hi as i32) as u8;
                (n as char).to_string()
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn batch(address: &str) -> Batch {
        Batch {
            address: address.into(),
            ..Batch::new(false)
        }
    }

    /// #661: the clipboard's address comes a frame or more after the window opens: it fills
    /// an empty box only, and only with an address MDM downloads.
    #[test]
    fn the_clipboard_fills_only_an_empty_box() {
        let mut b = batch("");
        b.fill_from_clipboard(
            " http://x/img*.jpg
",
        );
        assert_eq!(b.address, "http://x/img*.jpg");
        let mut b = batch("http://typed/*.png");
        b.fill_from_clipboard("http://x/img*.jpg");
        assert_eq!(b.address, "http://typed/*.png");
        let mut b = batch("");
        b.fill_from_clipboard("not an address");
        assert_eq!(b.address, "");
    }

    #[test]
    fn numbers_with_a_wildcard_size() {
        let mut b = batch("http://x/img*.jpg");
        b.from = "1".into();
        b.to = "100".into();
        b.size = "3".into();
        let urls = b.expand();
        assert_eq!(urls.len(), 100);
        assert_eq!(urls[0].as_str(), "http://x/img001.jpg");
        assert_eq!(urls[99].as_str(), "http://x/img100.jpg");
    }

    #[test]
    fn letters_and_invalid_patterns() {
        let mut b = batch("http://x/part-*.rar");
        b.letters = true;
        b.from = "a".into();
        b.to = "c".into();
        let urls: Vec<String> = b.expand().iter().map(|u| u.to_string()).collect();
        assert_eq!(
            urls,
            [
                "http://x/part-a.rar",
                "http://x/part-b.rar",
                "http://x/part-c.rar"
            ]
        );
        // Mixed case: the range keeps From's case, never the signs between the cases.
        b.from = "y".into();
        b.to = "Z".into();
        let urls: Vec<String> = b.expand().iter().map(|u| u.to_string()).collect();
        assert_eq!(urls, ["http://x/part-y.rar", "http://x/part-z.rar"]);
        b.from = "Y".into();
        b.to = "a".into();
        assert!(b.expand().is_empty(), "Y to A: nothing between the cases");
        assert!(batch("http://x/no-asterisk.jpg").expand().is_empty());
        assert!(
            batch("http://x/*/*.jpg").expand().is_empty(),
            "one asterisk only"
        );
        assert_eq!(
            batch("ftp://x/*.jpg").expand().len(),
            11,
            "ftp addresses too"
        );
    }
}
