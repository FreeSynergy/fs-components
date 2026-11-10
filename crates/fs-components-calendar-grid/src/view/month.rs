//! The month grid: six rows of seven day cells, bars over the days an item covers.
//!
//! Pattern: View function — pixel positions come from `layout` and `spans`; a click on a cell
//! selects its day, a click on a bar selects the item. Bars that do not fit a cell are counted in
//! a "+N" mark instead of squeezed.

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::parts::{at, chip, day_number};
use crate::handle::CalendarGrid;
use crate::layout::{Rect, cell_rect};
use crate::model::{GridItem, Slot};
use crate::spans::span_rows;

/// The month grid filling `width` x `height`.
pub fn month(grid: &CalendarGrid, items: &[GridItem], width: f32, height: f32) -> Div {
    let (look, state) = (grid.look, grid.snapshot());
    let (s, pal) = (look.style, look.palette());
    let days = state.days();
    let head = s.heights.row_dense;
    let (cw, ch) = (width / 7.0, ((height - head) / 6.0).max(s.heights.row));
    let number_h = s.text.caption * 1.8 + s.spacing.xs;
    let bar_h = (s.text.caption * 1.6).ceil();
    let capacity = (((ch - number_h) / (bar_h + 2.0)).floor() as usize).max(1);
    let (_, month, _) = state.anchor.ymd();
    let mut root = div().relative().w(width).h(ch * 6.0 + head).id("cal-month");
    for col in 0..7 {
        let name = grid.say.say(&format!(
            "calendar.weekday.{}",
            (usize::from(state.week_start) + col) % 7
        ));
        root = root.child(
            at(Rect {
                x: col as f32 * cw,
                y: 0.0,
                w: cw,
                h: head,
            })
            .padding_x_px(s.spacing.xs)
            .child(look.text(name, s.text.caption, pal.text_weak)),
        );
    }
    for (i, day) in days.iter().enumerate() {
        let r = cell_rect(i, 7, cw, ch, head);
        let (_, m, d) = day.ymd();
        let slot = Slot {
            day: *day,
            minute: None,
        };
        let picked = state.slot == Some(slot);
        let g = grid.clone();
        let mut cell = at(r)
            .id(format!("cal-cell-{i}"))
            .border(1.0, Look::color(pal.border))
            .padding_x_px(s.spacing.xs)
            .padding_y_px(s.spacing.xs)
            .on_click(move |_| g.select_slot(slot));
        if picked {
            cell = cell.bg(Look::color(pal.selected));
        }
        root = root.child(cell.child(day_number(&look, d, state.today == Some(*day), m != month)));
    }
    for (row, bars) in span_rows(items, &days, 7, false).iter().enumerate() {
        let top = head + row as f32 * ch + number_h;
        let lanes_fit = capacity;
        let mut hidden = [0usize; 7];
        for b in bars {
            if b.lane >= lanes_fit {
                (b.first..=b.last).for_each(|c| hidden[c] += 1);
                continue;
            }
            let it = &items[b.index];
            let r = Rect {
                x: b.first as f32 * cw + 2.0,
                y: top + b.lane as f32 * (bar_h + 2.0),
                w: (b.last - b.first + 1) as f32 * cw - 4.0,
                h: bar_h,
            };
            let g = grid.clone();
            let key = it.key.clone();
            let selected = state.item.as_deref() == Some(it.key.as_str());
            root = root.child(
                chip(&look, r, it.tone, &it.title, selected)
                    .id(format!("cal-item-{}-{row}", it.key))
                    .on_click(move |_| g.select_item(&key)),
            );
        }
        for (c, n) in hidden.iter().enumerate().filter(|(_, n)| **n > 0) {
            // The mark stands in the number row at the right of the cell: it never leaves the cell.
            let r = Rect {
                x: c as f32 * cw + cw / 2.0,
                y: top - number_h + s.spacing.xs,
                w: cw / 2.0 - 4.0,
                h: bar_h,
            };
            root = root.child(
                at(r)
                    .id(format!("cal-more-{row}-{c}"))
                    .pointer_events_none()
                    .flex_row()
                    .justify_end()
                    .child(look.text(format!("+{n}"), s.text.caption, pal.text_weak)),
            );
        }
    }
    root
}
