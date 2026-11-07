//! The text alternative of a chart: a summary and the data as a table.
//!
//! Pattern: none (pure functions from the drawn data to words; `Table` and `TextAlternative` are plain data). Charts must never be the only way to read
//! numbers (accessibility): the summary is the spec's own `description` when it has one, else
//! one made from the data; the table lists exactly what is drawn, then the notes list what is not.

use crate::data::{Datum, Prepared};
use crate::labels::Labels;
use crate::layout::describe;
use crate::spec::{Channel, FieldType, MarkKind, Spec};

/// A table of the drawn data.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    /// Column headings.
    pub headers: Vec<String>,
    /// Rows, as text.
    pub rows: Vec<Vec<String>>,
    /// How many more rows exist than are listed.
    pub omitted: usize,
}

/// Summary and table.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextAlternative {
    /// What the chart shows, in sentences.
    pub summary: String,
    /// The drawn data.
    pub table: Table,
    /// A sentence saying how many rows the table does not list (empty if it lists all).
    pub more: String,
}

impl TextAlternative {
    /// Everything as plain text: the summary, then one tab-separated line per row.
    pub fn to_text(&self) -> String {
        let mut out = self.summary.clone();
        out.push('\n');
        out.push_str(&self.table.headers.join("\t"));
        for row in &self.table.rows {
            out.push('\n');
            out.push_str(&row.join("\t"));
        }
        if !self.more.is_empty() {
            out.push('\n');
            out.push_str(&self.more);
        }
        out
    }
}

/// Build the text alternative; the table lists at most `max_rows` rows.
pub fn text_alternative(
    spec: &Spec,
    data: &Prepared,
    l: &Labels,
    max_rows: usize,
) -> TextAlternative {
    let channels = [Channel::X, Channel::Y, Channel::Theta, Channel::Color];
    let used: Vec<Channel> = channels
        .iter()
        .copied()
        .filter(|&c| spec.encoding.get(c).is_some())
        .collect();
    let headers = used
        .iter()
        .filter_map(|&c| spec.encoding.get(c))
        .map(|d| describe::heading(d, l))
        .collect();
    let row = |d: &Datum| -> Vec<String> {
        used.iter()
            .filter_map(|&c| {
                let def = spec.encoding.get(c)?;
                Some(describe::value(
                    &d.cells[c.index()],
                    def,
                    data.zones[c.index()],
                    l,
                ))
            })
            .collect()
    };
    let rows: Vec<Vec<String>> = data.data.iter().take(max_rows).map(row).collect();
    let table = Table {
        headers,
        omitted: data.data.len().saturating_sub(rows.len()),
        rows,
    };
    let more = if table.omitted > 0 {
        Labels::fill(&l.alt_more, &[("count", &table.omitted.to_string())])
    } else {
        String::new()
    };
    TextAlternative {
        summary: summary(spec, data, l),
        table,
        more,
    }
}

fn summary(spec: &Spec, data: &Prepared, l: &Labels) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(t) = &spec.title {
        parts.push(format!("{t}."));
    }
    if let Some(d) = &spec.description {
        parts.push(d.clone());
    } else {
        let kind = &l.kinds[match spec.mark.kind {
            MarkKind::Bar => 0,
            MarkKind::Line => 1,
            MarkKind::Area => 2,
            MarkKind::Point => 3,
            _ => 4,
        }];
        parts.push(Labels::fill(
            &l.alt_summary,
            &[("kind", kind), ("rows", &data.data.len().to_string())],
        ));
        for c in [Channel::X, Channel::Y, Channel::Theta] {
            let Some(def) = spec.encoding.get(c) else {
                continue;
            };
            if matches!(def.ftype, FieldType::Quantitative | FieldType::Temporal)
                && let Some((lo, hi)) = range(data, c)
            {
                {
                    let conv = |v: f64| {
                        describe::value(&crate::data::Cell::Num(v), def, data.zones[c.index()], l)
                    };
                    parts.push(Labels::fill(
                        &l.alt_range,
                        &[
                            ("field", &describe::heading(def, l)),
                            ("min", &conv(lo)),
                            ("max", &conv(hi)),
                        ],
                    ));
                }
            }
        }
    }
    for note in crate::layout::report_lines(&data.report, l) {
        parts.push(note);
    }
    parts.join(" ")
}

fn range(data: &Prepared, c: Channel) -> Option<(f64, f64)> {
    data.data
        .iter()
        .filter_map(|d| d.cells[c.index()].num())
        .fold(None, |r, v| {
            Some(r.map_or((v, v), |(a, b): (f64, f64)| (a.min(v), b.max(v))))
        })
}
