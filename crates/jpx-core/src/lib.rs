//! jpx-core: A complete JMESPath implementation using `serde_json::Value`.
//!
//! This crate provides a JMESPath parser, interpreter, and runtime that works
//! directly with `serde_json::Value`, eliminating the need for a separate
//! `Variable` type and the conversion overhead that comes with it.
//!
//! # Quick Start
//!
//! ```
//! use jpx_core::compile;
//! use serde_json::json;
//!
//! let expr = compile("foo.bar").unwrap();
//! let data = json!({"foo": {"bar": true}});
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result, json!(true));
//! ```

pub mod ast;
#[cfg(feature = "extensions")]
pub mod extensions;
pub mod functions;
pub mod query_library;
pub mod registry;
pub mod value_ext;

pub use crate::error::{ErrorReason, JmespathError, RuntimeError};
pub use crate::interpreter::SearchResult;
pub use crate::parser::{ParseResult, parse};
pub use crate::registry::{Category, Feature, FunctionInfo, FunctionRegistry};
pub use crate::runtime::{Runtime, RuntimeBuilder};
pub use crate::value_ext::{JmespathType, ValueExt};

mod error;
pub mod interpreter;
mod lexer;
mod parser;
mod runtime;

#[cfg(feature = "let-expr")]
use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

use serde_json::Value;

use crate::ast::Ast;
use crate::interpreter::interpret;

/// The default runtime with all 26 built-in JMESPath functions registered.
pub static DEFAULT_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    runtime
});

/// Compiles a JMESPath expression using the default Runtime.
#[inline]
pub fn compile(expression: &str) -> Result<Expression<'static>, JmespathError> {
    DEFAULT_RUNTIME.compile(expression)
}

/// A compiled JMESPath expression.
///
/// The compiled expression can be used multiple times without incurring
/// the cost of re-parsing the expression each time.
#[derive(Clone)]
pub struct Expression<'a> {
    ast: Ast,
    expression: String,
    runtime: &'a Runtime,
}

impl<'a> Expression<'a> {
    /// Creates a new JMESPath expression.
    #[inline]
    pub fn new<S>(expression: S, ast: Ast, runtime: &'a Runtime) -> Expression<'a>
    where
        S: Into<String>,
    {
        Expression {
            expression: expression.into(),
            ast,
            runtime,
        }
    }

    /// Searches data with the compiled expression.
    ///
    /// Takes a `&Value` and returns a `Value` directly -- no conversion needed.
    pub fn search(&self, data: &Value) -> SearchResult {
        let mut ctx = Context::new(&self.expression, self.runtime);
        let result = interpret(data, &self.ast, &mut ctx)?;
        // Strip expref sentinels from top-level results
        Ok(strip_expref_sentinels(result))
    }

    /// Returns the JMESPath expression string.
    pub fn as_str(&self) -> &str {
        &self.expression
    }

    /// Returns the AST of the parsed JMESPath expression.
    pub fn as_ast(&self) -> &Ast {
        &self.ast
    }
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> fmt::Debug for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<'a> PartialEq for Expression<'a> {
    fn eq(&self, other: &Expression<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

/// Context object used during expression evaluation.
///
/// The Context struct carries state needed by the interpreter and functions,
/// including the expression string (for error messages), runtime (for function
/// lookup), current AST offset, and expref side-channel table.
pub struct Context<'a> {
    /// Expression string being interpreted.
    pub expression: &'a str,
    /// JMESPath runtime used for function lookup.
    pub runtime: &'a Runtime,
    /// Current AST offset (for error reporting).
    pub offset: usize,
    /// Side-channel table for expression references.
    /// Exprefs are stored here and referenced by index in sentinel values.
    pub(crate) expref_table: Vec<Ast>,
    /// Current interpreter recursion depth, used to bound evaluation nesting
    /// and prevent stack overflow on deeply nested ASTs.
    pub(crate) eval_depth: usize,
    /// Variable scopes for let expressions (JEP-18).
    #[cfg(feature = "let-expr")]
    scopes: Vec<HashMap<String, Value>>,
}

impl<'a> Context<'a> {
    /// Creates a new context.
    #[inline]
    pub fn new(expression: &'a str, runtime: &'a Runtime) -> Context<'a> {
        Context {
            expression,
            runtime,
            offset: 0,
            expref_table: Vec::new(),
            eval_depth: 0,
            #[cfg(feature = "let-expr")]
            scopes: Vec::new(),
        }
    }

    /// Stores an expref AST and returns its index in the table.
    pub(crate) fn store_expref(&mut self, ast: Ast) -> usize {
        let id = self.expref_table.len();
        self.expref_table.push(ast);
        id
    }

    /// Retrieves an expref AST by index.
    pub fn get_expref(&self, id: usize) -> Option<&Ast> {
        self.expref_table.get(id)
    }

    /// Push a new scope onto the scope stack.
    #[cfg(feature = "let-expr")]
    #[inline]
    pub fn push_scope(&mut self, bindings: HashMap<String, Value>) {
        self.scopes.push(bindings);
    }

    /// Pop the innermost scope from the scope stack.
    #[cfg(feature = "let-expr")]
    #[inline]
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Look up a variable in the scope stack.
    #[cfg(feature = "let-expr")]
    #[inline]
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
}

/// Creates an expref sentinel value from a table index.
pub(crate) fn make_expref_sentinel(id: usize) -> Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "__jpx_expref__".to_string(),
        Value::Number(serde_json::Number::from(id)),
    );
    Value::Object(map)
}

