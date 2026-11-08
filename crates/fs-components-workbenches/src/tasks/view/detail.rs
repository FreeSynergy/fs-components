//! The task: read, or the form while editing.
//!
//! Pattern: none (pure functions of what is shown) — a task, the form, a failure or nothing. Actions the source cannot do
//! are not offered. Delete and discarding ask in one place and are done in another.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::{Act, TaskActions, button};
use super::form::form;
use crate::input_slots::InputSlots;
use crate::page_scroll::PageScroll;
use crate::tasks::context::TaskView;
use crate::tasks::model::{TaskSection, TaskSheet};
use crate::tasks::shown::Shown;
use crate::tasks::state::TasksState;

/// The area beside the work area. All of it scrolls, the buttons too: in a small window there is no
/// room to keep them apart from the page.
pub(in crate::tasks) fn detail(
    state: &TasksState,
    name: &str,
    actions: &TaskActions,
    slots: &InputSlots,
    scroller: &PageScroll,
) -> Div {
    let editing = state.editing().is_some();
    // Editing: the buttons stand on top as always, but come last in the tree, so Tab walks the
    // fields in reading order, then Save, then Cancel. The column and the bar are drawn in reverse.
    let mut column = div().class("fs-tasks-detail").w_full();
    column = if editing {
        column.flex_col_reverse()
    } else {
        column.flex_col()
    };
    // The compact variant offers nothing to press: no empty band.
    let bar =
        (editing || state.can_edit() || state.can_delete()).then(|| bar(state, name, actions));
    let notice = state.notice().map(|notice| {
        div()
            .id(format!("{name}-notice"))
            .class("fs-tasks-notice")
            .child(text(notice))
    });
    let deed = deed(state, name, actions);
    let page = div().class("fs-tasks-page").flex_col().w_full();
    let page = if let Some(edit) = state.editing() {
        form(edit, name, actions, slots, &|key| state.problem(key), page)
    } else {
        match state.shown() {
            Shown::Task { sheet, .. } => task(state, sheet, name, actions, page),
            Shown::Failed { reason, .. } => {
                page.child(div().class("fs-tasks-failed").child(text(reason.clone())))
            }
            _ => page.child(text(&state.labels().nothing_selected)),
        }
    };
    if editing {
        // Tree order; drawn from the bottom up.
        column = column
            .child(page)
            .children(deed)
            .children(notice)
            .children(bar);
    } else {
        column = column
            .children(bar)
            .children(notice)
            .children(deed)
            .child(page);
    }
    let what = match state.editing() {
        Some(edit) => format!("edit-{}", edit.id),
        None => state.selected().unwrap_or_default().to_string(),
    };
    div()
        .id(format!("{name}-detail"))
        .flex_1()
        .min_h(0.0)
        .w_full()
        .child(
            scroller
                .scroller(&what)
                .w_full()
                .h_full()
                .content(column.id(format!("{name}-page"))),
        )
}

/// The second action of a question, on a line of its own below the bar: a second press at the place of
/// the first can never be it, at any width.
fn deed(state: &TasksState, name: &str, actions: &TaskActions) -> Option<Div> {
    let labels = state.labels();
    let button = if state.editing().is_some_and(|edit| edit.discard_asked) {
        button(
            format!("{name}-discard"),
            &labels.discard_really,
            ButtonVariant::Destructive,
            &actions.confirm_discard,
        )
    } else if state.can_delete() && state.delete_asked() {
        button(
            format!("{name}-really"),
            &labels.delete_really,
            ButtonVariant::Destructive,
            &actions.confirm_delete,
        )
    } else {
        return None;
    };
    Some(div().class("fs-tasks-really").flex_row().child(button))
}

/// The buttons that fit the state. Save and Cancel stand at the far side, away from Edit and Delete, so
/// the second press of a double-click on Edit lands on nothing; Keep editing stands where Cancel was.
fn bar(state: &TasksState, name: &str, actions: &TaskActions) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-tasks-bar").flex_row().flex_wrap();
    if let Some(edit) = state.editing() {
        let cancel_or_keep = if edit.discard_asked {
            button(
                id("keep-editing"),
                &labels.discard_keep,
                ButtonVariant::Secondary,
                &actions.keep_editing,
            )
        } else {
            button(
                id("cancel"),
                &labels.cancel,
                ButtonVariant::Secondary,
                &actions.cancel,
            )
        };
        // Tree order Save, Cancel, spacer; drawn in reverse: spacer, Cancel, Save.
        return bar
            .flex_row_reverse()
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ))
            .child(cancel_or_keep)
            .child(div().class("fs-tasks-spacer"));
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
        bar = bar.child(if state.delete_asked() {
            button(
                id("keep"),
                &labels.delete_keep,
                ButtonVariant::Secondary,
                &actions.keep,
            )
        } else {
            button(
                id("delete"),
                &labels.delete,
                ButtonVariant::Ghost,
                &actions.ask_delete,
            )
        });
    }
    bar
}

fn task(
    state: &TasksState,
    sheet: &TaskSheet,
    name: &str,
    actions: &TaskActions,
    page: Div,
) -> Div {
    let head = div().class("fs-tasks-head").flex_row().flex_wrap().child(
        div()
            .class("fs-tasks-title")
            .child(text(&sheet.title).class("fs-tasks-title-text")),
    );
    let mut page = page.child(with_marks(head, &sheet.marks));
    if let Some(note) = &sheet.note {
        page = page.child(div().class("fs-tasks-note").child(text(note)));
    }
    if state.can_move() {
        page = page.child(move_row(state, sheet, name, actions));
    }
    for section in &sheet.sections {
        page = page.child(section_element(section));
    }
    page
}

/// One button per column, at fixed places, also for the one the task is in: pressing that does
/// nothing, so a double-click cannot go further than the person chose.
fn move_row(state: &TasksState, sheet: &TaskSheet, name: &str, actions: &TaskActions) -> Div {
    let labels = state.labels();
    let mut block = div().class("fs-tasks-section").flex_col().child(
        div()
            .class("fs-tasks-section-title")
            .child(text(&labels.move_to)),
    );
    if let Some(refusal) = &sheet.move_refusal {
        return block.child(div().class("fs-tasks-note").child(text(refusal)));
    }
    // The status view says it above the columns.
    if state.view() == TaskView::List {
        block = block.child(
            div()
                .class("fs-tasks-note")
                .child(text(&labels.move_warning)),
        );
    }
    let mut row = div().class("fs-tasks-choices").flex_row().flex_wrap();
    for column in state.columns().iter() {
        let (act, key) = (Arc::clone(&actions.move_selected), column.key.clone());
        let press: Act = Arc::new(move || act(&key));
        let variant = if column.key == sheet.column {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        };
        row = row.child(button(
            format!("{name}-move-{}", column.key),
            &column.title,
            variant,
            &press,
        ));
    }
    block.child(row)
}

/// `row` with the markings added.
pub(super) fn with_marks(mut row: Div, marks: &[String]) -> Div {
    for mark in marks {
        row = row.child(div().class("fs-tasks-mark").child(text(mark)));
    }
    row
}

fn section_element(section: &TaskSection) -> Div {
    let mut element = div().class("fs-tasks-section").flex_col();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-tasks-section-title")
                .child(text(&section.title)),
        );
    }
    for field in &section.fields {
        element = element.child(
            div()
                .class("fs-tasks-field")
                .flex_row()
                .child(div().class("fs-tasks-label").child(text(&field.label)))
                .child(
                    div()
                        .class("fs-tasks-value")
                        .flex_1()
                        .child(text(&field.value)),
                ),
        );
    }
    element
}
