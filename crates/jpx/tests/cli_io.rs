//! Integration tests for jpx CLI input/output behavior.
//!
//! Tests cover input modes (stdin, file, null-input, slurp, stream),
//! output formats (JSON, compact, raw, YAML, TOML, CSV, TSV, lines, table),
//! file output, and color modes.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

mod input_modes {
    use super::*;

    #[test]
    fn stdin_json_object() {
        jpx()
            .arg("a")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout("1\n");
    }

    #[test]
    fn stdin_json_array() {
        jpx()
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin("[1,2]")
            .assert()
            .success()
            .stdout("[\n  1,\n  2\n]\n");
    }

    #[test]
    fn file_input() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, r#"{{"name":"test"}}"#).unwrap();

        jpx()
            .arg("-f")
            .arg(tmp.path())
            .arg("name")
            .arg("--color")
            .arg("never")
            .assert()
            .success()
            .stdout("\"test\"\n");
    }

    #[test]
    fn file_not_found() {
        jpx()
            .arg("-f")
            .arg("/tmp/nonexistent_jpx_test_8f3a2c.json")
            .arg("@")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("not found")
                    .or(predicate::str::contains("File not found")),
            );
    }

    #[test]
    fn null_input() {
        jpx()
            .arg("-n")
            .arg("`42`")
            .assert()
            .success()
            .stdout("42\n");
    }

    #[test]
    fn null_input_ignores_stdin() {
        // With -n, @ evaluates to null. Null results produce no output.
        jpx()
            .arg("-n")
            .arg("@")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout("");
    }

    #[test]
    fn slurp_combines() {
        jpx()
            .arg("-s")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin("1\n2\n3")
            .assert()
            .success()
            .stdout(
                predicate::str::contains("[")
                    .and(predicate::str::contains("1"))
                    .and(predicate::str::contains("2"))
                    .and(predicate::str::contains("3")),
            );
    }

    #[test]
    fn slurp_with_expression() {
        jpx()
            .arg("-s")
            .arg("length(@)")
            .write_stdin("1\n2\n3")
            .assert()
            .success()
            .stdout("3\n");
    }

    #[test]
    fn stream_ndjson() {
        jpx()
            .arg("--stream")
            .arg("id")
            .write_stdin("{\"id\":1}\n{\"id\":2}")
            .assert()
            .success()
            .stdout("1\n2\n");
    }

    #[test]
    fn stream_skips_null() {
        // Second line has no "a" field, so result is null and gets skipped.
        jpx()
            .arg("--stream")
            .arg("a")
            .write_stdin("{\"a\":1}\n{\"b\":2}")
            .assert()
            .success()
            .stdout("1\n");
    }

    #[test]
    fn null_input_short() {
        // -n with now() returns a numeric timestamp
        jpx()
            .args(["-n", "now()"])
            .assert()
            .success()
            .stdout(predicate::str::is_match(r"\d+").unwrap());
    }

    #[test]
    fn empty_stdin_no_data() {
        // Empty stdin with an expression fails to parse
        jpx().arg("@").write_stdin("").assert().failure().code(1);
    }

    #[test]
    fn invalid_json_stdin() {
        jpx().arg("@").write_stdin("not json").assert().failure();
    }

    #[test]
    fn stdin_no_expression_defaults_to_identity() {
        jpx()
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"a\": 1").and(predicate::str::contains("\"b\": 2")));
    }

    #[test]
    fn file_no_expression_defaults_to_identity() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, r#"{{"name":"test"}}"#).unwrap();

        jpx()
            .arg("-f")
            .arg(tmp.path())
            .arg("--color")
            .arg("never")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\": \"test\""));
    }

    #[test]
    fn stream_unbuffered_outputs_records() {
        jpx()
            .arg("--stream")
            .arg("--unbuffered")
            .arg("id")
            .write_stdin("{\"id\":1}\n{\"id\":2}\n{\"id\":3}")
            .assert()
            .success()
            .stdout("1\n2\n3\n");
    }

    #[test]
    fn stream_no_expression_defaults_to_identity() {
        jpx()
            .arg("--stream")
            .arg("--color")
            .arg("never")
            .write_stdin("{\"id\":1}\n{\"id\":2}")
            .assert()
            .success()
            .stdout(
                predicate::str::contains("\"id\"")
                    .and(predicate::str::contains("1").and(predicate::str::contains("2"))),
            );
    }
}

mod raw_input {
    use super::*;

    #[test]
    fn raw_input_single_line() {
        jpx()
            .args(["-R", "@"])
            .write_stdin("hello world\n")
            .assert()
            .success()
            .stdout("\"hello world\"\n");
    }

