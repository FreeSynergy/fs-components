# fs-components-map - Design

The component *Karte* of ADR 0066: everything that shows a map lives here, no program draws one. A program gives markers ([`MapData`]) and, if it has them, vector tiles as bytes ([`TileBytes`]); the crate decodes (MVT), plans, draws and puts the view into the window shell as a `Surface`.

Moved here from `fs-components-widgets/src/map_view` (branch `maps-gui`, wip, 2026-11-06) and checked: its 18 tests pass, now in this crate. The crate has 37 tests now (math 13, ui 5, mvt 3, guard 7, loading 9).

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
| `surface.rs` `MapSurface` | Strategy consumer + Observer | `MapData`, `TileBytes` from the caller; `Live` draws again |
| `loader.rs` | Object pool with a lease + State per tile | places for fetching, time limits, tiles no longer wanted, failures, what is kept |
| `guard.rs` | Guard | a tile is scanned (bytes, features, rings x numbers) before `mvt-reader` decodes it |

## Tiles: limits (review of 2026-11-10)

* **Before decoding** (`guard.rs`): `mvt-reader` 2.5.0 (`parse_geometry`) reserves a buffer of the whole geometry of a feature (`G` numbers, 8 bytes a point) at the start, at every `ClosePath` and at every `MoveTo` of a line, so a feature costs `(commands + 1) x G` points, and what a layer decodes stays until the layer is done: the cost of a tile is the **sum over all its features and layers**. Running out of memory is no panic that `catch_unwind` catches. A tile is scanned in its raw protobuf first, with no allocation: at most `MAX_TILE_BYTES` (4 MiB), `MAX_FEATURES` (100 000) and `MAX_TILE_CELLS` (8 million points, 64 MiB) summed over the tile (`ClosePath` and `MoveTo` both counted). Over a limit: `MvtError`, no decode. Evidence, with no memory used: the real bomb (`LineTo` + `ClosePath` x 250 000, 1 MB) is one feature of 1 000 003 numbers with 250 000 `ClosePath`, so `mvt-reader` would reserve 250 000 buffers of 1 000 003 points, 2 TB; the estimate says 2.5e11 points against 8e6. Tests: that bomb, a `MoveTo` bomb, and 400 features that are each fine alone and over the limit together.
* **One extent per layer**: the result has the extent of the widest layer and the coordinates of a layer with another extent are scaled to it.
* **Loading** (`loader.rs`): at most 4 fetches at once and 8 workers alive; a fetch over `Limits::load_timeout` (20 s) gives up its place and its tile is `Failed`; a tile that the view no longer needs gives up its place at the next draw; `Ok(None)` is kept for good, an error, a panic or bytes that do not decode are `Failed` and asked for again after `Limits::retry_after` (30 s); at most `Limits::most_kept` (256) tiles are kept, the oldest go. A watchdog thread of the loader (it ends with it) applies the time limits on time, with no draw needed, and has the view draw again. A fetch that hangs cannot be stopped (no thread can), so its late answer is thrown away and `MOST_IN_FLIGHT` bounds the threads: **when 8 workers hang, no tile is fetched until one comes back.** `MapSurface::source_stalled()` says so (the map has no words of its own: the program shows "the source does not answer" while it is true); `tiles_loading()` counts the places held. The thread is made with `thread::Builder`: no thread to be had is a failed tile, not a panic. `MapSurface::with_limits` sets other limits (tests).
* **Attribution**: `TileBytes::attribution()` gives who made the data (for OpenStreetMap "© OpenStreetMap contributors"); the map shows it in the lower right corner, id `<name>-attribution`, whenever it is `Some` (a program that has tiles gives it from the data it has; `MapView::attribution` does the same for a program that draws by itself).

## Unsupported (named, not faked)

* No mouse drag or wheel: pan and zoom are the buttons and the keys (gestures come from the frontend, ADR 0037).
* The look of layers is a table of layer names (Protomaps/Shortbread); no style files (MapLibre style) yet. Labels are the `name` property only.
* No raster tiles, no clustering of markers.
