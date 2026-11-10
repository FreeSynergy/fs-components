//! The form as an `Edit`: what the person changed, field by field, typed.
//!
//! Pattern: Unit of Work, part of [`FormModel`] — it remembers what was there at the start and hands back only what
//! changed, as the `Edit` of the contract (a field not in it is unchanged, one with no values is cleared); the checking
//! is `Edit::check` of `fs-types`, so the form and the program say the same.

use std::collections::BTreeMap;

use fs_types::{Edit, FieldEdit, RecordIssue};

use crate::form::{FormModel, as_type, rules};

impl FormModel {
    /// Try to send a change. Only fields whose values differ from the start are in the edit; a field the person
    /// emptied has no values (cleared). With findings nothing is returned and they show from now on.
    ///
    /// # Errors
    /// All findings.
    pub fn submit_changes(&mut self) -> Result<Edit, Vec<RecordIssue>> {
        self.attempted = true;
        let found = self.own_issues();
        if !found.is_empty() {
            return Err(found);
        }
        let mut edit = Edit::new();
        let mut sent = BTreeMap::new();
        for f in &self.fields {
            let now = self.canonical(&f.name, &self.values);
            if now == self.canonical(&f.name, &self.initial) {
                continue;
            }
            let slots = self
                .values(&f.name)
                .iter()
                .enumerate()
                .filter(|(_, t)| !t.trim().is_empty())
                .map(|(i, _)| i)
                .collect();
            sent.insert(f.name.clone(), slots);
            edit.fields.push(FieldEdit {
                path: vec![f.name.clone()],
                basic: f.basic,
                values: now,
            });
        }
        let checked = edit.check(&as_type(&self.fields), rules())?;
        self.sent = sent;
        Ok(checked)
    }
}
