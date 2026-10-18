//! The borrowed builtin path must preserve the public function contract.

use jpx_core::functions::{Function, LengthFn};
use jpx_core::{Context, Runtime};
use serde_json::{Value, json};
use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn borrowed_length_matches_owned_evaluation() {
    let borrowed = Runtime::strict();
    let mut owned = Runtime::strict();
    // Public registration deliberately selects the legacy Function interface.
    owned.register_function("length", Box::new(LengthFn::new()));
    let data = json!({"items": [{"name": "one"}, {"name": "two"}], "text": "hé🦀"});

    for expression in [
        "length(items)",
        "length(@)",
        "length(text)",
        "length('')",
        "length(`[]`)",
        "length(`[1, 2, 3]`)",
        "length(items[*].name)",
        "length(reverse(items))",
        "length([length(items), length(text)])",
        "items | length(@)",
        "map(&length(@), items)",
        "length()",
        "length(items, text)",
        "length(missing)",
        "length(`true`)",
        "length(`42`)",
        "length(&items)",
        "length(abs('invalid'))",
        "length(items, abs('invalid'))",
        "unknown(length(`false`))",
    ] {
        assert_eq!(
            borrowed.compile(expression).unwrap().search(&data),
            owned.compile(expression).unwrap().search(&data),
            "{expression}"
        );
    }
}

/// A strict runtime whose built-ins all went through the public
/// `register_function`, so each one receives owned arguments.
fn owned_strict_runtime() -> Runtime {
    let mut runtime = Runtime::strict();
    let names: Vec<String> = runtime.function_names().map(str::to_owned).collect();
    assert_eq!(names.len(), 26);
    for name in names {
        let function = runtime.deregister_function(&name).unwrap();
        runtime.register_function(&name, function);
    }
    runtime
}

#[test]
fn every_borrowed_builtin_matches_owned_evaluation() {
    let borrowed = Runtime::strict();
    let owned = owned_strict_runtime();
    let data = json!({
        "items": [
            {"id": 2, "name": "b", "tag": "first"},
            {"id": 1, "name": "a", "tag": "second"},
            {"id": 2, "name": "c", "tag": "third"}
        ],
        "nums": [3, 1.0, 1, 2.5, -4],
        "ties": [1, 1.0],
        "words": ["pear", "apple", "fig"],
        "obj": {"x": 1, "y": [1, 2]},
        "other": {"y": "replaced", "z": null},
        "text": "hé🦀",
        "nothing": null
    });

    let cases = [
        // Each built-in on borrowed input, including whole subtrees.
        "abs(nums[-1])",
        "avg(nums)",
        "avg(`[]`)",
        "ceil(nums[3])",
        "contains(items, items[0])",
        "contains(nums, `1`)",
        "contains(text, 'é')",
        "contains(text, `1`)",
        "ends_with(text, '🦀')",
        "floor(nums[3])",
        "join(', ', words)",
        "keys(obj)",
        "length(items)",
        "map(&name, items)",
        "map(&missing, items)",
        "max(nums)",
        "max(words)",
        "max(ties)",
        "max(`[]`)",
        "min(nums)",
        "min(words)",
        "min(ties)",
        "max_by(items, &id)",
        "min_by(items, &id)",
        "max_by(items, &name).tag",
        "max_by(`[]`, &id)",
        "merge(obj, other)",
        "merge(obj)",
        "not_null(nothing, missing, items[1].name, text)",
        "not_null(nothing)",
        "reverse(items)",
        "reverse(text)",
        "sort(nums)",
        "sort(words)",
        "sort_by(items, &id)",
        "sort_by(items, &name)[*].tag",
        "sort_by(`[]`, &id)",
        "starts_with(text, 'h')",
        "sum(nums)",
        "to_array(items)",
        "to_array(obj)",
        "to_number('2.5')",
        "to_number(nums[0])",
        "to_number(text)",
        "to_string(obj)",
        "to_string(text)",
        "type(items)",
        "type(nothing)",
        "values(obj)",
        // Nesting, so borrowed and owned arguments meet.
        "length(sort_by(items, &id))",
        "type(max_by(items, &id))",
        "max(map(&id, items))",
        "items[?contains(['a', 'c'], name)].tag",
        // Errors: arity, argument types, expref return types.
        "abs()",
        "abs(text)",
        "contains(obj, `1`)",
        "join(', ', nums)",
        "keys(items)",
        "max(items)",
        "max_by(items, &tag)",
        "max_by(items, &obj)",
        "min_by(items, &id, &id)",
        "merge(obj, items)",
        "sort(items)",
        "sort_by(items, &obj)",
        "sort_by(items, &[id])",
        "values(text)",
    ];
    for expression in cases {
        assert_eq!(
            borrowed.compile(expression).unwrap().search(&data),
            owned.compile(expression).unwrap().search(&data),
            "{expression}"
        );
    }
}

