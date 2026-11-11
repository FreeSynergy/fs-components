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

* **The list is the command `find`** (`CommandDescription::standard_find`, input `term`, empty = everything, paging with the contract's cursor): `Backend::list(query)` takes the typed input of `find`. Decided by Thread T / the strategy (fs-interfaces v0.0.11, `types.capnp`).
* **A command works on one record** when it is `change` or `delete` or its input holds a reference to the type itself (`TypeDescription::record_commands`). The id of the record travels in the call (`Backend::change(record, edit)`, `Backend::run(name, Some(record), input)`), not in the input; a reference to the type itself in the input is still filled from the shown record.
* **Messages carry codes**, never sentences (ADR 0057): `message.<code>`; codes now: `done`, `rejected`, `not-found`, `unavailable`, `handshake`, `unsaved`.
* Columns: fields that fit one line (not multi-line text, not image), at most six, required ones first when there are more.

## State

| Part | State |
|---|---|
| Model: backend trait, form, list, detail, command plans, messages, session | done, tests against the traits over a `Backend` in memory (`model`, `session`, `review_model`, `review_session`, `fourth_session`, `fifth_session`, `sixth_session`, `seventh_session`, `third_session`) |
| Views (`view/`): list with sortable head, detail, form (typed components of `fs-components-fields`, plain input for the rest), confirm, result, About, as a `Surface` of the shell | done, tests drawn offscreen and driven with the window's router (`view`, `review_view`, `third_view`, `fifth_view` (also the locked controls), `eighth_view` (the command bar wraps, the title is above it), `window`): click, type, add a slot, save, refuse, quit, redraw |
| Messages draw the page again on another thread (a transition holds the session lock; found by the click test as a deadlock); bursts are folded, a success runs out by a timer | done |

## Locks and the backend (review 2026-11-10)

* The `Backend` is called **synchronously, with the session locked** (`act` holds the lock while `Backend::run` runs). A program that answers slowly freezes the window for that time; an asynchronous backend is a later change of the trait, not of the screens.
* Messages tell their listeners **outside** their own lock (a listener may ask or subscribe). The page reads the messages **after** letting go of the session. The surface draws on a worker thread of its own (one per surface; it ends when the session is dropped, which in a running window is when the window closes), because messages change while the session is locked and a draw reads the session.
* `change` is an `Edit` (fs-interfaces v0.0.11): the form is the type's own fields (a `change` command has no input fields), `FormModel::submit_changes` builds the `Edit` of only the changed fields (typed, with the basic type; a cleared field has no values), `Edit::check` validates it, `Backend::change` carries it. No JSON, no multi-line text.
* `submit` returns the changed fields and the required ones; a field emptied by the person is an empty list (cleared).
* **The rule for `change`** is the contract's (v0.0.11): a field missing from the edit is unchanged, one with no values is cleared, one with values is replaced. A refusal counts the values as they were sent; the form maps it back to the slots.
* A command on a record works on the record of the shown detail screen, and only if it is loaded; only an unreachable program (`Unavailable`) makes the records old (`is_stale`; a refusal or a missing thing does not, and a stored search that is refused later is dropped with a `rejected` message). While old: a record screen says "not loaded", offers no record command, the list carries a `list.stale` mark, and every screen that shows records (list, record) reads again when entered; a record that is gone after a successful reading leads to the list with `not-found`. A record screen can also say "not loaded" when `is_stale()` is false: the record was never in the list (a created record whose reading failed and was then repaired by a narrower search).
* **Locked controls ("coming soon", rule for R of 2026-11-10):** every described command has its control. One the core cannot carry out yet (`Backend::supported` = false, which is also the default of a program that does not answer) is shown locked with `command.soon` and cannot be started (`Session::begin` refuses). The core is asked when the records are read, so the control unlocks the next time they are read, with no change in the frontend. The control is free only if the description (`CommandDescription::supported`, v0.0.11; unset reads as false) **and** the program (`Backend::supported`, default false) both say so.
* The same error is told once per occurrence (`fail` closes the earlier one with that code first); while the program stays down every new try tells it again, which is wanted (each try failed).
* The slot number in a finding is the number as text in the sentence of `finding.slot`; it is not formatted for the reader's locale (noted, not changed).
* Unsaved text: a search form does not count; values are compared cleaned. Quitting with text shows the surface that has it (the visible one first, then the others in order), closes About and says it once.
* `open_external` (Help) returns nothing: a failure to open the address cannot be shown.
* `Session`, `Messages` and `FormModel` have no trait of their own (coding.md section 1): they are values the frontend owns; the boundary to the outside is `Backend`. Reported to the strategy.

## Translation keys the frontend asks for (through `Say`)

`action.{add,back,cancel,close,remove,retry,save}`, `menu.{program,view,help,quit,about}`, `message.<code>` (`done`, `rejected`, `not-found`, `unavailable`, `handshake`, `quit-failed`, `unsaved`), `problem.<code>` (also `problem.{wrong-type,nested-path,duplicate-path,read-only}`), `hint.<label>`, `sort.{ascending,descending}`, `finding.slot` (a sentence with the named placeholder `{n}`, the number of the value), `list.{empty,separator,stale}`, `detail.not-loaded`, `confirm.sure`, `command.soon`, `value.none`, `about.{version,origin,licence,repository}`, and the keys of the description (labels, `<label>.<choice>`).

## Unsupported (named, not faked)

* Ctrl+Q and F1 of ADR 0065, the Settings entry (this window is the settings) and Fullscreen: not built; the shell menus have no shortcuts. Quit closes the window (`AppWidget::quit`, the window's own close callback: the event loop ends and everything is dropped in order; no `std::process::exit`) without asking when no form holds unsent text; with text it shows that surface and an `unsaved` message and stays (a dialog "leave anyway?" comes later).
* People: waits for the type description of Thread P.
* Basic types without a component in `fs-components-fields` (text, multi-line text, place, date-time, language, link, image, key, choice, keywords, reference): a plain input stands in (choice: buttons). It belongs in `fs-components-fields`. Buttons are `blinc_cn`'s, their height is not the style's.
* Sort direction is written as a word, not drawn as an arrow (the icon set has no arrow).

## Version handshake and fs-interfaces v0.0.17 (review of 2026-11-10)

* **`BackendError::Handshake(String)`**: the program could not reach a server of the right version (`fs_interfaces::rpc::HandshakeError`: `CannotRestart`, `ServerMismatch`, `Io`). The text is `error.to_string()`; fs-interfaces words it for the person. The frontend does not depend on the `rpc` feature of fs-interfaces (it pulls in the runtime); the program maps its error with `BackendError::Handshake(e.to_string())`.
* **A persistent message with "Try again"** (ADR 0065 points 4 and 5, `fs-interfaces/DESIGN.md`, version handshake): code `handshake`, `Message::detail` = the words as they are (shown under the text of `message.handshake`, never joined with it), `Message::retry` = the button `action.retry`. The records are old while it stands (`is_stale`): never a window that looks empty without a word. "Try again" is `Session::retry`: `Backend::reconnect` (the program calls `connect_matching` once more; the default does nothing), then the records are read again; a failure is told once with its new words, success closes the message. The connecting runs on a worker thread with no lock held (`Session::retry_begin` / `retry_finish`; the window's button starts it and the messages tell the window when it is done), so the window does not freeze while the program starts a server; a second press while one try runs does nothing.
* **v0.0.17: read-only fields** (`FieldDescription::read_only`, `RecordProblem::ReadOnly`): the form of `change` leaves them out (they stay on the detail screen) and `submit_changes` never puts one into the `Edit`, which `Edit::check` would refuse; the finding has the key `problem.read-only` should a program still send one.
* A command that the core stops offering while its form is open does not run: `Session::submit` asks `is_supported` again.
* Tests: `tests/ninth_session.rs` (handshake, retry, read-only, locked while open) and `tests/ninth_view.rs` (the message drawn with words and button; Quit closes the window).

* **"Try again" never ends up dead (review of 2026-11-10):** everything slow of a try happens in `RetryJob::run` with no lock held (connect again, read the records, ask the core about its commands); only its answers go into the session in `retry_finish`, under the lock. A backend that panics in `reconnect`, `list` or `supported` is a failed try (`unavailable`), so `retrying` is always reset and the next "Try again" works. Tests: `ninth_session.rs`.
