//! Plural categories and the `PluralRules` of a language.
//!
//! Pattern: Strategy — one `PluralRules` per language maps a count to a category; the template
//! (not the code) says what each category looks like. English knows two categories, Polish
//! three for whole numbers, Arabic six; the enum holds all six (CLDR) so no language is squeezed.

/// Plural category (CLDR names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    /// Zero (Arabic, Latvian ...).
    Zero,
    /// One.
    One,
    /// Two (Arabic, Welsh ...).
    Two,
    /// Few (Polish, Czech, Russian ...).
    Few,
    /// Many (Polish, Russian, Arabic ...).
    Many,
    /// Everything else; every plural in a template should have it.
    Other,
}

impl PluralCategory {
    /// Read a category name as written in a template (`one`, `few` ...).
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "zero" => Self::Zero,
            "one" => Self::One,
            "two" => Self::Two,
            "few" => Self::Few,
            "many" => Self::Many,
            "other" => Self::Other,
            _ => return None,
        })
    }
}

/// The plural rule of one language, for whole numbers.
pub trait PluralRules: Send + Sync {
    /// The category of `n`.
    fn category(&self, n: u64) -> PluralCategory;
}

/// English: 1 is `One`, everything else `Other`.
#[derive(Debug, Clone, Copy, Default)]
pub struct EnglishPlural;

impl PluralRules for EnglishPlural {
    fn category(&self, n: u64) -> PluralCategory {
        if n == 1 {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }
}

/// Polish, whole numbers: 1 `One`; ends in 2-4 but not 12-14 `Few`; the rest `Many`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PolishPlural;

impl PluralRules for PolishPlural {
    fn category(&self, n: u64) -> PluralCategory {
        match (n, n % 10, n % 100) {
            (1, _, _) => PluralCategory::One,
            (_, 2..=4, r) if !(12..=14).contains(&r) => PluralCategory::Few,
            _ => PluralCategory::Many,
        }
    }
}
