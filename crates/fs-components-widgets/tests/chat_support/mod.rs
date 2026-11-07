//! Test support for the chat components: message makers and a generated long conversation.
//!
//! Tests: helpers that build the sources and scenes.
#![allow(dead_code)]

use fs_components_widgets::message_list::{ChatMessage, MessageSource, SourceError};
use fs_components_widgets::{Block, Inline, Sender, SenderKind};

/// A message with plain text.
pub fn msg(key: &str, sender: Sender, text: &str) -> ChatMessage {
    ChatMessage::new(key, sender, vec![Block::Paragraph(Inline::plain(text))])
}

/// From a person.
pub fn person(key: &str, name: &str) -> ChatMessage {
    msg(
        key,
        Sender::new("p", name).of_kind(SenderKind::Person),
        "Hello, how are you today?",
    )
}

/// From an AI.
pub fn ai(key: &str, name: &str) -> ChatMessage {
    let s = Sender::new("a", name)
        .of_kind(SenderKind::Ai)
        .operated_by("local llama");
    msg(key, s, "Hello, how are you today?")
}

/// From an unknown sender.
pub fn unknown(key: &str, name: &str) -> ChatMessage {
    msg(key, Sender::new("u", name), "Hello, how are you today?")
}

/// `n` messages made up on demand, of varied length; nothing is stored.
pub struct Generated(pub usize);

impl Generated {
    pub fn message(&self, i: usize) -> ChatMessage {
        let kind = [
            SenderKind::Person,
            SenderKind::Ai,
            SenderKind::Software,
            SenderKind::Unknown,
        ][i % 4];
        let words = 4 + (i * 7919) % 60;
        let text = (0..words)
            .map(|w| format!("word{}", (i + w) % 97))
            .collect::<Vec<_>>()
            .join(" ");
        msg(
            &format!("m{i}"),
            Sender::new(format!("s{}", i % 4), format!("Sender {}", i % 4)).of_kind(kind),
            &text,
        )
    }
}

impl MessageSource for Generated {
    fn count(&self) -> usize {
        self.0
    }

    fn messages(&self, first: usize, limit: usize) -> Result<Vec<ChatMessage>, SourceError> {
        Ok((first..(first + limit).min(self.0))
            .map(|i| self.message(i))
            .collect())
    }
}

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

/// A conversation the test can change at any time.
pub struct Shared(pub std::sync::Mutex<Vec<ChatMessage>>);

impl MessageSource for Shared {
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    fn messages(&self, first: usize, limit: usize) -> Result<Vec<ChatMessage>, SourceError> {
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
