//! The question "suggest" (command `vorschlagen` of the type description) as a trait.
//!
//! Pattern: Strategy behind the trait [`Suggest`] — the program answers from its data (domains of
//! the contacts it holds); [`SuggestFromList`] is a plain implementation for tests and for
//! programs that keep a list. The input only asks and shows; it decides nothing.

use crate::field::BasicField;

/// Something that can complete a part of a value.
pub trait Suggest: Send + Sync {
    /// Candidates for the part `segment` of `field`, given what has been typed so far in all parts.
    /// Best first; at most `limit`. Empty if there is nothing to say.
    fn suggest(
        &self,
        field: &dyn BasicField,
        segment: usize,
        parts: &[String],
        limit: usize,
    ) -> Vec<String>;
}

/// Suggestions from a list of values for one segment: those that start with what is typed
/// (compared without case), the most often seen first, an exact match left out.
#[derive(Debug, Clone, Default)]
pub struct SuggestFromList {
    /// The key of the type this list is for (`email`).
    pub field: &'static str,
    /// The index of the segment it completes (`1`, the domain).
    pub segment: usize,
    /// Known values, repeated as often as they were seen.
    pub seen: Vec<String>,
}

impl Suggest for SuggestFromList {
    fn suggest(
        &self,
        field: &dyn BasicField,
        segment: usize,
        parts: &[String],
        limit: usize,
    ) -> Vec<String> {
        if field.key() != self.field || segment != self.segment {
            return Vec::new();
        }
        let typed = parts.get(segment).map_or("", |p| p.trim()).to_lowercase();
        let mut counted: Vec<(String, usize)> = Vec::new();
        for value in &self.seen {
            let lower = value.to_lowercase();
            if !lower.starts_with(&typed) || lower == typed {
                continue;
            }
            match counted.iter_mut().find(|(v, _)| *v == lower) {
                Some((_, n)) => *n += 1,
                None => counted.push((lower, 1)),
            }
        }
        counted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counted.into_iter().take(limit).map(|(v, _)| v).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;

    fn list() -> SuggestFromList {
        let seen = [
            "example.org",
            "example.org",
            "example.net",
            "mail.test",
            "Example.ORG",
        ];
        SuggestFromList {
            field: "email",
            segment: 1,
            seen: seen.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn most_seen_first_and_prefix_only() {
        let parts = vec!["ada".to_string(), "ex".to_string()];
        let got = list().suggest(Kind::Email.field(), 1, &parts, 5);
        assert_eq!(got, ["example.org", "example.net"]);
    }

    #[test]
    fn an_empty_domain_offers_everything_up_to_the_limit() {
        let parts = vec!["ada".to_string(), String::new()];
        assert_eq!(
            list().suggest(Kind::Email.field(), 1, &parts, 2),
            ["example.org", "example.net"]
        );
    }

    #[test]
    fn exact_match_other_segment_and_other_type_give_nothing() {
        let l = list();
        let done = vec!["ada".to_string(), "mail.test".to_string()];
        assert_eq!(
            l.suggest(Kind::Email.field(), 1, &done, 5),
            Vec::<String>::new()
        );
        assert_eq!(
            l.suggest(Kind::Email.field(), 0, &done, 5),
            Vec::<String>::new()
        );
        assert_eq!(
            l.suggest(Kind::Phone.field(), 1, &done, 5),
            Vec::<String>::new()
        );
    }
}
