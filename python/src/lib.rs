//! Python bindings for jpx-core and jpx-engine.
//!
//! Provides both module-level convenience functions (backed by a global jpx-core
//! Runtime) and a `JpxEngine` class wrapping the full jpx-engine API.

use std::sync::OnceLock;

use jpx_core::{FunctionRegistry, Runtime};
use jpx_engine::JpxEngine as RustJpxEngine;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Value;

// =============================================================================
// Global runtime (for module-level convenience functions)
// =============================================================================

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        let mut registry = FunctionRegistry::new();
        registry.register_all();
        registry.apply(&mut runtime);
        runtime
    })
}

// =============================================================================
// Conversion helpers
// =============================================================================

/// Convert a Python object to serde_json::Value.
fn python_to_json(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value::Number(i.into()))
    } else if let Ok(u) = obj.extract::<u64>() {
        // Ints in (i64::MAX, u64::MAX] -- preserved exactly rather than being
        // coerced to a lossy f64.
        Ok(Value::Number(u.into()))
    } else if obj.is_instance_of::<pyo3::types::PyInt>() {
        // A Python int that fits neither i64 nor u64 cannot be represented in
        // JSON without losing precision; error rather than silently coercing it
        // to a float.
        Err(PyValueError::new_err(
            "integer is too large to represent in JSON (exceeds 64-bit range)",
        ))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(list) = obj.cast::<pyo3::types::PyList>() {
        let arr: Result<Vec<Value>, _> = list.iter().map(|item| python_to_json(&item)).collect();
        Ok(Value::Array(arr?))
    } else if let Ok(tuple) = obj.cast::<pyo3::types::PyTuple>() {
        let arr: Result<Vec<Value>, _> = tuple.iter().map(|item| python_to_json(&item)).collect();
        Ok(Value::Array(arr?))
    } else if let Ok(dict) = obj.cast::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = key
                .extract::<String>()
                .map_err(|_| PyValueError::new_err("Dictionary keys must be strings"))?;
            map.insert(key_str, python_to_json(&value)?);
        }
        Ok(Value::Object(map))
    } else {
        Err(PyValueError::new_err(format!(
            "Cannot convert {} to JSON",
            obj.get_type().name()?
        )))
    }
}

/// Convert serde_json::Value to a Python object.
fn json_to_python(py: Python<'_>, value: &Value) -> PyResult<Py<PyAny>> {
    use pyo3::IntoPyObject;

    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok((*b).into_pyobject(py)?.to_owned().into_any().unbind()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().into_any().unbind())
            } else if let Some(u) = n.as_u64() {
                // Values in (i64::MAX, u64::MAX] -- return an exact Python int
                // instead of falling through to a lossy float.
                Ok(u.into_pyobject(py)?.to_owned().into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        Value::String(s) => Ok(s.as_str().into_pyobject(py)?.to_owned().into_any().unbind()),
        Value::Array(arr) => {
            let list = pyo3::types::PyList::empty(py);
            for item in arr {
                list.append(json_to_python(py, item)?)?;
            }
            Ok(list.into())
        }
        Value::Object(obj) => {
            let dict = pyo3::types::PyDict::new(py);
            for (key, val) in obj {
                dict.set_item(key, json_to_python(py, val)?)?;
            }
            Ok(dict.into())
        }
    }
}

// =============================================================================
// Module-level functions (backed by global Runtime)
// =============================================================================

/// Search JSON data using a JMESPath expression with 490+ functions.
///
/// Args:
///     expression: A JMESPath expression string
///     data: JSON data as a Python object (dict, list, str, int, float, bool, None)
///
/// Returns:
///     The result of evaluating the expression against the data
///
/// Raises:
///     ValueError: If the expression is invalid or evaluation fails
#[pyfunction]
#[pyo3(signature = (expression, data))]
fn search(expression: &str, data: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let runtime = get_runtime();

    let expr = runtime
        .compile(expression)
        .map_err(|e| PyValueError::new_err(format!("Invalid expression: {}", e)))?;

    let json_value = python_to_json(data)?;

    let result = expr
        .search(&json_value)
        .map_err(|e| PyValueError::new_err(format!("Evaluation error: {}", e)))?;

    Python::attach(|py| json_to_python(py, &result))
}

