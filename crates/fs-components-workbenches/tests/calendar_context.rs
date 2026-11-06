//! What the workbench gives back to the program around it (ADR 0056).
//!
//! Pattern: State machine test, no window.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::{Diary, d};
use fs_components_workbenches::calendar::{
    CalendarLabels, CalendarState, Picked, Variant, ViewRange,
};

#[test]
fn the_context_is_what_is_shown_as_a_plain_value() {
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-04")]));
    let mut s = CalendarState::new(diary, CalendarLabels::default(), Variant::Full);
    let context = s.context();
    assert_eq!(
        (context.item, context.view, context.date),
        (None, ViewRange::Month, d("2026-11-06"))
    );
    s.set_view(ViewRange::Week);
    s.pick(&Picked::Day(d("2026-11-05")));
    s.select("ada");
    let context = s.context();
    assert_eq!(context.item.as_deref(), Some("ada"));
    assert_eq!(
        (context.view, context.date),
        (ViewRange::Week, d("2026-11-05"))
    );
    s.next();
    assert_eq!(
        s.context().date,
        d("2026-11-13"),
        "the day chosen is out of view"
    );
}
