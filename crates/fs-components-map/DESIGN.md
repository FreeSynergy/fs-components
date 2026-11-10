# fs-components-map - Design

The component *Karte* of ADR 0066: everything that shows a map lives here, no program draws one. A program gives markers ([`MapData`]) and, if it has them, vector tiles as bytes ([`TileBytes`]); the crate decodes (MVT), plans, draws and puts the view into the window shell as a `Surface`.

Moved here from `fs-components-widgets/src/map_view` (branch `maps-gui`, wip, 2026-11-06) and checked: its 18 tests pass, now in this crate.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `mercator.rs`, `position.rs`, `tiles.rs`, `command.rs` | none (pure arithmetic) | Web-Mercator maths, no graphics, tested with extreme values |
| `model.rs` | Value objects | neutral geometry per tile; no tile format, no colours of its own |
| `choose.rs`, `place.rs`, `scene.rs` | Specification / Value | what is drawn within budgets, what is where |
| `draw.rs` | Adapter | the only code that touches Blinc's canvas |
| `widget.rs` `MapView` | Adapter + Facade | position, tiles, markers in; element out; holds no state |
| `keys.rs` | Adapter + Guard | same rule as `fs-components-widgets::keys` (its binder is not public) |
| `mvt.rs` | Adapter + Strategy | `mvt-reader` behind `decode_tile`; the look of a layer by its name |
| `surface.rs` `MapSurface` | Strategy consumer + Observer | `MapData`, `TileBytes` from the caller; tiles on worker threads; `Live` draws again |

## Unsupported (named, not faked)

* No mouse drag or wheel: pan and zoom are the buttons and the keys (gestures come from the frontend, ADR 0037).
* The look of layers is a table of layer names (Protomaps/Shortbread); no style files (MapLibre style) yet. Labels are the `name` property only.
* No raster tiles, no clustering of markers.
