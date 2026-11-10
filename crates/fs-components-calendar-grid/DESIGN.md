# fs-components-calendar-grid - Design

Month grid, week grid and day column (time axis) of a calendar, as one component (ADR 0066: the program gives data, a trait and words; this crate draws). Entry: `CalendarGrid::new(state, source, say, look).events(..).build(width, height)`.

## Data boundary (stable)

| Type | Role |
|---|---|
| `GridItem`, `Tone`, `Day`, `Moment` | plain data, already localised (time zone is the caller's job); no JSCalendar type. All-day items cover `start.day..=end.day`; timed items end exclusive. |
| trait `GridSource` | where items of a range come from (`VecSource` for a fixed list); asked on every build. |
| trait `GridEvents` | `select_slot`, `select_item`, `navigated`; all default to nothing (`NoEvents`). |
| `Nav`, `Mode`, `Slot` | navigation message, month/week/day, a chosen day (+ minute on a time axis). |

Texts are translation keys through `Say` (`fs-components-fields`): `calendar.previous|next|today`, `calendar.mode.month|week|day`, `calendar.month.1..12`, `calendar.weekday.0..6` (0 = Monday). Looks come only from `fs-components-styles` through `Look` (Admin/Fiori, Simple, Apple, Windows all drawn in a test, no extra code).

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `day.rs`, `model.rs` | Value object | dates and items compare by value |
| `source.rs` | Strategy + Null object | the host owns data and decides what a choice means |
| `state.rs` | State | one set of transitions for buttons, clicks and keys |
| `layout.rs`, `axis.rs`, `spans.rs` | none (pure functions) | days of a range, side-by-side lanes, bars over several days, minutes to pixels; tested without a window |
| `handle.rs` | Facade + Observer | one clonable handle; a change updates the state, tells the host, redraws the live area only |
| `view/*` | View function / Adapter | state and measures in, Blinc elements out |
| `view/keys.rs` | Adapter | Blinc key events to `Key` (copy of a private helper of `fs-components-widgets`; move to a shared crate when one exists) |

## Behaviour

- Month: six rows, bars across the days an item covers (cut at week ends), "+N" when the cell is full. Week/day: header, all-day row (all-day and multi-day items), scrolling axis of 24 hours, overlapping items side by side, now line, marked slot, today mark (accent disc).
- Selection: one thing is selected (slot or item). Click on cell/column/bar, arrow keys (month: day, week; timed: day, slot), PageUp/PageDown navigate, Home is today, Enter/Space choose the slot.
- Updates: the toolbar and body live in one `Live` region; a change rebuilds that region only, never per frame.

## Tests

`tests/model.rs`: civil dates, ranges, spans, lanes, minutes/pixels, navigation, keys, source. `tests/view.rs`: drawn offscreen in all four styles, driven by clicks and keys through Blinc's event router.

## Unsupported (named, not built)

- Drag to create/move/resize an item (no gesture API used; resize/long-press avoided on purpose).
- Agenda/list view, year view, week numbers.
- Initial scroll position of the time axis (starts at 00:00; the Blinc scroll offset is not set).
- Locale formats: hour labels are `HH:00` and the year/"+N" are plain numbers; month and weekday names come from `Say`.
- Days of 23 or 25 hours (daylight saving): the axis is always 24 hours of 60 minutes, and an item's minutes are the caller's wall-clock minutes; the host must cut or place such days itself.
- Recurrence, time zones, working-hours shading: the host expands and localises.
- Per-style icon shapes, glass/backdrop effects (only colours, radii, spacing, fonts of the style).
- Buttons are `blinc_cn` buttons; their height is cn's, not the style's.
- Overlap lanes use 30-minute minimum height for lane choice only; a very long title is clipped, not wrapped.
