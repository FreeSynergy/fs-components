# fs-components - Design

Shared UI components of FreeSynergy on Blinc 0.6 (released crates, `blinc_* = "0.6"`, resolved 0.6.0). Rules: ADR 0005 (a widget is the core of an app), 0009 (Blinc only behind our API), 0015 (no second GUI framework), 0037 (a workbench is open, gestures come from the frontend), 0043 (patterns, small files), 0049 (components first, only what the next program needs).

## State

Version 0.1.0 (untagged until reviewed). First consumer: the wiki app (`fs-wiki-app`). The shared "Build first" widgets of `docs/en/components/planned.md` follow, one per commit: `avatar-group`, `transfer-list`, `sortable-list`, `lightbox`, `share-dialog`.

| Area | State |
|---|---|
| `search-field` (primitive): adapter over `blinc_cn` input, text kept in shared state | done |
| `text-view` (widget): neutral model (`Block`, `Inline`, `Mark`); headings, paragraphs, lists (also task items; bullet close to its text), quotes (nesting depth guarded at 24), code, tables, rules, emphasis (italic, drawn as a rich text), strong, strike, code span, links, hard breaks; unknown things arrive as `Block::Plain` and are shown as text. Text with marks is a flow of runs that wrap (also inside a long word) | done |
| Glyphs: characters Blinc's default fonts lack (Indic and Thai scripts, many emoji and symbols outside Blinc's bundled emoji subset) are set in an installed font that has them (`text_view/fallback.rs`, `fontdb` + `ttf-parser`; colour-vector emoji fonts such as the system "Noto Color Emoji" cannot be drawn by Blinc, so the monochrome "Noto Emoji" and the symbol fonts are used). On this machine all 469 distinct characters of `docs/en` have a font (`examples/glyph_coverage.rs`); 198 of them go through the fallback. A character no font has stays a box and is listed by `missing_glyphs()`. Cyrillic, Greek, CJK, Hebrew draw with the default fonts. **Arabic is drawn but not in reading order or right-aligned** (Blinc has no bidirectional text); Devanagari draws with wide word gaps | done, with that limit |
| `tree-view` (widget): adapter over `blinc_cn` tree (keys of the Blinc tree: arrows, Home, End, Enter) | done |
| `avatar-group` (widget): adapter over the `blinc_cn` avatar group; neutral `Person` (key, name, picture, presence); initials safe for any name; at most `max` circles (never more than 50), the rest is one "+N" circle and stays available as `hidden()`; a picture that does not exist is not handed to Blinc, so the initials show | done |
| `transfer-list` (widget): transfers given as data (`Transfer`: key, name, direction, state Queued/Running/Done/Failed(reason), done, total); progress bar (`blinc_cn` progress), size text, buttons by state (Cancel; Retry and Dismiss; Dismiss) that report `Outcome{key, action}` to the caller's callback. It moves nothing and remembers nothing; at most 100 rows drawn (`max_rows`), the rest is counted in a line; texts from `Labels`; long names and reasons are clipped, not wrapped | done |
| `sortable-list` (widget): lanes of neutral `Item`s (key, label, locked); lanes may be closed to items from others. Pure move logic `Sorter` (cursor, one item held, carried live, dropped = `Outcome::Moved{item, from, to}`, cancelled = back exactly where it was); every step answers with an `Outcome`, also refusals (locked, closed lane, edge, empty, busy). Keyboard through optional `Bindings` (default: arrows, Home/End, PageUp/PageDown, Space or Enter pick up and drop, Escape gives up; Ctrl and Alt are never taken). Pointer/touch: the frontend calls `grab`, `hover`, `drop_grabbed`, `cancel`, `move_item`; no pointer gesture is wired in the widget except click = cursor. The widget keeps a copy of the lanes: the caller applies `Moved` to its data, and `set_lanes` takes the data back (also to undo a move it refused). Long lanes draw a window of at most `max_rows` (default 200) around the cursor with "N more above/below" lines. Row content may be replaced by the caller (`content`) | done, see Not verified |
| `lightbox` (widget): neutral `Media` (key, source, caption, picture or not); pure `Viewer` (show, next, prev, first, last, optional wrap, set_items keeps the place by key); `LightBox` handle with a Blinc view: counter, Close/Previous/Next buttons (the edge button is greyed unless stepping wraps), the picture fitted inside the stage (Blinc image, `contain`, no crop), caption; optional `Bindings` (Left/Right/PageUp/PageDown step, Home/End jump, Escape closes). A picture path that is not a file, an item that is not a picture, and no items each show a note from `Labels`, never an empty stage. Does not open an overlay: the frontend places it at window size and removes it on `Outcome::Closed`. Not built: zoom and pan, video and other kinds (`Media::other` is announced), preloading of neighbours, touch swipe (a gesture: the frontend calls `command`) | done, see Not verified |
| `list-view` (widget): columns and rows from the caller through `RowSource` (`count`, `rows(first, limit)`, a failure is shown in place of the rows), fixed row height, only the rows in view exist as elements and are read a page at a time (8 pages of 64 kept). Selection reports the key through a callback; the caller draws it by `selected(key)`. `ListPosition` keeps the scroll place across rebuilds. Keys off by default, per widget through `keys::bind_keys`. Blinc's `virtual_list.rs` does not fit: it builds the first `window_size` rows once and a spacer, and never follows the scroll, so a scrolled list is blank. Used instead: what `blinc_cn`'s long select does (a box as tall as all rows, `for_each` over the window, `ScrollRef::watch_y`). 100,000 and 10,000,000 rows drawn offscreen; wheel, click and keys driven in-process through the router. Above about 500,000 rows of 32 px the positions are f32 with steps of 2 px and more (at 10 million: 32 px); nothing breaks but rows sit unevenly. Its sheet is `list_view::STYLES`; not yet in `fs_components_widgets::styles()` | done |
| `reader` (workbench): `DocumentSource` trait, `Labels` (all texts come from the caller), state machine `Reader` (open, follow, back, search, edit, export) | done |
| Embedding: `ReaderWidget::new(source, labels, export_target)` and `.build(w, h)` place the reader in a program's own window and layout; `.reader()` gives the state to menus and commands; `launch` is a convenience window; `snapshot` is a picture without a window | done |
| Editing: `DocumentSource::{can_edit(path), source_text -> EditableText{text, revision}, save(path, text, revision)}`. A save with an old revision is a `Conflict`: nothing is written, the draft stays and the reason is shown. Save without changes writes nothing. The text area cannot hold some text exactly (it drops a final newline, turns CRLF into LF): the draft is compared and saved in the form of the original (an untouched text is saved as the original, exactly), and what saving would change anyway (mixed line ends become one kind, blank lines at the end are dropped) is announced above the editor before the person saves. A final newline cannot be edited: Enter at the very end of a text that ends in a newline counts as no change. Unsaved changes block leaving the document and refuse `follow` (`Outcome::Blocked`). Cancel with unsaved changes asks first (second Cancel discards); Esc is not handled at all, so it can never throw a draft away. After a conflict the draft stays in view and the next Save replaces the other version knowingly. After a save the list, the title and the search results are read again; a failed list stays visible | done |
| Export: `can_export(path)`, `export(path, target, overwrite)`; the target is chosen by the caller; a taken target is a `Conflict` and the reader asks (press Export again to replace; the confirmation belongs to that document and that target and is cleared by editing) | done |
| Scrolling: every column on the way down has `min_h(0)`; a long document is clipped and scrolls (3000 paragraphs draw). Tested offscreen with a scroll offset | done |
| Embedding keys: Alt+Left and Ctrl+S act on every reader in the window (keys are broadcast); `ReaderWidget::keys(false)` switches them off, the program then calls `with_reader` itself (state access is a closure: no lock can be held across calls) | done |
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
| `widgets/avatar_group` | Adapter + Value objects | `blinc_cn` avatar group behind `AvatarGroup`/`Person`; the people who do not fit are counted by Blinc's own "+N" circle (empty circles stand in for them) |
| `widgets/transfer_list` | Adapter + State + Observer | `blinc_cn` progress and buttons; the state decides the buttons; the callback is the outcome channel |
| `widgets/sortable_list/{state,moves,steps}` | State machine + Command | pure, no Blinc; `Command` is the one vocabulary for keys and gestures |
| `widgets/sortable_list/bindings` | Strategy | key to command; optional and replaceable |
| `widgets/sortable_list/{handle,view}` | Facade + Observer + Adapter | `SortableList` handle (clonable, no guard returned, copies out), outcome callback, Blinc view |
| `widgets/keys`, `widgets/live` | Adapter + Guard, Observer | `Key` is ours; `bind_keys` lets a key reach a handler only while that element has the keyboard (a broadcast reaches all handlers; two lists must never both move); `Live` redraws a view after a state change |
| `widgets/lightbox` | State + Command + Adapter | same shape as `sortable-list`: pure `Viewer`, optional `Bindings`, handle plus view; Blinc's image behind `Media` |
| `widgets/tree_view` | Adapter + Composite | `TreeNode` to the `blinc_cn` tree |
| `primitives/search_field` | Adapter | `blinc_cn` input; clone keeps its text across rebuilds |
| `workbenches/source.rs`, `error.rs`, `labels.rs` | Strategy / Repository, Value objects | what a workbench needs from outside (it knows no store, no wiki, no language) |
| `workbenches/shell/widget.rs` | Facade + Observer | `ReaderWidget`: shared state, element, redraw counters |
| `workbenches/reader` | State | `Shown` + transitions, testable without a window |
| `workbenches/view` | Adapter | state to elements, `Actions` callbacks (Observer) report what the person does |
| `workbenches/shell` | Facade | `launch` (window) and `snapshot` (picture); hide Blinc completely |

