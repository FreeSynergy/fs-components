//! One way for field data: no crate of this workspace makes the data of an input field or the watch of a scroll
//! position by calling Blinc directly; they all go through `fs_components_primitives::field_data`. Data made directly
//! while a page is built belongs to that build's scope and is dead after the next one (Fall C, Blinc defect
//! ec776dcf).
//!
//! Pattern: none (tests). Reads the sources, needs nothing else.
#![allow(clippy::pedantic)]

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: [&str; 8] = [
    "text_input_data(",
    "text_input_data_with_placeholder(",
    "text_area_state(",
    "TextInputData::new",
    "TextAreaState::new",
    "ScrollRef::new(",
    ".watch_y(",
    "SharedTextAreaState::default",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_source_makes_field_data_or_a_scroll_watch_outside_field_data_rs() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut files = Vec::new();
    for entry in fs::read_dir(crates).unwrap() {
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
    }
    assert!(files.len() > 50, "the sources were found");
    let mut found = Vec::new();
    for file in files {
        if file.ends_with("fs-components-primitives/src/field_data.rs") {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for word in FORBIDDEN {
                if line.contains(word) {
                    found.push(format!("{}:{}: {word}", file.display(), n + 1));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "field data and scroll watches are made through fs_components_primitives::FieldData / scroll_ref / scroll_watch only:\n{}",
        found.join("\n")
    );
}
