//! The chat over a made-up conversation: no messenger, no model. A thread plays the host and
//! feeds an answer word by word, so the live paths (growing text, following, the new-message
//! button) can be tried by hand.
//!
//! Pattern: Strategy (a `ChatSource` implementation). `cargo run -p fs-components-workbenches
//! --example llm_chat` opens a window; `-- --compact` shows the compact form; `-- --seconds N`; `--feed-ms N` sets the pace of the feed (default 250); a question appears after about 6 s
//! closes it after N seconds (default: stays open).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use blinc_app::prelude::*;
use blinc_app::windowed::WindowedApp;
use fs_components_widgets::agent_tree::{AgentNode, AgentState};
use fs_components_widgets::message_list::{ChatMessage, MessageSource, SourceError};
use fs_components_widgets::tool_call::{ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk};
use fs_components_widgets::{Block, Inline, Sender, SenderKind};
use fs_components_workbenches::llm_chat::{
    ChatSource, LlmChat, Pending, StopControl, StopState, styles,
};
use fs_components_workbenches::register_fonts;

#[derive(Default)]
struct Demo(Mutex<Vec<ChatMessage>>, Mutex<Vec<Pending>>);

fn message(key: usize, who: &str, kind: SenderKind, text: &str) -> ChatMessage {
    let s = Sender::new(who, who).of_kind(kind);
    ChatMessage::new(
        key.to_string(),
        s,
        vec![Block::Paragraph(Inline::plain(text))],
    )
}

impl MessageSource for Demo {
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    fn messages(
        &self,
        first: usize,
        limit: usize,
    ) -> std::result::Result<Vec<ChatMessage>, SourceError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .skip(first)
            .take(limit)
            .cloned()
            .collect())
    }
}

impl ChatSource for Demo {
    fn key(&self) -> String {
        "demo".into()
    }

    fn title(&self) -> String {
        "Demo conversation".into()
    }

    fn agents(&self) -> Vec<AgentNode> {
        vec![
            AgentNode::new("a", "Helper")
                .of_kind(SenderKind::Ai)
                .state(AgentState::Working),
        ]
    }

    fn questions(&self) -> Vec<Pending> {
        self.1.lock().unwrap().clone()
    }

    fn stop_control(&self) -> Option<StopControl> {
        Some(StopControl::new("Stop all agents", StopState::Ready))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let compact = args.iter().any(|a| a == "--compact");
    let seconds = args
        .iter()
        .position(|a| a == "--seconds")
        .and_then(|i| args.get(i + 1)?.parse::<u64>().ok());
    let feed = args
        .iter()
        .position(|a| a == "--feed-ms")
        .and_then(|i| args.get(i + 1)?.parse::<u64>().ok())
        .unwrap_or(250);
    let demo = Arc::new(Demo::default());
    for i in 0..60 {
        let (who, kind) = if i % 2 == 0 {
            ("Ana", SenderKind::Person)
        } else {
            ("Helper", SenderKind::Ai)
        };
        demo.0.lock().unwrap().push(message(i, who, kind, "An older line of the conversation, long enough to wrap onto a second line when the window is narrow."));
    }
    let chat = LlmChat::new(demo.clone()).compact(compact);
    let feeder = chat.clone();
    std::thread::spawn(move || {
        let key = 1000;
        demo.0
            .lock()
            .unwrap()
            .push(message(key, "Helper", SenderKind::Ai, ""));
        let mut text = String::new();
        for i in 0..u64::MAX {
            std::thread::sleep(Duration::from_millis(feed));
            if i == 6000 / feed {
                let call = ToolCall::new("q1", "shell")
                    .kind(ToolKind::RunsCommand)
                    .risk(ToolRisk::Destroys)
                    .summary("Clean up")
                    .detail("rm -rf ./target")
                    .asking(vec![
                        ToolOption::new("no", "Refuse", ToolChoice::Refuse),
                        ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
                    ])
                    .revision(1)
                    .digest(b"x".to_vec());
                *demo.1.lock().unwrap() = vec![Pending::new("1000", call)];
            }
            text.push_str(&format!("word{i} "));
            let mut m = demo.0.lock().unwrap();
            let last = m.last_mut().unwrap();
            *last = message(key, "Helper", SenderKind::Ai, &text)
                .streaming(true)
                .revision(i);
            drop(m);
            feeder.refresh();
        }
    });
    if let Some(s) = seconds {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(s));
            std::process::exit(0);
        });
    }
    register_fonts();
    let config = WindowConfig {
        title: "llm-chat".into(),
        width: 1000,
        height: 700,
        ..Default::default()
    };
    let bundle = fs_components_render::bundle(&styles());
    WindowedApp::run_with_theme(
        config,
        bundle,
        blinc_theme::detect_system_color_scheme(),
        move |ctx| chat.build(ctx.width, ctx.height),
    )
    .expect("window");
}
