//! Fall C ("the input is dead") for the field component: a `FieldInput` made inside a build, kept, and drawn again
//! after that build is gone, 1000 times. Its signals must stay alive, no rebuild may add one, and typing must change
//! the value that is shown. Matrix over the kinds that have different numbers of parts.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use common::{Input, layout, serial};
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look};
use fs_components_styles::StyleId;

fn live() -> usize {
    global_graph().lock().unwrap().stats().signal_count
}

#[test]
fn a_field_input_made_in_a_build_stays_alive_and_leaks_nothing_over_1000_rebuilds() {
    let _g = serial();
    for (kind, first_id) in [(Kind::Email, "m-local"), (Kind::Phone, "m-0")] {
        let look = Look::new(StyleId::Admin, false);
        let mut scope = BuildScope::new();
        // Made while a build's scope is current, as a page does.
        let pass = scope.begin();
        let field = FieldInput::new(kind, look, Arc::new(KeysAsText)).name("m");
        scope.finish(pass);
        let draw = |scope: &mut BuildScope| {
            let pass = scope.begin();
            let tree = layout(field.build(), 600.0, 200.0);
            scope.finish(pass);
            tree
        };
        drop(draw(&mut scope));
        drop(draw(&mut scope));
        let settled = live();
        for _ in 0..1000 {
            drop(draw(&mut scope));
        }
        assert_eq!(live(), settled, "{kind:?}: 1000 rebuilds, constant");
        let mut tree = draw(&mut scope);
        let mut input = Input::new();
        if tree.query_by_id(first_id).is_none() {
            continue;
        }
        input.click(&mut tree, first_id);
        for ch in "ab".chars() {
            tree.broadcast_text_input_event(ch, false, false, false, false);
            common::frame(&mut tree, 600.0, 200.0);
        }
        assert!(
            field.text().contains("ab"),
            "{kind:?}: typing after 1000 rebuilds changes the value: {:?}",
            field.text()
        );
    }
}
