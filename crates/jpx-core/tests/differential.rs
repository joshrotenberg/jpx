//! Bounded differential campaigns. See differential/README.md for controls and replay.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use jpx_core::Runtime;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value, json};

mod support;

const DEFAULT_SEEDS: &[u64] = &[249, 99537481];
const MAX_CASES: usize = 10_000;
const MAX_SEEDS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ErrorClass {
    Syntax,
    InvalidValue,
    InvalidArity,
    UnknownFunction,
    InvalidType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
enum Outcome {
    Json(Value),
    Error(ErrorClass),
    // These are never agreement, even when both engines fail identically.
    Panic(String),
    Unexpected(String),
    ConversionFailure(String),
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Compile,
    Search,
}

fn jpx_error(reason: jpx_core::ErrorReason, phase: Phase) -> Outcome {
    use jpx_core::{ErrorReason as E, RuntimeError as R};
    let class = match reason {
        E::Parse(_) if matches!(phase, Phase::Compile) => ErrorClass::Syntax,
        E::Parse(message) => {
            return Outcome::Unexpected(format!("parse error during search: {message}"));
        }
        E::Runtime(R::InvalidSlice) => ErrorClass::InvalidValue,
        E::Runtime(R::TooManyArguments { .. } | R::NotEnoughArguments { .. }) => {
            ErrorClass::InvalidArity
        }
        E::Runtime(R::UnknownFunction(_)) => ErrorClass::UnknownFunction,
        E::Runtime(R::InvalidType { .. } | R::InvalidReturnType { .. }) => ErrorClass::InvalidType,
        other => return Outcome::Unexpected(format!("unmapped jpx error: {other:?}")),
    };
    Outcome::Error(class)
}

fn reference_error(reason: jmespath::ErrorReason, phase: Phase) -> Outcome {
    use jmespath::{ErrorReason as E, RuntimeError as R};
    // Exhaustive typed mapping: a new reference variant must be reviewed at
    // compile time. jpx's non_exhaustive variants fail closed above.
    Outcome::Error(match reason {
        E::Parse(_) if matches!(phase, Phase::Compile) => ErrorClass::Syntax,
        E::Parse(message) => {
            return Outcome::Unexpected(format!("parse error during search: {message}"));
        }
        E::Runtime(R::InvalidSlice) => ErrorClass::InvalidValue,
        E::Runtime(R::TooManyArguments { .. } | R::NotEnoughArguments { .. }) => {
            ErrorClass::InvalidArity
        }
        E::Runtime(R::UnknownFunction(_)) => ErrorClass::UnknownFunction,
        E::Runtime(R::InvalidType { .. } | R::InvalidReturnType { .. }) => ErrorClass::InvalidType,
    })
}

fn guarded(f: impl FnOnce() -> Outcome) -> Outcome {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(outcome) => outcome,
        Err(payload) => Outcome::Panic(
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                .unwrap_or_else(|| "non-string panic payload".to_owned()),
        ),
    }
}

fn reference_json(variable: &jmespath::Variable) -> Outcome {
    fn has_expref(variable: &jmespath::Variable) -> bool {
        match variable {
            jmespath::Variable::Expref(_) => true,
            jmespath::Variable::Array(values) => values.iter().any(|v| has_expref(v)),
            jmespath::Variable::Object(values) => values.values().any(|v| has_expref(v)),
            _ => false,
        }
    }
    // Variable's serializer renders expression references as strings. Those
    // are not JSON results; don't accidentally bless that coercion.
    if has_expref(variable) {
        return Outcome::Unexpected("reference returned a non-JSON expression reference".into());
    }
    match serde_json::to_value(variable) {
        Ok(value) => Outcome::Json(value),
        Err(error) => Outcome::ConversionFailure(format!("serialize reference result: {error}")),
    }
}

