# fs-components-styles - Design

The style system (ADR 0066, items 5 and 6): `Style` is plain data (spacing, heights, type, radii, light and dark palette, effects, icon set). No Blinc here; views read a style and make elements. Four styles are built in: Admin (SAP Fiori measures), Simple, Apple, Windows.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `color`, `tokens`, `effects`, `icons` | none | plain data and pure functions |
| `style.rs` | Registry + trait `StyleCatalog` | a frontend names a `StyleId`, a new style is one more entry; `Builtin` is the catalog of the four |
| `styles/*` | none | one function per style, returning data |

## State

| Part | State |
|---|---|
| Tokens, colours (light, dark), effects, `Style::demands()` | done |
| Admin, Apple, Windows numbers | **provisional**: written from the public guidelines by memory. The measurement sheets (`docs/en/components/styles/{fiori,apple,windows}.md`, Thread M) did not exist on 2026-11-10. When they do, replace value by value and set `Provenance::Sheet`. |
| Simple | our own design (`Provenance::Sheet`: there is no outside sheet) |
| Icon set | 11 glyphs of our own on a 24 grid; a style changes line width, ends, corners and size. Entirely different shapes per style: Unsupported |
| Fonts | named per style (open licences: Source Sans 3, Inter, Noto Sans, Atkinson Hyperlegible); loading them is the task of the app shell, not built here |
| Glass, Backdrop (Mica) | described, **not measured on Blinc yet**; `Style::demands()` lists what each style asks, the measurements go to git-bug in the Blinc clone |

## Known finding

Apple's blue `#007AFF` under white text has a contrast of 4.02:1 (WCAG wants 4.5:1 for small text). The 1:1 rebuild keeps Apple's colour; the test holds text on the accent to 3:1 and says why.
