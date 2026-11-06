//! Where the calendar workbench gets its items.
//!
//! Pattern: Strategy + Value objects — the workbench depends on this trait only (an Adapter over a
//! program's core implements it). The trait is synchronous and called on the interface thread, like
//! `DocumentSource` and `PeopleSource`; this is provisional (see DESIGN.md).

use fs_components_widgets::{Day, Moment};

use super::draft::ItemDraft;
use super::model::{CalendarCapabilities, ItemSheet, ItemsRead};
use crate::error::SourceError;

/// A source of things on a calendar.
pub trait CalendarSource: Send + Sync {
    /// What it can do.
    fn capabilities(&self) -> CalendarCapabilities;

    /// The moment now, in the zone the source shows.
    fn now(&self) -> Moment;

    /// The first day of a week: 0 is Monday, 6 is Sunday.
    fn week_start(&self) -> u8 {
        0
    }

    /// What is on the days `first` through `last`, with wall-clock times in the zone the source
    /// shows. Asked for a span of days at a time, never more than a view shows. What cannot be read
    /// goes into [`ItemsRead::problems`] by name; the rest is still shown.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read, if nothing at all can be read.
    fn items(&self, first: Day, last: Day) -> Result<ItemsRead, SourceError>;

    /// One item, to be read.
    ///
    /// # Errors
    /// [`SourceError`]; [`ErrorKind::NotFound`](crate::ErrorKind::NotFound) if there is none.
    fn read(&self, key: &str) -> Result<ItemSheet, SourceError>;

    /// One item, as a form.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot edit says `Unsupported`.
    fn draft(&self, _key: &str) -> Result<ItemDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// A form for a new item on `day`.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot create says `Unsupported`.
    fn blank(&self, _day: Day) -> Result<ItemDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// Store what changed in `draft`. Keep everything the form does not show. If the item has changed
    /// since `draft.revision`, answer [`ErrorKind::Conflict`](crate::ErrorKind::Conflict) and change
    /// nothing. Refuse what cannot be done properly rather than damage the item.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn save(&self, _key: &str, _draft: &ItemDraft) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// Make a new item from `draft`; returns the key.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn create(&self, _draft: &ItemDraft) -> Result<String, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// What a delete of `key` would take with it, for an item whose sheet could not be read.
    fn delete_warning(&self, _key: &str) -> Option<String> {
        None
    }

    /// Delete an item.
    ///
    /// # Errors
    /// [`SourceError`] with the reason.
    fn delete(&self, _key: &str) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot delete"))
    }
}
