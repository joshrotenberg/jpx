//! Function introspection, search, and similarity.
//!
//! This module provides the function discovery and exploration capabilities
//! of the [`JpxEngine`]: listing categories, searching by keyword with fuzzy
//! matching and synonyms, describing functions, and finding similar functions.

use crate::JpxEngine;
use jpx_core::registry::{Category, FunctionInfo, expand_search_terms, lookup_synonyms};
use serde::{Deserialize, Serialize};
use strsim::jaro_winkler;

/// Detailed information about a JMESPath function.
///
/// This struct provides a serializable representation of function metadata,
/// suitable for API responses, documentation generation, and introspection tools.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let info = engine.describe_function("upper").unwrap();
///
/// println!("Function: {}", info.name);
/// println!("Category: {}", info.category);
/// println!("Signature: {}", info.signature);
/// println!("Example: {}", info.example);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDetail {
    /// Function name (e.g., "upper", "sum", "now")
    pub name: String,
    /// Category name (e.g., "String", "Math", "Datetime")
    pub category: String,
    /// Human-readable description of what the function does
    pub description: String,
    /// Function signature showing parameter types (e.g., "string -> string")
    pub signature: String,
    /// Example usage demonstrating the function
    pub example: String,
    /// Whether this is a standard JMESPath function (vs extension)
    pub is_standard: bool,
    /// JMESPath Enhancement Proposal number, if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jep: Option<String>,
    /// Alternative names for this function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl From<&FunctionInfo> for FunctionDetail {
    fn from(info: &FunctionInfo) -> Self {
        Self {
            name: info.name.to_string(),
            category: format!("{:?}", info.category),
            description: info.description.to_string(),
            signature: info.signature.to_string(),
            example: info.example.to_string(),
            is_standard: info.is_standard,
            jep: info.jep.map(|s| s.to_string()),
            aliases: info.aliases.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Result from searching for functions.
///
/// Contains the matched function along with information about how it matched
/// the search query and its relevance score.
///
/// # Scoring
///
/// Match types and approximate scores:
/// - `exact_name` (1000): Query exactly matches function name
/// - `alias` (900): Query matches a function alias
/// - `name_prefix` (800): Function name starts with query
/// - `name_contains` (700): Function name contains query
/// - `category` (600): Query matches category name
/// - `description` (100-300): Query found in description
/// - `fuzzy_name` (variable): Jaro-Winkler similarity > 0.8
/// - `synonym` (300): Query synonym found in name/description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matched function's details
    pub function: FunctionDetail,
    /// How the function matched (e.g., "exact_name", "description")
    pub match_type: String,
    /// Relevance score (higher = better match)
    pub score: i32,
}

/// Result from finding functions similar to a given function.
///
/// Groups similar functions by relationship type: same category,
/// similar signature, or related concepts in descriptions.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let similar = engine.similar_functions("upper").unwrap();
///
/// // Functions in the same category (String)
/// for f in &similar.same_category {
///     println!("Same category: {}", f.name);
/// }
///
/// // Functions with similar signatures
/// for f in &similar.similar_signature {
///     println!("Similar signature: {}", f.name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFunctionsResult {
    /// Functions in the same category as the target
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub same_category: Vec<FunctionDetail>,
    /// Functions with similar parameter/return types
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub similar_signature: Vec<FunctionDetail>,
    /// Functions with overlapping description keywords
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_concepts: Vec<FunctionDetail>,
}

// =============================================================================
// JpxEngine introspection methods
// =============================================================================

impl JpxEngine {
    /// Lists all available function categories.
    ///
    /// Returns category names like "String", "Math", "Datetime", etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    /// let categories = engine.categories();
    ///
    /// assert!(categories.contains(&"String".to_string()));
    /// assert!(categories.contains(&"Math".to_string()));
    /// assert!(categories.contains(&"Array".to_string()));
    /// ```
    pub fn categories(&self) -> Vec<String> {
        Category::all().iter().map(|c| format!("{:?}", c)).collect()
    }

