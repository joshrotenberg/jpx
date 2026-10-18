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

criterion_group!(benches, bench_result_size, bench_length_scaling);
criterion_main!(benches);
