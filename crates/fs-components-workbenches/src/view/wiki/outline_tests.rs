//! The title is one thing: the big title, the first heading and the outline entry are the same line, shown once.
//!
//! Pattern: none (tests of the pure outline functions and of the block editor's title block).

use fs_components_widgets::{Block, Document, Inline};

use super::blocks::BlockEditor;
use super::outline::{body, headings, leads_with_title, outline_entries};

fn heading(level: u8, text: &str) -> Block {
    Block::Heading {
        level,
        text: Inline::plain(text),
    }
}

fn paragraph(text: &str) -> Block {
    Block::Paragraph(Inline::plain(text))
}

#[test]
fn a_title_heading_is_drawn_once_and_is_the_first_outline_entry() {
    let doc = Document::new("Handbook", vec![heading(1, "Handbook"), paragraph("Text")]);
    assert!(leads_with_title(&doc));
    assert_eq!(body(&doc), [paragraph("Text")]);
    assert_eq!(headings(&doc), []);
    // "No headings" must not show for a page that has a title heading.
    assert_eq!(outline_entries(&doc), [(1, "Handbook".to_string())]);
}

#[test]
fn headings_below_the_title_follow_it_in_the_outline() {
    let doc = Document::new(
        "Handbook",
        vec![
            heading(1, "Handbook"),
            heading(2, "Start"),
            heading(3, "Deep"),
        ],
    );
    assert_eq!(
        outline_entries(&doc),
        [
            (1, "Handbook".into()),
            (2, "Start".into()),
            (3, "Deep".into())
        ]
    );
    assert_eq!(headings(&doc), [(2, "Start".into()), (3, "Deep".into())]);
}

#[test]
fn a_first_heading_that_says_something_else_stays_in_the_text() {
    let doc = Document::new("Handbook", vec![heading(1, "Welcome"), paragraph("Text")]);
    assert!(!leads_with_title(&doc));
    assert_eq!(body(&doc).len(), 2);
    assert_eq!(outline_entries(&doc), [(1, "Welcome".to_string())]);
}

#[test]
fn a_page_without_headings_has_an_empty_outline() {
    let doc = Document::new("Notes", vec![paragraph("Text")]);
    assert_eq!(outline_entries(&doc), []);
}

#[test]
fn the_block_editor_knows_when_its_first_block_is_the_title() {
    assert!(BlockEditor::new(1, "# Handbook\n\nText").lead_heading());
    assert!(!BlockEditor::new(1, "## Start\n\nText").lead_heading());
    assert!(!BlockEditor::new(1, "Text\n\n# Later").lead_heading());
}
