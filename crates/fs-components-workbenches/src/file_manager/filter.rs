//! The filter by size and date, as it is typed.
//!
//! Pattern: State — what is typed stays text until Apply; a size or a date that
//! cannot be read is said, and the list stays as it was. Sizes take a plain number or KB, MB, GB (powers
//! of 1000; what `format_bytes` writes is powers of 1024 and also reads as KiB, MiB, GiB).

use super::error::FileProblem;
use super::labels::FileLabels;
use super::model::ListFilter;
use super::side::Side;
use super::state::{FilesState, Notice};

/// The fields of the filter as typed.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterDraft {
    /// At least (size).
    pub min_size: String,
    /// At most (size).
    pub max_size: String,
    /// Changed on or after (date).
    pub modified_from: String,
    /// Changed before (date).
    pub modified_before: String,
}

/// A size as typed; `None` if it is not one.
pub fn parse_bytes(text: &str) -> Option<u64> {
    let text = text.trim();
    let split = text
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let number: f64 = number.parse().ok()?;
    let factor: f64 = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "kb" => 1e3,
        "mb" => 1e6,
        "gb" => 1e9,
        "tb" => 1e12,
        "kib" => 1024.0,
        "mib" => 1024.0 * 1024.0,
        "gib" => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    let bytes = number * factor;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    (bytes.is_finite() && bytes < u64::MAX as f64).then_some(bytes as u64)
}

/// Is this a date `YYYY-MM-DD` that can be on a calendar?
pub fn is_day(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    let [year, month, day] = parts.as_slice() else {
        return false;
    };
    let number = |t: &&str, length: usize| {
        (t.len() == length && t.bytes().all(|b| b.is_ascii_digit()))
            .then(|| t.parse::<u32>().ok())
            .flatten()
    };
    let (Some(year), Some(month), Some(day)) = (number(year, 4), number(month, 2), number(day, 2))
    else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let last = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=last).contains(&day)
}

impl FilesState {
    /// Show the filter beside the list.
    pub fn open_filter(&mut self) {
        if self.capabilities.search {
            self.side = Side::Filter;
            self.detail_open = true;
        }
    }

    /// The filter as typed.
    pub fn filter_draft(&self) -> &FilterDraft {
        &self.filter_draft
    }

    /// The person typed into a field of the filter (`min`, `max`, `from`, `before`).
    pub fn set_filter_field(&mut self, field: &str, text: &str) {
        let draft = &mut self.filter_draft;
        let slot = match field {
            "min" => &mut draft.min_size,
            "max" => &mut draft.max_size,
            "from" => &mut draft.modified_from,
            "before" => &mut draft.modified_before,
            _ => return,
        };
        text.clone_into(slot);
    }

    /// Apply what is typed. What cannot be read is said, and the list stays.
    pub fn apply_filter(&mut self) {
        match self.parsed_filter() {
            Ok(filter) => {
                self.filter = filter;
                self.notice = None;
                self.filter_changed();
            }
            Err(text) => {
                self.notice = Some(Notice::Problem(FileProblem::Failed, text));
            }
        }
    }

    /// No filter.
    pub fn clear_filter(&mut self) {
        self.filter_draft = FilterDraft::default();
        self.filter = ListFilter::default();
        self.notice = None;
        self.filter_changed();
    }

    fn parsed_filter(&self) -> Result<ListFilter, String> {
        let draft = &self.filter_draft;
        let size = |text: &str| -> Result<Option<u64>, String> {
            if text.trim().is_empty() {
                return Ok(None);
            }
            parse_bytes(text)
                .map(Some)
                .ok_or_else(|| FileLabels::fill(&self.labels.bad_size, text))
        };
        let day = |text: &str| -> Result<Option<String>, String> {
            let text = text.trim();
            if text.is_empty() {
                Ok(None)
            } else if is_day(text) {
                Ok(Some(text.to_string()))
            } else {
                Err(FileLabels::fill(&self.labels.bad_date, text))
            }
        };
        Ok(ListFilter {
            min_size: size(&draft.min_size)?,
            max_size: size(&draft.max_size)?,
            modified_from: day(&draft.modified_from)?,
            modified_before: day(&draft.modified_before)?,
        })
    }
}
