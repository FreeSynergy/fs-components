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
use blinc_layout::widgets::scroll::Scroll;
use fs_components_primitives::SearchField;

use super::error::ShellError;
use super::picture::{Picture, ScrollInfo, Snapshot, Step};
use super::setup::{bundle, register_fonts};
use crate::reader::Reader;
use crate::view::{Actions, content, frame, search_box, side_body, tree_nodes};

/// Apply the steps to a reader and write a PNG to `path`.
///
/// # Errors
/// [`ShellError`] if the size is 0, there is no graphics adapter, or the picture cannot be written.
pub fn snapshot(
    picture: Picture,
    path: &std::path::Path,
) -> std::result::Result<Snapshot, ShellError> {
    // The framework keeps theme and reactive state in process-wide singletons: one picture at a time.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let _guard = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let fail = |e: &dyn std::fmt::Display| ShellError(e.to_string());
    let (width, height) = (picture.width, picture.height);
    if width == 0 || height == 0 {
        return Err(ShellError("the picture needs a width and a height".into()));
    }
    let mut reader = Reader::new(picture.source, picture.labels);
    reader.set_export_target(picture.export_target);
    let mut scroll_px = 0;
    for step in &picture.steps {
        match step {
            Step::Open(p) => reader.open(p),
            Step::Follow(href) => {
                reader.follow(href);
            }
            Step::Back => reader.back(),
            Step::Search(term) => reader.search(term),
            Step::Edit => reader.begin_edit(),
            Step::Type(text) => reader.set_draft(text),
            Step::Save => reader.save(),
            Step::Cancel => reader.cancel_edit(),
            Step::Export => reader.export(),
            Step::Scroll(px) => scroll_px += px,
        }
    }
    init_runtime();
    register_fonts();
    let field = SearchField::new(&reader.labels().search);
    let tree = tree_nodes(reader.entries());
    let actions = Actions::inert();
    let physics = Scroll::new().physics();
    #[allow(clippy::cast_precision_loss)] // window sizes are far below 2^23
    let (w, h) = (width as f32, height as f32);
    let ui = frame(
        search_box(&field),
        side_body(&reader, &tree, &actions),
        content(&reader, &actions, Some(&physics), None),
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
    if scroll_px > 0 {
        #[allow(clippy::cast_precision_loss)]
        physics
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .apply_scroll_delta(0.0, -(scroll_px as f32));
    }
    let scroll = {
        let p = physics
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ScrollInfo {
            content_height: p.content_height,
            viewport_height: p.viewport_height,
            offset: -p.offset_y,
        }
    };
    let texture = offscreen_texture(&app, width, height);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app.render_tree(&tree, &view, width, height)
        .map_err(|e| fail(&e))?;
    let rgba = app
        .read_texture_to_rgba8(&texture, width, height)
        .map_err(|e| fail(&e))?;
    image::save_buffer(path, &rgba, width, height, image::ColorType::Rgba8)
        .map_err(|e| fail(&e))?;
    Ok(Snapshot { reader, scroll })
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
