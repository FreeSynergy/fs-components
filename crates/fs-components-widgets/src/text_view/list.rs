//! A list.
//!
//! Pattern: Composite renderer — an item holds blocks, which may hold lists again.

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::LinkHandler;
use super::blocks::blocks;
use super::look::Look;
use crate::text::ListItem;

/// The element of a list.
pub(super) fn list(
    ordered: bool,
    start: u64,
    items: &[ListItem],
    look: &Look,
    on_link: Option<&LinkHandler>,
    depth: usize,
) -> Div {
    let mut column = div().class("fs-tv-block").flex_col();
    for (i, item) in items.iter().enumerate() {
        let marker = match (item.task, ordered) {
            (Some(true), _) => "\u{2611}".to_string(),
            (Some(false), _) => "\u{2610}".to_string(),
            (None, true) => format!("{}.", start + i as u64),
            (None, false) => "\u{2022}".to_string(),
        };
        column = column.child(
            div()
                .class("fs-tv-item")
                .child(
                    div()
                        .class("fs-tv-marker")
                        .child(text(marker).size(look.base)),
                )
                .child(
                    blocks(&item.blocks, look, on_link, depth)
                        .flex_grow()
                        .w(0.0),
                ),
        );
    }
    column
}
