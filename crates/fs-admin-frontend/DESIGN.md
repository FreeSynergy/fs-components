# fs-admin-frontend - Design

The generated Admin frontend of ADR 0066: from the type description of a program (`fs-types`, `DescribesType`) it makes list, detail, forms and command buttons, one code for every program. Style: SAP Fiori (`docs/en/components/styles/fiori.md`), through `fs-components-styles`; fields through `fs-components-fields`; window shell after ADR 0065. No render code and no look in any program (ADR 0066 rule of 2026-11-10): a program gives a `TypeDescription` and a `Backend`, nothing else.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `backend.rs` `Backend` | Repository (trait first, ADR 0043) | the only thing a program implements for the frontend; in memory for tests, over Cap'n Proto for People |
| `form.rs` `FormModel` | Unit of Work (the check is `fs-types` `check_record`, Strategy by kind there) | remembers the start, hands back what changed plus the required fields, checked; the form and the program's `check` command say the same |
| `list.rs` `ListModel` | State | records, sort, filter, selection and their transitions; the rows are a pure function of it |
| `detail.rs`, `view/kinds.rs` | none (pure functions) | description and record in, entries out; basic type to component kind |
| `view/value.rs` | Strategy by basic type | the component of `fs-components-fields` where there is one, else the canonical text |
| `view/detail.rs`, `view/chrome.rs` | View function | pure from data to elements |
| `view/findings.rs` | Adapter | reads the session under its lock and makes the elements of the findings |
| `view/list.rs` | Adapter over the `ListView` widget | a head of our own that can be clicked, rows from the model |
| `view/inputs.rs` | Adapter + Strategy | Blinc `text_input` for the plain input; component of the type or plain input by basic type |
| `view/form.rs` | Adapter + Observer | slots made once per form and kept, findings in an area of their own |
| `view/screens.rs`, `view/surface.rs` | Adapter (+ Observer in the surface) | session state to elements; the click handlers they build change the session |
| `view/redraw.rs` | Observer with a worker thread | messages signal, the worker draws (folding bursts) |
| `commands.rs` `plan_commands` | none (a small set of pure functions) | placement, confirmation and form fields come from the kind and the input of the description |
| `screen.rs` `Screen`, `session.rs` and `session_done.rs` `Session` | State | screens are the states, methods are the transitions; `session_done.rs` holds opening a command, the transitions after it, the refusals, the rule for old records and the support of commands |
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
| Model: backend trait, form, list, detail, command plans, messages, session | done, tests against the traits over a `Backend` in memory (`model`, `session`, `review_model`, `review_session`, `fourth_session`, `fifth_session`, `sixth_session`, `third_session`) |
| Views (`view/`): list with sortable head, detail, form (typed components of `fs-components-fields`, plain input for the rest), confirm, result, About, as a `Surface` of the shell | done, tests drawn offscreen and driven with the window's router (`view`, `review_view`, `third_view`, `fifth_view` (also the locked controls), `window`): click, type, add a slot, save, refuse, quit, redraw |
| Messages draw the page again on another thread (a transition holds the session lock; found by the click test as a deadlock); bursts are folded, a success runs out by a timer | done |

## Locks and the backend (review 2026-11-10)

* The `Backend` is called **synchronously, with the session locked** (`act` holds the lock while `Backend::run` runs). A program that answers slowly freezes the window for that time; an asynchronous backend is a later change of the trait, not of the screens.
* Messages tell their listeners **outside** their own lock (a listener may ask or subscribe). The page reads the messages **after** letting go of the session. The surface draws on a worker thread of its own (one per surface; it ends when the session is dropped, which in a running window is when the window closes), because messages change while the session is locked and a draw reads the session.
* `change` and `delete` without a reference to the type itself are **not offered** (`unoffered_commands` lists them), because they would run without a record id.
* `submit` returns the changed fields and the required ones; a field emptied by the person is an empty list (cleared).
* **The rule for `change` is part of the contract** (`Backend::run`, to be written into `types.capnp` by Thread T if the strategy agrees): a field missing from the input is unchanged, an empty list clears it, values replace; required fields are always sent. A program that reads the input as a whole record would delete the optional fields that were not sent. A refusal counts the values as they were sent; the form maps it back to the slots.
* A command on a record works on the record of the shown detail screen, and only if it is loaded; only an unreachable program (`Unavailable`) makes the records old (`is_stale`; a refusal or a missing thing does not, and a stored search that is refused later is dropped with a `rejected` message). While old: a record screen says "not loaded", offers no record command, the list carries a `list.stale` mark, and every screen that shows records (list, record) reads again when entered; a record that is gone after a successful reading leads to the list with `not-found`. A record screen can also say "not loaded" when `is_stale()` is false: the record was never in the list (a created record whose reading failed and was then repaired by a narrower search).
* **Locked controls ("coming soon", rule for R of 2026-11-10):** every described command has its control. One the core cannot carry out yet (`Backend::supported` = false, which is also the default of a program that does not answer) is shown locked with `command.soon` and cannot be started (`Session::begin` refuses). The core is asked when the records are read, so the control unlocks the next time they are read, with no change in the frontend. **Provisional:** `supported` is a method of `Backend` until Thread T puts `supported` into the description (`types.capnp`); then a command is supported only if the description and the backend both say so.
* The same error is told once per occurrence (`fail` closes the earlier one with that code first); while the program stays down every new try tells it again, which is wanted (each try failed).
* The slot number in a finding is the number as text in the sentence of `finding.slot`; it is not formatted for the reader's locale (noted, not changed).
* Unsaved text: a search form does not count; values are compared cleaned. Quitting with text shows the surface that has it (the visible one first, then the others in order), closes About and says it once.
* Commands that are not offered are named on the About page.
* `open_external` (Help) returns nothing: a failure to open the address cannot be shown.
* `Session`, `Messages` and `FormModel` have no trait of their own (coding.md section 1): they are values the frontend owns; the boundary to the outside is `Backend`. Reported to the strategy.

## Translation keys the frontend asks for (through `Say`)

`action.{add,back,cancel,close,remove,save}`, `menu.{program,view,help,quit,about}`, `message.<code>` (`done`, `rejected`, `not-found`, `unavailable`, `unsaved`), `problem.<code>`, `hint.<label>`, `sort.{ascending,descending}`, `finding.slot` (a sentence with the named placeholder `{n}`, the number of the value), `list.{empty,separator,stale}`, `detail.not-loaded`, `confirm.sure`, `command.soon`, `value.none`, `about.{version,origin,licence,repository,unoffered}`, and the keys of the description (labels, `<label>.<choice>`).

## Unsupported (named, not faked)

* Ctrl+Q and F1 of ADR 0065, the Settings entry (this window is the settings) and Fullscreen: not built; the shell menus have no shortcuts. Quit leaves without asking when no form holds unsent text; with text it shows that surface and an `unsaved` message and stays (a dialog "leave anyway?" comes later).
* People: waits for the type description of Thread P.
* Basic types without a component in `fs-components-fields` (text, multi-line text, place, date-time, language, link, image, key, choice, keywords, reference): a plain input stands in (choice: buttons). It belongs in `fs-components-fields`. Buttons are `blinc_cn`'s, their height is not the style's.
* Sort direction is written as a word, not drawn as an arrow (the icon set has no arrow).
