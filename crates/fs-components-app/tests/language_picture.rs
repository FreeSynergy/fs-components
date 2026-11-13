//! The language gate for the shell: the same window with the standard menus drawn once per language (fake environment
//! `LANG=de_DE.UTF-8` and `LANG=en_US.UTF-8`), closed, with Program > Language open and with View > Frontends open.
//!
//! Tests: Pixel tests — skipped (with a message) where there is no graphics adapter. The pictures stay in `FS_COMP_SHOTS` if that is
//! set (`shell-<language>-bar.png`, `-language.png`, `-frontends.png`); the strategy looks at them. A comparison against stored
//! reference pictures (`fs-components-pictures` of branch frontend-umbau-3: `Group::check`) is not used here: that crate is not on
//! this branch and the group of the shell (frontend-umbau group 10) is paused, so the pictures are looked at and asserted by
//! pixel difference only.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use fs_components_app::{
    AppWidget, FileLanguageControl, FrontendItem, MenuEntry, Screen, StandardCallbacks,
    StandardMenus,
};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_words::{LanguageSetting, Locale};
use image::RgbaImage;
use menu_common::fixture::{Target, entry};
use menu_common::{Harness, composed};

const SIZE: (u32, u32) = (640, 300);
const SIZE_F: (f32, f32) = (640.0, 300.0);

fn app(lang: &str, dir: &std::path::Path) -> AppWidget {
    let env: HashMap<String, String> = HashMap::from([("LANG".to_owned(), lang.to_owned())]);
    let tag = |t| Locale::parse(t).unwrap();
    let control = Arc::new(FileLanguageControl::new(
        Some(LanguageSetting::new(dir.join("own").join("language"))),
        None,
        Arc::new(env),
        vec![tag("de"), tag("en")],
    ));
    let app = AppWidget::new(
        "FreeSynergy Demo",
        vec![Screen::new("one", "One", Arc::new(Target::default()))],
    )
    .unwrap();
    let mut standard = StandardMenus::new(fs_components_app_resolve(&control));
    standard.views = vec![
        MenuEntry::new("tiles", "Tiles"),
        MenuEntry::new("list", "List"),
    ];
    standard.frontends = vec![
        FrontendItem::new("storefront", "Storefront"),
        FrontendItem::new("compact", "Compact"),
    ];
    standard.active_frontend = Some("storefront".into());
    standard.language = Some(control);
    app.set_standard_menus(standard, StandardCallbacks::new(|_| {}))
        .unwrap();
    app
}

fn fs_components_app_resolve(control: &FileLanguageControl) -> Locale {
    use fs_components_app::LanguageControl;
    control.resolve().chosen.locale
}

fn out_dir() -> PathBuf {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-app-lang-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn draw(app: &AppWidget, name: &str) -> Option<RgbaImage> {
    draw_in(app, name, SIZE)
}

fn draw_in(app: &AppWidget, name: &str, size: (u32, u32)) -> Option<RgbaImage> {
    let path = out_dir().join(name);
    let canvas = Canvas::new(size.0, size.1, app.styles());
    match draw_to_png(&canvas, |w, h| composed(app, w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
            Some(picture)
        }
    }
}

/// Closed, language submenu open, frontends submenu open.
fn three(lang: &str, short: &str) -> Option<[RgbaImage; 3]> {
    let dir = std::env::temp_dir().join(format!("fs-app-lang-pic-{short}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut h = Harness::new(app(lang, &dir), SIZE_F.0, SIZE_F.1);
    let closed = draw(&h.app, &format!("shell-{short}-bar.png"))?;
    h.click("fs-app-menu-0");
    h.click(&entry("fs.language"));
    let language = draw(&h.app, &format!("shell-{short}-language.png"))?;
    h.click("fs-app-menu-0");
    h.settle();
    h.click("fs-app-menu-1");
    h.click(&entry("fs.frontends"));
    let frontends = draw(&h.app, &format!("shell-{short}-frontends.png"))?;
    let _ = std::fs::remove_dir_all(dir);
    Some([closed, language, frontends])
}

#[test]
fn the_standard_menus_in_german_and_english_differ_in_the_words_and_open_a_next_level() {
    let (Some(de), Some(en)) = (three("de_DE.UTF-8", "de"), three("en_US.UTF-8", "en")) else {
        return;
    };
    let bar_differs = (0..SIZE.0).any(|x| de[0].get_pixel(x, 22) != en[0].get_pixel(x, 22));
    assert!(
        bar_differs,
        "the words of the bar are drawn in the language"
    );
    for (lang, [closed, language, frontends]) in [("de", &de), ("en", &en)] {
        for (name, open) in [("language", language), ("frontends", frontends)] {
            // An open menu with its next level covers part of the work area that the closed window shows empty.
            let covered = (60..SIZE.1 - 40)
                .any(|y| (150..560).any(|x| open.get_pixel(x, y) != closed.get_pixel(x, y)));
            assert!(covered, "{lang}: the {name} menu is drawn over the window");
        }
        assert_ne!(language, frontends, "{lang}: two different menus");
    }
    for (x, y) in [(20, 280), (620, 200)] {
        assert_eq!(
            de[0].get_pixel(x, y),
            en[0].get_pixel(x, y),
            "the work area at {x},{y} is the same in both languages"
        );
    }
}

/// A window 520 px wide: the next level of View > Frontends has no room on the right and folds to the left of its panel.
#[test]
fn a_narrow_window_folds_the_next_level_to_the_left_and_keeps_the_panel_under_its_entry() {
    for (lang, short) in [("en_US.UTF-8", "en"), ("de_DE.UTF-8", "de")] {
        let dir =
            std::env::temp_dir().join(format!("fs-app-lang-narrow-{short}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut h = Harness::new(app(lang, &dir), 520.0, SIZE_F.1);
        h.click("fs-app-menu-1");
        h.click(&entry("fs.frontends"));
        let (tx, _, _, _) = h.bounds("fs-app-menu-1");
        let (px, _, _, _) = h.bounds("fs-app-menu-panel");
        let (sx, _, sw, _) = h.bounds("fs-app-menu-sub-panel");
        assert!(
            (px - tx).abs() < 1.0,
            "{lang}: the panel stays under its entry"
        );
        assert!(sx >= 0.0 && sx + sw <= 520.0, "{lang}: inside the window");
        if draw_in(
            &h.app,
            &format!("shell-{short}-frontends-520.png"),
            (520, SIZE.1),
        )
        .is_none()
        {
            return;
        }
        let _ = std::fs::remove_dir_all(dir);
    }
}
