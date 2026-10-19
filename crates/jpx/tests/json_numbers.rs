#[path = "support/json_numbers.rs"]
mod json_numbers;

#[test]
fn cli_json_text_is_correctly_rounded() {
    for mode in [vec![], vec!["--stream"], vec!["--slurp"]] {
        json_numbers::check_json_numbers(|input| {
            let mut command = assert_cmd::cargo_bin_cmd!("jpx");
            let output = command
                .args(&mode)
                .args(["-c", "--color", "never", "@"])
                .write_stdin(format!("{input}\n"))
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();
            let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
            if mode.contains(&"--slurp") {
                value[0].clone()
            } else {
                value
            }
        });
    }
}

#[test]
fn cli_adjacent_float_is_not_equal_to_one() {
    assert_cmd::cargo_bin_cmd!("jpx")
        .args(["-c", "{a:a,b:b,equal:a == b}"])
        .write_stdin(r#"{"a":1,"b":0.9999999999999999}"#)
        .assert()
        .success()
        .stdout("{\"a\":1,\"b\":0.9999999999999999,\"equal\":false}\n");
}
