//! Bound execution time so a regression to nanoid 0.4's zero-size loop fails.
use std::time::Duration;

#[test]
fn nanoid_zero_returns_empty_string() {
    for expression in ["nanoid(`0`)", "nanoid(`0.0`)"] {
        assert_cmd::cargo_bin_cmd!("jpx")
            .timeout(Duration::from_secs(5))
            .args(["-n", expression])
            .assert()
            .success()
            .stdout("\"\"\n");
    }
}

#[test]
fn nanoid_discovery_describes_size_contract() {
    let output = assert_cmd::cargo_bin_cmd!("jpx")
        .args(["--describe", "nanoid"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let description = String::from_utf8(output).unwrap();
    for text in [
        "0 through 1024",
        "21 characters",
        "size 0 returns an empty string",
    ] {
        assert!(description.contains(text), "{description}");
    }
}
