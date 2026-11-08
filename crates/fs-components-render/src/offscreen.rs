//! An element as a picture, drawn without a window.
//!
//! Pattern: Facade over Blinc's offscreen renderer; one function keeps the fixed order (runtime,
//! element, stylesheet, layout, draw, write) with two steps supplied by the caller.

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_app::{BlincApp, BlincConfig};
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::{ElementBuilder, RenderTree};

use crate::error::RenderError;
use crate::setup::{bundle, register_fonts};

/// The size of a picture and the style sheet to draw it with.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    /// Width in pixels (not 0).
    pub width: u32,
    /// Height in pixels (not 0).
    pub height: u32,
    /// The caller's style sheet (theme variables only).
    pub css: String,
}

impl Canvas {
    /// A canvas of `width` x `height` with the style sheet `css`.
    pub fn new(width: u32, height: u32, css: impl Into<String>) -> Self {
        Self {
            width,
            height,
            css: css.into(),
        }
    }
}

/// Draw the element made by `build` to a PNG at `path`.
///
/// `build` gets the width and height and runs after the runtime and the text measurer are up, because
/// elements read theme and reactive state and measure their text. `after_layout` runs once the layout
/// is known and before drawing, for a direct change such as scroll physics; a change sent through a
/// signal is not drawn, because no frame runs. Not while a window of this process is open: it resets
/// that window's theme. The picture is on the light theme. Only one picture is drawn at a time: Blinc keeps its state in process-wide singletons.
///
/// # Errors
/// [`RenderError`] if the size is 0, there is no graphics adapter, the style sheet does not parse,
/// or the picture cannot be drawn or written.
pub fn draw_to_png<E, B, L>(
    canvas: &Canvas,
    build: B,
    after_layout: L,
    path: &Path,
) -> Result<(), RenderError>
where
    E: ElementBuilder + 'static,
    B: FnOnce(f32, f32) -> E,
    L: FnOnce(),
{
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let _guard = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
    let (width, height) = (canvas.width, canvas.height);
    if width == 0 || height == 0 {
        return Err(RenderError::EmptyCanvas);
    }
    init_runtime(&canvas.css);
    register_fonts();
    #[allow(clippy::cast_precision_loss)] // picture sizes are far below 2^23
    let (w, h) = (width as f32, height as f32);
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .map_err(|e| RenderError::NoGraphics(e.to_string()))?;
    blinc_layout::init_text_measurer();
    let ui = build(w, h);
    let mut tree = RenderTree::from_element(&ui);
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let mut css = String::new();
    for sheet in &bundle(&canvas.css).css_sources {
        css.push_str(sheet);
        css.push('\n');
    }
    let sheet = Stylesheet::parse_with_variables(&css, &vars)
        .map_err(|e| RenderError::Stylesheet(e.to_string()))?;
    tree.set_stylesheet(sheet);
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    after_layout();
    let texture = offscreen_texture(&app, width, height);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app.render_tree(&tree, &view, width, height)
        .map_err(|e| RenderError::Draw(e.to_string()))?;
    let rgba = app
        .read_texture_to_rgba8(&texture, width, height)
        .map_err(|e| RenderError::Draw(e.to_string()))?;
    image::save_buffer(path, &rgba, width, height, image::ColorType::Rgba8)
        .map_err(|e| RenderError::Write(e.to_string()))
}

/// What the window runner sets up before the first frame: theme and reactive state.
fn init_runtime(css: &str) {
    // `FS_RENDER_SCHEME=dark` draws with the dark scheme, to look at both without a window.
    let scheme = if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        blinc_theme::ColorScheme::Dark
    } else {
        blinc_theme::ColorScheme::Light
    };
    blinc_theme::ThemeState::init(bundle(css), scheme);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
}

/// A texture to draw into and read back from.
fn offscreen_texture(app: &BlincApp, width: u32, height: u32) -> wgpu::Texture {
    app.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("fs-components picture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: app.texture_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
