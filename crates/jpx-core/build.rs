//! Build script that generates compliance tests from JSON test suites
//! and registry code + documentation from functions.toml.

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use serde_json::Value;

pub fn main() {
    let out_dir_os = env::var_os("OUT_DIR").expect("OUT_DIR not specified");
    let out_dir = out_dir_os.to_str().expect("OUT_DIR not valid UTF-8");

    // Phase 1: Generate compliance tests
    generate_compliance_tests(out_dir);

    // Phase 2: Generate registry data from functions.toml
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&manifest_dir).join("functions.toml");

    if toml_path.exists() {
        println!("cargo:rerun-if-changed=functions.toml");

        let toml_content = fs::read_to_string(&toml_path).expect("Failed to read functions.toml");
        let data: TomlData = toml::from_str(&toml_content).expect("Failed to parse functions.toml");

        let mut by_category: BTreeMap<String, Vec<&Function>> = BTreeMap::new();
        for func in &data.functions {
            by_category
                .entry(func.category.clone())
                .or_default()
                .push(func);
        }

        validate_examples(&data.functions);
        generate_registry_data(out_dir, &data.functions);
        generate_module_docs(out_dir, &by_category);
        generate_quick_reference(out_dir, &by_category);
        generate_example_test_data(out_dir, &data.functions);
    } else {
        // Generate empty registry data if functions.toml doesn't exist
        let data_path = Path::new(out_dir).join("registry_data.rs");
        fs::write(
            data_path,
            "// No functions.toml found\nuse super::{Category, Feature, FunctionInfo};\npub const FUNCTIONS: &[FunctionInfo] = &[];\n",
        )
        .expect("Failed to write empty registry_data.rs");
    }
}

// =============================================================================
// Compliance test generation
// =============================================================================

fn generate_compliance_tests(out_dir: &str) {
    let compliance_path = Path::new(out_dir).join("compliance_tests.rs");
    let mut compliance_file = File::create(&compliance_path).expect("Could not create file");
    let suites = load_test_suites();

    for (suite_num, (filename, suite)) in suites.iter().enumerate() {
        let suite_obj = suite.as_object().expect("Suite not object");
        let given = suite_obj.get("given").expect("No given value");
        let cases = suite_obj.get("cases").expect("No cases value");
        let short_filename = filename
            .replace(".json", "")
            .replace("tests/compliance/", "");
        let given_string = serde_json::to_string(given).unwrap();
        for (case_num, case) in cases
            .as_array()
            .expect("cases not array")
            .iter()
            .enumerate()
        {
            let case_obj = case.as_object().expect("case not object");
            if case_obj.get("bench").is_none() {
                generate_test(
                    &short_filename,
                    suite_num,
                    case_num,
                    case_obj,
                    &given_string,
                    &mut compliance_file,
                );
            }
        }
    }
}

fn load_test_suites() -> Vec<(String, Value)> {
    let mut result = vec![];
    let compliance_dir = "tests/compliance";
    if !Path::new(compliance_dir).exists() {
        return result;
    }
    let files = fs::read_dir(compliance_dir).expect("Invalid directory: tests/compliance");
    for filename in files {
        let path = filename.expect("Invalid file").path();
        let file_path = path.to_str().expect("Could not to_str file").to_string();
        let mut f = File::open(path).expect("Unable to open file");
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)
            .expect("Could not read JSON to string");
        let mut suite_json: Value = serde_json::from_str(&file_data).expect("invalid JSON");
        let suites = suite_json
            .as_array_mut()
            .expect("Test suite is not a JSON array");
        while let Some(suite) = suites.pop() {
            result.push((file_path.clone(), suite));
        }
    }
    result
}

fn get_expr(case: &serde_json::Map<String, Value>) -> &str {
    case.get("expression")
        .expect("No expression in case")
        .as_str()
        .expect("Could not convert case to string")
}

fn slugify(s: &str) -> String {
    let mut slug = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '-' || c == '_' {
            slug.push('_');
        }
    }
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    slug = slug.trim_matches('_').to_string();
    if slug.len() > 25 {
        slug.truncate(25);
    }
    slug
}

