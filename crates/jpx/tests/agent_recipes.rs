//! Executable agent recipes.
//!
//! `fixtures/agent_recipes/recipes.json` is the canonical list of CLI recipes
//! that docs and skills point agents at. Each recipe is run two ways: as a
//! direct argv with the input on stdin, and (on Unix) through `/bin/sh` with
//! POSIX-quoted arguments and the input read from a path containing spaces and
//! a single quote. If a documented recipe stops producing its exact output or
//! exit status, this fails.
//!
//! Every run is isolated from the caller's environment: `JPX_*` variables are
//! removed, `JPX_CONFIG` points at an empty file, and `HOME` and the working
//! directory are a fresh temp dir so no user or project config is discovered.

use serde::Deserialize;
use std::path::Path;
use std::process::Output;
use tempfile::TempDir;

const RECIPES: &str = include_str!("fixtures/agent_recipes/recipes.json");

/// Prepended to every recipe so output is byte-stable.
const COMMON_ARGS: [&str; 4] = ["--color", "never", "-c", "-S"];

const EMPTY_CONFIG: &str = "empty config.toml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    id: String,
    outcome: String,
    #[serde(default)]
    docs: Vec<String>,
    profile: String,
    input: String,
    args: Vec<String>,
    expected: Expected,
    #[serde(default)]
    shell_args: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    status: i32,
    stdout: String,
    #[serde(default)]
    stderr_contains: Option<String>,
}

fn recipes() -> Vec<Recipe> {
    serde_json::from_str(RECIPES).expect("recipes.json must match the recipe schema")
}

fn sandbox() -> TempDir {
    let dir = tempfile::Builder::new()
        .prefix("jpx agent recipes ")
        .tempdir()
        .expect("create sandbox dir");
    std::fs::write(dir.path().join(EMPTY_CONFIG), "").expect("write empty config");
    dir
}

/// Strips inherited `JPX_*` settings and points config discovery at the sandbox.
/// Only the child process environment is touched.
fn isolate(cmd: &mut std::process::Command, dir: &Path) {
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("JPX_") {
            cmd.env_remove(&key);
        }
    }
    cmd.env("JPX_CONFIG", dir.join(EMPTY_CONFIG))
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir.join("xdg"))
        .current_dir(dir);
}

fn check(expected: &Expected, output: &Output) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut problems = Vec::new();
    if output.status.code() != Some(expected.status) {
        problems.push(format!(
            "status: expected {}, got {:?}",
            expected.status,
            output.status.code()
        ));
    }
    if stdout != expected.stdout {
        problems.push(format!(
            "stdout: expected {:?}, got {stdout:?}",
            expected.stdout
        ));
    }
    if let Some(needle) = &expected.stderr_contains
        && !stderr.contains(needle.as_str())
    {
        problems.push(format!("stderr: expected to contain {needle:?}"));
    }
    if expected.stderr_contains.is_none() && !stderr.is_empty() {
        problems.push(format!("unexpected stderr: {stderr:?}"));
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("{}\nstderr was: {stderr}", problems.join("\n")))
    }
}

/// Current CLI defaults enable core's let-expr transitively. Require the
/// declared outcome rather than accepting either success or a parse failure.
fn verify(recipe: &Recipe, mode: &str, output: &Output) {
    if let Err(problem) = check(&recipe.expected, output) {
        panic!(
            "recipe '{}' ({mode}) args {:?}\n{problem}",
            recipe.id, recipe.args
        );
    }
}

