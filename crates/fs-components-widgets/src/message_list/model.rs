//! A message as the list sees it: neutral data, and the trait the host fills it from.
//!
//! Pattern: Value objects + Strategy/Repository (`MessageSource`). No messenger type appears: a
//! host maps its own messages to [`ChatMessage`]. The list is told nothing about transport or
//! streaming; text that grows is the host changing its source and calling `refresh`.

use crate::participant::Sender;
use crate::text::Block;
use crate::tool_call::ToolCall;

/// Where a sent message stands. `None` on a message means: not applicable (received).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// Being sent (or, for an agent's text, still being written).
    Sending,
    /// Sent.
    Sent,
    /// Delivered.
    Delivered,
    /// Read.
    Read,
    /// Failed; the reason is `ChatMessage::failure`.
    Failed,
}

/// Something a message refers to (a task): shown through what the host gives, opened by the host.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    /// Key the host knows it by (handed back in the outcome).
    pub key: String,
    /// The words to show (title, state: whatever the host decided).
    pub label: String,
}

impl Reference {
    /// A reference.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// One message.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    /// Unique key.
    pub key: String,
    /// Who sent it. Always drawn, with its kind.
    pub sender: Sender,
    /// The person using this view wrote it.
    pub mine: bool,
    /// The text, as blocks of our text core. What cannot be drawn arrives as `Block::Plain`.
    pub blocks: Vec<Block>,
    /// Reasoning of an agent, kept apart from the answer.
    pub reasoning: Vec<Block>,
    /// Time to show, formatted by the host.
    pub time: String,
    /// Delivery, if it applies.
    pub delivery: Option<Delivery>,
    /// Why it failed (shown when `delivery` is `Failed`).
    pub failure: String,
    /// The text is still growing.
    pub streaming: bool,
    /// What an agent does in this message.
    pub tool_calls: Vec<ToolCall>,
    /// What it refers to.
    pub references: Vec<Reference>,
    /// Raised by the host when the content changes (the list measures it again).
    pub revision: u64,
}

impl ChatMessage {
    /// A message from `sender` with this text.
    pub fn new(key: impl Into<String>, sender: Sender, blocks: Vec<Block>) -> Self {
        Self {
            key: key.into(),
            sender,
            mine: false,
            blocks,
            reasoning: Vec::new(),
            time: String::new(),
            delivery: None,
            failure: String::new(),
            streaming: false,
            tool_calls: Vec::new(),
            references: Vec::new(),
            revision: 0,
        }
    }

    /// Mark it as written by the person using this view.
    #[must_use]
    pub fn mine(mut self, mine: bool) -> Self {
        self.mine = mine;
        self
    }

    /// Set the time text.
    #[must_use]
    pub fn at(mut self, time: impl Into<String>) -> Self {
        self.time = time.into();
        self
    }

    /// Set the delivery state.
    #[must_use]
    pub fn delivery(mut self, delivery: Delivery) -> Self {
        self.delivery = Some(delivery);
        self
    }

    /// Fail with a reason.
    #[must_use]
    pub fn failed(mut self, reason: impl Into<String>) -> Self {
        self.delivery = Some(Delivery::Failed);
        self.failure = reason.into();
        self
    }

    /// Say that the text is still growing.
    #[must_use]
    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    /// Add reasoning.
    #[must_use]
    pub fn reasoning(mut self, blocks: Vec<Block>) -> Self {
        self.reasoning = blocks;
        self
    }

    /// Set the tool calls.
    #[must_use]
    pub fn tool_calls(mut self, calls: Vec<ToolCall>) -> Self {
        self.tool_calls = calls;
        self
    }

    /// Set the references.
    #[must_use]
    pub fn references(mut self, references: Vec<Reference>) -> Self {
        self.references = references;
        self
    }

    /// Set the revision.
    #[must_use]
    pub fn revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }
}

/// Why messages could not be read.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceError(pub String);

impl SourceError {
    /// An error with a reason to show.
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

/// Where the messages come from. Index 0 is the oldest. Provisional: synchronous, called on the
/// interface thread (like `DocumentSource`).
pub trait MessageSource: Send + Sync {
    /// How many messages there are.
    fn count(&self) -> usize;
    /// Up to `limit` messages starting at `first`.
    ///
    /// # Errors
    /// When they cannot be read; the list shows the reason in place of the messages.
    fn messages(&self, first: usize, limit: usize) -> Result<Vec<ChatMessage>, SourceError>;
}

/// A source over a vector, for small conversations and tests.
#[derive(Clone, Debug, Default)]
pub struct VecMessages(pub Vec<ChatMessage>);

impl MessageSource for VecMessages {
    fn count(&self) -> usize {
        self.0.len()
    }

    fn messages(&self, first: usize, limit: usize) -> Result<Vec<ChatMessage>, SourceError> {
        Ok(self.0.iter().skip(first).take(limit).cloned().collect())
    }
}
