//! The neutral text model: plain text of every block kind.
//!
//! Pattern: Table-driven tests.

use fs_components_widgets::{Align, Block, Inline, ListItem, Mark, MarkKind};

#[test]
fn plain_text_of_inline_blocks() {
    let h = Block::Heading {
        level: 2,
        text: Inline::plain("Title"),
    };
    let p = Block::Paragraph(Inline::new(
        "bold word",
        vec![Mark::new(MarkKind::Strong, 0, 4)],
    ));
    assert_eq!(h.plain_text(), "Title");
    assert_eq!(p.plain_text(), "bold word");
}

#[test]
fn plain_text_goes_into_lists_quotes_and_tables() {
    let list = Block::List {
        ordered: true,
        start: 3,
        items: vec![
            ListItem {
                task: None,
                blocks: vec![Block::Paragraph(Inline::plain("one"))],
            },
            ListItem {
                task: Some(false),
                blocks: vec![Block::Paragraph(Inline::plain("two"))],
            },
        ],
    };
    assert_eq!(list.plain_text(), "one\ntwo");
    let quote = Block::Quote(vec![list.clone(), Block::Plain("x".into())]);
    assert_eq!(quote.plain_text(), "one\ntwo\nx");
    let table = Block::Table {
        align: vec![Align::Left],
        rows: vec![
            vec![Inline::plain("a"), Inline::plain("b")],
            vec![Inline::plain("c"), Inline::plain("d")],
        ],
    };
    assert_eq!(table.plain_text(), "a\tb\nc\td");
    assert_eq!(Block::Rule.plain_text(), "");
}
