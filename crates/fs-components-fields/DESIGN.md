# fs-components-fields - Design

The four components of each basic type (ADR 0066, item 4): **input**, **display**, **compact**, **cell**, for e-mail address, name, phone number, postal address and date, in every style of `fs-components-styles`. A program never draws a field; it names a `Kind`, a `Mode` and a `Look`.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `field.rs` | Strategy (trait `BasicField`) | what a type is: parts, rule, joined and short forms; written once per type, views written once for all |
| `kinds/*` | Strategy | one implementation per type; the rules come from the standard (`EmailAddress`, `PhoneNumber` of `fs-interfaces` `values`; JSContact `Name`/`Address`; RFC 3339 `full-date`) |
| `kinds/kind.rs` | Registry | `Kind` names a type, `Kind::field()` gives it |
| `suggest.rs` | Strategy (trait `Suggest`) | the command "suggest" of the type description: the program answers from its data, the input only asks |
| `problem.rs` | Value object | a code with a translation key; sentences are the program's (ADR 0057) |
| `view/look.rs` | Adapter | style data in, Blinc colours, text, icons out |
| `view/{display,compact,cell}.rs` | Adapter | Blinc elements from the type and the look |
| `view/input.rs`, `suggestions.rs` | Adapter + Observer | Blinc `text_input` per part (not `blinc_cn`'s: its heights are fixed at 32/40/48) plus a live area under the parts, outside the fields so a redraw never takes the keyboard |

## State

| Part | State |
|---|---|
| E-mail (local part, `@` fixed between, domain), name, phone (country, number), address, date (year, month, day) | done, 20 components, drawn and measured in 4 styles x light/dark |
| E-mail suggestions | done: the input asks `Suggest`, shows chips, a click fills the part. The program-side command is not wired (the type description of Thread T does not exist yet): `SuggestFromList` is the only implementation |
| `@` without the keyboard problem | an `@` typed or pasted into the first part is spread over both (`settle`), the focus goes on to the domain |
| Value types for name, address, date | here, local to this crate (provisional); move to the types crate of Thread T when it is tagged |
| Per-language order and format (date, name, address) | **Unsupported**: ISO date, Western name order, street/postcode/locality order; the words crate has no date formatting yet |
| Focus by Tab between parts, sizes by style | parts are normal text inputs; the style gives their height, radius and colours (tested: the control height equals the style's) |
| Glass, backdrop (Mica) behind the components | not drawn here: a surface belongs to the frontend, which sets the ground; measured separately against Blinc |

## Handtest errors of the input (2026-11-10)

- AltGr+Q gives "2": cause found in Blinc (`blinc_platform_desktop` `convert_key` folds `@` into `Key::Num2`, `blinc_app` `keyboard.rs` writes `2` back for it). Recorded in the Blinc clone as a git-bug entry with the proof; this crate stays independent of it (see `settle`).
- White text while typing and sluggishness: see `todo/handoff-programming.md` for the state of the probes.