    /// Lists functions, optionally filtered by category.
    ///
    /// # Arguments
    ///
    /// * `category` - Optional category name to filter by (case-insensitive)
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // All functions
    /// let all = engine.functions(None);
    /// assert!(all.len() > 100);
    ///
    /// // Just string functions
    /// let string_funcs = engine.functions(Some("String"));
    /// assert!(string_funcs.iter().all(|f| f.category == "String"));
    /// ```
    pub fn functions(&self, category: Option<&str>) -> Vec<FunctionDetail> {
        match category {
            Some(name) => match parse_category(name) {
                Some(cat) => self
                    .registry
                    .functions_in_category(cat)
                    .map(FunctionDetail::from)
                    .collect(),
                None => Vec::new(),
            },
            None => self
                .registry
                .functions()
                .map(FunctionDetail::from)
                .collect(),
        }
    }

    /// Gets detailed information about a function by name or alias.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name or alias (e.g., "upper", "md5", "len")
    ///
    /// # Returns
    ///
    /// `Some(FunctionDetail)` if found, `None` if no matching function exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// let info = engine.describe_function("upper").unwrap();
    /// assert_eq!(info.name, "upper");
    /// assert_eq!(info.category, "String");
    /// println!("Signature: {}", info.signature);
    /// println!("Example: {}", info.example);
    ///
    /// // Also works with aliases
    /// let info = engine.describe_function("len");  // alias for "length"
    /// ```
    pub fn describe_function(&self, name: &str) -> Option<FunctionDetail> {
        self.registry.get_function(name).map(FunctionDetail::from)
    }

    /// Describes multiple functions in a single call.
    ///
    /// Returns one `(name, detail)` entry per requested name, preserving input
    /// order, with `None` for names that do not resolve to a function (rather
    /// than failing the whole batch). Names may be aliases, just like
    /// [`describe_function`](Self::describe_function).
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    /// let results = engine.describe_functions(&["upper", "not_a_function"]);
    /// assert_eq!(results.len(), 2);
    /// assert_eq!(results[0].0, "upper");
    /// assert!(results[0].1.is_some());
    /// assert_eq!(results[1].0, "not_a_function");
    /// assert!(results[1].1.is_none());
    /// ```
    pub fn describe_functions(&self, names: &[&str]) -> Vec<(String, Option<FunctionDetail>)> {
        names
            .iter()
            .map(|&name| (name.to_string(), self.describe_function(name)))
            .collect()
    }

