//! A picture of the status view before and after a card is chosen, the second without a full rebuild
//! (a frame only, as the window does it). Run: `FS_CARD_DIR=/path cargo test --test card_picture -- --ignored`.
//! Not a check: it writes two pictures to compare (the hand test saw the card change its font).
//!
//! Tests: a probe over the real tree, drawn offscreen.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;

use blinc_app::{BlincApp, BlincConfig};
use fs_components_workbenches::tasks::{TaskLabels, TaskView, TasksWidget};
use tasks_common::Shelf;
use tasks_common::harness::Harness;

fn draw(h: &Harness, path: &std::path::Path) {
    let (w, hgt) = (1000u32, 500u32);
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .expect("graphics");
    let texture = app.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("card"),
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

#[test]
#[ignore = "a probe that writes pictures"]
fn a_card_before_and_after_it_is_chosen() {
    let Some(dir) = std::env::var_os("FS_CARD_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let shelf = Arc::new(Shelf::with(&["a", "b"]));
    let widget = TasksWidget::new(shelf, TaskLabels::default());
    widget.with_state(|s| s.set_view(TaskView::Status));
    let mut h = Harness::new(widget, 1000.0, 500.0);
    draw(&h, &dir.join("card-before.png"));
    h.press("board-todo-b");
    h.frame();
    draw(&h, &dir.join("card-after-frame.png"));
    h.rebuild();
    draw(&h, &dir.join("card-after-rebuild.png"));
}
