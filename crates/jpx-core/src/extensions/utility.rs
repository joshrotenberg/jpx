//! Utility functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// now(fallback?) -> number (Unix timestamp in seconds)
// =============================================================================

defn!(NowFn, vec![], Some(arg!(number)));

impl Function for NowFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        if let Some(fallback) = args.first()
            && let Some(n) = fallback.as_f64()
        {
            return Ok(Value::Number(
                Number::from_f64(n).unwrap_or_else(|| Number::from(0)),
            ));
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Value::Number(Number::from(timestamp)))
    }
}

// =============================================================================
// now_ms(fallback?) -> number (Unix timestamp in milliseconds)
// =============================================================================

defn!(NowMsFn, vec![], Some(arg!(number)));

impl Function for NowMsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        if let Some(fallback) = args.first()
            && let Some(n) = fallback.as_f64()
        {
            return Ok(Value::Number(
                Number::from_f64(n).unwrap_or_else(|| Number::from(0)),
            ));
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(Value::Number(Number::from(timestamp)))
    }
}

// =============================================================================
// default(value, default_value) -> value if not null, else default
// =============================================================================

defn!(DefaultFn, vec![arg!(any), arg!(any)], None);

impl Function for DefaultFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        if args[0].is_null() {
            Ok(args[1].clone())
        } else {
            Ok(args[0].clone())
        }
    }
}

// =============================================================================
// if(condition, then_value, else_value) -> any
// =============================================================================

defn!(IfFn, vec![arg!(any), arg!(any), arg!(any)], None);

impl Function for IfFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let condition = &args[0];
        let then_value = &args[1];
        let else_value = &args[2];

        let is_truthy = match condition {
            Value::Bool(b) => *b,
            Value::Null => false,
            _ => true,
        };

        if is_truthy {
            Ok(then_value.clone())
        } else {
            Ok(else_value.clone())
        }
    }
}

// =============================================================================
// coalesce(...) -> any (first non-null value)
// =============================================================================

defn!(CoalesceFn, vec![], Some(arg!(any)));

impl Function for CoalesceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        for arg in args {
            if !arg.is_null() {
                return Ok(arg.clone());
            }
        }
        Ok(Value::Null)
    }
}

// =============================================================================
// json_encode(any) -> string
// =============================================================================

defn!(JsonEncodeFn, vec![arg!(any)], None);

impl Function for JsonEncodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let json_str = serde_json::to_string(&args[0])
            .map_err(|_| crate::functions::custom_error(ctx, "Failed to encode as JSON"))?;

        Ok(Value::String(json_str))
    }
}

// =============================================================================
// pretty(any, indent?) -> string
// =============================================================================

defn!(PrettyFn, vec![arg!(any)], Some(arg!(number)));

impl Function for PrettyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let indent = if args.len() > 1 {
            args[1].as_f64().unwrap_or(2.0) as usize
        } else {
            2
        };

        // For default indent of 2, use built-in to_string_pretty
        if indent == 2 {
            let pretty_str = serde_json::to_string_pretty(&args[0])
                .map_err(|_| crate::functions::custom_error(ctx, "Failed to serialize as JSON"))?;
            return Ok(Value::String(pretty_str));
        }

        // For custom indent, manually format
        let json_str = serde_json::to_string(&args[0])
            .map_err(|_| crate::functions::custom_error(ctx, "Failed to serialize as JSON"))?;

        let pretty_str = pretty_print_json(&json_str, indent);
        Ok(Value::String(pretty_str))
    }
}

/// Pretty print JSON with custom indentation.
fn pretty_print_json(json: &str, indent_size: usize) -> String {
    let mut result = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let indent = " ".repeat(indent_size);

    for ch in json.chars() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }

        if in_string {
            result.push(ch);
            continue;
        }

        match ch {
            '{' | '[' => {
                result.push(ch);
                depth += 1;
                result.push('\n');
                for _ in 0..depth {
                    result.push_str(&indent);
                }
            }
            '}' | ']' => {
                depth -= 1;
                result.push('\n');
                for _ in 0..depth {
                    result.push_str(&indent);
                }
                result.push(ch);
            }
            ',' => {
                result.push(ch);
                result.push('\n');
                for _ in 0..depth {
                    result.push_str(&indent);
                }
            }
            ':' => {
                result.push_str(": ");
            }
            ' ' | '\n' | '\t' | '\r' => {
                // Skip whitespace in compact JSON
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

// =============================================================================
// json_decode(string) -> any
// =============================================================================

defn!(JsonDecodeFn, vec![arg!(string)], None);

impl Function for JsonDecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string argument"))?;

        // Return null for invalid JSON instead of erroring
        match serde_json::from_str::<Value>(s) {
            Ok(val) => Ok(val),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// json_pointer(any, string) -> any (RFC 6901 JSON Pointer)
// =============================================================================

defn!(JsonPointerFn, vec![arg!(any), arg!(string)], None);

impl Function for JsonPointerFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let pointer = args[1].as_str().ok_or_else(|| {
            crate::functions::custom_error(ctx, "Expected string pointer argument")
        })?;

        // Use serde_json's built-in pointer method directly on the Value
        match args[0].pointer(pointer) {
            Some(result) => Ok(result.clone()),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// env() -> object (all environment variables)
// =============================================================================

defn!(EnvFn, vec![], None);

impl Function for EnvFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut map = serde_json::Map::new();
        for (key, value) in std::env::vars() {
            map.insert(key, Value::String(value));
        }

        Ok(Value::Object(map))
    }
}

// =============================================================================
// get_env(name) -> string | null (single environment variable)
// =============================================================================

defn!(GetEnvFn, vec![arg!(string)], None);

impl Function for GetEnvFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let name = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string argument"))?;

        match std::env::var(name) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Ok(Value::Null),
        }
    }
}

/// Register utility functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "now", enabled, Box::new(NowFn::new()));
    register_if_enabled(runtime, "now_ms", enabled, Box::new(NowMsFn::new()));
    register_if_enabled(runtime, "default", enabled, Box::new(DefaultFn::new()));
    register_if_enabled(runtime, "if", enabled, Box::new(IfFn::new()));
    register_if_enabled(runtime, "coalesce", enabled, Box::new(CoalesceFn::new()));
    register_if_enabled(
        runtime,
        "json_encode",
        enabled,
        Box::new(JsonEncodeFn::new()),
    );
    register_if_enabled(runtime, "to_json", enabled, Box::new(JsonEncodeFn::new()));
    register_if_enabled(
        runtime,
        "json_decode",
        enabled,
        Box::new(JsonDecodeFn::new()),
    );
    register_if_enabled(
        runtime,
        "json_pointer",
        enabled,
        Box::new(JsonPointerFn::new()),
    );
    register_if_enabled(runtime, "pretty", enabled, Box::new(PrettyFn::new()));
    register_if_enabled(runtime, "env", enabled, Box::new(EnvFn::new()));
    register_if_enabled(runtime, "get_env", enabled, Box::new(GetEnvFn::new()));
}
