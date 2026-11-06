//! The words around a chart: title, and the notes about data that could not be drawn.
//!
//! Pattern: Template over the `Report`. Nothing is dropped silently: every dropped, broken or
//! odd value is counted in a sentence under the chart, in the caller's words.

use super::axis_items::fit;
use super::measure::Measure;
use super::measure::Metrics;
use crate::data::{Invalid, IssueKind, Report};
use crate::labels::Labels;

/// How many sentences at most; more are summed up by the first.
const MAX_LINES: usize = 4;

/// The sentences for a report, each at most `limit` pixels wide.
pub fn notes(r: &Report, l: &Labels, m: &dyn Measure, k: &Metrics, limit: f32) -> Vec<String> {
    let mut lines = report_lines(r, l);
    lines.truncate(MAX_LINES);
    lines
        .iter()
        .map(|s| fit(s, limit, k.label, false, m))
        .collect()
}

/// All the sentences for a report, not cut.
pub fn report_lines(r: &Report, l: &Labels) -> Vec<String> {
    let mut lines = Vec::new();
    if r.dropped > 0 || r.broken > 0 {
        lines.push(Labels::fill(
            &l.note_left_out,
            &[
                ("dropped", &r.dropped.to_string()),
                ("rows", &r.rows.to_string()),
                ("broken", &r.broken.to_string()),
            ],
        ));
    }
    for i in &r.issues {
        let why = match i.kind {
            IssueKind::Null => &l.why_null,
            IssueKind::Invalid(Invalid::NotANumber) => &l.why_not_number,
            IssueKind::Invalid(Invalid::NotADate) => &l.why_not_date,
            IssueKind::Invalid(Invalid::Overflow) => &l.why_overflow,
            IssueKind::Invalid(Invalid::Negative) => &l.why_negative,
            _ => &l.why_wrong_kind,
        };
        let mut line = Labels::fill(
            &l.note_issue,
            &[
                ("count", &i.count.to_string()),
                ("field", &i.field),
                ("why", why),
            ],
        );
        if let Some(e) = &i.example {
            line = format!("{line} ({e})");
        }
        lines.push(line);
    }
    lines
}
