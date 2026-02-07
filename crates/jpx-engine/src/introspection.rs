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
        match category.and_then(parse_category) {
            Some(cat) => self
                .registry
                .functions_in_category(cat)
                .map(FunctionDetail::from)
                .collect(),
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

        // Same category
        let same_category: Vec<FunctionDetail> = all_functions
            .iter()
            .filter(|f| f.category == info.category && f.name != info.name)
            .take(5)
            .map(|f| FunctionDetail::from(*f))
            .collect();

        // Similar signature (same arity)
        let this_arity = count_params(info.signature);
        let similar_signature: Vec<FunctionDetail> = all_functions
            .iter()
            .filter(|f| {
                f.name != info.name
                    && f.category != info.category
                    && count_params(f.signature) == this_arity
            })
            .take(5)
            .map(|f| FunctionDetail::from(*f))
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

        concept_scores.sort_by(|a, b| b.1.cmp(&a.1));

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
    Category::all()
        .iter()
        .find(|cat| format!("{:?}", cat).to_lowercase() == name.to_lowercase())
        .copied()
}

/// Count parameters in a function signature
fn count_params(signature: &str) -> usize {
    signature.matches(',').count() + 1
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
}
