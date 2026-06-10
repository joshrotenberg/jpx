//! Error types for the jpx engine.
//!
//! This module defines the error types used throughout the engine.
//! All public methods that can fail return [`Result<T>`](Result).
//!
//! # Error Handling
//!
//! ```rust
//! use jpx_engine::{JpxEngine, EngineError, EvaluationErrorKind};
//!
//! let engine = JpxEngine::new();
//!
//! // Handle specific error types
//! match engine.evaluate("invalid[", &serde_json::json!({})) {
//!     Ok(result) => println!("Result: {}", result),
//!     Err(EngineError::InvalidExpression(msg)) => {
//!         eprintln!("Syntax error: {}", msg);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! # Structured Evaluation Errors
//!
//! [`EvaluationFailed`](EngineError::EvaluationFailed) carries an
//! [`EvaluationErrorKind`] that lets consumers match on specific failure modes
//! without parsing error strings:
//!
//! ```rust
//! use jpx_engine::{JpxEngine, EngineError, EvaluationErrorKind};
//!
//! let engine = JpxEngine::strict();
//!
//! match engine.evaluate("sum(@)", &serde_json::json!({})) {
//!     Err(EngineError::EvaluationFailed { kind: EvaluationErrorKind::UndefinedFunction { ref name }, .. }) => {
//!         println!("Unknown function: {}", name);
//!     }
//!     _ => {}
//! }
//! ```

use std::fmt;
use thiserror::Error;

/// Classifies the specific failure mode of an evaluation error.
///
/// This allows consumers to programmatically handle different error types
/// without parsing error message strings.
///
/// Marked `#[non_exhaustive]`: new failure modes may be added in future
/// releases, so downstream matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvaluationErrorKind {
    /// Expression called a function that is not defined.
    ///
    /// Common when using extension functions in strict mode, or typos
    /// in function names.
    UndefinedFunction {
        /// The function name that was not found.
        name: String,
    },
    /// Wrong number of arguments passed to a function.
    ArgumentCount {
        /// Expected number of arguments (if parseable).
        expected: Option<u32>,
        /// Actual number of arguments (if parseable).
        actual: Option<u32>,
    },
    /// Argument type does not match what the function expects.
    TypeError {
        /// Description of the type mismatch.
        detail: String,
    },
    /// Any other evaluation failure.
    Other,
}

impl fmt::Display for EvaluationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluationErrorKind::UndefinedFunction { name } => {
                write!(f, "undefined function '{}'", name)
            }
            EvaluationErrorKind::ArgumentCount {
                expected: Some(exp),
                actual: Some(act),
            } => write!(f, "expected {} arguments, found {}", exp, act),
            EvaluationErrorKind::ArgumentCount { .. } => write!(f, "wrong number of arguments"),
            EvaluationErrorKind::TypeError { detail } => write!(f, "type error: {}", detail),
            EvaluationErrorKind::Other => write!(f, "evaluation error"),
        }
    }
}

/// Parse a jpx-core runtime error message into a structured kind.
///
/// The jpx-core runtime produces predictable error message patterns:
/// - `"Call to undefined function <name>"` for unknown functions
/// - `"Too many arguments: expected <n>, found <m>"` for argument count errors
/// - `"Not enough arguments: expected <n>, found <m>"` for argument count errors
/// - `"Argument <n> expects type <expected>, given <actual>"` for type errors
pub(crate) fn classify_evaluation_error(message: &str) -> EvaluationErrorKind {
    // "Call to undefined function <name>"
    if let Some(rest) = message
        .strip_prefix("Runtime error: Call to undefined function ")
        .or_else(|| message.strip_prefix("Call to undefined function "))
    {
        // The name is the next word (up to space or parens or end)
        let name = rest.split([' ', '(']).next().unwrap_or(rest).to_string();
        return EvaluationErrorKind::UndefinedFunction { name };
    }

    // "Too many arguments: expected <n>, found <m>"
    // "Not enough arguments: expected <n>, found <m>"
    if message.contains("arguments: expected") {
        let expected = extract_number_after(message, "expected ");
        let actual = extract_number_after(message, "found ");
        return EvaluationErrorKind::ArgumentCount { expected, actual };
    }

    // "Argument <n> expects type <expected>, given <actual>"
    if message.contains("expects type") {
        let detail = message
            .strip_prefix("Runtime error: ")
            .unwrap_or(message)
            .to_string();
        return EvaluationErrorKind::TypeError { detail };
    }

    EvaluationErrorKind::Other
}

