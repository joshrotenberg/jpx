//! Function registry for runtime control and introspection.
//!
//! The registry provides:
//! - Runtime enable/disable of functions (for ACLs, config-based gating)
//! - Introspection (list available functions, their signatures, descriptions)
//! - Category-based registration
//! - Metadata about standard vs extension functions and JEP alignment

use std::collections::{HashMap, HashSet};

use crate::Runtime;
use crate::functions::Function;

/// Function category matching compile-time features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Standard JMESPath built-in functions (always available)
    Standard,
    String,
    Array,
    Object,
    Math,
    Type,
    Utility,
    Validation,
    Path,
    Expression,
    Text,
    Hash,
    Encoding,
    Regex,
    Url,
    Uuid,
    Rand,
    Datetime,
    Fuzzy,
    Phonetic,
    Geo,
    Semver,
    Network,
    Ids,
    Duration,
    Color,
    Computing,
    MultiMatch,
    Jsonpatch,
    Format,
    Language,
    Discovery,
}

impl Category {
    /// Returns all categories (including Standard).
    pub fn all() -> &'static [Category] {
        &[
            Category::Standard,
            Category::String,
            Category::Array,
            Category::Object,
            Category::Math,
            Category::Type,
            Category::Utility,
            Category::Validation,
            Category::Path,
            Category::Expression,
            Category::Text,
            Category::Hash,
            Category::Encoding,
            Category::Regex,
            Category::Url,
            Category::Uuid,
            Category::Rand,
            Category::Datetime,
            Category::Fuzzy,
            Category::Phonetic,
            Category::Geo,
            Category::Semver,
            Category::Network,
            Category::Ids,
            Category::Duration,
            Category::Color,
            Category::Computing,
            Category::MultiMatch,
            Category::Jsonpatch,
            Category::Format,
            Category::Language,
            Category::Discovery,
        ]
    }

    /// Returns whether this category is available (compiled in).
    ///
    /// In jpx-core, all categories are always available when the
    /// `extensions` feature is enabled. Standard is always available.
    pub fn is_available(&self) -> bool {
        match self {
            Category::Standard => true,
            _ => cfg!(feature = "extensions"),
        }
    }

    /// Returns the category name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Category::Standard => "standard",
            Category::String => "string",
            Category::Array => "array",
            Category::Object => "object",
            Category::Math => "math",
            Category::Type => "type",
            Category::Utility => "utility",
            Category::Validation => "validation",
            Category::Path => "path",
            Category::Expression => "expression",
            Category::Text => "text",
            Category::Hash => "hash",
            Category::Encoding => "encoding",
            Category::Regex => "regex",
            Category::Url => "url",
            Category::Uuid => "uuid",
            Category::Rand => "rand",
            Category::Datetime => "datetime",
            Category::Fuzzy => "fuzzy",
            Category::Phonetic => "phonetic",
            Category::Geo => "geo",
            Category::Semver => "semver",
            Category::Network => "network",
            Category::Ids => "ids",
            Category::Duration => "duration",
            Category::Color => "color",
            Category::Computing => "computing",
            Category::MultiMatch => "multi-match",
            Category::Jsonpatch => "jsonpatch",
            Category::Format => "format",
            Category::Language => "language",
            Category::Discovery => "discovery",
        }
    }
}

/// Feature tags for function classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Standard JMESPath spec functions
    Spec,
    /// Core extension functions
    Core,
    /// Functional programming style functions
    Fp,
    /// JEP-aligned functions
    Jep,
    /// Format output functions (CSV, TSV)
    #[allow(non_camel_case_types)]
    format,
    /// Environment variable access (opt-in for security)
    #[allow(non_camel_case_types)]
    env,
    /// Discovery/search functions for tool discovery
    #[allow(non_camel_case_types)]
    discovery,
}

