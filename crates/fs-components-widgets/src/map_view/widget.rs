//! The map as a widget: a canvas with the scene, fixed buttons, and the keys.
//!
//! Pattern: Adapter + Facade — `MapView` holds what the caller gave (a position, tiles, markers) and
//! returns a Blinc element of the size asked for. It holds no state of its own: every pan and zoom is
//! reported as the new [`MapPosition`] and the caller draws again. The buttons and the markers are
//! plain elements, so they can be reached by Tab and pressed by Enter; the canvas takes no click.

use std::rc::Rc;
use std::sync::Arc;

use blinc_layout::canvas::canvas;
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::command::{MapCommand, PAN_STEP, ZOOM_STEP, command_for_key};
use super::draw::{PIN, draw};
use super::model::{Marker, TileGeometry};
use super::position::MapPosition;
use super::scene::Scene;
use super::tiles::NeededTile;
use crate::Element;
use crate::keys::bind_keys;

/// Side of a button of the view in pixels (at least what a finger needs).
pub const CONTROL: f32 = 44.0;
/// The most markers that are elements of their own (reachable by Tab); the rest are only drawn.
pub const MOST_PICKABLE: usize = 48;

/// Receives the new position after a key or a button.
pub type OnChange = Arc<dyn Fn(MapPosition) + Send + Sync>;
/// Receives the key of the marker that was chosen.
pub type OnPick = Arc<dyn Fn(&str) + Send + Sync>;

/// The signs on the buttons; the caller's words (translation is the caller's).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapViewLabels {
    /// Zoom in.
    pub zoom_in: String,
    /// Zoom out.
    pub zoom_out: String,
    /// Pan west.
    pub west: String,
    /// Pan east.
    pub east: String,
    /// Pan north.
    pub north: String,
    /// Pan south.
    pub south: String,
}

impl Default for MapViewLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            zoom_in: s("+"),
            zoom_out: s("-"),
            west: s("<"),
            east: s(">"),
            north: s("^"),
            south: s("v"),
        }
    }
}

/// A map ready to be placed in a layout.
pub struct MapView {
    name: String,
    position: MapPosition,
    labels: MapViewLabels,
    tiles: Vec<(NeededTile, Arc<TileGeometry>)>,
    markers: Vec<Marker>,
    on_change: Option<OnChange>,
    on_pick: Option<OnPick>,
}

impl MapView {
    /// A map named `name` (element ids begin with it) at `position`.
    pub fn new(name: impl Into<String>, position: MapPosition, labels: MapViewLabels) -> Self {
        Self {
            name: name.into(),
            position,
            labels,
            tiles: Vec::new(),
            markers: Vec::new(),
            on_change: None,
            on_pick: None,
        }
    }

    /// The tiles to draw, with their rectangles (`needed_tiles` says which are needed).
    #[must_use]
    pub fn tiles(mut self, tiles: Vec<(NeededTile, Arc<TileGeometry>)>) -> Self {
        self.tiles = tiles;
        self
    }

    /// The markers.
    #[must_use]
    pub fn markers(mut self, markers: Vec<Marker>) -> Self {
        self.markers = markers;
        self
    }

    /// Called with the new position after a key or a button.
    #[must_use]
    pub fn on_change(mut self, on_change: OnChange) -> Self {
        self.on_change = Some(on_change);
        self
    }

    /// Called with the key of a marker that was chosen.
    #[must_use]
    pub fn on_pick(mut self, on_pick: OnPick) -> Self {
        self.on_pick = Some(on_pick);
        self
    }

    /// The plan of the picture at a size; what `build` draws.
    pub fn scene(&self, width: f32, height: f32) -> Scene {
        Scene::new(
            self.position,
            width,
            height,
            self.tiles.clone(),
            &self.markers,
        )
    }

    /// The element, `width` x `height` pixels. A size of 0 gives an empty element, not an error.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let (width, height) = (width.max(0.0), height.max(0.0));
        let scene = Rc::new(self.scene(width, height));
        let drawn = Rc::clone(&scene);
        let (position, change) = (self.position, self.on_change.clone());
        let keys = Arc::new(move |event| {
            let (Some(command), Some(change)) = (command_for_key(event), change.as_ref()) else {
                return false;
            };
            change(command.apply(position, f64::from(width), f64::from(height)));
            true
        });
        let root = div()
            .id(self.name.clone())
            .relative()
            .w(width)
            .h(height)
            .overflow_clip();
        let root = bind_keys(root, keys).child(
            canvas(move |ctx, _| draw(ctx, &drawn))
                .redraw_while(|| false)
                .w(width)
                .h(height),
        );
        let mut root = root;
        for placed in scene.markers.iter().take(MOST_PICKABLE) {
            root = root.child(self.pickable(placed));
        }
        root.child(self.controls(width))
    }

    fn pickable(&self, placed: &super::scene::PlacedMarker) -> Div {
        let side = PIN * 2.0 + 8.0;
        let slot = div()
            .id(format!("{}-marker-{}", self.name, placed.marker.key))
            .absolute()
            .left(placed.x - side / 2.0)
            .top(placed.y - side / 2.0)
            .w(side)
            .h(side)
            .attr("tabindex", "0");
        match self.on_pick.clone() {
            Some(pick) => {
                let key = placed.marker.key.clone();
                slot.on_click(move |_| pick(&key))
            }
            None => slot,
        }
    }

    fn controls(&self, width: f32) -> Div {
        let button = |id: &str, label: &str, command: MapCommand| {
            let (position, change) = (self.position, self.on_change.clone());
            div()
                .id(format!("{}-{id}", self.name))
                .class("fs-map-control")
                .w(CONTROL)
                .h(CONTROL)
                .flex_shrink_0()
                .attr("tabindex", "0")
                .on_click(move |_| {
                    if let Some(change) = &change {
                        change(command.apply(position, 0.0, 0.0));
                    }
                })
                .child(text(label).no_wrap().no_cursor())
        };
        let l = &self.labels;
        let row = || div().flex_row().gap_px(2.0);
        let pan = div()
            .flex_col()
            .gap_px(2.0)
            .child(row().child(div().w(CONTROL).h(CONTROL)).child(button(
                "north",
                &l.north,
                MapCommand::Pan(0.0, -PAN_STEP),
            )))
            .child(
                row()
                    .child(button("west", &l.west, MapCommand::Pan(-PAN_STEP, 0.0)))
                    .child(button("south", &l.south, MapCommand::Pan(0.0, PAN_STEP)))
                    .child(button("east", &l.east, MapCommand::Pan(PAN_STEP, 0.0))),
            );
        let zoom = div()
            .flex_col()
            .gap_px(2.0)
            .child(button("zoom-in", &l.zoom_in, MapCommand::Zoom(ZOOM_STEP)))
            .child(button(
                "zoom-out",
                &l.zoom_out,
                MapCommand::Zoom(-ZOOM_STEP),
            ));
        div()
            .class("fs-map-controls")
            .absolute()
            .left((width - CONTROL * 4.0 - 24.0).max(0.0))
            .top(8.0)
            .flex_row()
            .gap_px(8.0)
            .child(zoom)
            .child(pan)
    }
}
