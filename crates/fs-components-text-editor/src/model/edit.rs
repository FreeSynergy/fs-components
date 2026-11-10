//! One applied change of the text.
//!
//! Pattern: Command — an `Edit` is the record of a command: it replaces `removed` by `inserted`
//! at `at`, and `invert` is the edit that undoes it. The history holds nothing else.

use super::selection::Selection;

/// A replacement of `removed` by `inserted` at character `at`, with the selection before and after.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    /// Character index where the replacement starts.
    pub at: usize,
    /// The text that is taken out.
    pub removed: String,
    /// The text that is put in.
    pub inserted: String,
    /// The selection before the edit.
    pub before: Selection,
    /// The selection after the edit.
    pub after: Selection,
}

fn byte_of(text: &str, chars: usize) -> usize {
    text.char_indices()
        .nth(chars)
        .map_or(text.len(), |(i, _)| i)
}

impl Edit {
    /// `text` with the edit made. An edit that does not fit the text is cut to it, never a panic.
    pub fn apply(&self, text: &str) -> String {
        let from = byte_of(text, self.at);
        let to = byte_of(text, self.at + self.removed.chars().count());
        let mut out = String::with_capacity(text.len() + self.inserted.len());
        out.push_str(&text[..from]);
        out.push_str(&self.inserted);
        out.push_str(&text[to..]);
        out
    }

    /// The edit that takes this one back.
    #[must_use]
    pub fn invert(&self) -> Self {
        Self {
            at: self.at,
            removed: self.inserted.clone(),
            inserted: self.removed.clone(),
            before: self.after,
            after: self.before,
        }
    }

    /// The smallest edit that turns `old` into `new` (common start and end are kept); `None` if equal.
    pub fn between(old: &str, new: &str, before: Selection, after: Selection) -> Option<Self> {
        if old == new {
            return None;
        }
        let (o, n): (Vec<char>, Vec<char>) = (old.chars().collect(), new.chars().collect());
        let head = o.iter().zip(&n).take_while(|(a, b)| a == b).count();
        let tail = o[head..]
            .iter()
            .rev()
            .zip(n[head..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        Some(Self {
            at: head,
            removed: o[head..o.len() - tail].iter().collect(),
            inserted: n[head..n.len() - tail].iter().collect(),
            before,
            after,
        })
    }
}
