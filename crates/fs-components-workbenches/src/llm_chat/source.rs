//! What the chat gets from outside.
//!
//! Pattern: Strategy / Repository — the workbench is open (ADR 0037): it knows no messenger, no
//! model and no store, only this trait. The host feeds it at run time; the workbench has no
//! network code and no streaming transport. Provisional: synchronous, called on the interface
//! thread (like `DocumentSource`).

use fs_components_widgets::agent_tree::AgentNode;
use fs_components_widgets::message_list::MessageSource;
use fs_components_widgets::tool_call::ToolCall;

/// Where the emergency stop stands, as the host (the engine) says. The workbench stops nothing.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopState {
    /// Can be pulled.
    Ready,
    /// Being carried out.
    Stopping,
    /// Done.
    Stopped,
    /// Not available here (for example no right to pull it).
    Unavailable,
}

/// A control given by the host. The workbench shows it and reports a click; the stop itself is
/// the engine's job (ADR 0054).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StopControl {
    /// The words on the button (the host's).
    pub label: String,
    /// Its state.
    pub state: StopState,
}

impl StopControl {
    /// A control.
    pub fn new(label: impl Into<String>, state: StopState) -> Self {
        Self {
            label: label.into(),
            state,
        }
    }
}

/// A question waiting for a person, with the message it belongs to.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Pending {
    /// The message holding the call.
    pub message: String,
    /// The call; its state is `Question` and it carries the options.
    pub call: ToolCall,
}

impl Pending {
    /// A pending question.
    pub fn new(message: impl Into<String>, call: ToolCall) -> Self {
        Self {
            message: message.into(),
            call,
        }
    }
}

/// One conversation as the chat needs it.
pub trait ChatSource: MessageSource {
    /// Names this conversation (its draft is kept under it).
    fn key(&self) -> String;
    /// The title to show.
    fn title(&self) -> String;
    /// The agents and subagents working in it.
    fn agents(&self) -> Vec<AgentNode> {
        Vec::new()
    }
    /// Questions waiting for a person: shown above the text field, so they cannot scroll away.
    fn questions(&self) -> Vec<Pending> {
        Vec::new()
    }
    /// The emergency-stop control, if the host gives one.
    fn stop_control(&self) -> Option<StopControl> {
        None
    }
    /// A line about use and cost in the host's words (tokens, context, money).
    fn usage(&self) -> Option<String> {
        None
    }
}
