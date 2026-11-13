//! `Locale`: a language tag in BCP 47 form (ADR 0074, layer "Sprache und Formate").
//!
//! Pattern: Newtype + Value object — a bare `String` could be a path or a sentence; a `Locale` is always a well-formed,
//! normalised tag (`de`, `de-AT`, `zh-Hans-CN`: language lower case, script Title case, region upper case). The parser
//! covers the language, script, region and variant subtags and keeps extensions and private use lower case; it does not
//! check the IANA registry (a well-formed but unregistered tag is accepted).

use std::fmt;

/// A well-formed BCP 47 language tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Locale(String);

/// Why a tag was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleError(&'static str);

impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a BCP 47 language tag: {}", self.0)
    }
}

impl std::error::Error for LocaleError {}

fn alpha(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_alphabetic())
}

fn alnum(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_alphanumeric())
}

impl Locale {
    /// English, the last stage of every chain.
    pub fn english() -> Self {
        Self("en".to_owned())
    }

    /// Parse and normalise a tag with `-` between the subtags (`DE-at` becomes `de-AT`).
    ///
    /// # Errors
    /// [`LocaleError`] for an empty tag, a language that is not 2 to 3 (or 5 to 8) letters, a subtag of the wrong shape or
    /// length, or a repeated script or region.
    pub fn parse(tag: &str) -> Result<Self, LocaleError> {
        let mut parts = tag.split('-');
        let language = parts.next().unwrap_or_default();
        let ok = matches!(language.len(), 2..=3 | 5..=8) && alpha(language);
        if !ok {
            return Err(LocaleError("language"));
        }
        let mut out = vec![language.to_ascii_lowercase()];
        let mut stage = 0; // 0 script allowed, 1 region allowed, 2 variants, 3 extensions and private use
        for part in parts {
            let (n, lower) = (part.len(), part.to_ascii_lowercase());
            if n == 0 || n > 8 || !alnum(part) {
                return Err(LocaleError("subtag"));
            }
            if stage == 3 {
                out.push(lower);
            } else if stage == 0 && n == 4 && alpha(part) {
                let mut c = lower.chars();
                let head = c.next().map(|h| h.to_ascii_uppercase()).unwrap_or_default();
                out.push(format!("{head}{}", c.as_str()));
                stage = 1;
            } else if stage <= 1
                && ((n == 2 && alpha(part)) || (n == 3 && part.bytes().all(|b| b.is_ascii_digit())))
            {
                out.push(lower.to_ascii_uppercase());
                stage = 2;
            } else if stage <= 2 && (n >= 5 || (n == 4 && part.as_bytes()[0].is_ascii_digit())) {
                out.push(lower);
                stage = 2;
            } else if n == 1 {
                out.push(lower);
                stage = 3;
            } else {
                return Err(LocaleError("subtag order"));
            }
        }
        if out.last().is_some_and(|last| last.len() == 1) {
            return Err(LocaleError("a singleton needs a subtag after it"));
        }
        Ok(Self(out.join("-")))
    }

    /// From the value of `LANG`, `LC_ALL` and the like (`de_DE.UTF-8@euro`): the code set and modifier are dropped, `_`
    /// becomes `-`. `None` for an empty value, `C`, `POSIX` (no language) or anything that is no tag.
    pub fn from_posix(value: &str) -> Option<Self> {
        let value = value.trim();
        let base = value.split(['.', '@']).next().unwrap_or_default();
        if base.is_empty() || base == "C" || base == "POSIX" {
            return None;
        }
        Self::parse(&base.replace('_', "-")).ok()
    }

    /// The tag as written, normalised.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The language subtag (`de` of `de-AT`).
    pub fn primary(&self) -> &str {
        self.0.split('-').next().unwrap_or_default()
    }

    /// The tag and its shorter forms, longest first: `zh-Hans-CN`, `zh-Hans`, `zh`. English is not added; the caller ends
    /// every chain with it.
    pub fn chain(&self) -> Vec<Locale> {
        let parts: Vec<&str> = self.0.split('-').collect();
        (1..=parts.len())
            .rev()
            .map(|n| Self(parts[..n].join("-")))
            .collect()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