fn evaluate(expression: &str, document: &Value) -> (Outcome, Outcome) {
    let jpx = guarded(|| {
        let runtime = Runtime::strict();
        let compiled = match runtime.compile(expression) {
            Ok(compiled) => compiled,
            Err(error) => return jpx_error(error.reason, Phase::Compile),
        };
        match compiled.search(document) {
            Ok(value) => Outcome::Json(value),
            Err(error) => jpx_error(error.reason, Phase::Search),
        }
    });
    let reference = guarded(|| {
        // Avoid Variable::from_json and serde's generic document round trip:
        // TryFrom clones JSON Numbers without rounding large integers.
        let input = match jmespath::Variable::try_from(document) {
            Ok(input) => input,
            Err(error) => return Outcome::ConversionFailure(format!("reference input: {error:?}")),
        };
        let compiled = match jmespath::compile(expression) {
            Ok(compiled) => compiled,
            Err(error) => return reference_error(error.reason, Phase::Compile),
        };
        match compiled.search(input) {
            Ok(value) => reference_json(&value),
            Err(error) => reference_error(error.reason, Phase::Search),
        }
    });
    (jpx, reference)
}

fn integer(number: &Number) -> Option<i128> {
    number
        .as_i64()
        .map(i128::from)
        .or_else(|| number.as_u64().map(i128::from))
}

fn integer_float_equal(integer: i128, float: f64) -> bool {
    // All serde_json integers are in [-2^63, 2^64). Cast the integral float
    // into a wider integer, never the integer into f64 (nor a saturating u64).
    float.fract() == 0.0
        && (-9_223_372_036_854_775_808.0..18_446_744_073_709_551_616.0).contains(&float)
        && float as i128 == integer
}

fn numbers_equal(a: &Number, b: &Number) -> bool {
    match (integer(a), integer(b)) {
        (Some(a), Some(b)) => a == b,
        (Some(a), None) => b.as_f64().is_some_and(|b| integer_float_equal(a, b)),
        (None, Some(b)) => a.as_f64().is_some_and(|a| integer_float_equal(b, a)),
        (None, None) => a.as_f64().zip(b.as_f64()).is_some_and(|(a, b)| a == b),
    }
}

fn json_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => numbers_equal(a, b),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| json_equal(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| json_equal(a, b)))
        }
        _ => a == b,
    }
}

