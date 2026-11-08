//! Number locale: the conventions numbers and dates are written and typed with.
//!
//! en-US is the default and the reference: every caller that does not pass a locale gets en-US,
//! byte for byte. German and French change the decimal and thousands separators, the month and
//! weekday names, the booleans and the short date order (`31.12.2026`, `31/12/2026`). Format
//! codes themselves are stored en-US (`#,##0.00`, `m/d/yyyy`), as in an `.xlsx` file.
//!
//! The locale is a session setting, not part of a workbook: opening a file never changes it.

use serde::{Deserialize, Serialize};

use crate::date::{MONTHS, WEEKDAYS};

/// The conventions numbers and dates are written with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    /// en-US: `.` decimal, `,` thousands, `m/d/yyyy`, English names.
    #[default]
    En,
    /// de-DE: `,` decimal, `.` thousands, `dd.mm.yyyy`, German names.
    De,
    /// fr-FR: `,` decimal, narrow no-break space (U+202F) thousands, `dd/mm/yyyy`, French names.
    Fr,
}

/// Three-letter English month names.
const EN_MONTHS_SHORT: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/// Three-letter English weekday names (index 0 = Sunday).
const EN_WEEKDAYS_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
/// Full German month names.
const DE_MONTHS: [&str; 12] = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];
/// Short German month names, the forms German Excel shows for `MMM`.
const DE_MONTHS_SHORT: [&str; 12] = ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"];
/// Full German weekday names (index 0 = Sunday).
const DE_WEEKDAYS: [&str; 7] = ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"];
/// Short German weekday names, the two-letter forms German Excel shows for `TTT`.
const DE_WEEKDAYS_SHORT: [&str; 7] = ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"];
/// Full French month names (lower case, as French Excel writes them).
const FR_MONTHS: [&str; 12] =
    ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
/// Short French month names, the forms French Excel shows for `mmm`.
const FR_MONTHS_SHORT: [&str; 12] = ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."];
/// Full French weekday names (index 0 = Sunday).
const FR_WEEKDAYS: [&str; 7] = ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"];
/// Short French weekday names, the forms French Excel shows for `jjj`.
const FR_WEEKDAYS_SHORT: [&str; 7] = ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."];

/// The narrow no-break space French uses to group thousands.
pub const NARROW_NBSP: char = '\u{202F}';

impl Locale {
    /// Every locale, en-US first.
    pub const ALL: [Locale; 3] = [Locale::En, Locale::De, Locale::Fr];

