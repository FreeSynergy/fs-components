//! The command bar and the title (the fs-tasks probe, Strategy F): a bar of many commands wraps instead of running out of
//! the window, and the title never lies over a button. Drawn offscreen; a picture is kept when `FS_COMP_SHOTS` is set.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_admin_frontend::view::AdminSurface;
use fs_admin_frontend::{ManualClock, Session};
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use fs_types::{CommandDescription, CommandKind};
use memory::{Memory, record};
use ui::{draw_tree, layout, serial};

fn bounds(tree: &blinc_layout::prelude::RenderTree, id: &str) -> (f32, f32, f32, f32) {
    let node = tree
        .query_by_id(id)
        .unwrap_or_else(|| panic!("no element {id}"));
    let b = tree.get_absolute_bounds(node).expect("bounds");
    (b.x, b.y, b.width, b.height)
}

/// A type with many commands that all work on the list, with long labels.
fn busy() -> (AdminSurface, Vec<String>) {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let mut d = mem.description.clone();
    let mut names = vec!["create".to_owned(), "find".to_owned()];
    for i in 0..12 {
        let n = format!("extra{i}");
        d = d.command(CommandDescription::new(
            &n,
            CommandKind::Other,
            &format!("command.a.long.label.number.{i}"),
        ));
        names.push(n);
    }
    let session = Session::new(d, mem, Arc::new(ManualClock::default()));
    let s = AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    );
    (s, names)
}

#[test]
fn many_commands_wrap_inside_the_window_and_the_title_is_above_the_first_button() {
    let _g = serial();
    let (s, names) = busy();
    for (w, h) in [(600.0_f32, 520.0_f32), (900.0, 600.0)] {
        let tree = layout(s.build(w, h), w, h);
        let (tx, ty, tw, th) = bounds(&tree, "page-title");
        assert!(
            tx + tw <= w + 0.5,
            "the title is wider than the window at {w}"
        );
        let mut lowest_top = f32::MAX;
        let mut rows = std::collections::BTreeSet::new();
        for n in &names {
            let (x, y, bw, _) = bounds(&tree, &format!("cmd-{n}"));
            assert!(
                x >= 0.0 && x + bw <= w + 0.5,
                "cmd-{n} runs out of the window at {w}: {x}+{bw}"
            );
            lowest_top = lowest_top.min(y);
            rows.insert(y as i32);
        }
        assert!(
            ty + th <= lowest_top + 0.5,
            "the title ({ty}+{th}) lies over the first button ({lowest_top}) at {w}"
        );
        if w < 700.0 {
            assert!(
                rows.len() > 1,
                "at {w} px the bar has to wrap, it has {} row",
                rows.len()
            );
        }
        let (_, ry, _, rh) = bounds(&tree, "records");
        assert!(
            ry + rh <= h + 0.5,
            "the list ({ry}+{rh}) runs out of the window at {h} high, under a wrapped bar"
        );
        let img = draw_tree(
            &tree,
            w as u32,
            h as u32,
            &format!("admin-command-bar-{}.png", w as u32),
        );
        assert_eq!(img.width(), w as u32);
    }
}
