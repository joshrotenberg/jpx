//! Type checking and conversion functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::value_ext::ValueExt;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// to_string(any) -> string
// =============================================================================

defn!(ToStringFn, vec![arg!(any)], None);

impl Function for ToStringFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let result = match &args[0] {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(&args[0]).unwrap_or_else(|_| "null".to_string()),
        };

        Ok(Value::String(result))
    }
}

// =============================================================================
// to_number(any) -> number
// =============================================================================

defn!(ToNumberFn, vec![arg!(any)], None);

impl Function for ToNumberFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let result = match &args[0] {
            Value::Number(n) => Some(n.clone()),
            Value::String(s) => s.parse::<f64>().ok().and_then(serde_json::Number::from_f64),
            Value::Bool(b) => Some(serde_json::Number::from(if *b { 1 } else { 0 })),
            _ => None,
        };

        match result {
            Some(n) => Ok(Value::Number(n)),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// to_boolean(any) -> boolean
// =============================================================================

defn!(ToBooleanFn, vec![arg!(any)], None);

impl Function for ToBooleanFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let result = match &args[0] {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Value::Array(a) => !a.is_empty(),
            Value::Object(_) => {
                if args[0].is_expref() {
                    true
                } else {
                    !args[0].as_object().unwrap().is_empty()
                }
            }
        };

        Ok(Value::Bool(result))
    }
}

// =============================================================================
// type_of(any) -> string
// =============================================================================

defn!(TypeOfFn, vec![arg!(any)], None);

impl Function for TypeOfFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let type_name = args[0].jmespath_type().to_string();
        Ok(Value::String(type_name))
    }
}

// =============================================================================
// is_string(any) -> boolean
// =============================================================================

defn!(IsStringFn, vec![arg!(any)], None);

impl Function for IsStringFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::Bool(args[0].is_string()))
    }
}

// =============================================================================
// is_number(any) -> boolean
// =============================================================================

defn!(IsNumberFn, vec![arg!(any)], None);

impl Function for IsNumberFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::Bool(args[0].is_number()))
    }
}

// =============================================================================
// is_boolean(any) -> boolean
// =============================================================================

defn!(IsBooleanFn, vec![arg!(any)], None);

impl Function for IsBooleanFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::Bool(args[0].is_boolean()))
    }
}

// =============================================================================
// is_array(any) -> boolean
// =============================================================================

defn!(IsArrayFn, vec![arg!(any)], None);

impl Function for IsArrayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::Bool(args[0].is_array()))
    }
}

// =============================================================================
// is_object(any) -> boolean
// =============================================================================

defn!(IsObjectFn, vec![arg!(any)], None);

impl Function for IsObjectFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::Bool(args[0].is_object()))
    }
}

// =============================================================================
// is_null(any) -> boolean
// =============================================================================

defn!(IsNullFn, vec![arg!(any)], None);

impl Function for IsNullFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::Bool(args[0].is_null()))
    }
}

// =============================================================================
// is_empty(any) -> boolean (empty string, array, or object)
// =============================================================================

defn!(IsEmptyFn, vec![arg!(any)], None);

impl Function for IsEmptyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let is_empty = match &args[0] {
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            Value::Null => true,
            _ => false,
        };

        Ok(Value::Bool(is_empty))
    }
}

// =============================================================================
// is_blank(string) -> boolean (empty or whitespace only)
// =============================================================================

defn!(IsBlankFn, vec![arg!(any)], None);

impl Function for IsBlankFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        match &args[0] {
            Value::String(s) => Ok(Value::Bool(s.trim().is_empty())),
            Value::Null => Ok(Value::Bool(true)),
            // Return null for non-string types
            _ => Ok(Value::Null),
        }
    }
}

// =============================================================================
// is_json(any) -> boolean|null (valid JSON string)
// =============================================================================

defn!(IsJsonFn, vec![arg!(any)], None);

impl Function for IsJsonFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // Return null for non-string types
        let s = match args[0].as_str() {
            Some(s) => s,
            None => return Ok(Value::Null),
        };

        let is_valid = serde_json::from_str::<Value>(s).is_ok();
        Ok(Value::Bool(is_valid))
    }
}

// =============================================================================
// parse_numbers(any) -> any (recursively convert numeric strings to numbers)
// =============================================================================

defn!(ParseNumbersFn, vec![arg!(any)], None);

impl Function for ParseNumbersFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(parse_numbers_recursive(&args[0]))
    }
}

