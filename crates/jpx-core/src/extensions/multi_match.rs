//! Multi-pattern matching functions.

use std::collections::HashSet;

use aho_corasick::AhoCorasick;
use serde_json::{Number, Value};

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// match_any(string, patterns) -> boolean
// Returns true if any of the patterns match the string
defn!(MatchAnyFn, vec![arg!(string), arg!(array)], None);

impl Function for MatchAnyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr.iter().filter_map(|p| p.as_str()).collect();

        if patterns.is_empty() {
            return Ok(Value::Bool(false));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let has_match = ac.find(text).is_some();

        Ok(Value::Bool(has_match))
    }
}

// match_all(string, patterns) -> boolean
// Returns true if all patterns match the string
defn!(MatchAllFn, vec![arg!(string), arg!(array)], None);

impl Function for MatchAllFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr.iter().filter_map(|p| p.as_str()).collect();

        if patterns.is_empty() {
            return Ok(Value::Bool(true));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();

        let mut found = vec![false; patterns.len()];

        for mat in ac.find_iter(text) {
            found[mat.pattern().as_usize()] = true;
        }

        let all_found = found.iter().all(|&f| f);
        Ok(Value::Bool(all_found))
    }
}

// match_which(string, patterns) -> array
// Returns array of patterns that matched
defn!(MatchWhichFn, vec![arg!(string), arg!(array)], None);

impl Function for MatchWhichFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr.iter().filter_map(|p| p.as_str()).collect();

        if patterns.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();

        let mut found = vec![false; patterns.len()];

        for mat in ac.find_iter(text) {
            found[mat.pattern().as_usize()] = true;
        }

        let matched: Vec<Value> = patterns
            .iter()
            .enumerate()
            .filter(|(i, _)| found[*i])
            .map(|(_, p)| Value::String((*p).to_string()))
            .collect();

        Ok(Value::Array(matched))
    }
}

// match_count(string, patterns) -> number
// Count total number of matches (non-overlapping) across all patterns
defn!(MatchCountFn, vec![arg!(string), arg!(array)], None);

impl Function for MatchCountFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr.iter().filter_map(|p| p.as_str()).collect();

        if patterns.is_empty() {
            return Ok(Value::Number(Number::from(0)));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let count = ac.find_iter(text).count();

        Ok(Value::Number(Number::from(count)))
    }
}

// replace_many(string, replacements) -> string
// Replace multiple patterns at once. replacements is an object {pattern: replacement, ...}
defn!(ReplaceManyFn, vec![arg!(string), arg!(object)], None);

impl Function for ReplaceManyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let replacements_obj = args[1].as_object().unwrap();

        if replacements_obj.is_empty() {
            return Ok(Value::String(text.to_string()));
        }

        let mut patterns: Vec<&str> = Vec::new();
        let mut replacements: Vec<String> = Vec::new();

        for (pattern, replacement) in replacements_obj.iter() {
            patterns.push(pattern);
            if let Some(s) = replacement.as_str() {
                replacements.push(s.to_string());
            } else {
                replacements.push(replacement.to_string());
            }
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let result = ac.replace_all(text, &replacements);

        Ok(Value::String(result))
    }
}

// extract_all(string, patterns) -> array of matches with pattern info
defn!(ExtractAllFn, vec![arg!(string), arg!(array)], None);

impl Function for ExtractAllFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr.iter().filter_map(|p| p.as_str()).collect();

        if patterns.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let mut results: Vec<Value> = Vec::new();

        for mat in ac.find_iter(text) {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "pattern".to_string(),
                Value::String(patterns[mat.pattern().as_usize()].to_string()),
            );
            obj.insert(
                "match".to_string(),
                Value::String(text[mat.start()..mat.end()].to_string()),
            );
            obj.insert(
                "start".to_string(),
                Value::Number(Number::from(mat.start())),
            );
            obj.insert("end".to_string(), Value::Number(Number::from(mat.end())));
            results.push(Value::Object(obj));
        }

        Ok(Value::Array(results))
    }
}

// match_positions(string, patterns) -> array of positions
defn!(MatchPositionsFn, vec![arg!(string), arg!(array)], None);

impl Function for MatchPositionsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr.iter().filter_map(|p| p.as_str()).collect();

        if patterns.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let mut results: Vec<Value> = Vec::new();

        for mat in ac.find_iter(text) {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "pattern".to_string(),
                Value::String(patterns[mat.pattern().as_usize()].to_string()),
            );
            obj.insert(
                "start".to_string(),
                Value::Number(Number::from(mat.start())),
            );
            obj.insert("end".to_string(), Value::Number(Number::from(mat.end())));
            results.push(Value::Object(obj));
        }

        Ok(Value::Array(results))
    }
}

// mm_tokenize(string, options?) -> array of tokens
// Smart word tokenization with optional configuration
defn!(MmTokenizeFn, vec![arg!(string)], Some(arg!(any)));

