//! The interface's texts: English only (the translations, the language picker and their tools
//! live on the `archive/multiplatform-i18n` branch). Every user-visible text still goes through
//! here, so the call sites keep one shape: `t("Options")` for a label, `tc("menu", "&Options")`
//! where the same English has different meanings, `tn(n, "{n} file", "{n} files")` for counts,
//! `fmt` to fill a template's `{name}` slots. Text that is data (file names, addresses, paths)
//! goes in as an isolate (`ltr`, `isolate`), so an Arabic file name can't reorder the sentence.
//!
//! craft-i18n still does the plurals, the isolates and the counts' number format (grouped as
//! the system region writes English numbers, #128).

use std::sync::OnceLock;

use craft_i18n::{Digits, Format, I18n, Lang, Region};

/// craft-i18n with no catalog: English, its plural rule and the region's number format.
fn i18n() -> &'static I18n {
    static I: OnceLock<I18n> = OnceLock::new();
    I.get_or_init(|| {
        let (i, _) = I18n::new(&[], &[]);
        i.set_format(number_format(&Region::system()));
        i
    })
}

/// How counts write their numbers (#128): as the system region does when it writes English
/// (a separator the user set in Windows too), else as CLDR writes English (`10,200`).
fn number_format(system: &Region) -> Format {
    let english = system
        .numbers
        .split(['-', '_'])
        .next()
        .is_some_and(|l| l.eq_ignore_ascii_case("en"));
    if english {
        Format::new(system, Digits::Locale)
    } else {
        Format::from_tag("en")
    }
}

/// Marks `source` as interface text where it is shown later (a menu label chosen here, drawn
/// with [`t`] when the menu is). Returns it unchanged.
pub const fn mark(source: &'static str) -> &'static str {
    source
}

/// The interface text for `source`: the English itself.
pub const fn t(source: &str) -> &str {
    source
}

/// `source` in `context` (the same English with different meanings): the English itself.
pub const fn tc<'a>(_context: &str, source: &'a str) -> &'a str {
    source
}

/// A count: `one` / `other` templates with `{n}`.
pub fn tn(n: u64, one: &str, other: &str) -> String {
    i18n().trn(Lang::EN, n, one, other)
}

/// Fill a template's `{name}` slots.
pub fn fmt(template: &str, args: &[(&str, &str)]) -> String {
    craft_i18n::fmt(template, args)
}

/// Data that reads left to right (a path, an address, a size, a date, a speed): a
/// left-to-right isolate, inside a sentence or alone.
pub fn ltr(value: &str) -> String {
    craft_i18n::ltr(value)
}

/// Data with a direction of its own (a file name, a description) shown alone where interface
/// text goes: a first-strong isolate, so an Arabic name reads right to left in the window.
pub fn isolate(value: &str) -> String {
    craft_i18n::isolate(value)
}