#[test]
fn recipes_are_well_formed() {
    let recipes = recipes();
    assert!(!recipes.is_empty());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut ids = std::collections::HashSet::new();
    for recipe in &recipes {
        assert!(
            ids.insert(recipe.id.as_str()),
            "duplicate id '{}'",
            recipe.id
        );
        assert!(
            !recipe.outcome.trim().is_empty(),
            "'{}' has no outcome",
            recipe.id
        );
        assert!(!recipe.args.is_empty(), "'{}' has no args", recipe.id);
        assert!(matches!(
            recipe.profile.as_str(),
            "default" | "default-with-core-let"
        ));
        for doc in &recipe.docs {
            assert!(
                root.join(doc).is_file(),
                "'{}' references missing doc {doc}",
                recipe.id
            );
            if let Some(shell_args) = &recipe.shell_args {
                let source =
                    std::fs::read_to_string(root.join(doc)).expect("read documented recipe");
                assert!(
                    source.contains(shell_args),
                    "'{}' command drifted from {doc}",
                    recipe.id
                );
            }
        }
    }
}

#[test]
fn recipes_run_with_argv_and_stdin() {
    for recipe in recipes() {
        let dir = sandbox();
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"));
        isolate(&mut cmd, dir.path());
        cmd.args(COMMON_ARGS).args(&recipe.args);
        let output = assert_cmd::Command::from_std(cmd)
            .write_stdin(recipe.input.as_bytes())
            .output()
            .expect("run jpx");
        verify(&recipe, "argv", &output);
    }
}

#[cfg(unix)]
fn posix_quote(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', r"'\''"))
}

#[cfg(unix)]
#[test]
fn posix_quote_round_trips_through_sh() {
    let dir = sandbox();
    for arg in ["plain", "with space", "it's", "'", "''", "$x `y` \"z\"", ""] {
        let output = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("printf %s {}", posix_quote(arg)))
            .current_dir(dir.path())
            .output()
            .expect("run sh");
        assert_eq!(String::from_utf8_lossy(&output.stdout), arg);
    }
}

#[cfg(unix)]
#[test]
fn documented_shell_fragments_have_the_same_arguments() {
    for recipe in recipes() {
        let Some(shell_args) = &recipe.shell_args else {
            continue;
        };
        let dir = sandbox();
        let mut cmd = std::process::Command::new("/bin/sh");
        isolate(&mut cmd, dir.path());
        let output = cmd
            .arg("-c")
            .arg(format!("printf '%s\\0' {shell_args}"))
            .output()
            .expect("tokenize documented command");
        assert!(output.status.success(), "{}", recipe.id);
        assert!(output.stderr.is_empty(), "{}", recipe.id);
        let actual: Vec<&[u8]> = output
            .stdout
            .strip_suffix(&[0])
            .unwrap()
            .split(|b| *b == 0)
            .collect();
        let expected: Vec<&[u8]> = recipe.args.iter().map(|a| a.as_bytes()).collect();
        assert_eq!(actual, expected, "{} shell/argv drift", recipe.id);
    }
}

#[cfg(unix)]
#[test]
fn recipes_run_through_posix_shell_with_spaced_paths() {
    for recipe in recipes() {
        let dir = sandbox();
        let input_dir = dir.path().join("input files");
        std::fs::create_dir(&input_dir).expect("create input dir");
        let input_path = input_dir.join(format!("{} it's.json", recipe.id));
        std::fs::write(&input_path, &recipe.input).expect("write input");

        let mut words = vec![posix_quote(env!("CARGO_BIN_EXE_jpx"))];
        words.extend(COMMON_ARGS.iter().map(|a| posix_quote(a)));
        words.push("-f".to_string());
        words.push(posix_quote(input_path.to_str().expect("utf-8 temp path")));
        if let Some(shell_args) = &recipe.shell_args {
            // Literal command fragment also asserted present in its docs.
            words.push(shell_args.clone());
        } else {
            words.extend(recipe.args.iter().map(|a| posix_quote(a)));
        }
        let script = format!("exec {} </dev/null", words.join(" "));

        let mut cmd = std::process::Command::new("/bin/sh");
        isolate(&mut cmd, dir.path());
        let output = cmd.arg("-c").arg(&script).output().expect("run sh");
        verify(&recipe, "sh", &output);
    }
}
