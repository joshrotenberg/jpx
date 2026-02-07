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
