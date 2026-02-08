//! Common types for engine requests and responses.
//!
//! These types are used for structured input/output, particularly useful
//! when building APIs or serializing results.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request to evaluate a JMESPath expression.
///
/// This struct packages an expression with its input data, useful for
/// API endpoints or batch processing.
///
/// # Example
///
/// ```rust
/// use jpx_engine::EvalRequest;
/// use serde_json::json;
///
/// let request = EvalRequest {
///     expression: "users[*].name".to_string(),
///     input: json!({"users": [{"name": "alice"}]}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRequest {
    /// The JMESPath expression to evaluate
    pub expression: String,
    /// The JSON input to evaluate against
    pub input: Value,
}

/// Response from evaluating a JMESPath expression.
///
/// Wraps the evaluation result in a structured response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResponse {
    /// The result of evaluation
    pub result: Value,
}

/// Result of validating a JMESPath expression.
///
/// Returned by [`JpxEngine::validate`](crate::JpxEngine::validate) to indicate
/// whether an expression has valid syntax.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
///
/// let result = engine.validate("users[*].name");
/// assert!(result.valid);
/// assert!(result.error.is_none());
///
/// let result = engine.validate("users[*.name");  // missing bracket
/// assert!(!result.valid);
/// assert!(result.error.is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// `true` if the expression has valid syntax
    pub valid: bool,
    /// Error message if validation failed, `None` if valid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for a single expression in batch evaluation.
///
/// Each expression in a batch produces one of these, containing either
/// a successful result or an error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExpressionResult {
    /// The expression that was evaluated
    pub expression: String,
    /// The result if evaluation succeeded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error message if evaluation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result of batch evaluation.
///
/// Contains results for all expressions evaluated in
/// [`JpxEngine::batch_evaluate`](crate::JpxEngine::batch_evaluate).
/// Results are in the same order as the input expressions.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
/// use serde_json::json;
///
/// let engine = JpxEngine::new();
/// let input = json!({"a": 1, "b": 2});
/// let exprs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
///
/// let batch = engine.batch_evaluate(&exprs, &input);
///
/// // Results are in order
/// assert_eq!(batch.results.len(), 3);
/// assert_eq!(batch.results[0].expression, "a");
/// assert_eq!(batch.results[0].result, Some(json!(1)));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEvaluateResult {
    /// Results for each expression, in order
    pub results: Vec<BatchExpressionResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eval_request_serde_roundtrip() {
        let request = EvalRequest {
            expression: "users[*].name".to_string(),
            input: json!({"users": [{"name": "alice"}]}),
        };

        let json_str = serde_json::to_string(&request).unwrap();
        let deserialized: EvalRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.expression, "users[*].name");
        assert_eq!(deserialized.input, json!({"users": [{"name": "alice"}]}));
    }

    #[test]
    fn test_eval_response_serde_roundtrip() {
        let response = EvalResponse {
            result: json!(["alice", "bob"]),
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let deserialized: EvalResponse = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.result, json!(["alice", "bob"]));
    }

    #[test]
    fn test_validation_result_valid_roundtrip() {
        let result = ValidationResult {
            valid: true,
            error: None,
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: ValidationResult = serde_json::from_str(&json_str).unwrap();

        assert!(deserialized.valid);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_validation_result_invalid_roundtrip() {
        let result = ValidationResult {
            valid: false,
            error: Some("unexpected token".to_string()),
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: ValidationResult = serde_json::from_str(&json_str).unwrap();

        assert!(!deserialized.valid);
        assert_eq!(deserialized.error.as_deref(), Some("unexpected token"));
    }

    #[test]
    fn test_validation_result_skip_none_error() {
        let result = ValidationResult {
            valid: true,
            error: None,
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let json_value: Value = serde_json::from_str(&json_str).unwrap();

        assert!(json_value.get("error").is_none());
    }

    #[test]
    fn test_batch_expression_result_success() {
        let result = BatchExpressionResult {
            expression: "a.b".to_string(),
            result: Some(json!(42)),
            error: None,
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: BatchExpressionResult = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.expression, "a.b");
        assert_eq!(deserialized.result, Some(json!(42)));
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_batch_expression_result_error() {
        let result = BatchExpressionResult {
            expression: "invalid[".to_string(),
            result: None,
            error: Some("parse error".to_string()),
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: BatchExpressionResult = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.expression, "invalid[");
        assert!(deserialized.result.is_none());
        assert_eq!(deserialized.error.as_deref(), Some("parse error"));
    }

    #[test]
    fn test_batch_expression_result_skip_none_fields() {
        let result = BatchExpressionResult {
            expression: "a".to_string(),
            result: Some(json!(1)),
            error: None,
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let json_value: Value = serde_json::from_str(&json_str).unwrap();

        assert!(json_value.get("expression").is_some());
        assert!(json_value.get("result").is_some());
        assert!(json_value.get("error").is_none());

        let error_result = BatchExpressionResult {
            expression: "bad".to_string(),
            result: None,
            error: Some("fail".to_string()),
        };

        let json_str = serde_json::to_string(&error_result).unwrap();
        let json_value: Value = serde_json::from_str(&json_str).unwrap();

        assert!(json_value.get("expression").is_some());
        assert!(json_value.get("result").is_none());
        assert!(json_value.get("error").is_some());
    }

    #[test]
    fn test_batch_evaluate_result_roundtrip() {
        let batch = BatchEvaluateResult {
            results: vec![
                BatchExpressionResult {
                    expression: "a".to_string(),
                    result: Some(json!(1)),
                    error: None,
                },
                BatchExpressionResult {
                    expression: "b".to_string(),
                    result: Some(json!(2)),
                    error: None,
                },
                BatchExpressionResult {
                    expression: "invalid[".to_string(),
                    result: None,
                    error: Some("parse error".to_string()),
                },
            ],
        };

        let json_str = serde_json::to_string(&batch).unwrap();
        let deserialized: BatchEvaluateResult = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.results.len(), 3);
        assert_eq!(deserialized.results[0].expression, "a");
        assert_eq!(deserialized.results[0].result, Some(json!(1)));
        assert_eq!(deserialized.results[1].expression, "b");
        assert_eq!(deserialized.results[1].result, Some(json!(2)));
        assert_eq!(deserialized.results[2].expression, "invalid[");
        assert!(deserialized.results[2].result.is_none());
        assert_eq!(
            deserialized.results[2].error.as_deref(),
            Some("parse error")
        );
    }

    #[test]
    fn test_eval_request_complex_input() {
        let request = EvalRequest {
            expression: "data.users[?age > `30`].name".to_string(),
            input: json!({
                "data": {
                    "users": [
                        {"name": "alice", "age": 25, "tags": ["admin", "user"]},
                        {"name": "bob", "age": 35, "tags": ["user"]},
                        {"name": "carol", "age": 40, "tags": []}
                    ],
                    "metadata": {
                        "count": 3,
                        "active": true,
                        "ratio": 0.75,
                        "nothing": null
                    }
                }
            }),
        };

        let json_str = serde_json::to_string(&request).unwrap();
        let deserialized: EvalRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.expression, "data.users[?age > `30`].name");
        assert_eq!(deserialized.input, request.input);
        assert_eq!(deserialized.input["data"]["users"][0]["name"], "alice");
        assert_eq!(deserialized.input["data"]["metadata"]["count"], 3);
        assert_eq!(deserialized.input["data"]["metadata"]["active"], true);
        assert_eq!(deserialized.input["data"]["metadata"]["ratio"], 0.75);
        assert!(deserialized.input["data"]["metadata"]["nothing"].is_null());
    }
}
