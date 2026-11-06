//! The words of the message list. They come from the caller; English by default.
//!
//! Pattern: Value object.

use crate::participant::KindLabels;
use crate::tool_call::Labels as ToolLabels;

/// Every word the list draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The words for the sender kinds.
    pub kinds: KindLabels,
    /// The words of the tool call cards.
    pub tools: ToolLabels,
    /// Delivery: sending.
    pub sending: String,
    /// Delivery: sent.
    pub sent: String,
    /// Delivery: delivered.
    pub delivered: String,
    /// Delivery: read.
    pub read: String,
    /// Delivery: failed.
    pub failed: String,
    /// Button: send a failed message again.
    pub retry: String,
    /// The text is still growing.
    pub writing: String,
    /// A message without text.
    pub empty_message: String,
    /// Button: show reasoning.
    pub show_reasoning: String,
    /// Button: hide reasoning.
    pub hide_reasoning: String,
    /// New messages below; `{}` = how many.
    pub new_messages: String,
    /// No messages at all.
    pub no_messages: String,
    /// Messages could not be read; `{}` = the reason.
    pub load_failed: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            kinds: KindLabels::default(),
            tools: ToolLabels::default(),
            sending: s("Sending"),
            sent: s("Sent"),
            delivered: s("Delivered"),
            read: s("Read"),
            failed: s("Not sent"),
            retry: s("Retry"),
            writing: s("Writing..."),
            empty_message: s("(no text)"),
            show_reasoning: s("Reasoning"),
            hide_reasoning: s("Hide reasoning"),
            new_messages: s("{} new messages"),
            no_messages: s("No messages yet."),
            load_failed: s("Messages could not be read: {}"),
        }
    }
}
