//! Prints the scene of a spec (stdin) as text, to look at layout numbers.
use std::io::Read;
fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    let c = fs_components_charts::Chart::from_json(&s).unwrap();
    let sc = c.scene(
        400.0,
        300.0,
        &fs_components_charts::Labels::default(),
        &fs_components_charts::layout::Fixed,
        &fs_components_charts::layout::Metrics::default(),
    );
    println!("plot {:?}", sc.plot);
    for i in &sc.items {
        println!("{i:?}");
    }
}
