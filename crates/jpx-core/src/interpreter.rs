//! Interprets JMESPath expressions.

use std::borrow::Cow;
#[cfg(feature = "let-expr")]
use std::collections::HashMap;

use serde_json::Value;

use crate::Context;
use crate::ast::Ast;
use crate::value_ext::ValueExt;
use crate::{ErrorReason, JmespathError, RuntimeError, make_expref_sentinel};

/// Result of searching data using a JMESPath Expression.
pub type SearchResult = Result<Value, JmespathError>;

/// Interpreter-internal result: borrowed from the input where possible.
///
/// Interpreting to `Cow` is what keeps search cost proportional to the size of
/// the result rather than the size of the input traversed. A field access, an
/// index, `@` and a literal all borrow; only nodes that genuinely construct a
/// new value (projections, slices, multiselects, function results) allocate.
/// The single copy happens at the boundary, in [`crate::Expression::search`].
pub(crate) type EvalResult<'a> = Result<Cow<'a, Value>, JmespathError>;

/// Borrowed `null`, so that a missing field or an out-of-range index does not
/// have to construct an owned value to say "absent".
static NULL: Value = Value::Null;

/// Maximum interpreter recursion depth.
///
/// Bounds evaluation nesting so that a deeply nested AST (e.g. a long
/// left-associative `a.a.a...` chain, which the parser builds without deep
/// recursion of its own) cannot overflow the stack and abort the process.
///
/// Chosen to stay safe even when the interpreter runs on a ~2 MiB stack (the
/// default for tokio worker threads and the Rust test harness), where each
/// recursive frame is large in debug builds. Still far above any realistic
/// expression, whose AST nesting is in the low tens.
const MAX_EVAL_DEPTH: usize = 128;

/// Interprets the given data using an AST node, returning an owned value.
///
/// Materialises whatever [`interpret_cow`] produced. Callers that evaluate an
/// expref against many elements (`map`, `sort_by`, the filter functions) go
/// through here and pay one copy per element, as they did before.
pub fn interpret(data: &Value, node: &Ast, ctx: &mut Context<'_>) -> SearchResult {
    interpret_cow(data, node, ctx).map(Cow::into_owned)
}

/// Interprets the given data using an AST node, borrowing from `data` and
/// `node` wherever the result is a value that already exists in one of them.
///
/// Thin wrapper that bounds recursion depth via `Context::eval_depth`;
/// the actual interpretation happens in `interpret_cow_inner`.
pub(crate) fn interpret_cow<'a>(
    data: &'a Value,
    node: &'a Ast,
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    ctx.eval_depth += 1;
    if ctx.eval_depth > MAX_EVAL_DEPTH {
        ctx.eval_depth -= 1;
        let reason = ErrorReason::Runtime(RuntimeError::RecursionLimitExceeded {
            limit: MAX_EVAL_DEPTH,
        });
        return Err(JmespathError::from_ctx(ctx, reason));
    }
    let result = interpret_cow_inner(data, node, ctx);
    ctx.eval_depth -= 1;
    result
}

