//! The item: read, or the form while editing.
//!
//! Pattern: Strategy by state — an item, the form, a failure or nothing. Actions the source cannot do
//! are not offered. Asking and confirming a delete are two actions in different places: what lies
//! where Delete was keeps the item, and the deed is across the bar.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;

use super::actions::CalendarActions;
use super::form::{InputSlots, form};
use super::toolbar::button;
use crate::calendar::model::{ItemSheet, SheetSection};
use crate::calendar::shown::Shown;
use crate::calendar::state::CalendarState;

/// The detail area. `back` adds the way back where the calendar is out of sight.
pub(super) fn detail(
    state: &CalendarState,
    name: &str,
    actions: &CalendarActions,
    slots: &InputSlots,
    back: bool,
) -> Div {
    let mut column = div()
        .flex_col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .child(bar(state, name, actions, back));
    if let Some(notice) = state.notice() {
        column = column.child(div().class("fs-calendar-notice").child(text(notice)));
    }
    let page = div().class("fs-calendar-page").flex_col().w_full();
    let page = if let Some(edit) = state.editing() {
        form(edit, state.labels(), name, actions, slots, page)
    } else {
        match state.shown() {
            Shown::Item { sheet, .. } => item(sheet, page),
            Shown::Failed { reason, .. } => page.child(
                div()
                    .class("fs-calendar-failed")
                    .child(text(reason.clone())),
            ),
            _ => page.child(text(&state.labels().nothing_selected)),
        }
    };
    column.child(
        div()
            .flex_1()
            .min_h(0.0)
            .w_full()
            .child(scroll().w_full().h_full().content(page)),
    )
}

/// The buttons that fit the state.
fn bar(state: &CalendarState, name: &str, actions: &CalendarActions, back: bool) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-calendar-bar").flex_row().items_center();
    if state.editing().is_some() {
        return bar
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ))
            .child(button(
                id("cancel"),
                &labels.cancel,
                ButtonVariant::Secondary,
                &actions.cancel,
            ));
    }
    if back {
        bar = bar.child(button(
            id("back"),
            &labels.back,
            ButtonVariant::Ghost,
            &actions.back,
        ));
    }
    if state.can_edit() {
        bar = bar.child(button(
            id("edit"),
            &labels.edit,
            ButtonVariant::Secondary,
            &actions.edit,
        ));
    }
    if state.can_delete() {
        if state.delete_asked() {
            bar = bar
                .child(button(
                    id("keep"),
                    &labels.delete_keep,
                    ButtonVariant::Secondary,
                    &actions.keep,
                ))
                .child(div().flex_1())
                .child(button(
                    id("really"),
                    &labels.delete_really,
                    ButtonVariant::Destructive,
                    &actions.confirm_delete,
                ));
        } else {
            bar = bar.child(button(
                id("delete"),
                &labels.delete,
                ButtonVariant::Ghost,
                &actions.ask_delete,
            ));
        }
    }
    bar
}

fn item(sheet: &ItemSheet, page: Div) -> Div {
    let head = div()
        .class("fs-calendar-title")
        .flex_row()
        .child(div().child(text(&sheet.title).class("fs-calendar-title-text")));
    let mut page = page.child(with_marks(head, &sheet.marks));
    if let Some(note) = &sheet.note {
        page = page.child(div().class("fs-calendar-note").child(text(note)));
    }
    for section in &sheet.sections {
        page = page.child(section_element(section));
    }
    page
}

/// `row` with the markings added.
pub(super) fn with_marks(mut row: Div, marks: &[String]) -> Div {
    for mark in marks {
        row = row.child(div().class("fs-calendar-sheet-mark").child(text(mark)));
    }
    row
}

fn section_element(section: &SheetSection) -> Div {
    let mut element = div().class("fs-calendar-section").flex_col();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-calendar-section-title")
                .child(text(&section.title)),
        );
    }
    for field in &section.fields {
        element = element.child(
            div()
                .class("fs-calendar-field")
                .flex_row()
                .child(div().class("fs-calendar-label").child(text(&field.label)))
                .child(
                    div()
                        .class("fs-calendar-value")
                        .flex_1()
                        .child(text(&field.value)),
                ),
        );
    }
    element
}
