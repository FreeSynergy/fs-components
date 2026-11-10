//! Live probe helper: send keys and mouse to the nested KWin through org_kde_kwin_fake_input.
//! Usage: inject click X Y | move X Y | type TEXT | key CODE [CODE..] | wait MS
use std::{thread::sleep, time::Duration};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::wl_registry,
};
use wayland_protocols_plasma::fake_input::client::org_kde_kwin_fake_input::OrgKdeKwinFakeInput;

struct S;
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for S {
    fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event, _: &GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<OrgKdeKwinFakeInput, ()> for S {
    fn event(_: &mut Self, _: &OrgKdeKwinFakeInput, _: <OrgKdeKwinFakeInput as wayland_client::Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

fn code(c: char) -> Option<(u32, bool)> {
    let row = "qwertyuiop"; let row2 = "asdfghjkl"; let row3 = "zxcvbnm";
    let lower = c.to_ascii_lowercase();
    let shift = c.is_ascii_uppercase();
    if let Some(i) = row.find(lower) { return Some((16 + i as u32, shift)); }
    if let Some(i) = row2.find(lower) { return Some((30 + i as u32, shift)); }
    if let Some(i) = row3.find(lower) { return Some((44 + i as u32, shift)); }
    match c {
        '1'..='9' => Some((2 + (c as u32 - '1' as u32), false)),
        '0' => Some((11, false)),
        ' ' => Some((57, false)),
        '.' => Some((52, false)),
        '@' => Some((3, true)), // shift+2 on a US layout
        '+' => Some((13, true)),
        '-' => Some((12, false)),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let conn = Connection::connect_to_env().expect("connect");
    let (globals, mut queue) = registry_queue_init::<S>(&conn).expect("registry");
    let qh = queue.handle();
    let fake: OrgKdeKwinFakeInput = globals.bind(&qh, 4..=5, ()).expect("fake_input missing");
    fake.authenticate("probe".into(), "live probe".into());
    let mut s = S;
    queue.roundtrip(&mut s).ok();
    if args.first().map(String::as_str) == Some("repl") {
        let stdin = std::io::stdin();
        for line in std::io::BufRead::lines(stdin.lock()) {
            let words: Vec<String> = line.unwrap().split_whitespace().map(String::from).collect();
            if words.is_empty() { continue; }
            if words[0] == "quit" { return; }
            run(&words, &fake, &mut queue, &mut s);
        }
        return;
    }
    run(&args, &fake, &mut queue, &mut s);
}

fn run(args: &[String], fake: &OrgKdeKwinFakeInput, queue: &mut wayland_client::EventQueue<S>, s: &mut S) {
    let flush = |q: &mut wayland_client::EventQueue<S>, s: &mut S| { q.roundtrip(s).ok(); };
    let mut it = args.iter();
    while let Some(cmd) = it.next() {
        match cmd.as_str() {
            "move" | "click" => {
                let x: f64 = it.next().unwrap().parse().unwrap();
                let y: f64 = it.next().unwrap().parse().unwrap();
                fake.pointer_motion_absolute(x, y);
                flush(queue, s);
                if cmd == "click" {
                    sleep(Duration::from_millis(80));
                    fake.button(0x110, 1); flush(queue, s);
                    sleep(Duration::from_millis(60));
                    fake.button(0x110, 0); flush(queue, s);
                }
            }
            "type" => {
                for c in it.next().unwrap().chars() {
                    let Some((k, shift)) = code(c) else { continue };
                    if shift { fake.keyboard_key(42, 1); }
                    fake.keyboard_key(k, 1); flush(queue, s);
                    fake.keyboard_key(k, 0);
                    if shift { fake.keyboard_key(42, 0); }
                    flush(queue, s);
                    sleep(Duration::from_millis(40));
                }
            }
            "key" => {
                let k: u32 = it.next().unwrap().parse().unwrap();
                fake.keyboard_key(k, 1); flush(queue, s);
                fake.keyboard_key(k, 0); flush(queue, s);
            }
            "down" | "up" => {
                let k: u32 = it.next().unwrap().parse().unwrap();
                fake.keyboard_key(k, u32::from(cmd == "down"));
                flush(queue, s);
            }
            "wait" => sleep(Duration::from_millis(it.next().unwrap().parse().unwrap())),
            other => panic!("unknown {other}"),
        }
        sleep(Duration::from_millis(30));
    }
}
