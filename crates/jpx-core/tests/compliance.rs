//! JMESPath compliance tests.
//!
//! Test cases are generated using build.rs from the JSON test suites in tests/compliance/.

use std::fmt;

use serde_json::Value;

use jpx_core::{ErrorReason, RuntimeError, compile};

/// Available error types in compliance tests.
pub enum ErrorType {
    InvalidArity,
    InvalidType,
    InvalidSlice,
    UnknownFunction,
    SyntaxError,
}

impl ErrorType {
    fn from_json(error_type: &Value) -> Result<Self, TestCaseError> {
        error_type
            .as_str()
            .ok_or(TestCaseError::ErrorIsNotString)
            .and_then(|b| match b {
                "syntax" => Ok(ErrorType::SyntaxError),
                "invalid-type" => Ok(ErrorType::InvalidType),
                "invalid-value" => Ok(ErrorType::InvalidSlice),
                "invalid-arity" => Ok(ErrorType::InvalidArity),
                "unknown-function" => Ok(ErrorType::UnknownFunction),
                e => Err(TestCaseError::UnknownErrorType(e.to_string())),
            })
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ErrorType::InvalidArity => write!(fmt, "invalid-arity"),
            ErrorType::InvalidType => write!(fmt, "invalid-type"),
            ErrorType::InvalidSlice => write!(fmt, "invalid-value"),
            ErrorType::UnknownFunction => write!(fmt, "unknown-function"),
            ErrorType::SyntaxError => write!(fmt, "syntax"),
        }
    }
}

/// Test case assertions.
pub enum Assertion {
    Error(ErrorType),
    ValidResult(Value),
}

impl Assertion {
    pub fn assert(&self, suite: &str, case: &TestCase, given: &Value) -> Result<(), String> {
        match self {
            Assertion::ValidResult(expected_result) => {
                let expr = self.try_parse(suite, case)?;
                match expr.search(given) {
                    Err(e) => Err(self.err_message(suite, case, format!("{e}"))),
                    Ok(r) => {
                        if values_equal(&r, expected_result) {
                            Ok(())
                        } else {
                            Err(self.err_message(
                                suite,
                                case,
                                format!("got {:?}, expected {:?}", r, expected_result),
                            ))
                        }
                    }
                }
            }
            Assertion::Error(error_type) => {
                let result = self.try_parse(suite, case);
                match error_type {
                    ErrorType::InvalidArity => match result?.search(given).map_err(|e| e.reason) {
                        Err(ErrorReason::Runtime(RuntimeError::NotEnoughArguments { .. })) => {
                            Ok(())
                        }
                        Err(ErrorReason::Runtime(RuntimeError::TooManyArguments { .. })) => Ok(()),
                        Err(e) => Err(self.err_message(suite, case, format!("{e}"))),
                        Ok(r) => Err(self.err_message(suite, case, r.to_string())),
                    },
                    ErrorType::InvalidType => match result?.search(given).map_err(|e| e.reason) {
                        Err(ErrorReason::Runtime(RuntimeError::InvalidType { .. })) => Ok(()),
                        Err(ErrorReason::Runtime(RuntimeError::InvalidReturnType { .. })) => Ok(()),
                        Err(e) => Err(self.err_message(suite, case, format!("{e}"))),
                        Ok(r) => Err(self.err_message(suite, case, r.to_string())),
                    },
                    ErrorType::InvalidSlice => match result?.search(given).map_err(|e| e.reason) {
                        Err(ErrorReason::Runtime(RuntimeError::InvalidSlice)) => Ok(()),
                        Err(e) => Err(self.err_message(suite, case, format!("{e}"))),
                        Ok(r) => Err(self.err_message(suite, case, r.to_string())),
                    },
                    ErrorType::UnknownFunction => {
                        match result?.search(given).map_err(|e| e.reason) {
                            Err(ErrorReason::Runtime(RuntimeError::UnknownFunction(_))) => Ok(()),
                            Err(e) => Err(self.err_message(suite, case, format!("{e}"))),
                            Ok(r) => Err(self.err_message(suite, case, r.to_string())),
                        }
                    }
                    ErrorType::SyntaxError => match result {
                        Err(_) => Ok(()),
                        Ok(expr) => Err(self.err_message(suite, case, format!("Parsed {expr:?}"))),
                    },
                }
            }
        }
    }

