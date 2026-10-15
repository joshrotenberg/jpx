//! The skills interface must work using only the installed binary.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::io::Read;
use std::process::{Command as ProcessCommand, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const INDEX: &str = include_str!("../skills/index.md");
const TOPIC: &str = include_str!("../skills/jpx-query-libraries/SKILL.md");

fn jpx() -> Command {
    let mut command = assert_cmd::cargo_bin_cmd!("jpx");
    command.env_clear().timeout(Duration::from_secs(10));
    command
}

fn expected_document(topic: bool) -> Value {
    if topic {
        json!({ "topic": "query-libraries", "content": TOPIC })
    } else {
        json!({ "topics": ["query-libraries"], "content": INDEX })
    }
}

#[test]
fn index_is_exact_small_and_lists_only_shipped_topics() {
    let output = jpx()
        .arg("--skills")
        .assert()
        .success()
        .get_output()
        .clone();
    assert_eq!(output.stdout, INDEX.as_bytes());
    assert!(output.stderr.is_empty());
    assert!(output.stdout.len() <= 2500, "index exceeds 2.5KB");
    for convention in [
        "-- :name",
        "-Q library.jpx:query-name -f data.json",
        "backtick",
    ] {
        assert!(INDEX.contains(convention), "missing {convention}");
    }

    let output = jpx()
        .args(["--skills", "--json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["topics"], json!(["query-libraries"]));
    for topic in document["topics"].as_array().unwrap() {
        jpx()
            .args(["--skills", topic.as_str().unwrap()])
            .assert()
            .success()
            .stdout(TOPIC);
    }
}

#[test]
fn topic_is_exact_installable_skill_with_valid_frontmatter() {
    jpx()
        .args(["--skills", "query-libraries"])
        .assert()
        .success()
        .stderr("")
        .stdout(TOPIC);
    let metadata = validate_skill_structure(TOPIC);
    assert_eq!(metadata["name"].as_str().unwrap(), "jpx-query-libraries");
}

/// Structural checks only; the embedded bytes are compared exactly elsewhere.
/// CRLF is normalized so a `core.autocrlf` checkout of the asset still parses.
fn validate_skill_structure(markdown: &str) -> serde_json::Value {
    let markdown = markdown.replace("\r\n", "\n");
    let (yaml, body) = markdown
        .strip_prefix("---\n")
        .unwrap()
        .split_once("\n---\n")
        .unwrap();
    let metadata: serde_json::Value = serde_saphyr::from_str(yaml).unwrap();
    let name = metadata["name"].as_str().unwrap();
    assert!(name.len() <= 64);
    assert!(name.bytes().all(|c| c.is_ascii_lowercase() || c == b'-'));
    assert!(!metadata["description"].as_str().unwrap().is_empty());
    assert!(body.contains("```sh\n"));
    metadata
}

#[test]
fn skill_structure_validation_accepts_lf_and_crlf() {
    let lf = TOPIC.replace("\r\n", "\n");
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(
        validate_skill_structure(&lf),
        validate_skill_structure(&crlf)
    );
}

#[test]
fn json_and_compact_forms_preserve_exact_markdown() {
    for topic in [false, true] {
        for format in [
            vec!["--json"],
            vec!["-c"],
            vec!["--compact"],
            vec!["--json", "-c"],
        ] {
            let mut command = jpx();
            command.arg("--skills");
            if topic {
                command.arg("query-libraries");
            }
            let output = command
                .args(&format)
                .assert()
                .success()
                .stderr("")
                .get_output()
                .clone();
            let expected = expected_document(topic);
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                expected
            );
            let text = if format == ["--json"] {
                serde_json::to_string_pretty(&expected).unwrap()
            } else {
                serde_json::to_string(&expected).unwrap()
            };
            assert_eq!(output.stdout, format!("{text}\n").as_bytes());
        }
    }
}

#[test]
fn ambient_defaults_do_not_select_a_skills_format_or_load_config() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    // Test valid defaults as well as a file that would warn if loaded.
    for contents in [
        "compact = true\nraw = true\nverbose = true\ncolor = 'always'\n",
        "this is not valid TOML [",
    ] {
        fs::write(&config, contents).unwrap();
        for topic in [false, true] {
            for format in [None, Some("--json"), Some("-c")] {
                let mut command = jpx();
                command
                    .current_dir(directory.path())
                    .env("JPX_CONFIG", &config)
                    .env("JPX_COMPACT", "1")
                    .env("JPX_RAW", "1")
                    .env("JPX_VERBOSE", "1")
                    .env("JPX_QUIET", "1")
                    .env("JPX_STRICT", "1")
                    .arg("--skills");
                if topic {
                    command.arg("query-libraries");
                }
                let expected = match format {
                    None => if topic { TOPIC } else { INDEX }.to_owned(),
                    Some("--json") => format!(
                        "{}\n",
                        serde_json::to_string_pretty(&expected_document(topic)).unwrap()
                    ),
                    Some(_) => format!(
                        "{}\n",
                        serde_json::to_string(&expected_document(topic)).unwrap()
                    ),
                };
                if let Some(format) = format {
                    command.arg(format);
                }
                command.assert().success().stderr("").stdout(expected);
            }
        }
    }
}

/// Keep stdin open and empty until the child exits. Reading stdin would hang;
/// drain output concurrently so the test also works for larger future assets.
fn output_without_closing_stdin(command: &mut ProcessCommand) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            drop(stdin);
            out.join().unwrap();
            err.join().unwrap();
            panic!("skills blocked on stdin or configuration");
        }
        thread::sleep(Duration::from_millis(10));
    };
    drop(stdin);
    Output {
        status,
        stdout: out.join().unwrap(),
        stderr: err.join().unwrap(),
    }
}