impl Function for MmTokenizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();

        let lowercase = args
            .get(1)
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("lowercase"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let min_length = args
            .get(1)
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("min_length"))
            .and_then(|v| v.as_f64())
            .map(|n| n as usize)
            .unwrap_or(1);

        let tokens: Vec<Value> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= min_length)
            .map(|s| {
                let token = if lowercase {
                    s.to_lowercase()
                } else {
                    s.to_string()
                };
                Value::String(token)
            })
            .collect();

        Ok(Value::Array(tokens))
    }
}

// extract_between(string, start, end) -> string or null
defn!(
    ExtractBetweenFn,
    vec![arg!(string), arg!(string), arg!(string)],
    None
);

impl Function for ExtractBetweenFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let start_delim = args[1].as_str().unwrap();
        let end_delim = args[2].as_str().unwrap();

        if let Some(start_pos) = text.find(start_delim) {
            let after_start = start_pos + start_delim.len();
            if let Some(end_pos) = text[after_start..].find(end_delim) {
                let extracted = &text[after_start..after_start + end_pos];
                return Ok(Value::String(extracted.to_string()));
            }
        }

        Ok(Value::Null)
    }
}

// split_keep(string, delimiter) -> array keeping delimiters
defn!(SplitKeepFn, vec![arg!(string), arg!(string)], None);

impl Function for SplitKeepFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().unwrap();
        let delimiter = args[1].as_str().unwrap();

        if delimiter.is_empty() {
            return Ok(Value::Array(vec![Value::String(text.to_string())]));
        }

        let mut result: Vec<Value> = Vec::new();
        let mut last_end = 0;

        for (start, part) in text.match_indices(delimiter) {
            if start > last_end {
                result.push(Value::String(text[last_end..start].to_string()));
            }
            result.push(Value::String(part.to_string()));
            last_end = start + part.len();
        }

        if last_end < text.len() {
            result.push(Value::String(text[last_end..].to_string()));
        }

        Ok(Value::Array(result))
    }
}

