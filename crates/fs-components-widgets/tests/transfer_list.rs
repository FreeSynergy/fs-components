//! Transfer list: the model, the picture and the buttons.
//!
//! Pattern: Model tests, pixel tests, and an in-process mouse over the real hit test.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, background, draw, ink, layout, serial};
use fs_components_widgets::Transfer;
use fs_components_widgets::TransferList;
use fs_components_widgets::transfer_list::{Action, Direction, Outcome, State, format_bytes};

fn mixed() -> Vec<Transfer> {
    vec![
        Transfer::new("a", "holiday.mp4", Direction::Upload)
            .with_state(State::Running)
            .with_progress(300, Some(1000)),
        Transfer::new("b", "report.pdf", Direction::Download)
            .with_state(State::Failed("timeout".into()))
            .with_progress(10, Some(1000)),
        Transfer::new("c", "notes.txt", Direction::Upload)
            .with_state(State::Done)
            .with_progress(2048, Some(2048)),
        Transfer::new("d", "stream", Direction::Download)
            .with_state(State::Running)
            .with_progress(5, None),
        Transfer::new("e", "later.bin", Direction::Upload),
    ]
}

#[test]
fn progress_is_clamped_and_unknown_is_none() {
    let t = Transfer::new("k", "n", Direction::Upload);
    assert_eq!(t.clone().with_progress(50, Some(100)).fraction(), Some(0.5));
    assert_eq!(
        t.clone().with_progress(500, Some(100)).fraction(),
        Some(1.0)
    );
    assert_eq!(t.clone().with_progress(5, Some(0)).fraction(), None);
    assert_eq!(t.clone().with_progress(5, None).fraction(), None);
    assert_eq!(t.with_state(State::Done).fraction(), Some(1.0));
}

#[test]
fn the_state_decides_the_buttons() {
    let t = Transfer::new("k", "n", Direction::Upload);
    assert_eq!(t.clone().actions(), vec![Action::Cancel]);
    assert_eq!(
        t.clone().with_state(State::Failed(String::new())).actions(),
        vec![Action::Retry, Action::Dismiss]
    );
    assert_eq!(t.with_state(State::Done).actions(), vec![Action::Dismiss]);
}

#[test]
fn sizes_read_well() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1536), "1.5 KiB");
    assert_eq!(format_bytes(u64::MAX), "16.0 EiB");
}

#[test]
fn what_is_not_drawn_is_counted() {
    let many: Vec<Transfer> = (0..250)
        .map(|i| Transfer::new(format!("k{i}"), format!("f{i}"), Direction::Upload))
        .collect();
    assert_eq!(TransferList::new(many).max_rows(100).not_shown(), 150);
    assert_eq!(TransferList::new(vec![]).not_shown(), 0);
}

#[test]
fn the_picture_shows_rows_with_progress_and_failure() {
    let _g = serial();
    let ui = TransferList::new(mixed()).build();
    let tree = layout(ui, 520.0, 420.0);
    let img = common::draw_tree(&tree, 520, 420, "transfer-list-mixed.png");
    let row_y = |id: &str| Input::centre(&tree, id).1 as u32 + 1;
    let bg = background(&img);
    assert!(ink(&img, (0, 0, 520, 420), bg) > 3000);
    // the failed row (second) carries the error background: find reddish pixels
    let reds = img
        .pixels()
        .filter(|p| {
            i32::from(p.0[0]) > i32::from(p.0[1]) + 8
                && i32::from(p.0[0]) > i32::from(p.0[2]) + 8
                && p.0[3] > 0
        })
        .count();
    assert!(reds > 1500, "failure is not marked: {reds}");
    // the bar of the first row is filled to 30 %, the third (done) to 100 %
    let blue = |y: u32| {
        (0..520)
            .filter(|&x| {
                let p = img.get_pixel(x, y).0;
                i32::from(p[2]) > i32::from(p[0]) + 80
            })
            .count()
    };
    let (first, third) = (
        blue(row_y("fs-tl-a-cancel")),
        blue(row_y("fs-tl-c-dismiss")),
    );
    assert!((45..=65).contains(&first), "first bar {first}");
    assert!((165..=185).contains(&third), "third bar {third}");
    // each row is its own box: five rows take most of the height
    let used = (0..420u32)
        .rev()
        .find(|&y| ink(&img, (0, y, 520, 1), bg) > 0)
        .unwrap_or(0);
    assert!((250..=420).contains(&used), "rows end at {used}");
}

