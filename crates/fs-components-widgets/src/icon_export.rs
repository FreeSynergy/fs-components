//! Our icon set as a theme package of the Icon Theme Specification: `index.theme` and an SVG per icon name.
//!
//! Pattern: none — a function that writes files. The package is what the repository `fs-icons` holds as the theme
//! "FreeSynergy": the names are the standard names (ours with the prefix `freesynergy-`), the files are plain SVG
//! without scripts and without references to anything outside.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use fs_components_styles::{Glyph, IconContext, Rgba, StyleId, style};

/// The contexts that have at least one glyph, in a fixed order.
const CONTEXTS: [IconContext; 6] = [
    IconContext::Actions,
    IconContext::MimeTypes,
    IconContext::Places,
    IconContext::Devices,
    IconContext::Status,
    IconContext::Categories,
];

/// The text of `index.theme` of the theme `name`.
pub fn index_theme(name: &str) -> String {
    let dirs: Vec<String> = CONTEXTS
        .iter()
        .filter(|c| Glyph::ALL.iter().any(|g| g.icon_context() == **c))
        .map(|c| format!("scalable/{}", c.directory()))
        .collect();
    let mut out = format!(
        "[Icon Theme]\nName={name}\nComment=The icons of FreeSynergy, drawn by us\nInherits=hicolor\nDirectories={}\n",
        dirs.join(",")
    );
    for c in CONTEXTS {
        if Glyph::ALL.iter().any(|g| g.icon_context() == c) {
            let _ = write!(
                out,
                "\n[scalable/{}]\nSize=24\nContext={}\nType=Scalable\nMinSize=8\nMaxSize=512\n",
                c.directory(),
                c.context()
            );
        }
    }
    out
}

/// Write the theme `name` into `dir`: `index.theme` and `scalable/<context>/<icon name>.svg` for every glyph.
///
/// # Errors
/// The error of the file system; what was written stays.
pub fn export_theme(dir: &Path, name: &str) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("index.theme"), index_theme(name))?;
    let set = style(StyleId::Admin).icons;
    for glyph in Glyph::ALL {
        let place = dir.join("scalable").join(glyph.icon_context().directory());
        std::fs::create_dir_all(&place)?;
        let svg = set.svg(glyph, Rgba::hex(0x0044_4444), Some(24.0));
        std::fs::write(place.join(format!("{}.svg", glyph.icon_name())), svg)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icon_theme::IconTheme;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("fs-icons-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_exported_theme_is_found_by_the_lookup_of_the_specification() {
        let base = scratch("export");
        export_theme(&base.join("FreeSynergy"), "FreeSynergy").unwrap();
        let theme = IconTheme::open(std::slice::from_ref(&base), "FreeSynergy").unwrap();
        for glyph in Glyph::ALL {
            let found = theme
                .lookup(glyph.icon_name(), 24)
                .unwrap_or_else(|| panic!("{}", glyph.icon_name()));
            assert!(found.ends_with(format!("{}.svg", glyph.icon_name())));
            let svg = std::fs::read_to_string(found).unwrap();
            assert!(
                !svg.contains("<script") && !svg.contains("href"),
                "plain svg"
            );
        }
        // a name with more parts finds the icon of its first parts (`edit-copy-symbolic` finds `edit-copy`)
        assert!(
            theme
                .lookup("edit-copy-symbolic", 16)
                .unwrap()
                .ends_with("edit-copy.svg")
        );
        assert!(theme.lookup("no-such-icon", 16).is_none());
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn a_theme_inherits_and_the_size_is_matched_or_the_closest_taken() {
        let base = scratch("inherit");
        let write = |p: &str, text: &str| {
            let path = base.join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            "child/index.theme",
            "[Icon Theme]\nName=c\nInherits=parent\nDirectories=16x16/apps,48x48/apps\n[16x16/apps]\nSize=16\nType=Fixed\n[48x48/apps]\nSize=48\nType=Fixed\n",
        );
        write(
            "parent/index.theme",
            "[Icon Theme]\nName=p\nDirectories=scalable/apps\n[scalable/apps]\nSize=48\nType=Scalable\nMinSize=8\nMaxSize=256\n",
        );
        write("child/16x16/apps/a.svg", "<svg/>");
        write("child/48x48/apps/a.svg", "<svg/>");
        write("parent/scalable/apps/only-parent.svg", "<svg/>");
        let theme = IconTheme::open(std::slice::from_ref(&base), "child").unwrap();
        assert!(
            theme.lookup("a", 16).unwrap().ends_with("16x16/apps/a.svg"),
            "exact size"
        );
        assert!(
            theme.lookup("a", 40).unwrap().ends_with("48x48/apps/a.svg"),
            "closest size"
        );
        assert!(
            theme
                .lookup("only-parent", 24)
                .unwrap()
                .ends_with("scalable/apps/only-parent.svg"),
            "inherited"
        );
        assert!(IconTheme::open(std::slice::from_ref(&base), "missing").is_none());
        let _ = std::fs::remove_dir_all(base);
    }
}
