//! An accepted spec must mean what Vega-Lite says: stacks, schemas and scales we do not follow
//! are refused by name.

mod refuse_util;

use fs_components_charts::spec::{ErrorKind, parse};
use refuse_util::refused;

fn spec(mark: &str, enc: &str, top: &str) -> String {
    format!(r#"{{{top}"data":{{"values":[]}},"mark":"{mark}","encoding":{{{enc}}}}}"#)
}

const X: &str = r#""x":{"field":"x","type":"quantitative"}"#;
const Y: &str = r#""y":{"field":"v","type":"quantitative"}"#;
const C: &str = r#""color":{"field":"g","type":"nominal"}"#;

#[test]
fn stack_is_only_followed_where_vega_lite_stacks_and_we_implemented_it() {
    for (mark, enc) in [
        (
            "line",
            format!(r#"{X},"y":{{"field":"v","type":"quantitative","stack":"zero"}},{C}"#),
        ),
        (
            "point",
            format!(r#"{X},"y":{{"field":"v","type":"quantitative","stack":"normalize"}},{C}"#),
        ),
        (
            "area",
            format!(r#""x":{{"field":"x","type":"quantitative","stack":"zero"}},{Y},{C}"#),
        ),
    ] {
        let e = parse(&spec(mark, &enc, "")).expect_err(&enc);
        assert_eq!(e.kind, ErrorKind::OutsideSubset, "{mark}: {e}");
        assert!(e.path.ends_with("/stack"), "{e}");
    }
    // stack on theta is no part of the subset (a pie is always one pile), null or not
    refused(
        r#"{"data":{"values":[]},"mark":"arc","encoding":{"theta":{"field":"v","type":"quantitative","stack":null}}}"#,
        ErrorKind::OutsideSubset,
        "/encoding/theta/stack",
        "stack",
    );
    // where it is harmless it is fine: no stacking on a line or point, stacked bars and areas
    assert!(
        parse(&spec(
            "line",
            &format!(r#"{X},"y":{{"field":"v","type":"quantitative","stack":null}},{C}"#),
            ""
        ))
        .is_ok()
    );
    assert!(
        parse(&spec(
            "area",
            &format!(r#"{X},"y":{{"field":"v","type":"quantitative","stack":"normalize"}},{C}"#),
            ""
        ))
        .is_ok()
    );
}

#[test]
fn only_real_vega_lite_5_and_6_schema_addresses_pass() {
    let ok = |s: &str| {
        parse(&spec(
            "point",
            &format!("{X},{Y}"),
            &format!(r#""$schema":"{s}","#),
        ))
        .is_ok()
    };
    let base = "https://vega.github.io/schema/vega-lite";
    for good in ["v5.json", "v5.23.0.json", "v6.json", "v6.5.json"] {
        assert!(ok(&format!("{base}/{good}")), "{good}");
    }
    for bad in [
        "v50.json",
        "v5-anything.json",
        "v5.json.evil",
        "v4.json",
        "v5..json",
        "v5.x.json",
        "v7.json",
        "v5.1.2.3.json",
    ] {
        assert!(!ok(&format!("{base}/{bad}")), "{bad} must be refused");
    }
    assert!(!ok("http://vega.github.io/schema/vega-lite/v5.json"));
}

#[test]
fn nice_on_a_time_scale_and_absurd_sizes_are_refused() {
    refused(
        &spec(
            "point",
            &format!(r#""x":{{"field":"d","type":"temporal","scale":{{"nice":true}}}},{Y}"#),
            "",
        ),
        ErrorKind::OutsideSubset,
        "/encoding/x/scale/nice",
        "zone",
    );
    for w in ["1e300", "1000001", "0", "-5"] {
        let e = parse(&spec(
            "point",
            &format!("{X},{Y}"),
            &format!(r#""width":{w},"#),
        ))
        .expect_err(w);
        assert_eq!(e.path, "/width", "{w}");
    }
    assert!(parse(&spec("point", &format!("{X},{Y}"), r#""width":1000000,"#)).is_ok());
}

#[test]
fn color_cannot_be_sorted_by_the_other_axis() {
    refused(
        &spec(
            "bar",
            &format!(
                r#""x":{{"field":"k","type":"nominal"}},{Y},"color":{{"field":"g","type":"nominal","sort":"-y"}}"#
            ),
            "",
        ),
        ErrorKind::OutsideSubset,
        "/encoding/color/sort",
        "sort",
    );
}
