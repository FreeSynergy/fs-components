# fs-admin-frontend - Design

The generated Admin frontend of ADR 0066: from the type description of a program (`fs-types`, `DescribesType`) it makes list, detail, forms and command buttons, one code for every program. Style: SAP Fiori (`docs/en/components/styles/fiori.md`), through `fs-components-styles`; fields through `fs-components-fields`; window shell after ADR 0065. No render code and no look in any program (ADR 0066 rule of 2026-11-10): a program gives a `TypeDescription` and a `Backend`, nothing else.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `backend.rs` `Backend` | Repository (trait first, ADR 0043) | the only thing a program implements for the frontend; in memory for tests, over Cap'n Proto for People |
| `form.rs` `FormModel` | Unit of Work (the check is `fs-types` `check_record`, Strategy by kind there) | remembers the start, hands back what changed plus the required fields, checked; the form and the program's `check` command say the same |
| `list.rs` `ListModel` | State | records, sort, filter, selection and their transitions; the rows are a pure function of it |
| `detail.rs`, `view/kinds.rs`, `view/value.rs` | none (pure functions) | description and record in, entries out; basic type to component kind |
| `view/*.rs` pages, `view/chrome.rs` | View function; `view/screens.rs`, `view/surface.rs` Adapter | pages are pure from state; the screens and the surface build click handlers that change the session |
| `view/redraw.rs` | Observer with a worker thread | messages signal, the worker draws (folding bursts) |
| `commands.rs` `plan_commands` | none (a small set of pure functions) | placement, confirmation and form fields come from the kind and the input of the description |
| `session.rs` `Session`, `Screen` | State | screens are the states, methods are the transitions |
| `messages.rs` `Messages` | Observer with a lifetime per message | ADR 0065 point 4: success goes by itself, an error stays, none follows to another screen |
| `window.rs` `admin_window` | Facade | one call builds the shell with a screen per type and the standard menus Program / View / Help |
| `clock.rs` `Clock` | Strategy | lifetimes are testable (`ManualClock`) |

## Provisional decisions (reported to the strategy)

* **The list is the command kind `find`.** The description names commands but not how all records are read; `Backend::list(query)` takes the typed input of `find` (empty = all).
* **A command works on one record** when its input has a `reference` to the type itself (the field is filled from the record of the detail screen, not shown). Everything else works on the list; a `change` or `delete` without such a reference is not offered. `delete` asks once more. These rules should be written into `types.capnp` comments by Thread T if the strategy agrees.
* **Messages carry codes**, never sentences (ADR 0057): `message.<code>`; codes now: `done`, `rejected`, `not-found`, `unavailable`, `unsaved`.
* Columns: fields that fit one line (not multi-line text, not image), at most six, required ones first when there are more.

## State

| Part | State |
|---|---|
| Model: backend trait, form, list, detail, command plans, messages, session | done, tests against the traits over a `Backend` in memory (`model`, `session`, `review_model`, `review_session`, `third_session`) |
| Views (`view/`): list with sortable head, detail, form (typed components of `fs-components-fields`, plain input for the rest), confirm, result, About, as a `Surface` of the shell | done, tests drawn offscreen and driven with the window's router (`view`, `review_view`, `third_view`, `window`): click, type, add a slot, save, refuse, quit, redraw |
| Messages draw the page again on another thread (a transition holds the session lock; found by the click test as a deadlock); bursts are folded, a success runs out by a timer | done |

## Locks and the backend (review 2026-11-10)

* The `Backend` is called **synchronously, with the session locked** (`act` holds the lock while `Backend::run` runs). A program that answers slowly freezes the window for that time; an asynchronous backend is a later change of the trait, not of the screens.
* Messages tell their listeners **outside** their own lock (a listener may ask or subscribe). The page reads the messages **after** letting go of the session. The surface draws on a worker thread of its own (one per surface; it ends when the session is dropped, which in a running window is when the window closes), because messages change while the session is locked and a draw reads the session.
* `change` and `delete` without a reference to the type itself are **not offered** (`unoffered_commands` lists them), because they would run without a record id.
* `submit` returns the changed fields and the required ones; a field emptied by the person is an empty list (cleared).
* **The rule for `change` is part of the contract** (`Backend::run`, to be written into `types.capnp` by Thread T if the strategy agrees): a field missing from the input is unchanged, an empty list clears it, values replace; required fields are always sent. A program that reads the input as a whole record would delete the optional fields that were not sent. A refusal counts the values as they were sent; the form maps it back to the slots.
* A command on a record works on the record of the shown detail screen, and only if it is loaded; after a failed reload the records are marked old (`is_stale`): the screen says "not loaded", offers no record command, and going back reads the list again.
* Unsaved text: a search form does not count; values are compared cleaned. Quitting with text shows the surface that has it, closes About and says it once.
* Commands that are not offered are named on the About page.
* `open_external` (Help) returns nothing: a failure to open the address cannot be shown.
* `Session`, `Messages` and `FormModel` have no trait of their own (coding.md section 1): they are values the frontend owns; the boundary to the outside is `Backend`. Reported to the strategy.

## Translation keys the frontend asks for (through `Say`)

`action.{add,back,cancel,close,remove,save}`, `menu.{program,view,help,quit,about}`, `message.<code>` (`done`, `rejected`, `not-found`, `unavailable`, `unsaved`), `problem.<code>`, `hint.<label>`, `sort.{ascending,descending}`, `list.{empty,separator}`, `detail.not-loaded`, `confirm.sure`, `value.none`, `about.{version,origin,licence,repository,unoffered}`, and the keys of the description (labels, `<label>.<choice>`).

## Unsupported (named, not faked)

* Ctrl+Q and F1 of ADR 0065, the Settings entry (this window is the settings) and Fullscreen: not built; the shell menus have no shortcuts. Quit leaves without asking when no form holds unsent text; with text it shows that surface and an `unsaved` message and stays (a dialog "leave anyway?" comes later).
* People: waits for the type description of Thread P.
* Basic types without a component in `fs-components-fields` (text, multi-line text, place, date-time, language, link, image, key, choice, keywords, reference): a plain input stands in (choice: buttons). It belongs in `fs-components-fields`. Buttons are `blinc_cn`'s, their height is not the style's.
* Sort direction is written as a word, not drawn as an arrow (the icon set has no arrow).
