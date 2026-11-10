//! The form page: one row per field, a slot per value, the findings under them.
//!
//! Pattern: Adapter + Observer — the slots are made once when the form opens and keep their text (the inputs are not
//! inside a live area, so a redraw never takes the keyboard, as in `fs-components-fields`); the findings live in an
//! area of their own that is drawn again after each change. The model ([`crate::FormModel`]) is the truth: a slot
//! tells it every change.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use fs_components_fields::{Look, Say, Suggest};
use fs_components_widgets::Element;
use fs_components_widgets::live::Live;

use super::chrome::{cn_button, panel};
use super::findings::findings_view;
use super::inputs::{Body, Slot, choice, plain};
use crate::session::Session;

/// The slots of the open form, by field and value index.
#[derive(Default)]
pub struct FormSlots {
    key: String,
    epoch: u64,
    slots: Vec<(String, usize, Slot)>,
}

impl FormSlots {
    fn slot(&self, field: &str, i: usize) -> Option<&Slot> {
        self.slots
            .iter()
            .find(|(f, n, _)| f == field && *n == i)
            .map(|(_, _, s)| s)
    }
}

/// What the page needs from its owner.
pub struct FormEnv {
    /// The session.
    pub session: Arc<Mutex<Session>>,
    /// Look and words.
    pub look: Look,
    /// Words.
    pub say: Arc<dyn Say>,
    /// Completions, if the program offers them.
    pub suggest: Option<Arc<dyn Suggest>>,
    /// The area of the findings.
    pub findings: Live,
    /// The whole page, after the structure changed (a slot added or removed, a choice made).
    pub page: Live,
}

fn lock(s: &Mutex<Session>) -> std::sync::MutexGuard<'_, Session> {
    s.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The page body for the open form.
pub fn form_body(env: &Arc<FormEnv>, slots: &Arc<Mutex<FormSlots>>) -> Div {
    let (s, pal) = (env.look.style, env.look.palette());
    let (key, fields) = {
        let session = lock(&env.session);
        let Some(form) = session.form() else {
            return div();
        };
        // The generation of the form is part of the key: the same command on another record is another form
        // (review finding 1: the old inputs, with the old record's text, were kept).
        (
            format!("{}#{}", session.screen().key(), form.generation()),
            form.fields().to_vec(),
        )
    };
    ensure(env, slots, &key);
    let mut page = panel(&env.look);
    for field in &fields {
        let values = lock(&env.session)
            .form()
            .map(|f| f.values(&field.name).to_vec())
            .unwrap_or_default();
        let label = env.say.say(&field.label);
        let mut column = div()
            .flex_col()
            .gap_px(s.spacing.xs)
            .flex_grow_value(1.0)
            .min_w(0.0);
        for (i, value) in values.iter().enumerate() {
            column = column.child(slot_row(env, slots, field, i, value, values.len()));
        }
        if field.multiple {
            let (e, name) = (Arc::clone(env), field.name.clone());
            column = column.child(cn_button(
                &env.say.say("action.add"),
                ButtonVariant::Ghost,
                &format!("add-{}", field.name),
                move || {
                    lock(&e.session).form_mut().map(|f| f.add_value(&name));
                    e.page.refresh_now();
                },
            ));
        }
        page = page.child(
            div()
                .class("fs-admin-field")
                .flex_row()
                .gap_px(s.spacing.m)
                .w_full()
                .child(
                    div()
                        .w(s.text.body * 12.0)
                        .flex_shrink_0()
                        .flex_row()
                        .gap_px(s.spacing.xs)
                        .child(env.look.text(label, s.text.body, pal.text_weak))
                        // the mark of a required field is a sign, not a word; other fields have none
                        .children(field.required.then(|| {
                            div()
                                .id(format!("required-{}", field.name))
                                .child(env.look.text("*", s.text.body, pal.error))
                        })),
                )
                .child(column),
        );
    }
    let on = Arc::clone(env);
    page.child(env.findings.region("findings", move || findings_view(&on)))
}

fn slot_row(
    env: &Arc<FormEnv>,
    slots: &Arc<Mutex<FormSlots>>,
    field: &fs_types::FieldDescription,
    i: usize,
    value: &str,
    count: usize,
) -> Div {
    let s = env.look.style;
    let name = format!("{}-{i}", field.name);
    let guard = slots.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(slot) = guard.slot(&field.name, i) else {
        return div();
    };
    let input: Box<dyn Element> = match &slot.body {
        Body::Typed(input) => Box::new(input.build()),
        Body::Plain(data) => Box::new(plain(
            &env.look,
            &*env.say,
            field,
            data,
            &name,
            Arc::clone(&slot.on),
        )),
        Body::Choice => Box::new(choice(&env.look, &*env.say, field, value, &name, &slot.on)),
    };
    let mut row = div()
        .flex_row()
        .items_start()
        .gap_px(s.spacing.s)
        .w_full()
        .child(div().flex_grow_value(1.0).min_w(0.0).child(input));
    if field.multiple && count > 1 {
        let (e, f) = (Arc::clone(env), field.name.clone());
        row = row.child(cn_button(
            &env.say.say("action.remove"),
            ButtonVariant::Ghost,
            &format!("remove-{name}"),
            move || {
                lock(&e.session).form_mut().map(|m| m.remove_value(&f, i));
                e.page.refresh_now();
            },
        ));
    }
    row
}

/// Make the slots again when another form opened or a slot came or went.
fn ensure(env: &Arc<FormEnv>, slots: &Arc<Mutex<FormSlots>>, key: &str) {
    let shape: Vec<(String, Vec<String>)> = lock(&env.session)
        .form()
        .map(|f| {
            f.fields()
                .iter()
                .map(|x| (x.name.clone(), f.values(&x.name).to_vec()))
                .collect()
        })
        .unwrap_or_default();
    let fields = lock(&env.session)
        .form()
        .map(|f| f.fields().to_vec())
        .unwrap_or_default();
    let mut guard = slots.lock().unwrap_or_else(PoisonError::into_inner);
    let now: usize = shape.iter().map(|(_, v)| v.len()).sum();
    if guard.key == key && guard.slots.len() == now && guard.epoch == epoch_of(&shape) {
        return;
    }
    key.clone_into(&mut guard.key);
    guard.epoch = epoch_of(&shape);
    guard.slots.clear();
    for field in &fields {
        let values = shape
            .iter()
            .find(|(n, _)| n == &field.name)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        for (i, value) in values.iter().enumerate() {
            let (e, name) = (Arc::clone(env), field.name.clone());
            let choice = field.basic == fs_types::BasicType::Choice;
            let on: super::inputs::OnText = Arc::new(move |text| {
                lock(&e.session).form_mut().map(|f| f.set(&name, i, text));
                if choice {
                    e.page.refresh_now();
                } else {
                    e.findings.refresh_now();
                }
            });
            guard.slots.push((
                field.name.clone(),
                i,
                Slot::new(
                    env.look,
                    &env.say,
                    field,
                    value,
                    &format!("{}-{i}", field.name),
                    env.suggest.as_ref(),
                    on,
                ),
            ));
        }
    }
}

/// A number that changes when a slot came or went, so the slots are made again.
fn epoch_of(shape: &[(String, Vec<String>)]) -> u64 {
    shape.iter().fold(0u64, |h, (n, v)| {
        h.wrapping_mul(31)
            .wrapping_add(n.len() as u64)
            .wrapping_mul(31)
            .wrapping_add(v.len() as u64)
    })
}
