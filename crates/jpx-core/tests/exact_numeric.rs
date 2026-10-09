//! Regression tests for exact numeric comparison (#259).
//!
//! Integers the JSON parser holds exactly (`i64`/`u64`) must not be rounded to
//! `f64` before comparison. All inputs are built from literal JSON text so that
//! no fixture is rounded before it reaches the engine.

use serde_json::Value;

use jpx_core::Runtime;

fn standard_runtime() -> Runtime {
    Runtime::builder().with_standard().build()
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).expect("fixture should be valid JSON")
}

fn eval(rt: &Runtime, expr: &str, data: &str) -> Value {
    rt.compile(expr)
        .expect("expression should compile")
        .search(&json(data))
        .expect("search should succeed")
}

/// Asserts the result equals `expected` exactly, including numeric representation.
fn check(expr: &str, data: &str, expected: &str) {
    let rt = standard_runtime();
    assert_eq!(eval(&rt, expr, data), json(expected), "{expr} over {data}");
}

// ---------------------------------------------------------------------------
// Comparators and filters
// ---------------------------------------------------------------------------

#[test]
fn issue_repro_adjacent_integers_above_2_pow_53() {
    check(
        "{a: a, b: b, equal: a == b, not_equal: a != b, less: a < b, greater: a > b}",
        r#"{"a":9007199254740992,"b":9007199254740993}"#,
        r#"{"a":9007199254740992,"b":9007199254740993,"equal":false,"not_equal":true,"less":true,"greater":false}"#,
    );
}

#[test]
fn comparators_both_operand_orders() {
    let cases = [
        // (lo, hi): lo < hi, and they are adjacent at a precision boundary.
        ("9007199254740992", "9007199254740993"),
        ("-9007199254740993", "-9007199254740992"),
        ("9223372036854775806", "9223372036854775807"),
        ("9223372036854775807", "9223372036854775808"),
        ("-9223372036854775808", "-9223372036854775807"),
        ("18446744073709551614", "18446744073709551615"),
        ("-1", "18446744073709551615"),
    ];
    for (lo, hi) in cases {
        let data = format!(r#"{{"lo":{lo},"hi":{hi}}}"#);
        check(
            "[lo == hi, hi == lo, lo != hi, lo < hi, hi > lo, lo <= hi, hi >= lo, hi < lo, lo > hi]",
            &data,
            "[false, false, true, true, true, true, true, false, false]",
        );
        check(
            "[lo == lo, hi == hi, lo <= lo, hi >= hi, lo < lo]",
            &data,
            "[true, true, true, true, false]",
        );
    }
}

#[test]
fn filter_selects_only_the_exact_id() {
    check(
        "[?@ == `9007199254740992`]",
        "[9007199254740992,9007199254740993]",
        "[9007199254740992]",
    );
    check(
        "[?`9007199254740993` == @]",
        "[9007199254740992,9007199254740993]",
        "[9007199254740993]",
    );
    check(
        "[?@ == `18446744073709551614`]",
        "[18446744073709551614,18446744073709551615]",
        "[18446744073709551614]",
    );
    check(
        "[?id != `9007199254740992`].id",
        r#"[{"id":9007199254740992},{"id":9007199254740993}]"#,
        "[9007199254740993]",
    );
    check(
        "[?id > `9007199254740992`].id",
        r#"[{"id":9007199254740992},{"id":9007199254740993}]"#,
        "[9007199254740993]",
    );
}

#[test]
fn nested_array_and_object_equality() {
    check(
        "[a == b, b == a, a == c, c == a]",
        r#"{
            "a": {"ids": [9007199254740992], "owner": {"id": 18446744073709551614}},
            "b": {"ids": [9007199254740993], "owner": {"id": 18446744073709551614}},
            "c": {"ids": [9007199254740992.0], "owner": {"id": 18446744073709551614}}
        }"#,
        "[false, false, true, true]",
    );
    check(
        "[?tags == `[1, 9007199254740993]`].name",
        r#"[{"name":"x","tags":[1,9007199254740992]},{"name":"y","tags":[1.0,9007199254740993]}]"#,
        r#"["y"]"#,
    );
}

