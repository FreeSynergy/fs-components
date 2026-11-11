//! SPIKE (not for merging): what can a program put back after a restart, with Blinc as it is?
//!
//! Window: size, maximized, fullscreen, position, centre come from the environment into `WindowConfig`.
//! View: folder, view, sort, selection and scroll row come from `SPIKE_RESTORE`
//! (`folder=d3,view=grid,sort=size,desc=1,select=f00042,row=120`), applied in the order the state needs.
//! Every second a line `SNAP ...` says what the widget reports; each build says `BUILD w h`.

#[path = "../tests/files_common/mod.rs"]
mod files_common;

use std::sync::Arc;
use std::time::Duration;

use blinc_app::prelude::WindowConfig;
use blinc_app::windowed::WindowedApp;
use files_common::Shelf;
use fs_components_workbenches::file_manager::{
    FileLabels, FileManagerWidget, FileView, SortKey,
};

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn restore_map() -> Vec<(String, String)> {
    env("SPIKE_RESTORE")
        .map(|t| {
            t.split(',')
                .filter_map(|p| p.split_once('='))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn main() {
    fs_components_render::register_fonts();
    let mut shelf = Shelf::new();
    for d in 0..5 {
        shelf = shelf.folder(&format!("d{d}"), None);
    }
    for i in 0..600 {
        shelf = shelf.file_in(&format!("g{i:05}"), Some("d3"));
    }
    shelf = shelf.numbered(600);
    let widget = FileManagerWidget::new(Arc::new(shelf), FileLabels::default());

    let mut config = WindowConfig {
        title: "spike".into(),
        width: env("SPIKE_W").and_then(|v| v.parse().ok()).unwrap_or(1000),
        height: env("SPIKE_H").and_then(|v| v.parse().ok()).unwrap_or(700),
        min_size: Some((400, 300)),
        maximized: env("SPIKE_MAX").is_some(),
        fullscreen: env("SPIKE_FULL").is_some(),
        center: env("SPIKE_CENTER").is_some(),
        ..Default::default()
    };
    if let Some(p) = env("SPIKE_POS") {
        if let Some((x, y)) = p.split_once(',') {
            config.position = Some((x.parse().unwrap_or(0), y.parse().unwrap_or(0)));
        }
    }

    // The view, in the order the state machine needs: opening a folder, a new sort or a search makes a new list
    // (nothing selected, scroll at the top), so the selection and the scroll come last.
    let map = restore_map();
    let get = |k: &str| map.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    widget.with_state(|s| {
        if let Some(f) = get("folder") {
            s.open_folder(Some(&f));
        }
        if let Some(v) = get("view") {
            s.set_view(if v == "grid" { FileView::Grid } else { FileView::List });
        }
        if let Some(k) = get("sort") {
            s.set_sort(match k.as_str() {
                "size" => SortKey::Size,
                "modified" => SortKey::Modified,
                "type" => SortKey::Type,
                _ => SortKey::Name,
            });
        }
        if get("desc").as_deref() == Some("1") {
            s.toggle_direction();
        }
        if let Some(key) = get("select") {
            // rows come in pages: load until the key is among them
            for _ in 0..50 {
                if s.rows().iter().any(|r| r.key == key) || !s.has_more() {
                    break;
                }
                s.load_more();
            }
            s.select(&key);
        }
    });
    let early_row: Option<usize> = get("row_early").and_then(|v| v.parse().ok());
    let late_row: Option<usize> = get("row").and_then(|v| v.parse().ok());
    let hook_row: Option<usize> = get("row_hook").and_then(|v| v.parse().ok());
    if let Some(row) = early_row {
        widget.position().scroll_to_row(row);
    }

    let probe = widget.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(3));
        if let Some(row) = late_row {
            probe.position().scroll_to_row(row);
            probe.refresh();
        }
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let ctx = probe.context();
            let (sort, rows) = probe.with_state(|s| (s.sort(), s.rows().len()));
            println!(
                "SNAP offset={} cursor={:?} folder={:?} selected={:?} view={:?} sort={:?}/{} rows={}",
                probe.position().offset(),
                probe.position().cursor(),
                ctx.folder,
                ctx.selected,
                ctx.view,
                sort.key,
                sort.descending,
                rows
            );
        }
    });

    let css = widget.styles();
    let run = WindowedApp::run_with_theme(
        config,
        fs_components_render::bundle(&css),
        blinc_theme::ColorScheme::Light,
        move |ctx| {
            println!("BUILD {} {} rebuild_count={}", ctx.width, ctx.height, ctx.rebuild_count);
            let element = widget.build(ctx.width, ctx.height);
            // After the first build has consumed "new list", ask for the row and draw once more: no timer.
            if let (0, Some(row)) = (ctx.rebuild_count, hook_row) {
                widget.position().scroll_to_row(row);
                widget.refresh();
            }
            element
        },
    );
    println!("END {run:?}");
}
