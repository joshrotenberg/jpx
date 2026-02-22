//! Extended integration tests for jpx CLI streaming mode (--stream / --each).
//!
//! Covers: large inputs, malformed line recovery, multiple expressions,
//! file-based input, complex expressions, and edge cases.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

// ---------------------------------------------------------------------------
// 1. Large NDJSON
// ---------------------------------------------------------------------------

#[test]
fn stream_large_ndjson() {
    let input: String = (0..1000)
        .map(|i| format!(r#"{{"id":{i}}}"#))
        .collect::<Vec<_>>()
        .join("\n");

    let expected: String = (0..1000)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    jpx()
        .args(["--stream", "id", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn stream_large_ndjson_with_filter() {
    // Each line is an object; use a pipe to extract and filter.
    // Lines where id >= 990 -> 10 results (990..999)
    let input: String = (0..1000)
        .map(|i| format!(r#"{{"id":{i}}}"#))
        .collect::<Vec<_>>()
        .join("\n");

    // The expression `id` yields a number; lines with id < 990 we want to skip.
    // Streaming evaluates per-line, so we can't use array filters directly.
    // Instead we use an if_expr or rely on null-skip behavior.
    // Actually: we can use a multi-select that returns null for non-matching.
    // Simplest: use the `if` extension function: if(id >= `990`, id, null)
    // Since nulls are skipped, only matching lines appear.
    let expected: String = (990..1000)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    jpx()
        .args([
            "--stream",
            r#"if(id >= `990`, id, null)"#,
            "--color",
            "never",
        ])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(expected);
}

// ---------------------------------------------------------------------------
// 2. Malformed line recovery
// ---------------------------------------------------------------------------

#[test]
fn stream_skips_malformed_lines() {
    let input = r#"{"id":1}
not json
{"id":2}
{broken
{"id":3}"#;

    jpx()
        .args(["--stream", "id", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout("1\n2\n3\n");
}

#[test]
fn stream_malformed_stderr() {
    let input = "not json\n";

    jpx()
        .args(["--stream", "@", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("Failed to parse JSON"));
}

#[test]
fn stream_malformed_quiet() {
    let input = "not json\n";

    jpx()
        .args(["--stream", "-q", "@", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// ---------------------------------------------------------------------------
// 3. Multiple expressions (chained via -e)
// ---------------------------------------------------------------------------

#[test]
fn stream_multiple_expressions() {
    let input = r#"{"user":{"name":"alice"}}
{"user":{"name":"bob"}}"#;

    // -e user -e name should chain: first extracts .user, second extracts .name
    jpx()
        .args([
            "--stream", "-e", "user", "-e", "name", "-r", "--color", "never",
        ])
        .write_stdin(input)
        .assert()
        .success()
        .stdout("alice\nbob\n");
}

// ---------------------------------------------------------------------------
// 4. Complex expressions
// ---------------------------------------------------------------------------

#[test]
fn stream_pipe_expression() {
    let input = r#"{"name":"alice"}
{"name":"bob"}"#;

    jpx()
        .args(["--stream", "name | upper(@)", "-r", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout("ALICE\nBOB\n");
}

#[test]
fn stream_multi_select_hash() {
    let input = r#"{"id":1,"name":"alice"}
{"id":2,"name":"bob"}"#;

    jpx()
        .args([
            "--stream",
            "{id: id, upper: upper(name)}",
            "--color",
            "never",
        ])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            r#"{"id":1,"upper":"ALICE"}
{"id":2,"upper":"BOB"}
"#,
        );
}

#[test]
fn stream_filter_expression() {
    // Each line contains an array; filter elements > 50
    let input = r#"[10,60,30,80]
[5,55,95,40]"#;

    jpx()
        .args(["--stream", "[?@ > `50`]", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout("[60,80]\n[55,95]\n");
}

// ---------------------------------------------------------------------------
// 5. File input
// ---------------------------------------------------------------------------

#[test]
fn stream_from_file() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"{\"id\":1}\n{\"id\":2}\n{\"id\":3}\n")
        .unwrap();

    jpx()
        .args(["--stream", "-f"])
        .arg(tmp.path())
        .args(["id", "--color", "never"])
        .assert()
        .success()
        .stdout("1\n2\n3\n");
}

// ---------------------------------------------------------------------------
// 6. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn stream_all_malformed() {
    let input = "not json\nalso bad\n{broken\n";

    jpx()
        .args(["--stream", "-q", "@", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn stream_unicode() {
    // Use pre-composed e-acute directly to avoid decomposition mismatches
    let input =
        "{\"msg\":\"hello world\"}\n{\"msg\":\"caf\u{00e9}\"}\n{\"msg\":\"日本語テスト\"}\n";

    jpx()
        .args(["--stream", "msg", "-r", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout("hello world\ncaf\u{00e9}\n日本語テスト\n");
}

#[test]
fn stream_mixed_types() {
    // Each line is a different JSON type
    let input = r#"{"a":1}
[1,2,3]
"hello"
42
true"#;

    jpx()
        .args(["--stream", "@", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            r#"{"a":1}
[1,2,3]
"hello"
42
true
"#,
        );
}

#[test]
fn stream_whitespace_lines() {
    let input = "  \n{\"id\":1}\n\t\n{\"id\":2}\n   \n";

    jpx()
        .args(["--stream", "id", "--color", "never"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout("1\n2\n");
}
