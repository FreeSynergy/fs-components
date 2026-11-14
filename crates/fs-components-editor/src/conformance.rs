//! The conformance tests every editor building block has to pass: one collection, run by each implementation.
//!
//! Pattern: Template method — [`check`] takes a function that makes a fresh editor and runs the same cases against it.
//!
//! The cases: a document loaded and read back is byte for byte the same and no operation is made; every edit is visible
//! as operations that, applied to the loaded document, give exactly the document the editor holds (and no operations
//! when nothing changed); the fixtures are the finding text of the user of 2026-11-14 and the GFM cases (table
//! alignment, task lists, strike-through, emphasis).
//!
//! Limits: the cases test the model of the editor, not its drawing; text that is not in the canonical form of
//! [`Doc`] (several blank lines, no line break at the end) is not a fixture, because the editors normalise it by design.

use crate::{Doc, Edit, Op, TextEditor, apply};

/// The text of the user: headings 1 and 2, a paragraph with strike-through (`~x~` and `~~x~~`) and emphasis (`_x_`, `*x*`),
/// a table with centred, left and right columns and a checkbox in a cell (our extension, GFM has none there), a task list.
pub const USER_FIXTURE: &str = "# Heading one\n\n## Heading two\n\nA paragraph with ~struck~, ~~struck too~~, _italic_ and *italic too*.\n\n| Name | State | Done |\n| :---: | :--- | ---: |\n| one | open | [ ] |\n| two | done | [x] |\n\n- [ ] open task\n- [x] done task\n- a plain item\n";

/// The GFM cases, each in the canonical form: (name, text).
pub const GFM_FIXTURES: &[(&str, &str)] = &[
    (
        "table alignment",
        "| a | b | c |\n| :--- | :---: | ---: |\n| 1 | 2 | 3 |\n",
    ),
    (
        "table without alignment",
        "| a | b |\n| --- | --- |\n| 1 | 2 |\n",
    ),
    (
        "task list",
        "- [ ] one\n- [x] two\n\n1. [ ] numbered\n2. [x] done\n",
    ),
    (
        "strike and emphasis",
        "~one~ ~~two~~ _three_ *four* **five** __six__\n",
    ),
    (
        "code with a blank line",
        "```rust\nfn a() {}\n\nfn b() {}\n```\n\nafter\n",
    ),
    ("only a title", "# Title\n"),
];

/// Every fixture: the user's text first.
#[must_use]
pub fn fixtures() -> Vec<(&'static str, &'static str)> {
    let mut all = vec![("user text 2026-11-14", USER_FIXTURE)];
    all.extend(GFM_FIXTURES.iter().copied());
    all
}

fn loaded(make: &dyn Fn() -> Box<dyn TextEditor>, text: &str) -> (Box<dyn TextEditor>, Doc) {
    let doc = Doc::parse(text);
    let mut editor = make();
    editor.load(&doc);
    (editor, doc)
}

/// Run all cases against the editors `make` gives.
///
/// # Panics
/// With the name of the case and of the editor when one fails.
pub fn check(make: &dyn Fn() -> Box<dyn TextEditor>) {
    let name = make().name();
    for (case, text) in fixtures() {
        assert!(
            Doc::is_canonical(text),
            "{case}: the fixture is not canonical"
        );
        unchanged_stays_the_same(make, name, case, text);
        edits_are_visible_as_operations(make, name, case, text);
        switching_between_blocks_loses_nothing(make, name, case, text);
    }
}

/// Loading and reading back changes no byte, and no operation is made.
fn unchanged_stays_the_same(
    make: &dyn Fn() -> Box<dyn TextEditor>,
    name: &str,
    case: &str,
    text: &str,
) {
    let (mut editor, doc) = loaded(make, text);
    assert_eq!(
        editor.document(),
        doc,
        "{name}/{case}: the document changed by loading"
    );
    assert_eq!(
        editor.document().to_markdown(),
        text,
        "{name}/{case}: the text changed by loading and saving"
    );
    assert!(
        editor.take_ops().is_empty(),
        "{name}/{case}: operations without an edit"
    );
}

