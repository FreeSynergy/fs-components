# fs-components-timeline - Design

A horizontal time axis with bars, progress, milestones, dependency arrows, three zoom levels and a line for today (ADR 0066: the program gives data, a trait and words; this crate draws). Entry: `Timeline::new(state, source, say, look).events(events).build(width, height)`.

## Data boundary (stable)

| Type | Role |
|---|---|
| `Item` (key, title, `Kind::Bar{start,end,progress}` or `Kind::Milestone{day}`, `Tone`), `Dependency{from,to}` | plain data; days inclusive, already localised |
| `Day`, `Tone` | re-exported from `fs-components-calendar-grid` (same rev; reuse, not a copy) |
| trait `TimelineSource` | `items()` (one row each) and `dependencies()`; read on every build. `VecSource` for a fixed list |
| trait `TimelineEvents` | `select_item`, `zoomed`; defaults do nothing (`NoEvents`) |
| `Zoom` (Day 40 px, Week 12 px, Month 4 px per day) | state of the host's choice, kept in `TimelineState` |

Texts are translation keys through `Say`: `timeline.zoom.day|week|month`, `timeline.month.1..12`, `timeline.today`. Looks come only from `fs-components-styles` (`Look`; all four styles drawn in a test).

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `model.rs` | Value object | small immutable data |
| `source.rs` | Strategy + Null object | the host owns data and decides what a choice means |
| `state.rs` | State | zoom, selection and keys in one set of transitions |
| `layout.rs`, `route.rs` | none (pure functions) | scale, rectangles, ticks, the path of an arrow; tested without a window |
| `handle.rs`, `scroll.rs` | Facade + Observer | one clonable handle; change state, tell host, redraw the live area only |
| `view/*` | View function / Adapter | `arrows.rs` and `axis.rs` are the only code that talks to Blinc's draw context (Blinc canvas, stroke and fill of paths) |
| `view/keys.rs` | Adapter | copy of a private helper of `fs-components-widgets` (move to a shared crate when one exists) |

## Behaviour

Rows are items in order; a bar is tinted with its progress share filled; a milestone is a diamond with its title. Arrows run from the right edge of one item to the left edge of the next (elbow); when the next item starts earlier they go round through the gap between the rows. The today line is a thin full-height bar when today is on the axis; the axis never grows to reach today (its cost grows with the days drawn), so a today outside it is an edge mark (`tl-today-edge`: a drawn triangle towards today and the word) on a row above the scroll box, so it is at the visible rim at any scroll position. The axis (lines and labels) is one canvas, not a node per tick: ten years at day zoom are 3650 ticks. Up/Down walk the rows (and tell the host), `+`/`-` (also number pad) zoom, a click chooses an item. A change rebuilds the live area only, never per frame.

## Tests

`tests/model.rs`: scale, range, rectangles, progress, ticks per zoom, routes (straight, elbow, round), zoom, keys. `tests/view.rs`: drawn offscreen in all four styles; arrows verified as ink on the canvas against the same plan without dependencies; zoom buttons rescale bars; click and keyboard through Blinc's event router. `tests/view_today.rs`: today as a line or an edge mark (direction and place checked), ten years of items within a time limit, wide milestone titles. `tests/view_scale.rs`: time limits for 120 items and the measurement series (ignored). `tests/view_scroll.rs`: a zoom by key keeps the view on content (left, middle, right; 365 and 1100 days), a zoom change scales the offset. `tests/view_requests.rs`: scroll requests before the build (also when read in between), NaN, a reader's scroll, the wheel. `tests/view_labels.rs`: month labels cut at their tick (pixel comparison).

## Scrolling: what is known to be imperfect

- A scroll asked for (`set_scroll_x`, a zoom change) counts at once but only for 100 ms and only while the view is still where it was. A reader who scrolls back to that place and zooms within the same 100 ms gets the old request's place.
- A request that the view does not draw for more than 100 ms (no frame) is forgotten by the next zoom, which then starts from the view's own offset.
- Building the same handle a second time into a new tree shows the view at 0, but `scroll_x()` still reports the old value (the request record outlives the first tree). Existing behaviour, not fixed.
- A request made before the first build is kept and bounded at the first build, whether it was read in between or not.

## Unsupported (named, not built)

- A month name in a tick too narrow for four letters is left out without a hint (the tick has a line, the days still name the place).
- Dragging or resizing bars, creating dependencies by hand (read-only plan; the host edits).
- Start-to-start / finish-to-finish dependency kinds, critical path, lags (only finish to start).
- Sticky axis and sticky title column (one scroll box holds all; titles sit inside the bars), grouped/collapsible rows, baselines.
- Zoom to fit, pinch zoom. (`set_scroll_x` / `scroll_x` / `max_scroll_x` exist; a zoom change keeps the day at the left of the view and keeps the offset inside the content.)
- Many rows: the redraw cost grows faster than the number of items (120 items 83 ms, 480 items 856 ms in release); filed in the Blinc tracker, no virtualising here.
- Weekend shading, working-day calendars, hours and minutes (days are the unit).
- Locale formats: day numbers and years are plain numbers; month names come from `Say`.
- Arrows are not hit-testable (the canvas does not take pointer events); no highlight of the arrows of the chosen item.
- Very long titles are clipped inside the bar, not shown outside it.
