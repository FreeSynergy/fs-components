//! The list page: head with sort, the rows, the search.
//!
//! Pattern: Adapter over the `ListView` widget (virtual rows, keys, scroll position) with a head of our own, because
//! the widget's head cannot be clicked to sort. Cells are the text of the cell component of the type (`BasicField::cell`)
//! or the canonical text.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_fields::{Look, Say};
use fs_components_widgets::Element;
use fs_components_widgets::list_view::{Column, ColumnWidth, ListPosition, ListView, Row, VecRows};

use super::kinds::kind_of;
use crate::list::{Direction, ListModel};

/// What the page tells its owner.
pub struct ListEvents {
    /// A row was chosen (key = record id).
    pub open: Arc<dyn Fn(&str) + Send + Sync>,
    /// A column head was pressed (index).
    pub sort: Arc<dyn Fn(usize) + Send + Sync>,
}

/// The rows as the list widget takes them.
pub fn rows(model: &ListModel) -> Vec<Row> {
    model
        .rows()
        .into_iter()
        .map(|record| {
            let cells = (0..model.columns().len())
                .map(|c| {
                    let (text, more) = model.cell(record, c);
                    let field = &model.columns()[c].field;
                    let shown = kind_of(field.basic).map_or(text.clone(), |k| {
                        let f = k.field();
                        f.cell(&f.split(&text))
                    });
                    if more > 0 {
                        format!("{shown}  +{more}")
                    } else {
                        shown
                    }
                })
                .collect();
            Row::new(record.id.clone(), cells)
        })
        .collect()
}

/// The clickable head, as wide as the columns below it.
fn head(look: &Look, say: &dyn Say, model: &ListModel, widths: &[f32], events: &ListEvents) -> Div {
    let (s, pal) = (look.style, look.palette());
    let mut row = div()
        .class("fs-admin-head")
        .flex_row()
        .items_center()
        .h(s.heights.head)
        .w_full()
        .bg(Look::color(pal.panel));
    for (i, (column, width)) in model.columns().iter().zip(widths).enumerate() {
        let mut cell = div()
            .flex_row()
            .items_center()
            .gap_px(s.spacing.xs)
            .w(*width)
            .padding_x_px(s.spacing.s)
            .cursor_pointer()
            .child(look.text(say.say(&column.field.label), s.text.body, pal.text_weak));
        if let Some((c, dir)) = model.sort()
            && c == i
        {
            let word = if dir == Direction::Ascending {
                "sort.ascending"
            } else {
                "sort.descending"
            };
            cell = cell.child(look.text(say.say(word), s.text.caption, pal.accent));
        }
        let sort = Arc::clone(&events.sort);
        row = row.child(
            cell.id(format!("head-{}", column.field.name))
                .on_click(move |_| sort(i)),
        );
    }
    row
}

/// The list page body, `width` x `height`.
pub fn list_body(
    look: &Look,
    say: &dyn Say,
    model: &ListModel,
    position: &ListPosition,
    (width, height): (f32, f32),
    events: &ListEvents,
) -> impl Element + use<> {
    let s = look.style;
    let n = model.columns().len().max(1);
    #[allow(clippy::cast_precision_loss)]
    let each = (width / n as f32).floor().max(1.0);
    let widths = vec![each; n];
    let columns = widths
        .iter()
        .map(|w| Column::new("", ColumnWidth::Pixels(*w)))
        .collect();
    let open = Arc::clone(&events.open);
    let list = ListView::new(columns, Arc::new(VecRows::new(rows(model))))
        .selected(model.selected())
        .on_select(Arc::new(move |key| open(key)))
        .row_height(s.heights.row_dense)
        .position(position)
        .keys(true)
        .named("records")
        .when_empty(say.say("list.empty"));
    div()
        .flex_col()
        .w(width)
        .child(head(look, say, model, &widths, events))
        .child(list.build(width, (height - s.heights.head).max(0.0)))
}
