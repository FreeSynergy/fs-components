//! A window with nothing but the reader.
//!
//! Pattern: Facade over `WindowedApp` — a convenience for the simplest program. A program with its
//! own window, menus and gestures builds a [`ReaderWidget`] into its layout instead.

use std::sync::Arc;

use blinc_app::prelude::*;
use blinc_app::windowed::WindowedApp;

use super::error::ShellError;
use super::setup::{bundle, register_fonts};
use super::widget::ReaderWidget;
use crate::labels::Labels;
use crate::reader::DraftStore;
use crate::source::DocumentSource;

/// Window settings.
#[non_exhaustive]
#[derive(Clone)]
pub struct Options {
    /// Window title.
    pub title: String,
    /// Initial width in logical pixels.
    pub width: u32,
    /// Initial height in logical pixels.
    pub height: u32,
    /// The words of the interface.
    pub labels: Labels,
    /// Where Export writes to; `None` = no Export button.
    pub export_target: Option<String>,
    /// Where drafts are backed up; `None` = no backup.
    pub drafts: Option<Arc<dyn DraftStore>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            title: "Reader".into(),
            width: 1200,
            height: 800,
            labels: Labels::default(),
            export_target: None,
            drafts: None,
        }
    }
}

/// Open a window with the reader over `source` and run until it is closed.
///
/// # Errors
/// [`ShellError`] if the window or the graphics cannot be started.
pub fn launch(
    options: &Options,
    source: Arc<dyn DocumentSource>,
) -> std::result::Result<(), ShellError> {
    register_fonts();
    let mut widget = ReaderWidget::new(
        source,
        options.labels.clone(),
        options.export_target.clone(),
    );
    if let Some(store) = &options.drafts {
        widget = widget.drafts(Arc::clone(store));
    }
    let config = WindowConfig {
        title: options.title.clone(),
        width: options.width,
        height: options.height,
        ..Default::default()
    };
    WindowedApp::run_with_theme(
        config,
        bundle(),
        blinc_theme::detect_system_color_scheme(),
        move |ctx| widget.build(ctx.width, ctx.height),
    )
    .map_err(|e| ShellError(e.to_string()))
}

impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options")
            .field("title", &self.title)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("labels", &self.labels)
            .field("export_target", &self.export_target)
            .field("drafts", &self.drafts.as_ref().map(|_| "DraftStore"))
            .finish()
    }
}
