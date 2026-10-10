//! Error messages are a contract: every user-facing failure must carry the
//! corrective action, not just the diagnosis.
//!
//! The principle is stated in AGENTS.md. These tests are what make it
//! load-bearing. Each case asserts the *prescriptive* part of the message, so
//! an error regressing from "here is how to fix it" to "here is what broke"
//! fails CI the same way a broken feature does.
//!
//! Asserting on exact wording would make every copy edit a test failure, so
//! each case pins the substring that carries the corrective action, and nothing
//! more. Rewording is free; dropping the fix is not.
//!
//! When you add an error path, add a case here.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

/// Writes a `.jpx` query library and returns the handle, which must stay alive
/// for the file to exist.
fn query_file(contents: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(".jpx")
        .tempfile()
        .expect("create temp .jpx");
    file.write_all(contents.as_bytes())
        .expect("write temp .jpx");
    file.flush().expect("flush temp .jpx");
    file
}

// ============================================================================
// Query library format: the failure an agent hits when guessing the syntax
// ============================================================================

mod query_library {
    use super::*;

    /// The exemplar. A file with no queries teaches the directive syntax, so a
    /// caller who guessed the format wrong recovers without reading docs.
    #[test]
    fn no_queries_teaches_the_directive_syntax() {
        let file = query_file("-- just a comment, no queries\n");
        jpx()
            .arg("-Q")
            .arg(file.path())
            .arg("--list-queries")
            .assert()
            .failure()
            .stderr(predicate::str::contains("-- :name <query-name>"));
    }

    /// Naming the available queries is what makes a typo recoverable in one
    /// step: the valid set is in the failure itself.
    #[test]
    fn unknown_query_name_lists_available_queries() {
        let file = query_file("-- :name good\n@\n");
        jpx()
            .args(["-n", "-Q"])
            .arg(file.path())
            .args(["--query", "missing"])
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("Available queries:")
                    .and(predicate::str::contains("good")),
            );
    }

    #[test]
    fn empty_query_name_shows_the_expected_form() {
        let file = query_file("-- :name \n@\n");
        jpx()
            .arg("-Q")
            .arg(file.path())
            .arg("--list-queries")
            .assert()
            .failure()
            .stderr(predicate::str::contains("-- :name my-query"));
    }

    #[test]
    fn missing_expression_says_where_to_put_it() {
        let file = query_file("-- :name noexpr\n-- :name other\n@\n");
        jpx()
            .arg("-Q")
            .arg(file.path())
            .arg("--list-queries")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("Add the expression on the line after")
                    .and(predicate::str::contains("-- :name noexpr")),
            );
    }

    #[test]
    fn duplicate_name_says_to_rename() {
        let file = query_file("-- :name dup\n@\n-- :name dup\n@\n");
        jpx()
            .arg("-Q")
            .arg(file.path())
            .arg("--list-queries")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("must be unique")
                    .and(predicate::str::contains("rename one of them")),
            );
    }
}

// ============================================================================
// Expression evaluation
// ============================================================================

mod expressions {
    use super::*;

    /// The near-miss suggestion is the whole reason a mistyped function name is
    /// a one-step recovery. `length` must be offered for `lenght`.
    #[test]
    fn unknown_function_suggests_near_misses() {
        jpx().args(["-n", "lenght(@)"]).assert().failure().stderr(
            predicate::str::contains("Did you mean").and(predicate::str::contains("length")),
        );
    }

    /// A type error has to name the types that *would* work, otherwise the
    /// caller is guessing.
    #[test]
    fn type_error_names_the_accepted_types() {
        jpx().args(["-n", "length(`1`)"]).assert().failure().stderr(
            predicate::str::contains("expected")
                .and(predicate::str::contains("array"))
                .and(predicate::str::contains("got number")),
        );
    }

