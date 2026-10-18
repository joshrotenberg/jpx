//! YAML output regressions for the serializer migration (#292).
//!
//! The dev dependency's `deserialize` feature unifies into integration-test builds;
//! production enables only serde-saphyr's `serialize` feature.

use serde_json::{Value, json};

fn jpx() -> assert_cmd::Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

fn yaml_stdout(value: &Value, flag: &str) -> String {
    let output = jpx()
        .args([flag, "@"])
        .write_stdin(serde_json::to_string(value).unwrap())
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("YAML output must be UTF-8")
}

fn assert_round_trip(yaml: &str, expected: &Value) {
    // Deserialize into an untyped JSON value, not String: a typed string target
    // can accept an unquoted YAML boolean/number and hide a quoting regression.
    let actual: Value =
        serde_saphyr::from_str(yaml).unwrap_or_else(|err| panic!("invalid YAML {yaml:?}: {err}"));
    assert_eq!(&actual, expected, "YAML: {yaml:?}");
    assert!(yaml.ends_with('\n'), "missing final newline: {yaml:?}");
}

#[test]
fn yaml_nested_json_types_and_empty_collections() {
    let value = json!({
        "nested": {
            "items": [null, true, false, 42, -17, 2.5, "42", "true", "null"],
            "empty_object": {},
            "empty_array": [],
            "empty_string": ""
        },
        "unicode": "café — 東京 🦀"
    });
    for flag in ["--yaml", "-y"] {
        let yaml = yaml_stdout(&value, flag);
        assert_round_trip(&yaml, &value);
        assert!(!yaml.ends_with("\n\n"), "extra final newline: {yaml:?}");
    }
}

#[test]
fn yaml_root_scalars_and_collections_round_trip() {
    for value in [
        json!(true),
        json!(false),
        json!(0),
        json!(-42),
        json!(i64::MIN),
        json!(u64::MAX),
        json!(9007199254740993_u64),
        json!(1.0),
        json!(-0.0),
        json!(1.25),
        json!(1e-100),
        json!(1e100),
        json!([]),
        json!({}),
        json!([null, {"name": "alice"}, [], {}]),
    ] {
        let yaml = yaml_stdout(&value, "--yaml");
        assert_round_trip(&yaml, &value);
        assert!(!yaml.ends_with("\n\n"), "extra final newline: {yaml:?}");
    }
}

#[test]
fn yaml_ambiguous_strings_and_keys_stay_strings() {
    let strings = [
        "",
        "null",
        "Null",
        "NULL",
        "~",
        "true",
        "false",
        "True",
        "FALSE",
        "yes",
        "NO",
        "on",
        "off",
        "y",
        "N",
        "0",
        "01",
        "-42",
        "+12",
        "1.0",
        "1e3",
        "0xFF",
        "0o77",
        ".nan",
        ".inf",
        "-.Inf",
        "12:34:56",
        "2026-10-12",
        "2026-10-12T12:34:56Z",
        "[]",
        "{}",
        "[a, b]",
        "{a: b}",
        "a: b",
        "a # comment",
        "# comment",
        "---",
        "...",
        "&anchor",
        "*alias",
        "!tag",
        "<<",
        "- item",
        "? key",
        "@name",
        "`value`",
        "it's quoted",
        "a \"quote\"",
        "a\\b",
        " leading",
        "trailing ",
        " ",
        "\t",
        "café — 東京 🦀",
        "\0\u{1b}",
        "a\u{85}b\u{2028}c\u{2029}d",
    ];
    for text in strings {
        let value = json!(text);
        assert_round_trip(&yaml_stdout(&value, "--yaml"), &value);
    }

    // Test the same ambiguous strings in mapping-key and nested-value positions.
    let object = Value::Object(
        strings
            .into_iter()
            .map(|text| (text.to_owned(), json!(text)))
            .collect(),
    );
    assert_round_trip(&yaml_stdout(&object, "--yaml"), &object);
}

#[test]
fn yaml_1_1_sensitive_strings_are_quoted() {
    // Inspect emitted text independently of serde-saphyr's own deserializer.
    fn assert_quoted(yaml: &str, text: &str) {
        let scalar = yaml.trim();
        assert!(
            scalar == format!("'{text}'") || scalar == format!("\"{text}\""),
            "expected quoted {text:?}, got {yaml:?}"
        );
    }

    for text in ["yes", "on", "NO", "y", "N", "12:34:56", "2026-10-12"] {
        assert_quoted(&yaml_stdout(&json!(text), "--yaml"), text);
    }

    let yaml = yaml_stdout(&json!({"yes": "on"}), "--yaml");
    let (key, value) = yaml.split_once(':').expect("expected a mapping entry");
    assert_quoted(key, "yes");
    assert_quoted(value, "on");
}

#[test]
fn yaml_preserves_multiline_content_and_trailing_whitespace() {
    for text in [
        "first\nsecond",
        "first\nsecond\n",
        "first\nsecond\n\n",
        "\n",
        "\n\n",
        "\nleading blank line\n",
        "first\n  indented\nlast  ",
        "first\nlast  \n\n",
        "first\r\nsecond\r\n",
        "first\tsecond\rthird",
        "  \n  ",
        concat!(
            "A long string with  repeated spaces that exceeds the serializer's wrapping width. ",
            "Keep every space and preserve the trailing spaces as well.  "
        ),
    ] {
        for value in [json!(text), json!([text]), json!({"last": text})] {
            assert_round_trip(&yaml_stdout(&value, "--yaml"), &value);
        }
    }
}

#[test]
fn yaml_file_matches_stdout_and_replaces_existing_content() {
    let out = tempfile::NamedTempFile::new().unwrap();
    for value in [
        json!({"items": [true, null, "true"], "unicode": "東京"}),
        json!("first\nlast  \n\n"),
        json!({"last": "first\nlast  \n\n"}),
        json!(""),
        json!([]),
    ] {
        let stdout = yaml_stdout(&value, "--yaml");
        std::fs::write(
            out.path(),
            "old content that must be completely replaced\n".repeat(20),
        )
        .unwrap();
        jpx()
            .args(["-y", "@", "--output"])
            .arg(out.path())
            .write_stdin(serde_json::to_string(&value).unwrap())
            .assert()
            .success()
            .stdout("")
            .stderr("");
        let file = std::fs::read_to_string(out.path()).unwrap();
        assert_eq!(file, stdout);
        assert_round_trip(&file, &value);
    }
}

#[test]
fn yaml_root_null_retains_cli_no_output_behavior() {
    jpx()
        .args(["--yaml", "@"])
        .write_stdin("null")
        .assert()
        .success()
        .stdout("")
        .stderr("");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("result.yaml");
    jpx()
        .args(["-y", "@", "-o"])
        .arg(&path)
        .write_stdin("null")
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert!(!path.exists(), "null should not create an output file");
}