    #[test]
    fn raw_input_multiple_lines() {
        jpx()
            .args(["-R", "upper(@)"])
            .write_stdin("hello\nworld\n")
            .assert()
            .success()
            .stdout("\"HELLO\"\n\"WORLD\"\n");
    }

    #[test]
    fn raw_input_slurp() {
        jpx()
            .args(["-Rs", "@", "--color", "never"])
            .write_stdin("a\nb\nc\n")
            .assert()
            .success()
            .stdout("[\n  \"a\",\n  \"b\",\n  \"c\"\n]\n");
    }

    #[test]
    fn raw_input_slurp_length() {
        jpx()
            .args(["-Rs", "length(@)"])
            .write_stdin("a\nb\nc\n")
            .assert()
            .success()
            .stdout("3\n");
    }

    #[test]
    fn raw_input_with_expression() {
        jpx()
            .args(["-R", "-c", "split(@, ' ')"])
            .write_stdin("hello world\n")
            .assert()
            .success()
            .stdout("[\"hello\",\"world\"]\n");
    }

    #[test]
    fn raw_input_empty_lines() {
        jpx()
            .args(["-R", "@"])
            .write_stdin("a\n\nb\n")
            .assert()
            .success()
            .stdout("\"a\"\n\"\"\n\"b\"\n");
    }
}

mod output_formats {
    use super::*;

    #[test]
    fn default_pretty_json() {
        jpx()
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\n").and(predicate::str::contains("  ")));
    }

