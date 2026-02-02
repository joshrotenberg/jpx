//! Build script that generates demo data from demos.toml

use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct DemosFile {
    demo: Vec<Demo>,
}

#[derive(Debug, Deserialize)]
struct Demo {
    name: String,
    description: String,
    root_key: String,
    data: String,
    queries: Vec<Query>,
}

#[derive(Debug, Deserialize)]
struct Query {
    expression: String,
    description: String,
    difficulty: u8,
}

fn main() {
    println!("cargo:rerun-if-changed=demos.toml");

    let demos_toml = fs::read_to_string("demos.toml").expect("Failed to read demos.toml");
    let demos_file: DemosFile = toml::from_str(&demos_toml).expect("Failed to parse demos.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("demos_generated.rs");

    let mut output = String::new();

    // Generate the Demo struct and DEMOS constant
    output.push_str(
        r#"/// Query complexity level (1-5)
pub type QueryLevel = u8;

/// Query example tuple: (expression, description, difficulty)
pub type QueryExample = (&'static str, &'static str, QueryLevel);

/// Demo dataset with sample data and example queries
pub struct Demo {
    pub name: &'static str,
    pub description: &'static str,
    pub data: &'static str,
    pub root_key: &'static str,
    pub queries: &'static [QueryExample],
}

pub const DEMOS: &[Demo] = &[
"#,
    );

    for demo in &demos_file.demo {
        output.push_str("    Demo {\n");
        output.push_str(&format!("        name: {:?},\n", demo.name));
        output.push_str(&format!("        description: {:?},\n", demo.description));
        output.push_str(&format!("        root_key: {:?},\n", demo.root_key));

        // Data needs to be a raw string literal to preserve JSON formatting
        output.push_str(&format!("        data: r#\"{}\"#,\n", demo.data.trim()));

        output.push_str("        queries: &[\n");
        for query in &demo.queries {
            output.push_str(&format!(
                "            ({:?}, {:?}, {}),\n",
                query.expression, query.description, query.difficulty
            ));
        }
        output.push_str("        ],\n");
        output.push_str("    },\n");
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write generated demos");
}
