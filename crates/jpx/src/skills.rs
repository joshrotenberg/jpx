//! Offline agent instructions, carried by the binary rather than runtime files.

use anyhow::{Result, bail};
use serde::Serialize;

const INDEX: &str = include_str!("../skills/index.md");

/// Shipped topics in index order: `(topic, installable SKILL.md)`.
const TOPICS: &[(&str, &str)] = &[
    (
        "pipelines",
        include_str!("../skills/jpx-pipelines/SKILL.md"),
    ),
    (
        "query-libraries",
        include_str!("../skills/jpx-query-libraries/SKILL.md"),
    ),
    (
        "shell-quoting",
        include_str!("../skills/jpx-shell-quoting/SKILL.md"),
    ),
];

/// Field order is fixed by these structs so the JSON output does not depend on
/// whether another crate in the build enables `serde_json/preserve_order`.
#[derive(Serialize)]
struct IndexDocument<'a> {
    topics: Vec<&'a str>,
    content: &'a str,
}

#[derive(Serialize)]
struct TopicDocument<'a> {
    topic: &'a str,
    content: &'a str,
}

fn render<T: Serialize>(document: &T, compact: bool) -> Result<String> {
    Ok(if compact {
        serde_json::to_string(document)?
    } else {
        serde_json::to_string_pretty(document)?
    })
}

pub(crate) fn print(topic: &str, json: bool, compact: bool) -> Result<()> {
    let content = if topic.is_empty() {
        INDEX
    } else {
        match TOPICS.iter().find(|(name, _)| *name == topic) {
            Some((_, content)) => content,
            None => {
                let names: Vec<&str> = TOPICS.iter().map(|(name, _)| *name).collect();
                bail!(
                    "Unknown skills topic '{topic}'. Use --skills to list available topics: {}.",
                    names.join(", ")
                )
            }
        }
    };

    if json || compact {
        let rendered = if topic.is_empty() {
            render(
                &IndexDocument {
                    topics: TOPICS.iter().map(|(name, _)| *name).collect(),
                    content,
                },
                compact,
            )?
        } else {
            render(&TopicDocument { topic, content }, compact)?
        };
        println!("{rendered}");
    } else {
        // Preserve the installable SKILL.md exactly, including its frontmatter.
        print!("{content}");
    }
    Ok(())
}
