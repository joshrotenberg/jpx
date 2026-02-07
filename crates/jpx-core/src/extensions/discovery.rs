//! Discovery and search functions for JSON arrays.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Match types for scoring, in order of relevance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
    None,
}

impl MatchType {
    fn as_str(&self) -> &'static str {
        match self {
            MatchType::Exact => "exact",
            MatchType::Prefix => "prefix",
            MatchType::Contains => "contains",
            MatchType::Fuzzy => "fuzzy",
            MatchType::None => "none",
        }
    }

    fn base_score(&self) -> i32 {
        match self {
            MatchType::Exact => 1000,
            MatchType::Prefix => 800,
            MatchType::Contains => 600,
            MatchType::Fuzzy => 400,
            MatchType::None => 0,
        }
    }
}

/// Calculate match score for a single field value against query
fn score_field(value: &str, query: &str, field_weight: i32) -> (i32, MatchType) {
    let value_lower = value.to_lowercase();
    let query_lower = query.to_lowercase();

    let match_type = if value_lower == query_lower {
        MatchType::Exact
    } else if value_lower.starts_with(&query_lower) {
        MatchType::Prefix
    } else if value_lower.contains(&query_lower) {
        MatchType::Contains
    } else {
        // Try fuzzy matching for longer strings
        if query.len() >= 3 && value.len() >= 3 {
            let similarity = strsim::jaro_winkler(&value_lower, &query_lower);
            if similarity > 0.8 {
                MatchType::Fuzzy
            } else {
                MatchType::None
            }
        } else {
            MatchType::None
        }
    };

    let score = match_type.base_score() * field_weight / 10;
    (score, match_type)
}

/// Score an item against a query across multiple fields
fn score_item(
    item: &Value,
    query: &str,
    fields: &[(String, i32)],
) -> Option<(i32, String, String)> {
    let obj = item.as_object()?;

    let mut best_score = 0;
    let mut best_match_type = MatchType::None;
    let mut best_field = String::new();

    for (field, weight) in fields {
        if let Some(val) = obj.get(field.as_str()) {
            let text = match val {
                Value::String(s) => s.clone(),
                Value::Array(arr) => {
                    // For arrays (like tags), join and search
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
                _ => continue,
            };

            let (score, match_type) = score_field(&text, query, *weight);
            if score > best_score {
                best_score = score;
                best_match_type = match_type;
                best_field = field.clone();
            }
        }
    }

    if best_score > 0 {
        Some((best_score, best_match_type.as_str().to_string(), best_field))
    } else {
        None
    }
}

/// Parse field specification - either a string "name,description" or object {"name": 10, "description": 5}
fn parse_fields(fields_arg: &Value) -> Result<Vec<(String, i32)>, String> {
    match fields_arg {
        Value::String(s) => {
            // Simple comma-separated list with default weight of 10
            Ok(s.split(',').map(|f| (f.trim().to_string(), 10)).collect())
        }
        Value::Object(obj) => {
            // Object with field weights
            let mut fields = Vec::new();
            for (k, v) in obj.iter() {
                let weight = v.as_f64().map(|n| n as i32).unwrap_or(10);
                fields.push((k.clone(), weight));
            }
            Ok(fields)
        }
        _ => Err("fields must be a string or object".to_string()),
    }
}

// fuzzy_search(array, fields, query) -> array
// Search an array of objects, returning matches sorted by relevance
defn!(
    FuzzySearchFn,
    vec![arg!(array), arg!(any), arg!(string)],
    None
);

impl Function for FuzzySearchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let array = args[0].as_array().unwrap();
        let fields = parse_fields(&args[1]).map_err(|e| crate::functions::custom_error(ctx, &e))?;
        let query = args[2].as_str().unwrap();

        if query.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut results: Vec<(i32, Value)> = Vec::new();

        for item in array.iter() {
            if let Some((score, match_type, matched_field)) = score_item(item, query, &fields) {
                let mut result_obj = serde_json::Map::new();
                result_obj.insert("item".to_string(), item.clone());
                result_obj.insert("score".to_string(), Value::Number(Number::from(score)));
                result_obj.insert("match_type".to_string(), Value::String(match_type));
                result_obj.insert("matched_field".to_string(), Value::String(matched_field));

                results.push((score, Value::Object(result_obj)));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.0.cmp(&a.0));

        let result_array: Vec<Value> = results.into_iter().map(|(_, item)| item).collect();
        Ok(Value::Array(result_array))
    }
}

// fuzzy_match(value, query) -> object
// Check if a single value matches a query, returning match details
defn!(FuzzyMatchFn, vec![arg!(string), arg!(string)], None);

impl Function for FuzzyMatchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = args[0].as_str().unwrap();
        let query = args[1].as_str().unwrap();

        let (score, match_type) = score_field(value, query, 10);

        let mut result = serde_json::Map::new();
        result.insert("matches".to_string(), Value::Bool(score > 0));
        result.insert("score".to_string(), Value::Number(Number::from(score)));
        result.insert(
            "match_type".to_string(),
            Value::String(match_type.as_str().to_string()),
        );

        // Add similarity score for fuzzy matches
        if match_type == MatchType::Fuzzy || match_type == MatchType::None {
            let similarity = strsim::jaro_winkler(&value.to_lowercase(), &query.to_lowercase());
            result.insert("similarity".to_string(), number_value(similarity));
        }

        Ok(Value::Object(result))
    }
}

// fuzzy_score(value, query) -> number
// Simple scoring function that returns just the match score
defn!(FuzzyScoreFn, vec![arg!(string), arg!(string)], None);

impl Function for FuzzyScoreFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = args[0].as_str().unwrap();
        let query = args[1].as_str().unwrap();

        let (score, _) = score_field(value, query, 10);

        Ok(Value::Number(Number::from(score)))
    }
}

/// Register discovery functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "fuzzy_search",
        enabled,
        Box::new(FuzzySearchFn::new()),
    );
    register_if_enabled(
        runtime,
        "fuzzy_match",
        enabled,
        Box::new(FuzzyMatchFn::new()),
    );
    register_if_enabled(
        runtime,
        "fuzzy_score",
        enabled,
        Box::new(FuzzyScoreFn::new()),
    );
}
