# fs-admin-frontend - Design

The generated Admin frontend of ADR 0066: from the type description of a program (`fs-types`, `DescribesType`) it makes list, detail, forms and command buttons, one code for every program. Style: SAP Fiori (`docs/en/components/styles/fiori.md`), through `fs-components-styles`; fields through `fs-components-fields`; window shell after ADR 0065. No render code and no look in any program (ADR 0066 rule of 2026-11-10): a program gives a `TypeDescription` and a `Backend`, nothing else.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `backend.rs` `Backend` | Repository (trait first, ADR 0043) | the only thing a program implements for the frontend; in memory for tests, over Cap'n Proto for People |
| `form.rs` `FormModel` | Unit of Work + Template method (the check is `fs-types` `check_record`) | remembers the start, hands back only a checked set; the form and the program's `check` command say the same |
| `list.rs` `ListModel` | View function (Model-View-Update) | state in, rows out; sorting and filtering are pure |
| `detail.rs` | View function | description and record in, entries out |
| `commands.rs` `plan_commands` | Strategy by command kind | placement, confirmation and form fields come from the kind and the input of the description |
| `session.rs` `Session`, `Screen` | State | screens are the states, methods are the transitions |
| `messages.rs` `Messages` | Observer with a lifetime per message | ADR 0065 point 4: success goes by itself, an error stays, none follows to another screen |
| `clock.rs` `Clock` | Strategy | lifetimes are testable (`ManualClock`) |

## Provisional decisions (reported to the strategy)

* **The list is the command kind `find`.** The description names commands but not how all records are read; `Backend::list(query)` takes the typed input of `find` (empty = all).
* **A command works on one record** when its input has a `reference` to the type itself (the field is filled from the selection, not shown), or when its kind is `change` or `delete`. Everything else works on the list. `delete` asks once more. These rules should be written into `types.capnp` comments by Thread T if the strategy agrees.
* **Messages carry codes**, never sentences (ADR 0057): `message.<code>`; codes now: `done`, `rejected`, `not-found`, `unavailable`.
* Columns: fields that fit one line (not multi-line text, not image), at most six, required ones first when there are more.

## State

| Part | State |
|---|---|
| Model: backend trait, form, list, detail, command plans, messages, session | done, 25 tests (against the traits, over a `Backend` in memory) |
| Views (`view/`): list with sortable head, detail, form (typed components of `fs-components-fields`, plain input for the rest), confirm, result, About, as a `Surface` of the shell | done, 5 tests drawn offscreen and driven with the window's router (click, type, add a slot, save, refuse) |
| Messages draw the page again on another thread (a transition holds the session lock; found by the click test as a deadlock) | done |

## Unsupported (named, not faked)

* Shell menus of ADR 0065 (Programm / Ansicht / Hilfe), Ctrl+Q and F1: not built; `fs-components-app` menus have no shortcuts.
* People: waits for the type description of Thread P.
* Basic types without a component in `fs-components-fields` (text, multi-line text, place, date-time, language, link, image, key, choice, keywords, reference): a plain input stands in (choice: buttons). It belongs in `fs-components-fields`. Buttons are `blinc_cn`'s, their height is not the style's.
* Sort direction is written as a word, not drawn as an arrow (the icon set has no arrow).