#[test]
fn mixed_integer_float_contract() {
    check(
        "[`1` == `1.0`, `1.0` == `1`, `0` == `-0.0`, `-0.0` == `0`, `-1` == `-1.0`]",
        "{}",
        "[true, true, true, true, true]",
    );
    check(
        "[`1` < `1.5`, `2` > `1.5`, `-1` < `-0.5`, `-1` > `-1.5`]",
        "{}",
        "[true, true, true, true]",
    );
    // 9007199254740992.0 is exactly 2^53; 2^53 + 1 must stay above it.
    check(
        "[a == f, f == a, b == f, f == b, b > f, f < b]",
        r#"{"a":9007199254740992,"b":9007199254740993,"f":9007199254740992.0}"#,
        "[true, true, false, false, true, true]",
    );
    // 2^64 as a float is above u64::MAX (no saturating cast).
    check(
        "[m == f, m < f, f > m]",
        r#"{"m":18446744073709551615,"f":18446744073709551616.0}"#,
        "[false, true, true]",
    );
    // Filters keep both integer and float spellings of the same value.
    check("[?@ == `1`]", "[1.0, 1, 2, 1.5]", "[1.0, 1]");
}

// ---------------------------------------------------------------------------
// Standard functions
// ---------------------------------------------------------------------------

#[test]
fn sort_orders_adjacent_large_integers() {
    // Each adjacent pair is given out of order; a lossy comparator would treat
    // them as equal and the stable sort would leave them unsorted.
    check(
        "sort(@)",
        "[18446744073709551615,18446744073709551614,9007199254740993,9007199254740992,-9223372036854775807,-9223372036854775808,1.5,1]",
        "[-9223372036854775808,-9223372036854775807,1,1.5,9007199254740992,9007199254740993,18446744073709551614,18446744073709551615]",
    );
}

#[test]
fn sort_by_orders_adjacent_large_integers() {
    check(
        "sort_by(@, &id)[*].name",
        r#"[{"name":"b","id":9007199254740993},{"name":"a","id":9007199254740992},{"name":"d","id":18446744073709551615},{"name":"c","id":18446744073709551614}]"#,
        r#"["a","b","c","d"]"#,
    );
}

#[test]
fn min_max_pick_exact_extremes() {
    for data in [
        "[9007199254740992,9007199254740993]",
        "[9007199254740993,9007199254740992]",
    ] {
        check("max(@)", data, "9007199254740993");
        check("min(@)", data, "9007199254740992");
    }
    for data in [
        "[18446744073709551614,18446744073709551615]",
        "[18446744073709551615,18446744073709551614]",
    ] {
        check("max(@)", data, "18446744073709551615");
        check("min(@)", data, "18446744073709551614");
    }
}

#[test]
fn min_by_max_by_pick_exact_extremes() {
    for data in [
        r#"[{"name":"a","id":9007199254740992},{"name":"b","id":9007199254740993}]"#,
        r#"[{"name":"b","id":9007199254740993},{"name":"a","id":9007199254740992}]"#,
    ] {
        check("max_by(@, &id).name", data, r#""b""#);
        check("min_by(@, &id).name", data, r#""a""#);
    }
}

#[test]
fn contains_uses_exact_numeric_equality() {
    check(
        "[contains(@, `9007199254740993`), contains(@, `9007199254740992`), contains(@, `1`)]",
        "[9007199254740992, 1.0]",
        "[false, true, true]",
    );
}

// ---------------------------------------------------------------------------
// Extension functions sharing the comparator
// ---------------------------------------------------------------------------

#[cfg(feature = "extensions")]
#[test]
fn extension_sorting_uses_exact_comparison() {
    let rt = Runtime::builder()
        .with_standard()
        .with_all_extensions()
        .build();
    let data = r#"[{"name":"b","id":9007199254740993},{"name":"a","id":9007199254740992}]"#;
    assert_eq!(
        eval(&rt, "sort_by_expr(&id, @)[*].name", data),
        json(r#"["a","b"]"#)
    );
    assert_eq!(eval(&rt, "max_by_expr(&id, @).name", data), json(r#""b""#));
    assert_eq!(eval(&rt, "min_by_expr(&id, @).name", data), json(r#""a""#));
    assert_eq!(
        eval(
            &rt,
            "bsearch(@, `9007199254740993`)",
            "[9007199254740992,9007199254740993]"
        ),
        json("1")
    );
}