#[test]
fn copied_binary_works_offline_in_empty_directory_without_reading_stdin() {
    let binary_dir = tempfile::tempdir().unwrap();
    let working_dir = tempfile::tempdir().unwrap();
    let copied_binary = binary_dir
        .path()
        .join(format!("jpx{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_jpx"), &copied_binary).unwrap();
    for topic in [false, true] {
        for format in [None, Some("--json"), Some("-c")] {
            let mut command = ProcessCommand::new(&copied_binary);
            command
                .env_clear()
                .current_dir(working_dir.path())
                .arg("--skills");
            if topic {
                command.arg("query-libraries");
            }
            if let Some(format) = format {
                command.arg(format);
            }
            let output = output_without_closing_stdin(&mut command);
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty());
            if format.is_some() {
                assert_eq!(
                    serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                    expected_document(topic)
                );
            } else {
                assert_eq!(output.stdout, if topic { TOPIC } else { INDEX }.as_bytes());
            }
        }
    }
    assert_eq!(fs::read_dir(working_dir.path()).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn skills_do_not_open_cli_or_engine_config() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let directory = tempfile::tempdir().unwrap();
    let cli_config = directory.path().join("config.toml");
    let engine_config = directory.path().join("jpx.toml");
    // Opening either FIFO for reading would block: neither has a writer.
    for path in [&cli_config, &engine_config] {
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        // Safety: path is NUL-terminated and lives through the call.
        assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    }
    for args in [
        vec!["--skills"],
        vec!["--skills", "query-libraries", "--json"],
    ] {
        let output = output_without_closing_stdin(
            ProcessCommand::new(env!("CARGO_BIN_EXE_jpx"))
                .env_clear()
                .env("JPX_CONFIG", &cli_config)
                .current_dir(directory.path())
                .args(args),
        );
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn skills_reject_execution_inputs_and_unsupported_formats() {
    let conflicts: &[&[&str]] = &[
        &["@"],
        &["-e", "@"],
        &["-Q", "missing.jpx"],
        &["--query", "name"],
        &["-f", "missing.json"],
        &["--list-queries"],
        &["--check"],
        &["--arg", "key", "value"],
        &["--argjson", "key", "1"],
        &["--per-file"],
        &["-n"],
        &["-s"],
        &["-R"],
        &["--stream"],
        &["--each"],
        &["--unbuffered"],
        &["--repl"],
        &["--demo", "users"],
        &["--no-history"],
        &["--diff", "a", "b"],
        &["--patch", "a"],
        &["--merge", "a"],
        &["--stats"],
        &["--paths"],
        &["--types"],
        &["--values"],
        &["--bench"],
        &["--warmup", "1"],
        &["--explain"],
        &["--debug"],
        &["--list-functions"],
        &["--cheatsheet"],
        &["--list-category", "array"],
        &["--search", "query"],
        &["--describe", "length"],
        &["--similar", "length"],
        &["--completions", "bash"],
        &["-o", "ignored.json"],
        &["-r"],
        &["-j"],
        &["--indent", "4"],
        &["--tab"],
        &["-S"],
        &["-y"],
        &["--toml"],
        &["--csv"],
        &["--tsv"],
        &["-l"],
        &["-t"],
        &["--columns", "name"],
        &["--table-style", "ascii"],
        &["--color", "always"],
        &["-x"],
        &["-q"],
        &["-v"],
        &["--strict"],
    ];
    for topic in [None, Some("query-libraries")] {
        for conflict in conflicts {
            let mut command = jpx();
            if let Some(topic) = topic {
                command.args(["--skills", topic]);
            } else {
                // End the optional value explicitly; @ must be a positional.
                command.arg("--skills=");
            }
            command
                .args(*conflict)
                .assert()
                .failure()
                .code(2)
                .stdout("")
                .stderr(
                    predicate::str::contains("cannot be used with")
                        .and(predicate::str::contains("--skills")),
                );
        }
    }
    #[cfg(feature = "parquet")]
    jpx()
        .args(["--skills", "--parquet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn unknown_topics_and_json_without_skills_fail_with_guidance() {
    for topic in ["missing", "search", "../query-libraries", "Query-Libraries"] {
        jpx()
            .args(["--skills", topic])
            .assert()
            .failure()
            .stdout("")
            .stderr(predicate::str::contains("Unknown skills topic").and(
                predicate::str::contains("Use --skills to list available topics"),
            ));
    }
    jpx()
        .args(["--json", "-n", "@"])
        .assert()
        .failure()
        .stdout("")
        .stderr(
            predicate::str::contains("remove --json").and(predicate::str::contains(
                "ordinary queries already output JSON by default",
            )),
        );
}

#[test]
fn both_help_forms_point_agents_to_skills() {
    for help in ["-h", "--help"] {
        jpx().arg(help).assert().success().stdout(
            predicate::str::contains("Agents: jpx --skills")
                .and(predicate::str::contains("--skills query-libraries")),
        );
    }
}

#[test]
fn ordinary_queries_keep_their_output_and_ambient_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    fs::write(&config, "").unwrap();
    let input = r#"{"users":[{"name":"Ada"},{"name":"Lin"}]}"#;
    for compact in [false, true] {
        let mut command = jpx();
        command
            .current_dir(directory.path())
            .env("JPX_CONFIG", &config);
        if compact {
            command.arg("-c");
        }
        command
            .arg("users[*].name")
            .write_stdin(input)
            .assert()
            .success()
            .stdout(if compact {
                "[\"Ada\",\"Lin\"]\n"
            } else {
                "[\n  \"Ada\",\n  \"Lin\"\n]\n"
            });
    }
    for env_compact in [false, true] {
        fs::write(&config, if env_compact { "" } else { "compact = true\n" }).unwrap();
        jpx()
            .current_dir(directory.path())
            .env("JPX_CONFIG", &config)
            .env("JPX_COMPACT", if env_compact { "1" } else { "0" })
            .arg("users[*].name")
            .write_stdin(input)
            .assert()
            .success()
            .stdout("[\"Ada\",\"Lin\"]\n");
    }
}

/// Collect every fenced shell example, failing on unclosed/unknown fences so
/// newly added examples cannot silently escape execution.
#[cfg(unix)]
fn shell_examples(markdown: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let mut block: Option<String> = None;
    for line in markdown.lines() {
        if line == "```" {
            examples.push(block.take().expect("closing fence without opening fence"));
        } else if line.starts_with("```") {
            assert!(matches!(line, "```sh" | "```bash" | "```shell"));
            assert!(block.replace(String::new()).is_none());
        } else if let Some(block) = &mut block {
            block.push_str(line);
            block.push('\n');
        }
    }
    assert!(block.is_none(), "unclosed shell example");
    examples
}

#[cfg(unix)]
#[test]
fn every_embedded_shell_example_runs_against_the_built_binary() {
    let binary_dir = tempfile::tempdir().unwrap();
    fs::copy(env!("CARGO_BIN_EXE_jpx"), binary_dir.path().join("jpx")).unwrap();
    let path = std::env::join_paths([
        binary_dir.path(),
        std::path::Path::new("/usr/bin"),
        std::path::Path::new("/bin"),
    ])
    .unwrap();
    let examples: Vec<_> = [INDEX, TOPIC]
        .into_iter()
        .flat_map(shell_examples)
        .collect();
    assert_eq!(
        examples.len(),
        2,
        "add verified results for each new shell example"
    );
    for (index, example) in examples.iter().enumerate() {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("config.toml");
        fs::write(&config, "").unwrap();
        let output = Command::new("/bin/sh")
            .env_clear()
            .env("PATH", &path)
            .env("JPX_CONFIG", &config)
            .env("NO_COLOR", "1")
            .current_dir(directory.path())
            .args(["-eu", "-c", example])
            .timeout(Duration::from_secs(10))
            .assert()
            .success()
            .stderr("")
            .get_output()
            .clone();
        match index {
            0 => {
                let stdout = String::from_utf8(output.stdout).unwrap();
                assert!(stdout.contains("All queries valid."));
                let results: Vec<Value> = stdout
                    .lines()
                    .filter(|line| line.starts_with('['))
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                assert_eq!(
                    results,
                    vec![json!(["Ada"]), json!(["Ada", "Lin"]), json!(["Ada"])]
                );
                assert!(directory.path().join("users.jpx").exists());
                assert!(directory.path().join("users.json").exists());
            }
            1 => {
                assert!(output.stdout.is_empty());
                assert_eq!(
                    fs::read_to_string(
                        directory.path().join("skills/jpx-query-libraries/SKILL.md")
                    )
                    .unwrap(),
                    TOPIC
                );
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn skills_index_snapshots() {
    let plain = jpx().arg("--skills").output().unwrap();
    assert!(plain.status.success());
    insta::assert_snapshot!("skills_index", String::from_utf8(plain.stdout).unwrap());

    let json = jpx().args(["--skills", "--json"]).output().unwrap();
    assert!(json.status.success());
    insta::assert_snapshot!("skills_index_json", String::from_utf8(json.stdout).unwrap());
}
