use anyhow::Result;
use colored::Colorize;
use jpx_engine::{Category, FunctionInfo, FunctionRegistry};

use crate::util::truncate_str;

pub(crate) fn print_functions(registry: &FunctionRegistry) {
    println!(
        "{}\n",
        "jpx - JMESPath with Extended Functions".bold().green()
    );

    // Count standard and extension functions
    let standard_count = registry.functions().filter(|f| f.is_standard).count();
    let extension_count = registry.functions().filter(|f| !f.is_standard).count();

    // Print standard functions header
    println!(
        "  {} {}:",
        "▸".dimmed(),
        format!("STANDARD ({})", standard_count).dimmed()
    );
    for func in registry.functions_in_category(Category::Standard) {
        println!(
            "    {} [{}] - {}",
            func.name.cyan().bold(),
            "std".dimmed(),
            func.description
        );
    }
    println!();

    // Print extension functions by category
    println!(
        "{} ({} available):\n",
        "Extension functions".bold(),
        extension_count.to_string().yellow()
    );

    for category in Category::all() {
        if *category == Category::Standard || !category.is_available() {
            continue;
        }

        let funcs: Vec<_> = registry.functions_in_category(*category).collect();
        if funcs.is_empty() {
            continue;
        }

        println!(
            "  {} {}:",
            "▸".dimmed(),
            format!("{} ({})", category.name().to_uppercase(), funcs.len())
                .green()
                .bold()
        );
        for func in funcs {
            println!("    {} - {}", func.name.cyan().bold(), func.description);
        }
        println!();
    }

    println!(
        "Use {} for details on a category",
        "--list-category <name>".cyan()
    );
    println!(
        "Use {} for details on a specific function",
        "--describe <function>".cyan()
    );
    println!("Use {} to find functions", "--search <query>".cyan());
    println!(
        "\nFor full documentation: {}",
        "https://joshrotenberg.github.io/jpx/".blue().underline()
    );
}

/// Search result with relevance score
struct SearchResult<'a> {
    func: &'a FunctionInfo,
    score: i32,
    match_type: MatchType,
}

#[derive(Debug, Clone, Copy)]
enum MatchType {
    ExactName,
    NamePrefix,
    NameContains,
    AliasMatch,
    CategoryMatch,
    DescriptionMatch,
    SignatureMatch,
}

impl MatchType {
    fn label(&self) -> &'static str {
        match self {
            MatchType::ExactName => "exact",
            MatchType::NamePrefix => "prefix",
            MatchType::NameContains => "name",
            MatchType::AliasMatch => "alias",
            MatchType::CategoryMatch => "category",
            MatchType::DescriptionMatch => "description",
            MatchType::SignatureMatch => "signature",
        }
    }
}

pub(crate) fn search_functions(registry: &FunctionRegistry, query: &str) {
    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    for func in registry.functions() {
        let name_lower = func.name.to_lowercase();
        let desc_lower = func.description.to_lowercase();
        let sig_lower = func.signature.to_lowercase();
        let cat_lower = func.category.name().to_lowercase();

        // Scoring: higher = better match
        let (score, match_type) = if name_lower == query_lower {
            (1000, MatchType::ExactName)
        } else if name_lower.starts_with(&query_lower) {
            (800, MatchType::NamePrefix)
        } else if name_lower.contains(&query_lower) {
            (600, MatchType::NameContains)
        } else if func
            .aliases
            .iter()
            .any(|a| a.to_lowercase().contains(&query_lower))
        {
            (500, MatchType::AliasMatch)
        } else if cat_lower.contains(&query_lower) {
            (400, MatchType::CategoryMatch)
        } else if desc_lower.contains(&query_lower) {
            // Boost if query appears early in description
            let pos = desc_lower.find(&query_lower).unwrap_or(100);
            (300 - pos.min(100) as i32, MatchType::DescriptionMatch)
        } else if sig_lower.contains(&query_lower) {
            (100, MatchType::SignatureMatch)
        } else {
            continue; // No match
        };

        results.push(SearchResult {
            func,
            score,
            match_type,
        });
    }

    // Sort by score descending
    results.sort_by_key(|b| std::cmp::Reverse(b.score));

    if results.is_empty() {
        println!(
            "{} No functions found matching '{}'\n",
            "✗".red(),
            query.yellow()
        );
        println!("Try searching for:");
        println!("  • Function names: {}", "median, split, json".cyan());
        println!("  • Categories: {}", "math, string, array".cyan());
        println!("  • Concepts: {}", "hash, encode, date".cyan());
        return;
    }

    println!(
        "{} Found {} functions matching '{}':\n",
        "✓".green(),
        results.len().to_string().yellow(),
        query.cyan()
    );

    // Group by match type for nicer display
    let mut current_group: Option<&str> = None;

    for result in results.iter().take(25) {
        let group = result.match_type.label();
        if current_group != Some(group) {
            if current_group.is_some() {
                println!();
            }
            println!("  {} {}:", "▸".dimmed(), group.to_uppercase().dimmed());
            current_group = Some(group);
        }

        let type_badge = if result.func.is_standard {
            "std".dimmed()
        } else {
            result.func.category.name().green()
        };

        println!(
            "    {} [{}] - {}",
            result.func.name.cyan().bold(),
            type_badge,
            result.func.description
        );
    }

    if results.len() > 25 {
        println!(
            "\n  {} and {} more... (refine your search)",
            "...".dimmed(),
            (results.len() - 25).to_string().yellow()
        );
    }

    println!(
        "\nUse {} for details on a specific function",
        "--describe <function>".cyan()
    );
}

