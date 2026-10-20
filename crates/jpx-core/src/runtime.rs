//! JMESPath runtime for compiling and evaluating expressions.

use std::collections::HashMap;

use crate::Expression;
use crate::JmespathError;
use crate::functions::*;
use crate::parse;
use crate::registry::{Category, FunctionRegistry};

/// Dispatch adapter that keeps the public custom-function API unchanged.
pub(crate) enum RegisteredFunction {
    Owned(Box<dyn Function>),
    Borrowed(Box<dyn BorrowedFunction>),
}

impl RegisteredFunction {
    pub(crate) fn as_function(&self) -> &dyn Function {
        match self {
            Self::Owned(function) => function.as_ref(),
            Self::Borrowed(function) => function.as_ref(),
        }
    }

    fn into_function(self) -> Box<dyn Function> {
        match self {
            Self::Owned(function) => function,
            Self::Borrowed(function) => function,
        }
    }
}

/// Compiles JMESPath expressions and manages registered functions.
pub struct Runtime {
    functions: HashMap<String, RegisteredFunction>,
    numeric_ordering_only: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            functions: HashMap::with_capacity(26),
            numeric_ordering_only: false,
        }
    }
}

impl Runtime {
    /// Creates a new Runtime with no functions registered.
    pub fn new() -> Runtime {
        Default::default()
    }

    /// Creates a runtime with the 26 spec functions and numeric-only ordering
    /// operators. Parser syntax remains controlled by crate features.
    pub fn strict() -> Runtime {
        let mut rt = Runtime::new();
        rt.numeric_ordering_only = true;
        rt.register_builtin_functions();
        rt
    }

    #[inline]
    pub(crate) fn numeric_ordering_only(&self) -> bool {
        self.numeric_ordering_only
    }

    /// Creates a RuntimeBuilder for selective function registration.
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    /// Returns whether this runtime can compile and evaluate JEP-18 let expressions.
    ///
    /// This reflects the features enabled on `jpx-core`, including features
    /// enabled by other dependencies. Function registration does not affect
    /// syntax support; higher-level engines may impose additional restrictions.
    pub const fn supports_let_expressions(&self) -> bool {
        cfg!(feature = "let-expr")
    }

    /// Creates a new JMESPath expression from an expression string.
    #[inline]
    pub fn compile<'a>(&'a self, expression: &str) -> Result<Expression<'a>, JmespathError> {
        parse(expression).map(|ast| Expression::new(expression, ast, self))
    }

    /// Adds a new function to the runtime.
    ///
    /// Functions registered through this interface receive owned arguments.
    /// Re-registering a built-in therefore preserves its behavior but bypasses
    /// any internal borrowed-argument optimization. Use
    /// [`Self::register_builtin_functions`] to restore optimized built-ins.
    #[inline]
    pub fn register_function(&mut self, name: &str, f: Box<dyn Function>) {
        self.functions
            .insert(name.to_owned(), RegisteredFunction::Owned(f));
    }

    /// Removes a function from the runtime.
    pub fn deregister_function(&mut self, name: &str) -> Option<Box<dyn Function>> {
        self.functions
            .remove(name)
            .map(RegisteredFunction::into_function)
    }

    /// Gets a function by name from the runtime.
    #[inline]
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a dyn Function> {
        self.functions
            .get(name)
            .map(RegisteredFunction::as_function)
    }

    pub(crate) fn get_registered_function(&self, name: &str) -> Option<&RegisteredFunction> {
        self.functions.get(name)
    }

    /// Returns an iterator over all registered function names.
    pub fn function_names(&self) -> impl Iterator<Item = &str> {
        self.functions.keys().map(|s| s.as_str())
    }

    /// Registers all 26 built-in JMESPath functions.
    ///
    /// Built-ins registered here receive borrowed arguments from the
    /// interpreter, so a call such as `type(items)` does not copy `items`.
    pub fn register_builtin_functions(&mut self) {
        self.register_borrowed("abs", Box::new(AbsFn::new()));
        self.register_borrowed("avg", Box::new(AvgFn::new()));
        self.register_borrowed("ceil", Box::new(CeilFn::new()));
        self.register_borrowed("contains", Box::new(ContainsFn::new()));
        self.register_borrowed("ends_with", Box::new(EndsWithFn::new()));
        self.register_borrowed("floor", Box::new(FloorFn::new()));
        self.register_borrowed("join", Box::new(JoinFn::new()));
        self.register_borrowed("keys", Box::new(KeysFn::new()));
        self.register_borrowed("length", Box::new(LengthFn::new()));
        self.register_borrowed("map", Box::new(MapFn::new()));
        self.register_borrowed("min", Box::new(MinFn::new()));
        self.register_borrowed("max", Box::new(MaxFn::new()));
        self.register_borrowed("max_by", Box::new(MaxByFn::new()));
        self.register_borrowed("min_by", Box::new(MinByFn::new()));
        self.register_borrowed("merge", Box::new(MergeFn::new()));
        self.register_borrowed("not_null", Box::new(NotNullFn::new()));
        self.register_borrowed("reverse", Box::new(ReverseFn::new()));
        self.register_borrowed("sort", Box::new(SortFn::new()));
        self.register_borrowed("sort_by", Box::new(SortByFn::new()));
        self.register_borrowed("starts_with", Box::new(StartsWithFn::new()));
        self.register_borrowed("sum", Box::new(SumFn::new()));
        self.register_borrowed("to_array", Box::new(ToArrayFn::new()));
        self.register_borrowed("to_number", Box::new(ToNumberFn::new()));
        self.register_borrowed("to_string", Box::new(ToStringFn::new()));
        self.register_borrowed("type", Box::new(TypeFn::new()));
        self.register_borrowed("values", Box::new(ValuesFn::new()));
    }

    pub(crate) fn register_borrowed(&mut self, name: &str, f: Box<dyn BorrowedFunction>) {
        self.functions
            .insert(name.to_owned(), RegisteredFunction::Borrowed(f));
    }
}

