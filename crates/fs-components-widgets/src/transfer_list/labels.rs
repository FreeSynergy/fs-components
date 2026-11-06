//! The words the transfer list shows.
//!
//! Pattern: Value object — every text comes from the caller (translation is the caller's); `{}`
//! is replaced by the thing the text talks about. Defaults are neutral English.

/// Texts of the transfer list.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Marks an upload.
    pub upload: String,
    /// Marks a download.
    pub download: String,
    /// State: waiting.
    pub queued: String,
    /// State: running.
    pub running: String,
    /// State: finished.
    pub done: String,
    /// State: failed; `{}` = the reason the caller gave (may be empty).
    pub failed: String,
    /// Button: try again.
    pub retry: String,
    /// Button: stop.
    pub cancel: String,
    /// Button: remove from the list.
    pub dismiss: String,
    /// Shown for an empty list.
    pub empty: String,
    /// More transfers than rows drawn; `{}` = how many.
    pub more: String,
    /// Some of the transfers not drawn have failed; `{}` = how many.
    pub more_failed: String,
    /// The size of a transfer is not known.
    pub unknown_size: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            upload: s("Upload"),
            download: s("Download"),
            queued: s("Waiting"),
            running: s("Running"),
            done: s("Done"),
            failed: s("Failed: {}"),
            retry: s("Retry"),
            cancel: s("Cancel"),
            dismiss: s("Dismiss"),
            empty: s("No transfers."),
            more: s("{} more transfers not shown."),
            more_failed: s("{} of them failed."),
            unknown_size: s("size unknown"),
        }
    }
}

impl Labels {
    /// `template` with its `{}` replaced by `value`.
    pub(crate) fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }
}
