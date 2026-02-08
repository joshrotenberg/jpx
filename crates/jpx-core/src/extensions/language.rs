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
    fn test_detect_language_english() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language('This is a test of the language detection system.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "English");
    }

    #[test]
    fn test_detect_language_spanish() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language('Esto es una prueba del sistema de deteccion de idiomas.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        // whatlang returns native language names
        assert_eq!(result.as_str().unwrap(), "Espa\u{f1}ol");
    }

    #[test]
    fn test_detect_language_french() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language('Ceci est un test du systeme de detection de langue.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        // whatlang returns native language names
        assert_eq!(result.as_str().unwrap(), "Fran\u{e7}ais");
    }

    #[test]
    fn test_detect_language_german() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language('Dies ist ein Test des Spracherkennungssystems.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        // whatlang returns native language names
        assert_eq!(result.as_str().unwrap(), "Deutsch");
    }

    #[test]
    fn test_detect_language_iso_english() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language_iso('This is English text.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "eng");
    }

    #[test]
    fn test_detect_language_iso_spanish() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language_iso('Este es un texto en espanol.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "spa");
    }

    #[test]
    fn test_detect_script_latin() {
        let runtime = setup_runtime();
        let expr = runtime.compile("detect_script('Hello world')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "Latin");
    }

    #[test]
    fn test_detect_script_cyrillic() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile(
                "detect_script('\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442} \u{43c}\u{438}\u{440}')",
            )
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "Cyrillic");
    }

    #[test]
    fn test_detect_script_arabic() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_script('\u{645}\u{631}\u{62d}\u{628}\u{627} \u{628}\u{627}\u{644}\u{639}\u{627}\u{644}\u{645}')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "Arabic");
    }

    #[test]
    fn test_detect_language_confidence() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language_confidence('This is definitely English text for testing.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let confidence = result.as_f64().unwrap();
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_detect_language_info() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("detect_language_info('This is a test.')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let obj = result.as_object().unwrap();

        assert!(obj.contains_key("language"));
        assert!(obj.contains_key("code"));
        assert!(obj.contains_key("script"));
        assert!(obj.contains_key("confidence"));
        assert!(obj.contains_key("reliable"));

        assert_eq!(obj.get("language").unwrap().as_str().unwrap(), "English");
        assert_eq!(obj.get("code").unwrap().as_str().unwrap(), "eng");
        assert_eq!(obj.get("script").unwrap().as_str().unwrap(), "Latin");
    }

    #[test]
    fn test_detect_language_empty_string() {
        let runtime = setup_runtime();
        let expr = runtime.compile("detect_language('')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        // Empty string may return null or a guess
        // Just verify it doesn't crash
        assert!(result.is_null() || result.as_str().is_some());
    }

    #[test]
    fn test_detect_language_short_text() {
        let runtime = setup_runtime();
        let expr = runtime.compile("detect_language('Hi')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        // Short text may have low confidence but should still work
        assert!(result.is_null() || result.as_str().is_some());
    }
}
