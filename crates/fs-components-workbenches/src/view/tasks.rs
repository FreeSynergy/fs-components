//! The checkboxes of the reading view, the same in every frontend.
//!
//! Pattern: Decorator — a text view is given its task handling: clickable with the right to change the page, shown locked
//! (with the reason under the text) without it.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Block, TextView, count_tasks};

use super::actions::Actions;
use crate::reader::Reader;

/// `view` with the checkboxes of the blocks it shows: they are numbered from `first` (the number of boxes in the blocks
/// before them on the page).
pub(crate) fn with_tasks(
    view: TextView,
    reader: &Reader,
    actions: &Actions,
    first: usize,
) -> TextView {
    if reader.can_tick() {
        view.on_task(actions.task.clone(), first)
    } else {
        view.tasks_locked(first)
    }
}

/// The line that says why the boxes of this page are locked: empty when they can be ticked or the page has none.
pub(crate) fn locked_note(reader: &Reader, blocks: &[Block]) -> Div {
    if reader.labels().tasks_locked.is_empty()
        || reader.can_tick()
        || reader.editing().is_some()
        || count_tasks(blocks) == 0
    {
        return div();
    }
    div()
        .class("fs-rd-notice")
        .child(text(&reader.labels().tasks_locked).no_cursor())
}
