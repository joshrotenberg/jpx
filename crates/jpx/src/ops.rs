use crate::args::ColorMode;
use crate::input::{self, read_json_from};
use crate::output::output_json;
use anyhow::{Context, Result};
use colored::Colorize;
use jpx_engine::Runtime;

/// Generate JSON Patch (RFC 6902) from two files
pub(crate) fn diff_files(
    source_path: &str,
    target_path: &str,
    compact: bool,
    color_mode: &ColorMode,
    sort_keys: bool,
) -> Result<()> {
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(source_path)
            .map_err(|e| input::file_read_error(source_path, &e))?,
    )
    .context("Failed to parse source JSON")?;

    let target: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(target_path)
            .map_err(|e| input::file_read_error(target_path, &e))?,
    )
    .context("Failed to parse target JSON")?;

    let patch = json_patch::diff(&source, &target);

    if patch.0.is_empty() {
        eprintln!("{} No differences found", "✓".green());
        println!("[]");
        return Ok(());
    }

    eprintln!(
        "{} Generated {} patch operation(s)",
        "✓".green(),
        patch.0.len().to_string().yellow()
    );

    let patch_value = serde_json::to_value(&patch)?;
    output_json(&patch_value, compact, color_mode, sort_keys)
}

/// List queries in a .jpx query library file
pub(crate) fn list_queries(query_path: &str, color_mode: &ColorMode) -> Result<()> {
    let (file_path, _) = jpx::query_library::parse_query_path(query_path);
    let content =
        std::fs::read_to_string(file_path).map_err(|e| input::file_read_error(file_path, &e))?;

    let library = jpx::query_library::QueryLibrary::parse(&content)
        .with_context(|| format!("Failed to parse query library: {}", file_path))?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    println!("Queries in {}:\n", file_path);

    // Calculate column widths
    let max_name_len = library
        .list()
        .iter()
        .map(|q| q.name.len())
        .max()
        .unwrap_or(4);
    let name_width = max_name_len.max(4);

    // Header
    if use_color {
        println!(
            "  {:<name_width$}  {}",
            "NAME".bold(),
            "DESCRIPTION".bold(),
            name_width = name_width
        );
    } else {
        println!(
            "  {:<name_width$}  DESCRIPTION",
            "NAME",
            name_width = name_width
        );
    }
    println!(
        "  {:-<name_width$}  {:-<40}",
        "",
        "",
        name_width = name_width
    );

    // Queries
    for query in library.list() {
        let desc = query.description.as_deref().unwrap_or("-");
        if use_color {
            println!(
                "  {:<name_width$}  {}",
                query.name.cyan(),
                desc,
                name_width = name_width
            );
        } else {
            println!(
                "  {:<name_width$}  {}",
                query.name,
                desc,
                name_width = name_width
            );
        }
    }

    println!("\nUse: jpx -Q {}:<query-name> <input>", file_path);

    Ok(())
}

/// Validate queries in a .jpx file without running them
pub(crate) fn check_queries(
    query_path: &str,
    color_mode: &ColorMode,
    runtime: &Runtime,
) -> Result<()> {
    let (file_path, _) = jpx::query_library::parse_query_path(query_path);
    let content =
        std::fs::read_to_string(file_path).map_err(|e| input::file_read_error(file_path, &e))?;

    let library = jpx::query_library::QueryLibrary::parse(&content)
        .with_context(|| format!("Failed to parse query library: {}", file_path))?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    let mut has_errors = false;

    println!("Validating {}...\n", file_path);

    for query in library.list() {
        // Use runtime.compile() to validate function names too
        match runtime.compile(&query.expression) {
            Ok(_) => {
                if use_color {
                    println!("  {} {}", "✓".green(), query.name);
                } else {
                    println!("  [OK] {}", query.name);
                }
            }
            Err(e) => {
                has_errors = true;
                if use_color {
                    println!(
                        "  {} {} (line {})",
                        "✗".red(),
                        query.name.red(),
                        query.line_number
                    );
                    println!("    {}", e.to_string().red());
                } else {
                    println!("  [ERROR] {} (line {})", query.name, query.line_number);
                    println!("    {}", e);
                }
            }
        }
    }

    println!();

    if has_errors {
        if use_color {
            println!("{}", "Validation failed.".red().bold());
        } else {
            println!("Validation failed.");
        }
        std::process::exit(1);
    } else if use_color {
        println!("{}", "All queries valid.".green().bold());
    } else {
        println!("All queries valid.");
    }

    Ok(())
}

/// Apply JSON Patch (RFC 6902) to a document
pub(crate) fn apply_patch(
    doc_path: &Option<String>,
    patch_path: &str,
    compact: bool,
    color_mode: &ColorMode,
    sort_keys: bool,
) -> Result<()> {
    // Read the document (from -f or stdin)
    let mut doc = read_json_from(doc_path.as_deref())?;

    // Read the patch from file
    let patch_content =
        std::fs::read_to_string(patch_path).map_err(|e| input::file_read_error(patch_path, &e))?;
    let patch_value: serde_json::Value =
        serde_json::from_str(&patch_content).context("Failed to parse patch JSON")?;

    // Convert to json_patch::Patch
    let patch: json_patch::Patch =
        serde_json::from_value(patch_value).context("Invalid JSON Patch format")?;

    // Apply the patch
    json_patch::patch(&mut doc, &patch).context("Failed to apply patch")?;

    eprintln!(
        "{} Applied {} patch operation(s)",
        "✓".green(),
        patch.0.len().to_string().yellow()
    );

    output_json(&doc, compact, color_mode, sort_keys)
}

/// Apply JSON Merge Patch (RFC 7396) to a document
pub(crate) fn apply_merge(
    doc_path: &Option<String>,
    merge_path: &str,
    compact: bool,
    color_mode: &ColorMode,
    sort_keys: bool,
) -> Result<()> {
    // Read the document (from -f or stdin)
    let mut doc = read_json_from(doc_path.as_deref())?;

    // Read the merge patch from file
    let merge_content =
        std::fs::read_to_string(merge_path).map_err(|e| input::file_read_error(merge_path, &e))?;
    let merge_patch: serde_json::Value =
        serde_json::from_str(&merge_content).context("Failed to parse merge patch JSON")?;

    // Apply the merge patch
    json_patch::merge(&mut doc, &merge_patch);

    eprintln!("{} Applied merge patch", "✓".green());

    output_json(&doc, compact, color_mode, sort_keys)
}
