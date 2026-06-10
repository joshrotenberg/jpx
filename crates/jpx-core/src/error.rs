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

    /// Returns the column number of the error (0-indexed, in characters).
    ///
    /// `offset` is a byte position, so the column is counted in characters
    /// since the last newline -- a byte count would misalign the `^` caret in
    /// [`Display`](Self) for expressions containing multibyte characters.
    pub fn column(&self) -> usize {
        let before = &self.expression[..self.offset.min(self.expression.len())];
        match before.rfind('\n') {
            Some(pos) => before[pos + 1..].chars().count(),
            None => before.chars().count(),
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
///
/// Marked `#[non_exhaustive]`: new reason variants may be added in future
/// releases, so downstream matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorReason {
    /// A parse-time error.
    #[error("Parse error: {0}")]
    Parse(String),
    /// A runtime error.
    #[error("Runtime error: {0}")]
    Runtime(RuntimeError),
}

/// Runtime errors that can occur during expression evaluation.
///
/// Marked `#[non_exhaustive]`: new runtime-error variants may be added in
/// future releases, so downstream matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
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
    /// Expression nesting exceeded the maximum evaluation depth.
    #[error("Recursion limit exceeded: maximum expression nesting depth is {limit}")]
    RecursionLimitExceeded {
        /// The maximum allowed nesting depth.
        limit: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_single_line_expression() {
        let err = JmespathError::new("foo.bar", 4, ErrorReason::Parse("test".into()));
        assert_eq!(err.line(), 1);
    }

    #[test]
    fn column_single_line_expression() {
        let err = JmespathError::new("foo.bar", 4, ErrorReason::Parse("test".into()));
        assert_eq!(err.column(), 4);
    }

    #[test]
    fn line_multi_line_expression() {
        let err = JmespathError::new("foo\nbar\nbaz", 8, ErrorReason::Parse("test".into()));
        assert_eq!(err.line(), 3);
    }

    #[test]
    fn column_multi_line_expression() {
        let err = JmespathError::new("foo\nbar\nbaz", 8, ErrorReason::Parse("test".into()));
        assert_eq!(err.column(), 0);
    }

    #[test]
    fn column_mid_second_line() {
        let err = JmespathError::new("foo\nbar.baz", 6, ErrorReason::Parse("test".into()));
        assert_eq!(err.line(), 2);
        assert_eq!(err.column(), 2);
    }

    #[test]
    fn offset_beyond_expression_length() {
        let err = JmespathError::new("foo", 100, ErrorReason::Parse("test".into()));
        // line() clamps to expression length, so still line 1
        assert_eq!(err.line(), 1);
        // column() clamps to the expression and counts characters (3 for "foo")
        assert_eq!(err.column(), 3);
    }

    #[test]
    fn column_counts_characters_not_bytes() {
        // "ä.x": 'ä' is 2 bytes, so the byte offset of 'x' is 3 but its column
        // (in characters, for caret alignment) is 2.
        let err = JmespathError::new("ä.x", 3, ErrorReason::Parse("test".into()));
        assert_eq!(err.column(), 2);
    }

    #[test]
    fn display_format_parse_error() {
        let err = JmespathError::new("foo.bar", 4, ErrorReason::Parse("bad token".into()));
        let display = format!("{err}");
        assert!(display.contains("Parse error: bad token"));
        assert!(display.contains("foo.bar"));
        assert!(display.contains("^"));
    }

    #[test]
    fn display_format_runtime_error() {
        let err = JmespathError::new(
            "foo()",
            0,
            ErrorReason::Runtime(RuntimeError::UnknownFunction("foo".into())),
        );
        let display = format!("{err}");
        assert!(display.contains("Runtime error"));
        assert!(display.contains("Unknown function: foo"));
    }

    #[test]
    fn runtime_error_invalid_slice_display() {
        let err = RuntimeError::InvalidSlice;
        assert_eq!(format!("{err}"), "Invalid slice: step cannot be 0");
    }

    #[test]
    fn runtime_error_too_many_args_display() {
        let err = RuntimeError::TooManyArguments {
            expected: 2,
            actual: 5,
        };
        assert_eq!(format!("{err}"), "Too many arguments: expected 2, got 5");
    }

    #[test]
    fn runtime_error_not_enough_args_display() {
        let err = RuntimeError::NotEnoughArguments {
            expected: 3,
            actual: 1,
        };
        assert_eq!(format!("{err}"), "Not enough arguments: expected 3, got 1");
    }

    #[test]
    fn runtime_error_invalid_type_display() {
        let err = RuntimeError::InvalidType {
            expected: "string".into(),
            actual: "number".into(),
            position: 0,
        };
        let display = format!("{err}");
        assert!(display.contains("expected string"));
        assert!(display.contains("got number"));
    }

    #[test]
    fn runtime_error_invalid_return_type_display() {
        let err = RuntimeError::InvalidReturnType {
            expected: "number".into(),
            actual: "string".into(),
            position: 1,
            invocation: 2,
        };
        let display = format!("{err}");
        assert!(display.contains("expected number"));
        assert!(display.contains("got string"));
        assert!(display.contains("position 1"));
        assert!(display.contains("invocation 2"));
    }

    #[test]
    fn error_reason_parse_display() {
        let reason = ErrorReason::Parse("unexpected token".into());
        assert_eq!(format!("{reason}"), "Parse error: unexpected token");
    }

    #[test]
    fn error_reason_runtime_display() {
        let reason = ErrorReason::Runtime(RuntimeError::InvalidSlice);
        assert!(format!("{reason}").contains("Invalid slice"));
    }

    #[test]
    fn from_ctx_uses_context_offset() {
        let runtime = crate::Runtime::new();
        let mut ctx = crate::Context::new("test_expr", &runtime);
        ctx.offset = 5;
        let err = JmespathError::from_ctx(&ctx, ErrorReason::Parse("test".into()));
        assert_eq!(err.offset, 5);
        assert_eq!(err.expression, "test_expr");
    }

    #[test]
    fn jmespath_error_implements_std_error() {
        let err = JmespathError::new("foo", 0, ErrorReason::Parse("test".into()));
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn jmespath_error_clone_and_eq() {
        let err = JmespathError::new("foo", 0, ErrorReason::Parse("test".into()));
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}
