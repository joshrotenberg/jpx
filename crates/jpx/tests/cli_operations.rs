//! Integration tests for jpx CLI operations: discovery, explain, bench,
//! stats/paths, diff/patch/merge, and query library features.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

// ============================================================================
// Discovery
// ============================================================================

mod discovery {
    use super::*;

    #[test]
    fn list_functions_shows_categories() {
        jpx().arg("--list-functions").assert().success().stdout(
            predicate::str::contains("STANDARD")
                .and(predicate::str::contains("ARRAY"))
                .and(predicate::str::contains("STRING"))
                .and(predicate::str::contains("MATH")),
        );
    }

    #[test]
    fn list_category_filters() {
        jpx()
            .args(["--list-category", "array"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains("ARRAY")
                    .and(predicate::str::contains("unique"))
                    .and(predicate::str::contains("Signature")),
            );
    }

    #[test]
    fn list_category_invalid() {
        jpx()
            .args(["--list-category", "bogus"])
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains("Unknown category"));
    }

    #[test]
    fn describe_function_detail() {
        jpx()
            .args(["--describe", "upper"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains("upper")
                    .and(predicate::str::contains("Signature"))
                    .and(predicate::str::contains("Category")),
            );
    }

    #[test]
    fn describe_unknown() {
        jpx()
            .args(["--describe", "xyznonexistent"])
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains("Unknown function"));
    }

    #[test]
    fn search_finds_matches() {
        jpx()
            .args(["--search", "hash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Found").and(predicate::str::is_empty().not()));
    }

