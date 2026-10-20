use jpx_core::Runtime;
use serde_json::{Value, json};

#[test]
fn owned_multiselect_flattens_one_level_in_order() {
    let runtime = Runtime::strict();
    let input = json!({"left": [1, [2, [3]], null], "right": [true, {"id": 4}, []], "scalar": 5});
    assert_eq!(
        runtime
            .compile("[left, right, scalar][]")
            .unwrap()
            .search(&input)
            .unwrap(),
        json!([1, [2, [3]], true, {"id": 4}, [], 5])
    );
}

#[test]
fn owned_function_and_borrowed_input_keep_the_same_flatten_semantics() {
    let runtime = Runtime::strict();
    let input = json!({"items": [null, [1, null, [2]], [], {"id": 3}, 4]});
    for source in ["items[]", "to_array(items)[]"] {
        assert_eq!(
            runtime.compile(source).unwrap().search(&input).unwrap(),
            json!([1, [2], {"id": 3}, 4])
        );
    }
}

#[test]
fn projections_and_filters_after_owned_flatten_drop_null_results() {
    let runtime = Runtime::strict();
    let input = json!({"items": [{"id": 1}, null, {"missing": true}, {"id": 2}, {"id": null}]});
    for (source, expected) in [
        ("to_array(items)[].id", json!([1, 2])),
        ("[items][] | [?id >= `2`].id", json!([2])),
    ] {
        assert_eq!(
            runtime.compile(source).unwrap().search(&input).unwrap(),
            expected
        );
    }
}

#[test]
fn flatten_results_remain_independent_of_input_and_other_searches() {
    let runtime = Runtime::strict();
    let compiled = runtime.compile("[items][]").unwrap();
    let input = json!({"items": [{"id": 1, "nested": {"value": "original"}}]});
    let original = input.clone();
    let mut first = compiled.search(&input).unwrap();
    first[0]["nested"]["value"] = json!("changed");
    assert_eq!(input, original);
    assert_eq!(compiled.search(&input).unwrap(), original["items"]);
    drop(input);
    assert_eq!(first[0]["nested"]["value"], json!("changed"));
}

#[test]
fn owned_non_arrays_still_flatten_to_null() {
    let runtime = Runtime::strict();
    let compiled = runtime.compile("not_null(value, `false`)[]").unwrap();
    for value in [
        Value::Null,
        json!(42),
        json!(false),
        json!({"a": 1}),
        json!("text"),
    ] {
        assert_eq!(
            compiled.search(&json!({"value": value})).unwrap(),
            Value::Null
        );
    }
}
