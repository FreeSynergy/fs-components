//! Channels, scales and combinations outside the subset are refused with a named message.

mod refuse_util;

use fs_components_charts::spec::{ErrorKind, parse};
use refuse_util::{XY, bar, refused};

#[test]
fn channels_and_their_properties_are_named() {
    for ch in [
        "size",
        "opacity",
        "shape",
        "tooltip",
        "text",
        "detail",
        "order",
        "x2",
        "y2",
        "row",
        "column",
        "xOffset",
        "strokeDash",
    ] {
        let t = bar(
            "",
            &format!(r#"{XY},"{ch}":{{"field":"c","type":"nominal"}}"#),
        );
        refused(&t, ErrorKind::OutsideSubset, &format!("/encoding/{ch}"), ch);
    }
    for (prop, v) in [
        ("bin", "true"),
        ("timeUnit", "\"year\""),
        ("condition", "{}"),
        ("format", "\"d\""),
        ("impute", "{}"),
    ] {
        let t = bar(
            "",
            &format!(
                r#""x":{{"field":"a","type":"nominal","{prop}":{v}}},"y":{{"field":"b","type":"quantitative"}}"#
            ),
        );
        refused(
            &t,
            ErrorKind::OutsideSubset,
            &format!("/encoding/x/{prop}"),
            prop,
        );
    }
    refused(
        &bar(
            "",
            r#""x":{"value":3},"y":{"field":"b","type":"quantitative"}"#,
        ),
        ErrorKind::Invalid,
        "/encoding/x",
        "type",
    );
    refused(
        &bar(
            "",
            r#""x":{"field":"a","type":"geojson"},"y":{"field":"b","type":"quantitative"}"#,
        ),
        ErrorKind::OutsideSubset,
        "/encoding/x/type",
        "geojson",
    );
    refused(
        &bar(
            "",
            r#""x":{"field":"a","type":"nominal"},"y":{"field":"b","type":"quantitative","aggregate":"median"}"#,
        ),
        ErrorKind::OutsideSubset,
        "/encoding/y/aggregate",
        "median",
    );
    refused(
        &bar(
            "",
            r#""x":{"field":"a[0]","type":"nominal"},"y":{"field":"b","type":"quantitative"}"#,
        ),
        ErrorKind::OutsideSubset,
        "/encoding/x/field",
        "bracket",
    );
}

#[test]
fn scale_sort_stack_and_colour_limits() {
    let y = r#""y":{"field":"b","type":"quantitative"}"#;
    for (x, path, word) in [
        (
            r#""x":{"field":"a","type":"nominal","sort":{"field":"b"}}"#,
            "/encoding/x/sort",
            "object",
        ),
        (
            r#""x":{"field":"a","type":"nominal","sort":"-x"}"#,
            "/encoding/x/sort",
            "-x",
        ),
        (
            r#""x":{"field":"a","type":"nominal","scale":{"type":"log"}}"#,
            "/encoding/x/scale/type",
            "log",
        ),
        (
            r#""x":{"field":"a","type":"nominal","scale":{"padding":0.2}}"#,
            "/encoding/x/scale/padding",
            "padding",
        ),
        (
            r#""x":{"field":"a","type":"nominal","axis":{"orient":"top"}}"#,
            "/encoding/x/axis/orient",
            "orient",
        ),
        (
            r#""x":{"field":"a","type":"nominal","legend":null}"#,
            "/encoding/x/legend",
            "legend",
        ),
    ] {
        let t = bar("", &format!("{x},{y}"));
        let e = parse(&t).expect_err(&t);
        assert_eq!(e.path, path, "{e}");
        assert!(e.message.contains(word), "{e}");
    }
    let x = r#""x":{"field":"a","type":"nominal"}"#;
    refused(
        &bar(
            "",
            &format!(r#"{x},"y":{{"field":"b","type":"quantitative","stack":"center"}}"#),
        ),
        ErrorKind::OutsideSubset,
        "/encoding/y/stack",
        "center",
    );
    refused(
        &bar(
            "",
            &format!(r#"{x},{y},"color":{{"field":"c","type":"quantitative"}}"#),
        ),
        ErrorKind::OutsideSubset,
        "/encoding/color/type",
        "sequential",
    );
    refused(
        &bar(
            "",
            &format!(r#"{x},{y},"color":{{"field":"c","type":"ordinal"}}"#),
        ),
        ErrorKind::OutsideSubset,
        "/encoding/color/type",
        "nominal",
    );
    refused(
        &bar("", &format!(r#"{x},{y},"color":{{"value":"red"}}"#)),
        ErrorKind::Invalid,
        "/encoding/color",
        "type",
    );
}

#[test]
fn combinations_vega_lite_would_draw_differently_are_refused() {
    refused(
        &bar(
            "",
            r#""x":{"field":"a","type":"temporal"},"y":{"field":"b","type":"quantitative"}"#,
        ),
        ErrorKind::OutsideSubset,
        "/encoding",
        "bars over time",
    );
    refused(
        &bar(
            "",
            r#""x":{"field":"a","type":"quantitative"},"y":{"field":"b","type":"quantitative"}"#,
        ),
        ErrorKind::OutsideSubset,
        "/encoding",
        "bar needs",
    );
    refused(
        &bar(
            "",
            r#""x":{"field":"a","type":"nominal"},"y":{"field":"b","type":"quantitative","scale":{"zero":false}}"#,
        ),
        ErrorKind::OutsideSubset,
        "/encoding/y/scale/zero",
        "baseline",
    );
    refused(
        &bar("", r#""x":{"field":"a","type":"nominal"}"#),
        ErrorKind::Invalid,
        "/encoding/y",
        "y",
    );
    refused(
        r#"{"data":{"values":[]},"mark":"arc","encoding":{"theta":{"field":"a","type":"nominal"}}}"#,
        ErrorKind::Invalid,
        "/encoding/theta/type",
        "quantitative",
    );
    refused(
        r#"{"data":{"values":[]},"mark":"arc","encoding":{"x":{"field":"a","type":"nominal"},"theta":{"field":"a","type":"quantitative"}}}"#,
        ErrorKind::Invalid,
        "/encoding",
        "theta",
    );
    refused(
        r#"{"data":{"values":[]},"mark":"line","encoding":{"x":{"field":"a","type":"quantitative"},"y":{"field":"b","type":"nominal"}}}"#,
        ErrorKind::OutsideSubset,
        "/encoding/y/type",
        "quantitative y",
    );
    refused(
        r#"{"$schema":"https://vega.github.io/schema/vega-lite/v3.json","data":{"values":[]},"mark":"bar","encoding":{}}"#,
        ErrorKind::OutsideSubset,
        "/$schema",
        "5 or 6",
    );
    refused(
        r#"{"data":{"values":[]},"mark":"bar","width":"container","encoding":{}}"#,
        ErrorKind::OutsideSubset,
        "/width",
        "container",
    );
}

#[test]
fn not_json_and_wrong_shapes_are_clear() {
    refused("{", ErrorKind::NotJson, "", "EOF");
    refused("[]", ErrorKind::Invalid, "", "object");
    refused("", ErrorKind::NotJson, "", "");
    refused(
        r#"{"data":{"values":[]},"mark":3,"encoding":{}}"#,
        ErrorKind::Invalid,
        "/mark",
        "string or an object",
    );
    refused(
        r#"{"data":{"values":[{"a":1e999}]},"mark":"bar","encoding":{}}"#,
        ErrorKind::NotJson,
        "",
        "out of range",
    );
}
