# FreeSynergy Components — one look, one feel

**This is a building block. It is why a task list, a contact list and a calendar behave the same way, and why you learn a thing once.**

FreeSynergy Components are the shared pieces of the screen: a search field, a tree, a list, a calendar view, a lightbox, charts. They are built on the GUI framework Blinc. Each FreeSynergy program uses these pieces for its window instead of drawing its own, so every program has the same keys, the same behaviour and, later, the same themes and translations.

What is different here: the pieces know nothing about our programs. A contact list does not know what a contact is; it gets rows and shows them. That keeps the engine and the frontend apart, so one engine can have several frontends, from very simple to complete. Words always come from the caller, so a program can be translated without touching a component.

## What it can do today

Latest tag: `v0.3.0`.

- Primitives and widgets: search field, text view, tree view, list view (only the visible rows exist), calendar view (month, week, day, agenda), avatar group, transfer list, sortable list, lightbox, breadcrumb, file tile and preview, a share dialog (dialog only), task card.
- Charts: a subset of Vega-Lite (bar, line, area, point, arc) with a text alternative and colour-blind-safe checking.
- Translatable text with plurals.
- Workbenches, each a full working surface for a program: reader (documents), people, calendar, tasks, file manager. All but the reader are marked provisional.
- Used by the first windows of Wiki, People, Tasks and Calendar.
- Try it without data: `cargo run -p fs-components-workbenches --example reader`, or add `-- --snapshot out.png` to draw a picture without a window.

## What is not there yet

- Accessibility, text selection, images in text and pointer cursors over links are not built. Tab reaches only part of the reader.
- The share dialog has nothing behind it: the sharing interface does not exist yet, so there is no permission model.
- Workbench sources are synchronous, so a slow program freezes its window.
- A handful of characters (some emoji) are not drawn correctly.
- Blinc is itself young; we keep its types behind our own API.

## Part of FreeSynergy

Components is what the windows of all programs are made of. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crates: `fs-components-primitives`, `fs-components-widgets`, `fs-components-workbenches`, `fs-components-app` (shared frontend helpers), `fs-components-charts`, `fs-components-words` (text, std only) and `fs-components-render`. The layers are described in `conventions/CONVENTIONS.md` of fs-documentation.
- Programs depend on this repo by git tag. No Blinc type crosses the component API.
- Build and test: `cargo build --workspace` and `cargo test --workspace`; some tests draw offscreen.
- Design and the full state table: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.
