//! The words of a tool call. They come from the caller; English by default.
//!
//! Pattern: Value object.

use super::model::{ToolEnd, ToolKind, ToolRisk, ToolState};

/// Every word the tool call draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Kind: not known.
    pub kind_unknown: String,
    /// Kind: reads.
    pub kind_reads: String,
    /// Kind: writes.
    pub kind_writes: String,
    /// Kind: runs a command.
    pub kind_runs_command: String,
    /// Kind: a connected tool.
    pub kind_connected_tool: String,
    /// State: forming.
    pub state_forming: String,
    /// State: working.
    pub state_working: String,
    /// State: a question waits for an answer.
    pub state_question: String,
    /// State: settled.
    pub state_settled: String,
    /// End: not known.
    pub end_open: String,
    /// End: done.
    pub end_done: String,
    /// End: failed.
    pub end_failed: String,
    /// End: refused.
    pub end_refused: String,
    /// End: cancelled.
    pub end_cancelled: String,
    /// Risk: not judged; `{}` is not used.
    pub risk_unknown: String,
    /// Risk: only reads.
    pub risk_reads_only: String,
    /// Risk: changes something.
    pub risk_changes: String,
    /// Risk: destroys something.
    pub risk_destroys: String,
    /// Risk: acts in a person's name.
    pub risk_acts_for_person: String,
    /// Button: show details.
    pub show_details: String,
    /// Button: hide details.
    pub hide_details: String,
    /// Heading of the detail text.
    pub detail: String,
    /// Heading of the result text.
    pub result: String,
    /// Caption: the summary is the agent's own words.
    pub by_agent: String,
    /// Caption above what the engine says it will do.
    pub engine_detail: String,
    /// The command is longer than its window; `{}` = how many lines are out of sight.
    pub more_lines: String,
    /// The engine refused an answer; `{}` = its reason. The question is asked again.
    pub refused: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            kind_unknown: s("Unknown kind"),
            kind_reads: s("Reads"),
            kind_writes: s("Writes"),
            kind_runs_command: s("Runs a command"),
            kind_connected_tool: s("Connected tool"),
            state_forming: s("Forming"),
            state_working: s("Working"),
            state_question: s("Question"),
            state_settled: s("Settled"),
            end_open: s("Outcome not known"),
            end_done: s("Done"),
            end_failed: s("Failed"),
            end_refused: s("Refused"),
            end_cancelled: s("Cancelled"),
            risk_unknown: s("Risk unknown"),
            risk_reads_only: s("Only reads"),
            risk_changes: s("Changes"),
            risk_destroys: s("Destroys"),
            risk_acts_for_person: s("Acts in your name"),
            show_details: s("Details"),
            hide_details: s("Hide"),
            detail: s("Full text"),
            result: s("Result"),
            by_agent: s("Written by the agent:"),
            engine_detail: s("The engine will do:"),
            more_lines: s(
                "{} more lines. Scroll to the end to read them: Allow stays off until you have.",
            ),
            refused: s("The engine refused your answer: {}. Decide again."),
        }
    }
}

impl Labels {
    pub(super) fn kind(&self, k: ToolKind) -> &str {
        match k {
            ToolKind::Reads => &self.kind_reads,
            ToolKind::Writes => &self.kind_writes,
            ToolKind::RunsCommand => &self.kind_runs_command,
            ToolKind::ConnectedTool => &self.kind_connected_tool,
            _ => &self.kind_unknown,
        }
    }

    pub(super) fn state(&self, s: ToolState) -> &str {
        match s {
            ToolState::Working => &self.state_working,
            ToolState::Question => &self.state_question,
            ToolState::Settled => &self.state_settled,
            _ => &self.state_forming,
        }
    }

    pub(super) fn end(&self, e: ToolEnd) -> &str {
        match e {
            ToolEnd::Done => &self.end_done,
            ToolEnd::Failed => &self.end_failed,
            ToolEnd::Refused => &self.end_refused,
            ToolEnd::Cancelled => &self.end_cancelled,
            _ => &self.end_open,
        }
    }

    pub(super) fn risk(&self, r: ToolRisk) -> &str {
        match r {
            ToolRisk::ReadsOnly => &self.risk_reads_only,
            ToolRisk::Changes => &self.risk_changes,
            ToolRisk::Destroys => &self.risk_destroys,
            ToolRisk::ActsForPerson => &self.risk_acts_for_person,
            _ => &self.risk_unknown,
        }
    }
}