fn parse_numbers_recursive(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            // Try to parse as number - be conservative, only pure numeric strings
            let trimmed = s.trim();
            if let Ok(n) = trimmed.parse::<f64>()
                && let Some(num) = serde_json::Number::from_f64(n)
            {
                return Value::Number(num);
            }
            value.clone()
        }
        Value::Object(obj) => {
            let parsed: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), parse_numbers_recursive(v)))
                .collect();
            Value::Object(parsed)
        }
        Value::Array(arr) => {
            let parsed: Vec<Value> = arr.iter().map(parse_numbers_recursive).collect();
            Value::Array(parsed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// parse_booleans(any) -> any (recursively convert boolean strings to booleans)
// =============================================================================

defn!(ParseBooleansFn, vec![arg!(any)], None);

impl Function for ParseBooleansFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(parse_booleans_recursive(&args[0]))
    }
}

fn parse_booleans_recursive(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            let lower = s.trim().to_lowercase();
            match lower.as_str() {
                "true" | "yes" | "on" | "1" => Value::Bool(true),
                "false" | "no" | "off" | "0" => Value::Bool(false),
                _ => value.clone(),
            }
        }
        Value::Object(obj) => {
            let parsed: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), parse_booleans_recursive(v)))
                .collect();
            Value::Object(parsed)
        }
        Value::Array(arr) => {
            let parsed: Vec<Value> = arr.iter().map(parse_booleans_recursive).collect();
            Value::Array(parsed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// parse_nulls(any) -> any (recursively convert null-like strings to null)
// =============================================================================

defn!(ParseNullsFn, vec![arg!(any)], None);

impl Function for ParseNullsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(parse_nulls_recursive(&args[0]))
    }
}

fn parse_nulls_recursive(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            let lower = s.trim().to_lowercase();
            match lower.as_str() {
                "null" | "none" | "nil" | "undefined" => Value::Null,
                _ => value.clone(),
            }
        }
        Value::Object(obj) => {
            let parsed: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), parse_nulls_recursive(v)))
                .collect();
            Value::Object(parsed)
        }
        Value::Array(arr) => {
            let parsed: Vec<Value> = arr.iter().map(parse_nulls_recursive).collect();
            Value::Array(parsed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// auto_parse(any) -> any (intelligently parse numbers, booleans, and nulls)
// =============================================================================

defn!(AutoParseFn, vec![arg!(any)], None);

impl Function for AutoParseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(auto_parse_recursive(&args[0]))
    }
}

fn auto_parse_recursive(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            let lower = trimmed.to_lowercase();

            // Try null-like values first
            if matches!(lower.as_str(), "null" | "none" | "nil" | "undefined") {
                return Value::Null;
            }

            // Try boolean values
            if matches!(lower.as_str(), "true" | "yes" | "on") {
                return Value::Bool(true);
            }
            if matches!(lower.as_str(), "false" | "no" | "off") {
                return Value::Bool(false);
            }

            // Try numeric values
            if let Ok(n) = trimmed.parse::<f64>()
                && let Some(num) = serde_json::Number::from_f64(n)
            {
                return Value::Number(num);
            }

            value.clone()
        }
        Value::Object(obj) => {
            let parsed: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), auto_parse_recursive(v)))
                .collect();
            Value::Object(parsed)
        }
        Value::Array(arr) => {
            let parsed: Vec<Value> = arr.iter().map(auto_parse_recursive).collect();
            Value::Array(parsed)
        }
        _ => value.clone(),
    }
}