/// Try to extract an unknown function name from an error message and return
/// up to `limit` similar function suggestions as a formatted string.
pub(crate) fn suggest_for_unknown_function(
    registry: &FunctionRegistry,
    err_msg: &str,
    limit: usize,
) -> Option<String> {
    // Error format: "Unknown function: <name>"
    let name = err_msg.strip_prefix("Unknown function: ").or_else(|| {
        // Also match when wrapped in larger error messages
        err_msg
            .find("Unknown function: ")
            .map(|pos| &err_msg[pos + 18..])
    })?;
    // Take just the function name (first word)
    let name = name.split_whitespace().next().unwrap_or(name).trim();
    if name.is_empty() {
        return None;
    }

    let name_lower = name.to_lowercase();
    let mut scored: Vec<(&FunctionInfo, i32)> = registry
        .functions()
        .filter_map(|f| {
            let f_lower = f.name.to_lowercase();
            let score = if f_lower.starts_with(&name_lower) {
                800
            } else if f_lower.contains(&name_lower) || name_lower.contains(&f_lower) {
                600
            } else if f
                .aliases
                .iter()
                .any(|a| a.to_lowercase().contains(&name_lower))
            {
                500
            } else {
                // Simple character overlap heuristic
                let common: usize = name_lower.chars().filter(|c| f_lower.contains(*c)).count();
                let ratio = common as f64 / name_lower.len().max(f_lower.len()) as f64;
                if ratio > 0.5 {
                    (ratio * 400.0) as i32
                } else {
                    return None;
                }
            };
            Some((f, score))
        })
        .collect();

    scored.sort_by_key(|b| std::cmp::Reverse(b.1));

    if scored.is_empty() {
        return None;
    }

    let mut lines = vec![String::new(), "Did you mean?".to_string()];
    for (f, _) in scored.iter().take(limit) {
        lines.push(format!(
            "  - {} ({}: {})",
            f.name,
            f.category.name(),
            f.description
        ));
    }
    Some(lines.join("\n"))
}

pub(crate) fn print_category(registry: &FunctionRegistry, category_name: &str) -> Result<()> {
    let category = Category::all()
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(category_name))
        .ok_or_else(|| {
            let available: Vec<_> = Category::all()
                .iter()
                .filter(|c| c.is_available())
                .map(|c| c.name())
                .collect();
            anyhow::anyhow!(
                "Unknown category '{}'. Available: {}",
                category_name,
                available.join(", ")
            )
        })?;

    if !category.is_available() {
        return Err(anyhow::anyhow!(
            "Category '{}' is not available (not compiled in)",
            category_name
        ));
    }

    println!(
        "{} functions:\n",
        category.name().to_uppercase().green().bold()
    );

    for func in registry.functions_in_category(*category) {
        println!("  {} - {}", func.name.cyan().bold(), func.description);
        println!("    {}: {}", "Signature".dimmed(), func.signature);
        println!("    {}: {}", "Example".dimmed(), func.example.yellow());
        println!();
    }

    Ok(())
}

pub(crate) fn describe_function(registry: &FunctionRegistry, func_name: &str) -> Result<()> {
    let func = registry.get_function(func_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown function '{}'. Use --list-functions to see available functions.",
            func_name
        )
    })?;

    println!("{}", func.name.cyan().bold());
    println!("{}", "=".repeat(func.name.len()).dimmed());
    println!();
    println!(
        "{}: {}",
        "Type".dimmed(),
        if func.is_standard {
            "standard JMESPath".normal()
        } else {
            "extension".green()
        }
    );
    println!("{}: {}", "Category".dimmed(), func.category.name().green());
    if let Some(jep) = func.jep {
        println!("{}: {}", "JEP".dimmed(), jep.yellow());
    }
    println!("{}: {}", "Description".dimmed(), func.description);
    println!("{}: {}", "Signature".dimmed(), func.signature.white());
    println!();
    println!("{}:", "Example".bold());
    println!("  {}", func.example.yellow());

    Ok(())
}

