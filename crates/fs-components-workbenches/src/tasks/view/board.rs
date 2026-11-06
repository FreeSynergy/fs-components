//! The status view: what moving does, the columns, and what follows them.
//!
//! Pattern: Adapter — `sortable-list` draws the columns in a box wide enough for all of them, which
//! scrolls: columns stay readable at any width. Keyboard: Tab to the columns, arrows, Space or Enter to
//! pick a card up and put it down, Escape to give up; Open shows the card under the cursor.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;

use super::actions::{TaskActions, button};
use crate::tasks::board::Board;
use crate::tasks::state::TasksState;

/// A column is not drawn narrower than this, in pixels; the box scrolls instead.
const COLUMN_WIDTH: f32 = 160.0;

/// The status view, with `after` below the columns.
pub(in crate::tasks) fn board_area(
    state: &TasksState,
    name: &str,
    actions: &TaskActions,
    board: &Board,
    after: impl ElementBuilder + 'static,
    (width, height): (f32, f32),
) -> Div {
    let labels = state.labels();
    let mut bar = div()
        .class("fs-tasks-board-bar")
        .flex_row()
        .flex_wrap()
        .child(button(
            format!("{name}-open"),
            &labels.open,
            ButtonVariant::Secondary,
            &actions.open_cursor,
        ));
    if state.can_move() && !state.moved_once() {
        bar = bar.child(
            div()
                .id(format!("{name}-move-warning"))
                .class("fs-tasks-warning")
                .child(text(&labels.move_warning)),
        );
    }
    let columns = state.columns().len().max(1);
    #[allow(clippy::cast_precision_loss)] // a few columns
    let inner = width.max(COLUMN_WIDTH * columns as f32);
    div()
        .flex_col()
        .w(width)
        .h(height)
        .min_h(0.0)
        .child(bar)
        .child(
            div().flex_1().min_h(0.0).w(width).child(
                scroll().both_directions().w_full().h_full().content(
                    div()
                        .w(inner)
                        .min_w(inner)
                        .flex_shrink_0()
                        .child(board.list.build()),
                ),
            ),
        )
        .child(after)
}
