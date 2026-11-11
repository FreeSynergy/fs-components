//! One case for every way `mvt-reader`'s `parse_geometry` can take memory, each in the size where the budget of the
//! guard (64 MiB, reached from about a megabyte of input) is what is left to hold: a tile of 2 MB whose geometry is as
//! bad as it can be for that branch. The rule: peak at most the budget, whether the guard refuses the tile or accepts it.
//!
//! Branches of `parse_geometry`: unknown type (error at once); `MoveTo` (a new line starts: a buffer in lines, a point
//! in points, nothing in polygons); `LineTo` (a coordinate: a point in points); `ClosePath` (a buffer in every type);
//! commands that are none (3 to 6, 0) and whose numbers are read as commands; a count that is larger than the data; the
//! end of each type (`MultiPoint`, `MultiLineString`, `MultiPolygon`, with the clones of the rings of a polygon).
//!
//! Pattern: none (tests with a counting allocator, one test so no other thread allocates). Run under `limited.sh -m 1G`.

mod bombs;
mod counting;

use bombs::*;
use counting::{Counting, allowed, peak_of};

#[global_allocator]
static COUNTING: Counting = Counting;

/// A feature of the given type with this geometry and, if `name` is given, the tag `name`, in the layer `layer_name`.
fn one(kind: u64, geometry: &[u64], layer_name: &str, name: Option<&str>) -> Vec<u8> {
    let mut f = Vec::new();
    if name.is_some() {
        let mut pair = Vec::new();
        varint(0, &mut pair);
        varint(0, &mut pair);
        bytes_field(2, &pair, &mut f);
    }
    number_field(3, kind, &mut f);
    let mut packed = Vec::new();
    for v in geometry {
        varint(*v, &mut packed);
    }
    bytes_field(4, &packed, &mut f);
    let mut l = Vec::new();
    bytes_field(1, layer_name.as_bytes(), &mut l);
    bytes_field(2, &f, &mut l);
    if let Some(name) = name {
        bytes_field(3, b"name", &mut l);
        let mut value = Vec::new();
        bytes_field(1, name.as_bytes(), &mut value);
        bytes_field(4, &value, &mut l);
    }
    number_field(5, 4096, &mut l);
    tile(&[l])
}

/// `command` with `count`, then `count` pairs of zeros.
fn run(command: u64, count: u64) -> Vec<u64> {
    let mut g = vec![(count << 3) | command];
    g.extend(std::iter::repeat_n(0, usize::try_from(2 * count).unwrap()));
    g
}

const N: u64 = 1_100_000;

