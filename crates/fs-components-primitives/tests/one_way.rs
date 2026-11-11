//! One way for field data: no crate of this workspace makes the data of an input field or the watch of a scroll
//! position by calling Blinc directly, in its sources, tests, examples or benches; they all go through
//! `fs_components_primitives::field_data`. Data made directly while a page is built belongs to that build's scope and
//! is dead after the next one (Fall C, Blinc defect ec776dcf).
//!
//! What is found: the calls (also split over lines, and as `Type::function(` from the path), the constructors of the
//! data types (`new`, `with_placeholder`, `default`), the keyed variants, and an alias made with `use ... as`. What is
//! not: a macro that expands to such a call. `Default` is looked for in three shapes (`default_holes`): a statement that
//! names one of the data types and ends in `Default::default()` or `<path>::default()` (`let d: TextInputData =
//! Default::default()`, `let d: Arc<Mutex<TextInputData>> = Arc::default()`, `<TextInputData>::default()`), and a struct
//! with `#[derive(Default)]` that has a field of one of the types (it would make the data without the one way). **Not
//! found by a text search:** `Default::default()` whose type is only known by inference (a function argument, a field in
//! `..Default::default()` of a struct that is not itself derived) and a `Default` that is written by hand in an `impl`.
//! Two places are allowed by name (below), each with its reason.
//!
//! Pattern: none (tests). Reads the sources, needs nothing else.
#![allow(clippy::pedantic)]

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
const ALIASED: [&str; 6] = [
    "text_input_data",
    "text_input_data_with_placeholder",
    "text_area_state",
    "TextInputData",
    "TextAreaState",
    "ScrollRef",
];

/// The types whose values must come from the one way: a `Default` of any of them (or of a struct that holds one) makes
/// the data under the current build scope.
const DATA_TYPES: [&str; 5] = [
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

/// `Default` as a way to make the data: a statement that names a data type and calls a `default`, and a struct that
/// derives `Default` and holds a data type. Works on the code without comments and white space.
fn default_holes(squeezed: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = squeezed[from..].find("::default()") {
        let at = from + at;
        from = at + 1;
        let start = squeezed[..at].rfind([';', '{', '}']).map_or(0, |i| i + 1);
        let statement = &squeezed[start..at];
        if let Some(name) = DATA_TYPES.iter().find(|n| statement.contains(**n)) {
            found.push(format!("{statement}::default() with {name}"));
        }
    }
    let mut from = 0;
    while let Some(at) = squeezed[from..].find("#[derive(") {
        let at = from + at;
        from = at + 1;
        let Some(end) = squeezed[at..].find(")]") else {
            break;
        };
        let derives = &squeezed[at + "#[derive(".len()..at + end];
        if !derives
            .split(',')
            .any(|d| d == "Default" || d.ends_with("::Default"))
        {
            continue;
        }
        if let Some((name, body)) = struct_after(&squeezed[at + end + 2..])
            && let Some(data) = made_by_default(body)
        {
            found.push(format!("#[derive(Default)] on {name} holds {data}"));
        }
    }
    found
}

/// The data type that the `Default` of a struct with these fields would make: a field whose type is a collection
/// (`HashMap<_, SharedTextInputData>`, `Vec<..>`, `Option<..>`) is empty by default and makes nothing.
fn made_by_default(body: &str) -> Option<&'static str> {
    const EMPTY: [&str; 8] = [
        "Vec<",
        "VecDeque<",
        "HashMap<",
        "BTreeMap<",
        "HashSet<",
        "BTreeSet<",
        "Option<",
        "collections::",
    ];
    let inner = body.get(1..body.len().saturating_sub(1))?;
    let (mut fields, mut depth, mut from) = (Vec::new(), 0_i32, 0);
    for (n, c) in inner.char_indices() {
        match c {
            '<' | '(' | '{' | '[' => depth += 1,
            '>' | ')' | '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                fields.push(&inner[from..n]);
                from = n + 1;
            }
            _ => {}
        }
    }
    fields.push(&inner[from..]);
    fields.iter().find_map(|field| {
        // a named field is `name:Type`; a tuple field is `Type`
        let ty = field.split_once(':').map_or(*field, |(_, t)| t);
        let ty = ty.strip_prefix("pub").unwrap_or(ty);
        if EMPTY.iter().any(|e| ty.contains(e)) {
            return None;
        }
        DATA_TYPES.iter().find(|n| ty.contains(**n)).copied()
    })
}

