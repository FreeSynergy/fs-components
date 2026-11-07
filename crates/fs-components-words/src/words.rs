//! `Words`: the one interface through which a program gets a sentence a person reads.
//!
//! Pattern: Strategy — callers hold a `Words` (generic or `dyn`); English, a table of another
//! language, Fluent or MF2 are interchangeable behind it. The caller passes an
//! identifier and named values with their kind; the template and the language decide the rest.

use crate::{Args, Said, TextId};

/// Turns a text identifier and named values into a sentence. Never panics and never returns an
/// empty text: what cannot be made comes back as [`Said::missing`] carrying the identifier.
pub trait Words: Send + Sync {
    /// The sentence for `id`, filled with `args`.
    fn say(&self, id: &TextId, args: &Args) -> Said;
}

impl<T: Words + ?Sized> Words for Box<T> {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        (**self).say(id, args)
    }
}

impl<T: Words + ?Sized> Words for &T {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        (**self).say(id, args)
    }
}
