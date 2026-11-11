//! One way for field data: no crate of this workspace makes the data of an input field or the watch of a scroll
//! position by calling Blinc directly, in its sources, tests, examples or benches; they all go through
//! `fs_components_primitives::field_data`. Data made directly while a page is built belongs to that build's scope and
//! is dead after the next one (Fall C, Blinc defect ec776dcf).
//!
//! What is found: the calls (also split over lines, and as `Type::function(` from the path), the constructors of the
//! data types (`new`, `with_placeholder`, `default`), the keyed variants, and an alias made with `use ... as`. What is
//! not: a macro that expands to such a call. `Default` is looked for in `holes.rs` (`default_holes`): a statement that
//! names one of the data types and makes a value from nothing (`Default::default()`, `<path>::default()`,
//! `unwrap_or_default()`, `or_default()`), a function that returns one and makes it so, a type alias that hides one, and
//! a struct with `#[derive(Default)]` or `#[cfg_attr(.., derive(Default))]` that has a field of one of the types (it
//! would make the data without the one way; collections of them are empty by default and are fine). **Not found by a
//! text search:** a `Default` whose type is only known by inference (a function argument, `..Default::default()` of a
//! struct that is not itself derived, a value made in one statement and typed in the next), a `Default` written by hand
//! in an `impl`, and a macro.
//! Two places are allowed by name (below), each with its reason.
//!
//! Pattern: none (tests). Reads the sources, needs nothing else.
#![allow(clippy::pedantic)]

#[path = "one_way/holes.rs"]
mod holes;

use std::fs;
use std::path::{Path, PathBuf};

/// Looked for in the code with comments and all white space taken out.
const FORBIDDEN: [&str; 17] = [
    "text_input_data(",
    "text_input_data_with_placeholder(",
    "text_input_data_keyed",
    "text_input_data_for",
    "text_area_state(",
    "text_area_state_with_placeholder(",
    "TextInputData::new",
    "TextInputData::with_placeholder",
    "TextInputData::default",
    "TextAreaState::new",
    "TextAreaState::with_placeholder",
    "TextAreaState::default",
    "ScrollRef::new(",
    "ScrollRef::default",
    "watch_y(",
    "SharedTextAreaState::default",
    "SharedTextInputData::default",
];

/// The names that must not be renamed with `use ... as`.
const ALIASED: [&str; 8] = [
    "text_input_data",
    "text_input_data_with_placeholder",
    "text_area_state",
    "TextInputData",
    "TextAreaState",
    "ScrollRef",
    "SharedTextInputData",
    "SharedTextAreaState",
];

/// Files that may call Blinc directly, and why.
const ALLOWED: [(&str, &str); 2] = [
    (
        "fs-components-primitives/src/field_data.rs",
        "the one way itself",
    ),
    (
        "fs-components-primitives/tests/field_data.rs",
        "the control that shows the defect the one way removes",
    ),
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

/// The code without `//` comments and `/* */` comments.
fn code(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if rest.starts_with("/*") {
            rest = rest.find("*/").map_or("", |i| &rest[i + 2..]);
        } else if rest.starts_with("//") {
            rest = rest.find('\n').map_or("", |i| &rest[i..]);
        } else {
            let c = rest.chars().next().unwrap();
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    out
}

#[test]
fn no_source_makes_field_data_or_a_scroll_watch_outside_the_one_way() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut files = Vec::new();
    for entry in fs::read_dir(crates).unwrap() {
        let krate = entry.unwrap().path();
        for dir in ["src", "tests", "examples", "benches"] {
            if krate.join(dir).is_dir() {
                sources(&krate.join(dir), &mut files);
            }
        }
    }
    assert!(files.len() > 100, "the files were found: {}", files.len());
    let mut found = Vec::new();
    for file in files {
        let name = file
            .strip_prefix(crates)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if name == "fs-components-primitives/tests/one_way.rs"
            || name == "fs-components-primitives/tests/one_way/holes.rs"
            || ALLOWED.iter().any(|(allowed, _)| *allowed == name)
        {
            continue;
        }
        let code = code(&fs::read_to_string(&file).unwrap());
        let squeezed: String = code.split_whitespace().collect();
        for word in FORBIDDEN {
            if squeezed.contains(word) {
                found.push(format!("{name}: {word}"));
            }
        }
        for hole in holes::default_holes(&squeezed) {
            found.push(format!("{name}: {hole}"));
        }
        let spaced = code.split_whitespace().collect::<Vec<_>>().join(" ");
        for word in ALIASED {
            if spaced.contains(&format!("{word} as ")) {
                found.push(format!("{name}: `{word} as ...` hides the name"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "field data and scroll watches are made through fs_components_primitives::FieldData / scroll_ref / scroll_watch only:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_checker_sees_what_it_claims_to_see() {
    let tricky = "let a = ScrollRef\n    ::\n    new();\nscroll\n    .watch_y(1.0);\nuse x::text_area_state as tas;\nUFCS::watch_y(&s, 1.0);";
    let squeezed: String = code(tricky).split_whitespace().collect();
    assert!(squeezed.contains("ScrollRef::new("));
    assert!(squeezed.contains(".watch_y("));
    assert!(squeezed.contains("::watch_y("));
    assert!(
        code(tricky)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains("text_area_state as ")
    );
    assert!(!code("// text_area_state()\n/* ScrollRef::new() */ x").contains("text_area_state"));
}
