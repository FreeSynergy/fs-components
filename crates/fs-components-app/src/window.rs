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
    WindowedApp::run_with_theme(
        config,
        bundle(&css),
        blinc_theme::detect_system_color_scheme(),
        move |ctx| app.build(ctx.width, ctx.height),
    )
    .map_err(|e| AppError::Window(e.to_string()))
}
