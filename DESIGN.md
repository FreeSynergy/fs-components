# fs-components - Design

Shared UI components of FreeSynergy on Blinc 0.6 (released crates, `blinc_* = "0.6"`, resolved 0.6.0). Rules: ADR 0005 (a widget is the core of an app), 0009 (Blinc only behind our API), 0015 (no second GUI framework), 0037 (a workbench is open, gestures come from the frontend), 0043 (patterns, small files), 0049 (components first, only what the next program needs).

## State

Version 0.1.0 (untagged until reviewed). Built for one consumer: the wiki reader (`fs-wiki-app`). Nothing else is planned or built here yet.

| Area | State |
|---|---|
| `search-field` (primitive): adapter over `blinc_cn` input, text kept in shared state | done |
| `text-view` (widget): text with marks is drawn as a flow of runs (see Blinc notes); neutral model (`Block`, `Inline`, `Mark`); headings, paragraphs, lists (also task items), quotes, code, tables, rules, emphasis, strong, strike, code span, links; unknown things arrive as `Block::Plain` and are shown as text | done |
| `tree-view` (widget): adapter over `blinc_cn` tree (keyboard of the Blinc tree: arrows, Home, End, Enter) | done |
| `reader` (workbench): `DataSource` trait (`entries`, `read`, `resolve`, `find`, `start`; `entries` and `find` return errors, the reader shows them), state machine `Reader` (open, follow, back, search), window (`launch`) and offscreen picture (`snapshot`) | done |
| Editing in the reader: `DataSource::can_edit`, `source_text`, `save`; `Reader::begin_edit`, `set_draft`, `save`, `cancel_edit`; unsaved changes block leaving the page; a refused save keeps the draft and shows why. Default of every method is "cannot be edited" | done (typing in the real text area not verified in a window) |
| Export: `DataSource::can_export`, `export`; an explicit button, saving never writes a file (ADR 0042, 0046) | done |
| Keyboard: tree keys (Blinc), Alt+Left = back, Tab to the search field and the Back button | done (Alt+Left unverified in a real window) |
| Pointer cursor over links; scrolling to a `#fragment`; table column alignment; selecting text; images | not built |
| Accessibility (Blinc has none for these elements, ADR 0009) | not built |

## Structure and pattern map (ADR 0043)

| Where | Pattern | Why |
|---|---|---|
| `widgets/text` | Composite + Value objects | the neutral description; lists and quotes hold blocks; marks are stand-off ranges in characters |
| `widgets/text_view/blocks.rs` | Strategy by block kind | one drawing function per kind; `Plain` is the safe fallback |
| `widgets/text_view/inline.rs` | Strategy by content | no marks: one Blinc rich text; marks: `flow.rs` |
| `widgets/text_view/flow.rs` | Adapter (Iterator over runs) | text cut into runs of one style (word plus its spaces), each a small text element in a wrapping flex row; a link run has its own click handler and reports to the `LinkHandler` (Blinc's own link would open a browser) |
| `workbenches/reader/edit.rs` | State | `Editing` (original, draft, error) and its transitions |
| `widgets/text_view/look.rs` | Value object snapshot | every colour and size read from the theme |
| `widgets/tree_view` | Adapter + Composite | `TreeNode` to the `blinc_cn` tree |
| `primitives/search_field` | Adapter | `blinc_cn` input; clone keeps its text across rebuilds |
| `workbenches/source.rs` | Strategy / Repository | what a workbench needs from outside; it knows no store, no wiki |
| `workbenches/reader` | State | `Shown` + transitions, testable without a window |
| `workbenches/view` | Adapter | state to elements, `Actions` callbacks (Observer) report what the person does |
| `workbenches/shell` | Facade | `launch` (window) and `snapshot` (picture); hide Blinc completely |

## Decisions

* **Layers.** `search-field` is a primitive (one value). `text-view` and `tree-view` are widgets: they know their data model when built (blocks and marks, labels). The `reader` is a workbench (ADR 0049: a subset of component): it gets its content at run time through `DataSource` and does not know what it means. Provisional: the crate split per layer and the name `reader` (see Open points).
* **The theme decides.** No colour or font is written in code. Sizes and colours are read from `ThemeState`; stylesheets only use `var(--...)` of the theme. The shell builds the theme bundle from `blinc_cn::cn_bundle()` plus our sheets.
* **Links.** The text view only reports `href` as written. Resolving it (relative path, external) is the data source's `resolve`, so the workbench does not know what a link means.
* **Gestures** are not here (ADR 0037); clicks and keys are the plain events of the framework.
* **Offscreen picture** uses Blinc's own `BlincApp::render_tree` into a texture and a PNG. It draws the same view functions as the window, from the same `Reader` state; clicks are not simulated.

## Open points

Questions for the owner (all provisional, easy to reverse):

* Names: `reader` for the workbench and the three crate names `fs-components-{primitives,widgets,workbenches}`.
* Layer of `text-view`: widget (chosen) or workbench, because it will later take any document.
* Tree labels show the document's title, folders show their last path part.

Blinc notes (0.6.0; to be passed upstream by the owner, reproductions in the report of the first release):

* A rich text is measured in one font weight but bold and italic spans are drawn in another face, so lines with marks run past the edge or overlap. Workaround: marks are drawn as a flow of single-style runs.
* `RichText` links always call `open_url`; there is no handler for an internal link. Workaround: own click handler per run.
* Emoji without a font are drawn blank but take their width; the bundled subset (`blinc_noto_emoji::register()`) lacks many (box glyphs).
* Italic is not drawn (`text(..).italic()` shows upright text with the default font); the `code` flag of a span draws no monospace or chip (`flow.rs` does it itself).
* `Scroll` has no `flex_1`; a `min_w` next to `w(0.0)` is ignored. The text area wraps at a narrower width than its box.
* `blinc_cn::cn_styles` documents `register_cn_styles(ctx)`, which does not exist in 0.6.0 (the bundle `cn_bundle()` does it).

Limits:

* `DataSource` is synchronous; a slow source blocks the interface thread.
* The tree is built completely (all nodes) once, because the Blinc tree is not lazy.
* Table columns share the width by the length of their longest text (no automatic table layout in Blinc).
* Table column alignment is not applied; a text area of the editor is not monospace.

## Dependencies

`blinc_app`, `blinc_cn`, `blinc_core`, `blinc_layout`, `blinc_theme` (0.6), `wgpu` 26 and `image` 0.25 (both already part of Blinc's tree, used for the offscreen picture). No other repo.
