//! What a person did that the host must answer.
//!
//! Pattern: none. Nothing here is a decision: a question's
//! answer and the stop are reported, the host (the engine) decides and carries them out.

use fs_components_widgets::tool_call::Decision;

/// What the chat reports.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Send this text exactly (keys of the listed attachments). Answer with `sent()` or `failed()`.
    Send {
        /// The text.
        text: String,
        /// The attachments listed.
        attachments: Vec<String>,
        /// The attempt; answer with the composer's `sent_for` / `failed_for` for late answers.
        attempt: u64,
    },
    /// Remove this attachment from the list.
    RemoveAttachment {
        /// The attachment's key.
        key: String,
    },
    /// The person wants to attach something.
    AttachRequested,
    /// A question was answered, bound to the revision and digest drawn when the person decided.
    /// Nothing has happened yet; the host passes it to the engine, which refuses it if it no longer fits.
    ToolChose(Decision),
    /// Send a failed message again.
    Retry {
        /// The message.
        message: String,
    },
    /// A reference (a task) was clicked.
    Reference {
        /// The message.
        message: String,
        /// The reference.
        reference: String,
    },
    /// An agent was chosen.
    AgentSelected {
        /// The agent.
        key: String,
    },
    /// The person pressed the stop control the host gave. The workbench stops nothing.
    StopRequested,
}
