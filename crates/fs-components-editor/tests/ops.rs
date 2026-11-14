//! Tests of the model operations and of the collection itself: a plain reference editor passes it, a careless one fails.
//!
//! Tests: model tests, no window.

use fs_components_editor::conformance;
use fs_components_editor::{Doc, Edit, Op, TextEditor, apply, diff};

/// The simplest editor there is: it holds the blocks and edits them in place. It also tells how the collection catches a
/// careless editor (`sloppy` reformats on load).
struct Plain {
    blocks: Vec<String>,
    start: Vec<String>,
    sloppy: bool,
}

impl TextEditor for Plain {
    fn name(&self) -> &'static str {
        "plain"
    }
    fn load(&mut self, doc: &Doc) {
        self.blocks.clone_from(&doc.blocks);
        if self.sloppy {
            // Reformatting on its own: a trailing space where the person put none.
            self.blocks[0].push(' ');
        }
        self.start.clone_from(&doc.blocks);
    }
    fn document(&self) -> Doc {
        Doc::new(self.blocks.clone())
    }
    fn edit(&mut self, edit: &Edit) {
        match edit {
            Edit::Type { block, at, text } => self.blocks[*block].insert_str(*at, text),
            Edit::Enter { block, at } => {
                let tail = self.blocks[*block].split_off(*at);
                if !tail.is_empty() {
                    self.blocks.insert(block + 1, tail);
                }
            }
        }
    }
    fn take_ops(&mut self) -> Vec<Op> {
        let ops = diff(&Doc::new(self.start.clone()), &self.document());
        self.start.clone_from(&self.blocks);
        ops
    }
}

fn plain(sloppy: bool) -> Box<dyn TextEditor> {
    Box::new(Plain {
        blocks: Vec::new(),
        start: Vec::new(),
        sloppy,
    })
}

#[test]
fn the_collection_passes_a_plain_editor() {
    conformance::check(&|| plain(false));
}

#[test]
#[should_panic(expected = "the document changed by loading")]
fn the_collection_catches_an_editor_that_reformats() {
    conformance::check(&|| plain(true));
}

#[test]
fn text_goes_through_the_model_unchanged() {
    for (case, text) in conformance::fixtures() {
        assert_eq!(Doc::parse(text).to_markdown(), text, "{case}");
    }
    // Several blank lines and a missing line break at the end become the canonical form (stated limit).
    assert_eq!(Doc::parse("a\n\n\n\nb").to_markdown(), "a\n\nb\n");
}

#[test]
fn diff_and_apply_agree() {
    let docs = [
        vec![],
        vec!["a"],
        vec!["a", "b", "c"],
        vec!["a", "x", "b", "c"],
        vec!["b", "c"],
        vec!["a", "b", "c", "d", "e"],
        vec!["z"],
    ];
    for before in &docs {
        for after in &docs {
            let (b, a) = (
                Doc::new(before.iter().map(|s| (*s).into()).collect()),
                Doc::new(after.iter().map(|s| (*s).into()).collect()),
            );
            let ops = diff(&b, &a);
            let mut got = b.clone();
            apply(&mut got, &ops).unwrap();
            assert_eq!(got, a, "{before:?} -> {after:?}: {ops:?}");
            assert_eq!(ops.is_empty(), b == a);
        }
    }
}

#[test]
fn an_operation_outside_the_document_is_refused() {
    let mut doc = Doc::new(vec!["a".into()]);
    assert!(apply(&mut doc, &[Op::RemoveBlock { index: 3 }]).is_err());
    assert!(
        apply(
            &mut doc,
            &[Op::InsertBlock {
                index: 2,
                text: "x".into()
            }]
        )
        .is_err()
    );
    assert_eq!(doc.blocks, ["a"]);
}
