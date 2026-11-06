//! The reader over a fake document source: a few made-up documents, no store, no files.
//!
//! Pattern: Strategy (a `DocumentSource` implementation) — shows that the workbench needs nothing
//! but the trait.
//!
//! `cargo run -p fs-components-workbenches --example reader` opens a window;
//! `... --example reader -- --snapshot out.png` writes a picture instead (no window).

use std::sync::Arc;

use fs_components_workbenches::{
    Block, Document, DocumentSource, Entry, Hit, Inline, LinkTarget, ListItem, Mark, MarkKind,
    Options, Picture, SourceError, Step, launch, snapshot,
};

/// Documents by path.
struct Fake(Vec<(&'static str, &'static str, Vec<Block>)>);

impl DocumentSource for Fake {
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(self
            .0
            .iter()
            .filter(|(p, ..)| p.starts_with(prefix))
            .map(|(p, t, _)| Entry::new(*p, *t))
            .collect())
    }

    fn read(&self, path: &str) -> Result<Document, SourceError> {
        self.0
            .iter()
            .find(|(p, ..)| *p == path)
            .map(|(_, t, b)| Document::new(*t, b.clone()))
            .ok_or_else(|| SourceError::not_found(format!("no document {path}")))
    }

    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        if href.starts_with("http") {
            LinkTarget::External(href.into())
        } else if self.0.iter().any(|(p, ..)| *p == href) {
            LinkTarget::Document(href.into())
        } else {
            LinkTarget::Missing(href.into())
        }
    }

    fn find(&self, term: &str) -> Result<Vec<Hit>, SourceError> {
        let term = term.to_lowercase();
        Ok(self
            .0
            .iter()
            .filter(|(_, t, b)| {
                t.to_lowercase().contains(&term)
                    || b.iter()
                        .any(|b| b.plain_text().to_lowercase().contains(&term))
            })
            .map(|(p, t, _)| Hit::new(*p, *t))
            .collect())
    }
}

fn documents() -> Fake {
    let link = |href: &str| MarkKind::Link { href: href.into() };
    let intro = vec![
        Block::Heading {
            level: 1,
            text: Inline::plain("Welcome"),
        },
        Block::Paragraph(Inline::new(
            "This is strong, emphasised, code and a link to the guide.",
            vec![
                Mark::new(MarkKind::Strong, 8, 14),
                Mark::new(MarkKind::Emphasis, 16, 26),
                Mark::new(MarkKind::Code, 28, 32),
                Mark::new(link("guide/start"), 46, 55),
            ],
        )),
        Block::List {
            ordered: false,
            start: 1,
            items: vec![
                ListItem::new(vec![Block::Paragraph(Inline::plain("first"))]),
                ListItem::new(vec![Block::Paragraph(Inline::plain("done"))]).task(true),
            ],
        },
        Block::Quote(vec![Block::Paragraph(Inline::plain("A quotation."))]),
    ];
    let start = vec![
        Block::Heading {
            level: 2,
            text: Inline::plain("Getting started"),
        },
        Block::Code {
            lang: "rust".into(),
            text: "fn main() {\n    println!(\"hello\");\n}".into(),
        },
        Block::Table {
            align: vec![],
            rows: vec![
                vec![Inline::plain("Name"), Inline::plain("Meaning")],
                vec![Inline::plain("a"), Inline::plain("the first letter")],
            ],
        },
        Block::Paragraph(Inline::new(
            "Back to the welcome page.",
            vec![Mark::new(link("index"), 8, 11)],
        )),
    ];
    Fake(vec![
        ("index", "Welcome", intro),
        ("guide/start", "Getting started", start),
    ])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Arc::new(documents());
    let args: Vec<String> = std::env::args().collect();
    if let Some(at) = args.iter().position(|a| a == "--snapshot") {
        let path = args.get(at + 1).ok_or("--snapshot needs a file name")?;
        let mut picture = Picture::new(source, 900, 600);
        picture.steps = vec![Step::Follow("guide/start".into())];
        let shot = snapshot(picture, std::path::Path::new(path))?;
        println!("wrote {path}, showing {:?}", shot.reader.current_path());
        return Ok(());
    }
    launch(&Options::default(), source)?;
    Ok(())
}
