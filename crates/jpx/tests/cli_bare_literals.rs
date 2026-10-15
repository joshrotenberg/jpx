//! Compatibility evidence for #221; no proposed grammar is enabled here.
//! Keep the shell recipes in docs/src/design/bare-literals.md executable.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

struct Cli {
    dir: TempDir,
}

impl Cli {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        // Override inherited strict settings in both CLI and engine config.
        std::fs::write(
            dir.path().join("jpx.toml"),
            "strict = false\n[engine]\nstrict = false\n",
        )
        .unwrap();
        Self { dir }
    }

    fn isolate(&self, cmd: &mut Command) {
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("JPX_") {
                cmd.env_remove(key);
            }
        }
        cmd.env("JPX_CONFIG", self.dir.path().join("jpx.toml"))
            .current_dir(self.dir.path());
    }

    fn command(&self, strict: bool) -> Command {
        let mut cmd = assert_cmd::cargo_bin_cmd!("jpx");
        self.isolate(&mut cmd);
        cmd.args(["--color", "never", "-c"]);
        if strict {
            cmd.arg("--strict");
        }
        cmd
    }

    fn check(&self, strict: bool, expression: &str, input: &str, expected: &str) {
        // The CLI suppresses a top-level null result, but preserves nested nulls.
        let stdout = if expected == "null" {
            String::new()
        } else {
            format!("{expected}\n")
        };
        self.command(strict)
            .args(["--", expression])
            .write_stdin(input)
            .assert()
            .success()
            .stdout(stdout)
            .stderr("");
    }
}

