//! The form model: values per field, what is wrong, what the person has touched.
//!
//! Pattern: Unit of Work — it remembers what was there at the start (`initial`) and hands back only a changed, checked
//! set on [`FormModel::submit`]; the checking is the one flow of `fs-types` (`check_record`), so this form and the
//! program's own `check` command say the same (Template method there). Findings show only where the person has been
//! (touched) or after a try to send; a `Missing` never shows before that.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use fs_types::{BasicRules, FieldDescription, RecordIssue, RecordProblem, TypeDescription};

use crate::backend::Values;

/// The rules of the basic types, built once.
pub fn rules() -> &'static BasicRules {
    static RULES: OnceLock<BasicRules> = OnceLock::new();
    RULES.get_or_init(BasicRules::standard)
}

/// A description of just these fields, to run `check_record` on them.
fn as_type(fields: &[FieldDescription]) -> TypeDescription {
    let mut t = TypeDescription::new("Form", "form", "form", "form");
    t.fields = fields.to_vec();
    t
}

/// A form over a list of fields (the fields of a type, or the input of a command).
#[derive(Clone, Debug)]
pub struct FormModel {
    fields: Vec<FieldDescription>,
    values: Values,
    initial: Values,
    touched: BTreeSet<(String, usize)>,
    attempted: bool,
    refused: Vec<RecordIssue>,
}

impl FormModel {
    /// A form for `fields`, filled with `initial` (values of other names are ignored). Every field has at least one
    /// slot to type into.
    pub fn new(fields: &[FieldDescription], initial: &Values) -> Self {
        let mut values = Values::new();
        for f in fields {
            let mut v = initial.get(&f.name).cloned().unwrap_or_default();
            if !f.multiple {
                v.truncate(1);
            }
            if v.is_empty() {
                v.push(String::new());
            }
            values.insert(f.name.clone(), v);
        }
        Self {
            fields: fields.to_vec(),
            initial: values.clone(),
            values,
            touched: BTreeSet::new(),
            attempted: false,
            refused: Vec::new(),
        }
    }

    /// The fields, in order.
    pub fn fields(&self) -> &[FieldDescription] {
        &self.fields
    }

    /// The slots of a field as typed (empty slots included); empty for an unknown field.
    pub fn values(&self, field: &str) -> &[String] {
        self.values.get(field).map_or(&[], Vec::as_slice)
    }

    /// Type into a slot. Returns `false` for an unknown field or slot.
    pub fn set(&mut self, field: &str, index: usize, text: &str) -> bool {
        let Some(slot) = self.values.get_mut(field).and_then(|v| v.get_mut(index)) else {
            return false;
        };
        text.clone_into(slot);
        self.touched.insert((field.to_owned(), index));
        self.refused.retain(|i| i.field != field);
        true
    }

    /// Add an empty slot to a `multiple` field. Returns `false` for another field.
    pub fn add_value(&mut self, field: &str) -> bool {
        let multiple = self.fields.iter().any(|f| f.name == field && f.multiple);
        match self.values.get_mut(field) {
            Some(v) if multiple => {
                v.push(String::new());
                true
            }
            _ => false,
        }
    }

    /// Remove a slot; the last slot of a field is emptied instead. Returns `false` for an unknown field or slot.
    pub fn remove_value(&mut self, field: &str, index: usize) -> bool {
        let Some(v) = self.values.get_mut(field) else {
            return false;
        };
        if index >= v.len() {
            return false;
        }
        if v.len() == 1 {
            v[0].clear();
        } else {
            v.remove(index);
        }
        self.touched.retain(|(f, _)| f != field);
        self.refused.retain(|i| i.field != field);
        true
    }

    /// Every finding on the values now, shown or not.
    pub fn issues(&self) -> Vec<RecordIssue> {
        let mut found = as_type(&self.fields).check_record(rules(), &self.values);
        found.extend(self.refused.iter().cloned());
        found
    }

    /// The findings to show: where the person has typed, and everything after a try to send.
    pub fn visible_issues(&self) -> Vec<RecordIssue> {
        self.issues()
            .into_iter()
            .filter(|i| {
                self.attempted
                    || (i.problem != RecordProblem::Missing
                        && self.touched.contains(&(i.field.clone(), i.index)))
                    || self.refused.contains(i)
            })
            .collect()
    }

    /// Whether a value differs from what the form started with.
    pub fn is_dirty(&self) -> bool {
        self.values != self.initial
    }

    /// The program refused the input: mark its findings.
    pub fn refuse(&mut self, issues: Vec<RecordIssue>) {
        self.refused = issues;
    }

    /// Try to send. The values are the canonical texts, empty slots left out, fields without a value absent. With
    /// findings nothing is returned and they show from now on.
    ///
    /// # Errors
    /// All findings.
    pub fn submit(&mut self) -> Result<Values, Vec<RecordIssue>> {
        self.attempted = true;
        let found = self.issues();
        if !found.is_empty() {
            return Err(found);
        }
        let mut out = Values::new();
        for f in &self.fields {
            let canon: Vec<String> = self
                .values(&f.name)
                .iter()
                .filter(|t| !t.trim().is_empty())
                .filter_map(|t| rules().check(f.basic, t, &f.choices).ok())
                .collect();
            if !canon.is_empty() {
                out.insert(f.name.clone(), canon);
            }
        }
        Ok(out)
    }
}