impl Feature {
    /// Returns all features.
    pub fn all() -> &'static [Feature] {
        &[
            Feature::Spec,
            Feature::Core,
            Feature::Fp,
            Feature::Jep,
            Feature::format,
            Feature::env,
            Feature::discovery,
        ]
    }

    /// Returns the feature name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Feature::Spec => "spec",
            Feature::Core => "core",
            Feature::Fp => "fp",
            Feature::Jep => "jep",
            Feature::format => "format",
            Feature::env => "env",
            Feature::discovery => "discovery",
        }
    }
}

/// Metadata about a function.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name as used in JMESPath expressions.
    pub name: &'static str,
    /// Category this function belongs to.
    pub category: Category,
    /// Human-readable description.
    pub description: &'static str,
    /// Argument signature (e.g., "string, string -> string").
    pub signature: &'static str,
    /// Example usage.
    pub example: &'static str,
    /// Whether this is a standard JMESPath function (vs extension).
    pub is_standard: bool,
    /// JMESPath Enhancement Proposal reference (e.g., "JEP-014").
    pub jep: Option<&'static str>,
    /// Alternative names for this function.
    pub aliases: &'static [&'static str],
    /// Feature tags for classification.
    pub features: &'static [Feature],
}

/// Registry for managing function availability at runtime.
#[derive(Debug, Clone)]
pub struct FunctionRegistry {
    /// Functions that have been registered.
    registered: HashMap<&'static str, FunctionInfo>,
    /// Functions that have been explicitly disabled.
    disabled: HashSet<String>,
    /// Categories that have been registered.
    categories: HashSet<Category>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
            disabled: HashSet::new(),
            categories: HashSet::new(),
        }
    }

    /// Registers all available functions.
    pub fn register_all(&mut self) -> &mut Self {
        for category in Category::all() {
            self.register_category(*category);
        }
        self
    }

    /// Registers all functions in a category.
    pub fn register_category(&mut self, category: Category) -> &mut Self {
        self.categories.insert(category);
        for info in get_category_functions(category) {
            self.registered.insert(info.name, info);
        }
        self
    }

    /// Disables a specific function (for ACLs).
    pub fn disable_function(&mut self, name: &str) -> &mut Self {
        self.disabled.insert(name.to_string());
        self
    }

    /// Enables a previously disabled function.
    pub fn enable_function(&mut self, name: &str) -> &mut Self {
        self.disabled.remove(name);
        self
    }

    /// Checks if a function is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.registered.contains_key(name) && !self.disabled.contains(name)
    }

    /// Gets info about a specific function.
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        if self.disabled.contains(name) {
            None
        } else {
            self.registered.get(name)
        }
    }

    /// Iterates over all enabled functions.
    pub fn functions(&self) -> impl Iterator<Item = &FunctionInfo> {
        self.registered
            .values()
            .filter(|f| !self.disabled.contains(f.name))
    }

    /// Iterates over functions in a specific category.
    pub fn functions_in_category(&self, category: Category) -> impl Iterator<Item = &FunctionInfo> {
        self.registered
            .values()
            .filter(move |f| f.category == category && !self.disabled.contains(f.name))
    }

    /// Gets all registered categories.
    pub fn categories(&self) -> impl Iterator<Item = &Category> {
        self.categories.iter()
    }

    /// Gets count of enabled functions.
    pub fn len(&self) -> usize {
        self.registered.len() - self.disabled.len()
    }

    /// Checks if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterates over functions with a specific feature tag.
    pub fn functions_with_feature(&self, feature: Feature) -> impl Iterator<Item = &FunctionInfo> {
        self.registered
            .values()
            .filter(move |f| f.features.contains(&feature) && !self.disabled.contains(f.name))
    }

    /// Gets all spec-only (standard JMESPath) function names.
    pub fn spec_function_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.functions_with_feature(Feature::Spec).map(|f| f.name)
    }

    /// Checks if a function is a standard JMESPath spec function.
    pub fn is_spec_function(&self, name: &str) -> bool {
        self.registered
            .get(name)
            .is_some_and(|f| f.features.contains(&Feature::Spec))
    }

    /// Gets function info by name or alias.
    pub fn get_function_by_name_or_alias(&self, name: &str) -> Option<&FunctionInfo> {
        if let Some(info) = self.get_function(name) {
            return Some(info);
        }
        self.registered
            .values()
            .find(|f| f.aliases.contains(&name) && !self.disabled.contains(f.name))
    }

    /// Gets all aliases as (alias, canonical_name) pairs.
    pub fn all_aliases(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.registered
            .values()
            .flat_map(|f| f.aliases.iter().map(move |alias| (*alias, f.name)))
    }

    /// Applies the registry to a runtime, registering extension functions.
    ///
    /// This calls each category's `register_filtered` function to add
    /// enabled extension functions to the runtime. Standard functions
    /// are registered separately via `runtime.register_builtin_functions()`.
    pub fn apply(&self, runtime: &mut Runtime) {
        for category in &self.categories {
            self.apply_category(runtime, *category);
        }
    }

    #[allow(unused_variables)]
    fn apply_category(&self, runtime: &mut Runtime, category: Category) {
        let enabled_in_category: HashSet<&str> = self
            .functions_in_category(category)
            .map(|f| f.name)
            .collect();

        if enabled_in_category.is_empty() {
            return;
        }

        // Extension function registration is gated behind the "extensions" feature.
        // Each category module provides a register_filtered function that registers
        // only the functions in the enabled set.
        #[cfg(feature = "extensions")]
        match category {
            Category::String => {
                crate::extensions::string::register_filtered(runtime, &enabled_in_category)
            }
            Category::Array => {
                crate::extensions::array::register_filtered(runtime, &enabled_in_category)
            }
            Category::Object => {
                crate::extensions::object::register_filtered(runtime, &enabled_in_category)
            }
            Category::Math => {
                crate::extensions::math::register_filtered(runtime, &enabled_in_category)
            }
            Category::Type => {
                crate::extensions::type_conv::register_filtered(runtime, &enabled_in_category)
            }
            Category::Utility => {
                crate::extensions::utility::register_filtered(runtime, &enabled_in_category)
            }
            Category::Validation => {
                crate::extensions::validation::register_filtered(runtime, &enabled_in_category)
            }
            Category::Path => {
                crate::extensions::path::register_filtered(runtime, &enabled_in_category)
            }
            Category::Expression => {
                crate::extensions::expression::register_filtered(runtime, &enabled_in_category)
            }
            Category::Text => {
                crate::extensions::text::register_filtered(runtime, &enabled_in_category)
            }
            Category::Hash => {
                crate::extensions::hash::register_filtered(runtime, &enabled_in_category)
            }
            Category::Encoding => {
                crate::extensions::encoding::register_filtered(runtime, &enabled_in_category)
            }
            Category::Regex => {
                crate::extensions::regex_fns::register_filtered(runtime, &enabled_in_category)
            }
            Category::Url => {
                crate::extensions::url_fns::register_filtered(runtime, &enabled_in_category)
            }
            Category::Uuid => {
                crate::extensions::random::register_filtered(runtime, &enabled_in_category)
            }
            Category::Rand => {
                crate::extensions::random::register_filtered(runtime, &enabled_in_category)
            }
            Category::Datetime => {
                crate::extensions::datetime::register_filtered(runtime, &enabled_in_category)
            }
            Category::Fuzzy => {
                crate::extensions::fuzzy::register_filtered(runtime, &enabled_in_category)
            }
            Category::Phonetic => {
                crate::extensions::phonetic::register_filtered(runtime, &enabled_in_category)
            }
            Category::Geo => {
                crate::extensions::geo::register_filtered(runtime, &enabled_in_category)
            }
            Category::Semver => {
                crate::extensions::semver_fns::register_filtered(runtime, &enabled_in_category)
            }
            Category::Network => {
                crate::extensions::network::register_filtered(runtime, &enabled_in_category)
            }
            Category::Ids => {
                crate::extensions::ids::register_filtered(runtime, &enabled_in_category)
            }
            Category::Duration => {
                crate::extensions::duration::register_filtered(runtime, &enabled_in_category)
            }
            Category::Color => {
                crate::extensions::color::register_filtered(runtime, &enabled_in_category)
            }
            Category::Computing => {
                crate::extensions::computing::register_filtered(runtime, &enabled_in_category)
            }
            Category::MultiMatch => {
                crate::extensions::multi_match::register_filtered(runtime, &enabled_in_category)
            }
            Category::Jsonpatch => {
                crate::extensions::jsonpatch::register_filtered(runtime, &enabled_in_category)
            }
            Category::Format => {
                crate::extensions::format::register_filtered(runtime, &enabled_in_category)
            }
            Category::Language => {
                crate::extensions::language::register_filtered(runtime, &enabled_in_category)
            }
            Category::Discovery => {
                crate::extensions::discovery::register_filtered(runtime, &enabled_in_category)
            }
            Category::Standard => {} // Standard functions are registered via register_builtin_functions()
        }

        // When extensions feature is not compiled, only standard functions are available.
        #[cfg(not(feature = "extensions"))]
        let _ = (runtime, category);
    }
}