/// Extracts the expref ID from a sentinel value.
pub fn get_expref_id(value: &Value) -> Option<usize> {
    value
        .as_object()
        .and_then(|m| m.get("__jpx_expref__"))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
}

/// Strips expref sentinels from a value (recursive for arrays/objects).
fn strip_expref_sentinels(value: Value) -> Value {
    match value {
        Value::Object(map) if map.contains_key("__jpx_expref__") => Value::Null,
        Value::Array(arr) => Value::Array(arr.into_iter().map(strip_expref_sentinels).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, strip_expref_sentinels(v)))
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_expression_as_string_or_debug() {
        let expr = compile("foo | baz").unwrap();
        assert_eq!("foo | baz/foo | baz", format!("{expr}/{expr:?}"));
    }

    #[test]
    fn implements_partial_eq() {
        let a = compile("@").unwrap();
        let b = compile("@").unwrap();
        assert!(a == b);
    }

    #[test]
    fn can_evaluate_jmespath_expression() {
        let expr = compile("foo.bar").unwrap();
        let data = json!({"foo": {"bar": true}});
        assert_eq!(json!(true), expr.search(&data).unwrap());
    }

    #[test]
    fn can_get_expression_ast() {
        let expr = compile("foo").unwrap();
        assert_eq!(
            &Ast::Field {
                offset: 0,
                name: "foo".to_string(),
            },
            expr.as_ast()
        );
    }

    #[test]
    fn expression_clone() {
        let expr = compile("foo").unwrap();
        let _ = expr.clone();
    }

    #[test]
    fn test_invalid_number() {
        let _ = compile("6455555524");
    }
}

#[cfg(all(test, feature = "let-expr"))]
mod let_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_let_expression() {
        let expr = compile("let $x = `1` in $x").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(1));
    }

    #[test]
    fn test_let_with_data_reference() {
        let expr = compile("let $name = name in $name").unwrap();
        let data = json!({"name": "Alice"});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("Alice"));
    }

    #[test]
    fn test_let_multiple_bindings() {
        let expr = compile("let $a = `1`, $b = `2` in [$a, $b]").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([1, 2]));
    }

    #[test]
    fn test_let_with_expression_body() {
        let expr = compile("let $items = items in $items[0].name").unwrap();
        let data = json!({"items": [{"name": "first"}, {"name": "second"}]});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("first"));
    }

    #[test]
    fn test_nested_let() {
        let expr = compile("let $x = `1` in let $y = `2` in [$x, $y]").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([1, 2]));
    }

    #[test]
    fn test_let_variable_shadowing() {
        let expr = compile("let $x = `1` in let $x = `2` in $x").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(2));
    }

    #[test]
    fn test_undefined_variable_error() {
        let expr = compile("$undefined").unwrap();
        let data = json!({});
        let result = expr.search(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_let_in_projection() {
        let expr = compile("let $threshold = `50` in numbers[? @ > $threshold]").unwrap();
        let data = json!({"numbers": [10, 30, 50, 70, 90]});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([70, 90]));
    }

    #[test]
    fn test_let_variable_used_multiple_times() {
        let expr = compile("let $foo = foo.bar in [$foo, $foo]").unwrap();
        let data = json!({"foo": {"bar": "baz"}});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["baz", "baz"]));
    }

    #[test]
    fn test_let_shadowing_in_projection() {
        let expr = compile("let $a = a in b[*].[a, $a, let $a = 'shadow' in $a]").unwrap();
        let data = json!({"a": "topval", "b": [{"a": "inner1"}, {"a": "inner2"}]});
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!([
                ["inner1", "topval", "shadow"],
                ["inner2", "topval", "shadow"]
            ])
        );
    }

    #[test]
    fn test_let_bindings_evaluated_in_outer_scope() {
        let expr = compile("let $a = 'top-a' in let $a = 'in-a', $b = $a in $b").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("top-a"));
    }

    #[test]
    fn test_let_projection_stopping() {
        let expr = compile("let $foo = foo[*] in $foo[0]").unwrap();
        let data = json!({"foo": [[0, 1], [2, 3], [4, 5]]});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([0, 1]));
    }

    #[test]
    fn test_let_shadow_and_restore() {
        let expr = compile("let $x = 'outer' in [let $x = 'inner' in $x, $x]").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["inner", "outer"]));
    }

    #[test]
    fn test_let_with_functions() {
        let expr = compile("let $arr = numbers in length($arr)").unwrap();
        let data = json!({"numbers": [1, 2, 3, 4, 5]});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(5));
    }

    #[test]
    fn test_let_deeply_nested_scopes() {
        let expr = compile("let $a = `1` in let $b = `2` in let $c = `3` in [$a, $b, $c]").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([1, 2, 3]));
    }

    #[test]
    fn test_let_with_flatten() {
        let expr = compile("let $data = nested in $data[].items[]").unwrap();
        let data = json!({"nested": [{"items": [1, 2]}, {"items": [3, 4]}]});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
    }

    #[test]
    fn test_let_with_slice() {
        let expr = compile("let $arr = numbers in $arr[1:3]").unwrap();
        let data = json!({"numbers": [0, 1, 2, 3, 4]});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!([1, 2]));
    }

    #[test]
    fn test_let_with_or_expression() {
        let expr = compile("let $default = 'N/A' in name || $default").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("N/A"));
    }

    #[test]
    fn test_let_with_not_expression() {
        let expr = compile("let $val = `false` in !$val").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_let_binding_to_literal() {
        let expr = compile(
            "let $str = 'hello', $num = `42`, $bool = `true`, $null = `null` in [$str, $num, $bool, $null]",
        )
        .unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["hello", 42, true, null]));
    }
}