/// Register type functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "to_string", enabled, Box::new(ToStringFn::new()));
    register_if_enabled(runtime, "to_number", enabled, Box::new(ToNumberFn::new()));
    register_if_enabled(runtime, "to_boolean", enabled, Box::new(ToBooleanFn::new()));
    register_if_enabled(runtime, "type_of", enabled, Box::new(TypeOfFn::new()));
    register_if_enabled(runtime, "is_string", enabled, Box::new(IsStringFn::new()));
    register_if_enabled(runtime, "is_number", enabled, Box::new(IsNumberFn::new()));
    register_if_enabled(runtime, "is_boolean", enabled, Box::new(IsBooleanFn::new()));
    register_if_enabled(runtime, "is_array", enabled, Box::new(IsArrayFn::new()));
    register_if_enabled(runtime, "is_object", enabled, Box::new(IsObjectFn::new()));
    register_if_enabled(runtime, "is_null", enabled, Box::new(IsNullFn::new()));
    register_if_enabled(runtime, "is_empty", enabled, Box::new(IsEmptyFn::new()));
    register_if_enabled(runtime, "is_blank", enabled, Box::new(IsBlankFn::new()));
    register_if_enabled(runtime, "is_json", enabled, Box::new(IsJsonFn::new()));
    register_if_enabled(
        runtime,
        "parse_numbers",
        enabled,
        Box::new(ParseNumbersFn::new()),
    );
    register_if_enabled(
        runtime,
        "parse_booleans",
        enabled,
        Box::new(ParseBooleansFn::new()),
    );
    register_if_enabled(
        runtime,
        "parse_nulls",
        enabled,
        Box::new(ParseNullsFn::new()),
    );
    register_if_enabled(runtime, "auto_parse", enabled, Box::new(AutoParseFn::new()));
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::json;

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_type_of() {
        let runtime = setup_runtime();
        let expr = runtime.compile("type_of(@)").unwrap();

        let result = expr.search(&json!("hello")).unwrap();
        assert_eq!(result.as_str().unwrap(), "string");

        let result = expr.search(&json!(42)).unwrap();
        assert_eq!(result.as_str().unwrap(), "number");
    }

    #[test]
    fn test_is_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_empty(@)").unwrap();

        let result = expr.search(&json!("")).unwrap();
        assert!(result.as_bool().unwrap());

        let result = expr.search(&json!("hello")).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    // =========================================================================
    // parse_numbers tests
    // =========================================================================

    #[test]
    fn test_parse_numbers_basic() {
        let runtime = setup_runtime();
        let data = json!({"count": "42", "price": "19.99", "name": "test"});
        let expr = runtime.compile("parse_numbers(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result["count"].as_f64().unwrap() as i64, 42);
        assert!((result["price"].as_f64().unwrap() - 19.99).abs() < 0.01);
        assert_eq!(result["name"].as_str().unwrap(), "test");
    }

    #[test]
    fn test_parse_numbers_nested() {
        let runtime = setup_runtime();
        let data = json!({"outer": {"inner": "123"}});
        let expr = runtime.compile("parse_numbers(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result["outer"]["inner"].as_f64().unwrap() as i64, 123);
    }

    #[test]
    fn test_parse_numbers_non_numeric() {
        let runtime = setup_runtime();
        let data = json!({"val": "42abc"});
        let expr = runtime.compile("parse_numbers(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // Should remain as string since it's not a pure number
        assert_eq!(result["val"].as_str().unwrap(), "42abc");
    }

    // =========================================================================
    // parse_booleans tests
    // =========================================================================

    #[test]
    fn test_parse_booleans_basic() {
        let runtime = setup_runtime();
        let data = json!({"active": "true", "verified": "false", "name": "test"});
        let expr = runtime.compile("parse_booleans(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result["active"].as_bool().unwrap());
        assert!(!result["verified"].as_bool().unwrap());
        assert_eq!(result["name"].as_str().unwrap(), "test");
    }

    #[test]
    fn test_parse_booleans_variants() {
        let runtime = setup_runtime();
        let data = json!({"a": "YES", "b": "no", "c": "ON", "d": "off", "e": "1", "f": "0"});
        let expr = runtime.compile("parse_booleans(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result["a"].as_bool().unwrap());
        assert!(!result["b"].as_bool().unwrap());
        assert!(result["c"].as_bool().unwrap());
        assert!(!result["d"].as_bool().unwrap());
        assert!(result["e"].as_bool().unwrap());
        assert!(!result["f"].as_bool().unwrap());
    }

    // =========================================================================
    // parse_nulls tests
    // =========================================================================

    #[test]
    fn test_parse_nulls_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": "null", "b": "NULL", "c": "None", "d": "nil", "e": "hello"});
        let expr = runtime.compile("parse_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result["a"].is_null());
        assert!(result["b"].is_null());
        assert!(result["c"].is_null());
        assert!(result["d"].is_null());
        assert_eq!(result["e"].as_str().unwrap(), "hello");
    }

    // =========================================================================
    // auto_parse tests
    // =========================================================================

    #[test]
    fn test_auto_parse_mixed() {
        let runtime = setup_runtime();
        let data = json!({"num": "42", "bool": "true", "nil": "null", "str": "hello"});
        let expr = runtime.compile("auto_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result["num"].as_f64().unwrap() as i64, 42);
        assert!(result["bool"].as_bool().unwrap());
        assert!(result["nil"].is_null());
        assert_eq!(result["str"].as_str().unwrap(), "hello");
    }

    #[test]
    fn test_auto_parse_array() {
        let runtime = setup_runtime();
        let data = json!(["42", "true", "null", "hello"]);
        let expr = runtime.compile("auto_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_f64().unwrap() as i64, 42);
        assert!(arr[1].as_bool().unwrap());
        assert!(arr[2].is_null());
        assert_eq!(arr[3].as_str().unwrap(), "hello");
    }
}
