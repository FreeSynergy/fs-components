//! Anything outside the subset is refused with a message that names it, never drawn partly.

mod refuse_util;

use fs_components_charts::spec::ErrorKind;
use refuse_util::{XY, bar, refused};

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