fn generate_fn_name(
    filename: &str,
    suite_num: usize,
    case_num: usize,
    case: &serde_json::Map<String, Value>,
) -> String {
    let expr = get_expr(case);
    let description = match case.get("comment") {
        Some(c) => c.as_str().expect("comment is not a string"),
        None => expr,
    };
    format!(
        "{}_{}_{}_{}",
        slugify(filename),
        suite_num,
        case_num,
        slugify(description)
    )
}

fn generate_test(
    filename: &str,
    suite_num: usize,
    case_num: usize,
    case: &serde_json::Map<String, Value>,
    given_string: &str,
    f: &mut File,
) {
    let fn_suffix = generate_fn_name(filename, suite_num, case_num, case);
    let case_string = serde_json::to_string(case).expect("Could not encode case");

    f.write_all(
        format!(
            r##"
#[test]
fn test_{fn_suffix}() {{
    let case: TestCase = TestCase::from_str({case_string:?}).unwrap();
    let data: serde_json::Value = serde_json::from_str({given_string:?}).unwrap();
    case.assert({filename:?}, &data).unwrap();
}}

"##
        )
        .as_bytes(),
    )
    .expect("Unable to write test");
}

// =============================================================================
// Registry data generation from functions.toml
// =============================================================================

fn generate_registry_data(out_dir: &str, functions: &[Function]) {
    let mut code = String::new();

    code.push_str("// Auto-generated from functions.toml - DO NOT EDIT\n\n");
    code.push_str("use super::{Category, Feature, FunctionInfo};\n\n");
    code.push_str("pub const FUNCTIONS: &[FunctionInfo] = &[\n");

    for func in functions {
        code.push_str("    FunctionInfo {\n");
        code.push_str(&format!("        name: \"{}\",\n", func.name));
        code.push_str(&format!(
            "        category: Category::{},\n",
            category_variant(&func.category)
        ));
        code.push_str(&format!(
            "        description: r##\"{}\"##,\n",
            func.description
        ));
        code.push_str(&format!(
            "        signature: r##\"{}\"##,\n",
            func.signature
        ));
        let examples = func.all_examples();
        let example = examples
            .first()
            .map(|e| e.code.replace("\\\"", "\""))
            .unwrap_or_default();
        code.push_str(&format!("        example: r##\"{}\"##,\n", example));
        code.push_str(&format!(
            "        is_standard: {},\n",
            func.is_standard.unwrap_or(false)
        ));

        match &func.jep {
            Some(jep) => code.push_str(&format!("        jep: Some(\"{}\"),\n", jep)),
            None => code.push_str("        jep: None,\n"),
        }

        match &func.aliases {
            Some(aliases) if !aliases.is_empty() => {
                let aliases_str: Vec<String> =
                    aliases.iter().map(|a| format!("\"{}\"", a)).collect();
                code.push_str(&format!(
                    "        aliases: &[{}],\n",
                    aliases_str.join(", ")
                ));
            }
            _ => code.push_str("        aliases: &[],\n"),
        }

        match &func.features {
            Some(features) if !features.is_empty() => {
                let features_str: Vec<String> = features
                    .iter()
                    .map(|f| format!("Feature::{}", feature_variant(f)))
                    .collect();
                code.push_str(&format!(
                    "        features: &[{}],\n",
                    features_str.join(", ")
                ));
            }
            _ => code.push_str("        features: &[],\n"),
        }

        code.push_str("    },\n");
    }

    code.push_str("];\n");

    let data_path = Path::new(out_dir).join("registry_data.rs");
    fs::write(data_path, code).expect("Failed to write registry_data.rs");
}