/// Text MDM hands to the system to draw (upstream issue 919): a window's title (the taskbar,
/// its preview and tip, Alt+Tab) and the notification-area icon's tip. MDM's own drawing keeps
/// a file name or a number apart from the sentence around it with Unicode's isolates (FSI,
/// LRI, RLI … PDI: [`isolate`], [`ltr`], [`fmt`]), which Windows 10's shell draws as boxes.
/// They go:
///
/// - in text with no right-to-left letter, without a trace: there is no order to keep;
/// - elsewhere each isolate becomes the older controls that keep the same order, which Windows
///   draws as nothing: an embedding (LRE U+202A or RLE U+202B … PDF U+202C) in the value's own
///   direction, so a Hebrew name with a Latin ".txt" reads as it does in MDM, and a
///   left-to-right mark (LRM U+200E) on each side of it, so "71%" stays before it. The first
///   mark also tells a shell that finds a line's direction from its first letter that the
///   (English) interface reads left to right.
///
/// What it gives has no isolate, so it gives that back as it is.
pub fn os_text(text: &str) -> String {
    const LRI: char = '\u{2066}';
    const RLI: char = '\u{2067}';
    const FSI: char = '\u{2068}';
    const PDI: char = '\u{2069}';
    const PDF: char = '\u{202C}';
    const LRM: char = '\u{200E}';
    const RLM: char = '\u{200F}';
    let isolate_mark = |c: char| matches!(c, LRI | RLI | FSI | PDI);
    if !text.chars().any(isolate_mark) {
        return text.to_owned();
    }
    if !text.chars().any(|c| strong(c) == Some(true)) {
        return text.chars().filter(|c| !isolate_mark(*c)).collect();
    }
    let mark = |rtl: bool| if rtl { RLM } else { LRM };
    let embedding = |rtl: bool| if rtl { '\u{202B}' } else { '\u{202A}' };
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 12);
    // Each line starts left to right, whatever its first letter.
    out.push(LRM);
    // The direction of each open isolate, the paragraph's (left to right) first.
    let mut open = vec![false];
    for (i, &c) in chars.iter().enumerate() {
        // The direction of the text this character is in.
        let here = *open.last().unwrap_or(&false);
        match c {
            LRI | RLI | FSI => {
                let inner = match c {
                    LRI => false,
                    RLI => true,
                    _ => first_strong(&chars[i + 1..]).unwrap_or(here),
                };
                out.push(mark(here));
                out.push(embedding(inner));
                open.push(inner);
            }
            // A PDI with no isolate open closes nothing.
            PDI if open.len() > 1 => {
                open.pop();
                out.push(PDF);
                out.push(mark(*open.last().unwrap_or(&false)));
            }
            PDI => {}
            '\n' => {
                out.push(c);
                out.push(LRM);
            }
            _ => out.push(c),
        }
    }
    // One left open ends with the text.
    while open.len() > 1 {
        open.pop();
        out.push(PDF);
        out.push(mark(*open.last().unwrap_or(&false)));
    }
    out
}

/// A letter's direction: right to left (Hebrew, Arabic, Syriac, Thaana…) `Some(true)`, left to
/// right `Some(false)`; digits, spaces and punctuation have none.
fn strong(c: char) -> Option<bool> {
    use unicode_bidi::{BidiClass, bidi_class};
    match bidi_class(c) {
        BidiClass::R | BidiClass::AL => Some(true),
        BidiClass::L => Some(false),
        _ => None,
    }
}

/// The direction of a first-strong isolate's first letter, `rest` starting right after its FSI:
/// isolates inside it are skipped, its PDI ends the search.
fn first_strong(rest: &[char]) -> Option<bool> {
    let mut depth = 0usize;
    for &c in rest {
        match c {
            '\u{2066}' | '\u{2067}' | '\u{2068}' => depth += 1,
            '\u{2069}' if depth == 0 => return None,
            '\u{2069}' => depth -= 1,
            _ if depth == 0 => {
                if let Some(rtl) = strong(c) {
                    return Some(rtl);
                }
            }
            _ => {}
        }
    }
    None
}

/// A whole number, plain digits as IDM writes it: `10200`.
pub fn integer(n: u64) -> String {
    n.to_string()
}

/// `v` with `decimals` digits after the decimal point: `1234.50`, as IDM.
pub fn decimal(v: f64, decimals: usize) -> String {
    format!("{v:.decimals$}")
}

