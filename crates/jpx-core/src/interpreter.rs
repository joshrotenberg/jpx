//! Interprets JMESPath expressions.

#[cfg(feature = "let-expr")]
use std::collections::HashMap;

use serde_json::Value;

use crate::Context;
use crate::ast::Ast;
use crate::value_ext::ValueExt;
use crate::{ErrorReason, JmespathError, RuntimeError, make_expref_sentinel};

/// Result of searching data using a JMESPath Expression.
pub type SearchResult = Result<Value, JmespathError>;

/// Interprets the given data using an AST node.
pub fn interpret(data: &Value, node: &Ast, ctx: &mut Context<'_>) -> SearchResult {
    match node {
        Ast::Field { name, .. } => Ok(data.get_field(name)),
        Ast::Subexpr { lhs, rhs, .. } => {
            let left_result = interpret(data, lhs, ctx)?;
            interpret(&left_result, rhs, ctx)
        }
        Ast::Identity { .. } => Ok(data.clone()),
        Ast::Literal { value, .. } => Ok(value.clone()),
        Ast::Index { idx, .. } => {
            if *idx >= 0 {
                Ok(data.get_index(*idx as usize))
            } else {
                Ok(data.get_negative_index((-idx) as usize))
            }
        }
        Ast::Or { lhs, rhs, .. } => {
            let left = interpret(data, lhs, ctx)?;
            if left.is_truthy() {
                Ok(left)
            } else {
                interpret(data, rhs, ctx)
            }
        }
        Ast::And { lhs, rhs, .. } => {
            let left = interpret(data, lhs, ctx)?;
            if !left.is_truthy() {
                Ok(left)
            } else {
                interpret(data, rhs, ctx)
            }
        }
        Ast::Not { node, .. } => {
            let result = interpret(data, node, ctx)?;
            Ok(Value::Bool(!result.is_truthy()))
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            let cond_result = interpret(data, predicate, ctx)?;
            if cond_result.is_truthy() {
                interpret(data, then, ctx)
            } else {
                Ok(Value::Null)
            }
        }
        Ast::Comparison {
            comparator,
            lhs,
            rhs,
            ..
        } => {
            let left = interpret(data, lhs, ctx)?;
            let right = interpret(data, rhs, ctx)?;
            Ok(left
                .compare(comparator, &right)
                .map_or(Value::Null, Value::Bool))
        }
        Ast::ObjectValues { node, .. } => {
            let subject = interpret(data, node, ctx)?;
            match subject {
                Value::Object(map) => Ok(Value::Array(map.into_values().collect())),
                _ => Ok(Value::Null),
            }
        }
        Ast::Projection { lhs, rhs, .. } => {
            let left = interpret(data, lhs, ctx)?;
            match left.as_array() {
                None => Ok(Value::Null),
                Some(arr) => {
                    let mut collected = vec![];
                    for element in arr {
                        let current = interpret(element, rhs, ctx)?;
                        if !current.is_null() {
                            collected.push(current);
                        }
                    }
                    Ok(Value::Array(collected))
                }
            }
        }
        Ast::Flatten { node, .. } => {
            let result = interpret(data, node, ctx)?;
            match result.as_array() {
                None => Ok(Value::Null),
                Some(arr) => {
                    let mut collected: Vec<Value> = vec![];
                    for element in arr {
                        match element.as_array() {
                            Some(inner) => collected.extend(inner.iter().cloned()),
                            _ => collected.push(element.clone()),
                        }
                    }
                    Ok(Value::Array(collected))
                }
            }
        }
        Ast::MultiList { elements, .. } => {
            if data.is_null() {
                Ok(Value::Null)
            } else {
                let mut collected = vec![];
                for node in elements {
                    collected.push(interpret(data, node, ctx)?);
                }
                Ok(Value::Array(collected))
            }
        }
        Ast::MultiHash { elements, .. } => {
            if data.is_null() {
                Ok(Value::Null)
            } else {
                let mut collected = serde_json::Map::new();
                for kvp in elements {
                    let value = interpret(data, &kvp.value, ctx)?;
                    collected.insert(kvp.key.clone(), value);
                }
                Ok(Value::Object(collected))
            }
        }
        Ast::Function { name, args, offset } => {
            let mut fn_args: Vec<Value> = vec![];
            for arg in args {
                fn_args.push(interpret(data, arg, ctx)?);
            }
            ctx.offset = *offset;
            match ctx.runtime.get_function(name) {
                Some(f) => f.evaluate(&fn_args, ctx),
                None => {
                    let reason =
                        ErrorReason::Runtime(RuntimeError::UnknownFunction(name.to_owned()));
                    Err(JmespathError::from_ctx(ctx, reason))
                }
            }
        }
        Ast::Expref { ast, .. } => {
            let id = ctx.store_expref(*ast.clone());
            Ok(make_expref_sentinel(id))
        }
        Ast::Slice {
            start,
            stop,
            step,
            offset,
        } => {
            if *step == 0 {
                ctx.offset = *offset;
                let reason = ErrorReason::Runtime(RuntimeError::InvalidSlice);
                Err(JmespathError::from_ctx(ctx, reason))
            } else {
                match data.slice(*start, *stop, *step) {
                    Some(array) => Ok(Value::Array(array)),
                    None => Ok(Value::Null),
                }
            }
        }
        #[cfg(feature = "let-expr")]
        Ast::VariableRef { name, offset } => match ctx.get_variable(name) {
            Some(value) => Ok(value),
            None => {
                ctx.offset = *offset;
                let reason = ErrorReason::Runtime(RuntimeError::UnknownFunction(format!(
                    "Undefined variable: ${name}"
                )));
                Err(JmespathError::from_ctx(ctx, reason))
            }
        },
        #[cfg(feature = "let-expr")]
        Ast::Let { bindings, expr, .. } => {
            let mut scope = HashMap::new();
            for (name, binding_expr) in bindings {
                let value = interpret(data, binding_expr, ctx)?;
                scope.insert(name.clone(), value);
            }
            ctx.push_scope(scope);
            let result = interpret(data, expr, ctx);
            ctx.pop_scope();
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Runtime;

    fn search(expr: &str, data: &serde_json::Value) -> serde_json::Value {
        let rt = Runtime::strict();
        let compiled = rt.compile(expr).unwrap();
        compiled.search(data).unwrap()
    }

    fn search_err(expr: &str, data: &serde_json::Value) -> crate::JmespathError {
        let rt = Runtime::strict();
        let compiled = rt.compile(expr).unwrap();
        compiled.search(data).unwrap_err()
    }

    #[test]
    fn null_propagation_field() {
        assert_eq!(search("foo", &json!(null)), json!(null));
        assert_eq!(search("foo.bar", &json!({"foo": null})), json!(null));
    }

    #[test]
    fn null_propagation_index() {
        assert_eq!(search("[0]", &json!(null)), json!(null));
    }

    #[test]
    fn null_propagation_projection() {
        assert_eq!(search("[*].foo", &json!(null)), json!(null));
    }

    #[test]
    fn projection_filters_null() {
        let data = json!([{"foo": "a"}, {"bar": "b"}, {"foo": "c"}]);
        assert_eq!(search("[*].foo", &data), json!(["a", "c"]));
    }

    #[test]
    fn wildcard_on_non_object() {
        assert_eq!(search("*", &json!("string")), json!(null));
        assert_eq!(search("*", &json!(42)), json!(null));
    }

    #[test]
    fn wildcard_on_object() {
        let result = search("*", &json!({"a": 1, "b": 2}));
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr.contains(&json!(1)));
        assert!(arr.contains(&json!(2)));
    }

    #[test]
    fn cross_type_equality_returns_false() {
        // jpx-core returns false for cross-type comparisons (not null)
        assert_eq!(search("`1` == `\"1\"`", &json!(null)), json!(false));
        assert_eq!(search("`1` == `true`", &json!(null)), json!(false));
    }

    #[test]
    fn same_type_equality() {
        assert_eq!(search("`1` == `1`", &json!(null)), json!(true));
        assert_eq!(search("`1` == `2`", &json!(null)), json!(false));
        assert_eq!(
            search("`\"hello\"` == `\"hello\"`", &json!(null)),
            json!(true)
        );
    }

    #[test]
    fn flatten_semantics() {
        let data = json!([[1, 2], [3, 4], [5]]);
        assert_eq!(search("[]", &data), json!([1, 2, 3, 4, 5]));
    }

    #[test]
    fn flatten_mixed() {
        let data = json!([[1, 2], 3, [4]]);
        assert_eq!(search("[]", &data), json!([1, 2, 3, 4]));
    }

    #[test]
    fn flatten_on_non_array() {
        assert_eq!(search("[]", &json!("string")), json!(null));
    }

    #[test]
    fn pipe_stops_projection() {
        let data = json!({
            "people": [
                {"name": "a", "age": 20},
                {"name": "b", "age": 25},
                {"name": "c", "age": 30}
            ]
        });
        assert_eq!(search("people[*].name | [0]", &data), json!("a"));
    }

    #[test]
    fn or_semantics() {
        assert_eq!(search("a || b", &json!({"a": 1, "b": 2})), json!(1));
        assert_eq!(search("a || b", &json!({"b": 2})), json!(2));
        assert_eq!(search("a || b", &json!({})), json!(null));
    }

    #[test]
    fn and_semantics() {
        assert_eq!(search("a && b", &json!({"a": 1, "b": 2})), json!(2));
        assert_eq!(search("a && b", &json!({"b": 2})), json!(null));
    }

    #[test]
    fn not_semantics() {
        assert_eq!(search("!`true`", &json!(null)), json!(false));
        assert_eq!(search("!`false`", &json!(null)), json!(true));
        assert_eq!(search("!`null`", &json!(null)), json!(true));
        assert_eq!(search("!`\"\"`", &json!(null)), json!(true));
        assert_eq!(search("!`\"hello\"`", &json!(null)), json!(false));
    }

    #[test]
    fn slice_step_zero_error() {
        let err = search_err("[::0]", &json!([1, 2, 3]));
        let display = format!("{err}");
        assert!(display.contains("Invalid slice"));
    }

    #[test]
    fn unknown_function_error() {
        let err = search_err("nonexistent(@)", &json!(null));
        let display = format!("{err}");
        assert!(display.contains("Unknown function"));
    }

    #[test]
    fn multilist_on_null() {
        assert_eq!(search("[a, b]", &json!(null)), json!(null));
    }

    #[test]
    fn multihash_on_null() {
        assert_eq!(search("{x: a, y: b}", &json!(null)), json!(null));
    }

    #[test]
    fn multilist_on_data() {
        assert_eq!(search("[a, b]", &json!({"a": 1, "b": 2})), json!([1, 2]));
    }

    #[test]
    fn multihash_on_data() {
        assert_eq!(
            search("{x: a, y: b}", &json!({"a": 1, "b": 2})),
            json!({"x": 1, "y": 2})
        );
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(search("`5` > `3`", &json!(null)), json!(true));
        assert_eq!(search("`5` < `3`", &json!(null)), json!(false));
        assert_eq!(search("`5` >= `5`", &json!(null)), json!(true));
        assert_eq!(search("`5` <= `5`", &json!(null)), json!(true));
        assert_eq!(search("`5` != `3`", &json!(null)), json!(true));
    }

    #[test]
    fn literal_passthrough() {
        assert_eq!(search("`42`", &json!(null)), json!(42));
        assert_eq!(search("`\"hello\"`", &json!(null)), json!("hello"));
        assert_eq!(search("`true`", &json!(null)), json!(true));
        assert_eq!(search("`null`", &json!(null)), json!(null));
    }

    #[test]
    fn identity() {
        assert_eq!(search("@", &json!(42)), json!(42));
        assert_eq!(search("@", &json!("hello")), json!("hello"));
    }

    #[test]
    fn filter_expression() {
        let data = json!([1, 2, 3, 4, 5]);
        assert_eq!(search("[? @ > `3`]", &data), json!([4, 5]));
    }

    #[test]
    fn builtin_function_length() {
        assert_eq!(search("length(@)", &json!([1, 2, 3])), json!(3));
        assert_eq!(search("length(@)", &json!("hello")), json!(5));
    }

    #[test]
    fn builtin_function_sort() {
        assert_eq!(search("sort(@)", &json!([3, 1, 2])), json!([1, 2, 3]));
    }
}
