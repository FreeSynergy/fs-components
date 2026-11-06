//! Offscreen drawing for tests: a chart on a real GPU texture, read back as pixels.
//!
//! No window is opened. Pictures go to `$CHARTS_PNG_DIR` when set (to look at them), else to a
//! temporary directory that is removed when the test ends.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use blinc_app::{BlincApp, BlincConfig};
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::{RenderTree, div};
use blinc_theme::{ColorScheme, ColorToken, ThemeState};
use fs_components_charts::{Chart, ChartView, Labels};

/// A temporary directory, removed on drop.
pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("fs-charts-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Where a picture goes.
pub fn out_dir(tag: &str) -> (PathBuf, Option<TempDir>) {
    match std::env::var_os("CHARTS_PNG_DIR") {
        Some(d) => {
            let p = PathBuf::from(d);
            std::fs::create_dir_all(&p).expect("png dir");
            (p, None)
        }
        None => {
            let t = TempDir::new(tag);
            (t.0.clone(), Some(t))
        }
    }
}

/// The pixels of a picture.
pub struct Picture {
    /// The series colours of the theme the picture was drawn in.
    pub series: Vec<[u8; 3]>,
    /// The surface colour of the theme the picture was drawn in.
    pub bg: [u8; 3],
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Picture {
    pub fn pixel(&self, x: f32, y: f32) -> [u8; 3] {
        let (x, y) = (
            (x as u32).min(self.width - 1),
            (y as u32).min(self.height - 1),
        );
        let i = ((y * self.width + x) * 4) as usize;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]]
    }

    /// Number of pixels that differ clearly from `bg` inside the box.
    pub fn ink(&self, area: [f32; 4], bg: [u8; 3]) -> usize {
        let mut n = 0;
        for y in area[1] as u32..(area[1] + area[3]) as u32 {
            for x in area[0] as u32..(area[0] + area[2]) as u32 {
                let p = self.pixel(x as f32, y as f32);
                if (0..3).any(|k| p[k].abs_diff(bg[k]) > 24) {
                    n += 1;
                }
            }
        }
        n
    }
}

pub fn to_u8(c: blinc_core::Color) -> [u8; 3] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    ]
}

pub fn near(a: [u8; 3], b: [u8; 3], tol: u8) -> bool {
    (0..3).all(|k| a[k].abs_diff(b[k]) <= tol)
}

/// Draw `view` at `width` x `height` on the given colour scheme, write `name.png` into `dir`.
pub fn render(
    view: &ChartView,
    width: u32,
    height: u32,
    dark: bool,
    dir: &Path,
    name: &str,
) -> Picture {
    // The framework keeps theme and reactive state in process-wide singletons: one at a time.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let _guard = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let bundle = blinc_cn::cn_bundle();
    ThemeState::init(
        bundle.clone(),
        if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
    );
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
    let bg = ThemeState::get().color(ColorToken::Background);
    // read under the lock: another test may switch the theme as soon as this one is done
    let look = fs_components_charts::view::Look::current();
    let series: Vec<[u8; 3]> = (0..8)
        .map(|i| {
            look.rgb(fs_components_charts::scene::Paint::Series(i))
                .map(|c| (c * 255.0).round() as u8)
        })
        .collect();
    let mut app = BlincApp::with_config(BlincConfig {
        sample_count: 1,
        ..Default::default()
    })
    .expect("graphics adapter");
    blinc_layout::init_text_measurer_with_registry(app.font_registry());
    let ui = div()
        .w(width as f32)
        .h(height as f32)
        .bg(bg)
        .child(view.build(width as f32, height as f32));
    let mut tree = RenderTree::from_element(&ui);
    let vars = ThemeState::get().to_css_variable_map();
    let css: String = bundle
        .css_sources
        .iter()
        .map(|s| format!("{s}\n"))
        .collect();
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).expect("css"));
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(width as f32, height as f32);
    let texture = app.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("fs-charts test"),
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
    });
    let tv = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app.render_tree(&tree, &tv, width, height).expect("render");
    let rgba = app
        .read_texture_to_rgba8(&texture, width, height)
        .expect("read back");
    image::save_buffer(
        dir.join(format!("{name}.png")),
        &rgba,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .expect("png");
    Picture {
        bg: to_u8(bg),
        series,
        width,
        height,
        rgba,
    }
}

/// A chart from JSON with default words.
pub fn view(json: &str) -> ChartView {
    ChartView::new(
        Chart::from_json(json).unwrap_or_else(|e| panic!("{e}")),
        Labels::default(),
    )
}
