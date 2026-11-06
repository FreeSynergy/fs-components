//! Agents and subagents as data, and what a person can do with the tree.
//!
//! Pattern: Value objects. Neutral: no messenger, no model. An agent is a marked participant: its
//! kind (`SenderKind`) comes with it and is drawn on every row.

use crate::participant::SenderKind;

/// Where an agent is in its life (the same four states as a tool call).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgentState {
    /// Being set up.
    #[default]
    Forming,
    /// Working.
    Working,
    /// Waiting for a person's answer.
    Question,
    /// Over.
    Settled,
}

/// How a settled agent ended.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgentEnd {
    /// Not known (or not over).
    #[default]
    Open,
    /// It delivered what was asked.
    Delivered,
    /// It failed.
    Failed,
    /// It was cancelled.
    Cancelled,
}

/// What asks for a person's attention: nothing else does.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Attention {
    /// An agent waits for an answer.
    Question,
    /// An agent delivered and nobody has looked yet.
    Delivery,
    /// An agent failed.
    Failure,
}

/// One agent.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentNode {
    /// Unique key; an update carries the same one.
    pub key: String,
    /// The parent's key; none for a top agent. An unknown parent counts as none.
    pub parent: Option<String>,
    /// Name to show.
    pub name: String,
    /// What kind of participant it is.
    pub kind: SenderKind,
    /// Where it is.
    pub state: AgentState,
    /// How it ended.
    pub end: AgentEnd,
    /// One line of what it does (the host's words).
    pub summary: String,
    /// The person has looked at a delivery.
    pub acknowledged: bool,
}

impl AgentNode {
    /// A forming agent of unknown kind.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            parent: None,
            name: name.into(),
            kind: SenderKind::Unknown,
            state: AgentState::Forming,
            end: AgentEnd::Open,
            summary: String::new(),
            acknowledged: false,
        }
    }

    /// Under this parent.
    #[must_use]
    pub fn under(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Of this kind.
    #[must_use]
    pub fn of_kind(mut self, kind: SenderKind) -> Self {
        self.kind = kind;
        self
    }

    /// In this state.
    #[must_use]
    pub fn state(mut self, state: AgentState) -> Self {
        self.state = state;
        self
    }

    /// Settled with this end.
    #[must_use]
    pub fn settled(mut self, end: AgentEnd) -> Self {
        self.state = AgentState::Settled;
        self.end = end;
        self
    }

    /// With this summary line.
    #[must_use]
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// The delivery was looked at.
    #[must_use]
    pub fn acknowledged(mut self, yes: bool) -> Self {
        self.acknowledged = yes;
        self
    }

    /// What this agent asks of the person: a question, a delivery not yet looked at, a failure.
    pub fn attention(&self) -> Option<Attention> {
        match (self.state, self.end) {
            (AgentState::Question, _) => Some(Attention::Question),
            (AgentState::Settled, AgentEnd::Failed) => Some(Attention::Failure),
            (AgentState::Settled, AgentEnd::Delivered) if !self.acknowledged => {
                Some(Attention::Delivery)
            }
            _ => None,
        }
    }
}

/// How many of each kind of attention.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Questions.
    pub questions: usize,
    /// Deliveries.
    pub deliveries: usize,
    /// Failures.
    pub failures: usize,
}

impl Counts {
    /// All together.
    pub fn total(&self) -> usize {
        self.questions + self.deliveries + self.failures
    }

    pub(super) fn add(&mut self, other: Counts) {
        self.questions += other.questions;
        self.deliveries += other.deliveries;
        self.failures += other.failures;
    }

    pub(super) fn count(&mut self, a: Option<Attention>) {
        match a {
            Some(Attention::Question) => self.questions += 1,
            Some(Attention::Delivery) => self.deliveries += 1,
            Some(Attention::Failure) => self.failures += 1,
            _ => {}
        }
    }

    /// The most urgent kind present: a question, then a failure, then a delivery.
    pub fn most_urgent(&self) -> Option<Attention> {
        if self.questions > 0 {
            Some(Attention::Question)
        } else if self.failures > 0 {
            Some(Attention::Failure)
        } else if self.deliveries > 0 {
            Some(Attention::Delivery)
        } else {
            None
        }
    }
}

/// What a person did.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// An agent was chosen: the host shows its card or answers its question.
    Selected {
        /// The agent's key.
        key: String,
    },
    /// A branch was opened or closed.
    Toggled {
        /// The agent's key.
        key: String,
        /// Is it open now?
        open: bool,
    },
}
