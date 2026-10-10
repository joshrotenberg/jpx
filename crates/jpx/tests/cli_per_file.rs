//! Integration coverage for corpus-style `--per-file` evaluation.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::io::Write;
use tempfile::NamedTempFile;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

fn input_file(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create input file");
    file.write_all(contents.as_bytes()).expect("write input");
    file.flush().expect("flush input");
    file
}

#[test]
fn repeated_file_flags_emit_one_result_per_file() {
    let first = input_file(r#"{"id":1}"#);
    let second = input_file(r#"{"id":2}"#);

    jpx()
        .args(["--per-file", "--lines", "-f"])
        .arg(first.path())
        .arg("-f")
        .arg(second.path())
        .arg("id")
        .assert()
        .success()
        .stdout("1\n2\n");
}

#[test]
fn positional_files_work_after_a_positional_expression() {
    let first = input_file(r#"{"id":1}"#);
    let second = input_file(r#"{"id":2}"#);

    jpx()
        .args(["--per-file", "--compact", "id"])
        .arg(first.path())
        .arg(second.path())
        .assert()
        .success()
        .stdout("[1,2]\n");
}

#[test]
fn query_file_positionals_slurp_each_file_and_bind_file() {
    let first = input_file("{\"id\":1}\n{\"id\":2}\n");
    let second = input_file("{\"id\":3}\n");
    let mut query = tempfile::Builder::new()
        .suffix(".jpx")
        .tempfile()
        .expect("create query file");
    writeln!(query, "-- :name row").unwrap();
    writeln!(query, "{{file: $file, count: length(@)}}").unwrap();
    query.flush().unwrap();

    let output = jpx()
        .args(["--per-file", "--slurp", "--lines", "-Q"])
        .arg(format!("{}:row", query.path().display()))
        .arg(first.path())
        .arg(second.path())
        .output()
        .expect("run jpx");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["file"], first.path().to_string_lossy().as_ref());
    assert_eq!(rows[0]["count"], 2);
    assert_eq!(rows[1]["file"], second.path().to_string_lossy().as_ref());
    assert_eq!(rows[1]["count"], 1);
}

#[test]
fn null_results_are_preserved_one_per_file() {
    let first = input_file(r#"{"id":1}"#);
    let second = input_file(r#"{"id":2}"#);

    jpx()
        .args(["--per-file", "--lines", "-f"])
        .arg(first.path())
        .arg("-f")
        .arg(second.path())
        .arg("missing")
        .assert()
        .success()
        .stdout("null\nnull\n");
}

#[test]
fn exit_status_succeeds_when_any_file_result_is_truthy() {
    let first = input_file(r#"{"ok":false}"#);
    let second = input_file(r#"{"ok":true}"#);

    jpx()
        .args(["--per-file", "--exit-status", "-f"])
        .arg(first.path())
        .arg("-f")
        .arg(second.path())
        .arg("ok")
        .assert()
        .success();
}

#[test]
fn exit_status_fails_when_every_file_result_is_falsy() {
    let first = input_file(r#"{"ok":false}"#);
    let second = input_file(r#"{}"#);

    jpx()
        .args(["--per-file", "--exit-status", "-f"])
        .arg(first.path())
        .arg("-f")
        .arg(second.path())
        .arg("ok")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn file_binding_handles_a_backtick_in_the_path() {
    let mut file = tempfile::Builder::new()
        .prefix("jpx-`-input-")
        .tempfile()
        .expect("create input with backtick in path");
    file.write_all(b"{}").unwrap();
    file.flush().unwrap();

    let output = jpx()
        .args(["--per-file", "--compact", "-f"])
        .arg(file.path())
        .arg("$file")
        .output()
        .expect("run jpx");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let outer: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(outer[0], file.path().to_string_lossy().as_ref());
}

#[test]
fn quoted_file_text_is_not_treated_as_a_variable() {
    let file = input_file(r#"{}"#);

    jpx()
        .args(["--per-file", "--strict", "--compact", "-f"])
        .arg(file.path())
        .arg("'$file'")
        .assert()
        .success()
        .stdout("[\"$file\"]\n");
}

#[test]
fn output_file_contains_one_line_per_input() {
    let first = input_file(r#"{"id":1}"#);
    let second = input_file(r#"{"id":2}"#);
    let output = NamedTempFile::new().expect("create output file");

    jpx()
        .args(["--per-file", "--lines", "--output"])
        .arg(output.path())
        .arg("-f")
        .arg(first.path())
        .arg("-f")
        .arg(second.path())
        .arg("id")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    assert_eq!(std::fs::read_to_string(output.path()).unwrap(), "1\n2\n");
}