## Decisions

* **Layers.** `search-field` is a primitive (one value). `text-view` and `tree-view` are widgets: they know their data model when built (blocks and marks, labels). The `reader` is a workbench (ADR 0049: a subset of component): it gets its content at run time through `DocumentSource` and does not know what it means. Provisional: the crate split per layer and the name `reader` (see Open points).
* **The theme decides.** No colour or font is written in code. Sizes and colours are read from `ThemeState`; stylesheets only use `var(--...)` of the theme. The shell builds the theme bundle from `blinc_cn::cn_bundle()` plus our sheets.
* **Links.** The text view only reports `href` as written. Resolving it (relative path, external) is the data source's `resolve`, so the workbench does not know what a link means.
* **Gestures** are not here (ADR 0037); clicks and keys are the plain events of the framework.
* **Offscreen picture** uses Blinc's own `BlincApp::render_tree` into a texture and a PNG. It draws the same view functions as the window, from the same `Reader` state; clicks are not simulated.

## Application shell and how a program uses it

Two crates beside the three layers (provisional, Thread 4 of the sprint):

* `fs-components-render`: `bundle(css)`, `register_fonts()`, `draw_to_png(canvas, build, after_layout, path)`. Knows no workbench and no app, so both may use it. Template method: the order runtime, element, sheet, layout, draw, write is fixed; the caller supplies the element and an optional step after the layout. Variants of `RenderError` tell "no graphics adapter" from a real failure.
* `fs-components-app`: `Surface` (Strategy: anything the shell can draw), `Screen`, `Shell` (State, no window needed), `AppWidget` (Facade + Observer: one signal per live area, so a status message does not rebuild the workbench), `launch`, `AppWidget::picture`. It holds no words and no colours: labels come from the program, colours from theme variables. Sizes the surface must be told are constants in `styles.rs`.