    /// Searches for functions matching a query string.
    ///
    /// Uses fuzzy matching, synonyms, and searches across names, descriptions,
    /// categories, and signatures. Results are ranked by relevance.
    ///
    /// # Arguments
    ///
    /// * `query` - Search term (e.g., "hash", "string manipulation", "date")
    /// * `limit` - Maximum number of results to return
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Search by concept
    /// let results = engine.search_functions("hash", 10);
    /// assert!(results.iter().any(|r| r.function.name == "md5"));
    /// assert!(results.iter().any(|r| r.function.name == "sha256"));
    ///
    /// // Results are ranked by relevance
    /// for result in &results {
    ///     println!("{}: {} (score: {})",
    ///         result.function.name,
    ///         result.match_type,
    ///         result.score
    ///     );
    /// }
    /// ```
    pub fn search_functions(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();

        // Expand query terms using synonyms
        let expanded_terms = expand_search_terms(&query_lower);

        let all_functions: Vec<_> = self.registry.functions().collect();
        let mut results: Vec<SearchResult> = Vec::new();

        for info in &all_functions {
            let name_lower = info.name.to_lowercase();
            let desc_lower = info.description.to_lowercase();
            let category_lower = format!("{:?}", info.category).to_lowercase();
            let signature_lower = info.signature.to_lowercase();
            let aliases_lower: Vec<String> = info
                .aliases
                .iter()
                .map(|a: &&str| a.to_lowercase())
                .collect();

            // Calculate match score and type
            let (score, match_type) = calculate_match_score(
                &query_lower,
                &expanded_terms,
                &MatchContext {
                    name: &name_lower,
                    aliases: &aliases_lower,
                    category: &category_lower,
                    description: &desc_lower,
                    signature: &signature_lower,
                },
            );

            if score > 0 {
                results.push(SearchResult {
                    function: FunctionDetail::from(*info),
                    match_type,
                    score,
                });
            }
        }

        // Sort by score descending, then by name
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.function.name.cmp(&b.function.name))
        });

        results.truncate(limit);
        results
    }

    /// Finds functions similar to a given function.
    ///
    /// Returns functions grouped by relationship type:
    /// - Same category (e.g., other string functions if input is "upper")
    /// - Similar signature (same parameter/return types)
    /// - Related concepts (overlapping description keywords)
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the function to find similar functions for
    ///
    /// # Returns
    ///
    /// `Some(SimilarFunctionsResult)` if the function exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// let similar = engine.similar_functions("upper").unwrap();
    ///
    /// // Other string functions
    /// println!("Same category:");
    /// for f in &similar.same_category {
    ///     println!("  - {}", f.name);
    /// }
    /// ```
    pub fn similar_functions(&self, name: &str) -> Option<SimilarFunctionsResult> {
        let info = self.registry.get_function(name)?;
        let all_functions: Vec<_> = self.registry.functions().collect();

        // Same category. Sorted by name before truncating so the result is
        // deterministic (the registry iterates in HashMap order, so an
        // unsorted `take(5)` returned an arbitrary, run-varying subset).
        let mut same_category_fns: Vec<&FunctionInfo> = all_functions
            .iter()
            .copied()
            .filter(|f| f.category == info.category && f.name != info.name)
            .collect();
        same_category_fns.sort_by_key(|f| f.name);
        let same_category: Vec<FunctionDetail> = same_category_fns
            .into_iter()
            .take(5)
            .map(FunctionDetail::from)
            .collect();

        // Similar signature (same arity), likewise name-sorted for determinism.
        let this_arity = count_params(info.signature);
        let mut similar_signature_fns: Vec<&FunctionInfo> = all_functions
            .iter()
            .copied()
            .filter(|f| {
                f.name != info.name
                    && f.category != info.category
                    && count_params(f.signature) == this_arity
            })
            .collect();
        similar_signature_fns.sort_by_key(|f| f.name);
        let similar_signature: Vec<FunctionDetail> = similar_signature_fns
            .into_iter()
            .take(5)
            .map(FunctionDetail::from)
            .collect();

        // Related concepts (description keyword overlap)
        let keywords = extract_keywords(info.description);
        let mut concept_scores: Vec<(&FunctionInfo, usize)> = all_functions
            .iter()
            .filter(|f| f.name != info.name)
            .map(|f| {
                let f_keywords = extract_keywords(f.description);
                let overlap = keywords.iter().filter(|k| f_keywords.contains(*k)).count();
                (*f, overlap)
            })
            .filter(|(_, score)| *score > 0)
            .collect();

        // Highest overlap first, breaking ties by name so the result is stable
        // across runs (the underlying iteration order is not).
        concept_scores.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(b.0.name)));

        let related_concepts: Vec<FunctionDetail> = concept_scores
            .into_iter()
            .take(5)
            .map(|(f, _)| FunctionDetail::from(f))
            .collect();

        Some(SimilarFunctionsResult {
            same_category,
            similar_signature,
            related_concepts,
        })
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Context for calculating match scores
struct MatchContext<'a> {
    name: &'a str,
    aliases: &'a [String],
    category: &'a str,
    description: &'a str,
    signature: &'a str,
}

/// Calculate match score and type for a function
fn calculate_match_score(
    query: &str,
    expanded_terms: &[String],
    ctx: &MatchContext,
) -> (i32, String) {
    // Exact name match
    if ctx.name == query {
        return (1000, "exact_name".to_string());
    }

    // Alias match
    if ctx.aliases.iter().any(|a| a == query) {
        return (900, "alias".to_string());
    }

    // Name starts with query
    if ctx.name.starts_with(query) {
        return (800, "name_prefix".to_string());
    }

    // Name contains query
    if ctx.name.contains(query) {
        return (700, "name_contains".to_string());
    }

    // Category match
    if ctx.category == query {
        return (600, "category".to_string());
    }

    // Check expanded terms in description/signature
    let mut desc_score = 0;
    let mut matched_terms = Vec::new();

    for term in expanded_terms {
        if ctx.description.contains(term) || ctx.signature.contains(term) {
            desc_score += 100;
            matched_terms.push(term.clone());
        }
    }

    if desc_score > 0 {
        return (
            desc_score,
            format!("description ({})", matched_terms.join(", ")),
        );
    }

    // Fuzzy name match using Jaro-Winkler
    let similarity = jaro_winkler(query, ctx.name);
    if similarity > 0.8 {
        return ((similarity * 500.0) as i32, "fuzzy_name".to_string());
    }

    // Check synonyms
    if let Some(synonyms) = lookup_synonyms(query) {
        for syn in synonyms {
            if ctx.name.contains(syn) || ctx.description.contains(syn) {
                return (300, format!("synonym ({})", syn));
            }
        }
    }

    (0, String::new())
}