fn interpret_cow_inner<'a>(
    data: &'a Value,
    node: &'a Ast,
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    match node {
        Ast::Field { name, .. } => Ok(Cow::Borrowed(data.get_field_ref(name).unwrap_or(&NULL))),
        // A borrowed left side keeps the whole chain borrowed, so `a.b.c.d`
        // costs nothing until the boundary. An owned left side (a function
        // result, a projection) can only be borrowed from locally, so its
        // result is materialised here; that copy is result-sized, not
        // input-sized.
        Ast::Subexpr { lhs, rhs, .. } => match interpret_cow(data, lhs, ctx)? {
            Cow::Borrowed(left) => interpret_cow(left, rhs, ctx),
            Cow::Owned(left) => Ok(Cow::Owned(interpret_cow(&left, rhs, ctx)?.into_owned())),
        },
        Ast::Identity { .. } => Ok(Cow::Borrowed(data)),
        Ast::Literal { value, .. } => Ok(Cow::Borrowed(value)),
        Ast::Index { idx, .. } => {
            let element = if *idx >= 0 {
                data.get_index_ref(*idx as usize)
            } else {
                data.get_negative_index_ref((-idx) as usize)
            };
            Ok(Cow::Borrowed(element.unwrap_or(&NULL)))
        }
        Ast::Or { lhs, rhs, .. } => {
            let left = interpret_cow(data, lhs, ctx)?;
            if left.is_truthy() {
                Ok(left)
            } else {
                interpret_cow(data, rhs, ctx)
            }
        }
        Ast::And { lhs, rhs, .. } => {
            let left = interpret_cow(data, lhs, ctx)?;
            if !left.is_truthy() {
                Ok(left)
            } else {
                interpret_cow(data, rhs, ctx)
            }
        }
        Ast::Not { node, .. } => {
            let result = interpret_cow(data, node, ctx)?;
            Ok(Cow::Owned(Value::Bool(!result.is_truthy())))
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            let cond_result = interpret_cow(data, predicate, ctx)?;
            if cond_result.is_truthy() {
                interpret_cow(data, then, ctx)
            } else {
                Ok(Cow::Borrowed(&NULL))
            }
        }
        Ast::Comparison {
            comparator,
            lhs,
            rhs,
            ..
        } => {
            let left = interpret_cow(data, lhs, ctx)?;
            let right = interpret_cow(data, rhs, ctx)?;
            Ok(Cow::Owned(
                left.compare(comparator, right.as_ref())
                    .map_or(Value::Null, Value::Bool),
            ))
        }
        Ast::ObjectValues { node, .. } => eval_object_values(data, node, ctx),
        Ast::Projection { lhs, rhs, .. } => eval_projection(data, lhs, rhs, ctx),
        Ast::Flatten { node, .. } => eval_flatten(data, node, ctx),
        Ast::MultiList { elements, .. } => eval_multi_list(data, elements, ctx),
        Ast::MultiHash { elements, .. } => eval_multi_hash(data, elements, ctx),
        Ast::Function { name, args, offset } => eval_function(data, name, args, *offset, ctx),
        Ast::Expref { ast, .. } => {
            let id = ctx.store_expref(*ast.clone());
            Ok(Cow::Owned(make_expref_sentinel(id)))
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
                    Some(array) => Ok(Cow::Owned(Value::Array(array))),
                    None => Ok(Cow::Borrowed(&NULL)),
                }
            }
        }
        #[cfg(feature = "let-expr")]
        Ast::VariableRef { name, offset } => match ctx.get_variable(name) {
            Some(value) => Ok(Cow::Owned(value)),
            None => {
                ctx.offset = *offset;
                let reason = ErrorReason::Runtime(RuntimeError::UnknownFunction(format!(
                    "Undefined variable: ${name}"
                )));
                Err(JmespathError::from_ctx(ctx, reason))
            }
        },
        #[cfg(feature = "let-expr")]
        Ast::Let { bindings, expr, .. } => eval_let(data, bindings, expr, ctx),
    }
}

// ---------------------------------------------------------------------------
// Value-constructing nodes
//
// These live in their own `#[inline(never)]` frames rather than inline in
// `interpret_cow_inner`. In a debug build every arm's locals contribute to the
// enclosing frame, and `interpret_cow_inner` is the function that recurses up
// to `MAX_EVAL_DEPTH` times on a `Subexpr` chain. Keeping the recursive frame
// down to the small arms is what leaves headroom for 128 nested evaluations on
// a 2 MiB stack, which `deep_ast_eval_errors_gracefully` pins down.
// ---------------------------------------------------------------------------

#[inline(never)]
fn eval_object_values<'a>(data: &'a Value, node: &'a Ast, ctx: &mut Context<'_>) -> EvalResult<'a> {
    let subject = interpret_cow(data, node, ctx)?;
    // Consume the map when we own it; copy its values when we do not.
    if let Cow::Owned(Value::Object(map)) = subject {
        return Ok(Cow::Owned(Value::Array(map.into_values().collect())));
    }
    match subject.as_ref() {
        Value::Object(map) => Ok(Cow::Owned(Value::Array(map.values().cloned().collect()))),
        _ => Ok(Cow::Borrowed(&NULL)),
    }
}

#[inline(never)]
fn eval_projection<'a>(
    data: &'a Value,
    lhs: &'a Ast,
    rhs: &'a Ast,
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    let left = interpret_cow(data, lhs, ctx)?;
    match left.as_ref().as_array() {
        None => Ok(Cow::Borrowed(&NULL)),
        Some(arr) => {
            let mut collected = vec![];
            for element in arr {
                let current = interpret_cow(element, rhs, ctx)?;
                if !current.is_null() {
                    collected.push(current.into_owned());
                }
            }
            Ok(Cow::Owned(Value::Array(collected)))
        }
    }
}