/// Extract the first number after a prefix string.
fn extract_number_after(s: &str, prefix: &str) -> Option<u32> {
    let idx = s.find(prefix)?;
    let after = &s[idx + prefix.len()..];
    let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

/// Errors that can occur during engine operations.
///
/// Each variant represents a specific failure mode, making it easy to
/// handle different error types appropriately.
///
/// Marked `#[non_exhaustive]`: new failure modes may be added in future
/// releases, so downstream matches should include a wildcard arm.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EngineError {
    /// JMESPath expression has invalid syntax.
    ///
    /// Returned when [`JpxEngine::evaluate`](crate::JpxEngine::evaluate) or
    /// [`JpxEngine::validate`](crate::JpxEngine::validate) encounters a
    /// malformed expression.
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    /// JSON input could not be parsed.
    ///
    /// Returned when [`JpxEngine::evaluate_str`](crate::JpxEngine::evaluate_str)
    /// or similar methods receive invalid JSON.
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    /// Expression evaluation failed at runtime.
    ///
    /// Carries a [`kind`](EvaluationErrorKind) field for programmatic matching
    /// on specific failure modes (undefined function, argument errors, type errors).
    #[error("Evaluation failed: {message}")]
    EvaluationFailed {
        /// Human-readable error message.
        message: String,
        /// Structured classification of the error.
        kind: EvaluationErrorKind,
    },

    /// Requested function does not exist.
    ///
    /// Returned by introspection methods when a function name is not found.
    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    /// Requested stored query does not exist.
    ///
    /// Returned by [`JpxEngine::run_query`](crate::JpxEngine::run_query)
    /// when the named query hasn't been defined.
    #[error("Query not found: {0}")]
    QueryNotFound(String),

    /// Discovery registration failed.
    ///
    /// Returned when registering a discovery spec fails validation
    /// or conflicts with an existing registration.
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Configuration error (parse failure, invalid settings, etc.).
    ///
    /// Returned when loading or merging configuration files fails.
    #[error("Config error: {0}")]
    ConfigError(String),

    /// Internal error (lock poisoning, serialization failure, etc.).
    ///
    /// These errors indicate bugs or unexpected conditions and should
    /// generally be reported.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Arrow conversion error (only available with `arrow` feature).
    ///
    /// Returned when converting between Arrow RecordBatches and JSON fails.
    #[cfg(feature = "arrow")]
    #[error("Arrow error: {0}")]
    ArrowError(String),
}

impl EngineError {
    /// Create an `EvaluationFailed` error, automatically classifying the error kind
    /// from the message string.
    pub(crate) fn evaluation_failed(message: String) -> Self {
        let kind = classify_evaluation_error(&message);
        EngineError::EvaluationFailed { message, kind }
    }
}

