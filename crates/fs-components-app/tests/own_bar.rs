//! A surface may bring its own bar (it gets the menus) and do without the status line: the window then has one top line and a
//! body that is as high as the room left.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_app::{AppWidget, Element, Screen, Surface};

struct Own;

impl Surface for Own {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(div().w(width).h(height))
    }
    fn bar(&self, menus: Option<Div>, _width: f32) -> Option<(Div, f32)> {
        let mut bar = div().h(47.0);
        if let Some(menus) = menus {
            bar = bar.child(menus);
        }
        Some((bar, 47.0))
    }
    fn status_line(&self) -> bool {
        false
    }
}

#[test]
fn a_surface_with_its_own_bar_and_no_status_line_draws() {
    let app = AppWidget::new("Own", vec![Screen::new("own", "Own", Arc::new(Own))]).unwrap();
    let dir = std::env::temp_dir().join(format!("fs-own-bar-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("own.png");
    app.picture(400, 300, &file).unwrap();
    assert!(std::fs::metadata(&file).unwrap().len() > 100);
    let _ = std::fs::remove_dir_all(&dir);
}
