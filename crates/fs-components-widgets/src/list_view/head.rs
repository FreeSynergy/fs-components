//! The row of headings of the list.
//!
//! Pattern: none — plain elements; a heading with a `sort` key is a thing to press, and an arrow shows the order.

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::model::ListView;

/// Height of the row of headings in pixels.
pub(super) const HEAD_HEIGHT: f32 = 28.0;

pub(super) fn head(list: &ListView, widths: &[f32]) -> Div {
    let mut element = div().class("fs-list-head").flex_row().h(HEAD_HEIGHT);
    for (column, width) in list.columns.iter().zip(widths) {
        let arrow = match column.order {
            Some(true) => " \u{25BE}",
            Some(false) => " \u{25B4}",
            None => "",
        };
        let mut cell = div().class("fs-list-cell").w(*width).child(
            text(format!("{}{arrow}", column.title))
                .no_wrap()
                .no_cursor(),
        );
        if let (Some(key), Some(on_head)) = (column.sort.clone(), list.on_head.clone()) {
            cell = cell
                .class("fs-list-cell-sortable")
                .on_click(move |_| on_head(&key));
        }
        element = element.child(cell);
    }
    element
}
