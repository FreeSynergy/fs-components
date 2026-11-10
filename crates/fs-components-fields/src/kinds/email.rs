//! E-mail address: local part and domain.
//!
//! Pattern: Strategy (one [`BasicField`] implementation); the rule is `EmailAddress` of the contract
//! crate, the type only splits the text so that the person sees two parts and the `@` is a fixed
//! sign between them. A pasted whole address, or an `@` typed into the first part, is split by
//! [`BasicField::settle`], so the keyboard never has to produce the sign.

use fs_components_styles::Glyph;
use fs_interfaces::values::{ContactValue, EmailAddress};

use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The e-mail type.
#[derive(Debug, Clone, Copy)]
pub struct Email;

const SEGMENTS: [Segment; 2] = [
    Segment {
        key: "local",
        weight: 5,
        required: true,
    },
    Segment {
        key: "domain",
        weight: 4,
        required: true,
    },
];

impl BasicField for Email {
    fn key(&self) -> &'static str {
        "email"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Mail
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn joiner(&self, after: usize) -> Option<&'static str> {
        (after == 0).then_some("@")
    }

    fn settle(&self, parts: &[String]) -> Vec<String> {
        let mut parts = padded(parts, 2);
        if let Some((local, rest)) = parts[0].clone().split_once('@') {
            let domain = rest.trim_start_matches('@');
            parts[0] = local.to_string();
            parts[1] = domain.to_string();
        }
        // no domain holds an `@`; the one just typed may arrive here too (see FieldInput)
        parts[1] = parts[1].replace('@', "");
        parts
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        EmailAddress::check(&self.join(parts))
            .err()
            .map(Problem::Contact)
    }

    fn split(&self, text: &str) -> Vec<String> {
        match text.trim().rsplit_once('@') {
            Some((local, domain)) => vec![local.to_string(), domain.to_string()],
            None => vec![text.trim().to_string(), String::new()],
        }
    }

    fn join(&self, parts: &[String]) -> String {
        if is_blank(parts) {
            return String::new();
        }
        let parts = self.settle(parts);
        format!("{}@{}", parts[0].trim(), parts[1].trim())
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }

    fn cell(&self, parts: &[String]) -> String {
        self.join(parts).to_lowercase()
    }
}

/// `parts` with at least `n` entries.
pub(super) fn padded(parts: &[String], n: usize) -> Vec<String> {
    let mut v = parts.to_vec();
    v.resize(v.len().max(n), String::new());
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_interfaces::values::ValueProblem;

    fn p(a: &str, b: &str) -> Vec<String> {
        vec![a.into(), b.into()]
    }

    #[test]
    fn empty_is_no_problem_and_the_rule_is_the_contracts() {
        assert_eq!(Email.problem(&p("", "")), None);
        assert_eq!(
            Email.problem(&p("ada", "")),
            Some(Problem::Contact(ValueProblem::EmptyDomain))
        );
        assert_eq!(
            Email.problem(&p("ada", "example")),
            Some(Problem::Contact(ValueProblem::DomainWithoutDot))
        );
        assert_eq!(Email.problem(&p("ada", "example.org")), None);
    }

    #[test]
    fn an_at_typed_into_the_first_part_moves_the_rest_to_the_domain() {
        assert_eq!(Email.settle(&p("ada@exam", "")), p("ada", "exam"));
        assert_eq!(
            Email.settle(&p("ada@example.org", "old.net")),
            p("ada", "example.org")
        );
        assert_eq!(Email.settle(&p("ada", "x.org")), p("ada", "x.org"));
    }

    #[test]
    fn split_and_join_round_trip() {
        assert_eq!(Email.split(" ada@example.org "), p("ada", "example.org"));
        assert_eq!(Email.join(&p("ada", "example.org")), "ada@example.org");
        assert_eq!(Email.join(&p(" ", "")), "");
        assert_eq!(Email.cell(&p("Ada", "Example.ORG")), "ada@example.org");
    }
}
