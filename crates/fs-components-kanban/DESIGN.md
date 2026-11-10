# fs-components-kanban - Design

A board of columns with cards (ADR 0066: the program gives data, a trait and words; this crate draws). Entry: `KanbanBoard::new(source, say, look).events(events).build(width, height)`.

## Data boundary (stable)

| Type | Role |
|---|---|
| `Column` (key, title, limit, cards), `Card` (key, title, subtitle, labels, due, assignee), `Label`, `Due`, `DueState`, `Tone` | plain data, texts already in the reader's language |
| `Operation` (`AddColumn`, `Rename`, `Limit`, `Move`) | what the host may support |
| trait `BoardSource` | `columns()` and `supports(op)`; read on every build, so a control unlocks by itself when `supports` turns true |
| trait `BoardEvents` | `select_card`, `move_card(key, from, to) -> Answer`, `add_column`, `rename_column`, `set_limit` (requests; defaults do nothing, `move_card` refuses). `NoEvents`, `FixedBoard` for tests |

Texts are translation keys through `Say`: `kanban.add_column|rename|limit|move_left|move_right|grab|drop|soon|empty`, `kanban.refused.<code>` (`locked` from the board, others from `Answer::Refused(code)`). `kanban.soon` is the "kommt noch" hint. Looks come only from `fs-components-styles` (`Look`).

## Moves

One state machine (`state.rs`) for every way: `grab(key)` / `hover(column)` / `drop_held()` / `cancel()` (the gesture API of the tasks workbench, for a frontend gesture), keys (arrows walk, Space/Enter-as-click picks up and puts down, Left/Right choose the column while held, Escape cancels, raw Enter selects), the move buttons on the selected card (left, grab/drop, right), and click-to-move (pick up, then click the target column). A finished move is a `MoveRequest`; the host answers `Accepted` or `Refused(code)` and the code is shown as the notice. A move to the same column is no move. A move on an unsupported host is refused with `locked` and nothing changes.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `model.rs` | Value object | small immutable data |
| `source.rs` | Strategy + Null object | host owns data and decisions |
| `state.rs`, `keys.rs` | State + Command | one machine for gestures, keys, buttons; yields a `MoveRequest` |
| `layout.rs` | none (pure functions) | column width, card height, initials, count mark, column under pointer |
| `handle.rs` | Facade + Observer | one clonable handle; change state, tell host, redraw the live area only |
| `view/*` | View function / Adapter | state in, Blinc elements out; `controls.rs` draws a control live or locked the same way |
| `view/keys.rs` | Adapter | copy of a private helper of `fs-components-widgets` (move to a shared crate when one exists) |

## Locked state

A control of an unsupported operation (add column, rename, limit, the three move buttons) is drawn with the same size, weak text and the `kanban.soon` hint, without a handler (`data-locked`). The move buttons are locked when `Move` is unsupported; a keyboard or gesture move then shows the `locked` notice.

## Blinc notes

Clicks bubble to ancestors and there is no way to stop them. So a click target that has buttons next to it is a transparent layer after the content with the buttons as its siblings, and controls inside a column tell the column through a mark (`swallow`) that the click is theirs.

## Tests

`tests/model.rs`: sizes, initials, grab/hover/drop, refusals, buttons, keys. `tests/model_keys.rs`: refusals while putting down, keys with nowhere to go, drop_at. `tests/view_locked.rs`: drawn offscreen in all four styles; locked controls do nothing and stay visible; unlock without a new build. `tests/view_moves.rs`: move buttons, refusal notice, API gesture, click-to-move, keyboard, through Blinc's event router. `tests/view_withdrawn.rs`: the host withdraws the permission while a card is held (call, click, key, a host calling back from `supports`).

## Unsupported (named, not built)

- Moving by mouse drag (Blinc drag events are not used; the gesture API `grab/hover/drop` is there for a frontend that has one).
- Reordering inside a column and dropping at a position (a move appends; order is the host's).
- Inline rename/limit editing: the board only asks the host (`rename_column`, `set_limit`).
- Column move/delete, card create/delete/edit, swimlanes, collapse, filters, WIP limit enforcement (the limit is a mark; over limit shows a warning glyph and colour).
- Vertical scroll per column (the board scrolls as a whole), virtualising very long columns.
- Locale formats: counts are plain numbers; due text comes from the host.
- Buttons of the card are plain controls of this crate, not `blinc_cn` buttons.