/// Builder for creating a Runtime with selective function registration.
///
/// # Example
///
/// ```
/// use jpx_core::Runtime;
/// use jpx_core::Category;
///
/// let rt = Runtime::builder()
///     .with_standard()
///     .with_category(Category::String)
///     .with_category(Category::Math)
///     .build();
/// ```
pub struct RuntimeBuilder {
    registry: FunctionRegistry,
    include_standard: bool,
}

impl RuntimeBuilder {
    fn new() -> Self {
        RuntimeBuilder {
            registry: FunctionRegistry::new(),
            include_standard: false,
        }
    }

    /// Includes the 26 standard JMESPath functions.
    pub fn with_standard(mut self) -> Self {
        self.include_standard = true;
        self
    }

    /// Includes all functions from a specific category.
    pub fn with_category(mut self, category: Category) -> Self {
        self.registry.register_category(category);
        self
    }

    /// Includes all available extension functions.
    pub fn with_all_extensions(mut self) -> Self {
        self.registry.register_all();
        self
    }

    /// Disables a specific function.
    pub fn without_function(mut self, name: &str) -> Self {
        self.registry.disable_function(name);
        self
    }

    /// Builds the runtime with the configured functions.
    pub fn build(self) -> Runtime {
        let mut rt = Runtime::new();
        if self.include_standard {
            rt.register_builtin_functions();
        }
        self.registry.apply(&mut rt);
        rt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_runtime_has_no_functions() {
        let rt = Runtime::new();
        assert!(rt.get_function("abs").is_none());
        assert_eq!(rt.function_names().count(), 0);
    }

    #[test]
    fn strict_runtime_has_26_builtins() {
        let rt = Runtime::strict();
        assert_eq!(rt.function_names().count(), 26);
        assert!(rt.get_function("abs").is_some());
        assert!(rt.get_function("length").is_some());
        assert!(rt.get_function("sort").is_some());
    }

    #[test]
    fn every_builtin_uses_borrowed_dispatch() {
        let rt = Runtime::strict();
        for name in rt.function_names() {
            assert!(
                matches!(
                    rt.get_registered_function(name),
                    Some(RegisteredFunction::Borrowed(_))
                ),
                "{name}"
            );
        }
    }

    /// Every function on the borrowed path returns what the same function
    /// returns through the public owned path, for its registry example.
    #[test]
    #[cfg(feature = "extensions")]
    fn borrowed_extensions_match_owned_evaluation_on_examples() {
        let build = || {
            let mut rt = Runtime::strict();
            let mut registry = FunctionRegistry::new();
            registry.register_all();
            registry.apply(&mut rt);
            (rt, registry)
        };
        let (borrowed, registry) = build();
        let (mut owned, _) = build();
        let names: Vec<String> = borrowed
            .function_names()
            .filter(|name| {
                matches!(
                    borrowed.get_registered_function(name),
                    Some(RegisteredFunction::Borrowed(_))
                )
            })
            .map(str::to_owned)
            .collect();
        // The 26 built-ins plus the opted-in extensions and their aliases.
        assert!(names.len() > 70, "{}", names.len());
        for name in &names {
            let function = owned.deregister_function(name).unwrap();
            owned.register_function(name, function);
        }

        let mut checked = 0;
        for name in &names {
            let Some(info) = registry.get_function_by_name_or_alias(name) else {
                continue;
            };
            let Some((expression, _)) = info.example.split_once(" -> ") else {
                continue;
            };
            let run = |rt: &Runtime| {
                rt.compile(expression)
                    .map(|e| e.search(&serde_json::Value::Null))
            };
            // Skip functions whose result is not deterministic.
            if run(&owned) != run(&owned) {
                continue;
            }
            assert_eq!(run(&borrowed), run(&owned), "{name}: {expression}");
            checked += 1;
        }
        assert!(checked > 50, "{checked}");
    }

    #[test]
    fn builtin_length_restores_borrowed_dispatch_after_public_override() {
        let mut rt = Runtime::strict();
        assert!(matches!(
            rt.get_registered_function("length"),
            Some(RegisteredFunction::Borrowed(_))
        ));
        rt.register_function("length", Box::new(LengthFn::new()));
        assert!(matches!(
            rt.get_registered_function("length"),
            Some(RegisteredFunction::Owned(_))
        ));
        rt.register_builtin_functions();
        assert!(matches!(
            rt.get_registered_function("length"),
            Some(RegisteredFunction::Borrowed(_))
        ));
    }

    #[test]
    fn let_capability_matches_compilation() {
        // Runtime::strict restricts functions and ordering, not syntax. The engine applies
        // the additional strict-mode syntax restriction.
        for rt in [Runtime::new(), Runtime::strict()] {
            let expr = rt.compile("let $value = value in $value");
            assert_eq!(rt.supports_let_expressions(), expr.is_ok());
            if let Ok(expr) = expr {
                assert_eq!(expr.search(&json!({"value": 42})).unwrap(), json!(42));
            }
        }
    }

    #[test]
    fn register_and_get_function() {
        let mut rt = Runtime::new();
        rt.register_function("abs", Box::new(AbsFn::new()));
        assert!(rt.get_function("abs").is_some());
        assert!(rt.get_function("nonexistent").is_none());
    }

    #[test]
    fn deregister_function() {
        let mut rt = Runtime::strict();
        assert!(rt.get_function("abs").is_some());
        let removed = rt.deregister_function("abs");
        assert!(removed.is_some());
        assert!(rt.get_function("abs").is_none());
        assert_eq!(rt.function_names().count(), 25);
    }

    #[test]
    fn deregister_nonexistent_returns_none() {
        let mut rt = Runtime::new();
        assert!(rt.deregister_function("nope").is_none());
    }

    #[test]
    fn builder_with_standard() {
        let rt = Runtime::builder().with_standard().build();
        assert_eq!(rt.function_names().count(), 26);
    }

    #[test]
    #[cfg(feature = "extensions")]
    fn builder_with_category() {
        let rt = Runtime::builder()
            .with_standard()
            .with_category(Category::String)
            .build();
        assert!(rt.function_names().count() > 26);
        assert!(rt.get_function("lower").is_some());
    }

    #[test]
    #[cfg(feature = "extensions")]
    fn builder_with_all_extensions() {
        let rt = Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build();
        assert!(rt.function_names().count() > 26);
    }

    #[test]
    #[cfg(feature = "extensions")]
    fn builder_without_function() {
        let rt = Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .without_function("lower")
            .build();
        assert!(rt.get_function("lower").is_none());
        assert!(rt.get_function("upper").is_some());
    }

    // Holds under every feature combination: the registry lists only
    // functions from compiled-in categories, and the runtime can call each.
    #[test]
    fn registry_matches_compiled_functions() {
        let mut registry = FunctionRegistry::new();
        registry.register_all();
        let rt = Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build();
        for info in registry.functions() {
            assert!(info.category.is_available(), "{}", info.name);
            assert!(rt.get_function(info.name).is_some(), "{}", info.name);
        }
    }

    #[test]
    #[cfg(feature = "ext-core")]
    fn redact_keys_follows_ext_regex() {
        let rt = Runtime::builder().with_category(Category::Object).build();
        assert!(rt.get_function("pick").is_some());
        assert_eq!(
            rt.get_function("redact_keys").is_some(),
            cfg!(feature = "ext-regex")
        );
    }

    #[test]
    #[cfg(not(feature = "ext-time"))]
    fn uncompiled_category_registers_nothing() {
        assert!(!Category::Datetime.is_available());
        assert_eq!(Category::Datetime.feature_name(), Some("ext-time"));
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Datetime);
        assert!(registry.is_empty());
        assert_eq!(registry.categories().count(), 0);
    }

    #[test]
    fn compile_with_runtime() {
        let rt = Runtime::strict();
        let expr = rt.compile("length(@)").unwrap();
        let result = expr.search(&json!([1, 2, 3])).unwrap();
        assert_eq!(result, json!(3));
    }

    #[test]
    fn unknown_function_compile_succeeds_search_fails() {
        let rt = Runtime::new();
        let expr = rt.compile("nonexistent(@)").unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    #[test]
    fn function_names_iterator() {
        let rt = Runtime::strict();
        let names: Vec<&str> = rt.function_names().collect();
        assert!(names.contains(&"abs"));
        assert!(names.contains(&"length"));
        assert!(names.contains(&"values"));
    }
}
