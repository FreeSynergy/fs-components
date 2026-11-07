//! `Catalog`: one language loaded from a table of `(identifier, template)`.
//!
//! Pattern: Strategy — a concrete `Words` backed by a plain table and one `PluralRules`. A
//! language package (ADR 0036) is a text file `id = template`, one per line; no library needed.
//! Templates are parsed once, when the catalog is made.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use crate::template::{Part, parse, render};
use crate::{Args, PluralRules, Said, TextId, Words};

/// A language: parsed templates by identifier (`None` = malformed), and its plural rule.
pub struct Catalog {
    rules: Box<dyn PluralRules>,
    texts: HashMap<String, Option<Vec<Part>>>,
}

/// A line of a text table that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError {
    line: usize,
    why: &'static str,
}

impl CatalogError {
    /// The 1-based line number.
    pub fn line(&self) -> usize {
        self.line
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.why)
    }
}

impl Error for CatalogError {}

/// The template of a line after `=`: leading blanks go, trailing blanks go unless the last
/// one is escaped (`\ `).
fn template_of(rest: &str) -> &str {
    let rest = rest.trim_start();
    let trimmed = rest.trim_end();
    let slashes = trimmed.len() - trimmed.trim_end_matches('\\').len();
    if slashes % 2 == 1 && rest.len() > trimmed.len() && rest[trimmed.len()..].starts_with(' ') {
        &rest[..=trimmed.len()]
    } else {
        trimmed
    }
}

impl Catalog {
    /// From a static table. A later entry replaces an earlier one with the same identifier.
    /// A malformed template is not refused here; asking for it gives a missing mark, and
    /// [`Catalog::validate`] lists it (call that in one test per program).
    pub fn from_table(rules: impl PluralRules + 'static, table: &[(&str, &str)]) -> Self {
        let texts = table
            .iter()
            .map(|(k, v)| ((*k).to_owned(), parse(v)))
            .collect();
        Self {
            rules: Box::new(rules),
            texts,
        }
    }

    /// From a text table: one `id = template` per line; empty lines and lines starting with `#`
    /// are skipped. Every template is checked; the error names the line. An identifier twice is
    /// an error.
    ///
    /// # Errors
    /// A line without `=`, with an empty or repeated identifier, or with a malformed template.
    pub fn from_text(rules: impl PluralRules + 'static, text: &str) -> Result<Self, CatalogError> {
        let mut texts = HashMap::new();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim_start();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let err = |why| CatalogError { line: i + 1, why };
            let (id, rest) = line.split_once('=').ok_or_else(|| err("no `=`"))?;
            let id = id.trim();
            if id.is_empty() {
                return Err(err("empty identifier"));
            }
            let parts = parse(template_of(rest)).ok_or_else(|| err("malformed template"))?;
            if texts.insert(id.to_owned(), Some(parts)).is_some() {
                return Err(err("identifier given twice"));
            }
        }
        Ok(Self {
            rules: Box::new(rules),
            texts,
        })
    }

    /// The identifiers (sorted) whose template is malformed, blank or nested too deeply.
    /// Empty means every template can be used.
    pub fn validate(&self) -> Vec<TextId> {
        let mut bad: Vec<_> = self
            .texts
            .iter()
            .filter(|(_, parts)| parts.is_none())
            .map(|(id, _)| TextId::new(id.as_str()))
            .collect();
        bad.sort();
        bad
    }
}

impl fmt::Debug for Catalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Catalog({} texts)", self.texts.len())
    }
}

impl Words for Catalog {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        let mut out = String::new();
        let made = self
            .texts
            .get(id.as_str())
            .and_then(Option::as_deref)
            .and_then(|parts| render(parts, args, self.rules.as_ref(), None, &mut out));
        match made {
            Some(()) => Said::found(id, out),
            None => Said::missing(id),
        }
    }
}