/// The matching close of the bracket that opens `text` (which starts with the opening one).
fn closing(text: &str) -> usize {
    let (mut depth, mut i) = (0_i32, 0);
    for (n, c) in text.char_indices() {
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => {
                depth -= 1;
                if depth == 0 {
                    i = n;
                    break;
                }
            }
            _ => {}
        }
    }
    i
}

/// The name and the fields of the struct that follows (after more attributes and `pub`), if what follows is one.
fn struct_after(mut text: &str) -> Option<(String, &str)> {
    while text.starts_with("#[") {
        text = &text[closing(&text[1..]) + 2..];
    }
    text = text.strip_prefix("pub").unwrap_or(text);
    if text.starts_with('(') {
        text = &text[closing(text) + 1..];
    }
    let text = text.strip_prefix("struct")?;
    let open = text.find(['{', '(', ';'])?;
    let name = text[..open].to_owned();
    if text[open..].starts_with(';') {
        return Some((name, ""));
    }
    let body = &text[open..];
    Some((name, &body[..=closing(body)]))
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
        for hole in default_holes(&squeezed) {
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

#[test]
fn the_checker_sees_a_default_that_makes_the_data() {
    let holes = |text: &str| default_holes(&code(text).split_whitespace().collect::<String>());
    // the statement forms
    assert_eq!(holes("let d: TextInputData = Default::default();").len(), 1);
    assert_eq!(holes("let d: TextAreaState = Default::default();").len(), 1);
    assert_eq!(
        holes("let d: ScrollRef =\n    Default::default();").len(),
        1
    );
    assert_eq!(
        holes("let d: Arc<Mutex<TextInputData>> = Arc::default();").len(),
        1
    );
    assert_eq!(holes("let d = <TextInputData>::default();").len(), 1);
    assert_eq!(
        holes("let d = Arc::<Mutex<TextInputData>>::default();").len(),
        1
    );
    assert_eq!(
        holes("let d: SharedTextInputData = Default::default();").len(),
        1
    );
    // the derive, also with other attributes and a path
    assert_eq!(
        holes("#[derive(Default)]\nstruct S { data: SharedTextInputData }").len(),
        1
    );
    assert_eq!(
        holes("#[derive(Clone, Default)]\n#[allow(dead_code)]\npub(crate) struct S {\n    pub a: u8,\n    scroll: ScrollRef,\n}").len(),
        1
    );
    assert_eq!(
        holes("#[derive(std::default::Default)] struct S(TextAreaState);").len(),
        1
    );
    // a collection of data is empty by default: nothing is made
    assert!(
        holes("#[derive(Default)] struct S { lines: HashMap<String, SharedTextInputData>, n: u8 }")
            .is_empty()
    );
    assert_eq!(
        holes("#[derive(Default)] struct S { lines: Vec<u8>, d: Arc<Mutex<TextInputData>> }").len(),
        1
    );
    // what is fine
    assert!(holes("let d: u8 = Default::default();").is_empty());
    assert!(holes("let d = TextInputData::from(x); let e: Vec<u8> = Vec::default();").is_empty());
    assert!(holes("#[derive(Clone, Debug)]\nstruct S { data: SharedTextInputData }").is_empty());
    assert!(holes("#[derive(Default)]\nstruct S { a: u8, keep: FieldData }").is_empty());
    assert!(
        holes("#[derive(Default)]\nstruct S { a: u8 }\nstruct T { data: TextInputData }")
            .is_empty()
    );
    assert!(holes("// let d: TextInputData = Default::default();").is_empty());
}