    fn try_parse(&self, suite: &str, case: &TestCase) -> Result<jpx_core::Expression<'_>, String> {
        match compile(&case.expression) {
            Err(e) => Err(self.err_message(suite, case, format!("{e}"))),
            Ok(expr) => Ok(expr),
        }
    }

    fn err_message(&self, suite: &str, case: &TestCase, message: String) -> String {
        format!(
            "Test suite: {}\nExpression: {}\nAssertion: {}\nResult: {}\n==============",
            suite, case.expression, self, message
        )
    }
}

impl fmt::Display for Assertion {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Assertion::Error(e) => write!(fmt, "expects error({e})"),
            Assertion::ValidResult(r) => write!(fmt, "expects result({r})"),
        }
    }
}

pub enum TestCaseError {
    InvalidJSON(String),
    NoCaseType,
    NoExpression,
    ExpressionIsNotString,
    ErrorIsNotString,
    UnknownErrorType(String),
}

impl fmt::Display for TestCaseError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TestCaseError::InvalidJSON(msg) => write!(fmt, "invalid test case JSON: {msg}"),
            TestCaseError::NoCaseType => write!(fmt, "case has no result, error, or bench"),
            TestCaseError::NoExpression => write!(fmt, "test case has no expression key"),
            TestCaseError::ExpressionIsNotString => {
                write!(fmt, "test case expression is not a string")
            }
            TestCaseError::ErrorIsNotString => {
                write!(fmt, "test case error value is not a string")
            }
            TestCaseError::UnknownErrorType(t) => write!(fmt, "unknown error type: {t}"),
        }
    }
}

impl fmt::Debug for TestCaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

pub struct TestCase {
    pub expression: String,
    pub assertion: Assertion,
}

impl TestCase {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(case: &str) -> Result<TestCase, TestCaseError> {
        serde_json::from_str::<Value>(case)
            .map_err(|e| TestCaseError::InvalidJSON(e.to_string()))
            .and_then(|json| TestCase::from_json(&json))
    }

    fn from_json(case: &Value) -> Result<TestCase, TestCaseError> {
        let case = case
            .as_object()
            .ok_or(TestCaseError::InvalidJSON("not an object".to_string()))?;
        Ok(TestCase {
            expression: case
                .get("expression")
                .ok_or(TestCaseError::NoExpression)
                .and_then(|expression| {
                    expression
                        .as_str()
                        .ok_or(TestCaseError::ExpressionIsNotString)
                        .map(|expression_str| expression_str.to_string())
                })?,
            assertion: match case.get("error") {
                Some(err) => Assertion::Error(ErrorType::from_json(err)?),
                None if case.contains_key("result") => {
                    Assertion::ValidResult(case.get("result").unwrap().clone())
                }
                None if case.contains_key("bench") => {
                    // Skip benchmark cases
                    return Err(TestCaseError::NoCaseType);
                }
                _ => return Err(TestCaseError::NoCaseType),
            },
        })
    }

    pub fn assert(&self, suite_filename: &str, given: &Value) -> Result<(), String> {
        self.assertion.assert(suite_filename, self, given)
    }
}

/// Deep equality comparison that handles integer/float comparison.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            a.as_f64().zip(b.as_f64()).is_some_and(|(af, bf)| af == bf)
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|bv| values_equal(v, bv)))
        }
        _ => false,
    }
}

include!(concat!(env!("OUT_DIR"), "/compliance_tests.rs"));
