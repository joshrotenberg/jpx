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

        if self.strict && crate::explain::has_let_nodes(expr.as_ast()) {
            return Err(EngineError::InvalidExpression(
                "Let expressions are not available in strict mode (standard JMESPath only). \
                 Remove --strict to use let expressions."
                    .to_string(),
            ));
        }

        let result = expr
            .search(input)
            .map_err(|e| EngineError::evaluation_failed(e.to_string()))?;

        Ok(result)
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
        match jpx_core::compile(expression) {
            Ok(expr) => {
                if self.strict {
                    // Reject let expressions in strict mode
                    if crate::explain::has_let_nodes(expr.as_ast()) {
                        return ValidationResult {
                            valid: false,
                            error: Some(
                                "Let expressions are not available in strict mode \
                                 (standard JMESPath only)."
                                    .to_string(),
                            ),
                        };
                    }

                    // Reject extension functions in strict mode
                    let func_names = crate::explain::collect_function_names(expr.as_ast());
                    let extension_fns: Vec<&String> = func_names
                        .iter()
                        .filter(|name| {
                            self.registry
                                .get_function(name)
                                .is_some_and(|f| !f.is_standard)
                        })
                        .collect();
                    if !extension_fns.is_empty() {
                        let names = extension_fns
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return ValidationResult {
                            valid: false,
                            error: Some(format!(
                                "Extension function(s) not available in strict mode: {names}. \
                                 Only standard JMESPath functions are allowed."
                            )),
                        };
                    }
                }

                ValidationResult {
                    valid: true,
                    error: None,
                }
            }
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

    #[test]
    fn test_evaluate_extension_function() {
        let engine = JpxEngine::new();
        let result = engine
            .evaluate("upper(name)", &json!({"name": "alice"}))
            .unwrap();
        assert_eq!(result, json!("ALICE"));
    }

    #[test]
    fn test_evaluate_strict_rejects_extensions() {
        let engine = JpxEngine::strict();
        let result = engine.evaluate("upper(name)", &json!({"name": "alice"}));
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::EngineError::EvaluationFailed { .. })
        ));
    }

    #[test]
    fn test_evaluate_invalid_expression() {
        let engine = JpxEngine::new();
        let result = engine.evaluate("users[*.name", &json!({}));
        assert!(matches!(
            result,
            Err(crate::EngineError::InvalidExpression(_))
        ));
    }

    #[test]
    fn test_evaluate_str_invalid_json() {
        let engine = JpxEngine::new();
        let result = engine.evaluate_str("@", "not json");
        assert!(matches!(result, Err(crate::EngineError::InvalidJson(_))));
    }

    #[test]
    fn test_batch_evaluate_with_errors() {
        let engine = JpxEngine::new();
        let exprs = vec!["a".to_string(), "invalid[".to_string()];
        let result = engine.batch_evaluate(&exprs, &json!({"a": 1}));

        assert_eq!(result.results.len(), 2);
        assert!(result.results[0].result.is_some());
        assert!(result.results[0].error.is_none());
        assert!(result.results[1].result.is_none());
        assert!(result.results[1].error.is_some());
    }

    #[test]
    fn test_evaluate_unicode() {
        let engine = JpxEngine::new();
        let input = json!({"name": "ñoño", "greeting": "こんにちは"});

        let result = engine.evaluate("name", &input).unwrap();
        assert_eq!(result, json!("ñoño"));

        let result = engine.evaluate("greeting", &input).unwrap();
        assert_eq!(result, json!("こんにちは"));
    }

    #[test]
    fn test_evaluate_deeply_nested() {
        let engine = JpxEngine::new();
        let input = json!({"a": {"b": {"c": {"d": {"e": "deep"}}}}});

        let result = engine.evaluate("a.b.c.d.e", &input).unwrap();
        assert_eq!(result, json!("deep"));

        let result = engine.evaluate("a.b.c.d", &input).unwrap();
        assert_eq!(result, json!({"e": "deep"}));
    }

    #[test]
    fn test_evaluate_null_result() {
        let engine = JpxEngine::new();
        let input = json!({"a": 1, "b": "hello"});

        let result = engine.evaluate("missing", &input).unwrap();
        assert_eq!(result, json!(null));

        let result = engine.evaluate("a.b.c", &input).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_batch_evaluate_large() {
        let engine = JpxEngine::new();
        let input = json!({"value": 42});
        let exprs: Vec<String> = (0..50).map(|_| "value".to_string()).collect();

        let result = engine.batch_evaluate(&exprs, &input);
        assert_eq!(result.results.len(), 50);
        for r in &result.results {
            assert_eq!(r.result, Some(json!(42)));
            assert!(r.error.is_none());
        }
    }

    #[test]
    fn test_strict_rejects_let_expression() {
        let engine = JpxEngine::strict();
        let result = engine.evaluate("let $x = name in $x", &json!({"name": "alice"}));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("strict mode"), "error was: {err}");
    }

    #[test]
    fn test_non_strict_allows_let_expression() {
        let engine = JpxEngine::new();
        let result = engine
            .evaluate("let $x = name in $x", &json!({"name": "alice"}))
            .unwrap();
        assert_eq!(result, json!("alice"));
    }

    #[test]
    fn test_batch_evaluate_empty() {
        let engine = JpxEngine::new();
        let input = json!({"a": 1});
        let exprs: Vec<String> = vec![];

        let result = engine.batch_evaluate(&exprs, &input);
        assert!(result.results.is_empty());
    }

    #[test]
    fn test_validate_complex_valid() {
        let engine = JpxEngine::new();

        let result = engine.validate("users[?age > `30`].name | sort(@) | join(', ', @)");
        assert!(result.valid);
        assert!(result.error.is_none());

        let result = engine.validate("items[*].{id: id, name: name} | [?id > `5`]");
        assert!(result.valid);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_validate_strict_rejects_let_expression() {
        let engine = JpxEngine::strict();
        let result = engine.validate("let $x = name in $x");
        assert!(!result.valid);
        let err = result.error.unwrap();
        assert!(err.contains("strict mode"), "error was: {err}");
        assert!(err.contains("Let expression"), "error was: {err}");
    }

    #[test]
    fn test_validate_strict_rejects_extension_function() {
        let engine = JpxEngine::strict();
        let result = engine.validate("upper(name)");
        assert!(!result.valid);
        let err = result.error.unwrap();
        assert!(err.contains("strict mode"), "error was: {err}");
        assert!(err.contains("upper"), "error was: {err}");
    }

    #[test]
    fn test_validate_strict_allows_standard_functions() {
        let engine = JpxEngine::strict();
        let result = engine.validate("length(sort(@))");
        assert!(result.valid, "error: {:?}", result.error);
    }

    #[test]
    fn test_validate_non_strict_allows_all() {
        let engine = JpxEngine::new();

        let result = engine.validate("upper(name)");
        assert!(result.valid, "error: {:?}", result.error);

        let result = engine.validate("let $x = name in $x");
        assert!(result.valid, "error: {:?}", result.error);
    }
}
