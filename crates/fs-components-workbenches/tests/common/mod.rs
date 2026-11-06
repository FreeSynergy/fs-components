//! A fake data source for the tests.
//!
//! Pattern: Strategy (test double) — implements `DataSource` over a list in memory.

use fs_components_workbenches::{
    Block, DataSource, Document, Entry, Hit, Inline, LinkTarget, ReadError,
};

/// Pages `a`, `dir/b`, and `broken` (which cannot be read).
pub struct Fake;

impl DataSource for Fake {
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, ReadError> {
        Ok(["a", "broken", "dir/b"]
            .iter()
            .filter(|p| p.starts_with(prefix))
            .map(|p| Entry {
                path: (*p).into(),
                title: p.to_uppercase(),
            })
            .collect())
    }

    fn read(&self, path: &str) -> Result<Document, ReadError> {
        match path {
            "broken" => Err(ReadError("the file is damaged".into())),
            "a" | "dir/b" => Ok(Document {
                title: path.into(),
                blocks: vec![Block::Paragraph(Inline::plain(format!("text of {path}")))],
            }),
            _ => Err(ReadError("unknown".into())),
        }
    }

    fn resolve(&self, from: &str, href: &str) -> LinkTarget {
        match href {
            "up" => LinkTarget::Page("a".into()),
            "web" => LinkTarget::External("https://example.org".into()),
            other => LinkTarget::Missing(format!("{from} -> {other}")),
        }
    }

    fn find(&self, term: &str) -> Result<Vec<Hit>, ReadError> {
        if term == "boom" {
            return Err(ReadError("the index is gone".into()));
        }
        Ok(self
            .entries("")?
            .into_iter()
            .filter(|e| e.title.to_lowercase().contains(&term.to_lowercase()))
            .map(|e| Hit {
                path: e.path,
                title: e.title,
            })
            .collect())
    }

    fn start(&self) -> Option<String> {
        Some("dir/b".into())
    }
}