#[test]
fn bare_words_are_fields_in_both_modes() {
    let cli = Cli::new();
    for strict in [false, true] {
        for word in ["null", "true", "false"] {
            let input = format!(r#"{{"{word}":"field"}}"#);
            cli.check(strict, word, &input, r#""field""#);
            cli.check(strict, &format!(r#""{word}""#), &input, r#""field""#);
            cli.check(strict, &format!("`{word}`"), &input, word);
            cli.check(strict, &format!("[`{word}`]"), &input, &format!("[{word}]"));
            cli.check(
                strict,
                &format!("'{word}'"),
                &input,
                &format!(r#""{word}""#),
            );
            cli.check(strict, word, "{}", "null");
            cli.check(strict, &format!("[{word}]"), "{}", "[null]");
            cli.check(
                strict,
                &format!("nested.{word}"),
                &format!(r#"{{"nested":{input}}}"#),
                r#""field""#,
            );
        }
    }
}

#[test]
fn comparison_rhs_words_are_fields_even_inside_filters() {
    let cli = Cli::new();
    for strict in [false, true] {
        for word in ["null", "true", "false"] {
            let input = format!(r#"{{"value":7,"{word}":7}}"#);
            cli.check(strict, &format!("value == {word}"), &input, "true");
            cli.check(strict, &format!("value == `{word}`"), &input, "false");

            let rows = format!(
                r#"[{{"id":"field","value":7,"{word}":7}},{{"id":"literal","value":{word},"{word}":7}}]"#
            );
            cli.check(
                strict,
                &format!("[?value == {word}].id"),
                &rows,
                r#"["field"]"#,
            );
            cli.check(
                strict,
                &format!("[?value == `{word}`].id"),
                &rows,
                r#"["literal"]"#,
            );
        }
        // Missing fields can conceal the mistake: missing == missing is true.
        cli.check(strict, "value == null", "{}", "true");
        cli.check(strict, "value == null", r#"{"value":null}"#, "true");
        cli.check(
            strict,
            "value == null",
            r#"{"value":null,"null":7}"#,
            "false",
        );
        cli.check(
            strict,
            "value == `null`",
            r#"{"value":null,"null":7}"#,
            "true",
        );
        cli.check(strict, "value == true", r#"{"value":true}"#, "false");
        cli.check(strict, "value == false", r#"{"value":false}"#, "false");
    }
}

#[test]
fn numbers_require_backticks_in_value_positions_in_both_modes() {
    let cli = Cli::new();
    for strict in [false, true] {
        for number in ["0", "42", "-42", "-0", "1.5", "-1.5", "1e3", "-1e3"] {
            let input = format!(r#"{{"value":{number}}}"#);
            for expression in [
                number.to_owned(),
                format!("value == {number}"),
                format!("[?value == {number}]"),
            ] {
                // -- bypasses clap's option parsing for leading minus signs.
                cli.command(strict)
                    .args(["--", &expression])
                    .write_stdin(input.clone())
                    .assert()
                    .code(1)
                    .stdout("")
                    .stderr(predicate::str::contains("Parse error"));
            }
            cli.check(strict, &format!("value == `{number}`"), &input, "true");
            cli.check(
                strict,
                &format!("[?value == `{number}`].value"),
                &format!("[{input}]"),
                &serde_json::from_str::<serde_json::Value>(&format!("[{number}]"))
                    .unwrap()
                    .to_string(),
            );
        }
        cli.check(
            strict,
            r#"["42", "-42"]"#,
            r#"{"42":"positive","-42":"negative"}"#,
            r#"["positive","negative"]"#,
        );
    }
}

#[test]
fn bracketed_numbers_are_indices_or_slices_not_literal_lists() {
    let cli = Cli::new();
    for strict in [false, true] {
        for (expression, expected) in [
            ("[0]", "10"),
            ("[-1]", "30"),
            ("[1:3]", "[20,30]"),
            ("[-2:]", "[20,30]"),
            ("[::-1]", "[30,20,10]"),
            ("[`0`]", "[0]"),
            ("[`-1`]", "[-1]"),
            ("`[0]`", "[0]"),
            ("`[-1]`", "[-1]"),
        ] {
            cli.check(strict, expression, "[10,20,30]", expected);
        }
    }
}

#[test]
fn leading_minus_also_needs_cli_option_disambiguation() {
    let cli = Cli::new();
    for strict in [false, true] {
        cli.command(strict)
            .args(["-n", "-42"])
            .assert()
            .code(2)
            .stdout("")
            .stderr(predicate::str::contains("unexpected argument"))
            .stderr(predicate::str::contains("-- -4"));
        cli.command(strict)
            .args(["-n", "--", "-42"])
            .assert()
            .code(1)
            .stderr(predicate::str::contains("Parse error"));
        cli.command(strict)
            .args(["-n", "`-42`"])
            .assert()
            .success()
            .stdout("-42\n")
            .stderr("");
    }
}

#[cfg(unix)]
#[test]
fn published_shell_recipes_work_in_both_modes() {
    let doc = include_str!("../../../docs/src/design/bare-literals.md");
    let recipes: Vec<_> = doc
        .split("```sh\n")
        .skip(1)
        .map(|block| block.split_once("\n```").unwrap().0)
        .collect();
    let expected = ["[null,true,false,42,-42]\n", "[1]\n", "[1]\n"];
    assert_eq!(recipes.len(), expected.len(), "update recipe expectations");

    for strict in [false, true] {
        for (recipe, expected) in recipes.iter().zip(expected) {
            let cli = Cli::new();
            let mode = if strict { "--strict" } else { "" };
            // Read the exact published snippets, and resolve jpx to this build.
            // No shell interpolation of the executable path is needed.
            let script =
                format!("jpx() {{ \"$JPX_TEST_BIN\" --color never {mode} \"$@\"; }}\n{recipe}");
            let mut shell = Command::new("/bin/sh");
            cli.isolate(&mut shell);
            shell
                .env("JPX_TEST_BIN", env!("CARGO_BIN_EXE_jpx"))
                .args(["-eu", "-c", &script])
                .assert()
                .success()
                .stdout(expected)
                .stderr("");
        }
    }
}
