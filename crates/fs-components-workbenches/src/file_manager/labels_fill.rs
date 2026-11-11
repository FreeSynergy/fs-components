//! Putting what a sentence talks about into it.
//!
//! Pattern: none (pure functions on the words of the caller).

use super::labels::FileLabels;

impl FileLabels {
    /// `text` with `{}` replaced by `what`.
    pub fn fill(text: &str, what: &str) -> String {
        text.replacen("{}", what, 1)
    }

    /// `text` with the placeholder `{name}` replaced by `value`: a sentence that names what goes
    /// into it, so a translation may move it.
    pub fn fill_named(text: &str, name: &str, value: &str) -> String {
        text.replace(&format!("{{{name}}}"), value)
    }
}
