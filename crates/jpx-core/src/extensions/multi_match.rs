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
