//! From spec rows to the data a mark draws.
//!
//! Pattern: none (pure functions: read, aggregate, drop-and-count; `Datum` and `Prepared` are plain data). The policy follows Vega-Lite's default
//! `invalid` mode: a bar, point or arc with an invalid position is left out; a line or area is
//! *broken* there. A category that is `null` is a category. Nothing is dropped without a count.

use super::aggregate::group;
use super::cell::{Cell, Invalid, Zones, read};
use super::report::{IssueKind, Report, shorten};
use super::time::Zone;
use crate::spec::{Channel, ChannelDef, Spec, SpecError, lookup};

/// One mark's worth of values, indexed by [`Channel`] (`x, y, color, theta`).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Datum {
    /// The cells; `Null` for a channel the spec does not use.
    pub cells: [Cell; 4],
    /// Index of the first source row.
    pub row: usize,
}

/// The data of a chart, ready for scales.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Prepared {
    /// What is drawn, in order.
    pub data: Vec<Datum>,
    /// What was dropped, broken or odd.
    pub report: Report,
    /// The zone of each temporal channel.
    pub zones: [Zone; 4],
}

impl Channel {
    /// Position of the channel in [`Datum::cells`].
    pub fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Color => 2,
            Self::Theta => 3,
        }
    }
}

const ALL: [Channel; 4] = [Channel::X, Channel::Y, Channel::Color, Channel::Theta];

/// Read, aggregate and clean the rows of `spec`.
///
/// # Errors
/// [`SpecError`] only for a temporal field whose texts mix times with and without an offset: the
/// chart cannot say where those lie on one line without guessing a zone.
pub fn prepare(spec: &Spec) -> Result<Prepared, SpecError> {
    let enc = &spec.encoding;
    let mut report = Report {
        rows: spec.data.len(),
        ..Report::default()
    };
    let mut zones = [Zone::Naive; 4];
    let mut data: Vec<Datum> = (0..spec.data.len())
        .map(|row| Datum {
            cells: [Cell::Null, Cell::Null, Cell::Null, Cell::Null],
            row,
        })
        .collect();
    for ch in ALL {
        let Some(def) = enc.get(ch) else { continue };
        let Some(field) = def.field.as_deref() else {
            continue;
        };
        let mut z = Zones::default();
        for (d, row) in data.iter_mut().zip(&spec.data) {
            let raw = lookup(row, field);
            let cell = read(raw, def.ftype, &mut z);
            note(&mut report, field, def, &cell, raw);
            d.cells[ch.index()] = cell;
        }
        if z.mixed {
            return Err(SpecError::invalid(
                &format!("/encoding/{}/field", ch.name()),
                format!(
                    "field \"{field}\" mixes times with and without a UTC offset; the chart will not guess a zone"
                ),
            ));
        }
        zones[ch.index()] = z.axis();
    }
    if ALL
        .iter()
        .any(|&c| enc.get(c).is_some_and(|d| d.aggregate.is_some()))
    {
        data = group(enc, data);
    }
    clean(spec, &mut data, &mut report);
    report.kept = data.len();
    Ok(Prepared {
        data,
        report,
        zones,
    })
}

fn note(
    report: &mut Report,
    field: &str,
    def: &ChannelDef,
    cell: &Cell,
    raw: Option<&serde_json::Value>,
) {
    let example = || raw.map(|v| shorten(&v.to_string()));
    match cell {
        Cell::Null | Cell::Absent if def.ftype.is_discrete() => {}
        Cell::Null | Cell::Absent => report.note(field, IssueKind::Null, None),
        Cell::Invalid(i) => report.note(field, IssueKind::Invalid(*i), example()),
        _ => {}
    }
}

/// Drop (or break) what cannot be drawn, counting it.
fn clean(spec: &Spec, data: &mut Vec<Datum>, report: &mut Report) {
    let enc = &spec.encoding;
    let name = |c: Channel| {
        enc.get(c)
            .and_then(|d| d.field.clone())
            .unwrap_or_else(|| "count".into())
    };
    for d in data.iter_mut() {
        if d.cells[3].num().is_some_and(|v| v < 0.0) {
            d.cells[3] = Cell::Invalid(Invalid::Negative);
            report.note(
                &name(Channel::Theta),
                IssueKind::Invalid(Invalid::Negative),
                None,
            );
        }
        for c in ALL {
            if matches!(d.cells[c.index()], Cell::Invalid(Invalid::Overflow)) {
                report.note(&name(c), IssueKind::Invalid(Invalid::Overflow), None);
            }
        }
    }
    let continuous = |c: Channel| enc.get(c).is_some_and(|d| !d.ftype.is_discrete());
    let unusable = |d: &Datum, c: Channel| {
        let cell = &d.cells[c.index()];
        if continuous(c) {
            cell.is_missing()
        } else {
            matches!(cell, Cell::Invalid(_))
        }
    };
    let path = spec.mark.kind.is_path();
    let before = data.len();
    data.retain(|d| {
        if unusable(d, Channel::Color)
            || [Channel::X, Channel::Y]
                .iter()
                .any(|&c| !continuous(c) && unusable(d, c))
        {
            return false;
        }
        let soft = [Channel::X, Channel::Y, Channel::Theta]
            .iter()
            .any(|&c| unusable(d, c));
        if soft && path {
            report.broken += 1;
        }
        !soft || path
    });
    report.dropped = before - data.len();
}
