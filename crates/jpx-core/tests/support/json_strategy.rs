use proptest::prelude::*;
use serde_json::{Value, json};

/// Generates an arbitrary JSON value with bounded recursion depth.
#[allow(dead_code)] // Not every integration-test binary uses this strategy.
pub fn arb_json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        any::<f64>()
            .prop_filter("finite", |f| f.is_finite())
            .prop_map(|n| json!(n)),
        "[a-zA-Z0-9_ ]{0,20}".prop_map(|s| json!(s)),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
            prop::collection::hash_map("[a-z]{1,5}", inner, 0..5)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// Differential documents use ordered maps and exactly replayable dyadic floats.
/// Keep the original property-test population above unchanged (including its
/// arbitrary finite floats). Full-width integers and precision boundaries are
/// intentional: a lossy reference result must remain a visible divergence.
#[allow(dead_code)] // Not every integration-test binary uses this strategy.
pub fn differential_document() -> BoxedStrategy<Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        any::<u64>().prop_map(|n| json!(n)),
        any::<i16>().prop_map(|n| json!(f64::from(n) / 4.0)),
        prop::sample::select(vec![
            json!(0),
            json!(1),
            json!(1.0),
            json!(-0.0),
            json!(9007199254740992u64),
            json!(9007199254740993u64),
            json!(i64::MIN),
            json!(i64::MAX),
            json!(u64::MAX),
        ]),
        "[a-zA-Z0-9_ '\"\\\\\nλ]{0,20}".prop_map(Value::String),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        let key = prop_oneof![
            "[a-z]{1,5}",
            prop::sample::select(vec!["foo", "bar", "items", "name", "age", "a", "b"])
                .prop_map(str::to_owned),
        ];
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
            prop::collection::btree_map(key, inner, 0..5)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
    .boxed()
}
