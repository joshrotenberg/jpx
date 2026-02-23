//! Regular expression functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::functions::custom_error;
use crate::functions::number_value;
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
    register_if_enabled(
        runtime,
        "regex_count",
        enabled,
        Box::new(RegexCountFn::new()),
    );
    register_if_enabled(
        runtime,
        "regex_split",
        enabled,
        Box::new(RegexSplitFn::new()),
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

// =============================================================================
// regex_count(string, pattern) -> number
// =============================================================================

defn!(RegexCountFn, vec![arg!(string), arg!(string)], None);

impl Function for RegexCountFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // Safe to unwrap after signature validation
        let input = args[0].as_str().unwrap();
        let pattern = args[1].as_str().unwrap();

        let re = Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {e}")))?;

        let count = re.find_iter(input).count();
        Ok(number_value(count as f64))
    }
}

// =============================================================================
// regex_split(string, pattern) -> array
// =============================================================================

defn!(RegexSplitFn, vec![arg!(string), arg!(string)], None);

impl Function for RegexSplitFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // Safe to unwrap after signature validation
        let input = args[0].as_str().unwrap();
        let pattern = args[1].as_str().unwrap();

        let re = Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {e}")))?;

        let parts: Vec<Value> = re
            .split(input)
            .map(|s| Value::String(s.to_string()))
            .collect();

        Ok(Value::Array(parts))
    }
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
    fn test_regex_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_match(@, '^hello')").unwrap();

        let data = json!("hello world");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("world hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_regex_extract() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_extract(@, '[0-9]+')").unwrap();
        let data = json!("abc123def456");
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().unwrap(), "123");
        assert_eq!(arr[1].as_str().unwrap(), "456");
    }

    #[test]
    fn test_regex_replace() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_replace(@, '[0-9]+', 'X')").unwrap();
        let data = json!("abc123def456");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "abcXdefX");
    }

    #[test]
    fn test_regex_count() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_count(@, '[0-9]+')").unwrap();

        let data = json!("abc123def456ghi789");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn test_regex_count_no_matches() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_count(@, '[0-9]+')").unwrap();

        let data = json!("abcdef");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(0.0));
    }

    #[test]
    fn test_regex_count_overlapping_pattern() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_count(@, '[aeiou]')").unwrap();

        let data = json!("hello world");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn test_regex_split() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_split(@, ',')").unwrap();

        let data = json!("a,b,c");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["a", "b", "c"]));
    }

    #[test]
    fn test_regex_split_whitespace() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_split(@, '\\s+')").unwrap();

        let data = json!("hello   world\tfoo");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["hello", "world", "foo"]));
    }

    #[test]
    fn test_regex_split_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_split(@, ',')").unwrap();

        let data = json!("no delimiters here");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["no delimiters here"]));
    }

    #[test]
    fn test_regex_match_invalid_pattern() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_match(@, '[invalid')").unwrap();
        let result = expr.search(&json!("test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_extract_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_extract(@, '[0-9]+')").unwrap();
        let data = json!("no numbers here");
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_regex_replace_capture_groups() {
        let runtime = setup_runtime();
        // Swap first and last name using capture groups
        let expr = runtime
            .compile(r#"regex_replace(@, '(\w+) (\w+)', '$2 $1')"#)
            .unwrap();
        let data = json!("John Doe");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "Doe John");
    }

    #[test]
    fn test_regex_replace_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_replace(@, '[0-9]+', 'X')").unwrap();
        let data = json!("no numbers");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "no numbers");
    }

    #[test]
    fn test_regex_match_anchored() {
        let runtime = setup_runtime();
        // Full string match with anchors
        let expr = runtime.compile(r"regex_match(@, '^\d{3}-\d{4}$')").unwrap();
        assert_eq!(expr.search(&json!("123-4567")).unwrap(), json!(true));
        assert_eq!(expr.search(&json!("abc-defg")).unwrap(), json!(false));
        assert_eq!(expr.search(&json!("123-45678")).unwrap(), json!(false));
    }

    #[test]
    fn test_regex_extract_email_pattern() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile(r"regex_extract(@, '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}')")
            .unwrap();
        let data = json!("Contact us at info@example.com or support@test.org");
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().unwrap(), "info@example.com");
        assert_eq!(arr[1].as_str().unwrap(), "support@test.org");
    }

    #[test]
    fn test_regex_split_complex_delimiter() {
        let runtime = setup_runtime();
        // Split on comma optionally followed by whitespace
        let expr = runtime.compile(r"regex_split(@, ',\s*')").unwrap();
        let data = json!("a, b,c,  d");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(["a", "b", "c", "d"]));
    }

    #[test]
    fn test_regex_count_invalid_pattern() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_count(@, '[bad')").unwrap();
        let result = expr.search(&json!("test"));
        assert!(result.is_err());
    }
}
