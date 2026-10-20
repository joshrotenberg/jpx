#![cfg(feature = "let-expr")]

use std::collections::HashMap;

use jpx_core::{Context, ErrorReason, Runtime, RuntimeError, ast::Ast, functions::Function};
use serde_json::{Value, json};

fn search(source: &str, input: &Value) -> Value {
    Runtime::strict()
        .compile(source)
        .unwrap()
        .search(input)
        .unwrap()
}

#[test]
fn variable_paths_match_input_paths() {
    let input = json!({"items": [{"id": 1, "nested": [null, {"id": 2}]}, {"id": 3}]});
    for suffix in [
        ".items[0].id",
        ".items[-1].id",
        ".items[0].nested[1].id",
        ".items[0].nested[0].id",
        ".items[0]",
        ".items[20].id",
        ".items[-20].id",
        ".missing.items[0]",
        ".items.wrong_type",
        ".items[0].id[0]",
        ".\"items\"[0].id",
    ] {
        let expected = search(&format!("@{suffix}"), &input);
        assert_eq!(
            search(&format!("let $x = @ in $x{suffix}"), &input),
            expected,
            "{suffix}"
        );
    }
}

#[test]
fn repeated_and_shadowed_paths_use_the_correct_scope() {
    let input = json!({"items": [{"id": 1}, {"id": 2}]});
    assert_eq!(
        search(
            "let $x = items in [$x[0].id, let $x = `[{\"id\":3}]`, $y = $x in [$x[0].id, $y[-1].id], $x[-1].id]",
            &input
        ),
        json!([1, [3, 2], 2])
    );
}

#[test]
fn constructed_bindings_and_exprefs_support_variable_paths() {
    let input = json!({"items": [{"id": 1}, {"id": 2}]});
    assert_eq!(
        search("let $x = reverse(items) in map(&$x[0].id, items)", &input),
        json!([2, 2])
    );
}

#[test]
fn non_path_operations_keep_their_existing_semantics() {
    let input = json!({"items": [{"id": 1}, {"id": 2}, {"id": null}]});
    for suffix in [
        "[0:2].id",
        "[].id",
        "[?id > `1`].id",
        " | length(@)",
        ".{first: [0].id, last: [-1].id, size: length(@)}",
    ] {
        assert_eq!(
            search(&format!("let $x = items in $x{suffix}"), &input),
            search(&format!("items{suffix}"), &input),
            "{suffix}"
        );
    }
}

struct ReplaceScope;

impl Function for ReplaceScope {
    fn evaluate(&self, _: &[Value], ctx: &mut Context<'_>) -> jpx_core::SearchResult {
        ctx.pop_scope();
        ctx.push_scope(HashMap::from([("x".into(), json!([{"id": 9}]))]));
        Ok(Value::Null)
    }
}

#[test]
fn custom_functions_can_still_mutate_scopes_between_lookups() {
    let mut runtime = Runtime::strict();
    runtime.register_function("replace_scope", Box::new(ReplaceScope));
    let expression = runtime
        .compile("let $x = items in [$x[0], replace_scope(), $x[0]]")
        .unwrap();
    assert_eq!(
        expression.search(&json!({"items": [{"id": 1}]})).unwrap(),
        json!([{"id": 1}, null, {"id": 9}])
    );
}

#[test]
fn public_variable_reads_still_return_independent_values() {
    let runtime = Runtime::strict();
    let mut ctx = Context::new("$x[0].id", &runtime);
    ctx.push_scope(HashMap::from([("x".into(), json!([{"id": 1}]))]));
    let mut copy = ctx.get_variable("x").unwrap();
    copy[0]["id"] = json!(2);
    assert_eq!(
        jpx_core::interpreter::interpret(
            &Value::Null,
            &jpx_core::parse(ctx.expression).unwrap(),
            &mut ctx
        )
        .unwrap(),
        json!(1)
    );
}

#[test]
fn path_errors_preserve_offsets_and_unwind_scopes() {
    let runtime = Runtime::strict();
    for (source, reason, offset) in [
        (
            "let $x = items in $missing[0].id",
            RuntimeError::UnknownFunction("Undefined variable: $missing".into()),
            18,
        ),
        ("let $x = items in $x[::0]", RuntimeError::InvalidSlice, 24),
    ] {
        let ast = jpx_core::parse(source).unwrap();
        let mut ctx = Context::new(source, &runtime);
        ctx.push_scope(HashMap::from([("outer".into(), json!(7))]));
        let error =
            jpx_core::interpreter::interpret(&json!({"items": [1]}), &ast, &mut ctx).unwrap_err();
        assert_eq!(error.reason, ErrorReason::Runtime(reason));
        assert_eq!(error.offset, offset, "{source}");
        assert_eq!(ctx.get_variable("x"), None);
        assert_eq!(ctx.get_variable("outer"), Some(json!(7)));
        assert_eq!(
            jpx_core::interpreter::interpret(
                &Value::Null,
                &jpx_core::parse("$outer").unwrap(),
                &mut ctx
            )
            .unwrap(),
            json!(7)
        );
    }
}

fn nested_path(root: Ast, depth: usize, right_nested: bool) -> Ast {
    if right_nested {
        let mut tail = Ast::Identity { offset: 0 };
        for _ in 1..depth {
            tail = Ast::Subexpr {
                offset: 0,
                lhs: Box::new(Ast::Identity { offset: 0 }),
                rhs: Box::new(tail),
            };
        }
        Ast::Subexpr {
            offset: 0,
            lhs: Box::new(root),
            rhs: Box::new(tail),
        }
    } else {
        let mut path = root;
        for _ in 0..depth {
            path = Ast::Subexpr {
                offset: 0,
                lhs: Box::new(path),
                rhs: Box::new(Ast::Identity { offset: 0 }),
            };
        }
        path
    }
}

#[test]
fn scoped_paths_retain_the_interpreter_depth_limit() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let runtime = Runtime::strict();
            // API users may supply either left- or right-nested ASTs directly.
            for variable_root in [false, true] {
                for right_nested in [false, true] {
                    for depth in [125, 126, 127, 128, 400] {
                        let root = if variable_root {
                            Ast::VariableRef {
                                offset: 0,
                                name: "x".into(),
                            }
                        } else {
                            Ast::Identity { offset: 0 }
                        };
                        let path = nested_path(root, depth, right_nested);
                        let mut ctx = Context::new("<scoped path>", &runtime);
                        ctx.push_scope(HashMap::from([("x".into(), json!(7))]));
                        let result = jpx_core::interpreter::interpret(&json!(7), &path, &mut ctx);
                        if depth < 128 {
                            assert_eq!(result.unwrap(), json!(7));
                        } else {
                            assert!(matches!(
                                result.unwrap_err().reason,
                                ErrorReason::Runtime(RuntimeError::RecursionLimitExceeded {
                                    limit: 128
                                })
                            ));
                        }
                        // Even after a depth error, the same context remains usable.
                        assert_eq!(
                            jpx_core::interpreter::interpret(
                                &Value::Null,
                                &Ast::VariableRef {
                                    offset: 0,
                                    name: "x".into()
                                },
                                &mut ctx
                            )
                            .unwrap(),
                            json!(7)
                        );
                    }
                }
            }
        })
        .unwrap()
        .join()
        .expect("scoped paths must not overflow the stack");
}