/// Find functions similar to the specified function
pub(crate) fn find_similar_functions(registry: &FunctionRegistry, func_name: &str) -> Result<()> {
    let target = registry.get_function(func_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown function '{}'. Use --list-functions to see available functions.",
            func_name
        )
    })?;

    println!("Functions similar to '{}':\n", target.name.cyan().bold());

    // Parse the target signature to extract input/output types
    let target_sig = parse_signature(target.signature);

    // 1. Same category (excluding the target itself)
    let same_category: Vec<_> = registry
        .functions_in_category(target.category)
        .filter(|f| f.name != target.name)
        .collect();

    if !same_category.is_empty() {
        println!(
            "  {} {} ({}):",
            "▸".dimmed(),
            "Same category".bold(),
            target.category.name().green()
        );
        for func in same_category.iter().take(8) {
            println!(
                "    {} - {}",
                func.name.cyan(),
                truncate_str(func.description, 50)
            );
        }
        if same_category.len() > 8 {
            println!(
                "    {} and {} more...",
                "...".dimmed(),
                (same_category.len() - 8).to_string().yellow()
            );
        }
        println!();
    }

    // 2. Similar signature (same input -> output pattern)
    let similar_sig: Vec<_> = registry
        .functions()
        .filter(|f| {
            f.name != target.name
                && f.category != target.category
                && signatures_match(&parse_signature(f.signature), &target_sig)
        })
        .collect();

    if !similar_sig.is_empty() {
        println!(
            "  {} {} ({}):",
            "▸".dimmed(),
            "Similar signature".bold(),
            target.signature.white()
        );
        for func in similar_sig.iter().take(8) {
            println!(
                "    {} [{}] - {}",
                func.name.cyan(),
                func.category.name().green(),
                truncate_str(func.description, 45)
            );
        }
        if similar_sig.len() > 8 {
            println!(
                "    {} and {} more...",
                "...".dimmed(),
                (similar_sig.len() - 8).to_string().yellow()
            );
        }
        println!();
    }

    // 3. Related by description keywords
    let keywords = extract_keywords(target.description);
    let mut related: Vec<(&FunctionInfo, usize)> = registry
        .functions()
        .filter(|f| f.name != target.name && f.category != target.category)
        .filter_map(|f| {
            let score = keywords
                .iter()
                .filter(|kw| f.description.to_lowercase().contains(&kw.to_lowercase()))
                .count();
            if score > 0 { Some((f, score)) } else { None }
        })
        .collect();

    related.sort_by_key(|b| std::cmp::Reverse(b.1));

    if !related.is_empty() {
        println!("  {} {}:", "▸".dimmed(), "Related concepts".bold());
        for (func, _score) in related.iter().take(6) {
            println!(
                "    {} [{}] - {}",
                func.name.cyan(),
                func.category.name().green(),
                truncate_str(func.description, 45)
            );
        }
        if related.len() > 6 {
            println!(
                "    {} and {} more...",
                "...".dimmed(),
                (related.len() - 6).to_string().yellow()
            );
        }
        println!();
    }

    println!(
        "Use {} for details on a specific function",
        "--describe <function>".cyan()
    );

    Ok(())
}

/// Parse a signature string into input types and output type
fn parse_signature(sig: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = sig.split("->").collect();
    if parts.len() != 2 {
        return (vec![], String::new());
    }

    let inputs: Vec<String> = parts[0]
        .split(',')
        .map(|s| normalize_type(s.trim()))
        .collect();
    let output = normalize_type(parts[1].trim());

    (inputs, output)
}

/// Normalize type names for comparison
fn normalize_type(t: &str) -> String {
    // Remove optional markers and variadic indicators
    let t = t.trim_end_matches('?').trim_end_matches("...");
    // Simplify to base type
    match t {
        "number" | "integer" => "number".to_string(),
        "string" | "str" => "string".to_string(),
        "array" | "list" => "array".to_string(),
        "object" | "hash" | "map" => "object".to_string(),
        "boolean" | "bool" => "boolean".to_string(),
        "any" | "expression" | "expref" => "any".to_string(),
        _ => t.to_lowercase(),
    }
}

