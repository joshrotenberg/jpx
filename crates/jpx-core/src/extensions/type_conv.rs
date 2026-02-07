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
