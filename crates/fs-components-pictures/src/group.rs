//! A group of blocks drawn in Craft and judged against reference pictures.
//!
//! Pattern: Facade over `fs_components_render::draw_to_png`, `gpu` and `compare`; one call draws, then writes the
//! reference (`UPDATE_UMBAU_PICTURES=1`) or compares.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use blinc_layout::div::div;
use blinc_layout::prelude::ElementBuilder;
use blinc_theme::{ColorToken, ThemeState};
use fs_components_render::{Canvas, RenderError, draw_to_png, use_style};
use fs_components_styles::StyleId;
use image::{ImageEncoder, RgbaImage, codecs::png};

use crate::compare::{Report, Tolerance, compare, diff_image};
use crate::gpu::force_software;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// What came of one picture.
#[derive(Debug)]
pub enum Verdict {
    /// The reference was written (`UPDATE_UMBAU_PICTURES=1`).
    Updated(PathBuf),
    /// Within the tolerance of the group.
    Equal(Report),
    /// Not drawn: no software GPU here (outside CI this is a skip, in CI (`CI` set) a failure).
    Skipped(String),
}

/// A group of blocks of one crate (or module), drawn at one size with one tolerance.
#[derive(Clone, Debug)]
pub struct Group {
    /// Directory name under `contract/umbau/bilder`.
    pub name: &'static str,
    /// Picture size (width, height).
    pub size: (u32, u32),
    /// What may differ in this group.
    pub tolerance: Tolerance,
}

impl Group {
    /// A group called `name`, pictures of `size`, judged with `tolerance`.
    pub fn new(name: &'static str, size: (u32, u32), tolerance: Tolerance) -> Self {
        Self {
            name,
            size,
            tolerance,
        }
    }

    /// Where the references of all groups live.
    pub fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contract/umbau/bilder")
    }

    /// The reference file of block `name`.
    pub fn reference_path(&self, name: &str) -> PathBuf {
        Self::root().join(self.name).join(format!("{name}.png"))
    }

    fn diff_path(&self, name: &str) -> PathBuf {
        let target = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"),
            PathBuf::from,
        );
        target
            .join("pictures-diff")
            .join(self.name)
            .join(format!("{name}.png"))
    }

    fn actual_path(&self, name: &str) -> PathBuf {
        self.diff_path(name)
            .with_file_name(format!("{name}.actual.png"))
    }

    /// Draw the element of `build` on the software GPU in Craft, on the theme background, at the size of the group.
    ///
    /// # Errors
    /// Text saying why nothing was drawn (no lavapipe, no adapter, a drawing error).
    pub fn draw<E, B>(&self, css: &str, build: B) -> Result<RgbaImage, String>
    where
        E: ElementBuilder + 'static,
        B: FnOnce(f32, f32) -> E,
    {
        force_software()?;
        use_style(Some(StyleId::Craft));
        let tmp = std::env::temp_dir().join(format!(
            "fs-umbau-{}-{}.png",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let canvas = Canvas::new(self.size.0, self.size.1, css);
        let drawn = draw_to_png(
            &canvas,
            |w, h| {
                let ground = ThemeState::get().color(ColorToken::Background);
                div().w(w).h(h).bg(ground).child(build(w, h))
            },
            || {},
            &tmp,
        );
        let picture = match drawn {
            Ok(()) => image::open(&tmp)
                .map(|i| i.to_rgba8())
                .map_err(|e| e.to_string()),
            Err(RenderError::NoGraphics(why)) => Err(why),
            Err(e) => panic!("drawing {} failed: {e}", self.name),
        };
        let _ = std::fs::remove_file(&tmp);
        picture
    }

    /// Write `picture` as the reference of block `name`; returns the file.
    pub fn write_reference(&self, name: &str, picture: &RgbaImage) -> PathBuf {
        let path = self.reference_path(name);
        write_png(&path, picture);
        path
    }

    /// Write or compare the picture of block `name`. Panics, naming the diff picture, when it differs or the
    /// reference is missing.
    pub fn judge(&self, name: &str, picture: &RgbaImage) -> Verdict {
        let path = self.reference_path(name);
        if std::env::var_os("UPDATE_UMBAU_PICTURES").is_some_and(|v| v == "1") {
            return Verdict::Updated(self.write_reference(name, picture));
        }
        let reference = image::open(&path)
            .unwrap_or_else(|e| {
                panic!(
                    "no reference {} ({e}); write it with UPDATE_UMBAU_PICTURES=1",
                    path.display()
                )
            })
            .to_rgba8();
        let report = compare(&reference, picture, self.tolerance);
        if !report.within(self.tolerance) {
            let diff = self.diff_path(name);
            write_png(&diff, &diff_image(&reference, picture, self.tolerance));
            let actual = self.actual_path(name);
            write_png(&actual, picture);
            panic!(
                "picture {}/{name} differs from the reference: {} (tolerance {:?}); diff picture {}; drawn picture {}",
                self.name,
                report.describe(),
                self.tolerance,
                diff.display(),
                actual.display()
            );
        }
        Verdict::Equal(report)
    }

    /// Draw and judge one block. Without a software GPU it skips (message on stderr), except when `CI` is set.
    pub fn check<E, B>(&self, name: &str, css: &str, build: B) -> Verdict
    where
        E: ElementBuilder + 'static,
        B: FnOnce(f32, f32) -> E,
    {
        match self.draw(css, build) {
            Ok(picture) => self.judge(name, &picture),
            Err(why) => {
                assert!(
                    std::env::var_os("CI").is_none(),
                    "picture {}/{name} cannot be drawn in CI: {why}",
                    self.name
                );
                eprintln!("skipped {}/{name}: {why}", self.name);
                Verdict::Skipped(why)
            }
        }
    }
}

/// A PNG at best compression (references are kept small).
fn write_png(path: &Path, picture: &RgbaImage) {
    std::fs::create_dir_all(path.parent().expect("a file has a directory"))
        .expect("picture directory");
    let file = std::fs::File::create(path).expect("picture file");
    png::PngEncoder::new_with_quality(
        std::io::BufWriter::new(file),
        png::CompressionType::Best,
        png::FilterType::Adaptive,
    )
    .write_image(
        picture.as_raw(),
        picture.width(),
        picture.height(),
        image::ExtendedColorType::Rgba8,
    )
    .expect("write picture");
}