#[test]
#[allow(clippy::too_many_lines)]
fn every_branch_of_the_decoder_stays_inside_the_budget_the_guard_counts() {
    let many = |n: u64, group: &[u64]| -> Vec<u64> {
        group
            .iter()
            .copied()
            .cycle()
            .take(group.len() * usize::try_from(n).unwrap())
            .collect()
    };
    let cases: Vec<(&str, Vec<u8>, Option<bool>)> = vec![
        // (name, tile, Some(true) = must be refused, Some(false) = must decode, None = either)
        (
            "unknown type 0, big geometry",
            one(0, &run(1, N), "x", None),
            None,
        ),
        (
            "unknown type 4, big geometry",
            one(4, &run(1, N), "x", None),
            None,
        ),
        ("point: MoveTo 1.1 M", one(1, &run(1, N), "x", None), None),
        (
            "point: MoveTo 1.1 M, labels",
            one(1, &run(1, N), "places", Some("a")),
            None,
        ),
        (
            "point: MoveTo 1 + LineTo 1.1 M, labels (the review's probe)",
            one(1, &[vec![9, 0, 0], run(2, N)].concat(), "places", Some("a")),
            Some(true),
        ),
        (
            "point: MoveTo 1 + LineTo 1.1 M, dots",
            one(1, &[vec![9, 0, 0], run(2, N)].concat(), "x", None),
            None,
        ),
        (
            "point: MoveTo 1 + 500 000 ClosePath",
            one(1, &[vec![9, 0, 0], vec![15; 500_000]].concat(), "x", None),
            Some(true),
        ),
        (
            "line: 1.1 M LineTo in one line (too much for the guard, which is honest about it)",
            one(2, &[vec![9, 0, 0], run(2, N)].concat(), "roads", None),
            None,
        ),
        (
            "line: 400 000 LineTo in one line (decodes)",
            one(2, &[vec![9, 0, 0], run(2, 400_000)].concat(), "roads", None),
            Some(false),
        ),
        (
            "line: 360 000 MoveTo",
            one(2, &many(360_000, &[9, 0, 0]), "roads", None),
            Some(true),
        ),
        (
            "line: MoveTo, LineTo x 180 000 (a line each)",
            one(2, &many(180_000, &[9, 0, 0, 10, 2, 2]), "roads", None),
            Some(true),
        ),
        (
            "line: 500 000 ClosePath after a MoveTo",
            one(
                2,
                &[vec![9, 0, 0, 10, 2, 2], vec![15; 500_000]].concat(),
                "roads",
                None,
            ),
            Some(true),
        ),
        (
            "polygon: one ring of 1.1 M LineTo",
            one(
                3,
                &[vec![9, 0, 0], run(2, N), vec![15]].concat(),
                "water",
                None,
            ),
            None,
        ),
        (
            "polygon: 500 000 ClosePath",
            one(
                3,
                &[vec![9, 0, 0, 10, 2, 2], vec![15; 500_000]].concat(),
                "water",
                None,
            ),
            Some(true),
        ),
        (
            "polygon: 360 000 MoveTo and one ring",
            one(
                3,
                &[many(360_000, &[9, 0, 0]), vec![15]].concat(),
                "water",
                None,
            ),
            None,
        ),
        (
            "polygon: 100 000 rings of a triangle",
            one(
                3,
                &many(30_000, &[9, 0, 0, 18, 2, 0, 0, 2, 15]),
                "water",
                None,
            ),
            None,
        ),
        (
            "commands that are none: 3 repeated",
            one(2, &vec![3; 1_100_000], "roads", None),
            None,
        ),
        (
            "a count larger than the data: LineTo (1<<29)-1",
            one(
                1,
                &[
                    vec![9, 0, 0, (((1 << 29) - 1) << 3) | 2],
                    vec![0; 1_000_000],
                ]
                .concat(),
                "x",
                None,
            ),
            None,
        ),
        (
            "a count of 2^29-1 then MoveTo-looking numbers",
            one(
                2,
                &[vec![(((1 << 29) - 1) << 3) | 1], many(300_000, &[9, 0, 0])].concat(),
                "roads",
                None,
            ),
            None,
        ),
    ];
    // the probe of the review in sizes from small to over the limit: each that is accepted stays inside the budget
    let mut sizes = Vec::new();
    for n in [
        100_000u64, 200_000, 300_000, 400_000, 500_000, 600_000, 800_000,
    ] {
        sizes.push((
            format!("labels: MoveTo 1 + LineTo {n}"),
            one(1, &[vec![9, 0, 0], run(2, n)].concat(), "places", Some("a")),
            None,
        ));
    }
    let cases: Vec<_> = cases
        .into_iter()
        .map(|(n, t, m)| (n.to_owned(), t, m))
        .chain(sizes)
        .collect();
    for (name, tile, must) in &cases {
        let (peak, ok) = peak_of(tile);
        let budget = allowed(tile.len());
        println!(
            "{name}: {} bytes in, peak {peak}, budget {budget}, decoded: {ok}",
            tile.len()
        );
        assert!(
            peak <= budget,
            "{name}: {} bytes of input took {peak} bytes, the budget is {budget} (decoded: {ok})",
            tile.len()
        );
        match must {
            Some(true) => assert!(!ok, "{name} must be refused"),
            Some(false) => assert!(ok, "{name} must decode"),
            None => {}
        }
    }
}
