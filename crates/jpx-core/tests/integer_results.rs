//! Regression tests for integer-valued function results.
//!
//! Integer-closed operations on integer inputs, and results that are integral
//! by construction, must return integer numbers so they serialize as `5`
//! rather than `5.0`. Results with a fractional part, and operations that are
//! not integer-closed (division, averages), keep their `f64` representation.
//!
//! `serde_json::Value` equality distinguishes `5` from `5.0`, so each check
//! pins the numeric representation as well as the value.

use serde_json::Value;

use jpx_core::Runtime;

fn json(text: &str) -> Value {
    serde_json::from_str(text).expect("fixture should be valid JSON")
}

fn eval(rt: &Runtime, expr: &str, data: &str) -> Value {
    rt.compile(expr)
        .expect("expression should compile")
        .search(&json(data))
        .expect("search should succeed")
}

fn check(rt: &Runtime, expr: &str, data: &str, expected: &str) {
    let result = eval(rt, expr, data);
    assert_eq!(result, json(expected), "{expr} over {data}");
    assert_eq!(
        result.to_string(),
        json(expected).to_string(),
        "{expr} over {data} should serialize as {expected}"
    );
}

// ---------------------------------------------------------------------------
// Built-in functions
// ---------------------------------------------------------------------------

#[test]
fn builtin_integer_results() {
    let rt = Runtime::builder().with_standard().build();
    check(&rt, "sum(a)", r#"{"a":[0,4,1]}"#, "5");
    check(&rt, "sum(@)", "[]", "0");
    check(&rt, "sum(@)", "[-3,1]", "-2");
    check(&rt, "abs(@)", "-3", "3");
    check(&rt, "abs(@)", "-9223372036854775808", "9223372036854775808");
    check(&rt, "ceil(@)", "1.5", "2");
    check(&rt, "floor(@)", "1.5", "1");
    check(&rt, "floor(@)", "-0.5", "-1");
    check(&rt, "ceil(@)", "7", "7");
}

#[test]
fn builtin_float_results_unchanged() {
    let rt = Runtime::builder().with_standard().build();
    check(&rt, "sum(@)", "[1.5,2]", "3.5");
    check(&rt, "sum(@)", "[2.0,3]", "5.0");
    check(&rt, "abs(@)", "-3.5", "3.5");
    check(&rt, "avg(@)", "[1,3]", "2.0");
    // Outside the i64 range a whole float stays a float.
    check(&rt, "floor(@)", "1e300", "1e300");
}

#[test]
fn builtin_large_integers_are_exact() {
    let rt = Runtime::builder().with_standard().build();
    // 2^53 + 1 is not representable as f64; the sum must not round it.
    check(&rt, "sum(@)", "[9007199254740992,1]", "9007199254740993");
    check(
        &rt,
        "sum(@)",
        "[9223372036854775807,1]",
        "9223372036854775808",
    );
    check(
        &rt,
        "floor(@)",
        "18446744073709551615",
        "18446744073709551615",
    );
    // A total beyond u64 falls back to f64 instead of failing.
    check(
        &rt,
        "sum(@)",
        "[18446744073709551615,1]",
        "18446744073709551616.0",
    );
}

// ---------------------------------------------------------------------------
// Extension functions
// ---------------------------------------------------------------------------

#[cfg(feature = "extensions")]
fn full_runtime() -> Runtime {
    Runtime::builder()
        .with_standard()
        .with_all_extensions()
        .build()
}

#[cfg(feature = "extensions")]
#[test]
fn math_extension_integer_results() {
    let rt = full_runtime();
    check(&rt, "add(`2`, `3`)", "null", "5");
    check(&rt, "subtract(`2`, `3`)", "null", "-1");
    check(&rt, "multiply(`4`, `3`)", "null", "12");
    check(&rt, "mod_fn(`7`, `3`)", "null", "1");
    check(&rt, "mod_fn(`-7`, `3`)", "null", "-1");
    check(&rt, "pow(`2`, `10`)", "null", "1024");
    check(&rt, "abs_fn(`-3`)", "null", "3");
    check(&rt, "clamp(`15`, `0`, `10`)", "null", "10");
    check(&rt, "round(`2.5`)", "null", "3");
    check(&rt, "round(`7`)", "null", "7");
    check(&rt, "round(`7`, `2`)", "null", "7");
    check(&rt, "round(`1234`, `-2`)", "null", "1200");
    check(&rt, "floor_fn(`1.5`)", "null", "1");
    check(&rt, "ceil_fn(`1.5`)", "null", "2");
    check(&rt, "cumulative_sum(@)", "[1,2,3]", "[1,3,6]");
    check(
        &rt,
        "multiply(`9223372036854775807`, `2`)",
        "null",
        "18446744073709551614",
    );
}

#[cfg(feature = "extensions")]
#[test]
fn math_extension_float_results_unchanged() {
    let rt = full_runtime();
    check(&rt, "add(`2`, `3.5`)", "null", "5.5");
    check(&rt, "add(`2.0`, `3`)", "null", "5.0");
    check(&rt, "divide(`10`, `2`)", "null", "5.0");
    check(&rt, "pow(`2`, `-1`)", "null", "0.5");
    check(&rt, "round(`2.567`, `2`)", "null", "2.57");
    check(&rt, "mod_fn(`7.5`, `2`)", "null", "1.5");
    check(&rt, "cumulative_sum(@)", "[1,2.5]", "[1.0,3.5]");
    check(&rt, "median(@)", "[1,2,3,4]", "2.5");
    // Out of range values used to saturate to i64::MAX via `as i64`.
    check(&rt, "floor_fn(`1e300`)", "null", "1e300");
    // Overflowing integer arithmetic falls back to f64.
    check(
        &rt,
        "multiply(`18446744073709551615`, `2`)",
        "null",
        "36893488147419103230.0",
    );
}

#[cfg(feature = "extensions")]
#[test]
fn integral_by_construction_results() {
    let rt = full_runtime();
    check(
        &rt,
        "ulid_timestamp(@)",
        r#""01D39ZY06FGSCTVN4T2V9PKHFZ""#,
        "1549744931023",
    );
    check(
        &rt,
        "to_epoch(@)",
        r#""2024-01-01T00:00:00Z""#,
        "1704067200",
    );
    check(
        &rt,
        "to_epoch_ms(@)",
        r#""2024-01-01T00:00:00Z""#,
        "1704067200000",
    );
    check(
        &rt,
        "parse_date(@)",
        r#""2024-01-01T00:00:00Z""#,
        "1704067200",
    );
    check(&rt, "date_add(`0`, `1`, 'days')", "null", "86400");
    check(&rt, "date_diff(`86400`, `0`, 'seconds')", "null", "86400");
    check(&rt, "quarter(`1704067200`)", "null", "1");
    check(&rt, "parse_duration(@)", r#""1h30m""#, "5400");
    check(&rt, "parse_bytes(@)", r#""1.5KB""#, "1500");
    check(&rt, "levenshtein('kitten', 'sitting')", "null", "3");
    check(&rt, "find_index_expr(&@ > `1`, @)", "[1,2,3]", "1");
    check(&rt, "find_index_expr(&@ > `5`, @)", "[1,2,3]", "-1");
    check(&rt, "regex_count(@, 'a')", r#""banana""#, "3");
    check(&rt, "semver_compare('1.0.0', '2.0.0')", "null", "-1");
    check(&rt, "ip_to_int('192.168.0.1')", "null", "3232235521");
    check(&rt, "now(`5`)", "null", "5");
}

#[cfg(feature = "extensions")]
#[test]
fn fractional_by_construction_results_unchanged() {
    let rt = full_runtime();
    // Units that divide seconds stay f64 even when the value is whole.
    check(&rt, "date_diff(`86400`, `0`, 'days')", "null", "1.0");
    check(&rt, "date_diff(`43200`, `0`, 'days')", "null", "0.5");
    check(&rt, "parse_bytes(@)", r#""0.5B""#, "0.5");
}

#[cfg(feature = "extensions")]
#[test]
fn numeric_string_parsing_keeps_integers_exact() {
    let rt = full_runtime();
    check(
        &rt,
        "parse_numbers(@)",
        r#"["42","1.5","18446744073709551615","1e3"]"#,
        "[42,1.5,18446744073709551615,1000.0]",
    );
    check(
        &rt,
        "auto_parse(@)",
        r#"{"n":"9007199254740993","f":"2.5"}"#,
        r#"{"n":9007199254740993,"f":2.5}"#,
    );
}
