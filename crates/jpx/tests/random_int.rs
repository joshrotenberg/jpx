//! Integer bounds remain exact through CLI input, evaluation, and output.

#[test]
fn random_int_preserves_large_singleton_bounds() {
    for bound in [
        "-9223372036854775808",
        "-9007199254740993",
        "9007199254740993",
        "9223372036854775807",
    ] {
        assert_cmd::cargo_bin_cmd!("jpx")
            .args(["-n", "-c", &format!("random_int(`{bound}`, `{bound}`)")])
            .assert()
            .success()
            .stdout(format!("{bound}\n"));
        assert_cmd::cargo_bin_cmd!("jpx")
            .args(["-c", "random_int(min, max)"])
            .write_stdin(format!(r#"{{"min":{bound},"max":{bound}}}"#))
            .assert()
            .success()
            .stdout(format!("{bound}\n"));
    }
}
