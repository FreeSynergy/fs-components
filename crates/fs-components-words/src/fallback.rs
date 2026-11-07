//! `Fallback`: ask a second `Words` when the first cannot make the sentence.
//!
//! Pattern: Decorator — wraps a `Words` and is a `Words`; a text missing in the reader's language
//! comes in English (ADR 0057). Decorators nest: regional language, language, English. An empty
//! answer of a foreign `Words` cannot exist (`Said` turns it into a missing mark), so it falls
//! through like any other missing text.

use crate::{Args, Said, TextId, Words};

/// `first`, and where it says missing, `then`.
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
        let said = self.first.say(id, args);
        if said.is_found() {
            said
        } else {
            self.then.say(id, args)
        }
    }
}
