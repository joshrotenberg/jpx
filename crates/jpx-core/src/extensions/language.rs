//! Language detection functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// detect_language(text) -> string (full name like "English")
// =============================================================================

defn!(DetectLanguageFn, vec![arg!(string)], None);

impl Function for DetectLanguageFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_str().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let name = info.lang().to_string();
                Ok(Value::String(name))
            }
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// detect_language_iso(text) -> string (ISO 639-3 code like "eng")
// =============================================================================

defn!(DetectLanguageIsoFn, vec![arg!(string)], None);

impl Function for DetectLanguageIsoFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_str().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let code = info.lang().code();
                Ok(Value::String(code.to_string()))
            }
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// detect_script(text) -> string (script name like "Latin")
// =============================================================================

defn!(DetectScriptFn, vec![arg!(string)], None);

impl Function for DetectScriptFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_str().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let script = format!("{:?}", info.script());
                Ok(Value::String(script))
            }
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// detect_language_confidence(text) -> number (0.0-1.0)
// =============================================================================

defn!(DetectLanguageConfidenceFn, vec![arg!(string)], None);

impl Function for DetectLanguageConfidenceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_str().unwrap();

        match whatlang::detect(text) {
            Some(info) => Ok(number_value(info.confidence())),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// detect_language_info(text) -> object with full detection info
// =============================================================================

defn!(DetectLanguageInfoFn, vec![arg!(string)], None);

impl Function for DetectLanguageInfoFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_str().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let mut result = serde_json::Map::new();

                result.insert(
                    "language".to_string(),
                    Value::String(info.lang().to_string()),
                );
                result.insert(
                    "code".to_string(),
                    Value::String(info.lang().code().to_string()),
                );
                result.insert(
                    "script".to_string(),
                    Value::String(format!("{:?}", info.script())),
                );
                result.insert(
                    "confidence".to_string(),
                    Number::from_f64(info.confidence()).map_or(Value::Null, Value::Number),
                );
                result.insert("reliable".to_string(), Value::Bool(info.is_reliable()));

                Ok(Value::Object(result))
            }
            None => Ok(Value::Null),
        }
    }
}

/// Register language detection functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "detect_language",
        enabled,
        Box::new(DetectLanguageFn::new()),
    );
    register_if_enabled(
        runtime,
        "detect_language_iso",
        enabled,
        Box::new(DetectLanguageIsoFn::new()),
    );
    register_if_enabled(
        runtime,
        "detect_script",
        enabled,
        Box::new(DetectScriptFn::new()),
    );
    register_if_enabled(
        runtime,
        "detect_language_confidence",
        enabled,
        Box::new(DetectLanguageConfidenceFn::new()),
    );
    register_if_enabled(
        runtime,
        "detect_language_info",
        enabled,
        Box::new(DetectLanguageInfoFn::new()),
    );
}
