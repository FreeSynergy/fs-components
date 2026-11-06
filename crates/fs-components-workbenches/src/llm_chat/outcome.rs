//! What a person did that the host must answer.
//!
//! Pattern: Value object, flattened from the parts. Nothing here is a decision: a question's
//! answer and the stop are reported, the host (the engine) decides and carries them out.

use fs_components_widgets::tool_call::ToolChoice;

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
    },
    /// Remove this attachment from the list.
    RemoveAttachment {
        /// The attachment's key.
        key: String,
    },
    /// The person wants to attach something.
    AttachRequested,
    /// A question was answered. Nothing has happened yet.
    ToolChose {
        /// The message holding the call.
        message: String,
        /// The call.
        call: String,
        /// The option chosen.
        option: String,
        /// What the option said it meant.
        choice: ToolChoice,
    },
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