#[inline(never)]
fn eval_flatten<'a>(data: &'a Value, node: &'a Ast, ctx: &mut Context<'_>) -> EvalResult<'a> {
    let result = interpret_cow(data, node, ctx)?;
    match result.as_ref().as_array() {
        None => Ok(Cow::Borrowed(&NULL)),
        Some(arr) => {
            let mut collected: Vec<Value> = vec![];
            for element in arr {
                match element.as_array() {
                    Some(inner) => collected.extend(inner.iter().cloned()),
                    _ => collected.push(element.clone()),
                }
            }
            Ok(Cow::Owned(Value::Array(collected)))
        }
    }
}

#[inline(never)]
fn eval_multi_list<'a>(
    data: &'a Value,
    elements: &'a [Ast],
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    if data.is_null() {
        return Ok(Cow::Borrowed(&NULL));
    }
    let mut collected = vec![];
    for node in elements {
        collected.push(interpret_cow(data, node, ctx)?.into_owned());
    }
    Ok(Cow::Owned(Value::Array(collected)))
}

#[inline(never)]
fn eval_multi_hash<'a>(
    data: &'a Value,
    elements: &'a [crate::ast::KeyValuePair],
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    if data.is_null() {
        return Ok(Cow::Borrowed(&NULL));
    }
    let mut collected = serde_json::Map::new();
    for kvp in elements {
        let value = interpret_cow(data, &kvp.value, ctx)?.into_owned();
        collected.insert(kvp.key.clone(), value);
    }
    Ok(Cow::Owned(Value::Object(collected)))
}

/// Arguments are materialised because `Function::evaluate` takes `&[Value]`.
/// Moving that boundary onto borrowed arguments is a separate change; it would
/// touch every function implementation.
#[inline(never)]
fn eval_function<'a>(
    data: &'a Value,
    name: &str,
    args: &'a [Ast],
    offset: usize,
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    let mut fn_args: Vec<Value> = Vec::with_capacity(args.len());
    for arg in args {
        fn_args.push(interpret_cow(data, arg, ctx)?.into_owned());
    }
    ctx.offset = offset;
    match ctx.runtime.get_function(name) {
        Some(f) => f.evaluate(&fn_args, ctx).map(Cow::Owned),
        None => {
            let reason = ErrorReason::Runtime(RuntimeError::UnknownFunction(name.to_owned()));
            Err(JmespathError::from_ctx(ctx, reason))
        }
    }
}

#[cfg(feature = "let-expr")]
#[inline(never)]
fn eval_let<'a>(
    data: &'a Value,
    bindings: &'a [(String, Ast)],
    expr: &'a Ast,
    ctx: &mut Context<'_>,
) -> EvalResult<'a> {
    let mut scope = HashMap::new();
    for (name, binding_expr) in bindings {
        let value = interpret_cow(data, binding_expr, ctx)?.into_owned();
        scope.insert(name.clone(), value);
    }
    ctx.push_scope(scope);
    let result = interpret_cow(data, expr, ctx);
    ctx.pop_scope();
    result
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

    #[test]
    fn deep_ast_eval_errors_gracefully() {
        // The parser rejects deeply nested input, so build a deep AST directly
        // to exercise the interpreter's own defense-in-depth guard (which also
        // protects API callers that construct an AST themselves). Run on a
        // 2 MiB stack (a typical async worker-thread size): if the guard were
        // too lax this thread would abort the whole test process instead of
        // returning an error.
        use crate::Runtime;
        use crate::ast::Ast;
        let msg = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let mut ast = Ast::Identity { offset: 0 };
                for _ in 0..400 {
                    ast = Ast::Subexpr {
                        offset: 0,
                        lhs: Box::new(ast),
                        rhs: Box::new(Ast::Identity { offset: 0 }),
                    };
                }
                let rt = Runtime::strict();
                let expr = crate::Expression::new("<deep>", ast, &rt);
                format!("{}", expr.search(&json!({})).unwrap_err())
            })
            .unwrap()
            .join()
            .expect("deep evaluation must not overflow the stack");
        assert!(msg.contains("Recursion limit"), "unexpected message: {msg}");
    }

    #[test]
    fn moderate_ast_eval_ok() {
        // Well within the depth limit: evaluates normally (null on empty object).
        let expr = format!("a{}", ".a".repeat(50));
        assert_eq!(search(&expr, &json!({})), json!(null));
    }
}
