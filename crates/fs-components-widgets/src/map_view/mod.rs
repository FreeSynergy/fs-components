//! `map-view`: a map drawn from what the caller gives it.
//!
//! The caller supplies decoded geometry per tile in a neutral model ([`TileGeometry`]: areas,
//! lines, dots, labels), markers, and a view ([`MapPosition`]: centre and zoom). The widget knows no
//! tile format, no data source and no program's types, has no words and no colours of its own (tones
//! of the theme, or the caller's RGBA), and holds no state: pan and zoom are values
//! ([`MapCommand`]) that keys and fixed buttons produce and the caller applies; gestures come from
//! the frontend (ADR 0037).
//!
//! Pattern: Facade over pure parts — `mercator`, `position` and `tiles` are arithmetic with no
//! graphics, `scene` works out what is where, `draw` is the only code that touches Blinc's canvas,
//! `widget` joins them into an element.

// Pixel arithmetic: every cast is of a small count or of a value already clamped.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod choose;
mod command;
mod draw;
mod look;
mod mercator;
mod model;
mod place;
mod position;
mod scene;
mod tiles;
mod widget;

pub use choose::{AREA_BUDGET, DOT_BUDGET, LINE_BUDGET};
pub use command::{MapCommand, PAN_STEP, ZOOM_STEP, command_for_key};
pub use mercator::{
    MAX_LAT, MAX_ZOOM, MIN_ZOOM, TILE_SIZE, clamp_lat, clamp_zoom, crosses_antimeridian, project,
    span_degrees, unproject, world_size, wrap_lon,
};
pub use model::{Area, Dot, Label, Line, Marker, Paint, TileGeometry, TilePoint, Tone};
pub use position::MapPosition;
pub use scene::{MOST_LABELS, Placed, PlacedLabel, PlacedMarker, Scene};
pub use tiles::{MOST_TILES, NeededTile, TileId, TileSet, needed_tiles, tile_zoom};
pub use widget::{CONTROL, MOST_PICKABLE, MapView, MapViewLabels, OnChange, OnPick};

/// CSS of the map view: only theme variables.
pub const STYLES: &str = r"
.fs-map-controls { z-index: 2; }
.fs-map-control { display: flex; align-items: center; justify-content: center; border-radius: var(--radius-default); background: var(--surface-elevated); color: var(--text-primary); border-width: 1px; border-color: var(--border); cursor: pointer; font-size: var(--text-lg); }
";
