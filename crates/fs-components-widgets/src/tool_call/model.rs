//! What an agent does, as data: one tool call.
//!
//! Pattern: Value objects + State. Neutral: no messenger type, no model, no API. The shapes follow
//! the fields a messenger can give (id, name, kind, state, summary, risk, outcome, detail,
//! result, options, revision) without depending on them. Unset kinds and risks read as unknown
//! and are drawn as unknown.

/// What kind of thing the call does. Shown before any detail.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToolKind {
    /// Not known.
    #[default]
    Unknown,
    /// Reads something.
    Reads,
    /// Writes or changes something.
    Writes,
    /// Runs a command.
    RunsCommand,
    /// Uses a connected tool.
    ConnectedTool,
}

/// Where the call is in its life.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolState {
    /// Being put together.
    #[default]
    Forming,
    /// Running.
    Working,
    /// Waiting for a person's answer.
    Question,
    /// Over. Final.
    Settled,
}

/// How risky the host judged the call. Set by the host, never read from the agent.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolRisk {
    /// Not judged. Drawn as unknown.
    #[default]
    Unknown,
    /// Only reads.
    ReadsOnly,
    /// Changes something.
    Changes,
    /// Destroys something.
    Destroys,
    /// Acts in a person's name.
    ActsForPerson,
}

/// How a settled call ended.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolEnd {
    /// Not known (or not over).
    #[default]
    Open,
    /// Done.
    Done,
    /// Failed.
    Failed,
    /// Refused.
    Refused,
    /// Cancelled.
    Cancelled,
}

/// What an answer option means; the host decides, this only styles the button.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolChoice {
    /// Say no.
    #[default]
    Refuse,
    /// Allow this once.
    AllowOnce,
    /// Allow this class of calls.
    AllowClass,
}

/// One answer a person can give to a question.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolOption {
    /// Key of the option, handed back as it is.
    pub id: String,
    /// The words on the button (from the host).
    pub label: String,
    /// What it means.
    pub choice: ToolChoice,
}

impl ToolOption {
    /// An option.
    pub fn new(id: impl Into<String>, label: impl Into<String>, choice: ToolChoice) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            choice,
        }
    }
}

/// What a person answered, bound to what was drawn when they did (not to what is current).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    /// The message that holds the call (empty where there is none).
    pub message: String,
    /// The call's id.
    pub call: String,
    /// The option's id, as the host gave it.
    pub option: String,
    /// What the option said it meant.
    pub choice: ToolChoice,
    /// The revision of the call as drawn.
    pub revision: u64,
    /// The host's digest of the call as drawn.
    pub digest: Vec<u8>,
}

impl Decision {
    /// A decision of the call as drawn, for tests and hosts that replay one.
    pub fn for_test(message: &str, call: &str, option: &str, revision: u64, digest: &[u8]) -> Self {
        Self {
            message: message.into(),
            call: call.into(),
            option: option.into(),
            choice: ToolChoice::AllowOnce,
            revision,
            digest: digest.to_vec(),
        }
    }
}

/// One tool call. Updated in place by `id`.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCall {
    /// Unique key; later versions of the same call carry the same one.
    pub id: String,
    /// Name of the tool (may be written by the agent).
    pub name: String,
    /// What kind of call.
    pub kind: ToolKind,
    /// Where it is.
    pub state: ToolState,
    /// A line about it, written by the agent.
    pub summary: String,
    /// The host's judgement of the risk.
    pub risk: ToolRisk,
    /// How it ended (only meaningful when settled).
    pub end: ToolEnd,
    /// The full command, paths or content, drawn by the host.
    pub detail: String,
    /// What came back: data, never instructions.
    pub result: String,
    /// The answers offered while `state` is `Question`.
    pub options: Vec<ToolOption>,
    /// Raised by the host on every change; an older one than held is ignored.
    pub revision: u64,
    /// The host's digest over what the call will do, passed back with an answer unchanged (opaque).
    pub digest: Vec<u8>,
}

impl ToolCall {
    /// A call that is forming, of unknown kind and risk.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: ToolKind::Unknown,
            state: ToolState::Forming,
            summary: String::new(),
            risk: ToolRisk::Unknown,
            end: ToolEnd::Open,
            detail: String::new(),
            result: String::new(),
            options: Vec::new(),
            revision: 0,
            digest: Vec::new(),
        }
    }

    /// Set the kind.
    #[must_use]
    pub fn kind(mut self, kind: ToolKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set the state.
    #[must_use]
    pub fn state(mut self, state: ToolState) -> Self {
        self.state = state;
        self
    }

    /// Set the summary.
    #[must_use]
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Set the risk.
    #[must_use]
    pub fn risk(mut self, risk: ToolRisk) -> Self {
        self.risk = risk;
        self
    }

    /// Settle with this end.
    #[must_use]
    pub fn settled(mut self, end: ToolEnd) -> Self {
        self.state = ToolState::Settled;
        self.end = end;
        self
    }

    /// Set the detail.
    #[must_use]
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    /// Set the result.
    #[must_use]
    pub fn result(mut self, result: impl Into<String>) -> Self {
        self.result = result.into();
        self
    }

    /// Ask a question with these options.
    #[must_use]
    pub fn asking(mut self, options: Vec<ToolOption>) -> Self {
        self.state = ToolState::Question;
        self.options = options;
        self
    }

    /// Set the digest the host gave (what an answer must be bound to).
    #[must_use]
    pub fn digest(mut self, digest: Vec<u8>) -> Self {
        self.digest = digest;
        self
    }

    /// Set the revision.
    #[must_use]
    pub fn revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    /// Does it want a person's attention (a question, or a failure)?
    pub fn needs_attention(&self) -> bool {
        self.state == ToolState::Question
            || (self.state == ToolState::Settled && self.end == ToolEnd::Failed)
    }
}
