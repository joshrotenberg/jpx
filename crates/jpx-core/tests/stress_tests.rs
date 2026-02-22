//! Stress tests for jpx-core exercising large/extreme inputs.
//!
//! Exercises correctness at scale: 10K-element arrays, 1K-key objects,
//! 100-level nesting, and 1MB strings.

use serde_json::{Value, json};

use jpx_core::Runtime;

fn runtime() -> Runtime {
    Runtime::builder()
        .with_standard()
        .with_all_extensions()
        .build()
}

fn eval(expr: &str, data: &Value) -> Value {
    let rt = runtime();
    let compiled = rt.compile(expr).expect("expression should compile");
    compiled.search(data).expect("search should succeed")
}

// ---------------------------------------------------------------------------
// 1. Large arrays (10K elements)
// ---------------------------------------------------------------------------

#[test]

fn large_array_wildcard_projection() {
    let data: Value = Value::Array(
        (0..10_000)
            .map(|i| json!({"id": i, "name": format!("item_{i}")}))
            .collect(),
    );
    let result = eval("[*].id", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 10_000);
    assert_eq!(arr[0], json!(0));
    assert_eq!(arr[9_999], json!(9_999));
}

#[test]

fn large_array_filter() {
    let data: Value = Value::Array((0..10_000).map(|i| json!({"id": i, "score": i})).collect());
    let result = eval("[?score > `5000`]", &data);
    let arr = result.as_array().expect("result should be array");
    // scores 5001..9999 -> 4999 elements
    assert_eq!(arr.len(), 4_999);
    assert_eq!(arr[0]["id"], json!(5001));
}

#[test]

fn large_array_sort_by() {
    let data: Value = Value::Array((0..10_000).rev().map(|i| json!({"id": i})).collect());
    let result = eval("sort_by(@, &id)", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 10_000);
    assert_eq!(arr[0]["id"], json!(0));
    assert_eq!(arr[9_999]["id"], json!(9_999));
    // Verify ordering
    for i in 1..arr.len() {
        assert!(arr[i - 1]["id"].as_i64() <= arr[i]["id"].as_i64());
    }
}

#[test]

fn large_array_group_by() {
    let data: Value = Value::Array(
        (0..10_000)
            .map(|i| json!({"id": i, "category": format!("cat_{}", i % 10)}))
            .collect(),
    );
    let result = eval("group_by(@, 'category')", &data);
    let obj = result.as_object().expect("result should be object");
    assert_eq!(obj.len(), 10);
    for (_key, group) in obj {
        let group_arr = group.as_array().expect("group should be array");
        assert_eq!(group_arr.len(), 1_000);
    }
}

#[test]

fn large_array_flatten() {
    // 1K sub-arrays of 10 elements each
    let data: Value = Value::Array(
        (0..1_000)
            .map(|i| Value::Array((0..10).map(|j| json!(i * 10 + j)).collect()))
            .collect(),
    );
    let result = eval("[]", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 10_000);
    assert_eq!(arr[0], json!(0));
    assert_eq!(arr[9_999], json!(9_999));
}

#[test]

fn large_array_unique() {
    // 10K elements with lots of duplicates (values 0..100 repeated)
    let data: Value = Value::Array((0..10_000).map(|i| json!(i % 100)).collect());
    let result = eval("unique(@)", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 100);
}

#[test]

fn large_array_map_join() {
    let data: Value = Value::Array(
        (0..10_000)
            .map(|i| json!({"name": format!("n{i}")}))
            .collect(),
    );
    let result = eval("[*].name | join(', ', @)", &data);
    let s = result.as_str().expect("result should be string");
    assert!(s.contains("n0"));
    assert!(s.contains("n9999"));
    // Commas between 10K items -> at least 9999 commas
    assert!(s.matches(", ").count() >= 9_999);
}

// ---------------------------------------------------------------------------
// 2. Wide objects (1K keys)
// ---------------------------------------------------------------------------

fn wide_object(n: usize) -> Value {
    let map: serde_json::Map<String, Value> =
        (0..n).map(|i| (format!("field_{i}"), json!(i))).collect();
    Value::Object(map)
}

#[test]

fn wide_object_keys() {
    let data = wide_object(1_000);
    let result = eval("keys(@)", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 1_000);
}

#[test]

