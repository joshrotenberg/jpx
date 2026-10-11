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
