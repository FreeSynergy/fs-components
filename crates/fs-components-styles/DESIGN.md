# fs-components-styles - Design

The style system (ADR 0066, items 5 and 6): `Style` is plain data (spacing, heights, type, radii, light and dark palette, effects, icon set). No Blinc here; views read a style and make elements. Four styles are built in: Admin (SAP Fiori measures), Simple, Apple, Windows.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `color`, `tokens`, `effects` | none | plain data and pure functions |
| `icons.rs` | Value object | `IconSet` holds how to draw, over a closed enum of glyphs |
| `style.rs` | Registry + trait `StyleCatalog` | a frontend names a `StyleId`, a new style is one more entry; `Builtin` is the catalog of the four |
| `styles/*` | none | one function per style, returning data |

## State

| Part | State |
|---|---|
| Tokens, colours (light, dark), effects, `Style::demands()` | done |
| Admin, Apple, Windows numbers | **partly from the sheets** (`docs/en/components/styles/{fiori,apple,windows}.md`, read 2026-11-10): sizes, type ramps, spacing, radii (Admin, Windows), focus (Admin), system colours (Apple), fonts; `Provenance::Partial`, a test keeps the sheet numbers. **Not in the sheets, written by memory** (named at the top of each style file): Admin palette (except focus and list colours), shadows, text line height; Apple row heights, radii, glass blur and tint, focus ring, neutral palette; Windows control and row heights, all colours, shadows, focus width. Replace when measured from the screenshots of the programs. Open decision: the sheet proposes compact heights for the generated Admin frontend (the dense ones here) |
| Simple | our own design (`Provenance::Sheet`: there is no outside sheet) |
| Icon set | 11 glyphs of our own on a 24 grid; a style changes line width, ends, corners and size. Entirely different shapes per style: Unsupported |
| Fonts | named per style (open licences: Source Sans 3, Inter, Noto Sans, Atkinson Hyperlegible); loading them is the task of the app shell, not built here |
| Glass, Backdrop (Mica) | glass measured on Blinc: it blurs (step 255 to 23). Backdrop: **Unsupported** by Blinc (no compositor blur), drawn flat. Effects exist per appearance (`effects`, `effects_dark`) |

## Known finding

Apple's blue (`#0088FF` in the sheet, `#007AFF` before) under white text has a contrast of about 3.5:1 (WCAG wants 4.5:1 for small text). The 1:1 rebuild keeps Apple's colour; the test holds text on the accent to 3:1 and says why.
