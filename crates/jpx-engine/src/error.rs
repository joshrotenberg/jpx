//! Error types for the jpx engine.
//!
//! This module defines the error types used throughout the engine.
//! All public methods that can fail return [`Result<T>`](Result).
//!
//! # Error Handling
//!
//! ```rust
//! use jpx_engine::{JpxEngine, EngineError};
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

use thiserror::Error;

/// Errors that can occur during engine operations.
///
/// Each variant represents a specific failure mode, making it easy to
/// handle different error types appropriately.
#[derive(Debug, Error)]
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
    /// This can happen when calling undefined functions (in strict mode),
    /// type mismatches, or other runtime errors.
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

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

/// A specialized Result type for engine operations.
///
/// This is defined as `std::result::Result<T, EngineError>` for convenience.
pub type Result<T> = std::result::Result<T, EngineError>;
