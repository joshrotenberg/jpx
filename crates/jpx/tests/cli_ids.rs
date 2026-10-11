//! ID generation behavior through the CLI.

use std::time::Duration;

#[test]
fn zero_length_nanoid_returns_empty_string_without_hanging() {
    // nanoid 0.4 repeatedly requested an empty random-byte buffer for size zero.
    // Keep this regression in a subprocess so a dependency regression fails
    // promptly rather than hanging the test runner.
    assert_cmd::cargo_bin_cmd!("jpx")
        .args(["-n", "nanoid(`0`)"])
        .timeout(Duration::from_secs(5))
        .assert()
        .success()
        .stdout("\"\"\n");
}
