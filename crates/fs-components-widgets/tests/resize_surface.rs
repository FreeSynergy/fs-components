//! Repro: surface texture is smaller than the size handed to the renderer (window resize race).
// Pixel and size arithmetic on small bounded values: the casts cannot lose anything that matters in a test.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
mod common;
use std::sync::{Arc, Mutex};

use blinc_app::{BlincApp, BlincConfig};
use blinc_layout::prelude::*;

#[test]
fn surface_two_pixels_narrower_than_the_render_size() {
    let _g = common::serial();
    let (w, h) = (1278u32, 800u32);
    let tree = common::layout(
        div()
            .w(w as f32)
            .h(h as f32)
            .bg(blinc_core::Color::rgb(0.2, 0.4, 0.8)),
        w as f32,
        h as f32,
    );
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .expect("gpu");
    let device = app.device().clone();
    let format = app.texture_format();
    let mk = |width: u32| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("surface"),
            size: wgpu::Extent3d {
                width,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    };
    let sched = Arc::new(Mutex::new(blinc_animation::AnimationScheduler::new()));
    let rs = blinc_layout::RenderState::new(sched);
    // first frame at the old size fills the static layer, the second has a narrower surface
    for width in [1278u32, 1276] {
        let tex = mk(width);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        app.render_tree_with_motion_opt(&tree, &rs, &view, Some(&tex), 1278, h, false)
            .expect("render");
    }
    app.device().poll(wgpu::PollType::Wait).ok();
}
