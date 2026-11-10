//! The words of a component come from the program, never from here (ADR 0057).
//!
//! Pattern: Strategy behind the trait [`Say`] — a key goes in, a sentence in the reader's language
//! comes out. The keys are `field.<type>`, `label.<type>.<part>`, `hint.<type>.<part>` and the
//! `problem.*` keys of [`crate::Problem::key`].

/// Something that turns a translation key into text.
pub trait Say: Send + Sync {
    /// The text for `key`; a missing one must come back as something that shows the key.
    fn say(&self, key: &str) -> String;
}

impl<F: Fn(&str) -> String + Send + Sync> Say for F {
    fn say(&self, key: &str) -> String {
        self(key)
    }
}

/// Shows the key itself: for tests and for a program that has no catalog yet.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeysAsText;

impl Say for KeysAsText {
    fn say(&self, key: &str) -> String {
        key.to_string()
    }
}
