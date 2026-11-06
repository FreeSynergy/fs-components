//! One item as an element.
//!
//! Pattern: Adapter — what every view shares: the words, the handler, a chip with its markings and a
//! title that fits. A click on an item is a click on the chip alone: Blinc passes a press on to every
//! ancestor and has no way to stop it, so the elements that report a day are never its ancestors.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::text_measure::measure_text;

use super::day::Day;
use super::model::{CalendarItem, CalendarNames, OnPick, Picked};
use crate::list_view::fit_cell;

/// The room `fit_cell` takes away for its padding, given back: a chip's padding is smaller.
const FIT_ALLOWANCE: f32 = 24.0;
/// `fit_cell` measures at its own, larger size; a chip's text is smaller than that.
const FIT_SCALE: f32 = 1.15;
/// Size and padding of a mark in pixels, about.
const MARK_FONT: f32 = 11.0;
const MARK_PADDING: f32 = 12.0;

/// What the elements of one drawing need.
#[derive(Clone)]
pub(super) struct Drawing {
    pub names: std::sync::Arc<CalendarNames>,
    pub name: Option<String>,
    pub on_pick: Option<OnPick>,
    pub selected_item: Option<String>,
    pub selected_day: Option<Day>,
    pub today: Option<Day>,
}

impl Drawing {
    /// Report `picked` when `element` is clicked.
    pub(super) fn reports(&self, element: Div, picked: Picked) -> Div {
        match self.on_pick.clone() {
            Some(on_pick) => element.on_click(move |_| on_pick(picked.clone())),
            None => element,
        }
    }

    /// `element` with the id `<name>-<what>`, if the view has a name.
    pub(super) fn named(&self, element: Div, what: &str) -> Div {
        match &self.name {
            Some(name) => element.id(format!("{name}-{what}")),
            None => element,
        }
    }

    /// `element` with its look as a day: today and selected.
    pub(super) fn day_classes(&self, element: Div, day: Day) -> Div {
        let element = if self.selected_day == Some(day) {
            element.class("fs-calendar-selected")
        } else {
            element
        };
        if self.today == Some(day) {
            element.class("fs-calendar-today")
        } else {
            element
        }
    }

    /// `item` on `day` as a chip `width` pixels wide, showing `label`. Its marks come first and stay
    /// whatever the width, then as much of the label as fits.
    pub(super) fn chip(&self, item: &CalendarItem, day: Day, label: &str, width: f32) -> Div {
        let mut element = div()
            .class("fs-calendar-item")
            .class(format!("fs-calendar-item-c{}", item.category % 6))
            .flex_row()
            .items_center()
            .overflow_clip();
        if self.selected_item.as_deref() == Some(item.key.as_str()) {
            element = element.class("fs-calendar-item-selected");
        }
        let mut room = width;
        for mark in &item.marks {
            room -= measure_text(mark, MARK_FONT).width + MARK_PADDING;
            element = element.child(
                div()
                    .class("fs-calendar-mark")
                    .flex_shrink_0()
                    .child(text(mark).no_wrap().no_cursor()),
            );
        }
        element = element.child(
            text(fit_cell(label, room * FIT_SCALE + FIT_ALLOWANCE))
                .no_wrap()
                .no_cursor(),
        );
        let element = self.named(element, &format!("item-{}@{day}", item.key));
        self.reports(element, Picked::Item(item.key.clone()))
    }

    /// The count of what did not fit on `day`.
    pub(super) fn more(&self, count: usize, day: Day) -> Div {
        let words = self.names.more.replacen("{}", &count.to_string(), 1);
        let element = div()
            .class("fs-calendar-more")
            .flex_row()
            .items_center()
            .child(text(words).no_wrap().no_cursor());
        let element = self.named(element, &format!("more-{day}"));
        self.reports(element, Picked::More(day))
    }
}

/// The places in `items` of what is on `day`: without a time first, then by start, then by title.
pub(super) fn places_on(items: &[CalendarItem], day: Day) -> Vec<usize> {
    let mut found: Vec<usize> = (0..items.len())
        .filter(|&i| super::layout::segment_on(&items[i], day).is_some())
        .collect();
    found.sort_by(|&a, &b| {
        let (a, b) = (&items[a], &items[b]);
        (!a.all_day, a.start, &a.title, &a.key).cmp(&(!b.all_day, b.start, &b.title, &b.key))
    });
    found
}

/// The items of `items` on `day`, in that order.
pub(super) fn items_on(items: &[CalendarItem], day: Day) -> Vec<&CalendarItem> {
    places_on(items, day)
        .into_iter()
        .map(|i| &items[i])
        .collect()
}
