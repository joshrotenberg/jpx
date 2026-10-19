//! Property-based tests for jpx-core using proptest.
//!
//! These tests verify safety invariants: no panics on arbitrary input,
//! reflexivity/symmetry of equality, and bounded output for slices.

use proptest::prelude::*;
use serde_json::Value;

use jpx_core::{Runtime, ValueExt};

mod support;
use support::{
    compliance_expressions, grammar_expression, grammar_smoke_expressions, near_valid_expression,
};

use support::json_strategy::arb_json_value;

proptest! {
    /// The parser must never panic on arbitrary input strings.
    #[test]
    fn parser_never_panics(s in ".*") {
        let _ = jpx_core::parse(&s);
    }

    /// Grammar-directed expressions remain syntactically valid as productions are composed.
    #[test]
    fn grammar_generated_expressions_parse(expr_str in grammar_expression()) {
        prop_assert!(
            jpx_core::parse(&expr_str).is_ok(),
            "grammar generator emitted invalid syntax: {expr_str}"
        );
    }

    /// The interpreter must never panic when given a valid expression and arbitrary data.
    #[test]
    fn interpreter_never_panics(
        expr_str in grammar_expression(),
        data in arb_json_value(),
    ) {
        let rt = Runtime::strict();
        if let Ok(expr) = rt.compile(&expr_str) {
            let _ = expr.search(&data);
        }
    }

    /// Small mutations of real compliance expressions must return a result or error, never panic.
    #[test]
    fn near_valid_parser_never_panics(expr_str in near_valid_expression()) {
        let _ = jpx_core::parse(&expr_str);
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

#[test]
fn grammar_smoke_corpus_covers_valid_production_families() {
    for expression in grammar_smoke_expressions() {
        assert!(
            jpx_core::parse(expression).is_ok(),
            "grammar smoke expression should parse: {expression}"
        );
    }
}

#[test]
fn near_valid_population_is_derived_from_the_vendored_compliance_suite() {
    let expressions = compliance_expressions();
    assert!(
        expressions.len() >= 100,
        "expected broad compliance seed population, found {}",
        expressions.len()
    );
    assert!(
        expressions
            .iter()
            .any(|expression| expression.contains("[?"))
    );
    assert!(
        expressions
            .iter()
            .any(|expression| expression.contains('('))
    );
    assert!(
        expressions
            .iter()
            .any(|expression| expression.contains(':'))
    );
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
