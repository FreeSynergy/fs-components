# fs-components-text-editor - Design

The text editor of the wiki as a component (ADR 0005, 0009, 0037, 0043, 0049, 0057, 0066). Markdown source with a toolbar and a preview. Reused, not copied: the preview is `fs-components-widgets::TextView` (blocks from `text`), redraws go through `live::Live`, the input is Blinc's text area as the reader's editor (`fs-components-workbenches`, `view/editor.rs`) and the composer use it. The reader's `Editing` (draft, save, conflict, backup) stays where it is; this crate is the layer a program puts on top of a text area. `fs-components-fields` does not exist on main, so nothing was built on it.

## State

First version, branch `r2-text-editor`, untagged. Model, trait and view are done and tested; what is missing is under Unsupported.

## Pattern map

| Where | Pattern | Why |
|-------|---------|-----|
| `model/command.rs` `EditCommand` | Command | closed set of operations, one `match` turns one into an `Edit` |
| `model/edit.rs` `Edit` | Command | the record of a command; `invert` undoes it, `between` makes one from typing |
| `model/state.rs` `EditorModel` | Command + Memento | history is two stacks of `Edit`; the saved text is the memento behind `is_dirty` |
| `model/selection.rs`, `counts.rs` | Value object | `start <= end`, counted in characters |
| `model/wrap.rs`, `lines.rs` | none | pure functions from text and selection to an edit |
| `editor.rs` `TextEditor` | Strategy | programs depend on the trait; `EditorModel` and `EditorView` implement it |
| `view/handle.rs` `EditorView` | Facade + Observer | one handle over model, text area and redraw; `OnChange` is the caller's closure |
| `view/area.rs` | Adapter | text area counts (line, column), the model counts characters |
| `view/toolbar.rs`, `build.rs` | View function | state to elements; a click only calls the handle |
| `Previewer` | Strategy | the caller's Markdown reader turns source into blocks |
| `labels.rs` | none | plain data, all words of the editor |

## Rules of the model

- Positions are characters (not bytes, not grapheme clusters). A combining accent is its own character.
- Bold `**`, italic `*`, code `` ` ``: a selection is wrapped; if the marks already surround it (or are at its two ends) they are removed. Runs of stars are counted (`***x***` is bold and italic), so italic does not take the stars of bold. No selection: an empty pair with the caret between; the same button removes it.
- Heading (level 1 to 6, 0 = off), list point `- ` (also `* `, `+ ` count), quote `> `: on every line the selection reaches; the same state on all lines takes it off. Empty lines between others are left alone.
- Link: `[selection](url)`, without selection `[](url)`.
- `set_text` loads: it is the saved text, the history is gone. `type_text` takes what was typed as one step (smallest change). History keeps 500 steps. `is_dirty` compares with the saved text, so undo back to it is clean.
- Line breaks are `\n`. The text area drops a final `\n`; the model keeps its own.

## Unsupported

- Markdown reader: the crate has none. The caller hands in a `Previewer` (the wiki's `fs-text`); there is no built-in fallback.
- Asking for the link address: the toolbar's Link button inserts `[selection]()` (no dialog, that is a gesture). A program calls `apply(EditCommand::Link(url))`.
- Keyboard shortcuts (Ctrl+B and so on), drag and drop, spell checking, side-by-side layout: the frontend's gestures (ADR 0037).
- Merging typed characters into one undo step; each report of the text area is one step.
- Numbered lists, task items, nested quotes, strike-through, tables as commands.
- Own style sheet: layout is plain Blinc builders; the colours come from the theme and `fs_components_widgets::styles()`.
- Hard line breaks `\r\n` are not treated specially.
