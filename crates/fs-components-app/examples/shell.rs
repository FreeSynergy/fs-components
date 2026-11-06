//! A program with two screens: `cargo run --example shell -- [picture.png]`.
//!
//! Shows what a program writes: no Blinc import, only surfaces, labels and a title. With a path it
//! draws a picture and opens no window.

use std::sync::Arc;

use fs_components_app::{AppWidget, Element, Screen, Status, Surface, WindowOptions, launch};
use fs_components_widgets::{Block, Inline, TextView, styles};

struct Page(&'static str);

impl Surface for Page {
    fn build(&self, _width: f32, _height: f32) -> Box<dyn Element> {
        Box::new(
            TextView::new(vec![Block::Heading {
                level: 1,
                text: Inline::plain(self.0),
            }])
            .build(),
        )
    }
    fn styles(&self) -> String {
        styles()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppWidget::new(
        "Example",
        vec![
            Screen::new("one", "First", Arc::new(Page("The first screen"))),
            Screen::new("two", "Second", Arc::new(Page("The second screen"))),
        ],
    )?;
    app.set_status(Some(Status::Info("Ready".into())));
    match std::env::args().nth(1) {
        Some(path) => app.picture(1000, 640, path.as_ref())?,
        None => launch(WindowOptions::default(), app)?,
    }
    Ok(())
}