    #[test]
    fn search_no_results() {
        jpx()
            .args(["--search", "xyzxyznonexistent"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No functions found"));
    }

    #[test]
    fn similar_finds_related() {
        jpx()
            .args(["--similar", "upper"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains("Functions similar to")
                    .and(predicate::str::contains("Same category"))
                    .and(predicate::str::contains("string")),
            );
    }
}

// ============================================================================
// Explain
// ============================================================================

mod explain {
    use super::*;

    #[test]
    fn explain_simple() {
        jpx()
            .args(["--explain", "name"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Expression:").and(predicate::str::contains("Field")));
    }

    #[test]
    fn explain_function() {
        jpx()
            .args(["--explain", "length(@)"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Function"));
    }

    #[test]
    fn explain_pipeline() {
        jpx()
            .args(["--explain", "a", "sort(@)"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains("Expression 1:")
                    .and(predicate::str::contains("Expression 2:")),
            );
    }

    #[test]
    fn explain_no_input_needed() {
        // --explain should not hang waiting for stdin; it only parses the expression.
        jpx()
            .args(["--explain", "items[*].name"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}

// ============================================================================
// Bench
// ============================================================================

mod bench {
    use super::*;

    #[test]
    fn bench_basic() {
        jpx()
            .args(["--bench=10", "@", "-n"])
            .assert()
            .success()
            .stdout(predicate::str::contains("BENCHMARK"));
    }

    #[test]
    fn bench_default_iterations() {
        // --bench without =N uses default of 100 iterations. Because --bench
        // uses num_args=0..=1, the expression must be provided via -e to avoid
        // clap consuming it as the iteration count.
        jpx()
            .args(["-n", "-e", "@", "--bench"])
            .assert()
            .success()
            .stdout(predicate::str::contains("100"));
    }

    #[test]
    fn bench_shows_stats() {
        jpx()
            .args(["--bench=10", "@", "-n"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains("Mean")
                    .and(predicate::str::contains("Median"))
                    .and(predicate::str::contains("Throughput")),
            );
    }
}

// ============================================================================
// Stats and Paths
// ============================================================================

mod stats_and_paths {
    use super::*;

    #[test]
    fn stats_array() {
        jpx()
            .args(["--stats", "--color", "never"])
            .write_stdin("[1,2,3]")
            .assert()
            .success()
            .stdout(predicate::str::contains("array"));
    }

    #[test]
    fn stats_object() {
        jpx()
            .args(["--stats", "--color", "never"])
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("object"));
    }

    #[test]
    fn paths_basic() {
        jpx()
            .args(["--paths", "--color", "never"])
            .write_stdin(r#"{"a":{"b":1}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("a").and(predicate::str::contains("b")));
    }

    #[test]
    fn paths_with_types() {
        jpx()
            .args(["--paths", "--types", "--color", "never"])
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("number"));
    }

    #[test]
    fn paths_with_values() {
        jpx()
            .args(["--paths", "--values", "--color", "never"])
            .write_stdin(r#"{"a":1}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("1"));
    }
}

// ============================================================================
// Diff / Patch / Merge
// ============================================================================

mod diff_patch_merge {
    use super::*;

    #[test]
    fn diff_generates_patch() {
        let mut source = NamedTempFile::new().unwrap();
        let mut target = NamedTempFile::new().unwrap();
        write!(source, r#"{{"name":"alice","age":30}}"#).unwrap();
        write!(target, r#"{{"name":"alice","age":31}}"#).unwrap();

        jpx()
            .arg("--diff")
            .arg(source.path())
            .arg(target.path())
            .assert()
            .success()
            .stdout(
                predicate::str::contains("op")
                    .and(predicate::str::contains("replace").or(
                        predicate::str::contains("add").or(predicate::str::contains("remove")),
                    )),
            );
    }

    #[test]
    fn diff_identical() {
        let mut source = NamedTempFile::new().unwrap();
        let mut target = NamedTempFile::new().unwrap();
        write!(source, r#"{{"name":"alice"}}"#).unwrap();
        write!(target, r#"{{"name":"alice"}}"#).unwrap();

        jpx()
            .arg("--diff")
            .arg(source.path())
            .arg(target.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("[]"))
            .stderr(predicate::str::contains("No differences"));
    }

    #[test]
    fn patch_applies() {
        let mut doc = NamedTempFile::new().unwrap();
        let mut patch = NamedTempFile::new().unwrap();
        write!(doc, r#"{{"name":"alice","age":30}}"#).unwrap();
        write!(patch, r#"[{{"op":"replace","path":"/age","value":31}}]"#).unwrap();

        jpx()
            .arg("--patch")
            .arg(patch.path())
            .arg("-f")
            .arg(doc.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("alice").and(predicate::str::contains("31")));
    }

    #[test]
    fn merge_applies() {
        let mut doc = NamedTempFile::new().unwrap();
        let mut merge = NamedTempFile::new().unwrap();
        write!(doc, r#"{{"name":"alice","age":30}}"#).unwrap();
        write!(merge, r#"{{"age":31,"city":"NYC"}}"#).unwrap();

        jpx()
            .arg("--merge")
            .arg(merge.path())
            .arg("-f")
            .arg(doc.path())
            .assert()
            .success()
            .stdout(
                predicate::str::contains("alice")
                    .and(predicate::str::contains("31"))
                    .and(predicate::str::contains("NYC")),
            );
    }
}

// ============================================================================
// Query Library
// ============================================================================

mod query_library {
    use super::*;

    #[test]
    fn query_file_plain_text() {
        let mut file = NamedTempFile::with_suffix(".txt").unwrap();
        writeln!(file, "length(@)").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .write_stdin("[1,2,3]")
            .assert()
            .success()
            .stdout(predicate::str::contains("3"));
    }

    #[test]
    fn query_file_jpx_single() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name my-query").unwrap();
        writeln!(file, "-- :desc Get length").unwrap();
        writeln!(file, "length(@)").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .write_stdin("[1,2,3]")
            .assert()
            .success()
            .stdout(predicate::str::contains("3"));
    }

    #[test]
    fn query_file_jpx_colon() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name get-len").unwrap();
        writeln!(file, "-- :desc Get length").unwrap();
        writeln!(file, "length(@)").unwrap();
        writeln!(file, "-- :name get-keys").unwrap();
        writeln!(file, "-- :desc Get keys").unwrap();
        writeln!(file, "keys(@)").unwrap();

        let path_with_query = format!("{}:get-len", file.path().display());
        jpx()
            .arg("-Q")
            .arg(&path_with_query)
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("2"));
    }

    #[test]
    fn query_file_jpx_query_flag() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name get-len").unwrap();
        writeln!(file, "length(@)").unwrap();
        writeln!(file, "-- :name get-keys").unwrap();
        writeln!(file, "keys(@)").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .args(["--query", "get-keys"])
            .write_stdin(r#"{"a":1,"b":2}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("a").and(predicate::str::contains("b")));
    }

    #[test]
    fn query_file_multiple_no_name() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name alpha").unwrap();
        writeln!(file, "length(@)").unwrap();
        writeln!(file, "-- :name beta").unwrap();
        writeln!(file, "keys(@)").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .write_stdin("{}")
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains("alpha").and(predicate::str::contains("beta")));
    }

    #[test]
    fn list_queries() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name first-query").unwrap();
        writeln!(file, "-- :desc The first query").unwrap();
        writeln!(file, "length(@)").unwrap();
        writeln!(file, "-- :name second-query").unwrap();
        writeln!(file, "-- :desc The second query").unwrap();
        writeln!(file, "keys(@)").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .arg("--list-queries")
            .assert()
            .success()
            .stdout(
                predicate::str::contains("first-query")
                    .and(predicate::str::contains("second-query")),
            );
    }

    #[test]
    fn check_valid() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name valid-one").unwrap();
        writeln!(file, "length(@)").unwrap();
        writeln!(file, "-- :name valid-two").unwrap();
        writeln!(file, "keys(@)").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .args(["--check", "--color", "never"])
            .assert()
            .success()
            .stdout(predicate::str::contains("All queries valid"));
    }

    #[test]
    fn check_invalid() {
        let mut file = NamedTempFile::with_suffix(".jpx").unwrap();
        writeln!(file, "-- :name bad-query").unwrap();
        writeln!(file, "invalid[[[").unwrap();

        jpx()
            .arg("-Q")
            .arg(file.path())
            .args(["--check", "--color", "never"])
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::contains("Validation failed"));
    }
}