/// Parse category string to Category enum
pub(crate) fn parse_category(name: &str) -> Option<Category> {
    let input = name.to_lowercase();
    Category::all()
        .iter()
        .find(|cat| cat.name() == input)
        .copied()
}

/// Count parameters in a function signature, ignoring commas inside brackets.
///
/// Handles signatures like `array, array[[string, string]] -> array` correctly
/// by tracking bracket depth and only counting top-level commas before `->`.
fn count_params(signature: &str) -> usize {
    // Only look at the input side (before "->")
    let input = match signature.find("->") {
        Some(pos) => &signature[..pos],
        None => signature,
    };
    let input = input.trim();
    if input.is_empty() {
        return 0;
    }

    let mut params = 1;
    let mut depth = 0;
    for ch in input.chars() {
        match ch {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            ',' if depth == 0 => params += 1,
            _ => {}
        }
    }
    params
}

/// Extract keywords from a description for related concept matching
fn extract_keywords(description: &str) -> Vec<&str> {
    let stopwords = [
        "a",
        "an",
        "the",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "must",
        "shall",
        "can",
        "to",
        "of",
        "in",
        "for",
        "on",
        "with",
        "at",
        "by",
        "from",
        "or",
        "and",
        "as",
        "if",
        "that",
        "which",
        "this",
        "these",
        "those",
        "it",
        "its",
        "such",
        "when",
        "where",
        "how",
        "all",
        "each",
        "every",
        "both",
        "few",
        "more",
        "most",
        "other",
        "some",
        "any",
        "no",
        "not",
        "only",
        "same",
        "than",
        "very",
        "just",
        "also",
        "into",
        "over",
        "after",
        "before",
        "between",
        "under",
        "again",
        "further",
        "then",
        "once",
        "here",
        "there",
        "why",
        "because",
        "while",
        "although",
        "though",
        "unless",
        "until",
        "whether",
        "returns",
        "return",
        "value",
        "values",
        "given",
        "input",
        "output",
        "function",
        "functions",
        "used",
        "using",
        "use",
    ];

    description
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(&w.to_lowercase().as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::JpxEngine;

    #[test]
    fn test_categories() {
        let engine = JpxEngine::new();
        let cats = engine.categories();
        assert!(!cats.is_empty());
        assert!(cats.iter().any(|c| c == "String"));
    }

    #[test]
    fn test_functions() {
        let engine = JpxEngine::new();

        // All functions
        let all = engine.functions(None);
        assert!(!all.is_empty());

        // Filtered by category
        let string_funcs = engine.functions(Some("String"));
        assert!(!string_funcs.is_empty());
        assert!(string_funcs.iter().all(|f| f.category == "String"));
    }

    #[test]
    fn test_describe_function() {
        let engine = JpxEngine::new();

        let info = engine.describe_function("upper").unwrap();
        assert_eq!(info.name, "upper");
        assert_eq!(info.category, "String");

        let missing = engine.describe_function("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_describe_functions_batch() {
        let engine = JpxEngine::new();

        let results = engine.describe_functions(&["upper", "nonexistent", "lower"]);
        assert_eq!(results.len(), 3);

        // Order is preserved and each name is echoed back.
        assert_eq!(results[0].0, "upper");
        assert_eq!(results[0].1.as_ref().unwrap().name, "upper");

        // Missing names yield None rather than failing the whole batch.
        assert_eq!(results[1].0, "nonexistent");
        assert!(results[1].1.is_none());

        assert_eq!(results[2].0, "lower");
        assert_eq!(results[2].1.as_ref().unwrap().name, "lower");

        // Empty input yields an empty result.
        assert!(engine.describe_functions(&[]).is_empty());
    }

    #[test]
    fn test_search_functions() {
        let engine = JpxEngine::new();

        let results = engine.search_functions("string", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_similar_functions() {
        let engine = JpxEngine::new();

        let result = engine.similar_functions("upper").unwrap();
        // Should have functions in same category
        assert!(!result.same_category.is_empty());
    }

    // =========================================================================
    // Categories tests
    // =========================================================================

    #[test]
    fn test_categories_contains_common() {
        let engine = JpxEngine::new();
        let cats = engine.categories();
        for expected in &["Math", "Array", "Object", "Utility"] {
            assert!(
                cats.iter().any(|c| c == expected),
                "Expected categories to contain {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_categories_no_duplicates() {
        let engine = JpxEngine::new();
        let cats = engine.categories();
        let mut seen = std::collections::HashSet::new();
        for cat in &cats {
            assert!(
                seen.insert(cat.clone()),
                "Duplicate category found: {:?}",
                cat
            );
        }
    }

    // =========================================================================
    // Functions tests
    // =========================================================================

    #[test]
    fn test_functions_all() {
        let engine = JpxEngine::new();
        let all = engine.functions(None);
        assert!(
            !all.is_empty(),
            "functions(None) should return a non-empty list"
        );
        // There should be a substantial number of functions (490+)
        assert!(
            all.len() > 100,
            "Expected at least 100 functions, got {}",
            all.len()
        );
    }

    #[test]
    fn test_functions_invalid_category() {
        let engine = JpxEngine::new();
        let invalid = engine.functions(Some("NonexistentCategory"));
        assert!(
            invalid.is_empty(),
            "Invalid category should return empty list, got {} functions",
            invalid.len()
        );
    }

    #[test]
    fn test_functions_multi_match_category() {
        let engine = JpxEngine::new();
        let results = engine.functions(Some("multi-match"));
        assert!(
            !results.is_empty(),
            "multi-match category should return functions"
        );
        let all = engine.functions(None);
        assert!(
            results.len() < all.len(),
            "multi-match should return a subset, not all {} functions",
            all.len()
        );
    }

    #[test]
    fn test_functions_case_insensitive() {
        let engine = JpxEngine::new();
        let lower = engine.functions(Some("string"));
        let upper = engine.functions(Some("String"));
        assert!(
            !lower.is_empty(),
            "functions(Some(\"string\")) should return results"
        );
        assert_eq!(
            lower.len(),
            upper.len(),
            "Case-insensitive category lookup should return the same number of results"
        );
        // Verify same function names in both results
        let lower_names: Vec<&str> = lower.iter().map(|f| f.name.as_str()).collect();
        let upper_names: Vec<&str> = upper.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(lower_names, upper_names);
    }

    // =========================================================================
    // describe_function tests
    // =========================================================================

    #[test]
    fn test_describe_function_detail_fields() {
        let engine = JpxEngine::new();
        let info = engine.describe_function("length").unwrap();
        assert_eq!(info.name, "length");
        assert!(!info.category.is_empty(), "category should not be empty");
        assert!(info.is_standard, "length should be a standard function");
        assert!(
            !info.description.is_empty(),
            "description should not be empty"
        );
        assert!(!info.signature.is_empty(), "signature should not be empty");
    }

    #[test]
    fn test_describe_function_extension() {
        let engine = JpxEngine::new();
        let info = engine.describe_function("upper").unwrap();
        assert_eq!(info.name, "upper");
        assert!(!info.is_standard, "upper should not be a standard function");
    }

    #[test]
    fn test_describe_function_by_alias() {
        let engine = JpxEngine::new();
        // "all_expr" has alias "every"
        let info = engine.describe_function("all_expr").unwrap();
        assert!(
            info.aliases.contains(&"every".to_string()),
            "all_expr should have alias 'every', aliases: {:?}",
            info.aliases
        );
        // Verify that searching for the alias via search_functions finds it
        let results = engine.search_functions("every", 5);
        assert!(
            results.iter().any(|r| r.function.name == "all_expr"),
            "Searching for alias 'every' should find 'all_expr'"
        );
        let alias_result = results
            .iter()
            .find(|r| r.function.name == "all_expr")
            .unwrap();
        assert_eq!(
            alias_result.match_type, "alias",
            "Match type for alias search should be 'alias'"
        );
        assert_eq!(alias_result.score, 900, "Alias match score should be 900");
    }

    // =========================================================================
    // search_functions tests
    // =========================================================================

    #[test]
    fn test_search_exact_name_match() {
        let engine = JpxEngine::new();
        let results = engine.search_functions("upper", 10);
        assert!(!results.is_empty(), "Should find results for 'upper'");
        let first = &results[0];
        assert_eq!(first.function.name, "upper");
        assert_eq!(first.match_type, "exact_name");
        assert_eq!(first.score, 1000);
    }

    #[test]
    fn test_search_name_prefix_match() {
        let engine = JpxEngine::new();
        let results = engine.search_functions("to_", 20);
        assert!(!results.is_empty(), "Should find results for prefix 'to_'");
        // All top results should have name_prefix match type
        let prefix_results: Vec<_> = results
            .iter()
            .filter(|r| r.match_type == "name_prefix")
            .collect();
        assert!(
            !prefix_results.is_empty(),
            "Should have at least one name_prefix match"
        );
        for r in &prefix_results {
            assert_eq!(r.score, 800, "name_prefix score should be 800");
            assert!(
                r.function.name.starts_with("to_"),
                "Function '{}' should start with 'to_'",
                r.function.name
            );
        }
    }

    #[test]
    fn test_search_name_contains_match() {
        let engine = JpxEngine::new();
        // "left" appears in "pad_left" but "pad_left" does not start with "left"
        let results = engine.search_functions("left", 20);
        let contains_results: Vec<_> = results
            .iter()
            .filter(|r| r.match_type == "name_contains")
            .collect();
        assert!(
            !contains_results.is_empty(),
            "Should have at least one name_contains match for 'left'"
        );
        for r in &contains_results {
            assert_eq!(r.score, 700, "name_contains score should be 700");
            assert!(
                r.function.name.contains("left"),
                "Function '{}' should contain 'left'",
                r.function.name
            );
        }
    }

    #[test]
    fn test_search_description_match() {
        let engine = JpxEngine::new();
        // "converts" is a word likely found in descriptions but not as a function name
        let results = engine.search_functions("converts", 10);
        if !results.is_empty() {
            // If we got description matches, verify the match type
            let desc_matches: Vec<_> = results
                .iter()
                .filter(|r| r.match_type.starts_with("description"))
                .collect();
            assert!(
                !desc_matches.is_empty(),
                "Matches for 'converts' should include description matches"
            );
            for r in &desc_matches {
                assert!(
                    r.score >= 100,
                    "Description match score should be at least 100"
                );
            }
        }
    }

    #[test]
    fn test_search_case_insensitive() {
        let engine = JpxEngine::new();
        let upper_results = engine.search_functions("UPPER", 10);
        let lower_results = engine.search_functions("upper", 10);
        // Both should find the "upper" function
        assert!(
            upper_results.iter().any(|r| r.function.name == "upper"),
            "Searching for 'UPPER' should find 'upper'"
        );
        assert!(
            lower_results.iter().any(|r| r.function.name == "upper"),
            "Searching for 'upper' should find 'upper'"
        );
    }

    #[test]
    fn test_search_respects_limit() {
        let engine = JpxEngine::new();
        let results = engine.search_functions("string", 3);
        assert!(
            results.len() <= 3,
            "Results should be limited to 3, got {}",
            results.len()
        );
    }

    #[test]
    fn test_search_results_ordered_by_score() {
        let engine = JpxEngine::new();
        let results = engine.search_functions("string", 20);
        for window in results.windows(2) {
            assert!(
                window[0].score >= window[1].score,
                "Results should be sorted by score descending: {} (score {}) came before {} (score {})",
                window[0].function.name,
                window[0].score,
                window[1].function.name,
                window[1].score,
            );
        }
    }

    #[test]
    fn test_search_empty_query() {
        let engine = JpxEngine::new();
        // Empty string search should not panic
        let results = engine.search_functions("", 10);
        // Empty query may or may not return results, but it must not panic
        let _ = results;
    }

    #[test]
    fn test_search_no_results() {
        let engine = JpxEngine::new();
        let results = engine.search_functions("xyzqwerty123", 10);
        assert!(
            results.is_empty(),
            "Should find no results for nonsense query, got {}",
            results.len()
        );
    }

    // =========================================================================
    // similar_functions tests
    // =========================================================================

    #[test]
    fn test_similar_functions_nonexistent() {
        let engine = JpxEngine::new();
        let result = engine.similar_functions("nonexistent");
        assert!(
            result.is_none(),
            "similar_functions for nonexistent function should return None"
        );
    }

    #[test]
    fn test_similar_same_category_populated() {
        let engine = JpxEngine::new();
        let result = engine.similar_functions("upper").unwrap();
        assert!(
            !result.same_category.is_empty(),
            "same_category should be populated for 'upper'"
        );
        // All same_category results should be String functions (same as upper)
        for f in &result.same_category {
            assert_eq!(
                f.category, "String",
                "same_category function '{}' should be in String category, got '{}'",
                f.name, f.category
            );
        }
        // "upper" itself should not appear in the same_category list
        assert!(
            !result.same_category.iter().any(|f| f.name == "upper"),
            "same_category should not contain the function itself"
        );
    }

    #[test]
    fn test_similar_functions_deterministic_and_ordered() {
        let engine = JpxEngine::new();
        let a = engine.similar_functions("upper").unwrap();
        let b = engine.similar_functions("upper").unwrap();

        let names = |r: &super::SimilarFunctionsResult| {
            (
                r.same_category
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<Vec<_>>(),
                r.similar_signature
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<Vec<_>>(),
                r.related_concepts
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<Vec<_>>(),
            )
        };
        // Repeated calls must return identical results (no HashMap-order flakiness).
        assert_eq!(names(&a), names(&b));

        // same_category and similar_signature are name-sorted.
        let cat: Vec<_> = a.same_category.iter().map(|f| &f.name).collect();
        let mut cat_sorted = cat.clone();
        cat_sorted.sort();
        assert_eq!(cat, cat_sorted, "same_category should be name-sorted");

        let sig: Vec<_> = a.similar_signature.iter().map(|f| &f.name).collect();
        let mut sig_sorted = sig.clone();
        sig_sorted.sort();
        assert_eq!(sig, sig_sorted, "similar_signature should be name-sorted");
    }

    #[test]
    fn test_count_params_simple() {
        assert_eq!(super::count_params("string -> string"), 1);
        assert_eq!(super::count_params("string, string -> string"), 2);
        assert_eq!(super::count_params("string, number, string -> string"), 3);
    }

    #[test]
    fn test_count_params_brackets() {
        // order_by: array, array[[string, string]] -> array
        assert_eq!(
            super::count_params("array, array[[string, string]] -> array"),
            2
        );
        // nested brackets
        assert_eq!(
            super::count_params("array[object[string, number]] -> array"),
            1
        );
    }

    #[test]
    fn test_count_params_optional_and_variadic() {
        assert_eq!(super::count_params("string, string? -> string"), 2);
        assert_eq!(super::count_params("string, ...any -> array"), 2);
    }

    #[test]
    fn test_count_params_empty() {
        assert_eq!(super::count_params("-> string"), 0);
    }

    #[test]
    fn test_similar_signature_uses_correct_arity() {
        let engine = super::super::JpxEngine::new();
        // order_by has 2 params; similar_signature should match other 2-param functions
        let result = engine.similar_functions("order_by").unwrap();
        for f in &result.similar_signature {
            let arity = super::count_params(&f.signature);
            assert_eq!(
                arity, 2,
                "similar_signature match '{}' (sig: '{}') should have arity 2, got {}",
                f.name, f.signature, arity
            );
        }
    }

    #[test]
    fn test_similar_functions_serde() {
        let engine = JpxEngine::new();
        let result = engine.similar_functions("upper").unwrap();
        // Should serialize to JSON without error
        let json = serde_json::to_string(&result)
            .expect("SimilarFunctionsResult should serialize to JSON");
        assert!(!json.is_empty(), "Serialized JSON should not be empty");
        // Should deserialize back
        let deserialized: super::SimilarFunctionsResult = serde_json::from_str(&json)
            .expect("SimilarFunctionsResult should deserialize from JSON");
        assert_eq!(
            result.same_category.len(),
            deserialized.same_category.len(),
            "Round-trip should preserve same_category length"
        );
    }
}
