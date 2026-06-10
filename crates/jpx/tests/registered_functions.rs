//! Reachability + output regression tests for functions that were implemented
//! and registered in code but missing from functions.toml (#190), so they were
//! silently unreachable at runtime. Each case asserts the function is now
//! callable end-to-end and produces the expected (compact) output.

use assert_cmd::Command;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

/// (expression, expected compact stdout without trailing newline)
const CASES: &[(&str, &str)] = &[
    // math
    ("acos(`1`)", "0.0"),
    ("asin(`0`)", "0.0"),
    ("atan(`0`)", "0.0"),
    ("atan2(`0`, `1`)", "0.0"),
    ("deg_to_rad(`0`)", "0.0"),
    ("rad_to_deg(`0`)", "0.0"),
    ("sign(`-5`)", "-1"),
    // array
    ("nth(`[1,2,3,4,5,6]`, `2`)", "[1,3,5]"),
    ("interleave(`[1,2,3]`, `[4,5,6]`)", "[1,4,2,5,3,6]"),
    ("rotate(`[1,2,3,4,5]`, `2`)", "[3,4,5,1,2]"),
    ("partition(`[1,2,3,4,5]`, `3`)", "[[1,2],[3,4],[5]]"),
    ("tail(`[1,2,3,4]`)", "[2,3,4]"),
    ("without(`[1,2,3,1,2]`, `[2]`)", "[1,3,1]"),
    ("xor(`[1,2,3]`, `[2,3,4]`)", "[1,4]"),
    ("combinations(`[1,2,3]`, `2`)", "[[1,2],[1,3],[2,3]]"),
    ("fill(`[1,2,3,4]`, `0`, `1`, `3`)", "[1,0,0,4]"),
    ("pull_at(`[10,20,30,40]`, `[0,2]`)", "[10,30]"),
    // string case transforms
    ("lower_case('fooBar')", "\"foobar\""),
    ("upper_case('fooBar')", "\"FOOBAR\""),
    ("start_case('hello_world')", "\"Hello World\""),
    ("title_case('hello_world')", "\"Hello World\""),
    ("train_case('hello_world')", "\"Hello-World\""),
    ("pascal_case('hello_world')", "\"HelloWorld\""),
    ("shouty_snake_case('helloWorld')", "\"HELLO_WORLD\""),
    ("shouty_kebab_case('helloWorld')", "\"HELLO-WORLD\""),
    // string utilities
    ("deburr('déjà vu')", "\"deja vu\""),
    ("escape('a < b & c')", "\"a &lt; b &amp; c\""),
    ("unescape('a &lt; b &amp; c')", "\"a < b & c\""),
    ("escape_regex('(group)')", "\"\\\\(group\\\\)\""),
    ("format('Hello {0}', 'World')", "\"Hello World\""),
    ("humanize('first_name')", "\"First name\""),
    ("words('helloWorld')", "[\"hello\",\"World\"]"),
    ("truncate('hello world', `5`)", "\"he...\""),
    // object key transforms
    (
        "pascal_keys(`{\"user_name\":\"alice\"}`)",
        "{\"UserName\":\"alice\"}",
    ),
    (
        "shouty_snake_keys(`{\"userName\":\"alice\"}`)",
        "{\"USER_NAME\":\"alice\"}",
    ),
    (
        "shouty_kebab_keys(`{\"userName\":\"alice\"}`)",
        "{\"USER-NAME\":\"alice\"}",
    ),
    (
        "train_keys(`{\"user_name\":\"bob\"}`)",
        "{\"User-Name\":\"bob\"}",
    ),
];

#[test]
fn previously_unreachable_functions_are_callable() {
    for (expr, expected) in CASES {
        jpx()
            .args(["-n", "-c"])
            .arg(expr)
            .assert()
            .success()
            .stdout(format!("{expected}\n"));
    }
}
