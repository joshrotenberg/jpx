//! Public AST callers can supply all i32 indices, including values the text
//! parser cannot currently represent. Out-of-range indices return JSON null.

use jpx_core::{Expression, Runtime, ast::Ast};
use serde_json::{Value, json};

fn index(idx: i32) -> Ast {
    Ast::Index { offset: 0, idx }
}

fn subexpr(lhs: Ast, rhs: Ast) -> Ast {
    Ast::Subexpr {
        offset: 0,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn search(ast: Ast, input: &Value) -> Value {
    let runtime = Runtime::strict();
    Expression::new("<public AST index>", ast, &runtime)
        .search(input)
        .unwrap()
}

#[test]
fn minimum_index_returns_null_for_all_input_types() {
    for input in [
        json!([10, 20, 30]),
        json!([]),
        json!({"0": 10}),
        json!("text"),
        json!(true),
        json!(42),
        json!(null),
    ] {
        assert_eq!(search(index(i32::MIN), &input), Value::Null);
    }
}

#[test]
fn ordinary_and_out_of_range_indices_keep_their_positions() {
    let input = json!([10, 20, 30]);
    for (idx, expected) in [
        (i32::MIN + 1, Value::Null),
        (-4, Value::Null),
        (-3, json!(10)),
        (-2, json!(20)),
        (-1, json!(30)),
        (0, json!(10)),
        (1, json!(20)),
        (2, json!(30)),
        (3, Value::Null),
        (i32::MAX, Value::Null),
    ] {
        assert_eq!(search(index(idx), &input), expected, "index {idx}");
    }
}

#[test]
fn minimum_index_propagates_null_through_a_field_path() {
    let ast = subexpr(
        subexpr(
            Ast::Field {
                offset: 0,
                name: "items".into(),
            },
            index(i32::MIN),
        ),
        Ast::Field {
            offset: 0,
            name: "id".into(),
        },
    );
    assert_eq!(search(ast, &json!({"items": [{"id": 10}]})), Value::Null);
}

#[cfg(feature = "let-expr")]
#[test]
fn scoped_variable_paths_handle_extreme_indices() {
    for (idx, expected) in [
        (i32::MIN, Value::Null),
        (i32::MAX, Value::Null),
        (-1, json!(30)),
        (0, json!(10)),
    ] {
        let ast = Ast::Let {
            offset: 0,
            bindings: vec![(
                "items".into(),
                Ast::Literal {
                    offset: 0,
                    value: json!([10, 20, 30]),
                },
            )],
            expr: Box::new(subexpr(
                Ast::VariableRef {
                    offset: 0,
                    name: "items".into(),
                },
                index(idx),
            )),
        };
        assert_eq!(search(ast, &json!([99])), expected, "index {idx}");
    }
}

#[cfg(feature = "let-expr")]
#[test]
fn scoped_input_paths_handle_extreme_indices() {
    for (idx, expected) in [
        (i32::MIN, Value::Null),
        (i32::MAX, Value::Null),
        (-1, json!(30)),
        (0, json!(10)),
    ] {
        let ast = Ast::Let {
            offset: 0,
            bindings: vec![(
                "unused".into(),
                Ast::Literal {
                    offset: 0,
                    value: json!([99]),
                },
            )],
            expr: Box::new(subexpr(
                Ast::Field {
                    offset: 0,
                    name: "items".into(),
                },
                index(idx),
            )),
        };
        assert_eq!(
            search(ast, &json!({"items": [10, 20, 30]})),
            expected,
            "index {idx}"
        );
    }
}