/// Check if two signatures are similar enough
fn signatures_match(a: &(Vec<String>, String), b: &(Vec<String>, String)) -> bool {
    // Must have same output type
    if a.1 != b.1 || a.1.is_empty() {
        return false;
    }

    // Input types should be similar (same count and compatible types)
    if a.0.len() != b.0.len() {
        return false;
    }

    // At least first input type should match
    if !a.0.is_empty() && !b.0.is_empty() && a.0[0] == b.0[0] {
        return true;
    }

    false
}

/// Extract meaningful keywords from a description
fn extract_keywords(description: &str) -> Vec<String> {
    let stopwords = [
        "a",
        "an",
        "the",
        "to",
        "of",
        "in",
        "for",
        "is",
        "are",
        "and",
        "or",
        "with",
        "from",
        "by",
        "on",
        "at",
        "as",
        "if",
        "be",
        "this",
        "that",
        "it",
        "its",
        "can",
        "will",
        "into",
        "using",
        "returns",
        "return",
        "value",
        "values",
        "given",
        "specified",
    ];

    description
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

/// Print a concise one-page quick reference of syntax and common functions.
///
/// The function count is read live from the registry so it cannot drift; the
/// featured functions are a curated sampling -- use `--list-functions` for the
/// complete set and `--describe <fn>` for details.
pub(crate) fn print_cheatsheet(registry: &FunctionRegistry) {
    let total = registry.functions().count();

    // Pad to a column width *before* coloring; padding a colored string would
    // count the ANSI escape bytes and misalign the columns.
    fn row(key: &str, desc: &str, width: usize) {
        println!("  {} {}", format!("{key:<width$}").cyan(), desc.dimmed());
    }
    fn header(title: &str) {
        println!("\n{}", title.bold().green());
    }

    println!("{}", "jpx cheatsheet".bold().cyan());
    println!(
        "{}",
        "JMESPath with extended functions -- quick reference".dimmed()
    );

    header("BASICS");
    for (k, v) in [
        (".field", "Select a field"),
        ("[0]  [-1]", "Index (negative counts from the end)"),
        ("[*]", "All array elements (projection)"),
        ("[1:5]  [::2]", "Slice / slice with step"),
        ("@", "The current element"),
        ("|", "Pipe: feed the result into the next expression"),
        (
            "&expr",
            "Expression reference (sort_by, group_by, map, ...)",
        ),
        (
            "`1`  `\"s\"`",
            "JSON literal (number, string, true/false/null)",
        ),
    ] {
        row(k, v, 14);
    }

    header("SELECT & RESHAPE");
    for (k, v) in [
        ("items[*].name", "Extract a field from each element"),
        ("items[?price > `10`]", "Filter by a condition"),
        ("items[?status=='active'].name", "Filter, then project"),
        (
            "{name: name, qty: count}",
            "Build an object (multiselect hash)",
        ),
        ("[name, count]", "Build an array (multiselect list)"),
    ] {
        row(k, v, 31);
    }

    header("COMMON PATTERNS");
    for (k, v) in [
        ("sort_by(items, &price)", "Sort ascending by a field"),
        ("reverse(sort_by(items, &price))", "Sort descending"),
        ("max_by(items, &score)", "Element with the largest field"),
        (
            "group_by(items, &category)",
            "Group into an object keyed by field",
        ),
        ("merge(a, b)", "Shallow-merge objects (later wins)"),
        ("pick(obj, ['id', 'name'])", "Keep only these keys"),
        ("omit(obj, ['secret'])", "Drop these keys"),
    ] {
        row(k, v, 33);
    }

    header("LET EXPRESSIONS (JEP-18)");
    row(
        "let $top = max(p[*].v) in p[?v == $top]",
        "Name an intermediate result",
        41,
    );

    header("A FEW FUNCTIONS BY CATEGORY");
    for (cat, fns) in [
        (
            "String",
            "split join upper lower trim replace pad_left contains",
        ),
        ("Array", "first last unique flatten chunk zip count_by"),
        ("Object", "keys values items merge pick omit map_values"),
        ("Math", "sum avg median min max round abs clamp percentile"),
        ("Date", "now format_date parse_date date_diff from_epoch"),
    ] {
        println!("  {} {}", format!("{cat:<8}").yellow(), fns);
    }

    header("DISCOVER");
    for (k, v) in [
        ("jpx --list-functions", "Every function, by category"),
        ("jpx --describe <fn>", "Signature, description, examples"),
        ("jpx --search <keyword>", "Find functions by keyword"),
        ("jpx --repl", "Interactive REPL with completion"),
    ] {
        row(k, v, 23);
    }

    println!(
        "\n{} functions available.  Docs: {}",
        total.to_string().yellow(),
        "https://joshrotenberg.github.io/jpx/".cyan()
    );
}
