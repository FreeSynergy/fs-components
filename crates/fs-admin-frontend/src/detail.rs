//! The detail of one record: every field of the type with its values.
//!
//! Pattern: none — one pure function from the description and a record to the entries the display component
//! draws. The Admin frontend shows all fields (ADR 0066 point 5), also the empty ones, so a person sees what could be
//! filled in.

use fs_types::{FieldDescription, TypeDescription};

use crate::backend::Record;

/// One field with the values the record has for it (empty slots left out).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The field.
    pub field: FieldDescription,
    /// The values; empty if the record has none.
    pub values: Vec<String>,
}

/// All fields of the type, in description order, with the record's values.
pub fn entries(description: &TypeDescription, record: &Record) -> Vec<Entry> {
    description
        .fields
        .iter()
        .map(|f| Entry {
            field: f.clone(),
            values: record
                .values
                .get(&f.name)
                .map(|v| v.iter().filter(|t| !t.trim().is_empty()).cloned().collect())
                .unwrap_or_default(),
        })
        .collect()
}
