//! `English`: the built-in language, the last resort of every [`Fallback`](crate::Fallback).
//!
//! Pattern: Strategy — the simplest concrete `Words`: a static table and the English plural rule.

use crate::{Args, Catalog, EnglishPlural, Said, TextId, Words};

/// English texts from a static table of `(identifier, template)`.
pub struct English(Catalog);

impl English {
    /// Build from a table, for example the program's `words.rs`.
    pub fn new(table: &[(&str, &str)]) -> Self {
        Self(Catalog::from_table(EnglishPlural, table))
    }
}

impl Words for English {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        self.0.say(id, args)
    }
}