fn wide_object_values() {
    let data = wide_object(1_000);
    let result = eval("values(@)", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 1_000);
}

#[test]

fn wide_object_direct_access() {
    let data = wide_object(1_000);
    let result = eval("field_999", &data);
    assert_eq!(result, json!(999));
}

// ---------------------------------------------------------------------------
// 3. Deep nesting (100+ levels)
// ---------------------------------------------------------------------------

fn deeply_nested(depth: usize) -> Value {
    let mut val = json!("leaf");
    for _ in 0..depth {
        val = json!({"a": val});
    }
    val
}

#[test]

fn deep_nesting_field_access() {
    let depth = 100;
    let data = deeply_nested(depth);
    let expr = (0..depth).map(|_| "a").collect::<Vec<_>>().join(".");
    let result = eval(&expr, &data);
    assert_eq!(result, json!("leaf"));
}

#[test]

fn deep_nesting_type() {
    let depth = 100;
    let data = deeply_nested(depth);
    let mut expr = (0..depth).map(|_| "a").collect::<Vec<_>>().join(".");
    expr = format!("type({expr})");
    let result = eval(&expr, &data);
    assert_eq!(result, json!("string"));
}

#[test]

fn deep_nesting_array_wildcard() {
    // 50 levels of single-element arrays
    let depth = 50;
    let mut val = json!(42);
    for _ in 0..depth {
        val = json!([val]);
    }
    let expr = (0..depth).map(|_| "[0]").collect::<Vec<_>>().join("");
    let result = eval(&expr, &val);
    assert_eq!(result, json!(42));
}

// ---------------------------------------------------------------------------
// 4. Large strings (100KB-1MB)
// ---------------------------------------------------------------------------

#[test]

fn large_string_length() {
    let big = "x".repeat(1_000_000);
    let data = json!(big);
    let result = eval("length(@)", &data);
    assert_eq!(result, json!(1_000_000));
}

#[test]

fn large_string_upper() {
    let big = "a".repeat(100_000);
    let data = json!(big);
    let result = eval("upper(@)", &data);
    let s = result.as_str().expect("result should be string");
    assert_eq!(s.len(), 100_000);
    assert!(s.chars().all(|c| c == 'A'));
}

#[test]

fn large_string_split() {
    // 10K commas -> 10,001 parts
    let parts: Vec<String> = (0..10_001).map(|i| format!("v{i}")).collect();
    let big = parts.join(",");
    let data = json!(big);
    let result = eval("split(@, `\",\"`)", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 10_001);
    assert_eq!(arr[0], json!("v0"));
    assert_eq!(arr[10_000], json!("v10000"));
}

#[test]

fn large_string_contains() {
    let mut big = "x".repeat(999_990);
    big.push_str("needle");
    let data = json!(big);
    let result = eval("contains(@, 'needle')", &data);
    assert_eq!(result, json!(true));

    let no_needle = json!("x".repeat(1_000_000));
    let result = eval("contains(@, 'needle')", &no_needle);
    assert_eq!(result, json!(false));
}

// ---------------------------------------------------------------------------
// 5. Combinations
// ---------------------------------------------------------------------------

#[test]

fn wide_and_nested() {
    // 100 keys, each with 100 nested keys
    let inner = wide_object(100);
    let map: serde_json::Map<String, Value> = (0..100)
        .map(|i| (format!("outer_{i}"), inner.clone()))
        .collect();
    let data = Value::Object(map);

    let result = eval("outer_50.field_99", &data);
    assert_eq!(result, json!(99));

    let result = eval("keys(@)", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 100);
}

#[test]

fn array_of_wide_objects() {
    // 1K objects x 100 keys each
    let data: Value = Value::Array(
        (0..1_000)
            .map(|i| {
                let mut map: serde_json::Map<String, Value> =
                    (0..100).map(|j| (format!("f_{j}"), json!(j))).collect();
                map.insert("id".to_string(), json!(i));
                Value::Object(map)
            })
            .collect(),
    );

    let result = eval("[*].id", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 1_000);
    assert_eq!(arr[0], json!(0));
    assert_eq!(arr[999], json!(999));

    let result = eval("[*].f_50", &data);
    let arr = result.as_array().expect("result should be array");
    assert_eq!(arr.len(), 1_000);
    assert!(arr.iter().all(|v| v == &json!(50)));
}
