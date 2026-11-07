//! What a transfer is and what the person can do with it.
//!
//! Pattern: State (the state decides which actions are offered). No Blinc type.

use std::sync::Arc;

use super::labels::Labels;

/// Which way the data goes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// From here to elsewhere.
    Upload,
    /// From elsewhere to here.
    Download,
}

/// Where a transfer is.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// Waiting to start.
    Queued,
    /// Moving data.
    Running,
    /// Finished.
    Done,
    /// Stopped by an error; the caller's reason (may be empty).
    Failed(String),
}

/// One transfer, as the caller describes it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    /// Identity given back in an [`Outcome`]; never shown.
    pub key: String,
    /// What is moved (a file name, any text).
    pub name: String,
    /// Which way.
    pub direction: Direction,
    /// Where it is.
    pub state: State,
    /// Bytes moved so far.
    pub done: u64,
    /// Bytes in all, if known.
    pub total: Option<u64>,
}

impl Transfer {
    /// A queued transfer of nothing yet.
    pub fn new(key: impl Into<String>, name: impl Into<String>, direction: Direction) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            direction,
            state: State::Queued,
            done: 0,
            total: None,
        }
    }

    /// With a state.
    #[must_use]
    pub fn with_state(mut self, state: State) -> Self {
        self.state = state;
        self
    }

    /// With progress.
    #[must_use]
    pub fn with_progress(mut self, done: u64, total: Option<u64>) -> Self {
        self.done = done;
        self.total = total;
        self
    }

    /// How far it is, `0.0..=1.0`; none when the size is not known (or zero) and it is not done.
    /// More done than total counts as complete.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // a bar needs no more precision
    pub fn fraction(&self) -> Option<f32> {
        if self.state == State::Done {
            return Some(1.0);
        }
        match self.total {
            Some(total) if total > 0 => {
                Some((self.done as f64 / total as f64).clamp(0.0, 1.0) as f32)
            }
            _ => None,
        }
    }

    /// What the person may do with it now.
    pub fn actions(&self) -> Vec<Action> {
        match self.state {
            State::Queued | State::Running => vec![Action::Cancel],
            State::Failed(_) => vec![Action::Retry, Action::Dismiss],
            _ => vec![Action::Dismiss],
        }
    }
}

/// What the person asks for.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Try a failed transfer again.
    Retry,
    /// Stop a waiting or running transfer.
    Cancel,
    /// Remove a finished or failed transfer from the list.
    Dismiss,
}

/// An action on one transfer; the caller carries it out and hands over the new state.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// The `key` of the transfer.
    pub key: String,
    /// What was asked.
    pub action: Action,
}

impl Outcome {
    /// An outcome.
    pub fn new(key: impl Into<String>, action: Action) -> Self {
        Self {
            key: key.into(),
            action,
        }
    }
}

/// Called when the person presses a button of a transfer.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

/// The list.
#[derive(Clone)]
pub struct TransferList {
    pub(super) transfers: Vec<Transfer>,
    pub(super) labels: Labels,
    pub(super) on_outcome: Option<OnOutcome>,
    pub(super) max_rows: usize,
}

impl TransferList {
    /// A list of `transfers`, at most 100 rows drawn (the rest is counted in a line).
    pub fn new(transfers: Vec<Transfer>) -> Self {
        Self {
            transfers,
            labels: Labels::default(),
            on_outcome: None,
            max_rows: 100,
        }
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Report the buttons pressed.
    #[must_use]
    pub fn on_outcome(mut self, handler: OnOutcome) -> Self {
        self.on_outcome = Some(handler);
        self
    }

    /// Draw at most `rows` rows.
    #[must_use]
    pub fn max_rows(mut self, rows: usize) -> Self {
        self.max_rows = rows;
        self
    }

    /// Indexes of the transfers drawn: the first `max_rows`, and every failed one after them
    /// (a failure never hides behind the cap; at most `max_rows` more of those, so the drawing
    /// stays bounded).
    pub fn shown(&self) -> Vec<usize> {
        let mut out: Vec<usize> = (0..self.transfers.len().min(self.max_rows)).collect();
        let late_failures = (self.max_rows..self.transfers.len())
            .filter(|&i| matches!(self.transfers[i].state, State::Failed(_)))
            .take(self.max_rows);
        out.extend(late_failures);
        out
    }

    /// The transfers not drawn.
    pub fn not_shown(&self) -> usize {
        self.transfers.len() - self.shown().len()
    }

    /// Failed transfers among those not drawn (only when there are more than `max_rows` of them).
    pub fn failed_not_shown(&self) -> usize {
        let shown = self.shown();
        (0..self.transfers.len())
            .filter(|i| matches!(self.transfers[*i].state, State::Failed(_)) && !shown.contains(i))
            .count()
    }
}

/// `1536` as `1.5 KiB`; whole bytes below 1 KiB.
#[allow(clippy::cast_precision_loss)]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}
