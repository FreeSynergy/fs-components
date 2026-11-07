//! What was dropped or broken, counted: the visible side of a real data problem.
//!
//! Pattern: none — plain data (a notification): the data stage never fails on bad values and never
//! hides them; it hands this report to the layout (a note in the chart) and to the text alternative.

use super::cell::Invalid;

/// What was wrong with a value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueKind {
    /// `null` or absent where a number or date is needed.
    Null,
    /// The value is there but cannot be used.
    Invalid(Invalid),
}

/// All values of one field with one problem.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    /// The field (`count` for an aggregate without field).
    pub field: String,
    /// The problem.
    pub kind: IssueKind,
    /// How many rows.
    pub count: usize,
    /// The first offending value as written, shortened.
    pub example: Option<String>,
}

/// The counts for a chart.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Rows in the data.
    pub rows: usize,
    /// Marks' worth of data left after dropping (groups, when aggregated).
    pub kept: usize,
    /// Left out of the drawing: a needed value was missing or invalid.
    pub dropped: usize,
    /// Kept in a line or area, where they break the path.
    pub broken: usize,
    /// Every problem found, by field.
    pub issues: Vec<Issue>,
}

impl Report {
    /// Anything the person should be told.
    pub fn is_clean(&self) -> bool {
        self.dropped == 0 && self.broken == 0 && self.issues.is_empty()
    }

    pub(crate) fn note(&mut self, field: &str, kind: IssueKind, example: Option<String>) {
        if let Some(i) = self
            .issues
            .iter_mut()
            .find(|i| i.field == field && i.kind == kind)
        {
            i.count += 1;
        } else {
            self.issues.push(Issue {
                field: field.to_string(),
                kind,
                count: 1,
                example,
            });
        }
    }
}

/// A value as written, cut to a length that fits a message.
pub(crate) fn shorten(text: &str) -> String {
    const MAX: usize = 24;
    if text.chars().count() <= MAX {
        text.to_string()
    } else {
        let mut s: String = text.chars().take(MAX).collect();
        s.push('…');
        s
    }
}
