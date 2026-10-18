//! CLI regression tests for exact numeric comparison (#259).
//!
//! Inputs are passed as literal JSON bytes so no value is rounded before jpx
//! parses it.

use serde_json::Value;
use std::io::Write;

fn jpx() -> assert_cmd::Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

fn run_stdin(expr: &str, input: &str) -> Value {
    let output = jpx()
        .arg("-c")
        .arg("--color")
        .arg("never")
        .arg(expr)
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("stdout should be JSON")
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).expect("fixture should be valid JSON")
}

#[test]
fn stdin_comparison_keeps_adjacent_integers_distinct() {
    assert_eq!(
        run_stdin(
            "{a:a,b:b,equal:a == b,less:a < b}",
            r#"{"a":9007199254740992,"b":9007199254740993}"#
        ),
        json(r#"{"a":9007199254740992,"b":9007199254740993,"equal":false,"less":true}"#)
    );
}

#[test]
fn stdin_filter_selects_only_the_exact_id() {
    assert_eq!(
        run_stdin(
            "[?@ == `9007199254740992`]",
            "[9007199254740992,9007199254740993]"
        ),
        json("[9007199254740992]")
    );
    assert_eq!(
        run_stdin(
            "[?@ == `18446744073709551614`]",
            "[18446744073709551614,18446744073709551615]"
        ),
        json("[18446744073709551614]")
    );
}

#[test]
fn file_input_sort_and_max_are_exact() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    write!(tmp, "[9007199254740993,9007199254740992]").unwrap();

    let output = jpx()
        .arg("-c")
        .arg("--color")
        .arg("never")
        .arg("-f")
        .arg(tmp.path())
        .arg("{sorted: sort(@), max: max(@), min: min(@)}")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&output).expect("stdout should be JSON");
    assert_eq!(
        result,
        json(
            r#"{"sorted":[9007199254740992,9007199254740993],"max":9007199254740993,"min":9007199254740992}"#
        )
    );
}
