//! Text with marks as a flow of words.
//!
//! Pattern: Adapter (Iterator over runs) — the text is cut into runs of one style, each run is a
//! small text element, and the runs flow and wrap like words in a flex row. Every run is measured
//! in the face it is drawn in, and a link is a run with its own click handler.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::LinkHandler;
use super::emphasis::emphasised;
use super::fallback::font_for;
use super::inline::Face;
use super::look::Look;
use crate::text::{Inline, MarkKind};

/// Runs longer than this many characters may wrap inside themselves.
const LONG_RUN: usize = 24;

/// The style of a run.
// Independent on/off flags of a run; a state enum would only mirror the mark kinds.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Style {
    pub strong: bool,
    pub emphasis: bool,
    pub strike: bool,
    pub code: bool,
    pub link: Option<String>,
    pub font: Option<std::sync::Arc<str>>,
}

/// The flow of runs of `inline`.
pub(super) fn flow(inline: &Inline, face: Face, look: &Look, on_link: Option<&LinkHandler>) -> Div {
    let mut row = div().flex_row().flex_wrap().items_baseline().w_full();
    for (piece, style) in runs(inline, face) {
        if piece == "\n" {
            // A hard line break: a full-width, empty line end forces the next run onto a new line.
            row = row.child(div().w_full().h(0.0));
        } else {
            row = row.child(run(&piece, &style, face, look, on_link));
        }
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
    let mut t = text(piece).size(size).color(color);
    let may_wrap = face.cell || piece.chars().count() > LONG_RUN;
    if !may_wrap {
        t = t.no_wrap();
    }
    if style.strong || face.bold {
        t = t.bold();
    }
    if style.strike {
        t = t.strikethrough();
    }
    if style.link.is_some() {
        t = t.underline();
    }
    if style.code || face.mono {
        t = t.monospace();
    }
    if let Some(family) = &style.font {
        t = t.font(family.as_ref());
    }
    // Blinc's `text` does not draw italic; a rich text does, so emphasis uses one.
    let mut holder = if style.emphasis {
        div().child(emphasised(piece, style, size, color, face.bold))
    } else {
        div().child(t)
    };
    if may_wrap {
        // A really long word, or a run in a table cell, may wrap (or break) inside its own box.
        holder = holder.class("fs-tv-run");
    }
    if style.code {
        holder = holder.class("fs-tv-codespan");
    }
    if style.font.is_some() && piece.chars().any(is_symbol) {
        // Blinc measures a symbol set in a fallback font as nearly nothing, so the glyphs would
        // pile up: give the run the width its characters need.
        let symbols = piece.chars().filter(|c| is_symbol(*c)).count();
        let spaces = piece.chars().filter(|c| c.is_whitespace()).count();
        #[allow(clippy::cast_precision_loss)]
        let width = symbols as f32 * size * 1.3 + spaces as f32 * size * 0.3;
        holder = holder.w(width).flex_shrink_0();
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
        let mut s = Style {
            font: chars.get(i).and_then(|&c| font_for(c)),
            ..Style::default()
        };
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
        let after_space = current.ends_with(|w: char| w.is_whitespace() && w != '\n');
        let breaks = !current.is_empty()
            && (style != current_style
                || (after_space && !c.is_whitespace())
                || c == '\n'
                || current == "\n");
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

/// Symbols and emoji (not letters of a script): these are the runs whose width Blinc misjudges.
fn is_symbol(c: char) -> bool {
    matches!(u32::from(c), 0x2300..=0x23FF | 0x2600..=0x27BF | 0x2900..=0x2BFF | 0x1F000..=0x1FAFF)
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
                mono: false,
                cell: false,
            },
        );
        let texts: Vec<&str> = r.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, ["a ", "bold", " ", "word"]);
        assert!(!r[0].1.strong && r[1].1.strong && !r[2].1.strong && !r[3].1.strong);
    }

    #[test]
    fn a_hard_break_is_its_own_run() {
        let inline = Inline::new("one\ntwo", vec![Mark::new(MarkKind::Strong, 0, 3)]);
        let r = runs(
            &inline,
            Face {
                size: 14.0,
                bold: false,
                mono: false,
                cell: false,
            },
        );
        let texts: Vec<&str> = r.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, ["one", "\n", "two"]);
    }

    #[test]
    fn a_style_change_inside_a_word_makes_two_runs() {
        let inline = Inline::new("unbelievable", vec![Mark::new(MarkKind::Emphasis, 2, 6)]);
        let r = runs(
            &inline,
            Face {
                size: 14.0,
                bold: false,
                mono: false,
                cell: false,
            },
        );
        let texts: Vec<&str> = r.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, ["un", "beli", "evable"]);
    }
}