/// Gets function metadata for a category (from generated data).
fn get_category_functions(category: Category) -> Vec<FunctionInfo> {
    generated::FUNCTIONS
        .iter()
        .filter(|f| f.category == category)
        .cloned()
        .collect()
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/registry_data.rs"));
}

/// Helper to register a function if its name is in the enabled set.
pub fn register_if_enabled(
    runtime: &mut Runtime,
    name: &str,
    enabled: &HashSet<&str>,
    f: Box<dyn Function>,
) {
    if enabled.contains(name) {
        runtime.register_function(name, f);
    }
}

// =============================================================================
// Synonym mapping for function discovery
// =============================================================================

/// Synonym entry mapping a search term to related function names/keywords.
pub struct SynonymEntry {
    /// The search term (e.g., "aggregate").
    pub term: &'static str,
    /// Related function names or keywords that should match.
    pub targets: &'static [&'static str],
}

/// Gets all synonym mappings for function discovery.
pub fn get_synonyms() -> &'static [SynonymEntry] {
    static SYNONYMS: &[SynonymEntry] = &[
        SynonymEntry {
            term: "aggregate",
            targets: &[
                "group_by",
                "group_by_expr",
                "sum",
                "avg",
                "count",
                "reduce",
                "fold",
            ],
        },
        SynonymEntry {
            term: "group",
            targets: &["group_by", "group_by_expr", "chunk", "partition"],
        },
        SynonymEntry {
            term: "collect",
            targets: &["group_by", "group_by_expr", "flatten", "merge"],
        },
        SynonymEntry {
            term: "bucket",
            targets: &["group_by", "group_by_expr", "chunk"],
        },
        SynonymEntry {
            term: "count",
            targets: &["length", "count_by", "count_if", "size"],
        },
        SynonymEntry {
            term: "size",
            targets: &["length", "count_by"],
        },
        SynonymEntry {
            term: "len",
            targets: &["length"],
        },
        SynonymEntry {
            term: "concat",
            targets: &["join", "merge", "combine"],
        },
        SynonymEntry {
            term: "combine",
            targets: &["join", "merge", "concat"],
        },
        SynonymEntry {
            term: "substring",
            targets: &["slice", "substr", "mid"],
        },
        SynonymEntry {
            term: "cut",
            targets: &["slice", "split", "trim"],
        },
        SynonymEntry {
            term: "strip",
            targets: &["trim", "trim_left", "trim_right"],
        },
        SynonymEntry {
            term: "lowercase",
            targets: &["lower", "to_lower", "downcase"],
        },
        SynonymEntry {
            term: "uppercase",
            targets: &["upper", "to_upper", "upcase"],
        },
        SynonymEntry {
            term: "replace",
            targets: &["substitute", "gsub", "regex_replace"],
        },
        SynonymEntry {
            term: "find",
            targets: &["contains", "index_of", "search", "match"],
        },
        SynonymEntry {
            term: "search",
            targets: &["contains", "find", "index_of", "regex_match"],
        },
        SynonymEntry {
            term: "filter",
            targets: &["select", "where", "find_all", "keep"],
        },
        SynonymEntry {
            term: "select",
            targets: &["filter", "map", "pluck"],
        },
        SynonymEntry {
            term: "transform",
            targets: &["map", "transform_values", "map_values"],
        },
        SynonymEntry {
            term: "first",
            targets: &["head", "take", "front"],
        },
        SynonymEntry {
            term: "last",
            targets: &["tail", "end", "back"],
        },
        SynonymEntry {
            term: "remove",
            targets: &["reject", "delete", "drop", "exclude"],
        },
        SynonymEntry {
            term: "unique",
            targets: &["distinct", "uniq", "dedupe", "deduplicate"],
        },
        SynonymEntry {
            term: "dedupe",
            targets: &["unique", "distinct", "uniq"],
        },
        SynonymEntry {
            term: "shuffle",
            targets: &["random", "randomize", "permute"],
        },
        SynonymEntry {
            term: "order",
            targets: &["sort", "sort_by", "order_by", "arrange"],
        },
        SynonymEntry {
            term: "arrange",
            targets: &["sort", "sort_by", "order_by"],
        },
        SynonymEntry {
            term: "rank",
            targets: &["sort", "sort_by", "order_by"],
        },
        SynonymEntry {
            term: "average",
            targets: &["avg", "mean", "arithmetic_mean"],
        },
        SynonymEntry {
            term: "mean",
            targets: &["avg", "average"],
        },
        SynonymEntry {
            term: "total",
            targets: &["sum", "add", "accumulate"],
        },
        SynonymEntry {
            term: "add",
            targets: &["sum", "plus", "addition"],
        },
        SynonymEntry {
            term: "subtract",
            targets: &["minus", "difference"],
        },
        SynonymEntry {
            term: "multiply",
            targets: &["times", "product", "mul"],
        },
        SynonymEntry {
            term: "divide",
            targets: &["quotient", "div"],
        },
        SynonymEntry {
            term: "remainder",
            targets: &["mod", "modulo", "modulus"],
        },
        SynonymEntry {
            term: "power",
            targets: &["pow", "exponent", "exp"],
        },
        SynonymEntry {
            term: "absolute",
            targets: &["abs", "magnitude"],
        },
        SynonymEntry {
            term: "round",
            targets: &["round", "round_to", "nearest"],
        },
        SynonymEntry {
            term: "random",
            targets: &["rand", "random_int", "random_float"],
        },
        SynonymEntry {
            term: "date",
            targets: &["now", "today", "parse_date", "format_date", "datetime"],
        },
        SynonymEntry {
            term: "time",
            targets: &["now", "time_now", "parse_time", "datetime"],
        },
        SynonymEntry {
            term: "timestamp",
            targets: &["now", "epoch", "unix_time", "to_epoch"],
        },
        SynonymEntry {
            term: "format",
            targets: &["format_date", "strftime", "date_format"],
        },
        SynonymEntry {
            term: "parse",
            targets: &["parse_date", "parse_time", "strptime", "from_string"],
        },
        SynonymEntry {
            term: "convert",
            targets: &["to_string", "to_number", "to_array", "type"],
        },
        SynonymEntry {
            term: "cast",
            targets: &["to_string", "to_number", "to_bool"],
        },
        SynonymEntry {
            term: "stringify",
            targets: &["to_string", "string", "str"],
        },
        SynonymEntry {
            term: "numberify",
            targets: &["to_number", "number", "int", "float"],
        },
        SynonymEntry {
            term: "object",
            targets: &["from_items", "to_object", "merge", "object_from_items"],
        },
        SynonymEntry {
            term: "dict",
            targets: &["from_items", "to_object", "object"],
        },
        SynonymEntry {
            term: "hash",
            targets: &["md5", "sha256", "sha1", "crc32"],
        },
        SynonymEntry {
            term: "encrypt",
            targets: &["md5", "sha256", "sha1", "hmac"],
        },
        SynonymEntry {
            term: "checksum",
            targets: &["md5", "sha256", "crc32"],
        },
        SynonymEntry {
            term: "encode",
            targets: &["base64_encode", "url_encode", "hex_encode"],
        },
        SynonymEntry {
            term: "decode",
            targets: &["base64_decode", "url_decode", "hex_decode"],
        },
        SynonymEntry {
            term: "escape",
            targets: &["url_encode", "html_escape"],
        },
        SynonymEntry {
            term: "unescape",
            targets: &["url_decode", "html_unescape"],
        },
        SynonymEntry {
            term: "check",
            targets: &[
                "is_string",
                "is_number",
                "is_array",
                "is_object",
                "validate",
            ],
        },
        SynonymEntry {
            term: "validate",
            targets: &["is_email", "is_url", "is_uuid", "is_valid"],
        },
        SynonymEntry {
            term: "test",
            targets: &["regex_match", "contains", "starts_with", "ends_with"],
        },
        SynonymEntry {
            term: "default",
            targets: &["coalesce", "if_null", "or_else", "default_value"],
        },
        SynonymEntry {
            term: "empty",
            targets: &["is_empty", "blank", "null"],
        },
        SynonymEntry {
            term: "null",
            targets: &["is_null", "coalesce", "not_null"],
        },
        SynonymEntry {
            term: "fallback",
            targets: &["coalesce", "default", "or_else"],
        },
        SynonymEntry {
            term: "equal",
            targets: &["eq", "equals", "same"],
        },
        SynonymEntry {
            term: "compare",
            targets: &["eq", "lt", "gt", "lte", "gte", "cmp"],
        },
        SynonymEntry {
            term: "between",
            targets: &["range", "in_range", "clamp"],
        },
        SynonymEntry {
            term: "copy",
            targets: &["clone", "dup", "duplicate"],
        },
        SynonymEntry {
            term: "debug",
            targets: &["debug", "inspect", "dump", "print"],
        },
        SynonymEntry {
            term: "reverse",
            targets: &["reverse", "flip", "invert"],
        },
        SynonymEntry {
            term: "repeat",
            targets: &["repeat", "replicate", "times"],
        },
        SynonymEntry {
            term: "uuid",
            targets: &["uuid", "uuid4", "guid", "generate_uuid"],
        },
        SynonymEntry {
            term: "id",
            targets: &["uuid", "nanoid", "ulid", "unique_id"],
        },
    ];
    SYNONYMS
}

/// Looks up synonyms for a search term.
pub fn lookup_synonyms(term: &str) -> Option<&'static [&'static str]> {
    let term_lower = term.to_lowercase();
    get_synonyms()
        .iter()
        .find(|s| s.term == term_lower)
        .map(|s| s.targets)
}

/// Expands a search query using synonyms.
pub fn expand_search_terms(query: &str) -> Vec<String> {
    let mut expanded = Vec::new();
    for word in query.split_whitespace() {
        let word_lower = word.to_lowercase();
        expanded.push(word_lower.clone());
        if let Some(targets) = lookup_synonyms(&word_lower) {
            for target in targets {
                let target_str = (*target).to_string();
                if !expanded.contains(&target_str) {
                    expanded.push(target_str);
                }
            }
        }
    }
    expanded
}
