//! Fuzzy string matching functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// levenshtein(s1, s2) -> number
defn!(LevenshteinFn, vec![arg!(string), arg!(string)], None);

impl Function for LevenshteinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let dist = strsim::levenshtein(s1, s2);
        Ok(number_value(dist as f64))
    }
}

// normalized_levenshtein(s1, s2) -> number (0.0-1.0)
defn!(
    NormalizedLevenshteinFn,
    vec![arg!(string), arg!(string)],
    None
);

impl Function for NormalizedLevenshteinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let sim = strsim::normalized_levenshtein(s1, s2);
        Ok(number_value(sim))
    }
}

// damerau_levenshtein(s1, s2) -> number
defn!(DamerauLevenshteinFn, vec![arg!(string), arg!(string)], None);

impl Function for DamerauLevenshteinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let dist = strsim::damerau_levenshtein(s1, s2);
        Ok(number_value(dist as f64))
    }
}

// normalized_damerau_levenshtein(s1, s2) -> number (0.0-1.0)
defn!(
    NormalizedDamerauLevenshteinFn,
    vec![arg!(string), arg!(string)],
    None
);

impl Function for NormalizedDamerauLevenshteinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let sim = strsim::normalized_damerau_levenshtein(s1, s2);
        Ok(number_value(sim))
    }
}

// jaro(s1, s2) -> number (0.0-1.0)
defn!(JaroFn, vec![arg!(string), arg!(string)], None);

impl Function for JaroFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let sim = strsim::jaro(s1, s2);
        Ok(number_value(sim))
    }
}

// jaro_winkler(s1, s2) -> number (0.0-1.0)
defn!(JaroWinklerFn, vec![arg!(string), arg!(string)], None);

impl Function for JaroWinklerFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let sim = strsim::jaro_winkler(s1, s2);
        Ok(number_value(sim))
    }
}

// sorensen_dice(s1, s2) -> number (0.0-1.0)
defn!(SorensenDiceFn, vec![arg!(string), arg!(string)], None);

impl Function for SorensenDiceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let sim = strsim::sorensen_dice(s1, s2);
        Ok(number_value(sim))
    }
}

// hamming(s1, s2) -> number (returns null if strings have different lengths)
defn!(HammingFn, vec![arg!(string), arg!(string)], None);

impl Function for HammingFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        match strsim::hamming(s1, s2) {
            Ok(dist) => Ok(number_value(dist as f64)),
            Err(_) => Ok(Value::Null), // Different lengths
        }
    }
}

// osa_distance(s1, s2) -> number (Optimal String Alignment distance)
defn!(OsaDistanceFn, vec![arg!(string), arg!(string)], None);

impl Function for OsaDistanceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();
        let dist = strsim::osa_distance(s1, s2);
        Ok(number_value(dist as f64))
    }
}

/// Register fuzzy matching functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "levenshtein",
        enabled,
        Box::new(LevenshteinFn::new()),
    );
    register_if_enabled(
        runtime,
        "normalized_levenshtein",
        enabled,
        Box::new(NormalizedLevenshteinFn::new()),
    );
    register_if_enabled(
        runtime,
        "damerau_levenshtein",
        enabled,
        Box::new(DamerauLevenshteinFn::new()),
    );
    register_if_enabled(
        runtime,
        "normalized_damerau_levenshtein",
        enabled,
        Box::new(NormalizedDamerauLevenshteinFn::new()),
    );
    register_if_enabled(runtime, "jaro", enabled, Box::new(JaroFn::new()));
    register_if_enabled(
        runtime,
        "jaro_winkler",
        enabled,
        Box::new(JaroWinklerFn::new()),
    );
    register_if_enabled(
        runtime,
        "sorensen_dice",
        enabled,
        Box::new(SorensenDiceFn::new()),
    );
    register_if_enabled(runtime, "hamming", enabled, Box::new(HammingFn::new()));
    register_if_enabled(
        runtime,
        "osa_distance",
        enabled,
        Box::new(OsaDistanceFn::new()),
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
    fn test_levenshtein() {
        let runtime = setup_runtime();
        let expr = runtime.compile("levenshtein('kitten', 'sitting')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_levenshtein_identical() {
        let runtime = setup_runtime();
        let expr = runtime.compile("levenshtein('hello', 'hello')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_normalized_levenshtein() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("normalized_levenshtein('hello', 'hello')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_normalized_levenshtein_different() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("normalized_levenshtein('hello', 'world')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let sim = result.as_f64().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_damerau_levenshtein() {
        let runtime = setup_runtime();
        // Transposition: "ab" -> "ba" is 1 edit in Damerau-Levenshtein
        let expr = runtime.compile("damerau_levenshtein('ab', 'ba')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_jaro() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jaro('hello', 'hallo')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let sim = result.as_f64().unwrap();
        assert!(sim > 0.8);
    }

    #[test]
    fn test_jaro_identical() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jaro('test', 'test')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_jaro_winkler() {
        let runtime = setup_runtime();
        // Jaro-Winkler boosts common prefixes
        let expr = runtime
            .compile("jaro_winkler('prefix_abc', 'prefix_xyz')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let sim = result.as_f64().unwrap();
        assert!(sim > 0.7);
    }

    #[test]
    fn test_jaro_winkler_vs_jaro() {
        let runtime = setup_runtime();
        // Jaro-Winkler should be >= Jaro for strings with common prefix
        let jw_expr = runtime.compile("jaro_winkler('hello', 'hella')").unwrap();
        let j_expr = runtime.compile("jaro('hello', 'hella')").unwrap();
        let jw = jw_expr.search(&json!(null)).unwrap();
        let j = j_expr.search(&json!(null)).unwrap();
        assert!(jw.as_f64().unwrap() >= j.as_f64().unwrap());
    }

    #[test]
    fn test_sorensen_dice() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sorensen_dice('night', 'nacht')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let sim = result.as_f64().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_sorensen_dice_identical() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sorensen_dice('test', 'test')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_normalized_damerau_levenshtein() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("normalized_damerau_levenshtein('hello', 'hello')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_normalized_damerau_levenshtein_transposition() {
        let runtime = setup_runtime();
        // "ab" vs "ba" - transposition
        let expr = runtime
            .compile("normalized_damerau_levenshtein('ab', 'ba')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let sim = result.as_f64().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_hamming() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hamming('karolin', 'kathrin')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_hamming_identical() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hamming('hello', 'hello')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_hamming_different_lengths() {
        let runtime = setup_runtime();
        // Different lengths should return null
        let expr = runtime.compile("hamming('hello', 'hi')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_osa_distance() {
        let runtime = setup_runtime();
        // OSA allows transpositions
        let expr = runtime.compile("osa_distance('ab', 'ba')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_osa_distance_identical() {
        let runtime = setup_runtime();
        let expr = runtime.compile("osa_distance('hello', 'hello')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }
}
