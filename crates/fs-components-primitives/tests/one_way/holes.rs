//! The search for a `Default` that makes field data (see `one_way.rs`).
//!
//! Pattern: none (tests). Reads text only. This file is skipped by the scan of the sources: its self-test holds the
//! very statements the scan looks for.
#![allow(clippy::pedantic)]

/// The types whose values must come from the one way: a `Default` of any of them (or of a struct that holds one) makes
/// the data under the current build scope.
const DATA_TYPES: [&str; 5] = [
    "TextInputData",
    "TextAreaState",
    "ScrollRef",
    "SharedTextInputData",
    "SharedTextAreaState",
];

/// `Default` as a way to make the data: a statement that names a data type and calls a `default`, and a struct that
/// derives `Default` and holds a data type. Works on the code without comments and white space.
pub fn default_holes(squeezed: &str) -> Vec<String> {
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
fn the_checker_sees_a_default_that_makes_the_data() {
    let holes = |text: &str| default_holes(&text.split_whitespace().collect::<String>());
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
}