fn generate_module_docs(out_dir: &str, by_category: &BTreeMap<String, Vec<&Function>>) {
    let mut doc = String::new();

    doc.push_str("# Complete Function Reference\n\n");
    doc.push_str("This documentation is auto-generated from `functions.toml`.\n\n");

    doc.push_str("## Categories\n\n");
    for (category, funcs) in by_category {
        let count = funcs.len();
        let cat_title = category_title(category);
        doc.push_str(&format!(
            "- [{}](#{}) ({} functions)\n",
            cat_title,
            category.to_lowercase().replace('-', "_"),
            count
        ));
    }
    doc.push('\n');

    for (category, funcs) in by_category {
        let cat_title = category_title(category);
        doc.push_str(&format!("## {}\n\n", cat_title));
        doc.push_str("| Function | Signature | Description |\n");
        doc.push_str("|----------|-----------|-------------|\n");

        for func in funcs {
            let sig = func.signature.replace('|', "\\|");
            let desc = func
                .description
                .replace('|', "\\|")
                .replace('[', r"\[")
                .replace(']', r"\]");
            doc.push_str(&format!("| `{}` | `{}` | {} |\n", func.name, sig, desc));
        }
        doc.push('\n');

        for func in funcs {
            doc.push_str(&format!("### `{}`\n\n", func.name));
            let desc = func.description.replace('[', r"\[").replace(']', r"\]");
            doc.push_str(&format!("{}\n\n", desc));
            doc.push_str(&format!("**Signature:** `{}`\n\n", func.signature));

            if let Some(jep) = &func.jep {
                doc.push_str(&format!("**JEP:** {}\n\n", jep));
            }

            if let Some(aliases) = &func.aliases
                && !aliases.is_empty()
            {
                doc.push_str(&format!("**Aliases:** {}\n\n", aliases.join(", ")));
            }

            let examples = func.all_examples();
            if !examples.is_empty() {
                if examples.len() == 1 {
                    doc.push_str("**Example:**\n");
                } else {
                    doc.push_str("**Examples:**\n");
                }
                doc.push_str("```text\n");
                for ex in &examples {
                    if let Some(ref desc) = ex.description {
                        doc.push_str(&format!("// {}\n", desc));
                    }
                    doc.push_str(&format!("{}\n", ex.code));
                }
                doc.push_str("```\n\n");
            }
        }
    }

    let doc_path = Path::new(out_dir).join("function_docs.md");
    fs::write(doc_path, doc).expect("Failed to write function_docs.md");
}

fn generate_quick_reference(out_dir: &str, by_category: &BTreeMap<String, Vec<&Function>>) {
    let mut doc = String::new();
    let total: usize = by_category.values().map(|v| v.len()).sum();

    doc.push_str(&format!("## Quick Reference ({} functions)\n\n", total));

    for (category, funcs) in by_category {
        let cat_title = category_title(category);
        doc.push_str(&format!("### {}\n\n", cat_title));
        doc.push_str("| Function | Signature | Description |\n");
        doc.push_str("|----------|-----------|-------------|\n");

        for func in funcs {
            let sig = func.signature.replace('|', "\\|");
            let desc = func
                .description
                .replace('|', "\\|")
                .replace('[', r"\[")
                .replace(']', r"\]");
            doc.push_str(&format!("| `{}` | `{}` | {} |\n", func.name, sig, desc));
        }
        doc.push('\n');
    }

    let doc_path = Path::new(out_dir).join("quick_reference.md");
    fs::write(doc_path, doc).expect("Failed to write quick_reference.md");
}

fn generate_example_test_data(out_dir: &str, functions: &[Function]) {
    let mut code = String::new();

    code.push_str("// Auto-generated from functions.toml - DO NOT EDIT\n\n");
    code.push_str("pub struct ExampleTest {\n");
    code.push_str("    pub function_name: &'static str,\n");
    code.push_str("    pub expression: &'static str,\n");
    code.push_str("    pub expected: &'static str,\n");
    code.push_str("    pub description: Option<&'static str>,\n");
    code.push_str("}\n\n");
    code.push_str("pub const EXAMPLE_TESTS: &[ExampleTest] = &[\n");

    for func in functions {
        for ex in func.all_examples() {
            let parts: Vec<&str> = ex.code.splitn(2, " -> ").collect();
            if parts.len() != 2 {
                continue;
            }

            let expression = parts[0].trim();
            let expected = parts[1].trim();
            let description = ex.description.as_deref();

            code.push_str("    ExampleTest {\n");
            code.push_str(&format!("        function_name: {:?},\n", func.name));
            code.push_str(&format!("        expression: {:?},\n", expression));
            code.push_str(&format!("        expected: {:?},\n", expected));
            match description {
                Some(desc) => code.push_str(&format!("        description: Some({:?}),\n", desc)),
                None => code.push_str("        description: None,\n"),
            }
            code.push_str("    },\n");
        }
    }

    code.push_str("];\n");

    let test_data_path = Path::new(out_dir).join("example_tests.rs");
    fs::write(test_data_path, code).expect("Failed to write example_tests.rs");
}

