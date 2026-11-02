//! CLI regression tests for integer-valued function results.
//!
//! Integer results must print without a trailing `.0` so shell arithmetic and
//! comparisons on jpx output work. Fractional results print unchanged.

fn jpx() -> assert_cmd::Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

fn assert_stdout(expr: &str, input: &str, expected: &str) {
    jpx()
        .arg("-c")
        .arg("--color")
        .arg("never")
        .arg(expr)
        .write_stdin(input)
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}

#[test]
fn sum_of_integers_prints_without_fraction() {
    assert_stdout("sum(a)", r#"{"a":[0,4,1]}"#, "5");
}

#[test]
fn sum_with_fraction_is_unchanged() {
    assert_stdout("sum(a)", r#"{"a":[0,4,1.5]}"#, "5.5");
}

#[test]
fn ulid_timestamp_prints_without_fraction() {
    assert_stdout(
        "ulid_timestamp(@)",
        r#""01D39ZY06FGSCTVN4T2V9PKHFZ""#,
        "1549744931023",
    );
}

#[test]
fn integer_results_in_projections_print_without_fraction() {
    assert_stdout(
        "[*].{n: name, top: ceil(avg(v)), total: sum(v)}",
        r#"[{"name":"a","v":[1,2]},{"name":"b","v":[2,3]}]"#,
        r#"[{"n":"a","top":2,"total":3},{"n":"b","top":3,"total":5}]"#,
    );
}

#[test]
fn raw_output_feeds_shell_arithmetic() {
    jpx()
        .arg("-r")
        .arg("sum(@)")
        .write_stdin("[10,20,12]")
        .assert()
        .success()
        .stdout("42\n");
}