    /// The locale's tag: `"en"`, `"de"` or `"fr"`.
    pub const fn tag(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::De => "de",
            Locale::Fr => "fr",
        }
    }

    /// The locale of a BCP 47-ish tag: `"de"`, `"de-DE"`, `"fr_CH"`, `"en-GB"`… A tag whose
    /// language part is none of `en`, `de` and `fr`, and an empty tag, is `None`.
    pub fn from_tag(tag: &str) -> Option<Self> {
        let lang = tag.trim().split(['-', '_']).next().unwrap_or_default();
        Locale::ALL.into_iter().find(|l| lang.eq_ignore_ascii_case(l.tag()))
    }

    /// The decimal separator: `.` in en-US, `,` in German and French.
    pub const fn decimal(self) -> char {
        match self {
            Locale::En => '.',
            Locale::De | Locale::Fr => ',',
        }
    }

    /// The thousands separator: `,` in en-US, `.` in German, U+202F in French.
    pub const fn group(self) -> char {
        match self {
            Locale::En => ',',
            Locale::De => '.',
            Locale::Fr => NARROW_NBSP,
        }
    }

    /// The separator of a short date: `/` in en-US and French, `.` in German.
    pub const fn date_separator(self) -> char {
        match self {
            Locale::En | Locale::Fr => '/',
            Locale::De => '.',
        }
    }

    /// True when a short date is written day first (`31.12.2026`, `31/12/2026`).
    pub const fn day_first(self) -> bool {
        !matches!(self, Locale::En)
    }

    /// How a boolean is shown: `TRUE`/`FALSE`, `WAHR`/`FALSCH`, `VRAI`/`FAUX`.
    pub const fn bool_name(self, b: bool) -> &'static str {
        match (self, b) {
            (Locale::En, true) => "TRUE",
            (Locale::En, false) => "FALSE",
            (Locale::De, true) => "WAHR",
            (Locale::De, false) => "FALSCH",
            (Locale::Fr, true) => "VRAI",
            (Locale::Fr, false) => "FAUX",
        }
    }

    /// The month name for a month number 1–12 (`short` = the `mmm` form), or `""` for a number
    /// outside that range.
    pub fn month_name(self, month: u32, short: bool) -> &'static str {
        let table: &[&'static str; 12] = match (self, short) {
            (Locale::En, false) => &MONTHS,
            (Locale::En, true) => &EN_MONTHS_SHORT,
            (Locale::De, false) => &DE_MONTHS,
            (Locale::De, true) => &DE_MONTHS_SHORT,
            (Locale::Fr, false) => &FR_MONTHS,
            (Locale::Fr, true) => &FR_MONTHS_SHORT,
        };
        usize::try_from(month).ok().and_then(|m| m.checked_sub(1)).and_then(|i| table.get(i)).copied().unwrap_or("")
    }

    /// The weekday name for a weekday number (0 = Sunday, as [`crate::date::DateTime::weekday`]),
    /// or `""` for a number outside 0–6.
    pub fn weekday_name(self, weekday: u32, short: bool) -> &'static str {
        let table: &[&'static str; 7] = match (self, short) {
            (Locale::En, false) => &WEEKDAYS,
            (Locale::En, true) => &EN_WEEKDAYS_SHORT,
            (Locale::De, false) => &DE_WEEKDAYS,
            (Locale::De, true) => &DE_WEEKDAYS_SHORT,
            (Locale::Fr, false) => &FR_WEEKDAYS,
            (Locale::Fr, true) => &FR_WEEKDAYS_SHORT,
        };
        usize::try_from(weekday).ok().and_then(|i| table.get(i)).copied().unwrap_or("")
    }

    /// The month number 1–12 a typed name stands for: a full or short name in this locale or in
    /// English, any case, with or without the trailing dot of a short French name (`mär`,
    /// `Okt.`, `févr`, `déc.`, `March`).
    pub fn month_from_name(self, name: &str) -> Option<u32> {
        let name = name.trim().trim_end_matches('.');
        if name.is_empty() {
            return None;
        }
        let same = |a: &str| a.trim_end_matches('.').to_lowercase() == name.to_lowercase();
        (1..=12u32).find(|&m| {
            [self, Locale::En].iter().any(|l| same(l.month_name(m, false)) || same(l.month_name(m, true)))
                // German also writes `Sept.`, French `sep`: any 3+ letter prefix of the full name.
                || (name.chars().count() >= 3 && self.month_name(m, false).to_lowercase().starts_with(&name.to_lowercase()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_name_the_locales() {
        assert_eq!(Locale::default(), Locale::En);
        for l in Locale::ALL {
            assert_eq!(Locale::from_tag(l.tag()), Some(l));
        }
        for tag in ["de", "DE", "de-DE", "de_AT", "de-CH"] {
            assert_eq!(Locale::from_tag(tag), Some(Locale::De), "{tag}");
        }
        for tag in ["fr", "fr-FR", "fr_BE", "FR-ch"] {
            assert_eq!(Locale::from_tag(tag), Some(Locale::Fr), "{tag}");
        }
        for tag in ["en", "en-GB", "en_US"] {
            assert_eq!(Locale::from_tag(tag), Some(Locale::En), "{tag}");
        }
        for tag in ["", "es", "deutsch", "francais"] {
            assert_eq!(Locale::from_tag(tag), None, "{tag}");
        }
    }

    #[test]
    fn separators() {
        assert_eq!((Locale::En.decimal(), Locale::En.group(), Locale::En.date_separator()), ('.', ',', '/'));
        assert_eq!((Locale::De.decimal(), Locale::De.group(), Locale::De.date_separator()), (',', '.', '.'));
        assert_eq!((Locale::Fr.decimal(), Locale::Fr.group(), Locale::Fr.date_separator()), (',', '\u{202F}', '/'));
        assert!(!Locale::En.day_first());
        assert!(Locale::De.day_first() && Locale::Fr.day_first());
    }

    #[test]
    fn names() {
        assert_eq!(Locale::En.month_name(1, false), MONTHS[0]);
        assert_eq!(Locale::En.weekday_name(4, false), WEEKDAYS[4]);
        assert_eq!(Locale::En.month_name(10, true), "Oct");
        assert_eq!(Locale::De.month_name(3, false), "März");
        assert_eq!(Locale::De.month_name(3, true), "Mär");
        assert_eq!(Locale::De.weekday_name(4, false), "Donnerstag");
        assert_eq!(Locale::De.weekday_name(4, true), "Do");
        assert_eq!(Locale::Fr.month_name(2, false), "février");
        assert_eq!(Locale::Fr.month_name(2, true), "févr.");
        assert_eq!(Locale::Fr.weekday_name(1, false), "lundi");
        assert_eq!(Locale::Fr.weekday_name(1, true), "lun.");
        assert_eq!(Locale::De.month_name(0, false), "");
        assert_eq!(Locale::Fr.month_name(13, true), "");
        assert_eq!(Locale::De.weekday_name(7, false), "");
        assert_eq!((Locale::En.bool_name(true), Locale::De.bool_name(false), Locale::Fr.bool_name(true)), ("TRUE", "FALSCH", "VRAI"));
    }

    #[test]
    fn typed_month_names() {
        assert_eq!(Locale::De.month_from_name("Mär"), Some(3));
        assert_eq!(Locale::De.month_from_name("okt."), Some(10));
        assert_eq!(Locale::De.month_from_name("Dezember"), Some(12));
        assert_eq!(Locale::De.month_from_name("Sept"), Some(9));
        assert_eq!(Locale::De.month_from_name("March"), Some(3));
        assert_eq!(Locale::Fr.month_from_name("févr."), Some(2));
        assert_eq!(Locale::Fr.month_from_name("DÉC"), Some(12));
        assert_eq!(Locale::Fr.month_from_name("juillet"), Some(7));
        assert_eq!(Locale::Fr.month_from_name("ju"), None);
        assert_eq!(Locale::En.month_from_name("Okt"), None);
        assert_eq!(Locale::En.month_from_name(""), None);
    }
}
