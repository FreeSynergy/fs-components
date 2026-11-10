//! The findings under a form: field, slot and kind of finding, each its own text and its own ids.
//!
//! Pattern: View function — session state in, elements out (the area is drawn again after each change by `Live`).

use blinc_layout::div::{Div, div};
use fs_types::{RecordIssue, RecordProblem};

use super::form::FormEnv;
use crate::form::FormModel;

fn lock(
    s: &std::sync::Mutex<crate::session::Session>,
) -> std::sync::MutexGuard<'_, crate::session::Session> {
    s.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The translation key of a finding.
pub(super) fn problem_key(p: RecordProblem) -> String {
    match p {
        RecordProblem::UnknownField => "problem.unknown-field".to_owned(),
        RecordProblem::Missing => "problem.missing".to_owned(),
        RecordProblem::NotMultiple => "problem.not-multiple".to_owned(),
        RecordProblem::Value(v) => format!("problem.{}", v.code()),
    }
}

pub(super) fn findings_view(env: &FormEnv) -> Div {
    let (s, pal) = (env.look.style, env.look.palette());
    let session = lock(&env.session);
    let issues: Vec<RecordIssue> = session
        .form()
        .map(FormModel::visible_issues)
        .unwrap_or_default();
    let mut column = div().flex_col().gap_px(s.spacing.xs).w_full();
    for i in issues {
        // A finding on a field the form does not have never gets here: the session raises it as a message.
        let Some(field) = session
            .form()
            .and_then(|f| f.fields().iter().find(|x| x.name == i.field))
        else {
            continue;
        };
        // Two texts side by side, never joined: word order is the translator's business (coding.md section 2a).
        let (name, code) = (&field.name, problem_key(i.problem));
        // Which value it is about, as a number, when the field holds several; ids tell field, slot and kind of finding.
        let slot = (field.multiple
            && session
                .form()
                .is_some_and(|f| f.values(&field.name).len() > 1))
        .then(|| {
            div()
                .id(format!("finding-{name}-{}-slot", i.index))
                .child(
                    env.look
                        .text((i.index + 1).to_string(), s.text.caption, pal.error),
                )
        });
        column = column.child(
            div()
                .flex_row()
                .gap_px(s.spacing.s)
                .child(
                    div()
                        .id(format!("finding-{name}-{}-{code}-label", i.index))
                        .child(
                            env.look
                                .text(env.say.say(&field.label), s.text.caption, pal.error),
                        ),
                )
                .children(slot)
                .child(
                    div()
                        .id(format!("finding-{name}-{}-{code}-problem", i.index))
                        .child(env.look.text(env.say.say(&code), s.text.caption, pal.error)),
                ),
        );
    }
    column
}
