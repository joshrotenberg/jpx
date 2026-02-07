//! Regular expression functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::functions::custom_error;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

use regex::Regex;

/// Register regex functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "regex_match",
        enabled,
        Box::new(RegexMatchFn::new()),
    );
    register_if_enabled(
        runtime,
        "regex_extract",
        enabled,
        Box::new(RegexExtractFn::new()),
    );
    register_if_enabled(
        runtime,
        "regex_replace",
        enabled,
        Box::new(RegexReplaceFn::new()),
    );
}

// =============================================================================
// regex_match(string, pattern) -> boolean
// =============================================================================

defn!(RegexMatchFn, vec![arg!(string), arg!(string)], None);

impl Function for RegexMatchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // Safe to unwrap after signature validation
        let input = args[0].as_str().unwrap();
        let pattern = args[1].as_str().unwrap();

        let re = Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {e}")))?;

        Ok(Value::Bool(re.is_match(input)))
    }
}

// =============================================================================
// regex_extract(string, pattern) -> array of matches
// =============================================================================

defn!(RegexExtractFn, vec![arg!(string), arg!(string)], None);

impl Function for RegexExtractFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // Safe to unwrap after signature validation
        let input = args[0].as_str().unwrap();
        let pattern = args[1].as_str().unwrap();

        let re = Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {e}")))?;

        let matches: Vec<Value> = re
            .find_iter(input)
            .map(|m| Value::String(m.as_str().to_string()))
            .collect();

        // Return null if no matches found
        if matches.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(Value::Array(matches))
        }
    }
}

// =============================================================================
// regex_replace(string, pattern, replacement) -> string
// =============================================================================

defn!(
    RegexReplaceFn,
    vec![arg!(string), arg!(string), arg!(string)],
    None
);

impl Function for RegexReplaceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // Safe to unwrap after signature validation
        let input = args[0].as_str().unwrap();
        let pattern = args[1].as_str().unwrap();
        let replacement = args[2].as_str().unwrap();

        let re = Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {e}")))?;

        let result = re.replace_all(input, replacement);
        Ok(Value::String(result.into_owned()))
    }
}
