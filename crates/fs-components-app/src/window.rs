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
/// `FS_COLOR_SCHEME=light|dark` wins; else the KDE colour scheme (its name says "Dark"); else what the
/// system reports. Text and ground then always come from one scheme of one theme bundle.
fn color_scheme() -> blinc_theme::ColorScheme {
    scheme_from(
        std::env::var("FS_COLOR_SCHEME").ok().as_deref(),
        kde_scheme_name().as_deref(),
    )
    .unwrap_or_else(blinc_theme::detect_system_color_scheme)
}

fn scheme_from(env: Option<&str>, kde: Option<&str>) -> Option<blinc_theme::ColorScheme> {
    match env.map(str::to_ascii_lowercase).as_deref() {
        Some("dark") => return Some(blinc_theme::ColorScheme::Dark),
        Some("light") => return Some(blinc_theme::ColorScheme::Light),
        _ => {}
    }
    let name = kde?.to_ascii_lowercase();
    Some(if name.contains("dark") {
        blinc_theme::ColorScheme::Dark
    } else {
        blinc_theme::ColorScheme::Light
    })
}

/// `ColorScheme=` of `[General]` in `kdeglobals`, if there is a KDE configuration.
fn kde_scheme_name() -> Option<String> {
    let home = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".config")))?;
    let text = std::fs::read_to_string(home.join("kdeglobals")).ok()?;
    let mut in_general = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_general = line == "[General]";
        } else if in_general && let Some(name) = line.strip_prefix("ColorScheme=") {
            return Some(name.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_theme::ColorScheme::{Dark, Light};

    #[test]
    fn the_environment_wins_then_the_kde_name_then_nothing() {
        assert_eq!(scheme_from(Some("DARK"), Some("BreezeLight")), Some(Dark));
        assert_eq!(scheme_from(Some("light"), Some("BreezeDark")), Some(Light));
        assert_eq!(scheme_from(None, Some("BreezeDark")), Some(Dark));
        assert_eq!(scheme_from(None, Some("BreezeLight")), Some(Light));
        assert_eq!(scheme_from(Some("auto"), None), None, "left to the system");
        assert_eq!(scheme_from(None, None), None);
    }
}
