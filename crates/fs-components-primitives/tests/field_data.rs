//! Leak matrix: the signals of every kind of field data that fs-components makes outlive the build that made them,
//! stay constant over 1000 rebuilds, and are given back when the keeper goes.
//!
//! Pattern: none (tests). No graphics adapter needed.
#![allow(clippy::pedantic)]

use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use fs_components_primitives::{FieldData, collect_field_data, scroll_ref, scroll_watch};

static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn live() -> usize {
    global_graph().lock().unwrap().stats().signal_count
}

/// One rebuild of a page: a fresh scope that makes nothing the data needs, the old scope is disposed.
fn rebuild(scope: &mut BuildScope) {
    let pass = scope.begin();
    scope.finish(pass);
}

type Make = fn(&FieldData) -> Box<dyn std::any::Any>;

fn kinds() -> Vec<(&'static str, Make)> {
    vec![
        ("text input", |k| Box::new(k.text_input())),
        ("text input with placeholder", |k| {
            Box::new(k.text_input_with_placeholder("Search"))
        }),
        ("text area", |k| Box::new(k.text_area())),
    ]
}

#[test]
fn field_data_of_every_kind_survives_rebuilds_without_leaking() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    for (name, make) in kinds() {
        collect_field_data();
        let base = live();
        let keep = FieldData::new();
        let mut scope = BuildScope::new();
        // The data is made while a build's scope is current, as a page build does.
        let pass = scope.begin();
        let data = make(&keep);
        scope.finish(pass);
        let made = live();
        assert!(made > base, "{name}: the data makes signals");
        // The next fresh build disposes the scope that made the data.
        rebuild(&mut scope);
        assert_eq!(
            live(),
            made,
            "{name}: the signals outlive the build that made them"
        );
        for _ in 0..1000 {
            rebuild(&mut scope);
        }
        assert_eq!(
            live(),
            made,
            "{name}: 1000 rebuilds, the number of live signals is constant"
        );
        drop(data);
        drop(keep);
        collect_field_data();
        assert_eq!(
            live(),
            base,
            "{name}: dropped keeper gives every signal back"
        );
    }
}

#[test]
fn many_keepers_made_and_dropped_leave_nothing_behind() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    collect_field_data();
    let base = live();
    for _ in 0..1000 {
        let keep = FieldData::new();
        let _a = keep.text_input();
        let _b = keep.text_area();
    }
    collect_field_data();
    assert_eq!(live(), base);
}

#[test]
fn the_watch_of_a_scroll_position_survives_rebuilds() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let scroll = scroll_ref();
    let mut scope = BuildScope::new();
    let pass = scope.begin();
    let watch = scroll_watch(&scroll, 3.0);
    scope.finish(pass);
    let made = live();
    for _ in 0..1000 {
        rebuild(&mut scope);
        // The same watch is given back on every build; nothing new is made.
        let again = scroll_watch(&scroll, 3.0);
        assert_eq!(again.id(), watch.id());
    }
    assert_eq!(live(), made, "1000 rebuilds, constant");
    assert_eq!(
        watch.try_get(),
        Some(3.0),
        "alive after the build that asked is gone"
    );
}

/// The defect itself (Blinc ec776dcf), so that the matrix above is known to test something: data made directly in
/// a build scope loses its signals with that scope.
#[test]
fn control_data_made_directly_in_a_build_scope_loses_its_signals() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let base = live();
    let mut scope = BuildScope::new();
    let pass = scope.begin();
    let data = blinc_layout::widgets::text_input::text_input_data();
    scope.finish(pass);
    assert!(live() > base);
    rebuild(&mut scope);
    assert_eq!(
        live(),
        base,
        "the scope took the signals of data that lives on"
    );
    drop(data);
}
