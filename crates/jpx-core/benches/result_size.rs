//! Does search cost track the size of the result, or the size of the input?
//!
//! Every expression here runs against the same ~44KB document. What differs is
//! how much of it each one returns: `@` returns all of it, `items[0].id`
//! returns one byte. If cost tracks the result, the timings should spread out
//! accordingly. If cost tracks the input, they collapse together and every
//! expression pays for the whole document.
//!
//! This is the shape #236 measured. Keep it around so a regression to
//! input-proportional cost shows up as a benchmark change rather than as a
//! puzzle someone re-derives later.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde_json::{Value, json};
use std::hint::black_box;

/// Three-field objects matching the original ~44KB, 1000-item fixture.
fn document(item_count: usize) -> Value {
    let items: Vec<Value> = (0..item_count)
        .map(|i| json!({"id": i, "value": format!("item_{}", i), "score": i as f64 * 1.5}))
        .collect();
    json!({"items": items})
}

fn bench_result_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_size");
    let data = document(1000);

    // Ordered by result size, largest first.
    let cases = [
        ("whole_document", "@"),
        ("whole_array", "items"),
        ("one_element", "items[0]"),
        ("one_field", "items[0].id"),
        ("array_length", "length(items)"),
        ("small_slice", "items[10:20:2]"),
        ("deep_path", "items[0].value"),
    ];

    for (name, expr) in &cases {
        let compiled = jpx_core::compile(expr).unwrap();
        group.bench_function(*name, |b| {
            b.iter(|| compiled.search(black_box(&data)).unwrap());
        });
    }

    group.finish();
}

/// A scalar length result should not copy every element of a larger input.
fn bench_length_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("length_input_size");
    let borrowed = jpx_core::Runtime::strict();
    let mut owned = jpx_core::Runtime::strict();
    // Keep the legacy public Function path as a control in the same process.
    owned.register_function("length", Box::new(jpx_core::functions::LengthFn::new()));
    for (mode, runtime) in [("borrowed", &borrowed), ("owned", &owned)] {
        let compiled = runtime.compile("length(items)").unwrap();
        for item_count in [1, 1000, 10_000] {
            let data = document(item_count);
            group.bench_with_input(BenchmarkId::new(mode, item_count), &data, |b, data| {
                b.iter(|| compiled.search(black_box(data)).unwrap());
            });
        }
    }
    group.finish();
}

/// Function calls on a large argument, at 1 and 1,000 items. Built-ins and
/// opted-in extensions receive borrowed arguments, so `type`, `first`,
/// `is_empty`, `any_expr` and `get_path` stay flat; `contains`, `max_by` and
/// `max` visit every element but no longer copy `items` before the call.
fn bench_function_arguments(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_arguments");
    // Extended below when the extension functions are compiled in.
    #[cfg_attr(not(feature = "ext-core"), allow(unused_mut))]
    let mut runtime = jpx_core::Runtime::strict();
    #[cfg_attr(not(feature = "ext-core"), allow(unused_mut))]
    let mut cases = vec![
        ("type", "type(items)"),
        ("contains", "contains(items[*].id, `5`)"),
        ("max_by", "max_by(items, &id).id"),
        ("max", "max(items[*].id)"),
    ];
    #[cfg(feature = "ext-core")]
    {
        let mut registry = jpx_core::FunctionRegistry::new();
        registry.register_all();
        registry.apply(&mut runtime);
        cases.extend([
            ("first", "first(items).id"),
            ("is_empty", "is_empty(items)"),
            ("any_expr", "any_expr(&(id == `5`), items)"),
            ("get_path", "get_path(@, 'items[0].id')"),
        ]);
    }
    for (name, expr) in cases {
        let compiled = runtime.compile(expr).unwrap();
        for item_count in [1, 1000] {
            let data = document(item_count);
            group.bench_with_input(BenchmarkId::new(name, item_count), &data, |b, data| {
                b.iter(|| compiled.search(black_box(data)).unwrap());
            });
        }
    }
    group.finish();
}

/// Selecting small paths from the same let-bound container should not copy
/// the whole container on each reference. Binding still owns one input copy.
#[cfg(feature = "let-expr")]
fn bench_let_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("let_paths");
    let runtime = jpx_core::Runtime::strict();
    let cases = [
        ("native_control", "items[0].id"),
        (
            "scoped_input_control",
            "let $unused = `null` in items[0].id",
        ),
        ("one_lookup", "let $x = items in $x[0].id"),
        (
            "repeated_lookup",
            "let $x = items in [$x[0].id, $x[-1].value, $x[1].score, $x[0].id]",
        ),
        ("whole_value_control", "let $x = items in $x"),
    ];
    for (name, source) in cases {
        let expression = runtime.compile(source).unwrap();
        for item_count in [1, 1000, 10_000] {
            let data = document(item_count);
            group.bench_with_input(BenchmarkId::new(name, item_count), &data, |b, data| {
                b.iter(|| expression.search(black_box(data)).unwrap());
            });
        }
    }
    for depth in [1, 25, 50] {
        let mut data = json!({"id": 7});
        for _ in 0..depth {
            data = json!({"child": data});
        }
        let source = format!("let $unused = `null` in {}id", "child.".repeat(depth));
        let expression = runtime.compile(&source).unwrap();
        group.bench_with_input(
            BenchmarkId::new("scoped_depth_control", depth),
            &data,
            |b, data| {
                b.iter(|| expression.search(black_box(data)).unwrap());
            },
        );
    }
    group.finish();
}

#[cfg(not(feature = "let-expr"))]
fn bench_let_paths(_: &mut Criterion) {}

criterion_group!(
    benches,
    bench_result_size,
    bench_length_scaling,
    bench_function_arguments,
    bench_let_paths
);
criterion_main!(benches);
