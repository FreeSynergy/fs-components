//! The form model: values per field, what is wrong, what the person has touched.
//!
//! Pattern: Unit of Work — it remembers what was there at the start (`initial`) and hands back what changed (and the
//! required fields), checked, on [`FormModel::submit`] (a field the person emptied comes back as an empty list =
//! cleared); the checking is the one flow of `fs-types` (`check_record`), so this form and the program's own `check`
//! command say the same (Strategy by kind there). Findings show only where the person has been (touched) or after a
//! try to send; a `Missing` never shows before that. No trait of its own: the boundary is `Backend`; this is a value
//! the session owns.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use fs_types::{BasicRules, FieldDescription, RecordIssue, RecordProblem, TypeDescription};

use crate::backend::Values;

/// Numbers for the generations of forms.
static NEXT: AtomicU64 = AtomicU64::new(1);

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
    generation: u64,
    fields: Vec<FieldDescription>,
    values: Values,
    initial: Values,
    touched: BTreeSet<(String, usize)>,
    attempted: bool,
    refused: Vec<RecordIssue>,
    /// Per sent field: the slot each sent value came from (empty slots are not sent), for mapping a program's finding
    /// back to the slot the person typed in.
    sent: BTreeMap<String, Vec<usize>>,
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
            generation: NEXT.fetch_add(1, Ordering::Relaxed),
            fields: fields.to_vec(),
            initial: values.clone(),
            values,
            touched: BTreeSet::new(),
            attempted: false,
            refused: Vec::new(),
            sent: BTreeMap::new(),
        }
    }

    /// A number no other form has: a view that keeps inputs for a form makes them again when it changes.
    pub fn generation(&self) -> u64 {
        self.generation
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
        self.refused
            .retain(|i| !(i.field == field && i.index == index));
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
        // The slots after the removed one move up by one: marks and refusals move with their values.
        self.touched = std::mem::take(&mut self.touched)
            .into_iter()
            .filter(|(f, n)| f != field || *n != index)
            .map(|(f, n)| {
                if f == field && n > index {
                    (f, n - 1)
                } else {
                    (f, n)
                }
            })
            .collect();
        self.refused
            .retain(|i| !(i.field == field && i.index == index));
        for i in &mut self.refused {
            if i.field == field && i.index > index {
                i.index -= 1;
            }
        }
        true
    }

    /// Every finding on the values now, shown or not.
    pub fn issues(&self) -> Vec<RecordIssue> {
        let mut found = self.own_issues();
        found.extend(self.refused.iter().cloned());
        let mut seen: Vec<RecordIssue> = Vec::new();
        found.retain(|i| {
            let new = !seen.contains(i);
            if new {
                seen.push(i.clone());
            }
            new
        });
        found
    }

    /// What the rules of the basic types find. A program's refusal is not among them: it does not stop a new try (the
    /// person may not be able to reach what it points at), the program is asked again.
    fn own_issues(&self) -> Vec<RecordIssue> {
        as_type(&self.fields).check_record(rules(), &self.values)
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

    /// Whether the person changed anything that would be sent: the cleaned values are compared (an empty slot, or the
    /// same value with other spaces or case in the domain, is no change). A text that fails its rule counts as typed.
    pub fn is_dirty(&self) -> bool {
        self.fields.iter().any(|f| {
            let norm = |from: &Values| -> Vec<String> {
                from.get(&f.name)
                    .map(|v| {
                        v.iter()
                            .map(|t| t.trim())
                            .filter(|t| !t.is_empty())
                            .map(|t| {
                                rules()
                                    .check(f.basic, t, &f.choices)
                                    .unwrap_or_else(|_| t.to_owned())
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };
            norm(&self.values) != norm(&self.initial)
        })
    }

    /// The program refused the input: mark its findings. A finding's index counts the values that were sent (empty
    /// slots are not), so it is mapped back to the slot the person typed in.
    pub fn refuse(&mut self, issues: Vec<RecordIssue>) {
        let sent = std::mem::take(&mut self.sent);
        self.refused = issues
            .into_iter()
            .map(|mut i| {
                // The program counts the values it got; one past the end points at the last value that was sent.
                let sent_slot = sent
                    .get(&i.field)
                    .and_then(|v| v.get(i.index).or_else(|| v.last()));
                if let Some(slot) = sent_slot {
                    i.index = *slot;
                }
                // a finding on a slot that is not there points at the last one, where the person can see it
                let slots = self.values.get(&i.field).map_or(0, Vec::len);
                i.index = i.index.min(slots.saturating_sub(1));
                i
            })
            .collect();
    }

    /// Try to send. Fields whose values changed since the start come back, and the required ones always (a program
    /// asks for them), as canonical texts with empty slots left out; a field the person emptied comes back as an
    /// empty list (cleared). With findings nothing is returned and they show from now on.
    ///
    /// # Errors
    /// All findings.
    pub fn submit(&mut self) -> Result<Values, Vec<RecordIssue>> {
        self.attempted = true;
        let found = self.own_issues();
        if !found.is_empty() {
            return Err(found);
        }
        let mut out = Values::new();
        let mut sent = BTreeMap::new();
        for f in &self.fields {
            let now = self.canonical(&f.name, &self.values);
            if f.required || now != self.canonical(&f.name, &self.initial) {
                let slots = self
                    .values(&f.name)
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| !t.trim().is_empty())
                    .map(|(i, _)| i)
                    .collect();
                sent.insert(f.name.clone(), slots);
                out.insert(f.name.clone(), now);
            }
        }
        self.sent = sent;
        Ok(out)
    }

    /// The canonical texts of a field, empty slots left out (a text that fails its rule is left out too; `submit`
    /// has found those before it asks).
    fn canonical(&self, field: &str, from: &Values) -> Vec<String> {
        let Some(f) = self.fields.iter().find(|f| f.name == field) else {
            return Vec::new();
        };
        from.get(field)
            .map(|v| {
                v.iter()
                    .filter(|t| !t.trim().is_empty())
                    .filter_map(|t| rules().check(f.basic, t, &f.choices).ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}
