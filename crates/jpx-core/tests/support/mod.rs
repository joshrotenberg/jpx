//! Shared property-test strategies.
//!
//! Keep these outside an individual integration test so the differential
//! harness can reuse exactly the same expression populations.

// Each integration-test binary uses a different subset of these strategies.
#![allow(dead_code, unused_imports)]

mod expression_strategy;
pub mod json_strategy;

pub use expression_strategy::{
    compliance_expressions, grammar_expression, grammar_smoke_expressions, near_valid_expression,
    standard_grammar_expression,
};
