//! Integration tests for jpx CLI argument parsing, conflicts, and modes.

use assert_cmd::Command;
use predicates::prelude::*;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

// ============================================================================
// Help and Version
// ============================================================================

mod help_and_version {
    use super::*;

    #[test]
    fn version_shows_jpx_pattern() {
        jpx()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::is_match(r"jpx \d+\.\d+\.\d+").unwrap());
    }

    #[test]
    fn short_help_exits_success() {
        jpx()
            .arg("-h")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }

    #[test]
    fn long_help_shows_examples() {
        jpx()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("EXAMPLES:"));
    }
}

// ============================================================================
// Expression Input
// ============================================================================

mod expression_input {
    use super::*;

    #[test]
    fn positional_expression_from_stdin() {
        jpx()
            .arg("name")
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"alice\""));
    }

    #[test]
    fn flag_expression_from_stdin() {
        jpx()
            .args(["-e", "name"])
            .write_stdin(r#"{"name":"alice"}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"alice\""));
    }

    #[test]
    fn multiple_flag_expressions_chain() {
        jpx()
            .args(["-e", "[*].name", "-e", "sort(@)"])
            .write_stdin(r#"[{"name":"b"},{"name":"a"}]"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"a\"").and(predicate::str::contains("\"b\"")));
    }

    #[test]
    fn multiple_positional_expressions_chain() {
        jpx()
            .args(["[*].name", "sort(@)"])
            .write_stdin(r#"[{"name":"b"},{"name":"a"}]"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"a\"").and(predicate::str::contains("\"b\"")));
    }

    #[test]
    fn missing_expression_errors() {
        jpx()
            .write_stdin("{}")
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains("Expression required"));
    }

    #[test]
    fn flag_and_positional_conflict() {
        jpx()
            .args(["-e", "x", "y"])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }
}

// ============================================================================
// Mode Flags
// ============================================================================

mod mode_flags {
    use super::*;

    #[test]
    fn strict_blocks_extension_functions() {
        jpx()
            .args(["--strict", "-n", "upper('hello')"])
            .assert()
            .failure()
            .code(1)
            .stderr(
                predicate::str::contains("Unknown function")
                    .or(predicate::str::contains("undefined function")),
            );
    }

    #[test]
    fn strict_allows_standard_functions() {
        jpx()
            .args(["--strict", "length(@)"])
            .write_stdin("[1,2,3]")
            .assert()
            .success()
            .stdout(predicate::str::contains("3"));
    }

    #[test]
    fn quiet_suppresses_stream_parse_errors() {
        // In streaming mode, -q suppresses per-line parse errors.
        // "bad" is not valid JSON, but {"a":1} is. Quiet mode should
        // suppress the error for "bad" and still output the valid result.
        jpx()
            .args(["-q", "--stream", "a"])
            .write_stdin("bad\n{\"a\":1}")
            .assert()
            .success()
            .stdout(predicate::str::contains("1"))
            .stderr(predicate::str::is_empty());
    }

    #[test]
    fn verbose_shows_timing() {
        jpx()
            .args(["-v", "--color", "never", "@"])
            .write_stdin("{}")
            .assert()
            .success()
            .stderr(predicate::str::contains("Total time:"));
    }

    #[test]
    fn verbose_shows_input_info() {
        jpx()
            .args(["-v", "--color", "never", "@"])
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stderr(predicate::str::contains("Input:"));
    }

    #[test]
    fn verbose_strict_shows_mode() {
        jpx()
            .args(["-v", "--strict", "--color", "never", "@"])
            .write_stdin("{}")
            .assert()
            .success()
            .stderr(predicate::str::contains("Mode: strict"));
    }

    #[test]
    fn debug_shows_diagnostic_info() {
        jpx()
            .args(["--debug", "--color", "never", "-n", "@"])
            .assert()
            .success()
            .stderr(
                predicate::str::contains("jpx debug info")
                    .and(predicate::str::contains("Version"))
                    .and(predicate::str::contains("Environment"))
                    .and(predicate::str::contains("Effective settings")),
            );
    }
}

// ============================================================================
// Flag Conflicts
// ============================================================================

mod flag_conflicts {
    use super::*;

    #[test]
    fn yaml_csv_conflict() {
        jpx()
            .args(["--yaml", "--csv", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }

    #[test]
    fn yaml_toml_conflict() {
        jpx()
            .args(["--yaml", "--toml", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }

    #[test]
    fn csv_tsv_conflict() {
        jpx()
            .args(["--csv", "--tsv", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }

    #[test]
    fn lines_table_conflict() {
        jpx()
            .args(["--lines", "--table", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }

    #[test]
    fn stream_slurp_conflict() {
        jpx()
            .args(["--stream", "--slurp", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }

    #[test]
    fn stream_null_input_conflict() {
        jpx()
            .args(["--stream", "--null-input", "@"])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }

    #[test]
    fn table_style_requires_table() {
        jpx()
            .args(["--table-style", "ascii", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn types_requires_paths() {
        jpx()
            .args(["--types", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn values_requires_paths() {
        jpx()
            .args(["--values", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn expression_flag_and_query_file_conflict() {
        jpx()
            .args(["-e", "x", "-Q", "nonexistent.txt"])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("cannot be used with"));
    }
}

// ============================================================================
// Shell Completions
// ============================================================================

mod shell_completions {
    use super::*;

    #[test]
    fn bash_completions() {
        jpx()
            .args(["--completions", "bash"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn zsh_completions() {
        jpx()
            .args(["--completions", "zsh"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn fish_completions() {
        jpx()
            .args(["--completions", "fish"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }

    #[test]
    fn powershell_completions() {
        jpx()
            .args(["--completions", "powershell"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}
