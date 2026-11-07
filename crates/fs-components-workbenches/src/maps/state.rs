//! The state of the maps workbench, testable without a window.
//!
//! Pattern: State machine over a Strategy (`MapSource`): every transition is one method; the view only
//! reads. Moving the map is in `navigate.rs`, reading what is open in `shown_read.rs`, editing,
//! saving and deleting in `edit.rs`, `save.rs` and `delete.rs`, the picture in `picture.rs`, the compact
//! variant's value in `pick.rs`. `Clone` is a cheap snapshot (the rows are shared, and so is the tile
//! cache), so a view draws from a copy and holds no lock while it builds elements. Nothing here opens a
//! place or moves the map to one on its own: only a person's choice does (ADR 0029).

use std::sync::Arc;

use fs_components_widgets::map_view::{MapPosition, MapView};

use super::edit::PlaceEdit;
use super::labels::MapLabels;
use super::model::{MapCapabilities, PlaceProblem, PlaceQuery, PlaceRow, Pick, TileRange};
use super::shown::{Shown, Variant};
use super::source::MapSource;
use super::tile_store::TileStore;
use crate::error::SourceError;

/// Rows read at a time.
pub const PAGE: usize = 200;
/// How many zoom steps are allowed beyond the source's finest tiles (they are drawn larger).
pub const OVERZOOM: f64 = 4.0;

/// View, places, the place shown, and the edit in progress.
#[derive(Clone)]
pub struct MapState {
    pub(super) source: Arc<dyn MapSource>,
    pub(super) labels: Arc<MapLabels>,
    pub(super) capabilities: MapCapabilities,
    pub(super) variant: Variant,
    pub(super) range: Option<TileRange>,
    pub(super) position: MapPosition,
    pub(super) tiles: TileStore,
    pub(super) rows: Arc<Vec<PlaceRow>>,
    pub(super) more: bool,
    pub(super) term: String,
    pub(super) list_generation: u64,
    pub(super) list_problem: Option<String>,
    pub(super) problems: Arc<Vec<PlaceProblem>>,
    pub(super) shown: Shown,
    pub(super) notice: Option<String>,
    pub(super) editing: Option<PlaceEdit>,
    pub(super) edit_counter: u64,
    pub(super) delete_asked: Option<String>,
    pub(super) go_text: String,
    pub(super) picked: Option<Pick>,
    pub(super) side: bool,
}

impl MapState {
    /// The workbench over `source`. Nothing is opened and the map is where the source says, or
    /// around the tiles it has, or the whole world.
    pub fn new(source: Arc<dyn MapSource>, labels: MapLabels, variant: Variant) -> Self {
        let capabilities = match variant {
            Variant::Compact => MapCapabilities::read_only(),
            _ => source.capabilities(),
        };
        let range = source.tile_range();
        let start = source.start().or_else(|| {
            let bounds = range.and_then(|r| r.bounds)?;
            Some(MapPosition::fitting(bounds, 800.0, 600.0, 18.0))
        });
        let mut state = Self {
            source,
            labels: Arc::new(labels),
            capabilities,
            variant,
            range,
            position: MapPosition::world(),
            tiles: TileStore::default(),
            rows: Arc::new(Vec::new()),
            more: false,
            term: String::new(),
            list_generation: 0,
            list_problem: None,
            problems: Arc::new(Vec::new()),
            shown: Shown::Nothing,
            notice: None,
            editing: None,
            edit_counter: 0,
            delete_asked: None,
            go_text: String::new(),
            picked: None,
            side: false,
        };
        state.position = state.limited(start.unwrap_or_else(MapPosition::world));
        state.reload_places();
        state
    }

    /// The words.
    pub fn labels(&self) -> &MapLabels {
        &self.labels
    }

    /// Full or compact.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// Centre and zoom.
    pub fn position(&self) -> MapPosition {
        self.position
    }

