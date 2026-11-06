# fs-components - Design

Shared UI components of FreeSynergy on Blinc 0.6 (released crates, `blinc_* = "0.6"`, resolved 0.6.0). Rules: ADR 0005 (a widget is the core of an app), 0009 (Blinc only behind our API), 0015 (no second GUI framework), 0037 (a workbench is open, gestures come from the frontend), 0043 (patterns, small files), 0049 (components first, only what the next program needs).

## State

Version 0.1.0 (untagged until reviewed). Built for one consumer: the wiki app (`fs-wiki-app`). Nothing else is planned or built here yet.

| Area | State |
|---|---|
| `search-field` (primitive): adapter over `blinc_cn` input, text kept in shared state | done |
| `text-view` (widget): neutral model (`Block`, `Inline`, `Mark`); headings, paragraphs, lists (also task items), quotes (nesting depth guarded at 24), code, tables, rules, emphasis (italic, drawn as a rich text), strong, strike, code span, links, hard breaks; unknown things arrive as `Block::Plain` and are shown as text. Text with marks is a flow of runs that wrap (also inside a long word) | done |
| `tree-view` (widget): adapter over `blinc_cn` tree (keys of the Blinc tree: arrows, Home, End, Enter) | done |
| `reader` (workbench): `DocumentSource` trait, `Labels` (all texts come from the caller), state machine `Reader` (open, follow, back, search, edit, export) | done |
| Embedding: `ReaderWidget::new(source, labels, export_target)` and `.build(w, h)` place the reader in a program's own window and layout; `.reader()` gives the state to menus and commands; `launch` is a convenience window; `snapshot` is a picture without a window | done |
| Editing: `DocumentSource::{can_edit(path), source_text -> EditableText{text, revision}, save(path, text, revision)}`. A save with an old revision is a `Conflict`: nothing is written, the draft stays and the reason is shown. Save without changes writes nothing. The text area's changes of line ends and final newline are undone before comparing and saving. Unsaved changes block leaving the document and refuse `follow` (`Outcome::Blocked`). Cancel with unsaved changes asks first (second Cancel discards); Esc is not handled at all, so it can never throw a draft away. After a save the list, the title and the search results are read again | done |
| Export: `can_export(path)`, `export(path, target, overwrite)`; the target is chosen by the caller; a taken target is a `Conflict` and the reader asks (press Export again to replace) | done |
| Scrolling: every column on the way down has `min_h(0)`; a long document is clipped and scrolls (3000 paragraphs draw). Tested offscreen with a scroll offset | done |
| Keyboard: tree keys (Blinc), Alt+Left back, Ctrl+S save, Tab to search field and buttons. Search results, links and the tree rows are **not** reachable by Tab (the Blinc tree is no Tab stop; links are click targets) | partly |
| Pointer cursor over links; scrolling to a `#fragment`; table column alignment; selecting text; images; accessibility | not built |

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
| `workbenches/source.rs`, `error.rs`, `labels.rs` | Strategy / Repository, Value objects | what a workbench needs from outside (it knows no store, no wiki, no language) |
| `workbenches/shell/widget.rs` | Facade + Observer | `ReaderWidget`: shared state, element, redraw counters |
| `workbenches/reader` | State | `Shown` + transitions, testable without a window |
| `workbenches/view` | Adapter | state to elements, `Actions` callbacks (Observer) report what the person does |
| `workbenches/shell` | Facade | `launch` (window) and `snapshot` (picture); hide Blinc completely |

## Decisions

* **Layers.** `search-field` is a primitive (one value). `text-view` and `tree-view` are widgets: they know their data model when built (blocks and marks, labels). The `reader` is a workbench (ADR 0049: a subset of component): it gets its content at run time through `DataSource` and does not know what it means. Provisional: the crate split per layer and the name `reader` (see Open points).
* **The theme decides.** No colour or font is written in code. Sizes and colours are read from `ThemeState`; stylesheets only use `var(--...)` of the theme. The shell builds the theme bundle from `blinc_cn::cn_bundle()` plus our sheets.
* **Links.** The text view only reports `href` as written. Resolving it (relative path, external) is the data source's `resolve`, so the workbench does not know what a link means.
* **Gestures** are not here (ADR 0037); clicks and keys are the plain events of the framework.
* **Offscreen picture** uses Blinc's own `BlincApp::render_tree` into a texture and a PNG. It draws the same view functions as the window, from the same `Reader` state; clicks are not simulated.