    #[test]
    fn compact_json() {
        jpx()
            .arg("-c")
            .arg("@")
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::starts_with("{\"a\":1,\"b\":2}"));
    }

    #[test]
    fn raw_string_output() {
        jpx()
            .arg("-r")
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice\n");
    }

    #[test]
    fn raw_non_string() {
        // Non-string values still print normally (as JSON) with -r.
        jpx()
            .arg("-r")
            .arg("n")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"n":42}"#)
            .assert()
            .success()
            .stdout("42\n");
    }

    #[test]
    fn join_output_no_newline() {
        jpx()
            .args(["-j", "name"])
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice");
    }

    #[test]
    fn join_output_non_string() {
        jpx()
            .args(["-j", "n"])
            .write_stdin(r#"{"n":42}"#)
            .assert()
            .success()
            .stdout("42");
    }

    #[test]
    fn join_output_streaming() {
        jpx()
            .args(["-j", "--stream", "name"])
            .write_stdin("{\"name\":\"a\"}\n{\"name\":\"b\"}")
            .assert()
            .success()
            .stdout("ab");
    }

    #[test]
    fn yaml_output() {
        jpx()
            .arg("--yaml")
            .arg("@")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("a: 1"));
    }

    #[test]
    fn yaml_short() {
        jpx()
            .arg("-y")
            .arg("@")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("a: 1"));
    }

    #[test]
    fn toml_output() {
        jpx()
            .arg("--toml")
            .arg("@")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("a = 1"));
    }

    #[test]
    fn csv_output() {
        let assert = jpx()
            .arg("--csv")
            .arg("@")
            .write_stdin(r#"[{"a":1,"b":2}]"#)
            .assert()
            .success();
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        // Should have a header line containing column names and a data line.
        let lines: Vec<&str> = stdout.trim().lines().collect();
        assert!(
            lines.len() >= 2,
            "CSV should have header + data rows, got: {:?}",
            lines
        );
        assert!(lines[0].contains("a") && lines[0].contains("b"));
    }

    #[test]
    fn tsv_output() {
        let assert = jpx()
            .arg("--tsv")
            .arg("@")
            .write_stdin(r#"[{"a":1,"b":2}]"#)
            .assert()
            .success();
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        assert!(stdout.contains('\t'), "TSV output should contain tabs");
        let lines: Vec<&str> = stdout.trim().lines().collect();
        assert!(lines.len() >= 2, "TSV should have header + data rows");
    }

    #[test]
    fn lines_output() {
        jpx()
            .arg("-l")
            .arg("@")
            .write_stdin("[1,2,3]")
            .assert()
            .success()
            .stdout("1\n2\n3\n");
    }

    #[test]
    fn table_output() {
        jpx()
            .arg("-t")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"[{"n":"a","v":1}]"#)
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn table_ascii_style() {
        jpx()
            .arg("-t")
            .arg("--table-style")
            .arg("ascii")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"[{"n":"a"}]"#)
            .assert()
            .success()
            .stdout(
                predicate::str::contains("+")
                    .and(predicate::str::contains("-"))
                    .and(predicate::str::contains("|")),
            );
    }

    #[test]
    fn table_markdown_style() {
        let assert = jpx()
            .arg("-t")
            .arg("--table-style")
            .arg("markdown")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"[{"n":"a"}]"#)
            .assert()
            .success();
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let lines: Vec<&str> = stdout.trim().lines().collect();
        assert!(
            lines.len() >= 2,
            "Markdown table should have at least 2 lines"
        );
        assert!(lines[0].contains('|'), "Header row should contain |");
        assert!(lines[1].contains('-'), "Separator row should contain -");
    }

    #[test]
    fn null_result_no_output() {
        // Accessing a missing key returns null, which produces no output.
        jpx()
            .arg("b")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout("");
    }
}

mod file_output {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn output_to_file() {
        let out = NamedTempFile::new().unwrap();
        let out_path = out.path().to_str().unwrap().to_string();

        jpx()
            .arg("@")
            .arg("-o")
            .arg(&out_path)
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success();

        let contents = std::fs::read_to_string(&out_path).unwrap();
        assert!(
            contents.contains("\"a\"") && contents.contains("1"),
            "Output file should contain JSON data"
        );
        // Pretty-printed output contains newlines and indentation.
        assert!(contents.contains('\n') && contents.contains("  "));
    }

    #[test]
    fn output_compact_to_file() {
        let out = NamedTempFile::new().unwrap();
        let out_path = out.path().to_str().unwrap().to_string();

        jpx()
            .arg("-c")
            .arg("@")
            .arg("-o")
            .arg(&out_path)
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success();

        let contents = std::fs::read_to_string(&out_path).unwrap();
        assert!(
            contents.trim().starts_with("{\"a\":1}"),
            "Compact output should be single-line JSON, got: {}",
            contents.trim()
        );
    }

    #[test]
    fn output_yaml_to_file() {
        let out = NamedTempFile::new().unwrap();
        let out_path = out.path().to_str().unwrap().to_string();

        jpx()
            .arg("--yaml")
            .arg("@")
            .arg("-o")
            .arg(&out_path)
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success();

        let contents = std::fs::read_to_string(&out_path).unwrap();
        assert!(
            contents.contains("a: 1"),
            "YAML file should contain 'a: 1', got: {}",
            contents
        );
    }
}

mod sort_keys {
    use super::*;

    #[test]
    fn sort_keys_alphabetical() {
        jpx()
            .arg("-S")
            .arg("-c")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"z":3,"a":1,"m":2}"#)
            .assert()
            .success()
            .stdout("{\"a\":1,\"m\":2,\"z\":3}\n");
    }

    #[test]
    fn sort_keys_nested() {
        jpx()
            .arg("--sort-keys")
            .arg("-c")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"b":{"z":1,"a":2},"a":3}"#)
            .assert()
            .success()
            .stdout("{\"a\":3,\"b\":{\"a\":2,\"z\":1}}\n");
    }

    #[test]
    fn sort_keys_preserves_array_order() {
        jpx()
            .arg("-S")
            .arg("-c")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"[3,1,2]"#)
            .assert()
            .success()
            .stdout("[3,1,2]\n");
    }

    #[test]
    fn sort_keys_with_compact() {
        jpx()
            .arg("-S")
            .arg("-c")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"c":3,"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout("{\"a\":1,\"b\":2,\"c\":3}\n");
    }

    #[test]
    fn sort_keys_streaming() {
        jpx()
            .arg("--stream")
            .arg("-S")
            .arg("@")
            .write_stdin("{\"z\":1,\"a\":2}\n{\"b\":3,\"a\":4}\n")
            .assert()
            .success()
            .stdout("{\"a\":2,\"z\":1}\n{\"a\":4,\"b\":3}\n");
    }
}

mod color_mode {
    use super::*;

    #[test]
    fn color_never_no_ansi() {
        let assert = jpx()
            .arg("--color")
            .arg("never")
            .arg("@")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success();
        let stdout = assert.get_output().stdout.clone();
        assert!(
            !stdout.contains(&b'\x1b'),
            "Output with --color never should not contain ANSI escape codes"
        );
    }

    #[test]
    fn color_always_has_ansi() {
        let assert = jpx()
            .arg("--color")
            .arg("always")
            .arg("@")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success();
        let stdout = assert.get_output().stdout.clone();
        assert!(
            stdout.contains(&b'\x1b'),
            "Output with --color always should contain ANSI escape codes"
        );
    }
}
