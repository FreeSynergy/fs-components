//! The window of the generated frontend: the standard shell of ADR 0065 around the surfaces.
//!
//! Pattern: Facade — one call builds the shell with a screen per data type and the standard menus
//! (Program, View, Help); the menu entries are fixed, their handler is this file's. Words come from [`Say`].
//!
//! Not yet (named in `DESIGN.md`): shortcuts Ctrl+Q and F1 (the shell's menus have none), the Settings entry (this
//! window is the settings), Fullscreen.

use std::sync::Arc;

use fs_components_app::{AppError, AppWidget, Menu, MenuEntry, Screen};
use fs_components_fields::Say;
use fs_components_widgets::open_external;

use crate::view::{AboutInfo, AdminSurface};

/// Ids of the standard entries.
pub const QUIT: &str = "program-quit";
/// Opens the help address.
pub const HELP: &str = "help-help";
/// Shows the About page.
pub const ABOUT: &str = "help-about";
/// Prefix of the entries that show a data type.
pub const VIEW_PREFIX: &str = "view-";

/// One data type in the window.
pub struct TypeView {
    /// Stable id (the type name).
    pub id: String,
    /// Translation key of the navigation label.
    pub label: String,
    /// Its surface.
    pub surface: Arc<AdminSurface>,
}

/// The standard menus for these views: Program (Quit), View (one entry per type), Help (Help, About).
pub fn standard_menus(say: &dyn Say, views: &[TypeView]) -> Vec<Menu> {
    vec![
        Menu::new(
            say.say("menu.program"),
            vec![MenuEntry::new(QUIT, say.say("menu.quit"))],
        ),
        Menu::new(
            say.say("menu.view"),
            views
                .iter()
                .map(|v| MenuEntry::new(format!("{VIEW_PREFIX}{}", v.id), say.say(&v.label)))
                .collect(),
        ),
        Menu::new(
            say.say("menu.help"),
            vec![
                MenuEntry::new(HELP, say.say("menu.help")),
                MenuEntry::new(ABOUT, say.say("menu.about")),
            ],
        ),
    ]
}

/// The window: a screen per type, the standard menus, About and Help from `about` and `help_url`.
///
/// # Errors
/// [`AppError`] if two types have the same id.
pub fn admin_window(
    title: &str,
    say: &Arc<dyn Say>,
    views: Vec<TypeView>,
    about: AboutInfo,
    help_url: String,
) -> Result<AppWidget, AppError> {
    let screens = views
        .iter()
        .map(|v| {
            Screen::new(
                v.id.clone(),
                say.say(&v.label),
                Arc::clone(&v.surface) as Arc<dyn fs_components_app::Surface>,
            )
        })
        .collect();
    let app = AppWidget::new(title, screens)?;
    let menus = standard_menus(&**say, &views);
    let (shell, surfaces) = (
        app.clone(),
        views
            .into_iter()
            .map(|v| (v.id, v.surface))
            .collect::<Vec<_>>(),
    );
    app.set_menus(menus, move |id| {
        let current = || shell.with_shell(|s| s.current().map(|c| c.id.clone()));
        match id {
            QUIT => {
                // Nothing is kept unsaved but an open form. With text in one the window stays and says why (an error
                // message that stays until closed); a dialog that asks "leave anyway?" comes later.
                // the surface that is shown comes first, then the others in order
                let here = current();
                let unsaved = surfaces
                    .iter()
                    .filter(|(_, s)| s.has_unsaved())
                    .min_by_key(|(id, _)| Some(id) != here.as_ref());
                match unsaved {
                    Some((id, s)) => {
                        // the surface that has the text, not the one that happens to be shown
                        shell.select(id);
                        s.warn_unsaved();
                    }
                    None => std::process::exit(0),
                }
            }
            // `open_external` returns nothing: a failure to open the address cannot be told (named in DESIGN.md).
            HELP => open_external(&help_url),
            ABOUT => {
                if let Some(here) = current()
                    && let Some((_, s)) = surfaces.iter().find(|(i, _)| *i == here)
                {
                    s.set_about(Some(about.clone()));
                }
            }
            other => {
                if let Some(view) = other.strip_prefix(VIEW_PREFIX) {
                    shell.select(view);
                }
            }
        }
    })?;
    Ok(app)
}