## Public types and Blinc

* **Closed types are avoided:** public enums and structs that a later feature will extend are `#[non_exhaustive]` (`Block`, `MarkKind`, `Align`, `LinkTarget`, `Outcome`, `Shown`, `Step`, `ErrorKind`, `Entry`, `Hit`, `Document`, `Labels`, `Options`, `Picture`, ...) and come with constructors; build `Labels`/`Options` by `Default` and assigning fields.
* **Blinc in signatures:** the elements our functions return are Blinc elements, because a component is placed in a Blinc layout and a program that embeds the reader owns a Blinc window (ADR 0009: Blinc is the GUI). They appear only as `impl Element` (`TextView::build`, `SearchField::build`, `ReaderWidget::build`), and `Element` is one re-export in `fs-components-widgets`; no other Blinc type is public (data types are ours). Wrapping the trait in a newtype would copy its whole interface for no gain today; revisit when a second GUI backend is real.
* **Blocking:** `DocumentSource` is synchronous and the reader calls it **on the interface thread**; a slow source freezes the window while it works. This is provisional (a later version makes it async or runs calls on a worker). Numbers for the wiki are in `fs-wiki` DESIGN.md.
* **Closing the window loses the draft** of an unsaved edit: Blinc has no close hook (listed as a Blinc item). "Unsaved changes block leaving" means leaving a document, not the window.

## Open points

Questions for the owner (all provisional, easy to reverse):

* Names: `reader` for the workbench and the three crate names `fs-components-{primitives,widgets,workbenches}`.
* Layer of `text-view`: widget (chosen) or workbench, because it will later take any document.
* Tree labels show the document's title, folders show their last path part.

Blinc notes (0.6.0; to be passed upstream by the owner, reproductions in the report of the first release):

* A rich text is measured in one font weight but bold and italic spans are drawn in another face, so lines with marks run past the edge or overlap. Workaround: marks are drawn as a flow of single-style runs.
* `RichText` links always call `open_url`; there is no handler for an internal link. Workaround: own click handler per run.
* Emoji without a font are drawn blank but take their width; the bundled subset (`blinc_noto_emoji::register()`) lacks many (box glyphs).
* `text(..).italic()` shows upright text; a rich text with an italic span does draw italic (used for emphasis). The `code` flag of a span draws no monospace or chip (`flow.rs` does it itself).
* Layout of a document of 3000 paragraphs without `min_h(0)` overruns a wgpu buffer and aborts; a plain paragraph of 30 KB takes seconds in Blinc's text measuring (reproduction in the report).
* `BlincContextState::init` panics when called twice, also across threads (two pictures at once): `snapshot` serialises itself.
* `Scroll` has no `flex_1`; a `min_w` next to `w(0.0)` is ignored. The text area wraps at a narrower width than its box.
* `blinc_cn::cn_styles` documents `register_cn_styles(ctx)`, which does not exist in 0.6.0 (the bundle `cn_bundle()` does it).

Limits:

* `DataSource` is synchronous; a slow source blocks the interface thread.
* The tree is built completely (all nodes) once, because the Blinc tree is not lazy.
* Table columns share the width by the length of their longest text (no automatic table layout in Blinc).
* Table column alignment is not applied; a table with many wide columns may be wider than its box (Blinc has no minimum content width, so words are kept whole instead of broken); the editor text area is not monospace.

## Dependencies

`blinc_app`, `blinc_cn`, `blinc_core`, `blinc_layout`, `blinc_theme` (0.6), `wgpu` 26 and `image` 0.25 (both already part of Blinc's tree, used for the offscreen picture). No other repo.
