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
