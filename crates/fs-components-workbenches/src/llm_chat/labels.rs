//! All words of the chat. They come from the caller; English by default.
//!
//! Pattern: Value object (composite of the widgets' labels).

use fs_components_widgets::KindLabels;
use fs_components_widgets::{agent_tree, message_composer, message_list, tool_call};

/// Every word the chat draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The message list.
    pub list: message_list::Labels,
    /// The text field.
    pub composer: message_composer::Labels,
    /// The agent tree.
    pub tree: agent_tree::Labels,
    /// The question dock.
    pub tools: tool_call::Labels,
    /// Heading of the agent side panel.
    pub agents: String,
    /// Heading of the dock; `{}` = how many are waiting.
    pub waiting: String,
    /// Agents at work, in the compact header; `{}` = how many.
    pub working: String,
    /// The stop is being carried out.
    pub stopping: String,
    /// The stop is done.
    pub stopped: String,
    /// The stop is not available here.
    pub stop_unavailable: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            list: message_list::Labels::default(),
            composer: message_composer::Labels::default(),
            tree: agent_tree::Labels::default(),
            tools: tool_call::Labels::default(),
            agents: s("Agents"),
            waiting: s("{} waiting for you"),
            working: s("{} working"),
            stopping: s("Stopping..."),
            stopped: s("Stopped"),
            stop_unavailable: s("Not available"),
        }
    }
}

impl Labels {
    /// Use these words for the participant kinds everywhere in the chat.
    #[must_use]
    pub fn with_kinds(mut self, kinds: &KindLabels) -> Self {
        self.list.kinds = kinds.clone();
        self.tree.kinds = kinds.clone();
        self
    }
}
