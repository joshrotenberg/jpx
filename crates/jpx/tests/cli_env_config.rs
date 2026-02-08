//! Integration tests for jpx CLI environment variables and configuration file support.
//!
//! Tests cover JPX_RAW, JPX_COMPACT, JPX_STRICT, JPX_VERBOSE, JPX_QUIET env vars,
//! truthy/falsy value parsing, config file loading via JPX_CONFIG, invalid TOML
//! handling, missing config files, and exit codes.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

// ============================================================================
// Environment Variables
// ============================================================================

mod env_vars {
    use super::*;

    #[test]
    fn jpx_raw_env() {
        jpx()
            .env("JPX_RAW", "1")
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice\n");
    }

    #[test]
    fn jpx_compact_env() {
        jpx()
            .env("JPX_COMPACT", "1")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::starts_with("{\"a\":1,\"b\":2}"));
    }

    #[test]
    fn jpx_strict_env() {
        jpx()
            .env("JPX_STRICT", "1")
            .args(["-n", "upper('hello')"])
            .assert()
            .failure()
            .code(1)
            .stderr(
                predicate::str::contains("Unknown function")
                    .or(predicate::str::contains("undefined function")),
            );
    }

    #[test]
    fn jpx_verbose_env() {
        jpx()
            .env("JPX_VERBOSE", "1")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin("{}")
            .assert()
            .success()
            .stderr(predicate::str::contains("Total time:"));
    }

    #[test]
    fn jpx_quiet_env() {
        jpx()
            .env("JPX_QUIET", "1")
            .args(["--stream", "a"])
            .write_stdin("bad\n{\"a\":1}")
            .assert()
            .success()
            .stdout(predicate::str::contains("1"))
            .stderr(predicate::str::is_empty());
    }

    #[test]
    fn env_truthy_values() {
        // "true" (case-insensitive) is truthy
        jpx()
            .env("JPX_RAW", "true")
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice\n");

        // "yes" is truthy
        jpx()
            .env("JPX_RAW", "yes")
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice\n");
    }

    #[test]
    fn cli_flag_overrides_env() {
        // Both -r flag and JPX_RAW=1 set: raw output should work
        jpx()
            .env("JPX_RAW", "1")
            .arg("-r")
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice\n");
    }

    #[test]
    fn env_falsy_ignored() {
        // "0" is falsy -- raw mode should NOT activate, so output has quotes
        jpx()
            .env("JPX_RAW", "0")
            .arg("name")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("\"alice\"\n");

        // "false" is also falsy
        jpx()
            .env("JPX_RAW", "false")
            .arg("name")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("\"alice\"\n");
    }
}

// ============================================================================
// Configuration File
// ============================================================================

mod config_file {
    use super::*;

    #[test]
    fn config_via_env() {
        let mut config = NamedTempFile::new().unwrap();
        writeln!(config, "raw = true").unwrap();

        jpx()
            .env("JPX_CONFIG", config.path().as_os_str())
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout("alice\n");
    }

    #[test]
    fn config_applies_defaults() {
        let mut config = NamedTempFile::new().unwrap();
        writeln!(config, "verbose = true").unwrap();

        jpx()
            .env("JPX_CONFIG", config.path().as_os_str())
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin("{}")
            .assert()
            .success()
            .stderr(predicate::str::contains("Total time:"));
    }

    #[test]
    fn config_compact_works() {
        let mut config = NamedTempFile::new().unwrap();
        writeln!(config, "compact = true").unwrap();

        jpx()
            .env("JPX_CONFIG", config.path().as_os_str())
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::starts_with("{\"a\":1,\"b\":2}"));
    }

    #[test]
    fn config_invalid_toml() {
        let mut config = NamedTempFile::new().unwrap();
        writeln!(config, "invalid toml {{{{").unwrap();

        jpx()
            .env("JPX_CONFIG", config.path().as_os_str())
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stderr(
                predicate::str::contains("warning")
                    .and(predicate::str::contains("Failed to parse")),
            );
    }

    #[test]
    fn config_missing_file() {
        jpx()
            .env("JPX_CONFIG", "/tmp/jpx_nonexistent_config_12345.toml")
            .arg("@")
            .arg("--color")
            .arg("never")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stderr(predicate::str::contains("warning").not());
    }
}

// ============================================================================
// Exit Codes
// ============================================================================

mod exit_codes {
    use super::*;

    #[test]
    fn success_exit_zero() {
        jpx()
            .arg("a")
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .code(0);
    }

    #[test]
    fn error_exit_one() {
        jpx()
            .arg("@")
            .write_stdin("not json")
            .assert()
            .failure()
            .code(1);
    }
}