A program's frontend (ADR 0005, 0009, 0037) is a crate `fs-<program>-app` in the program's repo: arguments, opening the core, labels, `Screen`s. It imports no `blinc_*` crate. Its work surface is a workbench in `fs-components` (open, knows no store and no program core, takes a trait), so another program can embed it.

Not built in the shell: menus, shortcuts for the navigation, window state across runs, compact variants, and a way to post to the interface thread from a worker (`select` and `set_status` must be called on it; a program with background work cannot report yet). Changing the screen is a full rebuild; only the status line is a live area. Tests open no window; `AppWidget::build`, which `launch` calls, is drawn offscreen and compared with `picture`. Clicks on the navigation are not simulated. `launch` ran once in a real Wayland window for 20 seconds without a crash or a log line (2026-11-06); that says it starts and keeps running, not that it looks right or reacts.

To try it by hand: `cargo run -p fs-components-app --example shell`. Check: the window shows the title, two navigation entries and the status line "Ready"; a click on "Second" changes the heading and the highlighted entry; Tab reaches the entries; resizing keeps the layout. `cargo run -p fs-components-app --example shell -- /tmp/shell.png` draws a picture and opens no window.

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

* Elements built outside a build pass (a `stateful` callback building nested `stateful` or `cn` elements) get a new key each time, because `key::reset_call_counters` only runs at the start of a rebuild. `STATEFUL_DEPS` never shrinks, so the old callbacks keep running. Found by reading the code, not measured. Workaround in the shell: a change of screen is a full rebuild. A `ReaderWidget` rebuilt inside a live area would be affected.
* `State::update` only reaches a `stateful` through a callback that `BlincContextState::init` does not install, and the window skips its own init when a picture came first; `Signal::update` works in both orders. Found by reading the code.
* On the visual update path a live area's own node keeps the classes of its first drawing (`renderer/build/subtree.rs`), and a child without a handler keeps the handler of the node it replaces. Found by reading the code. Workaround for the classes: they sit on an inner element.
* `blinc_noto_emoji::register()` adds the font again on each call (its doc says it does not).
* A rich text is measured in one font weight but bold and italic spans are drawn in another face, so lines with marks run past the edge or overlap. Workaround: marks are drawn as a flow of single-style runs.
* `RichText` links always call `open_url`; there is no handler for an internal link. Workaround: own click handler per run.
* Emoji without a font are drawn blank but take their width; the bundled subset (`blinc_noto_emoji::register()`) lacks many (box glyphs).
* `text(..).italic()` shows upright text; a rich text with an italic span does draw italic (used for emphasis). The `code` flag of a span draws no monospace or chip (`flow.rs` does it itself).
* Layout of a document of 3000 paragraphs without `min_h(0)` overruns a wgpu buffer and aborts; a plain paragraph of 30 KB takes seconds in Blinc's text measuring (reproduction in the report).
* `BlincContextState::init` panics when called twice, also across threads (two pictures at once): `snapshot` serialises itself.
* `Scroll` has no `flex_1`; a `min_w` next to `w(0.0)` is ignored. The text area wraps at a narrower width than its box.
* `blinc_cn::cn_styles` documents `register_cn_styles(ctx)`, which does not exist in 0.6.0 (the bundle `cn_bundle()` does it).

