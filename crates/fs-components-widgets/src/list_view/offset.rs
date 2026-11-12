//! Where the list stands in its scroll container.
//!
//! Pattern: none — pure functions over a `ListPosition`.

use super::position::ListPosition;

/// Where the list stands now that it is built: the row asked for, else where it was, within the
/// rows there are now.
#[allow(clippy::cast_precision_loss)]
pub(super) fn settle(position: &ListPosition, total: f32, view: f32, row_height: f32) -> f32 {
    let was = position
        .take_wanted()
        .map_or_else(|| position.offset(), |row| row as f32 * row_height);
    let offset = was.clamp(0.0, (total - view).max(0.0));
    position.apply_offset(offset);
    offset
}

#[allow(clippy::cast_precision_loss)]
pub(super) fn bring_into_view(position: &ListPosition, row: usize, row_height: f32, view: f32) {
    let (top, bottom) = (row as f32 * row_height, (row + 1) as f32 * row_height);
    let offset = position.offset();
    if top < offset {
        position.apply_offset(top);
    } else if bottom > offset + view {
        position.apply_offset(bottom - view);
    }
}
