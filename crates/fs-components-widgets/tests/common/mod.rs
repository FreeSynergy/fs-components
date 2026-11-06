//! Test support: draw an element offscreen (no window), look at the pixels, keep the picture.
//!
//! Pattern: Facade over Blinc's offscreen renderer. One picture at a time per process (Blinc
//! keeps theme and reactive state in process-wide singletons). Pictures are written only when the
//! environment variable `FS_COMP_SHOTS` names a directory; scratch files of a test live in a
//! [`Scratch`] that removes itself.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use blinc_app::{BlincApp, BlincConfig};
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::{ElementBuilder, RenderTree};
use image::RgbaImage;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Hold this while a test touches Blinc's singletons.
pub fn serial() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn bundle() -> blinc_theme::ThemeBundle {
    blinc_cn::cn_bundle().with_css(fs_components_widgets::styles())
}

/// Theme and reactive state, as a window would set them up.
pub fn init_runtime() {
    blinc_theme::ThemeState::init(bundle(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
}

/// The element laid out in `w` x `h`, with the stylesheet applied. No drawing.
pub fn layout(ui: &impl ElementBuilder, w: f32, h: f32) -> RenderTree {
    init_runtime();
    blinc_layout::init_text_measurer();
    let mut tree = RenderTree::from_element(ui);
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let mut css = String::new();
    for sheet in &bundle().css_sources {
        css.push_str(sheet);
        css.push('\n');
    }
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).expect("stylesheet"));
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    tree
}

/// Draw `ui` at `w` x `h` and return the pixels; `name` is the file name when pictures are kept.
pub fn draw(ui: &impl ElementBuilder, w: u32, h: u32, name: &str) -> RgbaImage {
    let tree = layout(ui, w as f32, h as f32);
    draw_tree(&tree, w, h, name)
}

/// Draw an already laid-out tree.
pub fn draw_tree(tree: &RenderTree, w: u32, h: u32, name: &str) -> RgbaImage {
    draw_frames(tree, w, h, name, 1)
}

/// Like [`draw`], but waits for pictures: Blinc fades an image in over its first frames.
pub fn draw_settled(ui: &impl ElementBuilder, w: u32, h: u32, name: &str) -> RgbaImage {
    let tree = layout(ui, w as f32, h as f32);
    draw_frames(&tree, w, h, name, 3)
}

fn draw_frames(tree: &RenderTree, w: u32, h: u32, name: &str, frames: u32) -> RgbaImage {
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .expect("graphics adapter");
    let texture = app.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("fs-components test"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
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
    for i in 0..frames {
        app.render_tree(tree, &view, w, h).expect("render");
        if i + 1 < frames {
            std::thread::sleep(std::time::Duration::from_millis(350));
        }
    }
    let rgba = app.read_texture_to_rgba8(&texture, w, h).expect("read");
    let img = RgbaImage::from_raw(w, h, rgba).expect("size");
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("shots dir");
        img.save(dir.join(name)).expect("save");
    }
    img
}

/// Number of pixels in `rect` (x, y, w, h) that differ clearly from `background`.
pub fn ink(img: &RgbaImage, rect: (u32, u32, u32, u32), background: [u8; 4]) -> usize {
    let (x0, y0, w, h) = rect;
    let mut n = 0;
    for y in y0..(y0 + h).min(img.height()) {
        for x in x0..(x0 + w).min(img.width()) {
            let p = img.get_pixel(x, y).0;
            let d: i32 = (0..3)
                .map(|i| (i32::from(p[i]) - i32::from(background[i])).abs())
                .sum();
            if d > 40 {
                n += 1;
            }
        }
    }
    n
}

/// The colour in the corner (the page background of a picture).
pub fn background(img: &RgbaImage) -> [u8; 4] {
    img.get_pixel(1, 1).0
}

/// The columns (left, right) that hold ink in rows `y0..y1`.
pub fn extent_x(img: &RgbaImage, y0: u32, y1: u32) -> Option<(u32, u32)> {
    let bg = background(img);
    let cols: Vec<u32> = (0..img.width())
        .filter(|&x| ink(img, (x, y0, 1, y1 - y0), bg) > 0)
        .collect();
    Some((*cols.first()?, *cols.last()?))
}

/// A directory for files a test makes; it is removed when this drops (also on a failed test).
pub struct Scratch(PathBuf);

impl Scratch {
    /// A new empty directory.
    pub fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("fs-comp-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        Self(dir)
    }

    /// Path of a file inside.
    pub fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Write a plain coloured PNG (`w` x `h`) and return its path as text.
pub fn write_png(scratch: &Scratch, name: &str, w: u32, h: u32, rgb: [u8; 3]) -> String {
    let img = RgbaImage::from_fn(w, h, |x, y| {
        // a diagonal so a wrongly scaled picture is visible
        let k = if x * h > y * w { 0.6 } else { 1.0 };
        image::Rgba([
            (f64::from(rgb[0]) * k) as u8,
            (f64::from(rgb[1]) * k) as u8,
            (f64::from(rgb[2]) * k) as u8,
            255,
        ])
    });
    let path: &Path = &scratch.path(name);
    img.save(path).expect("png");
    path.to_string_lossy().into_owned()
}