Blinc notes for `avatar-group` (0.6.0):

* `cn::avatar` shows either the picture or the initials: a picture that cannot be loaded leaves a grey disc and never falls back to the initials. Workaround: a local path that is not a file is not passed on (addresses cannot be checked without loading).
* A picture appears only after a few frames (it fades in); an offscreen test must draw several frames with a pause. Reproduction: draw one frame of `cn::avatar().src(<png>)` and the disc is white.
* Emoji without a font in the initials draw blank (same as the text notes above).

Notes on keys (Blinc 0.6.0): a window may deliver a key press to every key handler (broadcast), which is how the reader's Ctrl+S could act twice. Our `bind_keys` compares the handler's node with `keyboard_focused_node()` (falls back to the element's own focus flag when Blinc has none) and ignores keys while a text input has focus. Tested in process with two lists in one tree.

Blinc notes for `lightbox` (0.6.0): the image element loads the whole file at its full size (no downscaling on load) and shows nothing, not an error, for a file it cannot decode; only a missing local file is caught by us. An image fades in over its first frames; the first frame is empty. A `Button` has `id` only after `build_component()`.

Limits:

* `DocumentSource` is synchronous; a slow source blocks the interface thread.
* The tree is built completely (all nodes) once, because the Blinc tree is not lazy.
* Table columns share the width by the length of their longest text (no automatic table layout in Blinc).
* The reader's redraw counters are signals (`use_signal_keyed`), not `State`: `State::update` does not reach the live areas when a picture was drawn earlier in the same process (test `tests/refresh.rs` fails with `State`). A host must keep one `ReaderWidget` (see its `build` docs).
* The tree is rebuilt with every sidebar redraw and Blinc keys tree state by call site and counter, so folders that were opened by hand may close again; a blocked `open` can leave the tree highlight on the clicked row. Not fixed; needs a stable tree instance from Blinc.
* No test sends key events (Esc, Ctrl+S): Blinc offers no offscreen event dispatch we use; the handlers are two lines and Esc is not handled.
* Table column alignment is not applied; a table with many wide columns may be wider than its box (Blinc has no minimum content width, so words are kept whole instead of broken); the editor text area is not monospace.

## Dependencies

`blinc_app`, `blinc_cn`, `blinc_core`, `blinc_layout`, `blinc_theme` (0.6), `wgpu` 26 and `image` 0.25 (both already part of Blinc's tree, used for the offscreen picture). No other repo.
