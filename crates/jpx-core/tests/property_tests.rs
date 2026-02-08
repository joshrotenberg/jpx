//! Property-based tests for jpx-core using proptest.
//!
//! These tests verify safety invariants: no panics on arbitrary input,
//! reflexivity/symmetry of equality, and bounded output for slices.

use proptest::prelude::*;
use serde_json::{Value, json};

use jpx_core::{Runtime, ValueExt};

/// Generates an arbitrary JSON value with bounded recursion depth.
fn arb_json_value() -> impl Strategy<Value = Value> {
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

/// Generates syntactically plausible JMESPath expressions.
fn arb_jmespath_expr() -> impl Strategy<Value = String> {
    let simple = prop_oneof![
        Just("@".to_string()),
        "[a-zA-Z_][a-zA-Z0-9_]{0,5}",
        (0..10i32).prop_map(|n| format!("[{n}]")),
        Just("*".to_string()),
        Just("`null`".to_string()),
    ];
    prop_oneof![
        simple.clone(),
        (simple.clone(), simple.clone()).prop_map(|(a, b)| format!("{a}.{b}")),
        (simple.clone(), simple.clone()).prop_map(|(a, b)| format!("{a} | {b}")),
        simple.clone().prop_map(|s| format!("length({s})")),
    ]
}

proptest! {
    /// The parser must never panic on arbitrary input strings.
    #[test]
    fn parser_never_panics(s in ".*") {
        let _ = jpx_core::parse(&s);
    }

    /// The interpreter must never panic when given a valid expression and arbitrary data.
    #[test]
    fn interpreter_never_panics(
        expr_str in arb_jmespath_expr(),
        data in arb_json_value(),
    ) {
        let rt = Runtime::strict();
        if let Ok(expr) = rt.compile(&expr_str) {
            let _ = expr.search(&data);
        }
    }

    /// ValueExt field access must never panic on arbitrary values and field names.
    #[test]
    fn value_ext_field_never_panics(
        data in arb_json_value(),
        field in "[a-z]{1,10}",
    ) {
        let _ = data.get_field(&field);
    }

    /// ValueExt index access must never panic.
    #[test]
    fn value_ext_index_never_panics(
        data in arb_json_value(),
        idx in 0..100usize,
    ) {
        let _ = data.get_index(idx);
        let _ = data.get_negative_index(idx);
    }

    /// Equality must be reflexive: every value equals itself.
    #[test]
    fn equality_reflexive(v in arb_json_value()) {
        let result = v.compare(&jpx_core::ast::Comparator::Equal, &v);
        prop_assert_eq!(result, Some(true));
    }

    /// Equality must be symmetric: if v == w then w == v.
    #[test]
    fn equality_symmetric(
        v in arb_json_value(),
        w in arb_json_value(),
    ) {
        let vw = v.compare(&jpx_core::ast::Comparator::Equal, &w);
        let wv = w.compare(&jpx_core::ast::Comparator::Equal, &v);
        prop_assert_eq!(vw, wv);
    }

    /// Slice output length must be bounded by input length.
    #[test]
    fn slice_result_bounded(
        arr in prop::collection::vec(arb_json_value(), 0..20),
        start in prop::option::of(-20i32..20),
        stop in prop::option::of(-20i32..20),
        step in prop_oneof![(-5i32..-1), (1i32..5)],
    ) {
        let val = Value::Array(arr.clone());
        if let Some(result) = val.slice(start, stop, step) {
            prop_assert!(result.len() <= arr.len());
        }
    }
}

#[cfg(feature = "extensions")]
proptest! {
    /// Extension functions must never panic when given safe expressions and arbitrary JSON.
    #[test]
    fn extension_functions_never_panic(
        data in arb_json_value(),
    ) {
        let rt = Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build();

        let safe_exprs = [
            "lower(to_string(@))",
            "upper(to_string(@))",
            "length(to_string(@))",
            "type(@)",
            "to_string(@)",
            "to_number(to_string(@))",
            "is_string(@)",
            "is_number(@)",
            "is_array(@)",
            "is_object(@)",
            "is_null(@)",
            "is_boolean(@)",
        ];

        for expr_str in &safe_exprs {
            if let Ok(expr) = rt.compile(expr_str) {
                let _ = expr.search(&data);
            }
        }
    }
}
