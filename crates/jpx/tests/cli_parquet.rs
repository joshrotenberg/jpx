//! Parquet input must retain null-valued keys, including nested fields (#302).
#![cfg(feature = "parquet")]

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn write_parquet(input: &Value, dir: &Path) -> PathBuf {
    let source = dir.join("input.json");
    let parquet = dir.join("data.parquet");
    std::fs::write(&source, serde_json::to_vec(input).unwrap()).unwrap();
    assert_cmd::cargo_bin_cmd!("jpx")
        .args(["@", "--color", "never", "-f"])
        .arg(&source)
        .args(["--parquet", "--output"])
        .arg(&parquet)
        .assert()
        .success()
        .stderr("");
    parquet
}

fn read_query(parquet: &Path, expression: &str) -> Value {
    let output = assert_cmd::cargo_bin_cmd!("jpx")
        .args([expression, "--color", "never", "-c", "-f"])
        .arg(parquet)
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn parquet_read_preserves_null_keys_for_queries_and_json_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = json!([
        {"id":1,"optional":null,"flag":null,"score":null},
        {"id":2,"optional":"café 🦀","flag":false,"score":1.25}
    ]);
    let parquet = write_parquet(&input, dir.path());
    let output = read_query(&parquet, "@");
    assert_eq!(output, input);
    assert_eq!(output[0].get("optional"), Some(&Value::Null));
    assert_eq!(
        read_query(&parquet, "map(&contains(keys(@), 'optional'), @)"),
        json!([true, true])
    );
    assert_eq!(read_query(&parquet, "map(&length(@), @)"), json!([4, 4]));
}

#[test]
fn parquet_read_keeps_all_null_columns() {
    let dir = tempfile::tempdir().unwrap();
    let input = json!([{"id":1,"optional":null},{"id":2,"optional":null}]);
    let parquet = write_parquet(&input, dir.path());
    let output = read_query(&parquet, "@");
    assert_eq!(output, input);
    for row in output.as_array().unwrap() {
        assert_eq!(row.get("optional"), Some(&Value::Null));
    }
}

#[test]
fn parquet_read_preserves_nullable_nested_objects_and_lists() {
    let dir = tempfile::tempdir().unwrap();
    let input = json!([
        {"id":1,"record":{"name":"a","optional":null},
            "items":[{"name":"x","optional":null}]},
        {"id":2,"record":null,"items":[]},
        {"id":3,"record":{"name":null,"optional":"present"},"items":null}
    ]);
    let parquet = write_parquet(&input, dir.path());
    let output = read_query(&parquet, "@");
    assert_eq!(output, input);
    assert_eq!(output[0]["record"].get("optional"), Some(&Value::Null));
    assert_eq!(output[0]["items"][0].get("optional"), Some(&Value::Null));
    assert_eq!(output[1].get("record"), Some(&Value::Null));
}
