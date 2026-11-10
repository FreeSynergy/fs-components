//! A window with the five basic-type inputs in a style: the probe for typing (AltGr, text colour,
//! speed) and for looking at a style.
//!
//! Pattern: none (a small program over the public API).
//!
//! `cargo run -p fs-components-fields --example form -- [admin|simple|apple|windows] [dark]`

use std::sync::Arc;

use blinc_app::prelude::*;
use blinc_app::windowed::WindowedApp;
use blinc_layout::div::div;
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look, Say, SuggestFromList};
use fs_components_styles::StyleId;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let id = args
        .iter()
        .find_map(|a| StyleId::from_name(a))
        .unwrap_or(StyleId::Admin);
    let look = Look::new(id, args.iter().any(|a| a == "dark"));
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let domains = SuggestFromList {
        field: "email",
        segment: 1,
        seen: ["example.org", "example.org", "example.net", "mail.test"]
            .map(String::from)
            .to_vec(),
    };
    let fields: Vec<FieldInput> = Kind::ALL
        .into_iter()
        .map(|k| {
            let f = FieldInput::new(k, look, Arc::clone(&say)).name(k.field().key());
            if k == Kind::Email {
                f.suggest(Arc::new(domains.clone()))
            } else {
                f
            }
        })
        .collect();
    let config = WindowConfig {
        title: format!("fields: {}", id.name()),
        width: 760,
        height: 700,
        ..Default::default()
    };
    let bundle = blinc_cn::cn_bundle().with_css(fs_components_widgets::styles());
    WindowedApp::run_with_theme(
        config,
        bundle,
        blinc_theme::ColorScheme::Light,
        move |_ctx| {
            let mut page = div()
                .w_full()
                .h_full()
                .flex_col()
                .gap_px(look.style.spacing.m)
                .padding_x_px(look.style.spacing.l)
                .padding_y_px(look.style.spacing.l)
                .bg(Look::color(look.palette().ground));
            for f in &fields {
                page = page.child(f.build());
            }
            page
        },
    )
    .expect("window");
}
