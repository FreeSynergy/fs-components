//! Calendar date: year, month, day (RFC 3339 `full-date`).
//!
//! Pattern: Strategy (one [`BasicField`] implementation). The joined form is `YYYY-MM-DD`, the
//! stored one. Showing a date in the reader's order and language (`10.11.2026`, `Nov 10, 2026`)
//! is the job of the words crate (per-language date formatting is Unsupported there); until then
//! every component shows the ISO form, which is unambiguous.

use fs_components_styles::Glyph;

use super::email::padded;
use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The date type.
#[derive(Debug, Clone, Copy)]
pub struct Date;

const SEGMENTS: [Segment; 3] = [
    Segment {
        key: "year",
        weight: 3,
        required: true,
    },
    Segment {
        key: "month",
        weight: 2,
        required: true,
    },
    Segment {
        key: "day",
        weight: 2,
        required: true,
    },
];

fn number(text: &str) -> Result<Option<u32>, Problem> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(None);
    }
    if t.len() > 4 || !t.chars().all(|c| c.is_ascii_digit()) {
        return Err(Problem::NotANumber);
    }
    Ok(t.parse().ok())
}

const fn is_leap(y: u32) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

const fn days_in(y: u32, m: u32) -> u32 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl BasicField for Date {
    fn key(&self) -> &'static str {
        "date"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Calendar
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn joiner(&self, after: usize) -> Option<&'static str> {
        (after < 2).then_some("-")
    }

    fn settle(&self, parts: &[String]) -> Vec<String> {
        let parts = padded(parts, 3);
        let pasted: Vec<&str> = parts[0].trim().split('-').collect();
        if let [y, m, d] = pasted
            .as_slice()
            .iter()
            .map(|p| p.trim())
            .collect::<Vec<_>>()
            .as_slice()
        {
            if y.is_empty() || m.is_empty() || d.is_empty() {
                return parts;
            }
            return vec![(*y).to_string(), (*m).to_string(), (*d).to_string()];
        }
        parts
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        let parts = padded(parts, 3);
        if is_blank(&parts) {
            return None;
        }
        let mut n = [0u32; 3];
        for (slot, part) in n.iter_mut().zip(&parts) {
            match number(part) {
                Err(p) => return Some(p),
                Ok(None) => return Some(Problem::Incomplete),
                Ok(Some(v)) => *slot = v,
            }
        }
        let [y, m, d] = n;
        let ok = (1..=9999).contains(&y) && (1..=12).contains(&m) && d >= 1 && d <= days_in(y, m);
        (!ok).then_some(Problem::NoSuchDate)
    }

    fn split(&self, text: &str) -> Vec<String> {
        self.settle(&[text.trim().to_string()])
    }

    fn join(&self, parts: &[String]) -> String {
        let parts = padded(parts, 3);
        if is_blank(&parts) {
            return String::new();
        }
        let (y, m, d) = (parts[0].trim(), parts[1].trim(), parts[2].trim());
        format!("{y:0>4}-{m:0>2}-{d:0>2}")
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: &str, m: &str, day: &str) -> Vec<String> {
        vec![y.into(), m.into(), day.into()]
    }

    #[test]
    fn calendar_rules() {
        assert_eq!(Date.problem(&d("", "", "")), None);
        assert_eq!(Date.problem(&d("2026", "11", "10")), None);
        assert_eq!(
            Date.problem(&d("2026", "11", "")),
            Some(Problem::Incomplete)
        );
        assert_eq!(
            Date.problem(&d("2026", "1x", "10")),
            Some(Problem::NotANumber)
        );
        assert_eq!(
            Date.problem(&d("2026", "02", "29")),
            Some(Problem::NoSuchDate)
        );
        assert_eq!(Date.problem(&d("2028", "02", "29")), None);
        assert_eq!(
            Date.problem(&d("1900", "02", "29")),
            Some(Problem::NoSuchDate)
        );
        assert_eq!(Date.problem(&d("2000", "02", "29")), None);
        assert_eq!(
            Date.problem(&d("2026", "13", "01")),
            Some(Problem::NoSuchDate)
        );
    }

    #[test]
    fn joined_form_is_padded_iso_and_a_pasted_date_is_spread() {
        assert_eq!(Date.join(&d("2026", "3", "9")), "2026-03-09");
        assert_eq!(Date.join(&d("", "", "")), "");
        assert_eq!(Date.settle(&d("2026-11-10", "", "")), d("2026", "11", "10"));
        assert_eq!(Date.split("2026-11-10"), d("2026", "11", "10"));
    }
}
