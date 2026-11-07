//! `Fallback`: ask a second `Words` when the first cannot make the sentence.
//!
//! Pattern: Decorator — wraps a `Words` and is a `Words`; a text missing in the reader's language
//! comes in English (ADR 0057). Decorators nest: regional language, language, English.

use crate::{Args, Said, TextId, Words};

/// `first`, and where it says [`Said::Missing`], `then`.
pub struct Fallback<A, B> {
    first: A,
    then: B,
}

impl<A: Words, B: Words> Fallback<A, B> {
    /// Combine two.
    pub fn new(first: A, then: B) -> Self {
        Self { first, then }
    }
}

impl<A: Words, B: Words> Words for Fallback<A, B> {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        match self.first.say(id, args) {
            found @ Said::Found(_) => found,
            Said::Missing(_) => self.then.say(id, args),
        }
    }
}
