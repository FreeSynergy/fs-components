//! Test support: draw an element offscreen (no window), look at the pixels, keep the picture.
//!
//! Tests: a helper over Blinc's offscreen renderer. One picture at a time per process (Blinc
//! keeps theme and reactive state in process-wide singletons). Pictures are written only when the
//! environment variable `FS_COMP_SHOTS` names a directory; scratch files of a test live in a
//! [`Scratch`] that removes itself.
#![allow(dead_code)]
// Pixel arithmetic on small, bounded picture sizes and colour channels: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use blinc_app::{BlincApp, BlincConfig};
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::{ElementBuilder, RenderTree};
use image::RgbaImage;

mod input;
#[allow(unused_imports)]
pub use input::Input;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Hold this while a test touches Blinc's singletons.
pub fn serial() -> std::sync::MutexGuard<'static, ()> {
    let guard = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    init_runtime();
    // Focus of a text input outlives its test (it is process-wide); a widget ignores keys while
    // one has the keyboard, so every test starts without.
    blinc_layout::widgets::text_input::blur_all_text_inputs();
    guard
}

fn bundle() -> blinc_theme::ThemeBundle {
    blinc_cn::cn_bundle().with_css(fs_components_widgets::styles())
}

/// Theme and reactive state, as a window would set them up.
pub fn init_runtime() {
    fs_components_widgets::live::set_min_gap_ms(0); // every refresh draws: tests look at the result at once
    blinc_theme::ThemeState::init(bundle(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
    // the window does this too: a changed signal reaches the live areas that depend on it
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
}

/// One window frame: what asked to be drawn again is drawn again, then the layout runs.
pub fn frame(tree: &mut RenderTree, w: f32, h: f32) {
    let updates = tree.take_pending_property_updates();
    tree.apply_partial_property_updates(updates);
    tree.process_pending_subtree_rebuilds_routed(None);
    tree.compute_layout(w, h);
}

/// The element laid out in `w` x `h`, with the stylesheet applied. No drawing.
pub fn layout(ui: impl ElementBuilder + 'static, w: f32, h: f32) -> RenderTree {
    layout_with(ui, w, h, "")
}

/// Like [`layout`], with `extra` rules put before the theme's (a window installs Blinc's default look of its own widgets).
pub fn layout_with(ui: impl ElementBuilder + 'static, w: f32, h: f32, extra: &str) -> RenderTree {
    init_runtime();
    blinc_layout::init_text_measurer();
    // A window paints the theme's background; an offscreen texture would stay clear.
    let page = blinc_layout::div::div()
        .w(w)
        .h(h)
        .bg(blinc_theme::ThemeState::get().color(blinc_theme::ColorToken::Background))
        .child(ui);
    let mut tree = RenderTree::from_element(&page);
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let mut css = String::from(extra);
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
pub fn draw(ui: impl ElementBuilder + 'static, w: u32, h: u32, name: &str) -> RgbaImage {
    let tree = layout(ui, w as f32, h as f32);
    draw_tree(&tree, w, h, name)
}

/// Draw `ui` with the default look of Blinc's own widgets, as a window has it (checkboxes, buttons).
pub fn draw_window(ui: impl ElementBuilder + 'static, w: u32, h: u32, name: &str) -> RgbaImage {
    let tree = layout_with(ui, w as f32, h as f32, blinc_layout::ua_styles::UA_STYLES);
    draw_tree(&tree, w, h, name)
}

/// Draw an already laid-out tree.
pub fn draw_tree(tree: &RenderTree, w: u32, h: u32, name: &str) -> RgbaImage {
    draw_frames(tree, w, h, name, 1)
}

/// Like [`draw`], but waits for pictures: Blinc fades an image in over its first frames.
pub fn draw_settled(ui: impl ElementBuilder + 'static, w: u32, h: u32, name: &str) -> RgbaImage {
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
#[allow(clippy::many_single_char_names)] // x0/y0/w/h/n/x/y: pixel coordinates of one rectangle scan
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

/// Does this machine draw text? Pictures of text need system fonts; a test may add pixel checks
/// of text when it does, but must reach the same verdict about structure either way.
pub fn has_fonts() -> bool {
    static FONTS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FONTS.get_or_init(|| {
        init_runtime();
        let ui = blinc_layout::div::div().child(blinc_layout::prelude::text("MMMMMMMM").size(30.0));
        let img = draw(ui, 200, 60, "fonts-probe.png");
        ink(&img, (0, 0, 200, 60), background(&img)) > 100
    })
}

/// Size of the element with this id after layout.
pub fn size_of(tree: &RenderTree, id: &str) -> (f32, f32) {
    let node = tree
        .query_by_id(id)
        .unwrap_or_else(|| panic!("no element {id}"));
    let b = tree.get_absolute_bounds(node).expect("bounds");
    (b.width, b.height)
}
