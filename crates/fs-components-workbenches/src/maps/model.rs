//! What the maps workbench and its source talk about.
//!
//! Pattern: Value objects — rows, sheets and capabilities, in the source's words. Nothing here knows
//! what a place is made of, and nothing is a type of any program.

use fs_components_widgets::map_view::{MapPosition, TileId};

/// What a source can do; an action it cannot do is not offered.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCapabilities {
    /// Change a place.
    pub edit: bool,
    /// Make a new place.
    pub create: bool,
    /// Delete a place.
    pub delete: bool,
}

impl MapCapabilities {
    /// Reading only.
    pub fn read_only() -> Self {
        Self::default()
    }

    /// Everything.
    pub fn all() -> Self {
        Self {
            edit: true,
            create: true,
            delete: true,
        }
    }
}

/// The tiles a source has: zoom range and where they lie.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileRange {
    /// The lowest zoom with tiles.
    pub min_zoom: u8,
    /// The highest zoom with tiles.
    pub max_zoom: u8,
    /// West, south, east, north in degrees, if the source knows (west > east crosses the
    /// antimeridian).
    pub bounds: Option<(f64, f64, f64, f64)>,
}

impl TileRange {
    /// A range of zooms.
    pub fn new(min_zoom: u8, max_zoom: u8) -> Self {
        Self {
            min_zoom,
            max_zoom,
            bounds: None,
        }
    }

    /// With the area the tiles cover.
    #[must_use]
    pub fn with_bounds(mut self, west: f64, south: f64, east: f64, north: f64) -> Self {
        self.bounds = Some((west, south, east, north));
        self
    }
}

/// What the person asks the list for.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaceQuery {
    /// Part of a name; empty for all.
    pub term: String,
    /// Places to skip.
    pub offset: usize,
    /// At most this many.
    pub limit: usize,
}

/// One place of the list.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct PlaceRow {
    /// What the source calls it.
    pub key: String,
    /// What to call it.
    pub name: String,
    /// Where it is, if it is a point or has a middle.
    pub at: Option<(f64, f64)>,
    /// Short markings that must be seen wherever it appears, in the source's words.
    pub marks: Vec<String>,
}

impl PlaceRow {
    /// A row.
    pub fn new(key: impl Into<String>, name: impl Into<String>, at: Option<(f64, f64)>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            at,
            marks: Vec::new(),
        }
    }

    /// With these markings.
    #[must_use]
    pub fn marked(mut self, marks: Vec<String>) -> Self {
        self.marks = marks;
        self
    }
}

/// A place that could not be shown, named so that the person can deal with it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceProblem {
    /// What to open to deal with it.
    pub key: Option<String>,
    /// What is wrong, in the source's words. Of something marked it does not give the name.
    pub text: String,
    /// Short markings to be seen with it.
    pub marks: Vec<String>,
}

impl PlaceProblem {
    /// A problem with a place that can be opened.
    pub fn of(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            text: text.into(),
            marks: Vec::new(),
        }
    }
}

/// A page of the list: what could be read, and what could not. One bad place costs itself only.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlacesRead {
    /// The places.
    pub rows: Vec<PlaceRow>,
    /// What could not be shown.
    pub problems: Vec<PlaceProblem>,
}

impl PlacesRead {
    /// All of them could be read.
    pub fn all(rows: Vec<PlaceRow>) -> Self {
        Self {
            rows,
            problems: Vec::new(),
        }
    }
}

/// One labelled value of a place.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetField {
    /// What the value is.
    pub label: String,
    /// The value.
    pub value: String,
}

impl SheetField {
    /// A field.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Fields that belong together, under a heading.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetSection {
    /// The heading; empty shows none.
    pub title: String,
    /// The fields.
    pub fields: Vec<SheetField>,
}

impl SheetSection {
    /// A section.
    pub fn new(title: impl Into<String>, fields: Vec<SheetField>) -> Self {
        Self {
            title: title.into(),
            fields,
        }
    }
}

/// One place, to be read.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct PlaceSheet {
    /// What to call it.
    pub title: String,
    /// Short markings that must be seen at once.
    pub marks: Vec<String>,
    /// A sentence to read before acting on it.
    pub note: Option<String>,
    /// The details.
    pub sections: Vec<SheetSection>,
    /// Where it is, if it can be shown on the map.
    pub at: Option<(f64, f64)>,
    /// Can this one be changed here?
    pub can_edit: bool,
    /// Can this one be deleted here?
    pub can_delete: bool,
    /// What a delete would take with it; added to the question.
    pub delete_warning: Option<String>,
}

impl PlaceSheet {
    /// A sheet of a place that can be changed and deleted.
    pub fn new(title: impl Into<String>, sections: Vec<SheetSection>) -> Self {
        Self {
            title: title.into(),
            marks: Vec::new(),
            note: None,
            sections,
            at: None,
            can_edit: true,
            can_delete: true,
            delete_warning: None,
        }
    }
}

/// What the workbench is showing, for the program around it (ADR 0056 point 4): a plain value.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct MapContext {
    /// The place open or being edited; `None` while a new one is being made or none is open.
    pub place: Option<String>,
    /// Centre and zoom of the view.
    pub position: MapPosition,
}

/// What the compact variant hands back (ADR 0037): a value, never a window function.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum Pick {
    /// A place was chosen. Nothing but its key is handed out: of a marked place no name and no
    /// position leave with it.
    Place {
        /// The key.
        key: String,
        /// Whether it is marked as needing care (ADR 0029).
        marked: bool,
    },
    /// A point was chosen: the centre of the view.
    Point {
        /// Longitude.
        lon: f64,
        /// Latitude.
        lat: f64,
    },
}

impl Pick {
    /// The `geo:` URI (RFC 5870) of a picked point, to six decimals; `None` for a place.
    pub fn geo_uri(&self) -> Option<String> {
        match self {
            Self::Point { lon, lat } => Some(format!("geo:{lat:.6},{lon:.6}")),
            Self::Place { .. } => None,
        }
    }
}

/// A tile that could not be read, named.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileProblem {
    /// Which tile.
    pub id: TileId,
    /// Why not.
    pub why: String,
}
