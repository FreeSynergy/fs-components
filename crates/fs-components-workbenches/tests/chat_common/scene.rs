//! Test scenes for the chat: a clock by hand and a busy conversation.
//!
//! Tests: helpers that build the sources and scenes.
#![allow(dead_code)]

use std::sync::Arc;

use fs_components_widgets::SenderKind;
use fs_components_workbenches::llm_chat::{Pending, StopState};

use super::{Chat, say, stop};

/// A clock the test moves by hand.
#[derive(Default)]
pub struct Manual(pub std::sync::atomic::AtomicU64);

impl Manual {
    pub fn advance(&self, ms: u64) {
        self.0.fetch_add(ms, std::sync::atomic::Ordering::SeqCst);
    }
}

impl fs_components_widgets::tool_call::Clock for Manual {
    fn now_ms(&self) -> u64 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// A conversation with four senders of every kind, agents, a question and a stop control.
pub fn busy_chat() -> Arc<Chat> {
    use fs_components_widgets::agent_tree::{AgentEnd, AgentNode, AgentState};
    use fs_components_widgets::tool_call::{ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk};
    let c = Arc::new(Chat::default());
    {
        let mut m = c.messages.lock().unwrap();
        m.push(
            say(
                "1",
                "Ana",
                SenderKind::Person,
                "Can you prepare the release notes?",
            )
            .at("10:02"),
        );
        m.push(
            say(
                "2",
                "Helper",
                SenderKind::Ai,
                "Yes. I started two helpers: one collects the changes, one writes the text.",
            )
            .at("10:02"),
        );
        m.push(
            say(
                "3",
                "indexer",
                SenderKind::Software,
                "Index updated: 212 pages.",
            )
            .at("10:03"),
        );
        m.push(say("4", "Stranger", SenderKind::Unknown, "Who allowed this?").at("10:04"));
    }
    *c.agents.lock().unwrap() = vec![
        AgentNode::new("h", "Helper")
            .of_kind(SenderKind::Ai)
            .state(AgentState::Working)
            .summary("Plan the release"),
        AgentNode::new("h1", "Collector")
            .of_kind(SenderKind::Ai)
            .under("h")
            .settled(AgentEnd::Failed)
            .summary("tests broke"),
        AgentNode::new("h2", "Writer")
            .of_kind(SenderKind::Ai)
            .under("h")
            .state(AgentState::Question)
            .summary("May I delete the build?"),
    ];
    let call = ToolCall::new("q1", "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Delete the old build folder")
        .detail("rm -rf ./target")
        .asking(vec![
            ToolOption::new("no", "Refuse", ToolChoice::Refuse),
            ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
        ])
        .revision(1)
        .digest(b"rm -rf ./target".to_vec());
    *c.questions.lock().unwrap() = vec![Pending::new("2", call)];
    *c.stop.lock().unwrap() = stop(StopState::Ready);
    c
}
