//! What the composer is given and what it reports.
//!
//! Pattern: Value objects (the optional draft store is the shared `DraftStore`). The
//! composer knows no messenger: sending is an outcome, attachments are a list the host hands in.

/// One attachment shown beside the text. The host owns the file; the composer only lists it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    /// Key the host knows it by.
    pub key: String,
    /// Name to show.
    pub name: String,
    /// A second line (size, type), made by the host; may be empty.
    pub detail: String,
}

impl Attachment {
    /// An attachment.
    pub fn new(key: impl Into<String>, name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            detail: detail.into(),
        }
    }
}

/// Why a send did nothing.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// There is no text (or only white space).
    Empty,
    /// The last send has not been answered yet.
    Busy,
}

/// What a person did that the host must answer.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Send this. The text is exactly as typed. Nothing is cleared until the host calls `sent()`.
    Send {
        /// The text, unchanged.
        text: String,
        /// Keys of the attachments listed now.
        attachments: Vec<String>,
        /// The number of this attempt; answer with `sent_for` / `failed_for`.
        attempt: u64,
    },
    /// A send was asked for but not made.
    Refused(Refusal),
    /// The person asked to remove this attachment (the host removes it from the list).
    RemoveAttachment {
        /// The attachment's key.
        key: String,
    },
    /// The person asked to attach something (the host opens its own picker).
    AttachRequested,
}
