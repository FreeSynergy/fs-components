//! The reader over a fake data source: a few made-up pages, no store, no files.
//!
//! Pattern: Strategy (a `DataSource` implementation) — shows that the workbench needs nothing but
//! the trait.
//!
//! `cargo run -p fs-components-workbenches --example reader` opens a window;
//! `... --example reader -- --snapshot out.png` writes a picture instead (no window).

use std::sync::Arc;

use fs_components_workbenches::{
    Block, DataSource, Document, Entry, Hit, Inline, LinkTarget, ListItem, Mark, MarkKind, Options,
    ReadError, Step, launch, snapshot,
};

/// Pages by path.
struct Fake(Vec<(&'static str, &'static str, Vec<Block>)>);

impl DataSource for Fake {
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, ReadError> {
        Ok(self
            .0
            .iter()
            .filter(|(p, ..)| p.starts_with(prefix))
            .map(|(p, t, _)| Entry {
                path: (*p).into(),
                title: (*t).into(),
            })
            .collect())
    }

    fn read(&self, path: &str) -> Result<Document, ReadError> {
        self.0
            .iter()
            .find(|(p, ..)| *p == path)
            .map(|(_, t, b)| Document {
                title: (*t).into(),
                blocks: b.clone(),
            })
            .ok_or_else(|| ReadError(format!("no page {path}")))
    }

    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        if href.starts_with("http") {
            LinkTarget::External(href.into())
        } else if self.0.iter().any(|(p, ..)| *p == href) {
            LinkTarget::Page(href.into())
        } else {
            LinkTarget::Missing(href.into())
        }
    }

    fn find(&self, term: &str) -> Result<Vec<Hit>, ReadError> {
        let term = term.to_lowercase();
        Ok(self
            .0
            .iter()
            .filter(|(_, t, b)| {
                t.to_lowercase().contains(&term)
                    || b.iter()
                        .any(|b| b.plain_text().to_lowercase().contains(&term))
            })
            .map(|(p, t, _)| Hit {
                path: (*p).into(),
                title: (*t).into(),
            })
            .collect())
    }
}

fn pages() -> Fake {
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
                Mark::new(
                    MarkKind::Link {
                        href: "guide/start".into(),
                    },
                    46,
                    55,
                ),
            ],
        )),
        Block::List {
            ordered: false,
            start: 1,
            items: vec![
                ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(Inline::plain("first"))],
                },
                ListItem {
                    task: Some(true),
                    blocks: vec![Block::Paragraph(Inline::plain("done"))],
                },
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
            vec![Mark::new(
                MarkKind::Link {
                    href: "index".into(),
                },
                8,
                11,
            )],
        )),
    ];
    Fake(vec![
        ("index", "Welcome", intro),
        ("guide/start", "Getting started", start),
    ])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Arc::new(pages());
    let args: Vec<String> = std::env::args().collect();
    if let Some(at) = args.iter().position(|a| a == "--snapshot") {
        let path = args.get(at + 1).ok_or("--snapshot needs a file name")?;
        let reader = snapshot(
            source,
            &[Step::Follow("guide/start".into())],
            900,
            600,
            std::path::Path::new(path),
        )?;
        println!("wrote {path}, showing {:?}", reader.current_path());
        return Ok(());
    }
    launch(&Options::default(), source)?;
    Ok(())
}
