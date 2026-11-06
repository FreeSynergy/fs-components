//! The person: read, or the form while editing.
//!
//! Pattern: Strategy by state — a person, the form, a failure or nothing. Actions the source cannot
//! do are not offered.

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;

use super::actions::{Act, PeopleActions};
use super::form::{InputSlots, form};
use crate::people::model::{PersonSection, PersonSheet};
use crate::people::shown::Shown;
use crate::people::state::PeopleState;

/// A button with an element id on a wrapper, so a program or a test can find it.
pub(super) fn button(id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    let act = std::sync::Arc::clone(act);
    div().id(id).w_fit().child(
        cn::button(label.to_string())
            .variant(variant)
            .on_click(move |_| act()),
    )
}

/// A button named `<name>-<what>`.
pub(in crate::people) fn button_for(
    name: &str,
    what: &str,
    label: &str,
    variant: ButtonVariant,
    act: &Act,
) -> Div {
    button(format!("{name}-{what}"), label, variant, act)
}

/// The area right of the list.
pub(in crate::people) fn detail(
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    slots: &InputSlots,
) -> Div {
    let mut column = div()
        .flex_col()
        .w_full()
        .flex_1()
        .min_h(0.0)
        .child(bar(state, name, actions));
    if let Some(notice) = state.notice() {
        column = column.child(div().class("fs-people-notice").child(text(notice)));
    }
    let page = div().class("fs-people-page").flex_col().w_full();
    let page = if let Some(edit) = state.editing() {
        form(edit, state.labels(), name, actions, slots, page)
    } else {
        match state.shown() {
            Shown::Person { sheet, .. } => person(sheet, page),
            Shown::Failed { reason, .. } => {
                page.child(div().class("fs-people-failed").child(text(reason.clone())))
            }
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
fn bar(state: &PeopleState, name: &str, actions: &PeopleActions) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-people-bar").flex_row();
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
                    id("delete"),
                    &labels.delete_really,
                    ButtonVariant::Destructive,
                    &actions.delete,
                ))
                .child(button(
                    id("keep"),
                    &labels.delete_keep,
                    ButtonVariant::Secondary,
                    &actions.keep,
                ));
        } else {
            bar = bar.child(button(
                id("delete"),
                &labels.delete,
                ButtonVariant::Ghost,
                &actions.delete,
            ));
        }
    }
    bar
}

fn person(sheet: &PersonSheet, page: Div) -> Div {
    let mut head = div().class("fs-people-head").flex_row().child(
        div()
            .class("fs-people-title")
            .child(text(&sheet.title).class("fs-people-title-text")),
    );
    for mark in &sheet.marks {
        head = head.child(div().class("fs-people-mark").child(text(mark)));
    }
    let mut page = page.child(head);
    for section in &sheet.sections {
        page = page.child(section_element(section));
    }
    page
}

fn section_element(section: &PersonSection) -> Div {
    let mut element = div().class("fs-people-section").flex_col();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-people-section-title")
                .child(text(&section.title)),
        );
    }
    for field in &section.fields {
        element = element.child(
            div()
                .class("fs-people-field")
                .flex_row()
                .child(div().class("fs-people-label").child(text(&field.label)))
                .child(
                    div()
                        .class("fs-people-value")
                        .flex_1()
                        .child(text(&field.value)),
                ),
        );
    }
    element
}
