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

/// What makes a value out of nothing: `Default::default()`, `T::default()`, `.unwrap_or_default()`, `.or_default()`,
/// `.unwrap_or_else(Default::default)`.
const MAKERS: [&str; 4] = [
    "::default()",
    "unwrap_or_default()",
    "or_default()",
    "Default::default",
];

/// `Default` as a way to make the data. Works on the code without comments and white space. Found:
/// * a statement that names a data type and makes a value from nothing (`MAKERS`), also `<T>::default()` and
///   `Arc::<Mutex<T>>::default()`;
/// * a function whose return type names a data type and whose body uses one of `MAKERS`
///   (`fn f() -> TextInputData { Default::default() }`);
/// * a type alias that names a data type (`type X = TextInputData;`: the name is hidden from every other search);
/// * a struct with `#[derive(Default)]` or `#[cfg_attr(.., derive(Default))]` that has a field of a data type.
pub fn default_holes(squeezed: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut start = 0;
    for (end, c) in squeezed.char_indices() {
        if !matches!(c, ';' | '{' | '}') {
            continue;
        }
        let statement = &squeezed[start..end];
        start = end + 1;
        if let (Some(name), Some(maker)) = (
            DATA_TYPES.iter().find(|n| statement.contains(**n)),
            MAKERS.iter().find(|m| statement.contains(**m)),
        ) {
            found.push(format!("{statement} makes {name} with {maker}"));
        }
        let bare = statement.strip_prefix("pub").unwrap_or(statement);
        let bare = bare
            .strip_prefix(|c| c == '(')
            .map_or(bare, |_| bare.find(')').map_or(bare, |i| &bare[i + 1..]));
        if bare.starts_with("type")
            && let Some(name) = DATA_TYPES.iter().find(|n| statement.contains(**n))
        {
            found.push(format!("{statement} hides {name} behind a type alias"));
        }
    }
    // a function that returns a data type and makes it with a maker in its body
    let mut from = 0;
    while let Some(at) = squeezed[from..].find("->") {
        let at = from + at;
        from = at + 2;
        let Some(open) = squeezed[from..].find(['{', ';']) else {
            break;
        };
        let (ret, rest) = squeezed[from..].split_at(open);
        if !rest.starts_with('{') || !DATA_TYPES.iter().any(|n| ret.contains(*n)) {
            continue;
        }
        let body = &rest[..=closing(rest)];
        if let Some(maker) = MAKERS.iter().find(|m| body.contains(**m)) {
            found.push(format!("a function returning {ret} uses {maker}"));
        }
    }
    // a derive of Default, also inside cfg_attr
    let mut from = 0;
    while let Some(at) = squeezed[from..].find("#[") {
        let at = from + at;
        from = at + 2;
        let end = closing(&squeezed[at + 1..]) + at + 2;
        let attribute = squeezed.get(at..end).unwrap_or("");
        let derives_default = attribute.find("derive(").is_some_and(|i| {
            attribute[i..]
                .split([',', '(', ')'])
                .any(|d| d == "Default" || d.ends_with("::Default"))
        });
        if !derives_default {
            continue;
        }
        if let Some((name, body)) = struct_after(&squeezed[end..])
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
    // the ways round
    assert_eq!(
        holes("let d: SharedTextInputData = Arc::default();").len(),
        1
    );
    assert_eq!(
        holes("let d = Some(x).unwrap_or_default(); let _: TextInputData = d;").len(),
        0,
        "two statements: not seen"
    );
    assert_eq!(
        holes("let d: TextInputData = opt.unwrap_or_default();").len(),
        1
    );
    assert_eq!(
        holes("let d: TextAreaState = map.entry(k).or_default();").len(),
        1
    );
    assert_eq!(
        holes("let d: ScrollRef = opt.unwrap_or_else(Default::default);").len(),
        1
    );
    assert_eq!(
        holes("fn f() -> TextInputData { Default::default() }").len(),
        1
    );
    assert_eq!(
        holes("fn f() -> SharedTextInputData {\n    Arc::default()\n}").len(),
        1
    );
    assert_eq!(holes("fn f() -> TextInputData { make() }").len(), 0);
    assert_eq!(holes("fn f() -> u8 { Default::default() }").len(), 0);
    assert_eq!(holes("type Data = SharedTextInputData;").len(), 1);
    assert_eq!(
        holes("pub(crate) type Data = Arc<Mutex<TextInputData>>;").len(),
        1
    );
    assert_eq!(holes("type Names = Vec<String>;").len(), 0);
    assert_eq!(
        holes("#[cfg_attr(test, derive(Debug, Default))]\nstruct S { d: SharedTextInputData }")
            .len(),
        1
    );
    assert_eq!(
        holes("#[cfg_attr(test, derive(Debug))]\nstruct S { d: SharedTextInputData }").len(),
        0
    );
}