/// Every edit shows as operations that give the new document; an edit that changes nothing gives none.
fn edits_are_visible_as_operations(
    make: &dyn Fn() -> Box<dyn TextEditor>,
    name: &str,
    case: &str,
    text: &str,
) {
    let first_len = Doc::parse(text).blocks[0].len();
    let edits: [Vec<Edit>; 3] = [
        vec![Edit::Type {
            block: 0,
            at: first_len,
            text: " more".into(),
        }],
        vec![Edit::Enter {
            block: 0,
            at: first_len,
        }],
        vec![
            Edit::Enter {
                block: 0,
                at: first_len.min(2),
            },
            Edit::Type {
                block: 0,
                at: 0,
                text: "x".into(),
            },
        ],
    ];
    for (k, steps) in edits.iter().enumerate() {
        let (mut editor, doc) = loaded(make, text);
        for step in steps {
            editor.edit(step);
        }
        let ops: Vec<Op> = editor.take_ops();
        let mut replayed = doc.clone();
        apply(&mut replayed, &ops).unwrap_or_else(|e| panic!("{name}/{case}/{k}: {e}"));
        assert_eq!(
            replayed,
            editor.document(),
            "{name}/{case}/{k}: the operations do not give the document"
        );
        assert_eq!(
            ops.is_empty(),
            editor.document() == doc,
            "{name}/{case}/{k}: a change without operations or the reverse"
        );
        if k == 0 {
            assert!(!ops.is_empty(), "{name}/{case}: typing made no operation");
        }
        assert!(
            editor.take_ops().is_empty(),
            "{name}/{case}/{k}: operations are given once"
        );
    }
}

/// The group "switching between blocks": moving on from one block to the next (Enter, then typing elsewhere, then back) never
/// loses what a block held (the finding of the user test of 2026-11-14: the content of the block left behind vanished and came
/// back with the next key). Every character of the loaded document is still there, in order, after the edits, and the
/// operations still give exactly the document the editor holds.
fn switching_between_blocks_loses_nothing(
    make: &dyn Fn() -> Box<dyn TextEditor>,
    name: &str,
    case: &str,
    text: &str,
) {
    let doc = Doc::parse(text);
    let first_len = doc.blocks[0].len();
    let last = doc.blocks.len() - 1;
    let steps = [
        Edit::Enter {
            block: 0,
            at: first_len.min(2),
        },
        Edit::Type {
            block: last.min(1),
            at: 0,
            text: "x".into(),
        },
        Edit::Enter {
            block: last.min(1),
            at: 0,
        },
        Edit::Type {
            block: 0,
            at: 0,
            text: "y".into(),
        },
    ];
    let (mut editor, loaded) = loaded(make, text);
    for step in &steps {
        editor.edit(step);
        // After each step, not only at the end: every character of the loaded text is still held, in order.
        let held = editor.document().to_markdown();
        assert!(
            keeps_in_order(text, &held),
            "{name}/{case}: after {step:?} the editor lost content: {held:?}"
        );
    }
    let ops = editor.take_ops();
    let mut replayed = loaded;
    apply(&mut replayed, &ops).unwrap_or_else(|e| panic!("{name}/{case}: {e}"));
    assert_eq!(
        replayed,
        editor.document(),
        "{name}/{case}: the operations do not give the document"
    );
}

/// Are the characters of `original` (white space left out) all in `held` in the same order (others may lie between)?
fn keeps_in_order(original: &str, held: &str) -> bool {
    let mut rest = held.chars().filter(|c| !c.is_whitespace());
    original
        .chars()
        .filter(|c| !c.is_whitespace())
        .all(|c| rest.any(|h| h == c))
}
