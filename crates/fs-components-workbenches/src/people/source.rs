//! Where the people workbench gets its people.
//!
//! Pattern: Strategy — the workbench depends on this trait only (an Adapter over a
//! program's core implements it). The trait is synchronous and called on the interface thread,
//! like `DocumentSource`; this is provisional (see DESIGN.md).

use super::model::{PeopleCapabilities, PersonDraft, PersonRow, PersonSheet};
use crate::error::SourceError;

/// A source of people.
pub trait PeopleSource: Send + Sync {
    /// What it can do.
    fn capabilities(&self) -> PeopleCapabilities;

    /// People matching `term` (empty = everyone), in the source's order, skipping `offset`, at most
    /// `limit`.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read.
    fn rows(&self, term: &str, offset: usize, limit: usize) -> Result<Vec<PersonRow>, SourceError>;

    /// One person, to be read.
    ///
    /// # Errors
    /// [`SourceError`]; [`ErrorKind::NotFound`](crate::ErrorKind::NotFound) if there is none.
    fn read(&self, key: &str) -> Result<PersonSheet, SourceError>;

    /// One person, as a form.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot edit says `Unsupported`.
    fn draft(&self, _key: &str) -> Result<PersonDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// An empty form for a new person.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot create says `Unsupported`.
    fn blank(&self) -> Result<PersonDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// Store what changed in `draft`. Keep everything the form does not show. If the person has
    /// changed since `draft.revision`, answer [`ErrorKind::Conflict`](crate::ErrorKind::Conflict)
    /// and change nothing.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn save(&self, _key: &str, _draft: &PersonDraft) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// Make a new person from `draft`; returns the key.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn create(&self, _draft: &PersonDraft) -> Result<String, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// Delete a person. `Some` text: the person is gone, but this must still be said (something that
    /// belonged to them could not be removed).
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the person is still there.
    fn delete(&self, _key: &str) -> Result<Option<String>, SourceError> {
        Err(SourceError::unsupported("this source cannot delete"))
    }
}
