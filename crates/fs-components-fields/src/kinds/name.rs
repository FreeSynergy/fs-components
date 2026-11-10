//! Name of a person: title, given, additional, surname, generation (JSContact `Name` kinds).
//!
//! Pattern: Strategy (one [`BasicField`] implementation). Why: the parts are what JSContact keeps
//! apart (`title`, `given`, `given2`, `surname`, `generation`), the joined forms are the ones a
//! person reads. Name order and sort rules differ by culture; this type uses the Western order
//! and says so: other orders are Unsupported until `Name.isOrdered` and `defaultSeparator` are read.

use fs_components_styles::Glyph;

use super::email::padded;
use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The name type.
#[derive(Debug, Clone, Copy)]
pub struct Name;

const SEGMENTS: [Segment; 5] = [
    Segment {
        key: "title",
        weight: 2,
        required: false,
    },
    Segment {
        key: "given",
        weight: 4,
        required: false,
    },
    Segment {
        key: "given2",
        weight: 3,
        required: false,
    },
    Segment {
        key: "surname",
        weight: 4,
        required: false,
    },
    Segment {
        key: "generation",
        weight: 2,
        required: false,
    },
];

fn words(parts: &[String], which: &[usize]) -> String {
    which
        .iter()
        .filter_map(|i| parts.get(*i))
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

impl BasicField for Name {
    fn key(&self) -> &'static str {
        "name"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Person
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        let parts = padded(parts, 5);
        if is_blank(&parts) {
            return None;
        }
        (parts[1].trim().is_empty() && parts[3].trim().is_empty()).then_some(Problem::NoName)
    }

    fn split(&self, text: &str) -> Vec<String> {
        let text = text.trim();
        if let Some((surname, rest)) = text.split_once(',') {
            return vec![
                String::new(),
                rest.trim().to_string(),
                String::new(),
                surname.trim().to_string(),
                String::new(),
            ];
        }
        let w: Vec<&str> = text.split_whitespace().collect();
        let mut v = vec![String::new(); 5];
        match w.as_slice() {
            [] => {}
            [one] => v[1] = (*one).to_string(),
            [first, last] => {
                v[1] = (*first).to_string();
                v[3] = (*last).to_string();
            }
            [first, middle @ .., last] => {
                v[1] = (*first).to_string();
                v[2] = middle.join(" ");
                v[3] = (*last).to_string();
            }
        }
        v
    }

    fn join(&self, parts: &[String]) -> String {
        words(parts, &[0, 1, 2, 3, 4])
    }

    fn compact(&self, parts: &[String]) -> String {
        words(parts, &[1, 3])
    }

    fn cell(&self, parts: &[String]) -> String {
        let (given, surname) = (words(parts, &[1, 2]), words(parts, &[3]));
        match (surname.is_empty(), given.is_empty()) {
            (false, false) => format!("{surname}, {given}"),
            (false, true) => surname,
            _ => given,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(t: &str, g: &str, g2: &str, s: &str, gen_: &str) -> Vec<String> {
        [t, g, g2, s, gen_]
            .iter()
            .map(|x| (*x).to_string())
            .collect()
    }

    #[test]
    fn needs_a_given_name_or_a_surname_once_anything_is_given() {
        assert_eq!(Name.problem(&n("", "", "", "", "")), None);
        assert_eq!(
            Name.problem(&n("Dr.", "", "", "", "")),
            Some(Problem::NoName)
        );
        assert_eq!(Name.problem(&n("", "Ada", "", "", "")), None);
        assert_eq!(Name.problem(&n("", "", "", "Lovelace", "")), None);
    }

    #[test]
    fn forms_for_reading_and_for_lists() {
        let ada = n("Dr.", "Ada", "Augusta", "Lovelace", "");
        assert_eq!(Name.join(&ada), "Dr. Ada Augusta Lovelace");
        assert_eq!(Name.compact(&ada), "Ada Lovelace");
        assert_eq!(Name.cell(&ada), "Lovelace, Ada Augusta");
        assert_eq!(Name.cell(&n("", "Cher", "", "", "")), "Cher");
    }

    #[test]
    fn split_handles_comma_form_and_many_words() {
        assert_eq!(
            Name.split("Lovelace, Ada"),
            n("", "Ada", "", "Lovelace", "")
        );
        assert_eq!(
            Name.split("Ada Augusta King Lovelace"),
            n("", "Ada", "Augusta King", "Lovelace", "")
        );
        assert_eq!(Name.split("Ada"), n("", "Ada", "", "", ""));
        assert_eq!(Name.split("  "), n("", "", "", "", ""));
    }
}
