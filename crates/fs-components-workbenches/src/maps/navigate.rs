//! Going elsewhere on the map.
//!
//! Pattern: State — each transition is one method. Moving the map leaves an edit alone, because its
//! form is not part of the map; choosing another place does not, and is refused while there are unsaved
//! changes. Only an explicit act moves the map to a place (ADR 0029): choosing a row never does.

use fs_components_widgets::map_view::{MAX_ZOOM, MapCommand, MapPosition, clamp_zoom};

use super::labels::MapLabels;
use super::shown::{Outcome, Shown};
use super::state::{MapState, OVERZOOM};

/// The zoom a place is shown at, at least.
const SHOW_ZOOM: f64 = 14.0;

impl MapState {
    /// The finest zoom: a little beyond the source's finest tiles.
    pub fn max_zoom(&self) -> f64 {
        self.range
            .map_or(MAX_ZOOM, |r| (f64::from(r.max_zoom) + OVERZOOM).min(MAX_ZOOM))
    }

    /// The coarsest zoom: a little beyond the source's coarsest tiles.
    pub fn min_zoom(&self) -> f64 {
        self.range
            .map_or(0.0, |r| (f64::from(r.min_zoom) - 2.0).max(0.0))
    }

    /// `position` within the zoom the source supports.
    pub(super) fn limited(&self, position: MapPosition) -> MapPosition {
        position.with_zoom(clamp_zoom(position.zoom()).clamp(self.min_zoom(), self.max_zoom()))
    }

    /// Show this view (the zoom is kept within what the source supports).
    pub fn set_position(&mut self, position: MapPosition) {
        self.position = self.limited(position);
    }

    /// Do what a key, a button or a gesture of the frontend asked for, on a map `width` x `height`.
    pub fn apply(&mut self, command: MapCommand, width: f64, height: f64) {
        self.position = self.limited(command.apply(self.position, width, height));
    }

    /// Show the part of the world around a longitude and latitude, keeping the zoom.
    pub fn centre_on(&mut self, lon: f64, lat: f64) {
        self.position = self.limited(self.position.centred_on(lon, lat));
    }

    /// The person typed this for "go to"; it does not move anything yet.
    pub fn set_go_text(&mut self, text: &str) {
        self.go_text = text.to_string();
    }

    /// Go to the position `text` means (a `geo:` URI, if the source reads one). Says why if it cannot.
    pub fn go_to(&mut self, text: &str) {
        match self.source.locate(text.trim()) {
            Ok((lon, lat)) => {
                let zoom = self.position.zoom().max(SHOW_ZOOM.min(self.max_zoom()));
                self.position = self.limited(MapPosition::new(lon, lat, zoom));
                let at = format!("{lat:.5}, {lon:.5}");
                self.notice = Some(MapLabels::fill(&self.labels.went_to, &at));
            }
            Err(e) => {
                self.notice = Some(MapLabels::fill(&self.labels.cannot_go, &e.message));
            }
        }
    }

    /// Go to what was typed.
    pub fn go_to_typed(&mut self) {
        let text = self.go_text.clone();
        self.go_to(&text);
    }

    /// Show the place chosen, on the map: the one explicit act that moves the map to a place. A place
    /// without a position says so. Marked places are moved to the same way and no other.
    pub fn show_on_map(&mut self) {
        let at = match &self.shown {
            Shown::Place { sheet, .. } => sheet.at,
            _ => return,
        };
        match at {
            Some((lon, lat)) => {
                let zoom = self.position.zoom().max(SHOW_ZOOM.min(self.max_zoom()));
                self.position = self.limited(MapPosition::new(lon, lat, zoom));
                self.side = false;
                self.notice = None;
            }
            None => self.notice = Some(self.labels.no_position.clone()),
        }
    }

    /// Read the tiles again, those that could not be read before too.
    pub fn reload_tiles(&mut self) {
        self.tiles.forget_failures();
    }

    /// Show the places and not the map (the narrow layout), or the other way round.
    pub fn show_side(&mut self, side: bool) {
        self.side = side;
    }

    /// Show this place, read again also if it is the one open: it may have changed elsewhere. With
    /// unsaved changes this is refused and says why. In the compact variant choosing a place is also
    /// picking it.
    pub fn select(&mut self, key: &str) -> Outcome {
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.show(key);
        self.pick_selected();
        Outcome::Done
    }

    /// Show nothing. With unsaved changes this is refused and says why.
    pub fn close(&mut self) -> Outcome {
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.delete_asked = None;
        self.notice = None;
        self.shown = Shown::Nothing;
        Outcome::Done
    }
}
