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
