//! Inline text: marks become spans of one Blinc rich-text element.
//!
//! Pattern: Adapter — [`Inline`] (chars, stand-off marks) to Blinc `StyledText` (bytes, flat
//! non-overlapping spans). Links get no Blinc link (Blinc would open a browser for them); the
//! wrapping element hit-tests them itself and reports the href to the [`LinkHandler`].

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::styled_text::{StyledLine, StyledText, TextSpan};
use blinc_layout::text_hit::{TextHitSpan, TextHitSpans};
use blinc_layout::text_measure::TextLayoutOptions;
use blinc_layout::{RichText, rich_text_styled};

use super::LinkHandler;
use super::look::Look;
use crate::text::{Inline, MarkKind};

/// How the text of one block is set.
#[derive(Clone, Copy)]
pub(super) struct Face {
    pub size: f32,
    pub bold: bool,
}

/// The element for `inline` set in `face`.
pub(super) fn inline(
    inline: &Inline,
    face: Face,
    look: &Look,
    on_link: Option<&LinkHandler>,
) -> Div {
    let text = if inline.text.is_empty() {
        " "
    } else {
        &inline.text
    };
    let spans = spans(inline, face, look);
    let hits = hit_spans(inline, &spans, face);
    let rich: RichText = rich_text_styled(StyledText::from_lines(vec![StyledLine::new(
        text.to_string(),
        spans,
    )]))
    .size(face.size)
    .default_color(look.text);
    let holder = div().w_full().child(rich);
    match (on_link, hits) {
        (Some(handler), Some(hits)) => {
            let handler = Arc::clone(handler);
            holder.on_click(move |ctx| {
                if let Some(span) = hits.hit(ctx.local_x, ctx.local_y, ctx.bounds_width)
                    && let Some(href) = &span.url
                {
                    handler(href);
                }
            })
        }
        _ => holder,
    }
}

/// Cut the text at every mark boundary and give each piece the flags of the marks over it.
fn spans(inline: &Inline, face: Face, look: &Look) -> Vec<TextSpan> {
    let len = inline.text.chars().count();
    let mut cuts = vec![0, len];
    for m in &inline.marks {
        cuts.push(m.start.min(len));
        cuts.push(m.end.min(len));
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut out = Vec::new();
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let mut span = TextSpan::colored(inline.byte_of(a), inline.byte_of(b), look.text);
        span.bold = face.bold;
        for m in inline.marks.iter().filter(|m| m.start <= a && b <= m.end) {
            match &m.kind {
                MarkKind::Strong => span.bold = true,
                MarkKind::Emphasis => span.italic = true,
                MarkKind::Strike => span.strikethrough = true,
                MarkKind::Code => span.code = true,
                MarkKind::Link { .. } => {
                    span.underline = true;
                    span.color = look.link;
                }
            }
        }
        out.push(span);
    }
    out
}

/// The link ranges as pointer targets, if there are any.
fn hit_spans(inline: &Inline, _spans: &[TextSpan], face: Face) -> Option<Arc<TextHitSpans>> {
    let len = inline.text.chars().count();
    let targets: Vec<TextHitSpan> = inline
        .marks
        .iter()
        .filter_map(|m| match &m.kind {
            MarkKind::Link { href } if m.start < m.end.min(len) => Some(TextHitSpan {
                start: inline.byte_of(m.start),
                end: inline.byte_of(m.end),
                cursor: blinc_layout::element::CursorStyle::Pointer,
                url: Some(href.as_str().into()),
            }),
            _ => None,
        })
        .collect();
    if targets.is_empty() {
        return None;
    }
    let mut options = TextLayoutOptions::new();
    if face.bold {
        options.font_weight = 700;
    }
    Some(Arc::new(TextHitSpans {
        content: inline.text.as_str().into(),
        font_size: face.size,
        line_height: options.line_height,
        options,
        spans: targets.into(),
    }))
}