#[test]
fn length_registration_and_direct_calls_keep_the_public_contract() {
    let mut runtime = Runtime::strict();
    let args = [json!([1, 2, 3])];
    let direct = runtime.get_function("length").unwrap();
    assert_eq!(
        direct
            .evaluate(&args, &mut Context::new("length(@)", &runtime))
            .unwrap(),
        json!(3)
    );

    let removed = runtime.deregister_function("length").unwrap();
    assert!(
        runtime
            .compile("length(@)")
            .unwrap()
            .search(&args[0])
            .is_err()
    );
    assert_eq!(
        removed
            .evaluate(&args, &mut Context::new("length(@)", &runtime))
            .unwrap(),
        json!(3)
    );
    runtime.register_function("renamed_length", removed);
    assert_eq!(
        runtime
            .compile("renamed_length(@)")
            .unwrap()
            .search(&args[0])
            .unwrap(),
        json!(3)
    );

    runtime.register_function("length", Box::new(Replacement));
    assert_eq!(
        runtime
            .compile("length(@)")
            .unwrap()
            .search(&args[0])
            .unwrap(),
        json!(99)
    );
    runtime.register_builtin_functions();
    assert_eq!(
        runtime
            .compile("length(@)")
            .unwrap()
            .search(&args[0])
            .unwrap(),
        json!(3)
    );
}

struct Replacement;

impl Function for Replacement {
    fn evaluate(&self, args: &[Value], _ctx: &mut Context<'_>) -> jpx_core::SearchResult {
        assert_eq!(args, &[json!([1, 2, 3])]);
        Ok(json!(99))
    }
}

static REPLACEMENT_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    let mut runtime = Runtime::new();
    runtime.register_function("length", Box::new(Replacement));
    runtime.register_function("new_function", Box::new(Replacement));
    runtime
});

static EMPTY_RUNTIME: LazyLock<Runtime> = LazyLock::new(Runtime::new);
static BORROWED_RUNTIME: LazyLock<Runtime> = LazyLock::new(Runtime::strict);

struct SwitchRuntime(&'static LazyLock<Runtime>);

impl Function for SwitchRuntime {
    fn evaluate(&self, _args: &[Value], ctx: &mut Context<'_>) -> jpx_core::SearchResult {
        ctx.runtime = self.0;
        Ok(json!([1, 2, 3]))
    }
}

#[test]
fn arguments_can_switch_runtime_before_function_lookup() {
    let mut runtime = Runtime::strict();
    runtime.register_function("switch", Box::new(SwitchRuntime(&REPLACEMENT_RUNTIME)));
    runtime.register_function("clear", Box::new(SwitchRuntime(&EMPTY_RUNTIME)));

    for expression in ["length(switch())", "new_function(switch())"] {
        assert_eq!(
            runtime
                .compile(expression)
                .unwrap()
                .search(&Value::Null)
                .unwrap(),
            json!(99)
        );
    }
    let error = runtime
        .compile("length(clear())")
        .unwrap()
        .search(&Value::Null)
        .unwrap_err();
    assert!(
        matches!(error.reason, jpx_core::ErrorReason::Runtime(jpx_core::RuntimeError::UnknownFunction(name)) if name == "length")
    );
}

#[test]
fn arguments_can_switch_from_owned_to_borrowed_function() {
    let mut runtime = Runtime::new();
    runtime.register_function("length", Box::new(Replacement));
    runtime.register_function("switch", Box::new(SwitchRuntime(&BORROWED_RUNTIME)));
    assert_eq!(
        runtime
            .compile("length(switch())")
            .unwrap()
            .search(&Value::Null)
            .unwrap(),
        json!(3)
    );
}

#[test]
fn legacy_functions_move_owned_argument_results() {
    let address = Arc::new(AtomicUsize::new(0));
    let produced_address = Arc::clone(&address);
    let mut runtime = Runtime::new();
    runtime.register_function(
        "produce",
        Box::new(move |_args: &[Value], _ctx: &mut Context<'_>| {
            let text = "owned argument".repeat(1000);
            produced_address.store(text.as_ptr() as usize, Ordering::Relaxed);
            Ok(Value::String(text))
        }),
    );
    runtime.register_function(
        "consume",
        Box::new(move |args: &[Value], _ctx: &mut Context<'_>| {
            let text = args[0].as_str().unwrap();
            assert_eq!(text.as_ptr() as usize, address.load(Ordering::Relaxed));
            Ok(json!(text.len()))
        }),
    );
    assert_eq!(
        runtime
            .compile("consume(produce())")
            .unwrap()
            .search(&Value::Null)
            .unwrap(),
        json!(14_000)
    );
}
