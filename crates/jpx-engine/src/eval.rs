//! JMESPath expression evaluation.
//!
//! This module provides the core evaluation methods for the [`JpxEngine`]:
//! single evaluation, string-based evaluation, batch evaluation, and validation.

use crate::JpxEngine;
use crate::error::{EngineError, Result};
use crate::types::{BatchEvaluateResult, BatchExpressionResult, ValidationResult};
use serde_json::Value;

impl JpxEngine {
    /// Evaluates a JMESPath expression against JSON input.
    ///
    /// This is the primary method for running JMESPath queries. The expression
    /// is compiled and executed against the provided JSON value.
    ///
    /// # Arguments
    ///
    /// * `expression` - A JMESPath expression string
    /// * `input` - JSON data to query
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::InvalidExpression`] if the expression has syntax errors,
    /// or [`EngineError::EvaluationFailed`] if evaluation fails (e.g., calling an
    /// undefined function in strict mode).
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Simple field access
    /// let result = engine.evaluate("name", &json!({"name": "alice"})).unwrap();
    /// assert_eq!(result, json!("alice"));
    ///
    /// // Array projection with function
    /// let result = engine.evaluate("users[*].name | sort(@)", &json!({
    ///     "users": [{"name": "charlie"}, {"name": "alice"}, {"name": "bob"}]
    /// })).unwrap();
    /// assert_eq!(result, json!(["alice", "bob", "charlie"]));
    /// ```
    pub fn evaluate(&self, expression: &str, input: &Value) -> Result<Value> {
        let expr = self
            .runtime
            .compile(expression)
            .map_err(|e| EngineError::InvalidExpression(e.to_string()))?;

        // Convert input Value to Variable for jmespath
        let var = jmespath::Variable::from_json(&input.to_string())
            .map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let result = expr
            .search(&var)
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        // Convert Rcvar to Value
        let value: Value = serde_json::to_value(result.as_ref())
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        Ok(value)
    }

    /// Evaluates a JMESPath expression against a JSON string.
    ///
    /// Convenience method that parses the JSON string before evaluation.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::InvalidJson`] if the input is not valid JSON,
    /// or evaluation errors as with [`evaluate`](Self::evaluate).
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    /// let result = engine.evaluate_str("length(@)", r#"[1, 2, 3, 4, 5]"#).unwrap();
    /// assert_eq!(result, json!(5));
    /// ```
    pub fn evaluate_str(&self, expression: &str, input: &str) -> Result<Value> {
        let json: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        self.evaluate(expression, &json)
    }

    /// Evaluates multiple expressions against the same input.
    ///
    /// Useful for extracting multiple values from a document in one call.
    /// Each expression is evaluated independently; failures don't affect other expressions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    /// let input = json!({"name": "alice", "age": 30, "active": true});
    ///
    /// let exprs = vec![
    ///     "name".to_string(),
    ///     "age".to_string(),
    ///     "missing".to_string(),  // Returns null, not an error
    /// ];
    ///
    /// let results = engine.batch_evaluate(&exprs, &input);
    /// assert_eq!(results.results[0].result, Some(json!("alice")));
    /// assert_eq!(results.results[1].result, Some(json!(30)));
    /// assert_eq!(results.results[2].result, Some(json!(null)));
    /// ```
    pub fn batch_evaluate(&self, expressions: &[String], input: &Value) -> BatchEvaluateResult {
        let results = expressions
            .iter()
            .map(|expr| match self.evaluate(expr, input) {
                Ok(result) => BatchExpressionResult {
                    expression: expr.clone(),
                    result: Some(result),
                    error: None,
                },
                Err(e) => BatchExpressionResult {
                    expression: expr.clone(),
                    result: None,
                    error: Some(e.to_string()),
                },
            })
            .collect();

        BatchEvaluateResult { results }
    }

    /// Validates a JMESPath expression without evaluating it.
    ///
    /// Checks if an expression has valid syntax without needing input data.
    /// Useful for validating user-provided expressions before storing them.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Valid expression
    /// let result = engine.validate("users[*].name | sort(@)");
    /// assert!(result.valid);
    /// assert!(result.error.is_none());
    ///
    /// // Invalid expression (unclosed bracket)
    /// let result = engine.validate("users[*.name");
    /// assert!(!result.valid);
    /// assert!(result.error.is_some());
    /// ```
    pub fn validate(&self, expression: &str) -> ValidationResult {
        match jmespath::compile(expression) {
            Ok(_) => ValidationResult {
                valid: true,
                error: None,
            },
            Err(e) => ValidationResult {
                valid: false,
                error: Some(e.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_evaluate() {
        let engine = JpxEngine::new();
        let input = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
        let result = engine.evaluate("users[*].name", &input).unwrap();
        assert_eq!(result, json!(["alice", "bob"]));
    }

    #[test]
    fn test_evaluate_str() {
        let engine = JpxEngine::new();
        let result = engine.evaluate_str("length(@)", r#"[1, 2, 3]"#).unwrap();
        assert_eq!(result, json!(3));
    }

    #[test]
    fn test_batch_evaluate() {
        let engine = JpxEngine::new();
        let input = json!({"a": 1, "b": 2});
        let exprs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = engine.batch_evaluate(&exprs, &input);

        assert_eq!(result.results.len(), 3);
        assert_eq!(result.results[0].result, Some(json!(1)));
        assert_eq!(result.results[1].result, Some(json!(2)));
        assert_eq!(result.results[2].result, Some(json!(null)));
    }

    #[test]
    fn test_validate() {
        let engine = JpxEngine::new();

        let valid = engine.validate("users[*].name");
        assert!(valid.valid);
        assert!(valid.error.is_none());

        let invalid = engine.validate("users[*.name");
        assert!(!invalid.valid);
        assert!(invalid.error.is_some());
    }
}
