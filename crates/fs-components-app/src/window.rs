//! The shell in a window.
//!
//! Pattern: Facade over `WindowedApp`.

use blinc_app::prelude::WindowConfig;
use blinc_app::windowed::WindowedApp;
use fs_components_render::{bundle, register_fonts};

use crate::error::AppError;
use crate::styles::MIN_WINDOW;
use crate::widget::AppWidget;

/// Window settings. The title comes from the [`AppWidget`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowOptions {
    /// Initial width in logical pixels.
    pub width: u32,
    /// Initial height in logical pixels.
    pub height: u32,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
        }
    }
}

/// Open a window with this shell and run until it is closed.
///
/// # Errors
/// [`AppError::Window`] if the window or the graphics cannot be started.
pub fn launch(options: WindowOptions, app: AppWidget) -> Result<(), AppError> {
    register_fonts();
    let config = WindowConfig {
        title: app.with_shell(|shell| shell.title().to_owned()),
        width: options.width,
        height: options.height,
        min_size: Some(MIN_WINDOW),
        ..Default::default()
    };
    let css = app.styles();
    WindowedApp::run_with_theme(config, bundle(&css), color_scheme(), move |ctx| {
        app.build(ctx.width, ctx.height)
    })
    .map_err(|e| AppError::Window(e.to_string()))
}

/// The colour scheme the window starts with, named explicitly instead of left to Blinc's detection:
/// `FS_COLOR_SCHEME=light|dark` wins; else, on a KDE desktop, a KDE colour scheme whose name says
/// "Dark" gives dark; everything else is what the system reports. (A KDE scheme without "dark" in its
/// name says nothing: it is left to the system, so a leftover `kdeglobals` cannot turn a dark desktop
/// light.)
fn color_scheme() -> blinc_theme::ColorScheme {
    let kde_desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .is_ok_and(|d| d.to_ascii_uppercase().split(':').any(|part| part == "KDE"));
    scheme_from(
        std::env::var("FS_COLOR_SCHEME").ok().as_deref(),
        kde_desktop.then(kde_scheme_name).flatten().as_deref(),
    )
    .unwrap_or_else(blinc_theme::detect_system_color_scheme)
}

fn scheme_from(env: Option<&str>, kde: Option<&str>) -> Option<blinc_theme::ColorScheme> {
    match env.map(str::to_ascii_lowercase).as_deref() {
        Some("dark") => return Some(blinc_theme::ColorScheme::Dark),
        Some("light") => return Some(blinc_theme::ColorScheme::Light),
        _ => {}
    }
    kde.is_some_and(|name| name.to_ascii_lowercase().contains("dark"))
        .then_some(blinc_theme::ColorScheme::Dark)
}

/// An absolute directory from the environment; an empty or relative value is ignored (XDG).
fn absolute_dir(value: Option<std::ffi::OsString>) -> Option<std::path::PathBuf> {
    value
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
}

/// `ColorScheme=` of `[General]` in `kdeglobals`, if there is a KDE configuration and the name is not empty.
fn kde_scheme_name() -> Option<String> {
    let home = absolute_dir(std::env::var_os("XDG_CONFIG_HOME"))
        .or_else(|| absolute_dir(std::env::var_os("HOME")).map(|h| h.join(".config")))?;
    let text = std::fs::read_to_string(home.join("kdeglobals")).ok()?;
    general_color_scheme(&text)
}

fn general_color_scheme(text: &str) -> Option<String> {
    let mut in_general = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_general = line == "[General]";
        } else if in_general && let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            if key.trim() == "ColorScheme" && !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_theme::ColorScheme::{Dark, Light};

    #[test]
    fn the_environment_wins_a_dark_kde_name_gives_dark_everything_else_is_the_systems() {
        assert_eq!(scheme_from(Some("DARK"), Some("BreezeLight")), Some(Dark));
        assert_eq!(scheme_from(Some("light"), Some("BreezeDark")), Some(Light));
        assert_eq!(scheme_from(None, Some("BreezeDark")), Some(Dark));
        assert_eq!(
            scheme_from(None, Some("BreezeLight")),
            None,
            "left to the system"
        );
        assert_eq!(scheme_from(Some("auto"), None), None);
        assert_eq!(scheme_from(None, None), None);
    }

    #[test]
    fn kdeglobals_is_read_where_it_says_so() {
        let text = "[Other]\r\nColorScheme=Wrong\r\n[General]\r\nColorScheme = BreezeDark\r\n";
        assert_eq!(general_color_scheme(text).as_deref(), Some("BreezeDark"));
        assert_eq!(
            general_color_scheme("[General]\nColorScheme=\n"),
            None,
            "empty is nothing"
        );
        assert_eq!(general_color_scheme("[Other]\nColorScheme=X\n"), None);
    }

    #[test]
    fn an_empty_or_relative_directory_is_ignored() {
        use std::ffi::OsString;
        assert_eq!(absolute_dir(Some(OsString::new())), None);
        assert_eq!(absolute_dir(Some(OsString::from("rel/dir"))), None);
        assert!(absolute_dir(Some(OsString::from("/abs"))).is_some());
        assert_eq!(absolute_dir(None), None);
    }
}