/// A specialized Result type for engine operations.
///
/// This is defined as `std::result::Result<T, EngineError>` for convenience.
pub type Result<T> = std::result::Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_undefined_function() {
        let kind = classify_evaluation_error(
            "Runtime error: Call to undefined function foo_bar (line 0, column 7)",
        );
        assert_eq!(
            kind,
            EvaluationErrorKind::UndefinedFunction {
                name: "foo_bar".to_string()
            }
        );
    }

    #[test]
    fn test_classify_too_many_arguments() {
        let kind = classify_evaluation_error(
            "Runtime error: Too many arguments: expected 1, found 2 (line 0, column 6)",
        );
        assert_eq!(
            kind,
            EvaluationErrorKind::ArgumentCount {
                expected: Some(1),
                actual: Some(2)
            }
        );
    }

    #[test]
    fn test_classify_not_enough_arguments() {
        let kind = classify_evaluation_error(
            "Runtime error: Not enough arguments: expected 2, found 1 (line 0, column 4)",
        );
        assert_eq!(
            kind,
            EvaluationErrorKind::ArgumentCount {
                expected: Some(2),
                actual: Some(1)
            }
        );
    }

    #[test]
    fn test_classify_type_error() {
        let kind = classify_evaluation_error(
            "Runtime error: Argument 0 expects type array[number], given object (line 0, column 3)",
        );
        match kind {
            EvaluationErrorKind::TypeError { detail } => {
                assert!(detail.contains("expects type"));
            }
            other => panic!("Expected TypeError, got {:?}", other),
        }
    }

    #[test]
    fn test_classify_other() {
        let kind = classify_evaluation_error("Some unknown error");
        assert_eq!(kind, EvaluationErrorKind::Other);
    }

    // Display formatting tests for EvaluationErrorKind

    #[test]
    fn test_display_undefined_function() {
        let kind = EvaluationErrorKind::UndefinedFunction {
            name: "foo".to_string(),
        };
        assert_eq!(kind.to_string(), "undefined function 'foo'");
    }

    #[test]
    fn test_display_argument_count_with_numbers() {
        let kind = EvaluationErrorKind::ArgumentCount {
            expected: Some(2),
            actual: Some(3),
        };
        assert_eq!(kind.to_string(), "expected 2 arguments, found 3");
    }

    #[test]
    fn test_display_argument_count_without_numbers() {
        let kind = EvaluationErrorKind::ArgumentCount {
            expected: None,
            actual: None,
        };
        assert_eq!(kind.to_string(), "wrong number of arguments");
    }

    #[test]
    fn test_display_type_error() {
        let kind = EvaluationErrorKind::TypeError {
            detail: "some detail".to_string(),
        };
        assert_eq!(kind.to_string(), "type error: some detail");
    }

    #[test]
    fn test_display_other() {
        let kind = EvaluationErrorKind::Other;
        assert_eq!(kind.to_string(), "evaluation error");
    }

    // EngineError Display tests

    #[test]
    fn test_engine_error_display_invalid_expression() {
        let err = EngineError::InvalidExpression("unexpected token".to_string());
        assert_eq!(err.to_string(), "Invalid expression: unexpected token");
    }

    #[test]
    fn test_engine_error_display_invalid_json() {
        let err = EngineError::InvalidJson("expected value at line 1".to_string());
        assert_eq!(err.to_string(), "Invalid JSON: expected value at line 1");
    }

    #[test]
    fn test_engine_error_display_evaluation_failed() {
        let err = EngineError::EvaluationFailed {
            message: "something went wrong".to_string(),
            kind: EvaluationErrorKind::Other,
        };
        assert_eq!(err.to_string(), "Evaluation failed: something went wrong");
    }

    #[test]
    fn test_engine_error_display_all_variants() {
        let unknown = EngineError::UnknownFunction("mystery".to_string());
        assert_eq!(unknown.to_string(), "Unknown function: mystery");

        let not_found = EngineError::QueryNotFound("my_query".to_string());
        assert_eq!(not_found.to_string(), "Query not found: my_query");

        let reg_failed = EngineError::RegistrationFailed("duplicate name".to_string());
        assert_eq!(
            reg_failed.to_string(),
            "Registration failed: duplicate name"
        );

        let config = EngineError::ConfigError("missing field".to_string());
        assert_eq!(config.to_string(), "Config error: missing field");

        let internal = EngineError::Internal("lock poisoned".to_string());
        assert_eq!(internal.to_string(), "Internal error: lock poisoned");
    }

    // Constructor test

    #[test]
    fn test_evaluation_failed_constructor() {
        let err = EngineError::evaluation_failed("Call to undefined function foo".to_string());
        match err {
            EngineError::EvaluationFailed {
                ref message,
                ref kind,
            } => {
                assert_eq!(message, "Call to undefined function foo");
                assert_eq!(
                    *kind,
                    EvaluationErrorKind::UndefinedFunction {
                        name: "foo".to_string()
                    }
                );
            }
            other => panic!("Expected EvaluationFailed, got {:?}", other),
        }
    }
}
