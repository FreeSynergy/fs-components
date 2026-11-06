//! A program that embeds the reader in its own window, using only the public API: the theme
//! bundle gets `styles()`, the widget is built with a size, and the two columns must be as wide
//! as the reader says (sidebar 320 px), not collapsed.
//!
//! Pattern: Pixel test of a host — skipped (with a message) where there is no graphics adapter.

mod common;

use std::sync::{Arc, Mutex};

use blinc_app::{BlincApp, BlincConfig};
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::RenderTree;
use common::Fake;
use fs_components_workbenches::{Labels, ReaderWidget, register_fonts, styles};

#[test]
fn embedded_with_the_public_styles_the_columns_have_their_width() {
    let bundle = blinc_cn::cn_bundle().with_css(styles());
    blinc_theme::ThemeState::init(bundle.clone(), blinc_theme::ColorScheme::Light);
    BlincContextState::init(
        global_graph(),
        Arc::new(Mutex::new(HookState::new())),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
    );
    register_fonts();
    let Ok(mut app) = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    }) else {
        eprintln!("skipped: no graphics");
        return;
    };
    let (w, h) = (900_u32, 600_u32);
    let widget = ReaderWidget::new(Arc::new(Fake), Labels::default(), None);
    let ui = widget.build(900.0, 600.0);
    blinc_layout::init_text_measurer();
    let mut tree = RenderTree::from_element(&ui);
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let css = bundle.css_sources.join("\n");
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).unwrap());
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(900.0, 600.0);
    let texture = app.device().create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: app.texture_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app.render_tree(&tree, &view, w, h).unwrap();
    let rgba = app.read_texture_to_rgba8(&texture, w, h).unwrap();
    let img = image::RgbaImage::from_raw(w, h, rgba).unwrap();
    // Along a row near the bottom (nothing but the two backgrounds there) find the first change.
    let y = 580;
    let left = *img.get_pixel(150, y);
    let right = *img.get_pixel(700, y);
    assert_ne!(left, right, "sidebar and main must differ in colour");
    let boundary = (0..w).find(|&x| *img.get_pixel(x, y) != left).unwrap();
    assert!(
        (316..=324).contains(&boundary),
        "the sidebar must be 320 px wide, the first other pixel is at {boundary}"
    );
}
