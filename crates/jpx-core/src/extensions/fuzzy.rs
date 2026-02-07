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
