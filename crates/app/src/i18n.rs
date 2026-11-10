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
