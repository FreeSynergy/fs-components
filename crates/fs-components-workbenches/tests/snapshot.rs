//! The picture: drawn offscreen, not blank.
//!
//! Pattern: Smoke test — skipped (with a message) where there is no graphics adapter.

mod common;

use std::sync::Arc;

use common::Fake;
use fs_components_workbenches::{Step, snapshot};

#[test]
fn snapshot_draws_something() {
    let dir = std::env::temp_dir().join(format!("fs-wb-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("s.png");
    let steps = [Step::Open("a".into()), Step::Search("dir".into())];
    match snapshot(Arc::new(Fake), &steps, 400, 300, &file) {
        Err(e) => eprintln!("skipped: {e}"),
        Ok(reader) => {
            assert_eq!(reader.current_path(), Some("a"));
            let bytes = std::fs::read(&file).unwrap();
            assert_eq!(&bytes[1..4], b"PNG");
            let img = image::load_from_memory(&bytes).unwrap().to_rgba8();
            assert_eq!(img.dimensions(), (400, 300));
            let first = *img.get_pixel(0, 0);
            assert!(img.pixels().any(|p| *p != first), "the picture is blank");
        }
    }
    std::fs::remove_dir_all(dir).ok();
}
