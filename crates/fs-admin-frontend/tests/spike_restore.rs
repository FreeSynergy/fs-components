//! SPIKE (not for merging): can the view of the Admin frontend (screen, sort, filter, selection) be read out and put
//! back through the public API of `Session` alone?
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;

use fs_admin_frontend::{Direction, Screen, Session};
use memory::{Memory, clock, contact, record};

/// What a restart should bring back: ids and names only, never an index into rows.
#[derive(Debug, PartialEq)]
struct View {
    screen_detail: Option<String>,
    sort: Option<(String, bool)>, // column field name, descending
    filter: String,
    selected: Option<String>,
}

fn save(s: &Session) -> View {
    let sort = s.list().sort().map(|(c, d)| {
        (
            s.list().columns()[c].field.name.clone(),
            d == Direction::Descending,
        )
    });
    View {
        screen_detail: match s.screen() {
            Screen::Detail(id) => Some(id.clone()),
            _ => None,
        },
        sort,
        filter: s.list().filter().to_owned(),
        selected: s.list().selected().map(str::to_owned),
    }
}

fn restore(s: &mut Session, v: &View) {
    s.reload();
    if let Some((name, desc)) = &v.sort {
        if let Some(c) = s.list().columns().iter().position(|c| &c.field.name == name) {
            s.list_mut().cycle_sort(c); // ascending
            if *desc {
                s.list_mut().cycle_sort(c); // descending
            }
        }
    }
    s.list_mut().set_filter(&v.filter);
    if let Some(id) = &v.selected {
        s.list_mut().select(id);
    }
    if let Some(id) = &v.screen_detail {
        s.open_detail(id);
    }
}

#[test]
fn the_view_goes_out_and_comes_back_through_the_public_api() {
    let recs = || {
        vec![
            record("urn:1", &[("name", &["Ada"])]),
            record("urn:2", &[("name", &["Grace"])]),
            record("urn:3", &[("name", &["Alan"])]),
        ]
    };
    let mut a = Session::new(contact(), Memory::new(recs()), clock());
    a.reload();
    let c = a.list().columns().iter().position(|c| c.field.name == "name").unwrap();
    a.list_mut().cycle_sort(c);
    a.list_mut().cycle_sort(c);
    a.list_mut().set_filter("a");
    assert!(a.open_detail("urn:3"));
    let saved = save(&a);
    assert_eq!(saved.sort, Some(("name".into(), true)));

    let mut b = Session::new(contact(), Memory::new(recs()), clock());
    restore(&mut b, &saved);
    assert_eq!(save(&b), saved);
    assert!(matches!(b.screen(), Screen::Detail(id) if id == "urn:3"));

    // an id that is gone is dropped without a word and without a failure
    let mut c2 = Session::new(contact(), Memory::new(vec![record("urn:9", &[("name", &["Zed"])])]), clock());
    restore(&mut c2, &saved);
    assert!(matches!(c2.screen(), Screen::List));
    assert_eq!(c2.list().selected(), None);
}
