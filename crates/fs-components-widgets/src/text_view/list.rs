//! A list.
//!
//! Pattern: Adapter — an item holds blocks, which may hold lists again.

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
        let marker = if let Some(done) = item.task {
            super::task::marker(look, done, look.base)
        } else {
            let mark = if ordered {
                format!("{}.", start + i as u64)
            } else {
                "\u{2022}".to_string()
            };
            div()
                .class("fs-tv-marker")
                .child(text(mark).size(look.base))
        };
        column = column.child(
            div().class("fs-tv-item").child(marker).child(
                blocks(&item.blocks, look, on_link, depth)
                    .flex_grow()
                    .w(0.0),
            ),
        );
    }
    column
}
