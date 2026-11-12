//! The plan screens clicked in process (no window): the real router and hit test, the callbacks recorded.
//!
//! Tests: ticking a box and opening a line are two requests; a card, "+ Create" and a tab report; a locked tab does not.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial};
use fs_components_widgets::plan::{
    BoardView, Column, Entry, Events, ListScreen, PlanCard, Section, SideItem, Tab, TaskLine, Tone,
};

type Log = Arc<Mutex<Vec<String>>>;

fn events(log: &Log) -> Events {
    let say = |tag: &'static str, log: &Log| {
        let log = Arc::clone(log);
        Arc::new(move |k: &str| log.lock().unwrap().push(format!("{tag}:{k}")))
    };
    let mut e = Events::default();
    e.tick = Some(say("tick", log));
    e.open = Some(say("open", log));
    e.pick = Some(say("pick", log));
    e.create = Some(say("create", log));
    e.tab = Some(say("tab", log));
    e
}

#[test]
fn ticking_the_box_and_opening_the_line_are_two_requests() {
    let _g = serial();
    let log: Log = Arc::default();
    let ev = events(&log);
    let entry = Entry::new("New to-do");
    let lines = vec![
        TaskLine::new("t1", "Call the printer"),
        TaskLine::new("t2", "Pay the bill"),
    ];
    let sections = [Section::new("Today", Tone::Accent, lines)];
    let side = [SideItem::new("today", "Today", Some(2))];
    let ui = ListScreen {
        side: &side,
        title: "Today",
        note: None,
        sections: &sections,
        place: "today",
        entry: &entry,
        empty: "Nothing here.",
        events: &ev,
    }
    .build("L", 1000.0, 640.0);
    let mut tree = layout(ui, 1000.0, 640.0);
    let mut input = Input::new();
    input.click(&mut tree, "L-t1-check");
    input.click(&mut tree, "L-t2");
    input.click(&mut tree, "L-list-today");
    assert_eq!(*log.lock().unwrap(), ["tick:t1", "open:t2", "pick:today"]);
}

#[test]
fn a_card_create_and_a_tab_report_and_a_locked_tab_does_not() {
    let _g = serial();
    let log: Log = Arc::default();
    let ev = events(&log);
    let entry = Entry::new("What needs to be done?");
    let mut col = Column::new("todo", "To do");
    col.cards.push(PlanCard::new("c1", "Reply to the landlord"));
    let tabs = [
        Tab::new("board", "Board"),
        Tab::new("list", "List"),
        Tab::new("calendar", "Calendar").locked(),
    ];
    let ui = BoardView {
        tabs: &tabs,
        active: "board",
        soon: "Not built yet",
        projects: &[],
        columns: &[col],
        panel: None,
        creating: None,
        entry: &entry,
        create: "Create",
        close: "Close",
        events: &ev,
    }
    .build("B", 1000.0, 640.0);
    let mut tree = layout(ui, 1000.0, 640.0);
    let mut input = Input::new();
    input.click(&mut tree, "B-c1");
    input.click(&mut tree, "B-todo-create");
    input.click(&mut tree, "B-tab-list");
    input.click(&mut tree, "B-tab-calendar");
    assert_eq!(
        *log.lock().unwrap(),
        ["pick:c1", "create:todo", "tab:list"],
        "the locked tab reports nothing"
    );
}
