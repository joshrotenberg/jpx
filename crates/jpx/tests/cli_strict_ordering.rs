use tempfile::TempDir;

fn check(config_strict: bool, flag_strict: bool, expected: &str) {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("jpx.toml");
    std::fs::write(
        &config,
        format!("strict = {config_strict}\n[engine]\nstrict = {config_strict}\n"),
    )
    .unwrap();
    let mut command = assert_cmd::cargo_bin_cmd!("jpx");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("JPX_") {
            command.env_remove(key);
        }
    }
    command
        .env("JPX_CONFIG", &config)
        .current_dir(dir.path())
        .args(["--color", "never", "-c"]);
    if flag_strict {
        command.arg("--strict");
    }
    command
        .arg("[left < right, left <= right, left > right, left >= right]")
        .write_stdin(r#"{"left":"a","right":"b"}"#)
        .assert()
        .success()
        .stdout(expected.to_owned())
        .stderr("");
}

#[test]
fn strict_flag_changes_string_ordering_to_null() {
    check(false, false, "[true,true,false,false]\n");
    check(false, true, "[null,null,null,null]\n");
}

#[test]
fn strict_config_uses_the_same_comparison_rules() {
    check(true, false, "[null,null,null,null]\n");
}
