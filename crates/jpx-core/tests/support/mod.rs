//! Shared property-test strategies.
//!
//! Keep these outside an individual integration test so the differential
//! harness can reuse exactly the same expression populations.

mod expression_strategy;

pub use expression_strategy::{
    compliance_expressions, grammar_expression, grammar_smoke_expressions, near_valid_expression,
};
