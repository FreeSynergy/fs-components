//! What a basic type is, as far as a component needs to know.
//!
//! Pattern: Strategy behind the trait [`BasicField`] — one implementation per basic type in
//! `kinds/`, picked by [`crate::Kind`]; the views in `view/` are written once against the trait.
//! The standard is the trait's source: `EmailAddress` and `PhoneNumber` come from the contract
//! crate `fs-interfaces` (`values`); the other types follow JSContact and RFC 3339 and move to the
//! types crate of Thread T when it is tagged.

use fs_components_styles::Glyph;

use crate::problem::Problem;

/// One part of a value (the local part of an address, the day of a date).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    /// Stable key, also the suffix of the translation keys `label.<type>.<key>`, `hint.<type>.<key>`.
    pub key: &'static str,
    /// Width of the part in the input, in units of the body font size (relative weight).
    pub weight: u8,
    /// Whether the part is required once any part is given.
    pub required: bool,
}

/// A basic type as the components see it. The value is its parts, in the order of
/// [`BasicField::segments`]; an empty string is "not given".
pub trait BasicField: Send + Sync {
    /// Stable key of the type (`email`, `name`, `phone`, `address`, `date`).
    fn key(&self) -> &'static str;

    /// The icon of the type.
    fn glyph(&self) -> Glyph;

    /// The parts, in order.
    fn segments(&self) -> &'static [Segment];

    /// The sign written between two parts in the input (`@` in an address), if any.
    fn joiner(&self, _after: usize) -> Option<&'static str> {
        None
    }

    /// The parts after an edit, put in order: a whole value typed or pasted into one part is spread
    /// over the parts it belongs to (an address typed into the first part of an e-mail field).
    /// The view writes the result back when it differs.
    fn settle(&self, parts: &[String]) -> Vec<String> {
        parts.to_vec()
    }

    /// What is wrong with these parts; `None` if they are fine or all empty (empty means no value).
    fn problem(&self, parts: &[String]) -> Option<Problem>;

    /// The parts of a text as it was pasted or stored, as well as can be told.
    fn split(&self, text: &str) -> Vec<String>;

    /// The parts as one text to store and compare; empty if no part is given.
    fn join(&self, parts: &[String]) -> String;

    /// The value in one line of running text (the compact component).
    fn compact(&self, parts: &[String]) -> String;

    /// The value for a list cell: the form to sort and scan by.
    fn cell(&self, parts: &[String]) -> String {
        self.compact(parts)
    }
}

/// Whether no part holds anything but spaces.
pub fn is_blank(parts: &[String]) -> bool {
    parts.iter().all(|p| p.trim().is_empty())
}