fn agrees(a: &Outcome, b: &Outcome) -> bool {
    match (a, b) {
        (Outcome::Json(a), Outcome::Json(b)) => json_equal(a, b),
        (Outcome::Error(a), Outcome::Error(b)) => a == b,
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Population {
    Grammar,
    NearValid,
    Compliance,
}

impl Population {
    fn name(self) -> &'static str {
        match self {
            Self::Grammar => "grammar",
            Self::NearValid => "near-valid",
            Self::Compliance => "compliance",
        }
    }
    fn tag(self) -> u8 {
        match self {
            Self::Grammar => 1,
            Self::NearValid => 2,
            Self::Compliance => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Repro {
    version: u32,
    population: Population,
    seed: u64,
    case_index: usize,
    source: Option<String>,
    expression: String,
    document: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    #[serde(flatten)]
    repro: Repro,
    jpx: Outcome,
    reference: Outcome,
    agreement: bool,
    allowlist_reason: Option<String>,
}

// Deliberately empty. Any future exception must match this exact input AND
// both normalized outcomes, with a reviewed reason/issue. No regexes, error-
// class-wide exemptions, or exceptions for panics/unexpected/conversion errors.
// Reviewed reference-side failures use REFERENCE_FAILURES below instead.
struct AllowedDivergence {
    expression: &'static str,
    document_json: &'static str,
    jpx: Outcome,
    reference: Outcome,
    reason: &'static str,
}
const ALLOWLIST: &[AllowedDivergence] = &[];

// A reviewed failure of the reference engine itself. The reference outcome
// must be `Unexpected` with exactly `reference_failure` as its message, the
// expression must match exactly, and jpx must produce exactly `jpx`, which can
// only be a JSON result or a typed error. The document is not part of the key:
// the expression plus both exact outcomes pin the reference defect. jpx panics,
// jpx unexpected errors, and conversion failures on either side are never
// covered, and a blank reason never matches.
struct ReferenceFailure {
    expression: &'static str,
    jpx: Outcome,
    reference_failure: &'static str,
    reason: &'static str,
}
const AVG_EMPTY_REFERENCE_FAILURE: &str = "parse error during search: Expected to be a valid f64";
const REFERENCE_FAILURES: &[ReferenceFailure] = &[ReferenceFailure {
    expression: "avg(empty_list)",
    jpx: Outcome::Json(Value::Null),
    reference_failure: AVG_EMPTY_REFERENCE_FAILURE,
    reason: "Reference bug in the jmespath crate (0.3 through 0.5): avg([]) computes 0/0 and \
             fails converting NaN to a JSON number, reported as a Parse error during search. \
             The specification and vendored functions.json require null. \
             See differential/README.md and issue #249.",
}];

fn reference_failure_reason(
    repro: &Repro,
    jpx: &Outcome,
    reference: &Outcome,
    entries: &[ReferenceFailure],
) -> Option<String> {
    let Outcome::Unexpected(failure) = reference else {
        return None;
    };
    if !matches!(jpx, Outcome::Json(_) | Outcome::Error(_)) {
        return None;
    }
    entries
        .iter()
        .find(|entry| {
            !entry.reason.trim().is_empty()
                && entry.expression == repro.expression
                && entry.reference_failure == failure
                && matches!(entry.jpx, Outcome::Json(_) | Outcome::Error(_))
                && &entry.jpx == jpx
        })
        .map(|entry| entry.reason.to_owned())
}

fn allowed_reason(
    repro: &Repro,
    jpx: &Outcome,
    reference: &Outcome,
    entries: &[AllowedDivergence],
) -> Option<String> {
    if !matches!(jpx, Outcome::Json(_) | Outcome::Error(_))
        || !matches!(reference, Outcome::Json(_) | Outcome::Error(_))
    {
        return None;
    }
    entries
        .iter()
        .find(|entry| {
            !entry.reason.trim().is_empty()
                && entry.expression == repro.expression
                && serde_json::from_str::<Value>(entry.document_json)
                    .ok()
                    .as_ref()
                    == Some(&repro.document)
                && &entry.jpx == jpx
                && &entry.reference == reference
        })
        .map(|entry| entry.reason.to_owned())
}

fn record(repro: Repro) -> Record {
    let (jpx, reference) = evaluate(&repro.expression, &repro.document);
    Record {
        agreement: agrees(&jpx, &reference),
        allowlist_reason: allowed_reason(&repro, &jpx, &reference, ALLOWLIST)
            .or_else(|| reference_failure_reason(&repro, &jpx, &reference, REFERENCE_FAILURES)),
        repro,
        jpx,
        reference,
    }
}

fn require_agreement(record: &Record) -> Result<(), String> {
    if record.agreement || record.allowlist_reason.is_some() {
        Ok(())
    } else {
        Err(format!(
            "Differential divergence. Replay this full record with JPX_DIFF_REPLAY (see differential/README.md):\n{}",
            serde_json::to_string_pretty(record).expect("serialize divergence")
        ))
    }
}

fn runner(seed: u64, population: Population) -> TestRunner {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    bytes[8] = population.tag();
    TestRunner::new_with_rng(
        Config {
            failure_persistence: None,
            max_shrink_iters: 0,
            max_local_rejects: 1024,
            max_global_rejects: 1024,
            ..Config::default()
        },
        TestRng::from_seed(RngAlgorithm::ChaCha, &bytes),
    )
}

fn generated_strategy(population: Population) -> BoxedStrategy<(String, Value)> {
    let expression = match population {
        Population::Grammar => support::standard_grammar_expression(),
        Population::NearValid => support::near_valid_expression(),
        Population::Compliance => unreachable!("use real compliance pairs"),
    };
    (expression, support::json_strategy::differential_document()).boxed()
}

fn compliance_pairs() -> Vec<(String, String, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/compliance");
    let mut paths = fs::read_dir(&root)
        .expect("read compliance seeds")
        .map(|entry| entry.expect("read compliance entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    let mut pairs = Vec::new();
    for path in paths {
        let groups: Value = serde_json::from_slice(&fs::read(&path).expect("read compliance file"))
            .expect("parse compliance file");
        for (group_index, group) in groups
            .as_array()
            .expect("compliance groups")
            .iter()
            .enumerate()
        {
            for (case_index, case) in group["cases"]
                .as_array()
                .expect("compliance cases")
                .iter()
                .enumerate()
            {
                pairs.push((
                    format!(
                        "{}:{group_index}:{case_index}",
                        path.file_name().unwrap().to_string_lossy()
                    ),
                    case["expression"]
                        .as_str()
                        .expect("compliance expression")
                        .to_owned(),
                    group.get("given").expect("compliance document").clone(),
                ));
            }
        }
    }
    assert!(
        pairs.len() >= 100,
        "restore the vendored compliance seed corpus"
    );
    pairs
}

fn parse_cases(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|n| (1..=MAX_CASES).contains(n))
        .ok_or_else(|| {
            format!("JPX_DIFF_CASES must be an integer in 1..={MAX_CASES}; set it to 32 for smoke")
        })
}

fn parse_seeds(value: &str) -> Result<Vec<u64>, String> {
    let seeds: Option<Vec<_>> = value
        .split(',')
        .map(|seed| seed.trim().parse::<u64>().ok())
        .collect();
    seeds.filter(|seeds| !seeds.is_empty() && seeds.len() <= MAX_SEEDS)
        .ok_or_else(|| format!("JPX_DIFF_SEEDS must contain 1..={MAX_SEEDS} comma-separated decimal u64 seeds; use 249,99537481"))
}

fn env_value(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => panic!("{name} must be Unicode; unset it to use the default"),
    }
}

fn artifact_dir(campaign: &str) -> PathBuf {
    let features = if cfg!(feature = "let-expr") {
        "let-expr"
    } else {
        "standard"
    };
    // Cargo sets this for integration tests to `<target-dir>/tmp`, so it
    // follows CARGO_TARGET_DIR and defaults to the workspace `target/tmp`.
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("due-scratch/differential")
        .join(campaign)
        .join(features)
}

fn write_json(path: &Path, value: &impl Serialize) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize artifact"),
    )
    .unwrap_or_else(|error| {
        panic!(
            "Cannot write {}: {error}; make the Cargo target tmp directory writable",
            path.display()
        )
    });
}

fn campaign(name: &str, default_cases: usize) {
    let cases = env_value("JPX_DIFF_CASES")
        .map(|value| parse_cases(&value).unwrap_or_else(|e| panic!("{e}")))
        .unwrap_or(default_cases);
    let seeds = env_value("JPX_DIFF_SEEDS")
        .map(|value| parse_seeds(&value).unwrap_or_else(|e| panic!("{e}")))
        .unwrap_or_else(|| DEFAULT_SEEDS.to_vec());
    let dir = artifact_dir(name);
    fs::create_dir_all(&dir)
        .expect("make differential artifact directory under the Cargo target tmp directory");
    let mut results =
        BufWriter::new(File::create(dir.join("results.jsonl")).expect("create results artifact"));
    let mut failures = 0;
    let mut allowed = 0;
    let mut counts = BTreeMap::<&str, usize>::new();
    let pairs = compliance_pairs();
    write_json(
        &dir.join("summary.json"),
        &json!({"status": "running", "cases_per_population_per_seed": cases, "seeds": seeds}),
    );
    for &seed in &seeds {
        for population in [
            Population::Grammar,
            Population::NearValid,
            Population::Compliance,
        ] {
            let mut runner = runner(seed, population);
            let strategy = if population == Population::Compliance {
                prop::sample::select(pairs.clone())
                    .prop_map(|(source, expression, document)| (Some(source), expression, document))
                    .boxed()
            } else {
                generated_strategy(population)
                    .prop_map(|(expression, document)| (None, expression, document))
                    .boxed()
            };
            for case_index in 0..cases {
                let (source, expression, document) = strategy
                    .new_tree(&mut runner)
                    .expect("bounded differential strategy must generate a case")
                    .current();
                let repro = Repro {
                    version: 1,
                    population,
                    seed,
                    case_index,
                    source,
                    expression,
                    document,
                };
                // Preserve the current input even for an abort that catch_unwind cannot catch.
                write_json(&dir.join("current-case.json"), &repro);
                let record = record(repro);
                *counts.entry(population.name()).or_default() += 1;
                if !record.agreement {
                    let path = dir.join(format!("{}-{seed}-{case_index}.json", population.name()));
                    write_json(&path, &record);
                    if let Err(message) = require_agreement(&record) {
                        failures += 1;
                        eprintln!("{message}\nReproducer: {}", path.display());
                    } else {
                        allowed += 1;
                    }
                }
                serde_json::to_writer(&mut results, &record).expect("serialize result");
                writeln!(results).expect("write result newline");
                results.flush().expect("flush results artifact");
            }
        }
    }
    let summary = json!({"status": if failures == 0 { "passed" } else { "failed" },
        "seeds": seeds, "cases_per_population_per_seed": cases,
        "populations": counts, "divergences": failures, "allowlisted": allowed});
    write_json(&dir.join("summary.json"), &summary);
    eprintln!("{summary}\nArtifacts: {}", dir.display());
    assert_eq!(
        failures, 0,
        "Unresolved differential divergences; inspect artifacts and use JPX_DIFF_REPLAY. Classify before changing either engine or the allowlist."
    );
}

#[test]
fn differential_smoke() {
    campaign("smoke", 32);
}

#[test]
#[ignore = "explicit/scheduled larger campaign; see differential/README.md"]
fn differential_large() {
    campaign("large", 1024);
}

fn replay_json(bytes: &[u8]) -> Record {
    // Record artifacts have extra outcome fields, ignored when loading Repro.
    // Always recompute outcomes; never trust an artifact's agreement flag.
    let repro: Repro = serde_json::from_slice(bytes)
        .expect("replay must be a JSON Repro/Record; use an emitted *.json artifact");
    assert_eq!(
        repro.version, 1,
        "unsupported replay version; use a version 1 artifact"
    );
    record(repro)
}

#[test]
#[ignore = "set JPX_DIFF_REPLAY to a reproducer JSON file"]
fn differential_replay() {
    let path = env_value("JPX_DIFF_REPLAY")
        .expect("set JPX_DIFF_REPLAY to the emitted reproducer JSON file");
    let bytes = fs::read(&path)
        .unwrap_or_else(|error| panic!("Cannot read {path}: {error}; check JPX_DIFF_REPLAY"));
    let record = replay_json(&bytes);
    eprintln!("{}", serde_json::to_string_pretty(&record).unwrap());
    require_agreement(&record).unwrap_or_else(|message| panic!("{message}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repro(expression: &str, document: Value) -> Repro {
        Repro {
            version: 1,
            population: Population::Grammar,
            seed: 249,
            case_index: 0,
            source: None,
            expression: expression.to_owned(),
            document,
        }
    }

    #[test]
    fn numeric_and_composite_comparison_is_exact() {
        for (a, b, equal) in [
            ("1", "1.0", true),
            ("0", "-0.0", true),
            ("-1", "-1.0", true),
            ("1", "1.5", false),
            ("-1", "18446744073709551615", false),
            ("9007199254740992", "9007199254740993", false),
            ("9007199254740992.0", "9007199254740993", false),
            ("9007199254740992.0", "9007199254740992", true),
            ("9223372036854775807", "9223372036854775808", false),
            ("18446744073709551614", "18446744073709551615", false),
            ("18446744073709551615", "18446744073709551616.0", false),
            ("-9223372036854775808", "-9223372036854775808.0", true),
            ("-9223372036854775807", "-9223372036854775808.0", false),
            ("1.25", "1.5", false),
            (
                r#"{"a":[1,{"n":2}],"b":null}"#,
                r#"{"b":null,"a":[1.0,{"n":2.0}]}"#,
                true,
            ),
            ("[1,2]", "[2,1]", false),
            ("[1]", "[1,2]", false),
            (
                r#"{"n":[9007199254740992]}"#,
                r#"{"n":[9007199254740993]}"#,
                false,
            ),
            (r#"{"a":null}"#, "{}", false),
            ("null", "false", false),
        ] {
            let a = serde_json::from_str(a).unwrap();
            let b = serde_json::from_str(b).unwrap();
            assert_eq!(json_equal(&a, &b), equal, "{a} vs {b}");
            assert_eq!(json_equal(&b, &a), equal, "{b} vs {a}");
        }
    }

    #[test]
    fn typed_errors_compare_by_class_not_message() {
        for (expression, document, class) in [
            ("foo[", json!(null), ErrorClass::Syntax),
            ("[::0]", json!([1]), ErrorClass::InvalidValue),
            ("length()", json!(null), ErrorClass::InvalidArity),
            ("length(@, @)", json!(null), ErrorClass::InvalidArity),
            (
                "unknown_differential_function()",
                json!(null),
                ErrorClass::UnknownFunction,
            ),
            ("length(@)", json!(true), ErrorClass::InvalidType),
            (
                "sort_by(@, &foo)",
                json!([{"foo": true}]),
                ErrorClass::InvalidType,
            ),
        ] {
            let record = record(repro(expression, document));
            assert_eq!(record.jpx, Outcome::Error(class.clone()), "{expression}");
            assert_eq!(record.reference, Outcome::Error(class), "{expression}");
            assert!(require_agreement(&record).is_ok());
        }
        // Even text that resembles another error must not influence classification.
        assert_eq!(
            jpx_error(
                jpx_core::ErrorReason::Parse("invalid-type".into()),
                Phase::Compile
            ),
            Outcome::Error(ErrorClass::Syntax)
        );
        assert_eq!(
            reference_error(
                jmespath::ErrorReason::Parse("unknown-function".into()),
                Phase::Compile
            ),
            Outcome::Error(ErrorClass::Syntax)
        );
        assert!(!agrees(
            &Outcome::Error(ErrorClass::InvalidType),
            &Outcome::Error(ErrorClass::Syntax)
        ));
        assert!(!agrees(
            &Outcome::Json(json!(null)),
            &Outcome::Error(ErrorClass::InvalidType)
        ));
    }

    #[test]
    fn runtime_parse_errors_are_unexpected_not_syntax_agreement() {
        let jpx = jpx_error(jpx_core::ErrorReason::Parse("same".into()), Phase::Search);
        let reference = reference_error(jmespath::ErrorReason::Parse("same".into()), Phase::Search);
        assert!(matches!(jpx, Outcome::Unexpected(_)));
        assert!(matches!(reference, Outcome::Unexpected(_)));
        assert!(!agrees(&jpx, &reference));

        // The reference (0.3 through 0.5) compiles this valid expression, then
        // fails while converting the empty average (NaN) into a JSON Number.
        let (jpx, reference) = evaluate("avg(@)", &json!([]));
        assert_eq!(jpx, Outcome::Json(Value::Null));
        assert!(matches!(reference, Outcome::Unexpected(_)));
        assert!(!agrees(&jpx, &reference));
    }

    #[test]
    fn mismatches_fail_with_complete_reproducers() {
        let mut record = record(repro("@", json!({"id": 9007199254740993u64})));
        record.reference = Outcome::Json(json!({"id": 9007199254740992u64}));
        record.agreement = agrees(&record.jpx, &record.reference);
        let message = require_agreement(&record).expect_err("injected mismatch must fail");
        for field in [
            "expression",
            "document",
            "population",
            "seed",
            "case_index",
            "jpx",
            "reference",
            "9007199254740993",
        ] {
            assert!(message.contains(field), "missing {field}: {message}");
        }
    }

    #[test]
    fn panics_unexpected_errors_and_conversion_failures_never_agree() {
        let panic = guarded(|| panic!("injected engine panic"));
        assert!(matches!(panic, Outcome::Panic(_)));
        let unmapped = jpx_error(
            jpx_core::ErrorReason::Runtime(jpx_core::RuntimeError::RecursionLimitExceeded {
                limit: 128,
            }),
            Phase::Search,
        );
        assert!(matches!(unmapped, Outcome::Unexpected(_)));
        for outcome in [
            panic,
            unmapped,
            Outcome::ConversionFailure("injected conversion failure".into()),
        ] {
            assert!(!agrees(&outcome, &outcome));
            assert!(!agrees(&outcome, &Outcome::Json(json!(null))));
            let repro = repro("@", json!(null));
            let entry = AllowedDivergence {
                expression: "@",
                document_json: "null",
                jpx: outcome.clone(),
                reference: outcome.clone(),
                reason: "must never mask a harness/engine failure",
            };
            assert!(allowed_reason(&repro, &outcome, &outcome, &[entry]).is_none());
            let record = Record {
                repro,
                jpx: outcome.clone(),
                reference: outcome,
                agreement: false,
                allowlist_reason: None,
            };
            assert!(require_agreement(&record).is_err());
        }
    }

    #[test]
    fn reference_conversion_preserves_numbers_and_rejects_expression_references() {
        let document = json!({"n": [i64::MIN, i64::MAX, 9007199254740993u64, u64::MAX, 1.25]});
        let (jpx, reference) = evaluate("@", &document);
        assert_eq!(jpx, Outcome::Json(document.clone()));
        assert_eq!(reference, Outcome::Json(document));
        let expref = jmespath::Variable::Expref(jmespath::ast::Ast::Identity { offset: 0 });
        assert!(matches!(reference_json(&expref), Outcome::Unexpected(_)));
        let nested = jmespath::Variable::Array(vec![jmespath::Rcvar::new(expref)]);
        assert!(matches!(reference_json(&nested), Outcome::Unexpected(_)));
    }

    #[test]
    fn allowlist_is_empty_and_matching_is_narrow_and_reason_bearing() {
        assert_eq!(
            ALLOWLIST.len(),
            0,
            "exceptions require explicit review and a retained reproducer"
        );
        let mut repro = repro("@", json!(1));
        let jpx = Outcome::Json(json!(1));
        let reference = Outcome::Json(json!(2));
        let mut entries = [AllowedDivergence {
            expression: "@",
            document_json: "1",
            jpx: jpx.clone(),
            reference: reference.clone(),
            reason: "test only: reviewed issue link",
        }];
        assert!(allowed_reason(&repro, &jpx, &reference, &entries).is_some());
        repro.expression = "foo".into();
        assert!(allowed_reason(&repro, &jpx, &reference, &entries).is_none());
        repro.expression = "@".into();
        repro.document = json!(2);
        assert!(allowed_reason(&repro, &jpx, &reference, &entries).is_none());
        repro.document = json!(1);
        assert!(allowed_reason(&repro, &reference, &jpx, &entries).is_none());
        entries[0].reason = " ";
        assert!(allowed_reason(&repro, &jpx, &reference, &entries).is_none());
    }

    #[test]
    fn reference_failures_are_reviewable_but_only_narrowly() {
        // The reviewed avg([]) entry covers the real retained reproducers.
        for name in ["compliance-249-322.json", "compliance-249-872.json"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/differential/reproducers")
                .join(name);
            let record = replay_json(&fs::read(&path).expect("read retained reproducer"));
            assert!(
                !record.agreement,
                "{name} no longer diverges; drop the entry"
            );
            assert_eq!(record.jpx, Outcome::Json(Value::Null));
            assert_eq!(
                record.reference,
                Outcome::Unexpected(AVG_EMPTY_REFERENCE_FAILURE.into())
            );
            assert!(record.allowlist_reason.is_some(), "{name}");
            assert!(require_agreement(&record).is_ok());
        }

        let repro = repro("avg(empty_list)", json!({"empty_list": []}));
        let jpx = Outcome::Json(Value::Null);
        let reference = Outcome::Unexpected(AVG_EMPTY_REFERENCE_FAILURE.into());
        let covered = |repro: &Repro, jpx: &Outcome, reference: &Outcome| {
            reference_failure_reason(repro, jpx, reference, REFERENCE_FAILURES).is_some()
        };
        assert!(covered(&repro, &jpx, &reference));
        // A different expression, jpx result, or reference failure is not covered.
        assert!(!covered(
            &self::repro("avg(@)", json!([])),
            &jpx,
            &reference
        ));
        assert!(!covered(&repro, &Outcome::Json(json!(0)), &reference));
        assert!(!covered(
            &repro,
            &jpx,
            &Outcome::Unexpected("parse error during search: other".into())
        ));
        // jpx-side panics and unexpected errors are never covered, nor are
        // conversion failures or panics in the reference.
        for bad_jpx in [
            Outcome::Panic("injected".into()),
            Outcome::Unexpected(AVG_EMPTY_REFERENCE_FAILURE.into()),
            Outcome::ConversionFailure("injected".into()),
        ] {
            assert!(!covered(&repro, &bad_jpx, &reference));
        }
        for bad_reference in [
            Outcome::Panic(AVG_EMPTY_REFERENCE_FAILURE.into()),
            Outcome::ConversionFailure(AVG_EMPTY_REFERENCE_FAILURE.into()),
            Outcome::Json(json!(0)),
        ] {
            assert!(!covered(&repro, &jpx, &bad_reference));
        }
        // An entry cannot be written to cover a jpx failure, and needs a reason.
        let entries = [
            ReferenceFailure {
                expression: "avg(empty_list)",
                jpx: Outcome::Unexpected(String::new()),
                reference_failure: AVG_EMPTY_REFERENCE_FAILURE,
                reason: "must never mask a jpx failure",
            },
            ReferenceFailure {
                expression: "avg(empty_list)",
                jpx: Outcome::Json(Value::Null),
                reference_failure: AVG_EMPTY_REFERENCE_FAILURE,
                reason: " ",
            },
        ];
        assert!(
            reference_failure_reason(
                &repro,
                &Outcome::Unexpected(String::new()),
                &reference,
                &entries
            )
            .is_none()
        );
        assert!(reference_failure_reason(&repro, &jpx, &reference, &entries).is_none());
        for entry in REFERENCE_FAILURES {
            assert!(entry.reason.contains("Reference bug"));
            assert!(matches!(entry.jpx, Outcome::Json(_) | Outcome::Error(_)));
        }
    }

    #[test]
    fn replay_reproduces_a_real_divergence_without_generation() {
        // Reference 0.3 compares numbers through f64: a likely reference bug,
        // independently established by jpx's existing exact_numeric regressions.
        let original = record(repro(
            "a == b",
            json!({"a": 9007199254740992u64, "b": 9007199254740993u64}),
        ));
        assert_eq!(original.jpx, Outcome::Json(json!(false)));
        assert_eq!(original.reference, Outcome::Json(json!(true)));
        assert!(require_agreement(&original).is_err());
        let mut bytes = serde_json::to_value(&original).unwrap();
        bytes["agreement"] = json!(true); // A saved verdict cannot suppress replay.
        let replayed = replay_json(&serde_json::to_vec(&bytes).unwrap());
        assert_eq!(original, replayed);
        assert!(require_agreement(&replayed).is_err());
    }

    #[test]
    fn both_generated_populations_are_deterministic_bounded_and_replayable() {
        for population in [Population::Grammar, Population::NearValid] {
            let strategy = generated_strategy(population);
            let mut first = runner(249, population);
            let mut second = runner(249, population);
            for case_index in 0..32 {
                let input = strategy.new_tree(&mut first).unwrap().current();
                assert_eq!(input, strategy.new_tree(&mut second).unwrap().current());
                if population == Population::Grammar {
                    assert!(
                        !input.0.contains('$'),
                        "standard grammar must never enable let syntax"
                    );
                }
                assert!(input.0.len() < 100_000);
                assert!(serde_json::to_vec(&input.1).unwrap().len() < 100_000);
                let original = record(Repro {
                    version: 1,
                    population,
                    seed: 249,
                    case_index,
                    source: None,
                    expression: input.0,
                    document: input.1,
                });
                let replayed = replay_json(&serde_json::to_vec(&original).unwrap());
                assert_eq!(original, replayed);
            }
        }
    }

    #[cfg(feature = "let-expr")]
    #[test]
    fn factoring_keeps_let_expressions_in_the_property_population() {
        let strategy = support::grammar_expression();
        let mut runner = runner(249, Population::Grammar);
        assert!((0..128).any(|_| {
            strategy
                .new_tree(&mut runner)
                .unwrap()
                .current()
                .contains("let $x =")
        }));
    }

    #[test]
    fn compliance_seeds_keep_real_expression_document_pairs() {
        assert!(
            compliance_pairs()
                .iter()
                .any(|(source, expression, document)| {
                    source.starts_with("basic.json:")
                        && expression == "foo"
                        && document.get("foo").is_some()
                })
        );
    }

    #[test]
    fn controls_reject_unbounded_or_empty_campaigns() {
        for invalid in ["0", "10001", "-1", "lots", ""] {
            assert!(parse_cases(invalid).is_err());
        }
        for invalid in ["", "-1", "249,", "0,1,2,3,4,5,6,7,8"] {
            assert!(parse_seeds(invalid).is_err());
        }
        assert_eq!(parse_cases("1024").unwrap(), 1024);
        assert_eq!(parse_seeds("249,99537481").unwrap(), DEFAULT_SEEDS);
    }
}
