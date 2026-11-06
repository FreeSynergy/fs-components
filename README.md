# fs-components

The shared UI components of FreeSynergy, built on the GUI framework [Blinc](https://github.com/project-blinc/Blinc) (ADR 0009). Programs depend on this repo by git tag; Blinc types stay behind the component API (ADR 0009, ADR 0043).

Three crates, one per layer (`conventions/CONVENTIONS.md` §2 of fs-documentation):

| Crate | Layer | Contains |
|---|---|---|
| `fs-components-primitives` | 1 primitives | `search-field` |
| `fs-components-widgets` | 2 widgets | `text-view` (draws a neutral model of blocks and marks), `tree-view` |
| `fs-components-workbenches` | 3 workbenches | `reader`: lists, opens, follows and searches documents of any `DocumentSource`; window and offscreen picture |

Try it without any data: `cargo run -p fs-components-workbenches --example reader`, or `... --example reader -- --snapshot out.png` for a picture drawn without a window.

See [DESIGN.md](DESIGN.md). Licence: MIT OR Apache-2.0.
