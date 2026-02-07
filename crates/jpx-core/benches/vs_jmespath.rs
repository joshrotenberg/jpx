//! Head-to-head benchmarks: jpx-core (Value-native) vs jmespath.rs (Rcvar/Variable).
//!
//! Each benchmark group runs the same expression and data through both implementations
//! so we can compare like-for-like.

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
// jpx-core helpers
// ---------------------------------------------------------------------------

fn jpx_compile_and_search(expr: &str, data: &Value) -> Value {
    let expression = jpx_core::compile(expr).unwrap();
    expression.search(data).unwrap()
}

fn jpx_search_precompiled(expression: &jpx_core::Expression<'_>, data: &Value) -> Value {
    expression.search(data).unwrap()
}

// ---------------------------------------------------------------------------
// jmespath.rs helpers
// ---------------------------------------------------------------------------

fn jmespath_compile_and_search(expr: &str, data: &Value) -> Value {
    let expression = jmespath::compile(expr).unwrap();
    // jmespath.rs requires Variable, so we need to convert
    let var = jmespath::Variable::from_json(&data.to_string()).unwrap();
    let result = expression.search(&var).unwrap();
    // Convert back to Value for fair comparison
    serde_json::to_value(result.as_ref()).unwrap()
}

fn jmespath_search_precompiled(expression: &jmespath::Expression<'_>, data: &Value) -> Value {
    let var = jmespath::Variable::from_json(&data.to_string()).unwrap();
    let result = expression.search(&var).unwrap();
    serde_json::to_value(result.as_ref()).unwrap()
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
        group.bench_function(format!("jpx/{}", name), |b| {
            b.iter(|| jpx_core::compile(black_box(expr)).unwrap());
        });
        group.bench_function(format!("jmespath/{}", name), |b| {
            b.iter(|| jmespath::compile(black_box(expr)).unwrap());
        });
    }

    group.finish();
}

fn bench_simple_field(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_field");
    let data = small_object();
    let expr = "foo.bar.baz";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_wildcard_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("wildcard_projection");
    let data = medium_array();
    let expr = "people[*].name";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");
    let data = medium_array();
    let expr = "people[?age > `30`].name";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");
    let data = medium_array();
    let expr = "sort_by(people, &age)";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_multiselect(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiselect");
    let data = medium_array();
    let expr = "people[*].{name: name, city: city}";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_large_array_wildcard(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_array_wildcard");
    let data = large_array();
    let expr = "items[*].value";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_large_array_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_array_filter");
    let data = large_array();
    let expr = "items[?score > `750`].id";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_deep_nesting(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nesting");
    let data = nested_object();
    let expr = "a.b.c.d.e.f.g";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });

    group.finish();
}

fn bench_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("functions");
    let data = medium_array();

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

        group.bench_function(format!("jpx/{}", name), |b| {
            b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
        });
        group.bench_function(format!("jmespath/{}", name), |b| {
            b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
        });
    }

    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    let data = medium_array();

    let cases = [
        ("simple", "people[0].name"),
        (
            "complex",
            "people[?age > `25` && city == 'NYC'] | [*].name | sort(@)",
        ),
    ];

    for (name, expr) in &cases {
        group.bench_function(format!("jpx/{}", name), |b| {
            b.iter(|| jpx_compile_and_search(black_box(expr), black_box(&data)));
        });
        group.bench_function(format!("jmespath/{}", name), |b| {
            b.iter(|| jmespath_compile_and_search(black_box(expr), black_box(&data)));
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
