//! Head-to-head benchmarks: jpx-core (Value-native) vs jmespath.rs (Rcvar/Variable).
//!
//! # Measurement contract
//!
//! Each engine is timed doing its own work on its own native types, and nothing else.
//!
//! - Input is converted to each engine's native representation **outside** the timed
//!   closure. jpx-core searches `serde_json::Value`; jmespath.rs searches
//!   `jmespath::Variable`. Building the `Variable` inside `b.iter` would charge
//!   jmespath.rs a full `Value` -> `String` -> `Variable` round-trip per iteration,
//!   which is setup cost, not search cost.
//! - Results are **not** converted. jpx-core returns `Value`, jmespath.rs returns
//!   `Rcvar`. Converting one side and not the other is the same asymmetry in reverse;
//!   converting neither leaves each engine producing its own native output.
//!
//! Both rules exist because violating them inflates the reported gap. Route new search
//! benchmarks through `bench_pair` rather than hand-rolling a group, so the conversion
//! stays outside the closure by construction.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

fn small_object() -> Value {
    json!({"foo": {"bar": {"baz": "value"}}})
}

fn medium_array() -> Value {
    json!({
        "people": [
            {"name": "Alice", "age": 30, "city": "NYC"},
            {"name": "Bob", "age": 25, "city": "LA"},
            {"name": "Charlie", "age": 35, "city": "NYC"},
            {"name": "Diana", "age": 28, "city": "SF"},
            {"name": "Eve", "age": 32, "city": "NYC"},
            {"name": "Frank", "age": 45, "city": "LA"},
            {"name": "Grace", "age": 22, "city": "SF"},
            {"name": "Hank", "age": 38, "city": "NYC"},
            {"name": "Ivy", "age": 27, "city": "LA"},
            {"name": "Jack", "age": 33, "city": "SF"}
        ]
    })
}

fn large_array() -> Value {
    let items: Vec<Value> = (0..1000)
        .map(|i| json!({"id": i, "value": format!("item_{}", i), "score": i as f64 * 1.5}))
        .collect();
    json!({"items": items})
}

fn nested_object() -> Value {
    json!({
        "a": {"b": {"c": {"d": {"e": {"f": {"g": "deep"}}}}}},
        "x": [1, 2, [3, 4, [5, 6]]]
    })
}

// ---------------------------------------------------------------------------
// Conversion (setup only -- never call this inside a timed closure)
// ---------------------------------------------------------------------------

/// Converts a `Value` into jmespath.rs's native `Variable`.
///
/// This is setup. Calling it inside `b.iter` charges jmespath.rs a serialise and
/// reparse of the entire input on every iteration.
fn to_variable(data: &Value) -> jmespath::Variable {
    jmespath::Variable::from_json(&data.to_string()).unwrap()
}

// ---------------------------------------------------------------------------
// Shared harness
// ---------------------------------------------------------------------------

/// Benchmarks one expression against both engines, precompiled, with input already
/// in each engine's native form.
fn bench_pair(c: &mut Criterion, group_name: &str, expr: &str, data: &Value) {
    let mut group = c.benchmark_group(group_name);

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let jmespath_data = to_variable(data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_expr.search(black_box(data)).unwrap());
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_expr.search(black_box(&jmespath_data)).unwrap());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark groups
// ---------------------------------------------------------------------------

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let expressions = [
        ("simple_field", "foo.bar.baz"),
        ("index", "items[0].name"),
        ("wildcard", "people[*].name"),
        ("filter", "people[?age > `30`].name"),
        ("multiselect", "people[*].{name: name, city: city}"),
        ("pipe", "people[*].name | sort(@) | [0]"),
        ("function", "length(people[?city == 'NYC'])"),
        ("nested_fn", "sort_by(people, &age)[0].name"),
        ("slice", "items[10:20:2]"),
        (
            "complex",
            "people[?age > `25` && city == 'NYC'] | [*].name | sort(@)",
        ),
    ];

    for (name, expr) in &expressions {
        group.bench_function(format!("jpx/{name}"), |b| {
            b.iter(|| jpx_core::compile(black_box(expr)).unwrap());
        });
        group.bench_function(format!("jmespath/{name}"), |b| {
            b.iter(|| jmespath::compile(black_box(expr)).unwrap());
        });
    }

    group.finish();
}

fn bench_simple_field(c: &mut Criterion) {
    bench_pair(c, "simple_field", "foo.bar.baz", &small_object());
}

fn bench_wildcard_projection(c: &mut Criterion) {
    bench_pair(c, "wildcard_projection", "people[*].name", &medium_array());
}

fn bench_filter(c: &mut Criterion) {
    bench_pair(c, "filter", "people[?age > `30`].name", &medium_array());
}

fn bench_sort(c: &mut Criterion) {
    bench_pair(c, "sort", "sort_by(people, &age)", &medium_array());
}

fn bench_multiselect(c: &mut Criterion) {
    bench_pair(
        c,
        "multiselect",
        "people[*].{name: name, city: city}",
        &medium_array(),
    );
}

fn bench_large_array_wildcard(c: &mut Criterion) {
    bench_pair(c, "large_array_wildcard", "items[*].value", &large_array());
}

fn bench_large_array_filter(c: &mut Criterion) {
    bench_pair(
        c,
        "large_array_filter",
        "items[?score > `750`].id",
        &large_array(),
    );
}

fn bench_deep_nesting(c: &mut Criterion) {
    bench_pair(c, "deep_nesting", "a.b.c.d.e.f.g", &nested_object());
}

fn bench_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("functions");
    let data = medium_array();
    let jmespath_data = to_variable(&data);

    let cases = [
        ("length", "length(people)"),
        ("sort", "sort(people[*].name)"),
        ("join", "join(', ', people[*].name)"),
        ("map", "map(&name, people)"),
        ("max_by", "max_by(people, &age)"),
        ("keys", "keys(people[0])"),
    ];

    for (name, expr) in &cases {
        let jpx_expr = jpx_core::compile(expr).unwrap();
        let jmespath_expr = jmespath::compile(expr).unwrap();

        group.bench_function(format!("jpx/{name}"), |b| {
            b.iter(|| jpx_expr.search(black_box(&data)).unwrap());
        });
        group.bench_function(format!("jmespath/{name}"), |b| {
            b.iter(|| jmespath_expr.search(black_box(&jmespath_data)).unwrap());
        });
    }

    group.finish();
}

/// Compile plus search. Each engine pays its own parse cost and searches its own
/// native input; neither pays a cross-format conversion.
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    let data = medium_array();
    let jmespath_data = to_variable(&data);

    let cases = [
        ("simple", "people[0].name"),
        (
            "complex",
            "people[?age > `25` && city == 'NYC'] | [*].name | sort(@)",
        ),
    ];

    for (name, expr) in &cases {
        group.bench_function(format!("jpx/{name}"), |b| {
            b.iter(|| {
                let expression = jpx_core::compile(black_box(expr)).unwrap();
                expression.search(black_box(&data)).unwrap()
            });
        });
        group.bench_function(format!("jmespath/{name}"), |b| {
            b.iter(|| {
                let expression = jmespath::compile(black_box(expr)).unwrap();
                expression.search(black_box(&jmespath_data)).unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_simple_field,
    bench_wildcard_projection,
    bench_filter,
    bench_sort,
    bench_multiselect,
    bench_large_array_wildcard,
    bench_large_array_filter,
    bench_deep_nesting,
    bench_functions,
    bench_end_to_end,
);
criterion_main!(benches);