#[test]
fn buttons_report_outcomes_and_nothing_else_does() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let ui = TransferList::new(mixed())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())))
        .build();
    let mut tree = layout(ui, 520.0, 420.0);
    let mut input = Input::new();
    input.click(&mut tree, "fs-tl-b-retry");
    input.click(&mut tree, "fs-tl-a-cancel");
    input.click(&mut tree, "fs-tl-c-dismiss");
    input.click_at(&mut tree, 5.0, 5.0); // empty space
    let got = got.lock().unwrap().clone();
    assert_eq!(
        got,
        vec![
            Outcome::new("b", Action::Retry),
            Outcome::new("a", Action::Cancel),
            Outcome::new("c", Action::Dismiss)
        ]
    );
}

#[test]
fn odd_data_draws_without_a_crash() {
    let _g = serial();
    let odd = vec![
        Transfer::new("", "", Direction::Upload),
        Transfer::new(
            "x",
            "ein sehr langer Dateiname äöü 李雷 😀 ".repeat(6),
            Direction::Download,
        )
        .with_state(State::Failed("e".repeat(500)))
        .with_progress(u64::MAX, Some(1)),
    ];
    let img = draw(
        TransferList::new(odd).build(),
        400,
        200,
        "transfer-list-odd.png",
    );
    // nothing sticks out to the right of the box
    let bg = background(&img);
    assert!(ink(&img, (0, 0, 400, 200), bg) > 500);
    // empty list shows the empty text
    let tree = layout(TransferList::new(vec![]).build(), 300.0, 60.0);
    assert!(tree.query_by_id("fs-tl-empty").is_some());
    // huge list: bounded drawing
    let many: Vec<Transfer> = (0..20_000)
        .map(|i| Transfer::new(format!("k{i}"), format!("f{i}"), Direction::Upload))
        .collect();
    let img = draw(
        TransferList::new(many).max_rows(30).build(),
        500,
        400,
        "transfer-list-huge.png",
    );
    assert!(ink(&img, (0, 0, 500, 400), background(&img)) > 500);
    let _ = layout(TransferList::new(mixed()).build(), 0.0, 0.0);
}

#[test]
fn a_long_failure_reason_is_readable_to_its_end() {
    let _g = serial();
    let reason =
        "the disk quota of the target is used up, free some space and try again: DISK QUOTA";
    let list = vec![
        Transfer::new("q", "big.iso", Direction::Upload).with_state(State::Failed(reason.into())),
    ];
    let img = draw(
        TransferList::new(list.clone()).build(),
        420,
        200,
        "transfer-list-long-reason.png",
    );
    // the reason wraps: the row is taller than the same row with a short reason
    let short = vec![
        Transfer::new("q", "big.iso", Direction::Upload).with_state(State::Failed("quota".into())),
    ];
    let img2 = draw(
        TransferList::new(short).build(),
        420,
        200,
        "transfer-list-short-reason.png",
    );
    let bottom = |i: &image::RgbaImage| {
        (0..200u32)
            .rev()
            .find(|&y| {
                (0..420).any(|x| {
                    i32::from(i.get_pixel(x, y).0[0]) > i32::from(i.get_pixel(x, y).0[1]) + 8
                })
            })
            .unwrap_or(0)
    };
    assert!(
        bottom(&img) > bottom(&img2) + 15,
        "{} vs {}",
        bottom(&img),
        bottom(&img2)
    );
}

#[test]
fn a_failure_never_hides_behind_the_cap() {
    let _g = serial();
    let mut many: Vec<Transfer> = (0..300)
        .map(|i| Transfer::new(format!("k{i}"), format!("f{i}"), Direction::Upload))
        .collect();
    many[250] = Transfer::new("bad", "bad.bin", Direction::Upload)
        .with_state(State::Failed("DISK QUOTA".into()));
    let list = TransferList::new(many.clone()).max_rows(100);
    assert!(list.shown().contains(&250));
    assert_eq!(list.not_shown(), 199);
    assert_eq!(list.failed_not_shown(), 0);
    let tree = layout(list.build(), 520.0, 900.0);
    assert!(
        tree.query_by_id("fs-tl-bad-retry").is_some(),
        "the failed row is drawn with its retry button"
    );
    // 20 000 failures: bounded drawing, and the line says failures are among the rest
    let all_bad: Vec<Transfer> = (0..20_000)
        .map(|i| {
            Transfer::new(format!("k{i}"), "x", Direction::Upload)
                .with_state(State::Failed("e".into()))
        })
        .collect();
    let list = TransferList::new(all_bad).max_rows(50);
    assert_eq!(list.shown().len(), 100);
    assert_eq!(list.failed_not_shown(), 19_900);
}

#[test]
fn the_list_takes_no_keys_of_its_own() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let ui = TransferList::new(mixed())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())))
        .build();
    let mut tree = layout(ui, 520.0, 420.0);
    let mut input = Input::new();
    for code in [13, 27, 32, 37, 38, 39, 40, 46] {
        input.key_broadcast(&mut tree, code);
    }
    assert!(
        got.lock().unwrap().is_empty(),
        "keys alone must not retry, cancel or dismiss anything"
    );
}