/// Compile a JMESPath expression for repeated use.
///
/// Args:
///     expression: A JMESPath expression string
///
/// Returns:
///     A CompiledExpression object
///
/// Raises:
///     ValueError: If the expression is invalid
#[pyfunction]
#[pyo3(name = "compile")]
fn compile_expr(expression: &str) -> PyResult<CompiledExpression> {
    let runtime = get_runtime();

    runtime
        .compile(expression)
        .map_err(|e| PyValueError::new_err(format!("Invalid expression: {}", e)))?;

    Ok(CompiledExpression {
        expression: expression.to_string(),
    })
}

/// Validate a JMESPath expression without evaluating it.
///
/// Args:
///     expression: A JMESPath expression string
///
/// Returns:
///     A dict with keys "valid" (bool) and optionally "error" (str)
#[pyfunction]
fn validate(py: Python<'_>, expression: &str) -> PyResult<Py<PyAny>> {
    let dict = pyo3::types::PyDict::new(py);
    match jpx_core::compile(expression) {
        Ok(_) => {
            dict.set_item("valid", true)?;
            dict.set_item("error", py.None())?;
        }
        Err(e) => {
            dict.set_item("valid", false)?;
            dict.set_item("error", e.to_string())?;
        }
    }
    Ok(dict.into())
}

/// List all available function names, optionally filtered by category.
///
/// Args:
///     category: Optional category name to filter by (case-insensitive)
///
/// Returns:
///     A list of function name strings
#[pyfunction]
#[pyo3(signature = (category=None))]
fn list_functions(category: Option<&str>) -> PyResult<Vec<String>> {
    let mut registry = FunctionRegistry::new();
    registry.register_all();

    let functions: Vec<String> = registry
        .functions()
        .filter(|f| {
            if let Some(cat) = category {
                let cat_lower = cat.to_lowercase();
                let func_cat = format!("{:?}", f.category).to_lowercase();
                func_cat == cat_lower
            } else {
                true
            }
        })
        .map(|f| f.name.to_string())
        .collect();

    Ok(functions)
}

/// List all available function categories.
///
/// Returns:
///     A list of category name strings
#[pyfunction]
fn list_categories() -> Vec<String> {
    use jpx_core::Category;
    Category::all().iter().map(|c| format!("{:?}", c)).collect()
}

/// Get information about a specific function.
///
/// Args:
///     name: The function name
///
/// Returns:
///     A dict with function info, or None if not found
#[pyfunction]
fn describe(py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
    let mut registry = FunctionRegistry::new();
    registry.register_all();

    if let Some(info) = registry.functions().find(|f| f.name == name) {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("name", info.name)?;
        dict.set_item("category", format!("{:?}", info.category))?;
        dict.set_item("description", info.description)?;
        dict.set_item("signature", info.signature)?;
        dict.set_item("example", info.example)?;
        dict.set_item("is_standard", info.is_standard)?;
        Ok(Some(dict.into()))
    } else {
        Ok(None)
    }
}

// =============================================================================
// CompiledExpression class
// =============================================================================

/// A compiled JMESPath expression for efficient repeated searches.
#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct CompiledExpression {
    expression: String,
}

#[pymethods]
impl CompiledExpression {
    /// Search JSON data using this compiled expression.
    fn search(&self, data: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        search(&self.expression, data)
    }

    /// The expression string.
    #[getter]
    fn expression(&self) -> &str {
        &self.expression
    }

    fn __repr__(&self) -> String {
        format!("CompiledExpression({:?})", self.expression)
    }

    fn __str__(&self) -> String {
        self.expression.clone()
    }
}

// =============================================================================
// JpxEngine class
// =============================================================================

/// Full-featured JMESPath query engine with introspection, query store, and JSON utilities.
#[pyclass]
struct JpxEngine {
    inner: RustJpxEngine,
}

#[pymethods]
impl JpxEngine {
    /// Create a new JpxEngine.
    ///
    /// Args:
    ///     strict: If True, only standard JMESPath functions are available for evaluation.
    #[new]
    #[pyo3(signature = (strict=false))]
    fn new(strict: bool) -> Self {
        Self {
            inner: RustJpxEngine::with_options(strict),
        }
    }

