//! Group `primitives`: the search field in its sizes.
//!
//! Tests: picture comparison against `contract/umbau/bilder/primitives`.

use blinc_layout::div::div;
use fs_components_pictures::{Group, Tolerance};
use fs_components_primitives::{FieldSize, SEARCH_FIELD_STYLES, SearchField};

fn group() -> Group {
    Group::new("primitives", (240, 80), Tolerance::SYSTEM_FONT_SMALL)
}

#[test]
fn the_search_field_medium() {
    let f = SearchField::new("Search");
    group().check("search_medium", SEARCH_FIELD_STYLES, |w, _| {
        div().p(8.0).w(w).child(f.build())
    });
}

#[test]
fn the_search_field_small_and_body() {
    let (s, b) = (
        SearchField::new("Small").size(FieldSize::Small),
        SearchField::new("Body").size(FieldSize::Body),
    );
    group().check("search_small_body", SEARCH_FIELD_STYLES, |w, _| {
        div().p(8.0).gap(8.0).w(w).child(s.build()).child(b.build())
    });
}
