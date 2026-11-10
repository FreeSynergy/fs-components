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
| Surfaces | `Look::surface` draws the chrome/overlay surface of a style: flat, glass (measured: blurs), **Backdrop (Mica) is Unsupported**: drawn as the flat tint, Blinc has no compositor blur (git-bug in the Blinc clone). The frontend places the surface, components do not paint a ground |
| Focus and hover look of the input | **partly Unsupported**: Blinc draws hover and focus of a text input with its own theme (`ua_styles`), so the style's focus colour and ring (`Palette::focus`, `Effects::focus`) are not applied to the inputs yet. Text and idle colours are the style's. Possible cause of the "white text while typing" hand test (not reproduced in the live probe, not confirmed) |
| Pasting a whole value into one part | e-mail and date spread it (`settle`); name, phone, address do not (Unsupported) |
| Rules of the standard types in `fs-types` (Thread T, `fs-interfaces` v0.0.4: `BasicRules` check/suggest, `Date`, `Name`, `Address`) | **not used yet**: this crate still has its own date, name and address rules and the `Suggest` list. To do: delegate rules to `fs-types`, keep only the parts and the views here |

## Handtest errors of the input (2026-11-10)

- AltGr+Q gives "2": cause found in Blinc (`blinc_platform_desktop` `convert_key` folds `@` into `Key::Num2`, `blinc_app` `keyboard.rs` writes `2` back for it). Recorded in the Blinc clone as a git-bug entry with the proof; this crate cannot repair it: `settle` acts only on an `@` that arrives (typed with a layout that gives it, or pasted). **AltGr and layout keys are Unsupported until Blinc is fixed**: typing `ada@x.org` on a German layout silently gives the local part `ada2x.org`, and `+` becomes `=` in the phone country part (`tools/live-probe/scnD-keys.txt`).
- White text while typing and sluggishness: **open, not reproduced, not fixed here.** The only measurement is the headless test `a_key_costs_little` (under 50 ms per key), which does not reproduce the slowness seen by hand; the live probe (`tools/live-probe`) did not show white text. Blinc's own focus/hover theme is a possible cause (see the focus row above). Thread B measures typing latency in Blinc.
