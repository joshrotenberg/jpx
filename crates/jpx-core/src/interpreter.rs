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
