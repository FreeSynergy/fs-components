//! What the frontend needs from a program: the records and the commands.
//!
//! Pattern: Repository (`coding.md` §2) — the trait is the interface (ADR 0043); the frontend depends on it and on the
//! type description, never on a program's types. A program implements it over its Cap'n Proto contract
//! (`people.capnp` …), a test over memory.
//!
//! Provisional (reported to the strategy): the description names commands but not how a list is read. Here the list is
//! the command kind `find`: [`Backend::list`] gets the typed input of that command (empty = everything).

use std::collections::BTreeMap;

use fs_types::RecordIssue;

/// Typed values by field name, several per field when the field is `multiple`. Texts are what a person typed or the
/// canonical text the program stores.
pub type Values = BTreeMap<String, Vec<String>>;

/// One stored record of the data type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// The identity (an AS2 `id`, an IRI); the frontend never makes one up.
    pub id: String,
    /// The values.
    pub values: Values,
}

/// What a command answers: the identity it created or changed, and the values of its result fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The record the command created or changed.
    pub id: Option<String>,
    /// The result fields of the command.
    pub values: Values,
}

/// Why a program refused or failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendError {
    /// The program checked the input and found these; the form marks them.
    Rejected(Vec<RecordIssue>),
    /// There is no such record or command.
    NotFound,
    /// The program cannot be reached or failed; the text is for the log (English), a person sees a code.
    Unavailable(String),
}

/// The records of one data type and the commands of its description.
pub trait Backend: Send + Sync {
    /// The records the command `find` selects with `query` (typed input; empty = all), in the program's order.
    ///
    /// # Errors
    /// [`BackendError`] when the program cannot answer.
    fn list(&self, query: &Values) -> Result<Vec<Record>, BackendError>;

    /// Whether the core can carry out the command `name` now. A command that is described but not yet built answers
    /// `false`: the frontend shows its control locked with "coming soon" and unlocks it when this turns `true`, with no
    /// change in the frontend. Asked when the records are read. **The default is `false`**: a program that does not
    /// answer has not said it can do anything, so nothing is unlocked by silence.
    ///
    /// Provisional (reported to the strategy and Thread T): the description is to carry `supported` per command
    /// (`types.capnp`); when it does, a command is supported only if the description and this both say so.
    fn supported(&self, command: &str) -> bool {
        let _ = command;
        false
    }

    /// Carry out the command `name` of the description with `input` (already checked against the description).
    ///
    /// **The rule for changing (part of the contract, reported to the strategy):** a command of kind `change` is
    /// *partial*. A field that is missing from `input` is unchanged; a field with an empty list is cleared; a field
    /// with values replaces the old values. Required fields are always sent. A program that reads `input` as a whole
    /// record would delete the optional fields that were not sent. A finding of `Rejected` counts the values of a
    /// field as they were sent (empty slots are not sent); the form maps it back to its slots.
    ///
    /// # Errors
    /// [`BackendError::Rejected`] with the findings, [`BackendError::NotFound`], [`BackendError::Unavailable`].
    fn run(&self, name: &str, input: &Values) -> Result<Outcome, BackendError>;
}