// =============================================================================
// Helper functions
// =============================================================================

fn category_title(category: &str) -> String {
    match category {
        "standard" => "Standard JMESPath".to_string(),
        "multi-match" => "Multi-Match".to_string(),
        "jsonpatch" => "JSON Patch".to_string(),
        _ => {
            let mut chars = category.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

fn category_variant(category: &str) -> String {
    match category {
        "standard" => "Standard".to_string(),
        "multi-match" | "multimatch" => "MultiMatch".to_string(),
        "jsonpatch" => "Jsonpatch".to_string(),
        _ => category
            .split(['_', '-'])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect(),
    }
}

fn feature_variant(feature: &str) -> String {
    match feature {
        "spec" => "Spec".to_string(),
        "core" => "Core".to_string(),
        "fp" => "Fp".to_string(),
        "jep" => "Jep".to_string(),
        _ => feature.to_string(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct TomlData {
    functions: Vec<Function>,
}

#[derive(Debug, serde::Deserialize)]
struct Function {
    name: String,
    category: String,
    description: String,
    signature: String,
    #[serde(default)]
    example: Option<String>,
    #[serde(default)]
    examples: Option<Vec<Example>>,
    #[serde(default)]
    is_standard: Option<bool>,
    jep: Option<String>,
    aliases: Option<Vec<String>>,
    features: Option<Vec<String>>,
}

impl Function {
    fn all_examples(&self) -> Vec<Example> {
        let mut result = Vec::new();
        if let Some(ref ex) = self.example {
            result.push(Example {
                code: ex.clone(),
                description: None,
            });
        }
        if let Some(ref exs) = self.examples {
            result.extend(exs.iter().cloned());
        }
        result
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Example {
    code: String,
    #[serde(default)]
    description: Option<String>,
}

#[allow(clippy::collapsible_if)]
fn validate_examples(functions: &[Function]) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for func in functions {
        let examples = func.all_examples();

        if examples.is_empty() {
            warnings.push(format!("Function '{}' has no examples", func.name));
            continue;
        }

        for (i, ex) in examples.iter().enumerate() {
            if !ex.code.contains(" -> ") {
                errors.push(format!(
                    "Function '{}' example {} missing ' -> ' separator: {}",
                    func.name,
                    i + 1,
                    ex.code
                ));
                continue;
            }

            let expression = ex.code.split(" -> ").next().unwrap_or("");

            if !expression.contains(&func.name) {
                let has_alias = func
                    .aliases
                    .as_ref()
                    .is_some_and(|aliases| aliases.iter().any(|a| expression.contains(a)));

                if !has_alias {
                    warnings.push(format!(
                        "Function '{}' example {} may not use the function: {}",
                        func.name,
                        i + 1,
                        ex.code
                    ));
                }
            }

            let open_parens = expression.matches('(').count();
            let close_parens = expression.matches(')').count();
            if open_parens != close_parens {
                errors.push(format!(
                    "Function '{}' example {} has unbalanced parentheses: {}",
                    func.name,
                    i + 1,
                    expression
                ));
            }

            let open_brackets = expression.matches('[').count();
            let close_brackets = expression.matches(']').count();
            if open_brackets != close_brackets {
                errors.push(format!(
                    "Function '{}' example {} has unbalanced brackets: {}",
                    func.name,
                    i + 1,
                    expression
                ));
            }
        }
    }

    for warning in &warnings {
        println!("cargo:warning={}", warning);
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("ERROR: {}", error);
        }
        panic!(
            "Example validation failed with {} error(s). See messages above.",
            errors.len()
        );
    }
}
