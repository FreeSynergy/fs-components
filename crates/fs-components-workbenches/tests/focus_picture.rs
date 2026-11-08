//! A picture of the form with the focus in a field and text typed: what the window shows then.
//! Run: `FS_FOCUS_PNG=/path/x.png cargo test --test focus_picture -- --ignored`. Not a check: it writes a
//! picture to look at (the hand test saw white text in the focused field).
//!
//! Tests: a probe over the real tree, laid out and driven like a window, drawn offscreen.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use blinc_app::{BlincApp, BlincConfig};
use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::{Book, harness::Harness};

#[test]
#[ignore = "a probe that writes a picture"]
fn the_focused_field_with_text() {
    let Some(path) = std::env::var_os("FS_FOCUS_PNG") else {
        return;
    };
    let book = Arc::new(Book::with(&["ada"]));
    let widget = PeopleWidget::new(book, PeopleLabels::default());
    let mut h = Harness::new(widget, 900.0, 600.0);
    h.click("edit");
    h.press("field-name");
    assert!(h.type_text("Hello focus"));
    h.frame();
    let (w, hgt) = (900u32, 600u32);
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .expect("graphics");
    if std::env::var_os("FS_SHARED_FONTS").is_some() {
        // The measurer over the registry the renderer draws with, as `blinc_layout` documents.
        blinc_layout::init_text_measurer_with_registry(app.font_registry());
        h.rebuild();
    }
    let texture = app.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("focus"),
        size: wgpu::Extent3d {
            width: w,
            height: hgt,
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
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app.render_tree(&h.tree, &view, w, hgt).expect("draw");
    let rgba = app.read_texture_to_rgba8(&texture, w, hgt).expect("read");
    image::save_buffer(path, &rgba, w, hgt, image::ColorType::Rgba8).unwrap();
}
