//! Limited compatibility probes, not a conformance suite or capability registry.
//! The fixture records spec expectations separately from observed jpx behavior.
//! Run: cargo test --offline -p jpx --test compatibility_audit

use jpx_engine::{JpxEngine, Runtime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Audit {
    schema_version: u32,
    scope: String,
    original_compliance: Value,
    community_spec: Value,
    sources: BTreeMap<String, String>,
    cases: Vec<Probe>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    id: String,
    area: String,
    coverage: String,
    expression: String,
    given: Value,
    original: Value,
    community: Value,
    note: String,
    original_fixture: Option<OriginalFixture>,
    jpx: Observation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OriginalFixture {
    file: String,
    group: usize,
    case: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    parse: Parse,
    default: Outcome,
    strict: Outcome,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parse {
    accepted: bool,
    message_contains: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum Outcome {
    Value {
        value: Value,
    },
    Error {
        error: String,
        message_contains: Vec<String>,
    },
}

fn audit() -> Audit {
    serde_json::from_str(include_str!("fixtures/compatibility_audit/cases.json"))
        .expect("compatibility audit fixture must match its schema")
}

#[test]
fn fixture_provenance_and_expectations_are_consistent() {
    let audit = audit();
    assert_eq!(audit.schema_version, 1);
    assert!(audit.scope.contains("not a conformance claim"));
    assert!(!audit.cases.is_empty());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let upstream = root.join(audit.original_compliance["path"].as_str().unwrap());
    let provenance = std::fs::read_to_string(upstream.join("UPSTREAM.md")).unwrap();
    assert!(provenance.contains(audit.original_compliance["commit"].as_str().unwrap()));
    let community_commit = audit.community_spec["commit"].as_str().unwrap();
    assert_eq!(community_commit.len(), 40);
    let mut ids = HashSet::new();
    for probe in &audit.cases {
        assert!(ids.insert(&probe.id), "duplicate probe {}", probe.id);
        assert_eq!(probe.coverage, "probe", "{}", probe.id);
        assert!(!probe.area.is_empty() && !probe.note.is_empty());
        for spec in [&probe.original, &probe.community] {
            let source = spec["source"].as_str().unwrap();
            assert!(audit.sources.contains_key(source), "{}: {source}", probe.id);
            if source.starts_with("community-") {
                assert!(audit.sources[source].contains(community_commit));
            }
            match spec["kind"].as_str().unwrap() {
                "value" => assert!(spec.get("value").is_some()),
                "error" => assert!(spec["error"].is_string()),
                other => panic!("{}: unknown spec expectation {other}", probe.id),
            }
        }
        for expected in [&probe.jpx.default, &probe.jpx.strict] {
            if let Outcome::Error {
                error,
                message_contains,
            } = expected
            {
                assert!(matches!(
                    error.as_str(),
                    "syntax"
                        | "strict-policy"
                        | "invalid-type"
                        | "invalid-arity"
                        | "unknown-function"
                        | "undefined-variable"
                ));
                assert!(!message_contains.is_empty(), "{}", probe.id);
                assert!(message_contains.iter().all(|s| !s.is_empty()));
                assert_eq!(error == "syntax", !probe.jpx.parse.accepted, "{}", probe.id);
            }
        }
        if let Some(reference) = &probe.original_fixture {
            let groups: Value = serde_json::from_str(
                &std::fs::read_to_string(upstream.join(&reference.file)).unwrap(),
            )
            .unwrap();
            let group = &groups[reference.group];
            let case = &group["cases"][reference.case];
            assert_eq!(probe.given, group["given"], "{} input drift", probe.id);
            assert_eq!(probe.expression, case["expression"], "{}", probe.id);
            match probe.original["kind"].as_str().unwrap() {
                "value" => assert_eq!(probe.original["value"], case["result"], "{}", probe.id),
                "error" => assert_eq!(probe.original["error"], case["error"], "{}", probe.id),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn function_registration_does_not_choose_a_different_parser() {
    let default = Runtime::builder()
        .with_standard()
        .with_all_extensions()
        .build();
    let strict = Runtime::strict();
    // CLI defaults enable core's let-expr transitively, even without the CLI
    // feature of that name. A feature change must not silently skip these cases.
    assert!(default.supports_let_expressions());
    assert!(strict.supports_let_expressions());
    for probe in audit().cases {
        for (label, runtime) in [("default", &default), ("strict functions", &strict)] {
            let result = runtime.compile(&probe.expression);
            assert_eq!(
                result.is_ok(),
                probe.jpx.parse.accepted,
                "{} ({label}): {result:?}",
                probe.id
            );
            if let Some(needle) = &probe.jpx.parse.message_contains {
                assert!(
                    result.unwrap_err().to_string().contains(needle),
                    "{}",
                    probe.id
                );
            }
        }
    }
}

#[test]
fn engine_default_and_strict_match_observed_behavior() {
    let default = JpxEngine::new();
    let strict = JpxEngine::strict();
    assert!(default.supports_let_expressions());
    assert!(!strict.supports_let_expressions());
    for probe in audit().cases {
        for (label, engine, expected) in [
            ("default", &default, &probe.jpx.default),
            ("strict", &strict, &probe.jpx.strict),
        ] {
            let result = engine.evaluate(&probe.expression, &probe.given);
            match expected {
                Outcome::Value { value } => {
                    assert_eq!(&result.unwrap(), value, "{} ({label})", probe.id);
                }
                Outcome::Error {
                    message_contains, ..
                } => {
                    let message = result.unwrap_err().to_string();
                    for needle in message_contains {
                        assert!(
                            message.contains(needle),
                            "{} ({label}): {message}",
                            probe.id
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn cli_default_and_strict_match_observed_behavior() {
    // Follow the agent_recipes isolation pattern. These settings affect only
    // child processes, never the caller's environment or configuration files.
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("jpx.toml");
    std::fs::write(&config, "").unwrap();
    for probe in audit().cases {
        for (label, expected) in [
            ("default", &probe.jpx.default),
            ("strict", &probe.jpx.strict),
        ] {
            let mut cmd = assert_cmd::Command::new(env!("CARGO_BIN_EXE_jpx"));
            for (key, _) in std::env::vars_os() {
                if key.to_string_lossy().starts_with("JPX_") {
                    cmd.env_remove(key);
                }
            }
            cmd.current_dir(dir.path())
                .env("HOME", dir.path())
                .env("XDG_CONFIG_HOME", dir.path().join("xdg"))
                .env("JPX_CONFIG", &config)
                .args(["--color", "never", "-c"]);
            if label == "strict" {
                cmd.arg("--strict");
            }
            let output = cmd
                .arg("--")
                .arg(&probe.expression)
                .write_stdin(serde_json::to_vec(&probe.given).unwrap())
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let context = format!("{} ({label}): {stderr}", probe.id);
            match expected {
                Outcome::Value { value } => {
                    assert_eq!(output.status.code(), Some(0), "{context}");
                    assert!(stderr.is_empty(), "{context}");
                    if value.is_null() {
                        // CLI suppresses null output; the engine test above
                        // independently asserts the actual JSON null value.
                        assert!(output.stdout.is_empty(), "{context}");
                    } else {
                        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
                        assert_eq!(&actual, value, "{context}");
                    }
                }
                Outcome::Error {
                    message_contains, ..
                } => {
                    assert_eq!(output.status.code(), Some(1), "{context}");
                    assert!(output.stdout.is_empty(), "{context}");
                    for needle in message_contains {
                        assert!(stderr.contains(needle), "{context}; missing {needle:?}");
                    }
                }
            }
        }
    }
}
