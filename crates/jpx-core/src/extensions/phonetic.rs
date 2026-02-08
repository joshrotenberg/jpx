//! Phonetic encoding functions.

use std::collections::HashSet;

use rphonetic::{
    Caverphone1, Caverphone2, Encoder, MatchRatingApproach, Metaphone, Nysiis, Soundex,
};
use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// soundex(string) -> string
// =============================================================================

defn!(SoundexFn, vec![arg!(string)], None);

impl Function for SoundexFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let soundex = Soundex::default();
        let result = soundex.encode(s);
        Ok(Value::String(result))
    }
}

// =============================================================================
// metaphone(string) -> string
// =============================================================================

defn!(MetaphoneFn, vec![arg!(string)], None);

impl Function for MetaphoneFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let metaphone = Metaphone::default();
        let result = metaphone.encode(s);
        Ok(Value::String(result))
    }
}

// =============================================================================
// double_metaphone(string) -> [primary, alternate]
// =============================================================================

defn!(DoubleMetaphoneFn, vec![arg!(string)], None);

impl Function for DoubleMetaphoneFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let dm = rphonetic::DoubleMetaphone::default();
        let result = dm.double_metaphone(s);
        let primary = Value::String(result.primary());
        let alt = result.alternate();
        let alternate = if alt.is_empty() {
            Value::Null
        } else {
            Value::String(alt)
        };
        Ok(Value::Array(vec![primary, alternate]))
    }
}

// =============================================================================
// nysiis(string) -> string
// =============================================================================

defn!(NysiisFn, vec![arg!(string)], None);

impl Function for NysiisFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let nysiis = Nysiis::default();
        let result = nysiis.encode(s);
        Ok(Value::String(result))
    }
}

// =============================================================================
// match_rating_codex(string) -> string
// =============================================================================

defn!(MatchRatingCodexFn, vec![arg!(string)], None);

impl Function for MatchRatingCodexFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let mra = MatchRatingApproach;
        let result = mra.encode(s);
        Ok(Value::String(result))
    }
}

// =============================================================================
// caverphone(string) -> string
// =============================================================================

defn!(CaverphoneFn, vec![arg!(string)], None);

impl Function for CaverphoneFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let caverphone = Caverphone1;
        let result = caverphone.encode(s);
        Ok(Value::String(result))
    }
}

// =============================================================================
// caverphone2(string) -> string
// =============================================================================

defn!(Caverphone2Fn, vec![arg!(string)], None);

impl Function for Caverphone2Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let caverphone = Caverphone2;
        let result = caverphone.encode(s);
        Ok(Value::String(result))
    }
}

// =============================================================================
// sounds_like(s1, s2) -> bool
// =============================================================================

defn!(SoundsLikeFn, vec![arg!(string), arg!(string)], None);

impl Function for SoundsLikeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let soundex = Soundex::default();
        let result = soundex.is_encoded_equals(s1, s2);
        Ok(Value::Bool(result))
    }
}

// =============================================================================
// phonetic_match(s1, s2, algorithm?) -> bool
// =============================================================================

defn!(
    PhoneticMatchFn,
    vec![arg!(string), arg!(string)],
    Some(arg!(string))
);

impl Function for PhoneticMatchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();

        let algorithm = if args.len() > 2 {
            args[2]
                .as_str()
                .map(|s| s.to_lowercase())
                .unwrap_or_else(|| "soundex".to_string())
        } else {
            "soundex".to_string()
        };

        let result = match algorithm.as_str() {
            "soundex" => {
                let encoder = Soundex::default();
                encoder.is_encoded_equals(s1, s2)
            }
            "metaphone" => {
                let encoder = Metaphone::default();
                encoder.encode(s1) == encoder.encode(s2)
            }
            "double_metaphone" | "doublemetaphone" => {
                let encoder = rphonetic::DoubleMetaphone::default();
                let r1 = encoder.double_metaphone(s1);
                let r2 = encoder.double_metaphone(s2);
                // Match if primary codes match, or if any combination matches
                r1.primary() == r2.primary()
                    || (!r1.alternate().is_empty() && r1.alternate() == r2.primary())
                    || (!r2.alternate().is_empty() && r2.alternate() == r1.primary())
                    || (!r1.alternate().is_empty()
                        && !r2.alternate().is_empty()
                        && r1.alternate() == r2.alternate())
            }
            "nysiis" => {
                let encoder = Nysiis::default();
                encoder.encode(s1) == encoder.encode(s2)
            }
            "match_rating" | "mra" => {
                let encoder = MatchRatingApproach;
                encoder.is_encoded_equals(s1, s2)
            }
            "caverphone" | "caverphone1" => {
                let encoder = Caverphone1;
                encoder.encode(s1) == encoder.encode(s2)
            }
            "caverphone2" => {
                let encoder = Caverphone2;
                encoder.encode(s1) == encoder.encode(s2)
            }
            _ => {
                // Default to soundex for unknown algorithms
                let encoder = Soundex::default();
                encoder.is_encoded_equals(s1, s2)
            }
        };

        Ok(Value::Bool(result))
    }
}