    /// Whether this engine is in strict mode.
    #[getter]
    fn strict(&self) -> bool {
        self.inner.is_strict()
    }

    // =========================================================================
    // Evaluation
    // =========================================================================

    /// Evaluate a JMESPath expression against data.
    ///
    /// Args:
    ///     expression: A JMESPath expression string
    ///     data: JSON data as a Python object
    ///
    /// Returns:
    ///     The result of evaluating the expression
    ///
    /// Raises:
    ///     ValueError: If the expression is invalid or evaluation fails
    #[pyo3(signature = (expression, data))]
    fn evaluate(&self, expression: &str, data: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let json_value = python_to_json(data)?;
        let result = self
            .inner
            .evaluate(expression, &json_value)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Python::attach(|py| json_to_python(py, &result))
    }

    /// Evaluate a JMESPath expression against a JSON string.
    ///
    /// Args:
    ///     expression: A JMESPath expression string
    ///     json_str: A JSON string
    ///
    /// Returns:
    ///     The result of evaluating the expression
    #[pyo3(signature = (expression, json_str))]
    fn evaluate_str(&self, expression: &str, json_str: &str) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .evaluate_str(expression, json_str)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Python::attach(|py| json_to_python(py, &result))
    }

    /// Evaluate multiple expressions against the same data.
    ///
    /// Args:
    ///     expressions: A list of JMESPath expression strings
    ///     data: JSON data as a Python object
    ///
    /// Returns:
    ///     A list of dicts, each with "expression", "result", and "error" keys
    #[pyo3(signature = (expressions, data))]
    fn batch_evaluate(
        &self,
        py: Python<'_>,
        expressions: Vec<String>,
        data: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        let json_value = python_to_json(data)?;
        let batch = self.inner.batch_evaluate(&expressions, &json_value);

        let result_list = pyo3::types::PyList::empty(py);
        for r in &batch.results {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("expression", &r.expression)?;
            match &r.result {
                Some(val) => dict.set_item("result", json_to_python(py, val)?)?,
                None => dict.set_item("result", py.None())?,
            }
            match &r.error {
                Some(err) => dict.set_item("error", err)?,
                None => dict.set_item("error", py.None())?,
            }
            result_list.append(dict)?;
        }

        Ok(result_list.into())
    }

    /// Validate a JMESPath expression without evaluating it.
    ///
    /// Args:
    ///     expression: A JMESPath expression string
    ///
    /// Returns:
    ///     A dict with "valid" (bool) and optionally "error" (str)
    fn validate_expression(&self, py: Python<'_>, expression: &str) -> PyResult<Py<PyAny>> {
        let v = self.inner.validate(expression);
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("valid", v.valid)?;
        match v.error {
            Some(e) => dict.set_item("error", e)?,
            None => dict.set_item("error", py.None())?,
        }
        Ok(dict.into())
    }

    /// Explain a JMESPath expression step by step.
    ///
    /// Args:
    ///     expression: A JMESPath expression string
    ///
    /// Returns:
    ///     A dict with "expression", "steps", "functions_used", and "complexity"
    fn explain(&self, py: Python<'_>, expression: &str) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .explain(expression)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let json =
            serde_json::to_value(&result).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &json)
    }

    // =========================================================================
    // Introspection
    // =========================================================================

    /// List all available function categories.
    ///
    /// Returns:
    ///     A list of category name strings
    fn categories(&self) -> Vec<String> {
        self.inner.categories()
    }

    /// List functions, optionally filtered by category.
    ///
    /// Args:
    ///     category: Optional category name to filter by (case-insensitive)
    ///
    /// Returns:
    ///     A list of dicts with function details
    #[pyo3(signature = (category=None))]
    fn functions(&self, py: Python<'_>, category: Option<&str>) -> PyResult<Py<PyAny>> {
        let funcs = self.inner.functions(category);
        let json =
            serde_json::to_value(&funcs).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &json)
    }

    /// Get detailed information about a function by name.
    ///
    /// Args:
    ///     name: The function name
    ///
    /// Returns:
    ///     A dict with function details, or None if not found
    fn describe_function(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.describe_function(name) {
            Some(detail) => {
                let json = serde_json::to_value(&detail)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(Some(json_to_python(py, &json)?))
            }
            None => Ok(None),
        }
    }

    /// Search for functions matching a query string.
    ///
    /// Args:
    ///     query: Search term (e.g., "hash", "string", "date")
    ///     limit: Maximum number of results (default 10)
    ///
    /// Returns:
    ///     A list of dicts with function details, match_type, and score
    #[pyo3(signature = (query, limit=10))]
    fn search_functions(&self, py: Python<'_>, query: &str, limit: usize) -> PyResult<Py<PyAny>> {
        let results = self.inner.search_functions(query, limit);
        let json =
            serde_json::to_value(&results).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &json)
    }

    /// Find functions similar to a given function.
    ///
    /// Args:
    ///     name: The function name to find similar functions for
    ///
    /// Returns:
    ///     A dict with "same_category", "similar_signature", and "related_concepts" lists,
    ///     or None if the function is not found
    fn similar_functions(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.similar_functions(name) {
            Some(result) => {
                let json = serde_json::to_value(&result)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(Some(json_to_python(py, &json)?))
            }
            None => Ok(None),
        }
    }

    // =========================================================================
    // JSON utilities
    // =========================================================================

    /// Format JSON with configurable indentation.
    ///
    /// Args:
    ///     json_str: JSON string to format
    ///     indent: Number of spaces per indent level (default 2, 0 = compact)
    ///
    /// Returns:
    ///     Formatted JSON string
    #[pyo3(signature = (json_str, indent=2))]
    fn format_json(&self, json_str: &str, indent: usize) -> PyResult<String> {
        self.inner
            .format_json(json_str, indent)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Generate a JSON Patch (RFC 6902) from two JSON strings.
    ///
    /// Args:
    ///     source: Original JSON string
    ///     target: Modified JSON string
    ///
    /// Returns:
    ///     The patch as a Python object (list of operations)
    #[pyo3(signature = (source, target))]
    fn diff(&self, py: Python<'_>, source: &str, target: &str) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .diff(source, target)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &result)
    }

    /// Apply a JSON Patch (RFC 6902) to a document.
    ///
    /// Args:
    ///     json_str: JSON document string
    ///     patch_str: JSON array of patch operations
    ///
    /// Returns:
    ///     The patched document as a Python object
    #[pyo3(signature = (json_str, patch_str))]
    fn patch(&self, py: Python<'_>, json_str: &str, patch_str: &str) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .patch(json_str, patch_str)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &result)
    }

    /// Apply a JSON Merge Patch (RFC 7396) to a document.
    ///
    /// Args:
    ///     json_str: JSON document string
    ///     patch_str: JSON merge patch object
    ///
    /// Returns:
    ///     The merged document as a Python object
    #[pyo3(signature = (json_str, patch_str))]
    fn merge(&self, py: Python<'_>, json_str: &str, patch_str: &str) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .merge(json_str, patch_str)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &result)
    }

    /// Analyze JSON data and return structural statistics.
    ///
    /// Args:
    ///     json_str: JSON string to analyze
    ///
    /// Returns:
    ///     A dict with root_type, size_bytes, size_human, depth, and more
    fn stats(&self, py: Python<'_>, json_str: &str) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .stats(json_str)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let json =
            serde_json::to_value(&result).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &json)
    }

    /// Extract all paths from a JSON document.
    ///
    /// Args:
    ///     json_str: JSON string to analyze
    ///     include_types: Include type info for each path (default True)
    ///     include_values: Include leaf values (default False)
    ///
    /// Returns:
    ///     A list of dicts with "path", optionally "path_type" and "value"
    #[pyo3(signature = (json_str, include_types=true, include_values=false))]
    fn paths(
        &self,
        py: Python<'_>,
        json_str: &str,
        include_types: bool,
        include_values: bool,
    ) -> PyResult<Py<PyAny>> {
        let result = self
            .inner
            .paths(json_str, include_types, include_values)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let json =
            serde_json::to_value(&result).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_to_python(py, &json)
    }

    /// Extract keys from a JSON object.
    ///
    /// Args:
    ///     json_str: JSON string (must be an object for non-recursive mode)
    ///     recursive: If True, extract all nested paths in dot notation (default False)
    ///
    /// Returns:
    ///     A list of key strings
    #[pyo3(signature = (json_str, recursive=false))]
    fn keys(&self, json_str: &str, recursive: bool) -> PyResult<Vec<String>> {
        self.inner
            .keys(json_str, recursive)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    // =========================================================================
    // Query store
    // =========================================================================

    /// Store a named query for reuse.
    ///
    /// Args:
    ///     name: Query name
    ///     expression: JMESPath expression string
    ///     description: Optional description
    ///
    /// Returns:
    ///     The previously stored query dict if one existed with this name, else None
    #[pyo3(signature = (name, expression, description=None))]
    fn define_query(
        &self,
        py: Python<'_>,
        name: String,
        expression: String,
        description: Option<String>,
    ) -> PyResult<Option<Py<PyAny>>> {
        let prev = self
            .inner
            .define_query(name, expression, description)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        match prev {
            Some(q) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("name", &q.name)?;
                dict.set_item("expression", &q.expression)?;
                match &q.description {
                    Some(d) => dict.set_item("description", d)?,
                    None => dict.set_item("description", py.None())?,
                }
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Get a stored query by name.
    ///
    /// Args:
    ///     name: Query name
    ///
    /// Returns:
    ///     A dict with "name", "expression", "description", or None if not found
    fn get_query(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
        let query = self
            .inner
            .get_query(name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        match query {
            Some(q) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("name", &q.name)?;
                dict.set_item("expression", &q.expression)?;
                match &q.description {
                    Some(d) => dict.set_item("description", d)?,
                    None => dict.set_item("description", py.None())?,
                }
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Run a stored query against data.
    ///
    /// Args:
    ///     name: Query name
    ///     data: JSON data as a Python object
    ///
    /// Returns:
    ///     The result of evaluating the stored expression against the data
    ///
    /// Raises:
    ///     ValueError: If the query is not found or evaluation fails
    #[pyo3(signature = (name, data))]
    fn run_query(&self, name: &str, data: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let json_value = python_to_json(data)?;
        let result = self
            .inner
            .run_query(name, &json_value)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Python::attach(|py| json_to_python(py, &result))
    }

    /// List all stored queries.
    ///
    /// Returns:
    ///     A list of dicts with "name", "expression", "description"
    fn list_queries(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let queries = self
            .inner
            .list_queries()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let result_list = pyo3::types::PyList::empty(py);
        for q in &queries {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", &q.name)?;
            dict.set_item("expression", &q.expression)?;
            match &q.description {
                Some(d) => dict.set_item("description", d)?,
                None => dict.set_item("description", py.None())?,
            }
            result_list.append(dict)?;
        }

        Ok(result_list.into())
    }

    /// Delete a stored query.
    ///
    /// Args:
    ///     name: Query name
    ///
    /// Returns:
    ///     The deleted query dict if it existed, else None
    fn delete_query(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
        let query = self
            .inner
            .delete_query(name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        match query {
            Some(q) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("name", &q.name)?;
                dict.set_item("expression", &q.expression)?;
                match &q.description {
                    Some(d) => dict.set_item("description", d)?,
                    None => dict.set_item("description", py.None())?,
                }
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    fn __repr__(&self) -> String {
        if self.inner.is_strict() {
            "JpxEngine(strict=True)".to_string()
        } else {
            "JpxEngine()".to_string()
        }
    }
}

// =============================================================================
// Module definition
// =============================================================================

/// jpx Python module - JMESPath query engine with 490+ functions.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Module-level convenience functions
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(compile_expr, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(list_functions, m)?)?;
    m.add_function(wrap_pyfunction!(list_categories, m)?)?;
    m.add_function(wrap_pyfunction!(describe, m)?)?;

    // Classes
    m.add_class::<CompiledExpression>()?;
    m.add_class::<JpxEngine>()?;

    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
