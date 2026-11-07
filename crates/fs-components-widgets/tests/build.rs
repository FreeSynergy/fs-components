//! Every block kind can be built into an element, also with odd marks.
//!
//! Tests: Smoke tests against the real theme (no window needed).

use std::sync::Arc;

use fs_components_widgets::{
    Block, Inline, ListItem, Mark, MarkKind, TextView, TreeNode, TreeView,
};

fn all_blocks() -> Vec<Block> {
    let link = MarkKind::Link {
        href: "a.md".into(),
    };
    vec![
        Block::Heading {
            level: 9,
            text: Inline::plain("level out of range is clamped"),
        },
        Block::Paragraph(Inline::new(
            "grüße 😀 mit Link und Code",
            vec![
                Mark::new(link, 0, 5),
                Mark::new(MarkKind::Code, 6, 99),
                Mark::new(MarkKind::Strike, 3, 3),
            ],
        )),
        Block::List {
            ordered: false,
            start: 1,
            items: vec![ListItem::default()],
        },
        Block::Quote(vec![Block::Rule]),
        Block::Code {
            lang: String::new(),
            text: "a\n\nb\n".into(),
        },
        Block::Table {
            align: vec![],
            rows: vec![vec![Inline::plain("only")], vec![]],
        },
        Block::Plain("<html> as text".into()),
    ]
}

#[test]
fn text_view_builds_every_block_kind() {
    blinc_theme::ThemeState::init_default();
    let handler: fs_components_widgets::LinkHandler = Arc::new(|_| {});
    let _ = TextView::new(all_blocks()).on_link(handler).build();
    let _ = TextView::new(Vec::new()).build();
}

#[test]
fn tree_node_counts_itself_and_children() {
    let mut root = TreeNode::new("a/", "a");
    root.children.push(TreeNode::new("a/b", "b"));
    assert_eq!(root.count(), 2);
    let _ = TreeView::new(vec![root]).selected("a/b").build();
}
