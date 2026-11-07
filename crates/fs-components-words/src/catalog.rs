//! `Catalog`: one language loaded from a table of `(identifier, template)`.
//!
//! Pattern: Strategy — a concrete `Words` backed by a plain table and one `PluralRules`. A
//! language package (ADR 0036) is a text file `id = template`, one per line; no library needed.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use crate::template::{parse, render};
use crate::{Args, PluralRules, Said, TextId, Words};

/// A language: templates by identifier, and its plural rule.
pub struct Catalog {
    rules: Box<dyn PluralRules>,
    texts: HashMap<String, String>,
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

impl Catalog {
    /// From a static table. A later entry replaces an earlier one with the same identifier.
    /// A malformed template is not refused here; asking for it gives [`Said::Missing`].
    pub fn from_table(rules: impl PluralRules + 'static, table: &[(&str, &str)]) -> Self {
        let texts = table
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        Self {
            rules: Box::new(rules),
            texts,
        }
    }

    /// From a text table: one `id = template` per line; empty lines and lines starting with `#`
    /// are skipped. Every template is checked; the error names the line.
    ///
    /// # Errors
    /// A line without `=`, with an empty identifier, or with a malformed template.
    pub fn from_text(rules: impl PluralRules + 'static, text: &str) -> Result<Self, CatalogError> {
        let mut texts = HashMap::new();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let err = |why| CatalogError { line: i + 1, why };
            let (id, template) = line.split_once('=').ok_or_else(|| err("no `=`"))?;
            if id.trim().is_empty() {
                return Err(err("empty identifier"));
            }
            if parse(template.trim()).is_none() {
                return Err(err("malformed template"));
            }
            texts.insert(id.trim().to_owned(), template.trim().to_owned());
        }
        Ok(Self {
            rules: Box::new(rules),
            texts,
        })
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
            .and_then(|t| parse(t))
            .and_then(|parts| render(&parts, args, self.rules.as_ref(), None, &mut out));
        if made.is_some() && !out.is_empty() {
            Said::Found(out)
        } else {
            Said::missing(id.as_str())
        }
    }
}
