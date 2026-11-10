//! Phone number: country code and number.
//!
//! Pattern: Strategy (one [`BasicField`] implementation); the rule is `PhoneNumber` (E.164) of the
//! contract crate. The country code is its own part so that the person is asked for it; the stored
//! form is the plain `+<digits>`. A number pasted whole is split at its first space when it starts
//! with `+`; without one the digits stay in the number part (no country is guessed; splitting by
//! known country codes is Unsupported).

use fs_components_styles::Glyph;
use fs_interfaces::values::{ContactValue, PhoneNumber};

use super::email::padded;
use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The phone type.
#[derive(Debug, Clone, Copy)]
pub struct Phone;

const SEGMENTS: [Segment; 2] = [
    Segment {
        key: "country",
        weight: 2,
        required: true,
    },
    Segment {
        key: "number",
        weight: 6,
        required: true,
    },
];

/// The two parts as one typed text: `+49 30 1234567`. A country code typed without `+` gets one.
fn typed(parts: &[String]) -> String {
    let parts = padded(parts, 2);
    let (country, number) = (parts[0].trim(), parts[1].trim());
    if country.is_empty() && number.is_empty() {
        return String::new();
    }
    let country = if country.is_empty() || country.starts_with('+') {
        country.to_string()
    } else {
        format!("+{country}")
    };
    format!("{country} {number}").trim().to_string()
}

impl BasicField for Phone {
    fn key(&self) -> &'static str {
        "phone"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Phone
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        PhoneNumber::check(&typed(parts))
            .err()
            .map(Problem::Contact)
    }

    fn split(&self, text: &str) -> Vec<String> {
        let text = text.trim();
        match text.split_once(char::is_whitespace) {
            Some((country, number)) if country.starts_with('+') => {
                vec![country.to_string(), number.trim().to_string()]
            }
            _ => vec![String::new(), text.to_string()],
        }
    }

    fn join(&self, parts: &[String]) -> String {
        let text = typed(parts);
        PhoneNumber::parse(&text).map_or(text, |p| p.as_str().to_string())
    }

    fn compact(&self, parts: &[String]) -> String {
        typed(parts)
    }

    fn cell(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_interfaces::values::ValueProblem;

    fn p(a: &str, b: &str) -> Vec<String> {
        vec![a.into(), b.into()]
    }

    #[test]
    fn rule_is_the_contracts_and_a_missing_plus_is_added() {
        assert_eq!(Phone.problem(&p("", "")), None);
        assert_eq!(
            Phone.problem(&p("", "030 123")),
            Some(Problem::Contact(ValueProblem::NoCountryCode))
        );
        assert_eq!(Phone.problem(&p("49", "30 1234567")), None);
        assert_eq!(
            Phone.problem(&p("+49", "30 12")),
            Some(Problem::Contact(ValueProblem::PhoneLength))
        );
    }

    #[test]
    fn forms() {
        let ok = p("49", "30 1234567");
        assert_eq!(Phone.compact(&ok), "+49 30 1234567");
        assert_eq!(Phone.join(&ok), "+49301234567");
        assert_eq!(Phone.split("+49 30 1234567"), p("+49", "30 1234567"));
        assert_eq!(Phone.split("0301234"), p("", "0301234"));
    }
}
