//! `fs-components-map`: the component *Karte* (ADR 0066: everything that shows something is a component here).
//!
//! The caller supplies markers ([`MapData`]) and, if it has them, vector tiles as bytes ([`TileBytes`]); this crate knows
//! no program's types, decodes MVT ([`decode_tile`]), draws the view ([`MapView`]) and puts it into the window shell
//! ([`MapSurface`]). It has no words of its own (the caller gives the button signs) and no colours (tones of the theme).
//!
//! Pattern: Facade — this file declares the modules and re-exports. Inside: `mercator`, `position`, `tiles`, `scene`
//! are arithmetic with no graphics; `draw` is the only code that touches Blinc's canvas; `widget` joins them into an
//! element; `mvt` is an Adapter over `mvt-reader`; `surface` is a Strategy consumer (data and tiles come from traits).

// Pixel arithmetic: every cast is of a small count or of a value already clamped.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#![warn(missing_docs)]

mod choose;
mod command;
mod draw;
mod guard;
mod keys;
mod loader;
mod look;
mod mercator;
mod model;
mod mvt;
mod place;
mod position;
mod scene;
mod surface;
mod tiles;
mod widget;

pub use choose::{AREA_BUDGET, DOT_BUDGET, LINE_BUDGET};
pub use command::{MapCommand, PAN_STEP, ZOOM_STEP, command_for_key};
pub use guard::{MAX_FEATURES, MAX_RING_CELLS, MAX_TILE_BYTES};
pub use loader::{Limits, MOST_IN_FLIGHT, MOST_KEPT, MOST_LOADING};
pub use mercator::{
    MAX_LAT, MAX_ZOOM, MIN_ZOOM, TILE_SIZE, clamp_lat, clamp_zoom, crosses_antimeridian, project,
    span_degrees, unproject, world_size, wrap_lon,
};
pub use model::{Area, Dot, Label, Line, Marker, Paint, TileGeometry, TilePoint, Tone};
pub use mvt::{MvtError, decode_tile};
pub use position::MapPosition;
pub use scene::{MOST_LABELS, Placed, PlacedLabel, PlacedMarker, Scene};
pub use surface::{MapData, MapSurface, TileBytes};
pub use tiles::{MOST_TILES, NeededTile, TileId, TileSet, needed_tiles, tile_zoom};
pub use widget::{CONTROL, MOST_PICKABLE, MapView, MapViewLabels, OnChange, OnPick};

/// CSS of the map view: only theme variables.
pub const STYLES: &str = r"
.fs-map-controls { z-index: 2; }
.fs-map-attribution { z-index: 2; padding: 1px 6px; border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-secondary); font-size: var(--text-xs); }
.fs-map-control { display: flex; align-items: center; justify-content: center; border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-primary); border-width: 1px; border-color: var(--border); cursor: pointer; font-size: var(--text-lg); }
";
