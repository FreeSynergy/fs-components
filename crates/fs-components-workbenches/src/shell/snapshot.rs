//! The reader as a picture, drawn without a window.
//!
//! Pattern: Facade over Blinc's offscreen renderer. The same view functions as the window draw
//! the same `Reader` state; the steps below change the state the way a person's actions would
//! (but no click is simulated: the handlers of the elements are not exercised here).

use std::sync::{Arc, Mutex};

use blinc_app::{BlincApp, BlincConfig};
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::RenderTree;
use fs_components_primitives::SearchField;

use super::error::ShellError;
use super::setup::bundle;
use crate::reader::{Outcome, Reader};
use crate::source::DataSource;
use crate::view::{Actions, content, frame, search_box, side_body, tree_nodes};

/// One thing a person does, to be applied before the picture is taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Open the document at this path.
    Open(String),
    /// Follow this link in the document shown.
    Follow(String),
    /// Go back.
    Back,
    /// Search for this text.
    Search(String),
}

/// Apply `steps` to a reader over `source` and write a PNG of `width` x `height` to `path`.
/// Returns the reader, so a caller can say what it ended up showing.
///
/// # Errors
/// [`ShellError`] if there is no graphics adapter or the picture cannot be written.
pub fn snapshot(
    source: Arc<dyn DataSource>,
    steps: &[Step],
    width: u32,
    height: u32,
    path: &std::path::Path,
) -> std::result::Result<Reader, ShellError> {
    let fail = |e: &dyn std::fmt::Display| ShellError(e.to_string());
    let mut reader = Reader::new(source);
    for step in steps {
        match step {
            Step::Open(p) => reader.open(p),
            Step::Follow(href) => if let Outcome::External(_) = reader.follow(href) {},
            Step::Back => reader.back(),
            Step::Search(term) => reader.search(term),
        }
    }
    init_runtime();
    let field = SearchField::new("Search pages");
    let tree = tree_nodes(reader.entries());
    let actions = Actions::inert();
    #[allow(clippy::cast_precision_loss)] // window sizes are far below 2^23
    let (w, h) = (width as f32, height as f32);
    let ui = frame(
        search_box(&field),
        side_body(&reader, &tree, &actions),
        content(&reader, &actions),
        w,
        h,
    );
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .map_err(|e| fail(&e))?;
    blinc_layout::init_text_measurer();
    let mut tree = RenderTree::from_element(&ui);
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let mut css = String::new();
    for sheet in &bundle().css_sources {
        css.push_str(sheet);
        css.push('\n');
    }
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).map_err(|e| fail(&e))?);
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    let texture = offscreen_texture(&app, width, height);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app.render_tree(&tree, &view, width, height)
        .map_err(|e| fail(&e))?;
    let rgba = app
        .read_texture_to_rgba8(&texture, width, height)
        .map_err(|e| fail(&e))?;
    image::save_buffer(path, &rgba, width, height, image::ColorType::Rgba8)
        .map_err(|e| fail(&e))?;
    Ok(reader)
}

/// What the window runner sets up before the first frame: theme and reactive state.
fn init_runtime() {
    blinc_theme::ThemeState::init(bundle(), blinc_theme::ColorScheme::Light);
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
        label: Some("fs-components snapshot"),
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
