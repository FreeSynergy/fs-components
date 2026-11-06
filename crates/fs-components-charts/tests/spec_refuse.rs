//! Anything outside the subset is refused with a message that names it, never drawn partly.

use fs_components_charts::spec::{ErrorKind, parse};

const XY: &str = r#""x":{"field":"a","type":"nominal"},"y":{"field":"b","type":"quantitative"}"#;

fn bar(extra_top: &str, enc: &str) -> String {
    format!(r#"{{"data":{{"values":[]}},"mark":"bar",{extra_top}"encoding":{{{enc}}}}}"#)
}

fn refused(text: &str, kind: ErrorKind, path: &str, word: &str) {
    let e = parse(text).expect_err(text);
    assert_eq!(e.kind, kind, "{text}: {e}");
    assert_eq!(e.path, path, "{text}: {e}");
    assert!(
        e.message.contains(word),
        "{text}: message {:?} lacks {word:?}",
        e.message
    );
    assert!(e.to_string().contains(path), "{e}");
}

#[test]
fn whole_spec_features_are_named() {
    for (key, value) in [
        ("transform", "[]"),
        ("layer", "[]"),
        ("params", "[]"),
        ("config", "{}"),
        ("autosize", "\"fit\""),
        ("background", "\"red\""),
        ("padding", "5"),
        ("facet", "{}"),
        ("projection", "{}"),
        ("resolve", "{}"),
        ("datasets", "{}"),
    ] {
        let t = bar(&format!(r#""{key}":{value},"#), XY);
        refused(&t, ErrorKind::OutsideSubset, &format!("/{key}"), key);
    }
}

#[test]
fn other_data_sources_are_refused() {
    refused(
        r#"{"data":{"url":"d.csv"},"mark":"bar","encoding":{}}"#,
        ErrorKind::OutsideSubset,
        "/data",
        "inline",
    );
    refused(
        r#"{"data":{"name":"t"},"mark":"bar","encoding":{}}"#,
        ErrorKind::OutsideSubset,
        "/data",
        "inline",
    );
    refused(
        r#"{"data":{"values":[1,2]},"mark":"bar","encoding":{}}"#,
        ErrorKind::Invalid,
        "/data/values/0",
        "object",
    );
    refused(
        r#"{"data":{"values":"a,b\n1,2"},"mark":"bar","encoding":{}}"#,
        ErrorKind::OutsideSubset,
        "/data/values",
        "array",
    );
    refused(
        r#"{"data":{"values":[],"format":{"type":"csv"}},"mark":"bar","encoding":{}}"#,
        ErrorKind::OutsideSubset,
        "/data/format",
        "format",
    );
    refused(
        r#"{"mark":"bar","encoding":{}}"#,
        ErrorKind::Invalid,
        "",
        "data",
    );
}

#[test]
fn marks_outside_the_subset_are_named() {
    for m in [
        "rect", "rule", "text", "tick", "circle", "square", "trail", "boxplot", "errorbar",
        "geoshape",
    ] {
        let t = format!(r#"{{"data":{{"values":[]}},"mark":"{m}","encoding":{{{XY}}}}}"#);
        refused(&t, ErrorKind::OutsideSubset, "/mark", m);
    }
    refused(
        &format!(r#"{{"data":{{"values":[]}},"mark":"pizza","encoding":{{{XY}}}}}"#),
        ErrorKind::Invalid,
        "/mark",
        "pizza",
    );
    refused(
        &format!(
            r#"{{"data":{{"values":[]}},"mark":{{"type":"bar","tooltip":true}},"encoding":{{{XY}}}}}"#
        ),
        ErrorKind::OutsideSubset,
        "/mark/tooltip",
        "tooltip",
    );
    refused(
        &format!(
            r#"{{"data":{{"values":[]}},"mark":{{"type":"line","interpolate":"monotone"}},"encoding":{{{XY}}}}}"#
        ),
        ErrorKind::OutsideSubset,
        "/mark/interpolate",
        "monotone",
    );
    refused(
        &format!(
            r#"{{"data":{{"values":[]}},"mark":{{"type":"bar","color":"red"}},"encoding":{{{XY}}}}}"#
        ),
        ErrorKind::OutsideSubset,
        "/mark/color",
        "color",
    );
}

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
