use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn jpx() -> Command {
    assert_cmd::cargo_bin_cmd!("jpx")
}

fn write_json(dir: &TempDir, name: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, r#"{"name":"alice","count":2}"#).expect("write JSON fixture");
    path
}

fn all_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(all_files(&path));
        } else {
            files.push(path);
        }
    }
    files
}

#[test]
fn exit_aliases_use_normal_teardown() {
    for alias in [".exit", ".quit", ".q"] {
        jpx()
            .args(["--repl", "--no-history", "--color", "never"])
            .write_stdin(format!("{alias}\n"))
            .assert()
            .success()
            .stdout(predicate::str::contains("Goodbye!"));
    }
}

#[test]
fn color_mode_controls_repl_ansi_sequences() {
    jpx()
        .args(["--repl", "--no-history", "--color", "never"])
        .write_stdin(".exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}[").not());

    jpx()
        .args(["--repl", "--no-history", "--color", "always"])
        .write_stdin(".exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}["));
}

#[test]
fn explicit_file_is_initial_data_while_stdin_remains_commands() {
    let dir = TempDir::new().expect("create temp directory");
    let path = write_json(&dir, "input.json");

    jpx()
        .args(["--repl", "--no-history", "--color", "never", "-f"])
        .arg(&path)
        .write_stdin("name\n.exit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Loaded file:")
                .and(predicate::str::contains("\"alice\""))
                .and(predicate::str::contains("Goodbye!")),
        );
}

#[test]
fn load_accepts_quoted_paths_with_spaces_and_tab_separators() {
    let dir = TempDir::new().expect("create temp directory");
    let path = write_json(&dir, "input data.json");
    let commands = format!(".load\t\"{}\"\nname\n.exit\n", path.display());

    jpx()
        .args(["--repl", "--no-history", "--color", "never"])
        .write_stdin(commands)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Loaded:")
                .and(predicate::str::contains("input data.json"))
                .and(predicate::str::contains("\"alice\"")),
        );
}

#[test]
fn multiline_queries_work_with_redirected_command_input() {
    let dir = TempDir::new().expect("create temp directory");
    let path = write_json(&dir, "input.json");

    jpx()
        .args(["--repl", "--no-history", "--color", "never", "-f"])
        .arg(&path)
        .write_stdin("name |\n@\n.exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"alice\""));
}

#[test]
fn normal_exit_persists_history_and_no_history_leaves_none() {
    let history_home = TempDir::new().expect("create history home");
    jpx()
        .args(["--repl", "--color", "never"])
        .env("HOME", history_home.path())
        .env("XDG_DATA_HOME", history_home.path())
        .write_stdin(".exit\n")
        .assert()
        .success();

    let history_files = all_files(history_home.path());
    assert!(
        history_files.iter().any(|path| {
            path.file_name().is_some_and(|name| name == "history.txt")
                && fs::read_to_string(path).is_ok_and(|text| text.contains(".exit"))
        }),
        "normal exit should save .exit in history; found {history_files:?}"
    );

    let private_home = TempDir::new().expect("create no-history home");
    jpx()
        .args(["--repl", "--no-history", "--color", "never"])
        .env("HOME", private_home.path())
        .env("XDG_DATA_HOME", private_home.path())
        .write_stdin(".exit\n")
        .assert()
        .success();
    assert!(
        all_files(private_home.path()).is_empty(),
        "--no-history must not create history files"
    );
}

#[test]
fn malformed_quoted_load_path_teaches_the_recovery() {
    jpx()
        .args(["--repl", "--no-history", "--color", "never"])
        .write_stdin(".load \"missing.json\n.exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Close the \" quote and retry"));
}
