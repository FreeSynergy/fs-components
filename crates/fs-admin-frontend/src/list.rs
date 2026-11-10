//! The list: columns from the description, rows from the records, sorting, filtering, selection.
//!
//! Pattern: State — the model holds records, sort, filter and selection and their transitions (the filter is a plain
//! predicate; a combinable Specification is not needed yet). The view function over it is pure: the
//! rows are computed from it; the cell component draws them. No Blinc here.

use fs_types::{BasicType, FieldDescription, TypeDescription};

use crate::backend::Record;

/// Most columns a list shows; the rest are in the detail.
pub const MAX_COLUMNS: usize = 6;

/// One column: a field of the type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// The field.
    pub field: FieldDescription,
}

/// Sort direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// A to Z.
    Ascending,
    /// Z to A.
    Descending,
}

/// A basic type a cell can show in one line.
fn fits_a_cell(basic: BasicType) -> bool {
    !matches!(basic, BasicType::MultilineText | BasicType::Image)
}

/// The columns of a type: fields that fit a cell, required ones first when there are too many, in description order.
pub fn columns(description: &TypeDescription, max: usize) -> Vec<Column> {
    let fit: Vec<&FieldDescription> = description
        .fields
        .iter()
        .filter(|f| fits_a_cell(f.basic))
        .collect();
    let mut keep: Vec<&FieldDescription> = fit.clone();
    if keep.len() > max {
        keep.sort_by_key(|f| !f.required); // stable: description order inside each group
        keep.truncate(max);
    }
    fit.into_iter()
        .filter(|f| keep.iter().any(|k| k.name == f.name))
        .map(|f| Column { field: f.clone() })
        .collect()
}

/// The first value of a field of a record and how many more there are.
fn first(record: &Record, field: &str) -> (String, usize) {
    let v = record.values.get(field).map_or(&[][..], Vec::as_slice);
    let mut given = v.iter().filter(|t| !t.trim().is_empty());
    let one = given.next().cloned().unwrap_or_default();
    (one, given.count())
}

/// The records of a type as a list.
#[derive(Clone, Debug)]
pub struct ListModel {
    columns: Vec<Column>,
    records: Vec<Record>,
    sort: Option<(usize, Direction)>,
    filter: String,
    selected: Option<String>,
}

impl ListModel {
    /// A list of `records`, with the columns of the description.
    pub fn new(description: &TypeDescription, records: Vec<Record>) -> Self {
        Self {
            columns: columns(description, MAX_COLUMNS),
            records,
            sort: None,
            filter: String::new(),
            selected: None,
        }
    }

    /// The columns, left to right.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Replace the records (after a command); the selection stays if its record is still there.
    pub fn set_records(&mut self, records: Vec<Record>) {
        self.records = records;
        if let Some(id) = &self.selected
            && !self.records.iter().any(|r| &r.id == id)
        {
            self.selected = None;
        }
    }

    /// All records, in the program's order.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// The record with this id.
    pub fn record(&self, id: &str) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Show only records with this text in any value (without regard to case); empty shows all.
    pub fn set_filter(&mut self, text: &str) {
        text.trim().clone_into(&mut self.filter);
    }

    /// The filter text.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// A click on a column head: ascending, then descending, then the program's order. Returns `false` for a column
    /// that is not there.
    pub fn cycle_sort(&mut self, column: usize) -> bool {
        if column >= self.columns.len() {
            return false;
        }
        self.sort = match self.sort {
            Some((c, Direction::Ascending)) if c == column => Some((c, Direction::Descending)),
            Some((c, Direction::Descending)) if c == column => None,
            _ => Some((column, Direction::Ascending)),
        };
        true
    }

    /// The sort now.
    pub fn sort(&self) -> Option<(usize, Direction)> {
        self.sort
    }

    /// Select a record. Returns `false` and changes nothing if there is none with this id.
    pub fn select(&mut self, id: &str) -> bool {
        let there = self.records.iter().any(|r| r.id == id);
        if there {
            self.selected = Some(id.to_owned());
        }
        there
    }

    /// Select nothing.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// The selected id.
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// The records to show: filtered, then sorted (stable; records with no value in the column last).
    pub fn rows(&self) -> Vec<&Record> {
        let needle = self.filter.to_lowercase();
        let mut rows: Vec<&Record> = self
            .records
            .iter()
            .filter(|r| {
                needle.is_empty()
                    || r.values
                        .values()
                        .flatten()
                        .any(|t| t.to_lowercase().contains(&needle))
            })
            .collect();
        if let Some((c, dir)) = self.sort {
            let name = &self.columns[c].field.name;
            rows.sort_by(|a, b| {
                let (ka, kb) = (
                    first(a, name).0.to_lowercase(),
                    first(b, name).0.to_lowercase(),
                );
                match (ka.is_empty(), kb.is_empty()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    (false, false) if dir == Direction::Ascending => ka.cmp(&kb),
                    (false, false) => kb.cmp(&ka),
                }
            });
        }
        rows
    }
}
