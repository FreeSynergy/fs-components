//! What the standard menus refuse (a view with an id of the shell, an active frontend that is not in the list) and what the
//! language line says when the setting shared by all programs answers. Fake environment, temporary folders.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::sync::Arc;

use fs_components_app::{
    AppError, AppWidget, FrontendItem, LanguageChoice, LanguageControl, MenuEntry, Scope, Screen,
    StandardCallbacks, StandardMenus,
};
use fs_components_words::Locale;
use menu_common::fixture::Target;
use menu_common::standard::{fixture, labels, menus};

fn bare() -> AppWidget {
    AppWidget::new(
        "T",
        vec![Screen::new("one", "One", Arc::new(Target::default()))],
    )
    .unwrap()
}

#[test]
fn a_view_with_an_id_of_the_shell_is_refused_at_every_level_and_nothing_is_installed() {
    for id in ["fs.language.xx", "fs.frontend.board", "fs.quit", "fs."] {
        let app = bare();
        let mut standard = StandardMenus::new(Locale::english());
        standard.views = vec![MenuEntry::new(id, "A view")];
        let err = app
            .set_standard_menus(standard, StandardCallbacks::new(|_| {}))
            .unwrap_err();
        assert_eq!(err, AppError::ReservedMenuId(id.to_owned()));
        assert!(
            app.with_shell(|s| s.menus().is_empty()),
            "nothing was installed for {id}"
        );
    }
    // In a next level of a view too; counter-probe: ids that merely contain "fs" are fine.
    let app = bare();
    let mut standard = StandardMenus::new(Locale::english());
    standard.views = vec![MenuEntry::submenu(
        "more",
        "More",
        vec![MenuEntry::new("fs.x", "X")],
    )];
    assert!(matches!(
        app.set_standard_menus(standard, StandardCallbacks::new(|_| {})),
        Err(AppError::ReservedMenuId(_))
    ));
    let mut ok = StandardMenus::new(Locale::english());
    ok.views = vec![
        MenuEntry::new("fsx", "A"),
        MenuEntry::new("my.fs.view", "B"),
    ];
    assert!(
        app.set_standard_menus(ok, StandardCallbacks::new(|_| {}))
            .is_ok()
    );
}

#[test]
fn an_active_frontend_that_is_not_in_the_list_is_an_error_and_changes_nothing() {
    let f = fixture("unknown", "en_US.UTF-8");
    let before = menus(&f.h.app);
    let err =
        f.h.app
            .set_frontends(vec![FrontendItem::new("a", "A")], "b", |_| {})
            .unwrap_err();
    assert_eq!(err, AppError::UnknownFrontend("b".into()));
    assert_eq!(
        f.h.app.set_active_frontend("nowhere"),
        Err(AppError::UnknownFrontend("nowhere".into()))
    );
    assert_eq!(menus(&f.h.app), before, "the menus are as they were");
    // Counter-probe: a known one works.
    assert!(f.h.app.set_active_frontend("board").is_ok());
    // At the start too.
    let app = bare();
    let mut standard = StandardMenus::new(Locale::english());
    standard.frontends = vec![FrontendItem::new("a", "A")];
    standard.active_frontend = Some("z".into());
    assert_eq!(
        app.set_standard_menus(standard, StandardCallbacks::new(|_| {})),
        Err(AppError::UnknownFrontend("z".into()))
    );
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn the_system_line_says_so_when_the_setting_shared_by_all_programs_answers() {
    for (lang, plain, shared) in [
        (
            "en_US.UTF-8",
            "System language (default)",
            "Like all programs (Deutsch)",
        ),
        (
            "de_DE.UTF-8",
            "Systemsprache (Standard)",
            "Wie alle Programme (English)",
        ),
    ] {
        let f = fixture("shared-line", lang);
        let line = |f: &menu_common::standard::Fixture| {
            labels(&menus(&f.h.app)[0].entries[1].children)[0].to_owned()
        };
        assert_eq!(line(&f), plain, "{lang}: nothing set");
        let other = if lang.starts_with("en") { "de" } else { "en" };
        f.control
            .choose(
                Scope::AllPrograms,
                &LanguageChoice::Language(Locale::parse(other).unwrap()),
            )
            .unwrap();
        // The menus are made again by the program's next change; ask the shell to rebuild them with the same words.
        f.h.app.set_active_frontend("board").unwrap();
        assert_eq!(line(&f), shared, "{lang}: the shared setting answers");
        assert!(
            menus(&f.h.app)[0].entries[1].children[0].checked,
            "still the line that has no choice of its own"
        );
        let _ = std::fs::remove_dir_all(f.dir);
    }
}
