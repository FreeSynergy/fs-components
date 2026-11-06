//! Text with marks as a flow of words.
//!
//! Pattern: Adapter (Iterator over runs) — the text is cut into runs of one style, each run is a
//! small text element, and the runs flow and wrap like words in a flex row. Every run is measured
//! in the face it is drawn in, and a link is a run with its own click handler.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::LinkHandler;
use super::inline::Face;
use super::look::Look;
use crate::text::{Inline, MarkKind};

/// The style of a run.
// Independent on/off flags of a run; a state enum would only mirror the mark kinds.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, PartialEq, Eq)]
struct Style {
    strong: bool,
    emphasis: bool,
    strike: bool,
    code: bool,
    link: Option<String>,
}

/// The flow of runs of `inline`.
pub(super) fn flow(inline: &Inline, face: Face, look: &Look, on_link: Option<&LinkHandler>) -> Div {
    let mut row = div().flex_row().flex_wrap().w_full();
    for (piece, style) in runs(inline, face) {
        row = row.child(run(&piece, &style, face, look, on_link));
    }
    row
}

/// The element of one run.
fn run(piece: &str, style: &Style, face: Face, look: &Look, on_link: Option<&LinkHandler>) -> Div {
    let color = if style.link.is_some() {
        look.link
    } else {
        look.text
    };
    let size = if style.code {
        face.size * 0.92
    } else {
        face.size
    };
    let mut t = text(piece).size(size).color(color).no_wrap();
    if style.strong || face.bold {
        t = t.bold();
    }
    if style.emphasis {
        t = t.italic();
    }
    if style.strike {
        t = t.strikethrough();
    }
    if style.link.is_some() {
        t = t.underline();
    }
    if style.code {
        t = t.monospace();
    }
    let mut holder = div().child(t);
    if style.code {
        holder = holder.class("fs-tv-codespan");
    }
    if let (Some(href), Some(handler)) = (&style.link, on_link) {
        let (href, handler) = (href.clone(), Arc::clone(handler));
        holder = holder.on_click(move |_| handler(&href));
    }
    holder
}

/// Runs: the text cut where the style changes and after each stretch of spaces, so that a run is
/// a word (or part of one) with the spaces behind it.
fn runs(inline: &Inline, _face: Face) -> Vec<(String, Style)> {
    let chars: Vec<char> = inline.text.chars().collect();
    let style_at = |i: usize| {
        let mut s = Style::default();
        for m in inline.marks.iter().filter(|m| m.start <= i && i < m.end) {
            match &m.kind {
                MarkKind::Strong => s.strong = true,
                MarkKind::Emphasis => s.emphasis = true,
                MarkKind::Strike => s.strike = true,
                MarkKind::Code => s.code = true,
                MarkKind::Link { href } => s.link = Some(href.clone()),
            }
        }
        s
    };
    let mut out: Vec<(String, Style)> = Vec::new();
    let mut current = String::new();
    let mut current_style = Style::default();
    for (i, &c) in chars.iter().enumerate() {
        let style = style_at(i);
        let after_space = current.ends_with(char::is_whitespace);
        let breaks =
            !current.is_empty() && (style != current_style || (after_space && !c.is_whitespace()));
        if breaks {
            out.push((std::mem::take(&mut current), current_style.clone()));
        }
        if current.is_empty() {
            current_style = style;
        }
        current.push(c);
    }
    if !current.is_empty() {
        out.push((current, current_style));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Mark;

    #[test]
    fn runs_split_at_words_and_style_changes() {
        let inline = Inline::new("a bold word", vec![Mark::new(MarkKind::Strong, 2, 6)]);
        let r = runs(
            &inline,
            Face {
                size: 14.0,
                bold: false,
            },
        );
        let texts: Vec<&str> = r.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, ["a ", "bold", " ", "word"]);
        assert!(!r[0].1.strong && r[1].1.strong && !r[2].1.strong && !r[3].1.strong);
    }

    #[test]
    fn a_style_change_inside_a_word_makes_two_runs() {
        let inline = Inline::new("unbelievable", vec![Mark::new(MarkKind::Emphasis, 2, 6)]);
        let r = runs(
            &inline,
            Face {
                size: 14.0,
                bold: false,
            },
        );
        let texts: Vec<&str> = r.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, ["un", "beli", "evable"]);
    }
}