/// `ratio` as a percentage with `decimals` digits: `13.38%`, as IDM.
pub fn percent(ratio: f64, decimals: usize) -> String {
    format!("{:.decimals$}%", 100.0 * ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LRI: char = '\u{2066}';
    const FSI: char = '\u{2068}';
    const PDI: char = '\u{2069}';

    #[test]
    fn data_goes_in_as_isolates() {
        // A path or an address reads left to right; a file name takes its own direction.
        assert_eq!(ltr(r"C:\dl\a.zip"), format!(r"{LRI}C:\dl\a.zip{PDI}"));
        assert_eq!(isolate("ملف.zip"), format!("{FSI}ملف.zip{PDI}"));
        // English values get no marks; an Arabic name is isolated, so it can't reorder the
        // sentence.
        assert_eq!(
            fmt("Delete \"{name}\"?", &[("name", "a.zip")]),
            "Delete \"a.zip\"?"
        );
        assert_eq!(
            fmt("Delete \"{name}\"?", &[("name", "ملف.zip")]),
            format!("Delete \"{FSI}ملف.zip{PDI}\"?")
        );
    }

    /// The order a shell shows `text` in, without the invisible marks: the Unicode Bidi
    /// Algorithm's, left to right, or found from its first letter (`by_itself`).
    fn shown(text: &str, by_itself: bool) -> String {
        use unicode_bidi::{BidiInfo, Level};
        let info = BidiInfo::new(text, (!by_itself).then(Level::ltr));
        let para = &info.paragraphs[0];
        info.reorder_line(para, para.range.clone())
            .chars()
            .filter(|c| {
                !matches!(*c, '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
            })
            .collect()
    }

    /// Upstream issue 919: what goes to the system has no isolate (Windows 10's taskbar drew
    /// them as boxes), and reads in the same order as MDM's own drawing of it.
    #[test]
    fn text_for_the_system_has_no_isolates_and_keeps_its_order() {
        // The report's title: a Latin name. Only the isolates go.
        let title = format!("71% {}", isolate("TeraLace_1.2.3_universal.apk"));
        assert_eq!(os_text(&title), "71% TeraLace_1.2.3_universal.apk");
        // No isolate: untouched.
        assert_eq!(os_text("Mister Download Manager"), "Mister Download Manager");
        assert_eq!(os_text("مدير التنزيل"), "مدير التنزيل");
        let cases = [
            // A Hebrew or Arabic name can't pull "71%" over to its side.
            format!("71% {}", isolate("שלום (1).txt")),
            format!("{} 71%", isolate("שלום")),
            format!("45% {}", isolate("ملف 1.zip")),
            fmt(
                "{file} was saved to {folder}.",
                &[("file", "ملف (2).mp4"), ("folder", &ltr(r"C:\dl\new"))],
            ),
            // The tray's tip, one count per line; a name with its own isolate inside.
            format!(
                "Mister Download Manager\n{} {}",
                isolate("ملف"),
                ltr("1.2 MB/s")
            ),
            format!("x {}", isolate(&format!("ب {} ج", ltr("1 2")))),
        ];
        for text in &cases {
            let os = os_text(text);
            assert!(
                !os.chars().any(|c| ('\u{2066}'..='\u{2069}').contains(&c)),
                "{os:?}"
            );
            for (mdm, sys) in text.split('\n').zip(os.split('\n')) {
                let want = shown(mdm, false);
                assert_eq!(shown(sys, false), want, "{text:?} as {os:?}");
                // A shell that finds the direction itself, from the first letter, too.
                assert_eq!(shown(sys, true), want, "{text:?} as {os:?}, by itself");
            }
            // Given its own result, it gives it back.
            assert_eq!(os_text(&os), os);
        }
        // An isolate left open, a PDI that closes nothing.
        assert_eq!(os_text(&format!("{FSI}abc {PDI}{PDI}d {LRI}e")), "abc d e");
        let odd = os_text(&format!("{FSI}ب {PDI}{PDI}d {LRI}e"));
        assert!(!odd.chars().any(|c| ('\u{2066}'..='\u{2069}').contains(&c)));
    }

    #[test]
    fn english_texts_and_numbers() {
        assert_eq!(t("Options"), "Options");
        assert_eq!(tc("menu", "&Options"), "&Options");
        assert_eq!(
            craft_i18n::strip_isolates(&tn(1, "{n} file", "{n} files")),
            "1 file"
        );
        assert_eq!(
            craft_i18n::strip_isolates(&tn(3, "{n} file", "{n} files")),
            "3 files"
        );
        assert_eq!(integer(10_200), "10200");
        assert_eq!(decimal(1234.5, 2), "1234.50");
        assert_eq!(percent(0.1338, 2), "13.38%");
    }

    /// #128: counts group as the region writes English, else as CLDR writes English.
    #[test]
    fn counts_follow_the_region_when_it_writes_english() {
        assert_eq!(
            number_format(&Region::from_tag("en-US")).integer(10_200),
            "10,200"
        );
        assert_eq!(
            number_format(&Region::from_tag("de-DE")).integer(10_200),
            "10,200"
        );
        let mut own = Region::from_tag("en-GB");
        own.grouping = Some(Some('\''));
        assert_eq!(number_format(&own).integer(10_200), "10'200");
    }
}
