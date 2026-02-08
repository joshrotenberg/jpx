//! ID generation functions (nanoid, ulid).

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// nanoid(size?) -> string
// =============================================================================

defn!(NanoidFn, vec![], Some(arg!(number)));

impl Function for NanoidFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let id = if args.is_empty() {
            nanoid::nanoid!()
        } else {
            let size = args[0].as_f64().unwrap_or(21.0) as usize;
            nanoid::nanoid!(size)
        };

        Ok(Value::String(id))
    }
}

// =============================================================================
// ulid() -> string
// =============================================================================

defn!(UlidFn, vec![], None);

impl Function for UlidFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let id = ulid::Ulid::new().to_string();
        Ok(Value::String(id))
    }
}

// =============================================================================
// ulid_timestamp(ulid) -> number (unix ms)
// =============================================================================

defn!(UlidTimestampFn, vec![arg!(string)], None);

impl Function for UlidTimestampFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let ulid_str = args[0].as_str().unwrap();

        match ulid::Ulid::from_string(ulid_str) {
            Ok(id) => {
                let ts = id.timestamp_ms();
                Ok(number_value(ts as f64))
            }
            Err(_) => Ok(Value::Null),
        }
    }
}

/// Register ID functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "nanoid", enabled, Box::new(NanoidFn::new()));
    register_if_enabled(runtime, "ulid", enabled, Box::new(UlidFn::new()));
    register_if_enabled(
        runtime,
        "ulid_timestamp",
        enabled,
        Box::new(UlidTimestampFn::new()),
    );
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
    fn test_nanoid_default() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("nanoid()").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_str().unwrap();
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn test_nanoid_custom_size() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("nanoid(`10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_str().unwrap();
        assert_eq!(id.len(), 10);
    }

    #[test]
    fn test_nanoid_unique() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("nanoid()").unwrap();
        let id1 = expr.search(&data).unwrap();
        let id2 = expr.search(&data).unwrap();
        assert_ne!(id1.as_str().unwrap(), id2.as_str().unwrap());
    }

    #[test]
    fn test_ulid() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("ulid()").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_str().unwrap();
        // ULID is 26 characters
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn test_ulid_unique() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("ulid()").unwrap();
        let id1 = expr.search(&data).unwrap();
        let id2 = expr.search(&data).unwrap();
        assert_ne!(id1.as_str().unwrap(), id2.as_str().unwrap());
    }

    #[test]
    fn test_ulid_timestamp() {
        let runtime = setup_runtime();
        // Generate a ULID and extract its timestamp
        let ulid = ulid::Ulid::new();
        let ulid_str = ulid.to_string();
        let expected_ts = ulid.timestamp_ms();

        let data = json!(ulid_str);
        let expr = runtime.compile("ulid_timestamp(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), expected_ts as f64);
    }

    #[test]
    fn test_ulid_timestamp_invalid() {
        let runtime = setup_runtime();
        let data = json!("not-a-ulid");
        let expr = runtime.compile("ulid_timestamp(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_ulid_format() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("ulid()").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_str().unwrap();

        // ULID should be 26 characters of Crockford's Base32
        assert_eq!(id.len(), 26);
        // All characters should be valid Base32
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
