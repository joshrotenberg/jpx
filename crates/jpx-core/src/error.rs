//! Error types for jpx-core.

use std::fmt;

/// A JMESPath error with position information.
#[derive(Debug, Clone, PartialEq)]
pub struct JmespathError {
    /// Character offset in the expression where the error occurred.
    pub offset: usize,
    /// The expression that caused the error.
    pub expression: String,
    /// The reason for the error.
    pub reason: ErrorReason,
}

impl JmespathError {
    /// Creates a new error.
    pub fn new(expression: &str, offset: usize, reason: ErrorReason) -> Self {
        Self {
            offset,
            expression: expression.to_owned(),
            reason,
        }
    }

    /// Creates an error from a Context, using its current offset and expression.
    pub fn from_ctx(ctx: &crate::Context<'_>, reason: ErrorReason) -> Self {
        Self {
            offset: ctx.offset,
            expression: ctx.expression.to_owned(),
            reason,
        }
    }

    /// Returns the line number of the error (1-indexed).
    pub fn line(&self) -> usize {
        self.expression[..self.offset.min(self.expression.len())]
            .chars()
            .filter(|c| *c == '\n')
            .count()
            + 1
    }

    /// Returns the column number of the error (0-indexed).
    pub fn column(&self) -> usize {
        let before = &self.expression[..self.offset.min(self.expression.len())];
        match before.rfind('\n') {
            Some(pos) => self.offset - pos - 1,
            None => self.offset,
        }
    }
}

impl fmt::Display for JmespathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col = self.column();
        write!(
            f,
            "{}\n{}\n{}",
            self.reason,
            self.expression,
            " ".repeat(col)
        )?;
        write!(f, "^")
    }
}

impl std::error::Error for JmespathError {}

/// The reason for a JMESPath error.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorReason {
    /// A parse-time error.
    Parse(String),
    /// A runtime error.
    Runtime(RuntimeError),
}

impl fmt::Display for ErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorReason::Parse(msg) => write!(f, "Parse error: {msg}"),
            ErrorReason::Runtime(err) => write!(f, "Runtime error: {err}"),
        }
    }
}

/// Runtime errors that can occur during expression evaluation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RuntimeError {
    /// A slice expression with step of 0.
    #[error("Invalid slice: step cannot be 0")]
    InvalidSlice,
    /// Too many arguments provided to a function.
    #[error("Too many arguments: expected {expected}, got {actual}")]
    TooManyArguments { expected: usize, actual: usize },
    /// Not enough arguments provided to a function.
    #[error("Not enough arguments: expected {expected}, got {actual}")]
    NotEnoughArguments { expected: usize, actual: usize },
    /// An unknown function was called.
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    /// Invalid type provided to a function.
    #[error("Invalid type at position {position}: expected {expected}, got {actual}")]
    InvalidType {
        expected: String,
        actual: String,
        position: usize,
    },
    /// Invalid return type from an expression reference.
    #[error(
        "Invalid return type at position {position}, invocation {invocation}: expected {expected}, got {actual}"
    )]
    InvalidReturnType {
        expected: String,
        actual: String,
        position: usize,
        invocation: usize,
    },
}
