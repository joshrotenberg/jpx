use jpx_core::Runtime;
use serde_json::{Value, json};

fn search(runtime: &Runtime, source: &str, input: &Value) -> Value {
    runtime.compile(source).unwrap().search(input).unwrap()
}

#[test]
fn strict_ordering_returns_null_for_non_numbers() {
    let runtime = Runtime::strict();
    for (left, right) in [
        (json!("a"), json!("b")),
        (json!("same"), json!("same")),
        (json!(false), json!(true)),
        (json!([]), json!([])),
        (json!({}), json!({})),
        (Value::Null, Value::Null),
        (json!(1), json!("2")),
        (json!("1"), json!(2)),
    ] {
        let input = json!({"left": left, "right": right});
        for operator in ["<", "<=", ">", ">="] {
            assert_eq!(
                search(&runtime, &format!("left {operator} right"), &input),
                Value::Null
            );
        }
    }
}

#[test]
fn default_runtimes_keep_string_ordering() {
    for runtime in [
        Runtime::new(),
        Runtime::default(),
        Runtime::builder().with_standard().build(),
    ] {
        assert_eq!(
            search(
                &runtime,
                "['a' < 'b', 'a' <= 'b', 'b' > 'a', 'b' >= 'a']",
                &json!({})
            ),
            json!([true, true, true, true])
        );
    }
}

#[test]
fn numeric_ordering_and_equality_are_unchanged() {
    for runtime in [Runtime::strict(), Runtime::new()] {
        assert_eq!(
            search(
                &runtime,
                "[`1` < `2`, `1` <= `1`, `2` > `1`, `2` >= `2`, 'a' == 'a', 'a' != 'b', `1` == '1']",
                &json!({})
            ),
            json!([true, true, true, true, true, true, false])
        );
        assert_eq!(
            search(
                &runtime,
                "`18446744073709551615` > `18446744073709551614`",
                &json!({})
            ),
            json!(true)
        );
    }
}

#[test]
fn strict_string_functions_still_order_strings() {
    let runtime = Runtime::strict();
    assert_eq!(
        search(
            &runtime,
            "[sort(@), min(@), max(@)]",
            &json!(["b", "a", "c"])
        ),
        json!([["a", "b", "c"], "a", "c"])
    );
}

#[test]
fn registering_functions_does_not_reset_strict_ordering() {
    let mut runtime = Runtime::strict();
    runtime.register_builtin_functions();
    assert_eq!(search(&runtime, "'a' < 'b'", &json!({})), Value::Null);
}
