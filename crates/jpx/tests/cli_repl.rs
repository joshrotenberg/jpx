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

/// Runs a REPL session with history enabled but isolated to `home`.
///
/// Unix only: on Windows `dirs::data_local_dir()` resolves the LocalAppData
/// known folder and ignores HOME/XDG_DATA_HOME, so these sessions would touch
/// the real user history.
#[cfg(unix)]
fn run_with_history_home(home: &TempDir, commands: &str) -> String {
    let output = jpx()
        .args(["--repl", "--color", "never"])
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", home.path())
        .write_stdin(commands.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("REPL output is UTF-8")
}

/// Lines printed by `.history`: a right-aligned number, two spaces, the entry.
#[cfg(unix)]
fn history_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|line| line.rsplit("> ").next().unwrap_or(line).trim_start())
        .filter(|line| {
            line.split_once("  ").is_some_and(|(number, _)| {
                !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit())
            })
        })
        .map(str::to_string)
        .collect()
}

#[cfg(unix)]
#[test]
fn history_lists_recent_entries_in_order_without_recording_itself() {
    let home = TempDir::new().expect("create history home");
    let output = run_with_history_home(
        &home,
        ".json {\"name\":\"alice\",\"count\":2}\nname\ncount\n.history\n.history 2\n.exit\n",
    );

    assert_eq!(
        history_lines(&output),
        [
            "1  .json {\"name\":\"alice\",\"count\":2}",
            "2  name",
            "3  count",
            "2  name",
            "3  count",
        ],
        "default shows all three in order, explicit count keeps stable numbers; got:\n{output}"
    );

    let saved: String = all_files(home.path())
        .iter()
        .filter(|path| path.file_name().is_some_and(|name| name == "history.txt"))
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect();
    assert!(saved.contains(".exit"), "history should be saved: {saved}");
    assert!(
        !saved.contains(".history"),
        ".history must not be recorded: {saved}"
    );
}

#[cfg(unix)]
#[test]
fn history_default_shows_last_twenty() {
    let home = TempDir::new().expect("create history home");
    let mut commands = String::from(".json {}\n");
    for n in 0..25 {
        commands.push_str(&format!("`{n}`\n"));
    }
    commands.push_str(".history\n.exit\n");
    let lines = history_lines(&run_with_history_home(&home, &commands));

    assert_eq!(lines.len(), 20, "{lines:?}");
    assert_eq!(lines.first().map(String::as_str), Some("7  `5`"));
    assert_eq!(lines.last().map(String::as_str), Some("26  `24`"));
}

#[cfg(unix)]
#[test]
fn history_shows_previously_loaded_entries_and_joined_multiline_queries() {
    let home = TempDir::new().expect("create history home");
    // dirs::data_local_dir(): macOS uses $HOME/Library/Application Support,
    // other Unix platforms use $XDG_DATA_HOME.
    for dir in [
        home.path().join("Library/Application Support/jpx"),
        home.path().join("jpx"),
    ] {
        fs::create_dir_all(&dir).expect("create history directory");
        fs::write(dir.join("history.txt"), "#V2\nfrom_last_session\n").expect("seed history");
    }

    let output = run_with_history_home(&home, ".json {\"a\":1}\na |\n@\n.history\n.exit\n");
    assert_eq!(
        history_lines(&output),
        ["1  from_last_session", "2  .json {\"a\":1}", "3  a | @"],
        "got:\n{output}"
    );
}

#[cfg(unix)]
#[test]
fn history_numbers_survive_eviction_beyond_capacity() {
    let home = TempDir::new().expect("create history home");
    // 1 + 150 entries overflows the 100-entry REPL history.
    let mut commands = String::from(".json {}\n");
    for n in 1..=150 {
        commands.push_str(&format!("`{n}`\n"));
    }
    commands.push_str(".history 1\n`151`\n.history 2\n.history 2\n`151`\n.history 2\n.exit\n");
    let output = run_with_history_home(&home, &commands);

    assert_eq!(
        history_lines(&output),
        [
            "151  `150`",
            "151  `150`",
            "152  `151`",
            "151  `150`",
            "152  `151`",
            "151  `150`",
            "152  `151`",
        ],
        "retained entries keep their numbers across eviction, repeated display, \
         and declined duplicates; got:\n{output}"
    );
}

#[cfg(unix)]
#[test]
fn invalid_history_count_teaches_syntax_and_repl_recovers() {
    let home = TempDir::new().expect("create history home");
    for bad in ["0", "-3", "101", "two", "1 2"] {
        let output = run_with_history_home(
            &home,
            &format!(".json {{\"name\":\"alice\"}}\n.history {bad}\nname\n.exit\n"),
        );
        assert!(
            output.contains(&format!("Invalid history count '{bad}'"))
                && output.contains("Usage: .history [count]")
                && output.contains("from 1 to 100 "),
            "{bad}: {output}"
        );
        assert!(
            output.contains("\"alice\""),
            "REPL should recover: {output}"
        );
    }
}

#[test]
fn history_with_no_history_explains_how_to_enable_and_writes_nothing() {
    let home = TempDir::new().expect("create no-history home");
    jpx()
        .args(["--repl", "--no-history", "--color", "never"])
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", home.path())
        .write_stdin(".json {\"a\":1}\na\n.history\n.exit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("History is disabled")
                .and(predicate::str::contains("without --no-history"))
                .and(predicate::str::contains("1  .json").not()),
        );
    assert!(
        all_files(home.path()).is_empty(),
        "--no-history must not create history files"
    );
}

#[test]
fn help_lists_history_command() {
    jpx()
        .args(["--repl", "--no-history", "--color", "never"])
        .write_stdin(".help\n.exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(".history [n]"));
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
