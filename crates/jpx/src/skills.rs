//! Offline agent instructions, carried by the binary rather than runtime files.

use anyhow::{Result, bail};
use serde_json::json;

const INDEX: &str = include_str!("../skills/index.md");
const QUERY_LIBRARIES: &str = include_str!("../skills/jpx-query-libraries/SKILL.md");

pub(crate) fn print(topic: &str, json: bool, compact: bool) -> Result<()> {
    let content = match topic {
        "" => INDEX,
        "query-libraries" => QUERY_LIBRARIES,
        _ => bail!(
            "Unknown skills topic '{topic}'. Use --skills to list available topics, or --skills query-libraries."
        ),
    };

    if json || compact {
        let document = if topic.is_empty() {
            json!({ "topics": ["query-libraries"], "content": content })
        } else {
            json!({ "topic": topic, "content": content })
        };
        if compact {
            println!("{}", serde_json::to_string(&document)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&document)?);
        }
    } else {
        // Preserve the installable SKILL.md exactly, including its frontmatter.
        print!("{content}");
    }
    Ok(())
}
