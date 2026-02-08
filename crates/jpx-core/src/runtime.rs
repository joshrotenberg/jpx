//! JMESPath runtime for compiling and evaluating expressions.

use std::collections::HashMap;

use crate::Expression;
use crate::JmespathError;
use crate::functions::*;
use crate::parse;
use crate::registry::{Category, FunctionRegistry};

/// Compiles JMESPath expressions and manages registered functions.
pub struct Runtime {
    functions: HashMap<String, Box<dyn Function>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            functions: HashMap::with_capacity(26),
        }
    }
}

impl Runtime {
    /// Creates a new Runtime with no functions registered.
    pub fn new() -> Runtime {
        Default::default()
    }

    /// Creates a strict-mode runtime with only the 26 spec functions.
    pub fn strict() -> Runtime {
        let mut rt = Runtime::new();
        rt.register_builtin_functions();
        rt
    }

    /// Creates a RuntimeBuilder for selective function registration.
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    /// Creates a new JMESPath expression from an expression string.
    #[inline]
    pub fn compile<'a>(&'a self, expression: &str) -> Result<Expression<'a>, JmespathError> {
        parse(expression).map(|ast| Expression::new(expression, ast, self))
    }

    /// Adds a new function to the runtime.
    #[inline]
    pub fn register_function(&mut self, name: &str, f: Box<dyn Function>) {
        self.functions.insert(name.to_owned(), f);
    }

    /// Removes a function from the runtime.
    pub fn deregister_function(&mut self, name: &str) -> Option<Box<dyn Function>> {
        self.functions.remove(name)
    }

    /// Gets a function by name from the runtime.
    #[inline]
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a dyn Function> {
        self.functions.get(name).map(AsRef::as_ref)
    }

    /// Returns an iterator over all registered function names.
    pub fn function_names(&self) -> impl Iterator<Item = &str> {
        self.functions.keys().map(|s| s.as_str())
    }

    /// Registers all 26 built-in JMESPath functions.
    pub fn register_builtin_functions(&mut self) {
        self.register_function("abs", Box::new(AbsFn::new()));
        self.register_function("avg", Box::new(AvgFn::new()));
        self.register_function("ceil", Box::new(CeilFn::new()));
        self.register_function("contains", Box::new(ContainsFn::new()));
        self.register_function("ends_with", Box::new(EndsWithFn::new()));
        self.register_function("floor", Box::new(FloorFn::new()));
        self.register_function("join", Box::new(JoinFn::new()));
        self.register_function("keys", Box::new(KeysFn::new()));
        self.register_function("length", Box::new(LengthFn::new()));
        self.register_function("map", Box::new(MapFn::new()));
        self.register_function("min", Box::new(MinFn::new()));
        self.register_function("max", Box::new(MaxFn::new()));
        self.register_function("max_by", Box::new(MaxByFn::new()));
        self.register_function("min_by", Box::new(MinByFn::new()));
        self.register_function("merge", Box::new(MergeFn::new()));
        self.register_function("not_null", Box::new(NotNullFn::new()));
        self.register_function("reverse", Box::new(ReverseFn::new()));
        self.register_function("sort", Box::new(SortFn::new()));
        self.register_function("sort_by", Box::new(SortByFn::new()));
        self.register_function("starts_with", Box::new(StartsWithFn::new()));
        self.register_function("sum", Box::new(SumFn::new()));
        self.register_function("to_array", Box::new(ToArrayFn::new()));
        self.register_function("to_number", Box::new(ToNumberFn::new()));
        self.register_function("to_string", Box::new(ToStringFn::new()));
        self.register_function("type", Box::new(TypeFn::new()));
        self.register_function("values", Box::new(ValuesFn::new()));
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
