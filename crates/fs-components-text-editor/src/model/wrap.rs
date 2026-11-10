//! Wrapping the selection in Markdown marks and taking them off again.
//!
//! Pattern: none — pure functions from text and selection to an edit.

use super::edit::Edit;
use super::selection::Selection;

/// A mark that surrounds text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Wrap {
    /// `**text**`
    Bold,
    /// `*text*`
    Italic,
    /// `` `text` ``
    Code,
}

impl Wrap {
    fn mark(self) -> char {
        match self {
            Self::Bold | Self::Italic => '*',
            Self::Code => '`',
        }
    }

    fn len(self) -> usize {
        usize::from(self == Self::Bold) + 1
    }

    /// Is the mark part of a run of `run` mark characters? A run of three stars is bold and italic.
    fn in_run(self, run: usize) -> bool {
        match self {
            Self::Italic => run % 2 == 1,
            _ => run >= self.len(),
        }
    }
}

fn text_of(chars: &[char], from: usize, to: usize) -> String {
    chars[from..to].iter().collect()
}

/// Wrap the selection, or take the mark off when it is already there (around the selection or
/// at its two ends). An empty selection gets an empty pair with the caret between.
pub(super) fn toggle(chars: &[char], sel: Selection, wrap: Wrap) -> Edit {
    let sel = sel.clamp(chars.len());
    let (m, n) = (wrap.mark(), wrap.len());
    let mark: String = std::iter::repeat_n(m, n).collect();
    let inner = text_of(chars, sel.start, sel.end);
    let before = chars[..sel.start]
        .iter()
        .rev()
        .take_while(|c| **c == m)
        .count();
    let after = chars[sel.end..].iter().take_while(|c| **c == m).count();
    if wrap.in_run(before.min(after)) {
        return Edit {
            at: sel.start - n,
            removed: format!("{mark}{inner}{mark}"),
            inserted: inner,
            before: sel,
            after: Selection::new(sel.start - n, sel.end - n),
        };
    }
    let size = sel.end - sel.start;
    let lead = inner.chars().take_while(|c| *c == m).count().min(size / 2);
    let trail = inner
        .chars()
        .rev()
        .take_while(|c| *c == m)
        .count()
        .min(size / 2);
    if !sel.is_empty() && wrap.in_run(lead.min(trail)) {
        let kept: String = inner.chars().skip(n).take(size - 2 * n).collect();
        return Edit {
            at: sel.start,
            removed: inner,
            inserted: kept,
            before: sel,
            after: Selection::new(sel.start, sel.end - 2 * n),
        };
    }
    Edit {
        at: sel.start,
        removed: inner.clone(),
        inserted: format!("{mark}{inner}{mark}"),
        before: sel,
        after: Selection::new(sel.start + n, sel.end + n),
    }
}

/// `[selection](url)`; with nothing selected `[](url)` with the caret between the brackets.
pub(super) fn link(chars: &[char], sel: Selection, url: &str) -> Edit {
    let sel = sel.clamp(chars.len());
    let inner = text_of(chars, sel.start, sel.end);
    Edit {
        at: sel.start,
        removed: inner.clone(),
        inserted: format!("[{inner}]({url})"),
        before: sel,
        after: Selection::new(sel.start + 1, sel.end + 1),
    }
}
