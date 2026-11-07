//! Where the maps workbench gets its tiles and its places.
//!
//! Pattern: Strategy + Value objects — the workbench depends on this trait only (an Adapter over a
//! program's core implements it). The trait is synchronous and called on the interface thread, like
//! `CalendarSource`; this is provisional (see DESIGN.md): a slow source freezes the window while it
//! works. Tiles arrive already decoded into the neutral geometry of the `map-view` widget: decoding a
//! tile format is the source's business, not the workbench's.

use std::sync::Arc;

use fs_components_widgets::map_view::{MapPosition, TileGeometry, TileId};

use super::draft::PlaceDraft;
use super::model::{MapCapabilities, PlaceQuery, PlaceSheet, PlacesRead, TileRange};
use crate::error::SourceError;

/// A source of a map and of places on it.
pub trait MapSource: Send + Sync {
    /// What it can do.
    fn capabilities(&self) -> MapCapabilities;

    /// The tiles it has; `None` if it has none (the map is then the empty background, places
    /// still show).
    fn tile_range(&self) -> Option<TileRange> {
        None
    }

    /// One tile, decoded. `Ok(None)` means there is no such tile, which is not an error and is not
    /// shown as one.
    ///
    /// # Errors
    /// [`SourceError`] naming what is wrong with this tile; the others are still shown.
    fn tile(&self, _id: TileId) -> Result<Option<Arc<TileGeometry>>, SourceError> {
        Ok(None)
    }

    /// The view to start with, if the source has an opinion.
    fn start(&self) -> Option<MapPosition> {
        None
    }

    /// Places whose name contains the query's term (empty = all), in the source's order.
    ///
    /// # Errors
    /// [`SourceError`] with a reason a person can read, if nothing at all can be read.
    fn places(&self, query: &PlaceQuery) -> Result<PlacesRead, SourceError>;

    /// One place, to be read.
    ///
    /// # Errors
    /// [`SourceError`]; [`ErrorKind::NotFound`](crate::ErrorKind::NotFound) if there is none.
    fn read(&self, key: &str) -> Result<PlaceSheet, SourceError>;

    /// The longitude and latitude a text means, for "go to": a `geo:` URI, for one.
    ///
    /// # Errors
    /// [`SourceError`] saying what is wrong with the text.
    fn locate(&self, _text: &str) -> Result<(f64, f64), SourceError> {
        Err(SourceError::unsupported("this source cannot find a position"))
    }

    /// One place, as a form.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot edit says `Unsupported`.
    fn draft(&self, _key: &str) -> Result<PlaceDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// A form for a new place at `lon`, `lat`.
    ///
    /// # Errors
    /// [`SourceError`]; a source that cannot create says `Unsupported`.
    fn blank(&self, _lon: f64, _lat: f64) -> Result<PlaceDraft, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// Store what changed in `draft`. Keep everything the form does not show, and never take a
    /// marking away. If the place has changed since `draft.revision`, answer
    /// [`ErrorKind::Conflict`](crate::ErrorKind::Conflict) and change nothing.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn save(&self, _key: &str, _draft: &PlaceDraft) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot edit"))
    }

    /// Make a new place from `draft`; returns the key.
    ///
    /// # Errors
    /// [`SourceError`] with the reason; the workbench keeps the draft.
    fn create(&self, _draft: &PlaceDraft) -> Result<String, SourceError> {
        Err(SourceError::unsupported("this source cannot create"))
    }

    /// What a delete of `key` would take with it, for a place whose sheet could not be read.
    fn delete_warning(&self, _key: &str) -> Option<String> {
        None
    }

    /// Delete a place.
    ///
    /// # Errors
    /// [`SourceError`] with the reason.
    fn delete(&self, _key: &str) -> Result<(), SourceError> {
        Err(SourceError::unsupported("this source cannot delete"))
    }
}
