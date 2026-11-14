# Editor contract (fs-components-editor)

Several editor building blocks stand behind one trait, `TextEditor` (pattern: Strategy).

- **Input:** a `Doc`, the document in the text model (blocks of Markdown source, one blank line between, one line break at the end).
- **Output:** only change operations of the same model: `Op::InsertBlock`, `Op::RemoveBlock`, `Op::ReplaceBlock`.
- **Rule:** an editor changes data only through these operations and never reformats on its own. A document that is loaded and saved without a change is byte for byte the same, and every change is visible as an operation. `apply(loaded, take_ops())` is exactly `document()`.

## Building blocks

| Name | What it is | Status |
|---|---|---|
| `SourceEditor` | plain text (Markdown, code, configuration) in one text area; first implementation: the classic and Confluence-like editor | built |
| `BlockEditor` | Notion-like: every block a field of its own, Enter makes a new block, `/` opens the block menu | built (menu: block types only) |
| `RichTextEditor` | WYSIWYG | later |

A building block is not an editor program. Opening, saving and the choice of block and frontend belong to the program (later a facet of fs-text). The style is a layer of its own (ADR 0074); each block has a fixed name for its style parts.

## Why a crate of its own

The text model of the wiki (`fs-text`, repository fs-wiki) depends on fs-components, so fs-components cannot depend on it. The contract has small block-addressed operation types of its own; the wiki maps them onto its model.

## Conformance tests

One collection, `fs_components_editor::conformance::check`, is run by every implementation (`crates/fs-components-workbenches/tests/conformance.rs`): the text of the user of 2026-11-14 and the GFM cases (table alignment, task lists, `~strike~`, `~~strike~~`, `_italic_`, `*italic*`), round trip byte for byte, every edit visible as operations. A new building block has to pass it.

## Extension points (declared, not built)

Modules like hot plugins (ADR 0045), one trait per block: `SourceModule` (languages, language server, preview), `BlockModule` (block types, `/` menu entries), `RichModule` (marks, bar buttons), kept in a `Registry`. The traits are empty for now.

## Limits

Text that is not in the canonical form (several blank lines, no line break at the end) is normalised by the block editor; the fixtures are canonical.
