//! Cell: the value in a list, one line, the height of a row, no icon.
//!
//! Pattern: Adapter over Blinc's row and text elements; height and padding from the [`Look`].

use blinc_layout::div::div;

use super::Look;
use crate::kinds::Kind;

/// The cell component of `kind` for `parts`: the sort-friendly form on one line, clipped at the
/// edge of the cell, `row` high (dense: the style's dense row).
pub fn cell(
    kind: Kind,
    parts: &[String],
    look: &Look,
    dense: bool,
) -> impl fs_components_widgets::Element + use<> {
    let s = look.style;
    let pal = look.palette();
    let height = if dense {
        s.heights.row_dense
    } else {
        s.heights.row
    };
    div()
        .class("fs-field-cell")
        .flex_row()
        .items_center()
        .h(height)
        .padding_x_px(s.spacing.s)
        .overflow_clip()
        .child(look.text(kind.field().cell(parts), s.text.body, pal.text))
}