/// Register multi-match functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "match_any", enabled, Box::new(MatchAnyFn::new()));
    register_if_enabled(runtime, "match_all", enabled, Box::new(MatchAllFn::new()));
    register_if_enabled(
        runtime,
        "match_which",
        enabled,
        Box::new(MatchWhichFn::new()),
    );
    register_if_enabled(
        runtime,
        "match_count",
        enabled,
        Box::new(MatchCountFn::new()),
    );
    register_if_enabled(
        runtime,
        "replace_many",
        enabled,
        Box::new(ReplaceManyFn::new()),
    );
    register_if_enabled(
        runtime,
        "extract_all",
        enabled,
        Box::new(ExtractAllFn::new()),
    );
    register_if_enabled(
        runtime,
        "match_positions",
        enabled,
        Box::new(MatchPositionsFn::new()),
    );
    register_if_enabled(
        runtime,
        "mm_tokenize",
        enabled,
        Box::new(MmTokenizeFn::new()),
    );
    register_if_enabled(
        runtime,
        "extract_between",
        enabled,
        Box::new(ExtractBetweenFn::new()),
    );
    register_if_enabled(runtime, "split_keep", enabled, Box::new(SplitKeepFn::new()));
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

    // match_any tests

    #[test]
    fn test_match_any_found() {
        let runtime = setup_runtime();
        let data = json!("an error occurred in the system");
        let expr = runtime
            .compile("match_any(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_match_any_not_found() {
        let runtime = setup_runtime();
        let data = json!("everything is fine");
        let expr = runtime
            .compile("match_any(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_match_any_empty_patterns() {
        let runtime = setup_runtime();
        let data = json!({"text": "some text", "patterns": []});
        let expr = runtime.compile("match_any(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_match_any_multiple_matches() {
        let runtime = setup_runtime();
        let data = json!("error and warning detected");
        let expr = runtime
            .compile("match_any(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    // match_all tests

    #[test]
    fn test_match_all_all_found() {
        let runtime = setup_runtime();
        let data = json!("error and warning detected");
        let expr = runtime
            .compile("match_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_match_all_some_missing() {
        let runtime = setup_runtime();
        let data = json!("error detected");
        let expr = runtime
            .compile("match_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_match_all_empty_patterns() {
        let runtime = setup_runtime();
        let data = json!({"text": "some text", "patterns": []});
        let expr = runtime.compile("match_all(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    // match_which tests

    #[test]
    fn test_match_which_some_found() {
        let runtime = setup_runtime();
        let data = json!("error and warning detected");
        let expr = runtime
            .compile("match_which(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let strs: Vec<&str> = arr.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(strs.contains(&"error"));
        assert!(strs.contains(&"warning"));
    }

    #[test]
    fn test_match_which_none_found() {
        let runtime = setup_runtime();
        let data = json!("everything is fine");
        let expr = runtime
            .compile("match_which(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_match_which_preserves_order() {
        let runtime = setup_runtime();
        let data = json!("warning then error");
        let expr = runtime
            .compile("match_which(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should return in pattern order, not match order
        assert_eq!(arr[0].as_str().unwrap(), "error");
        assert_eq!(arr[1].as_str().unwrap(), "warning");
    }

    // match_count tests

    #[test]
    fn test_match_count_multiple() {
        let runtime = setup_runtime();
        let data = json!("error error warning error");
        let expr = runtime
            .compile("match_count(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_match_count_none() {
        let runtime = setup_runtime();
        let data = json!("everything is fine");
        let expr = runtime
            .compile("match_count(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_match_count_empty_patterns() {
        let runtime = setup_runtime();
        let data = json!({"text": "some text", "patterns": []});
        let expr = runtime.compile("match_count(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    // replace_many tests

    #[test]
    fn test_replace_many_basic() {
        let runtime = setup_runtime();
        let data = json!({"text": "hello world"});
        let expr = runtime
            .compile("replace_many(text, {hello: 'hi', world: 'there'})")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "hi there");
    }

    #[test]
    fn test_replace_many_no_matches() {
        let runtime = setup_runtime();
        let data = json!({"text": "hello world"});
        let expr = runtime.compile("replace_many(text, {foo: 'bar'})").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_replace_many_empty_replacements() {
        let runtime = setup_runtime();
        let data = json!({"text": "hello world", "replacements": {}});
        let expr = runtime.compile("replace_many(text, replacements)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_replace_many_multiple_occurrences() {
        let runtime = setup_runtime();
        let data = json!({"text": "error: connection error"});
        let expr = runtime
            .compile("replace_many(text, {error: 'ERROR', connection: 'CONN'})")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "ERROR: CONN ERROR");
    }

    // extract_all tests

    #[test]
    fn test_extract_all_basic() {
        let runtime = setup_runtime();
        let data = json!("error and warning detected");
        let expr = runtime
            .compile("extract_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_object().unwrap();
        assert_eq!(first.get("match").unwrap().as_str().unwrap(), "error");
        assert!(first.get("start").is_some());
        assert!(first.get("end").is_some());
    }

    #[test]
    fn test_extract_all_empty() {
        let runtime = setup_runtime();
        let data = json!("no matches here");
        let expr = runtime
            .compile("extract_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // match_positions tests

    #[test]
    fn test_match_positions_basic() {
        let runtime = setup_runtime();
        let data = json!("The quick brown fox");
        let expr = runtime
            .compile("match_positions(@, ['quick', 'fox'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_object().unwrap();
        assert_eq!(first.get("pattern").unwrap().as_str().unwrap(), "quick");
        assert_eq!(first.get("start").unwrap().as_f64().unwrap() as i64, 4);
        assert_eq!(first.get("end").unwrap().as_f64().unwrap() as i64, 9);
    }

    #[test]
    fn test_mm_tokenize_basic() {
        let runtime = setup_runtime();
        let data = json!("Hello, world! This is a test.");
        let expr = runtime.compile("mm_tokenize(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.len() >= 6);
        assert_eq!(arr[0].as_str().unwrap(), "Hello");
    }

    #[test]
    fn test_mm_tokenize_with_options() {
        let runtime = setup_runtime();
        let data = json!("Hello, world! A test.");
        let expr = runtime
            .compile("mm_tokenize(@, {lowercase: `true`, min_length: `2`})")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should have: hello, world, test (not "A" due to min_length)
        let tokens: Vec<&str> = arr.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(tokens.contains(&"hello"));
        assert!(tokens.contains(&"world"));
        assert!(!tokens.iter().any(|t| t.len() < 2));
    }

    // extract_between tests

    #[test]
    fn test_extract_between_basic() {
        let runtime = setup_runtime();
        let data = json!("<title>Page Title</title>");
        let expr = runtime
            .compile("extract_between(@, '<title>', '</title>')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "Page Title");
    }

    #[test]
    fn test_extract_between_not_found() {
        let runtime = setup_runtime();
        let data = json!("no delimiters here");
        let expr = runtime
            .compile("extract_between(@, '<start>', '<end>')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    // split_keep tests

    #[test]
    fn test_split_keep_basic() {
        let runtime = setup_runtime();
        let data = json!("a-b-c");
        let expr = runtime.compile("split_keep(@, '-')").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_str().unwrap(), "a");
        assert_eq!(arr[1].as_str().unwrap(), "-");
        assert_eq!(arr[2].as_str().unwrap(), "b");
    }

    #[test]
    fn test_split_keep_no_delimiter() {
        let runtime = setup_runtime();
        let data = json!("no delimiters");
        let expr = runtime.compile("split_keep(@, '-')").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_str().unwrap(), "no delimiters");
    }
}
