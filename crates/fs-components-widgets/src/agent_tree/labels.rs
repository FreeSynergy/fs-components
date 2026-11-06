//! The words of the agent tree. They come from the caller; English by default.
//!
//! Pattern: Value object.

use crate::participant::KindLabels;

/// Every word the tree draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The words for the participant kinds.
    pub kinds: KindLabels,
    /// State: forming.
    pub forming: String,
    /// State: working.
    pub working: String,
    /// State: a question waits.
    pub question: String,
    /// State: settled.
    pub settled: String,
    /// End: not known.
    pub open: String,
    /// End: delivered.
    pub delivered: String,
    /// End: failed.
    pub failed: String,
    /// End: cancelled.
    pub cancelled: String,
    /// Button on a closed branch.
    pub expand: String,
    /// Button on an open branch.
    pub collapse: String,
    /// On a closed branch: something below needs attention; `{}` = how many.
    pub below: String,
    /// Above the tree: something needs attention; `{}` = how many.
    pub needs_attention: String,
    /// Above the tree: nothing needs attention.
    pub all_quiet: String,
    /// No agents.
    pub empty: String,
    /// On a row deeper than the indentation shows; `{}` = the depth.
    pub depth: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            kinds: KindLabels::default(),
            forming: s("Forming"),
            working: s("Working"),
            question: s("Question"),
            settled: s("Settled"),
            open: s("Outcome not known"),
            delivered: s("Delivered"),
            failed: s("Failed"),
            cancelled: s("Cancelled"),
            expand: s("+"),
            collapse: s("-"),
            below: s("{} below need attention"),
            needs_attention: s("{} need attention"),
            all_quiet: s("Nothing needs attention"),
            empty: s("No agents."),
            depth: s("level {}"),
        }
    }
}