    /// Stating the violated constraint is what implies the fix here.
    #[test]
    fn invalid_slice_states_the_constraint() {
        jpx()
            .args(["-n", "[::0]"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("step cannot be 0"));
    }

    /// Errors carry a caret line locating the failure in the expression. That
    /// is the positional half of the contract.
    #[test]
    fn evaluation_errors_point_at_the_offending_position() {
        jpx()
            .args(["-n", "lenght(@)"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("^"));
    }
}

// ============================================================================
// Argument conflicts
// ============================================================================

mod arguments {
    use super::*;

    /// The other exemplar from the issue: `-Q` plus a trailing positional. The
    /// message must name the workaround, not just the conflict.
    #[test]
    fn query_file_with_positional_points_at_the_workaround() {
        let file = query_file("-- :name good\n@\n");
        jpx()
            .args(["-n", "-Q"])
            .arg(file.path())
            .arg("@")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("-f/--file")
                    .or(predicate::str::contains("cannot be used with")),
            );
    }

    #[test]
    fn list_queries_without_query_file_names_the_missing_flag() {
        jpx()
            .arg("--list-queries")
            .assert()
            .failure()
            .stderr(predicate::str::contains("-Q/--query-file"));
    }

    #[test]
    fn multiple_files_name_per_file_recovery() {
        let first = query_file("{}\n");
        let second = query_file("{}\n");
        jpx()
            .arg("-f")
            .arg(first.path())
            .arg("-f")
            .arg(second.path())
            .arg("@")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Add --per-file"));
    }

    #[test]
    fn per_file_without_files_teaches_both_input_forms() {
        jpx().args(["--per-file", "@"]).assert().failure().stderr(
            predicate::str::contains("repeated -f/--file")
                .and(predicate::str::contains("trailing paths")),
        );
    }

    #[test]
    fn per_file_reserved_binding_names_the_fix() {
        let file = query_file("{}\n");
        jpx()
            .args(["--per-file", "--arg", "file", "override", "-f"])
            .arg(file.path())
            .arg("$file")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Remove '--arg file"));
    }

    #[test]
    fn per_file_binding_in_strict_mode_names_both_recoveries() {
        let file = query_file("{}\n");
        jpx()
            .args(["--per-file", "--strict", "-f"])
            .arg(file.path())
            .arg("$file")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("Remove --strict")
                    .and(predicate::str::contains("remove $file")),
            );
    }

    #[test]
    fn per_file_read_failure_names_the_file_and_recovery() {
        let good = query_file("{}\n");
        let missing = good.path().with_file_name("missing-per-file-input.json");
        jpx()
            .args(["--per-file", "-f"])
            .arg(good.path())
            .arg("-f")
            .arg(&missing)
            .arg("@")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains(missing.to_string_lossy().as_ref())
                    .and(predicate::str::contains("Check the path and JSON format")),
            );
    }

    #[test]
    fn columns_without_tabular_output_names_valid_flags() {
        jpx()
            .args(["--columns", "name,age", "@"])
            .write_stdin("{}")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("--table")
                    .and(predicate::str::contains("--csv"))
                    .and(predicate::str::contains("--tsv")),
            );
    }

    #[test]
    fn columns_reject_empty_names_with_example() {
        jpx()
            .args(["--csv", "--columns", "name,,age", "@"])
            .write_stdin("[{}]")
            .assert()
            .failure()
            .stderr(predicate::str::contains("--columns name,age"));
    }

    #[test]
    fn columns_reject_duplicates_with_recovery() {
        jpx()
            .args(["--csv", "--columns", "name,name", "@"])
            .write_stdin("[{}]")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("Duplicate column 'name'")
                    .and(predicate::str::contains("List each column once")),
            );
    }

    #[test]
    fn columns_non_object_output_names_both_recoveries() {
        jpx()
            .args(["--csv", "--columns", "name", "@"])
            .write_stdin("[1,2]")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("returns objects")
                    .and(predicate::str::contains("remove --columns")),
            );
    }

    #[test]
    fn streaming_columns_non_object_output_names_both_recoveries() {
        jpx()
            .args(["--stream", "--csv", "--columns", "name", "@"])
            .write_stdin("1\n")
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("returns objects")
                    .and(predicate::str::contains("remove --columns")),
            );
    }

    #[test]
    fn no_history_without_repl_names_both_recoveries() {
        jpx().arg("--no-history").assert().failure().stderr(
            predicate::str::contains("Add --repl")
                .and(predicate::str::contains("remove --no-history")),
        );
    }

    #[test]
    fn repl_startup_rejects_two_initial_data_sources() {
        let file = query_file("{}\n");
        jpx()
            .args(["--repl", "--demo", "users", "-f"])
            .arg(file.path())
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("Remove one")
                    .and(predicate::str::contains(".load"))
                    .and(predicate::str::contains(".demo")),
            );
    }

    #[test]
    fn malformed_repl_startup_json_names_the_fix() {
        let mut file = NamedTempFile::new().expect("create temp JSON");
        file.write_all(b"{not json}").expect("write invalid JSON");
        file.flush().expect("flush invalid JSON");
        jpx()
            .args(["--repl", "--no-history", "-f"])
            .arg(file.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("Fix the JSON syntax and retry"));
    }
}

// ============================================================================
// Input handling
// ============================================================================

mod input {
    use super::*;

    #[test]
    fn missing_input_file_is_identified_as_not_found() {
        jpx()
            .args(["-f", "definitely-not-here-9d3f.json", "@"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such")));
    }

    #[test]
    fn malformed_json_input_explains_what_to_check() {
        let mut file = NamedTempFile::new().expect("create temp json");
        file.write_all(b"{not json}").expect("write temp json");
        file.flush().expect("flush temp json");

        jpx()
            .arg("-f")
            .arg(file.path())
            .arg("@")
            .assert()
            .failure()
            .stderr(predicate::str::contains("JSON"));
    }
}
