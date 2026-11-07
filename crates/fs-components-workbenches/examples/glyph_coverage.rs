//! Which characters of a folder of Markdown files need a fallback font, and which no installed
//! font can draw.
//!
//! Pattern: none (a thin binary over `fs_components_widgets::fallback_family`).
//!
//! `cargo run -p fs-components-workbenches --example glyph_coverage [folder]`
fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/srv/FreeSynergy/fs-documentation/docs/en".into());
    let mut chars = std::collections::BTreeMap::<char, usize>::new();
    let mut stack = vec![std::path::PathBuf::from(root)];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p)
            } else if p.extension().is_some_and(|x| x == "md") {
                for c in std::fs::read_to_string(&p).unwrap_or_default().chars() {
                    *chars.entry(c).or_default() += 1;
                }
            }
        }
    }
    let (mut fallback, mut default) = (0, 0);
    for c in chars.keys() {
        match fs_components_widgets::fallback_family(*c) {
            Some(_) => fallback += 1,
            None => default += 1,
        }
    }
    let missing = fs_components_widgets::missing_glyphs();
    println!(
        "distinct characters: {}, set in a fallback font: {fallback}, default: {default}",
        chars.len()
    );
    println!("no installed font has: {}", missing.len());
    for c in missing {
        println!("  U+{:04X} {c} x{}", c as u32, chars[&c]);
    }
}
