//! Postal address: street, postcode, locality, region, country (JSContact `Address` components).
//!
//! Pattern: Strategy (one [`BasicField`] implementation). Why: JSContact keeps the components
//! apart and leaves their order to the country (`defaultSeparator`, `isOrdered`); this type uses
//! the order street, postcode locality, region, country and says so: country-specific order
//! (Japan, Hungary) is Unsupported until the address `format` is read. No lookup of a postcode.

use fs_components_styles::Glyph;

use super::email::padded;
use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The address type.
#[derive(Debug, Clone, Copy)]
pub struct Address;

const SEGMENTS: [Segment; 5] = [
    Segment {
        key: "street",
        weight: 6,
        required: false,
    },
    Segment {
        key: "postcode",
        weight: 2,
        required: false,
    },
    Segment {
        key: "locality",
        weight: 4,
        required: false,
    },
    Segment {
        key: "region",
        weight: 3,
        required: false,
    },
    Segment {
        key: "country",
        weight: 4,
        required: false,
    },
];

fn at(parts: &[String], i: usize) -> &str {
    parts.get(i).map_or("", |p| p.trim())
}

fn joined(items: &[&str], sep: &str) -> String {
    items
        .iter()
        .copied()
        .filter(|i| !i.is_empty())
        .collect::<Vec<_>>()
        .join(sep)
}

fn place(parts: &[String]) -> String {
    joined(&[at(parts, 1), at(parts, 2)], " ")
}

impl BasicField for Address {
    fn key(&self) -> &'static str {
        "address"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Place
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        let parts = padded(parts, 5);
        if is_blank(&parts) {
            return None;
        }
        let needs_place = !at(&parts, 0).is_empty() || !at(&parts, 1).is_empty();
        (needs_place && at(&parts, 2).is_empty()).then_some(Problem::Incomplete)
    }

    fn split(&self, text: &str) -> Vec<String> {
        let lines: Vec<&str> = text
            .split([',', '\n'])
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let mut v = vec![String::new(); 5];
        let (street, mid, country) = match lines.as_slice() {
            [] => return v,
            [one] => (*one, "", ""),
            [a, b] => (*a, *b, ""),
            [a, rest @ .., z] => (*a, rest.first().copied().unwrap_or(""), *z),
        };
        if let [_, _, region @ .., _] = lines.as_slice() {
            v[3] = region.join(", ");
        }
        v[0] = street.to_string();
        match mid.split_once(' ') {
            Some((code, city)) if code.chars().next().is_some_and(|c| c.is_ascii_digit()) => {
                v[1] = code.to_string();
                v[2] = city.trim().to_string();
            }
            _ => v[2] = mid.to_string(),
        }
        v[4] = country.to_string();
        v
    }

    fn join(&self, parts: &[String]) -> String {
        joined(
            &[at(parts, 0), &place(parts), at(parts, 3), at(parts, 4)],
            ", ",
        )
    }

    fn compact(&self, parts: &[String]) -> String {
        joined(&[at(parts, 0), &place(parts)], ", ")
    }

    fn cell(&self, parts: &[String]) -> String {
        let city = joined(&[at(parts, 2), at(parts, 4)], ", ");
        if city.is_empty() {
            self.compact(parts)
        } else {
            city
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(street: &str, code: &str, city: &str, region: &str, country: &str) -> Vec<String> {
        [street, code, city, region, country]
            .iter()
            .map(|x| (*x).to_string())
            .collect()
    }

    #[test]
    fn a_street_or_postcode_needs_a_locality() {
        assert_eq!(Address.problem(&a("", "", "", "", "")), None);
        assert_eq!(
            Address.problem(&a("Main St 1", "", "", "", "")),
            Some(Problem::Incomplete)
        );
        assert_eq!(
            Address.problem(&a("", "10115", "", "", "")),
            Some(Problem::Incomplete)
        );
        assert_eq!(Address.problem(&a("", "", "", "", "Germany")), None);
        assert_eq!(
            Address.problem(&a("Main St 1", "10115", "Berlin", "", "Germany")),
            None
        );
    }

    #[test]
    fn forms() {
        let v = a("Main St 1", "10115", "Berlin", "BE", "Germany");
        assert_eq!(Address.join(&v), "Main St 1, 10115 Berlin, BE, Germany");
        assert_eq!(Address.compact(&v), "Main St 1, 10115 Berlin");
        assert_eq!(Address.cell(&v), "Berlin, Germany");
        assert_eq!(Address.cell(&a("Main St 1", "", "", "", "")), "Main St 1");
    }

    #[test]
    fn split_reads_the_usual_three_lines() {
        assert_eq!(
            Address.split("Main St 1, 10115 Berlin, Germany"),
            a("Main St 1", "10115", "Berlin", "", "Germany")
        );
        assert_eq!(Address.split("Berlin"), a("Berlin", "", "", "", ""));
        let full = a("Main St 1", "10115", "Berlin", "BE", "Germany");
        assert_eq!(
            Address.split(&Address.join(&full)),
            full,
            "split undoes join"
        );
    }
}