/// Register phonetic functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "soundex", enabled, Box::new(SoundexFn::new()));
    register_if_enabled(runtime, "metaphone", enabled, Box::new(MetaphoneFn::new()));
    register_if_enabled(
        runtime,
        "double_metaphone",
        enabled,
        Box::new(DoubleMetaphoneFn::new()),
    );
    register_if_enabled(runtime, "nysiis", enabled, Box::new(NysiisFn::new()));
    register_if_enabled(
        runtime,
        "match_rating_codex",
        enabled,
        Box::new(MatchRatingCodexFn::new()),
    );
    register_if_enabled(
        runtime,
        "caverphone",
        enabled,
        Box::new(CaverphoneFn::new()),
    );
    register_if_enabled(
        runtime,
        "caverphone2",
        enabled,
        Box::new(Caverphone2Fn::new()),
    );
    register_if_enabled(
        runtime,
        "sounds_like",
        enabled,
        Box::new(SoundsLikeFn::new()),
    );
    register_if_enabled(
        runtime,
        "phonetic_match",
        enabled,
        Box::new(PhoneticMatchFn::new()),
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
    fn test_soundex() {
        let runtime = setup_runtime();
        let data = json!("Robert");
        let expr = runtime.compile("soundex(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "R163");
    }

    #[test]
    fn test_soundex_similar_names() {
        let runtime = setup_runtime();
        // Robert and Rupert should have the same Soundex code
        let data = json!("Rupert");
        let expr = runtime.compile("soundex(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "R163");
    }

    #[test]
    fn test_metaphone() {
        let runtime = setup_runtime();
        let data = json!("Smith");
        let expr = runtime.compile("metaphone(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "SM0");
    }

    #[test]
    fn test_double_metaphone() {
        let runtime = setup_runtime();
        let data = json!("Schmidt");
        let expr = runtime.compile("double_metaphone(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Primary encoding
        assert!(!arr[0].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_nysiis() {
        let runtime = setup_runtime();
        let data = json!("Johnson");
        let expr = runtime.compile("nysiis(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_str().unwrap().is_empty());
    }

    #[test]
    fn test_match_rating_codex() {
        let runtime = setup_runtime();
        let data = json!("Smith");
        let expr = runtime.compile("match_rating_codex(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_str().unwrap().is_empty());
    }

    #[test]
    fn test_caverphone() {
        let runtime = setup_runtime();
        let data = json!("Thompson");
        let expr = runtime.compile("caverphone(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_str().unwrap().is_empty());
    }

    #[test]
    fn test_caverphone2() {
        let runtime = setup_runtime();
        let data = json!("Thompson");
        let expr = runtime.compile("caverphone2(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_str().unwrap().is_empty());
    }

    #[test]
    fn test_sounds_like_true() {
        let runtime = setup_runtime();
        let data = json!(["Robert", "Rupert"]);
        let expr = runtime.compile("sounds_like(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_sounds_like_false() {
        let runtime = setup_runtime();
        let data = json!(["Robert", "Smith"]);
        let expr = runtime.compile("sounds_like(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_phonetic_match_default() {
        let runtime = setup_runtime();
        let data = json!(["Robert", "Rupert"]);
        let expr = runtime.compile("phonetic_match(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_phonetic_match_metaphone() {
        let runtime = setup_runtime();
        let data = json!(["Smith", "Smyth"]);
        let expr = runtime
            .compile("phonetic_match(@[0], @[1], 'metaphone')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Both should encode to SM0
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_phonetic_match_nysiis() {
        let runtime = setup_runtime();
        let data = json!(["Johnson", "Jonson"]);
        let expr = runtime
            .compile("phonetic_match(@[0], @[1], 'nysiis')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }
}
