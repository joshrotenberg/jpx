//! Head-to-head benchmarks: jpx-core vs jmespath.rs vs jmespath-community.
//!
//! Each benchmark group runs the same expression and data through all implementations
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
    let var = jmespath::Variable::from_json(&data.to_string()).unwrap();
    let result = expression.search(&var).unwrap();
    serde_json::to_value(result.as_ref()).unwrap()
}

fn jmespath_search_precompiled(expression: &jmespath::Expression<'_>, data: &Value) -> Value {
    let var = jmespath::Variable::from_json(&data.to_string()).unwrap();
    let result = expression.search(&var).unwrap();
    serde_json::to_value(result.as_ref()).unwrap()
}

// ---------------------------------------------------------------------------
// jmespath-community helpers
// ---------------------------------------------------------------------------

fn community_compile_and_search(
    expr: &str,
    data: &jmespath_community::Value,
) -> jmespath_community::Value {
    let ast = jmespath_community::parse(expr).unwrap();
    ast.search(data).unwrap()
}

fn community_search_precompiled(
    ast: &jmespath_community::AST,
    data: &jmespath_community::Value,
) -> jmespath_community::Value {
    ast.search(data).unwrap()
}

/// Try to parse and search with community; returns None if the expression
/// uses features/functions that jmespath-community doesn't support.
fn community_try_compile(
    expr: &str,
    data: &jmespath_community::Value,
) -> Option<jmespath_community::AST> {
    let ast = jmespath_community::parse(expr).ok()?;
    // Verify it actually works (some functions may be unimplemented)
    ast.search(data).ok()?;
    Some(ast)
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
        // Only bench community parse if it can actually parse the expression
        if jmespath_community::parse(expr).is_ok() {
            group.bench_function(format!("community/{name}"), |b| {
                b.iter(|| jmespath_community::parse(black_box(expr)).unwrap());
            });
        }
    }

    group.finish();
}

/// Helper macro to add a community benchmark only if the expression is supported.
macro_rules! bench_community {
    ($group:expr, $name:expr, $ast:expr, $data:expr) => {
        if let Some(ref ast) = $ast {
            let data = $data;
            $group.bench_function($name, |b| {
                b.iter(|| community_search_precompiled(ast, black_box(data)));
            });
        }
    };
}

fn bench_simple_field(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_field");
    let data = small_object();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "foo.bar.baz";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_wildcard_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("wildcard_projection");
    let data = medium_array();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "people[*].name";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");
    let data = medium_array();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "people[?age > `30`].name";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");
    let data = medium_array();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "sort_by(people, &age)";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_multiselect(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiselect");
    let data = medium_array();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "people[*].{name: name, city: city}";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_large_array_wildcard(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_array_wildcard");
    let data = large_array();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "items[*].value";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_large_array_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_array_filter");
    let data = large_array();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "items[?score > `750`].id";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_deep_nesting(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nesting");
    let data = nested_object();
    let community_data = jmespath_community::Value::map_from_json(&data);
    let expr = "a.b.c.d.e.f.g";

    let jpx_expr = jpx_core::compile(expr).unwrap();
    let jmespath_expr = jmespath::compile(expr).unwrap();
    let community_ast = community_try_compile(expr, &community_data);

    group.bench_function("jpx", |b| {
        b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
    });
    group.bench_function("jmespath", |b| {
        b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
    });
    bench_community!(group, "community", community_ast, &community_data);

    group.finish();
}

fn bench_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("functions");
    let data = medium_array();
    let community_data = jmespath_community::Value::map_from_json(&data);

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
        let community_ast = community_try_compile(expr, &community_data);

        group.bench_function(format!("jpx/{name}"), |b| {
            b.iter(|| jpx_search_precompiled(&jpx_expr, black_box(&data)));
        });
        group.bench_function(format!("jmespath/{name}"), |b| {
            b.iter(|| jmespath_search_precompiled(&jmespath_expr, black_box(&data)));
        });
        if let Some(ref ast) = community_ast {
            group.bench_function(format!("community/{name}"), |b| {
                b.iter(|| community_search_precompiled(ast, black_box(&community_data)));
            });
        }
    }

    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    let data = medium_array();
    let community_data = jmespath_community::Value::map_from_json(&data);

    let cases = [
        ("simple", "people[0].name"),
        (
            "complex",
            "people[?age > `25` && city == 'NYC'] | [*].name | sort(@)",
        ),
    ];

    for (name, expr) in &cases {
        let community_supported = community_try_compile(expr, &community_data).is_some();

        group.bench_function(format!("jpx/{name}"), |b| {
            b.iter(|| jpx_compile_and_search(black_box(expr), black_box(&data)));
        });
        group.bench_function(format!("jmespath/{name}"), |b| {
            b.iter(|| jmespath_compile_and_search(black_box(expr), black_box(&data)));
        });
        if community_supported {
            group.bench_function(format!("community/{name}"), |b| {
                b.iter(|| {
                    community_compile_and_search(black_box(expr), black_box(&community_data))
                });
            });
        }
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