    /// The places loaded so far.
    pub fn rows(&self) -> &Arc<Vec<PlaceRow>> {
        &self.rows
    }

    /// Are there more places than are loaded?
    pub fn has_more(&self) -> bool {
        self.more
    }

    /// The search text.
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Counts up whenever another list takes the place of this one: its scroll position is void.
    pub fn list_generation(&self) -> u64 {
        self.list_generation
    }

    /// Why the list failed, if it did.
    pub fn list_problem(&self) -> Option<&str> {
        self.list_problem.as_deref()
    }

    /// Places that could not be shown, by name; the rest are.
    pub fn problems(&self) -> &Arc<Vec<PlaceProblem>> {
        &self.problems
    }

    /// What the detail area shows.
    pub fn shown(&self) -> &Shown {
        &self.shown
    }

    /// The key of the place selected, if any.
    pub fn selected(&self) -> Option<&str> {
        match &self.shown {
            Shown::Place { key, .. } | Shown::Failed { key, .. } => Some(key),
            Shown::Nothing => None,
        }
    }

    /// A short message for the person, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// The tiles the source has.
    pub fn tile_range(&self) -> Option<TileRange> {
        self.range
    }

    /// Where the narrow layout is: showing the places and not the map.
    pub fn showing_side(&self) -> bool {
        self.side
    }

    /// The text typed for "go to".
    pub fn go_text(&self) -> &str {
        &self.go_text
    }

    /// Search; the list starts over. It does not leave the place shown, so it works during an edit.
    pub fn search(&mut self, term: &str) {
        let term = term.trim();
        if term == self.term {
            return;
        }
        self.term = term.to_string();
        self.list_generation += 1;
        self.reload_count(PAGE);
    }

    /// Load the next places.
    pub fn load_more(&mut self) {
        if !self.more {
            return;
        }
        match self.read_rows(self.rows.len(), PAGE) {
            Ok((read, more)) => {
                let mut all = self.rows.as_ref().clone();
                all.extend(read.rows);
                let mut problems = self.problems.as_ref().clone();
                problems.extend(read.problems);
                self.rows = Arc::new(all);
                self.problems = Arc::new(problems);
                self.more = more;
                self.list_problem = None;
            }
            Err(e) => self.list_failed(&e),
        }
    }

    /// Read the places loaded so far again (at least a page), as the list has changed.
    pub(super) fn reload_places(&mut self) {
        self.reload_count(self.rows.len());
    }

    fn reload_count(&mut self, count: usize) {
        match self.read_rows(0, count.max(PAGE)) {
            Ok((read, more)) => {
                self.rows = Arc::new(read.rows);
                self.problems = Arc::new(read.problems);
                self.more = more;
                self.list_problem = None;
            }
            Err(e) => {
                self.rows = Arc::new(Vec::new());
                self.problems = Arc::new(Vec::new());
                self.more = false;
                self.list_failed(&e);
            }
        }
    }

    /// `limit` places from `offset`, and whether there are more: one more is asked for. The count is of
    /// everything the source answered, rows and problems together, so a page of bad places is a page.
    fn read_rows(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(super::model::PlacesRead, bool), SourceError> {
        let query = PlaceQuery {
            term: self.term.clone(),
            offset,
            limit: limit + 1,
        };
        let mut read = self.source.places(&query)?;
        let more = read.rows.len() + read.problems.len() > limit;
        if read.rows.len() > limit {
            read.rows.truncate(limit);
        }
        Ok((read, more))
    }

    fn list_failed(&mut self, e: &SourceError) {
        self.list_problem = Some(MapLabels::fill(&self.labels.list_failed, &e.message));
    }

    /// The map as a widget over the tiles the view needs, loaded now through the cache: this is where a
    /// slow source blocks the interface thread. `build` of the widget calls it.
    pub fn view(&self, name: &str, width: f32, height: f32) -> (MapView, super::picture::Picture) {
        super::picture::view(self, name, width, height)
    }
}
