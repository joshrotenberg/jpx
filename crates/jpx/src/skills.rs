//! Offline agent instructions, carried by the binary rather than runtime files.

use anyhow::{Result, bail};
use serde::Serialize;

const INDEX: &str = include_str!("../skills/index.md");
const QUERY_LIBRARIES: &str = include_str!("../skills/jpx-query-libraries/SKILL.md");

/// Field order is fixed by these structs so the JSON output does not depend on
/// whether another crate in the build enables `serde_json/preserve_order`.
#[derive(Serialize)]
struct IndexDocument<'a> {
    topics: [&'a str; 1],
    content: &'a str,
}

#[derive(Serialize)]
struct TopicDocument<'a> {
    topic: &'a str,
    content: &'a str,
}

pub(crate) fn print(topic: &str, json: bool, compact: bool) -> Result<()> {
    let content = match topic {
        "" => INDEX,
        "query-libraries" => QUERY_LIBRARIES,
        _ => bail!(
            "Unknown skills topic '{topic}'. Use --skills to list available topics, or --skills query-libraries."
        ),
    };

    if json || compact {
        let rendered = match (topic.is_empty(), compact) {
            (true, true) => serde_json::to_string(&IndexDocument {
                topics: ["query-libraries"],
                content,
            })?,
            (true, false) => serde_json::to_string_pretty(&IndexDocument {
                topics: ["query-libraries"],
                content,
            })?,
            (false, true) => serde_json::to_string(&TopicDocument { topic, content })?,
            (false, false) => serde_json::to_string_pretty(&TopicDocument { topic, content })?,
        };
        println!("{rendered}");
    } else {
        // Preserve the installable SKILL.md exactly, including its frontmatter.
        print!("{content}");
    }
    Ok(())
}
