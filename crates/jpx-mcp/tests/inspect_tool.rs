//! Integration tests for the bounded `inspect` MCP tool.

use jpx_mcp::build_router;
use serde_json::{Map, Value, json};
use tower_mcp::TestClient;

const SECRET: &str = "SECRET-LEAF-VALUE-31d7";

async fn create_client() -> TestClient {
    let router = build_router(false).expect("Failed to build router");
    let mut client = TestClient::from_router(router);
    client.initialize().await;
    client
}

/// Calls `inspect` and returns (serialized CallToolResult size, parsed report).
async fn inspect(client: &mut TestClient, args: Value) -> (usize, Value) {
    let raw = client.call_tool_raw("inspect", args).await;
    let size = serde_json::to_vec(&raw).unwrap().len();
    assert_ne!(raw.get("isError"), Some(&json!(true)), "{raw}");
    let text = raw["content"][0]["text"].as_str().expect("text content");
    assert!(!text.contains(SECRET), "leaf value leaked: {text}");
    (size, serde_json::from_str(text).expect("report is JSON"))
}

async fn inspect_error(client: &mut TestClient, args: Value) -> String {
    let result = client.call_tool("inspect", args).await;
    assert!(result.is_error);
    let text = result.all_text();
    assert!(!text.contains(SECRET), "leaf value leaked: {text}");
    text
}

fn pointers(report: &Value) -> Vec<&str> {
    report["paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["pointer"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn inspect_is_discoverable_with_accurate_schema() {
    let mut client = create_client().await;
    let tools = client.list_tools().await;
    let tool = tools
        .iter()
        .find(|t| t["name"] == "inspect")
        .expect("inspect tool listed");
    assert_eq!(tool["annotations"]["readOnlyHint"], json!(true));
    let props = &tool["inputSchema"]["properties"];
    for (name, min, max, default) in [
        ("max_depth", 0, 32, 3),
        ("max_paths", 1, 1000, 100),
        ("array_sample", 0, 100, 3),
        ("max_bytes", 512, 65536, 16384),
    ] {
        let p = &props[name];
        assert_eq!(p["type"], "integer", "{name}: {p}");
        assert_eq!(p["minimum"], min, "{name}: {p}");
        assert_eq!(p["maximum"], max, "{name}: {p}");
        assert_eq!(p["default"], default, "{name}: {p}");
    }
    assert_eq!(props["input"]["type"], "string");
    assert_eq!(tool["inputSchema"]["required"], json!(["input"]));
}

#[tokio::test]
async fn inspect_defaults_and_report_shape() {
    let mut client = create_client().await;
    let input = json!({"users": [{"name": SECRET, "tags": [1, 2, 3, 4]}]}).to_string();
    let (_, report) = inspect(&mut client, json!({ "input": input })).await;
    assert_eq!(
        report["limits"],
        json!({"max_depth": 3, "max_paths": 100, "array_sample": 3, "max_bytes": 16384})
    );
    assert_eq!(
        pointers(&report),
        ["", "/users", "/users/0", "/users/0/name", "/users/0/tags"]
    );
    assert_eq!(report["complete"], false);
    assert_eq!(report["returned_paths"], 5);
    assert_eq!(
        report["omitted"],
        json!({"depth": true, "paths": false, "array_sample": false, "bytes": false})
    );
    assert_eq!(
        report["paths"][4],
        json!({"pointer": "/users/0/tags", "type": "array", "depth": 3, "length": 4, "truncated": "depth"})
    );
    assert_eq!(
        report["paths"][3],
        json!({"pointer": "/users/0/name", "type": "string", "depth": 3})
    );
}

/// Mirrors the example in docs/src/mcp/tools.md.
#[tokio::test]
async fn inspect_documented_example() {
    let mut client = create_client().await;
    let result = client
        .call_tool(
            "inspect",
            json!({
                "input": r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#,
                "array_sample": 1
            }),
        )
        .await;
    assert!(!result.is_error);
    assert_eq!(
        result.first_text(),
        Some(concat!(
            r#"{"complete":false,"returned_paths":4,"#,
            r#""omitted":{"depth":false,"paths":false,"array_sample":true,"bytes":false},"#,
            r#""limits":{"max_depth":3,"max_paths":100,"array_sample":1,"max_bytes":16384},"#,
            r#""paths":[{"pointer":"","type":"object","depth":0,"length":1},"#,
            r#"{"pointer":"/users","type":"array","depth":1,"length":2,"truncated":"array_sample"},"#,
            r#"{"pointer":"/users/0","type":"object","depth":2,"length":1},"#,
            r#"{"pointer":"/users/0/name","type":"string","depth":3}]}"#
        ))
    );
}

#[tokio::test]
async fn inspect_scalars_empty_containers_and_awkward_keys() {
    let mut client = create_client().await;
    for (input, ty) in [
        (json!(SECRET), "string"),
        (json!(1), "number"),
        (json!(null), "null"),
        (json!({}), "object"),
        (json!([]), "array"),
    ] {
        let args =
            json!({"input": input.to_string(), "max_depth": 0, "array_sample": 0, "max_paths": 1});
        let (_, report) = inspect(&mut client, args).await;
        assert_eq!(report["complete"], true, "{input}");
        assert_eq!(report["paths"][0]["pointer"], "");
        assert_eq!(report["paths"][0]["type"], ty);
        assert_eq!(report["paths"][0]["depth"], 0);
    }

    let input = json!({"": 1, "a/b": {"~": 2}}).to_string();
    let (_, report) = inspect(&mut client, json!({ "input": input })).await;
    assert_eq!(report["complete"], true);
    assert_eq!(pointers(&report), ["", "/", "/a~1b", "/a~1b/~0"]);
}

#[tokio::test]
async fn inspect_depth_sample_and_path_omissions() {
    let mut client = create_client().await;
    let input = json!({"a": [SECRET, SECRET], "b": {"c": SECRET}}).to_string();

    let (_, report) = inspect(&mut client, json!({"input": input, "max_depth": 0})).await;
    assert_eq!(pointers(&report), [""]);
    assert_eq!(report["complete"], false);
    assert_eq!(report["omitted"]["depth"], true);

    let (_, report) = inspect(&mut client, json!({"input": input, "array_sample": 0})).await;
    assert_eq!(pointers(&report), ["", "/a", "/b", "/b/c"]);
    assert_eq!(
        report["omitted"],
        json!({"depth": false, "paths": false, "array_sample": true, "bytes": false})
    );

    // Exactly as many paths as nodes: complete, not a paths omission.
    let (_, report) = inspect(&mut client, json!({"input": input, "max_paths": 6})).await;
    assert_eq!(report["complete"], true);
    assert_eq!(report["returned_paths"], 6);

    let (_, report) = inspect(&mut client, json!({"input": input, "max_paths": 5})).await;
    assert_eq!(report["complete"], false);
    assert_eq!(report["returned_paths"], 5);
    assert_eq!(
        report["omitted"],
        json!({"depth": false, "paths": true, "array_sample": false, "bytes": false})
    );

    // Repeat calls are byte-identical.
    let first = client
        .call_tool_raw("inspect", json!({ "input": input }))
        .await;
    let second = client
        .call_tool_raw("inspect", json!({ "input": input }))
        .await;
    assert_eq!(first, second);
}

#[tokio::test]
async fn inspect_rejects_invalid_limits_with_guidance() {
    let mut client = create_client().await;
    for (field, value, expected) in [
        (
            "max_depth",
            json!(-1),
            "max_depth must be a whole number from 0 to 32 (default 3); got -1",
        ),
        ("max_depth", json!(33), "got 33"),
        ("max_depth", json!("deep"), "got a string"),
        (
            "max_paths",
            json!(0),
            "max_paths must be a whole number from 1 to 1000 (default 100); got 0",
        ),
        ("max_paths", json!(1001), "got 1001"),
        ("max_paths", json!(2.5), "got a non-integer number"),
        (
            "array_sample",
            json!(-1),
            "array_sample must be a whole number from 0 to 100 (default 3); got -1",
        ),
        ("array_sample", json!(101), "got 101"),
        (
            "max_bytes",
            json!(511),
            "max_bytes must be a whole number from 512 to 65536 (default 16384); got 511",
        ),
        ("max_bytes", json!(65537), "got 65537"),
        ("max_bytes", json!({"n": 1}), "got an object"),
        // Explicit null is not the same as omitting the field.
        (
            "max_depth",
            Value::Null,
            "max_depth must be a whole number from 0 to 32 (default 3); got null",
        ),
        (
            "max_paths",
            Value::Null,
            "max_paths must be a whole number from 1 to 1000 (default 100); got null",
        ),
        (
            "array_sample",
            Value::Null,
            "array_sample must be a whole number from 0 to 100 (default 3); got null",
        ),
        (
            "max_bytes",
            Value::Null,
            "max_bytes must be a whole number from 512 to 65536 (default 16384); got null",
        ),
    ] {
        let mut args = Map::new();
        args.insert("input".into(), json!(json!({"k": SECRET}).to_string()));
        args.insert(field.into(), value);
        let text = inspect_error(&mut client, Value::Object(args)).await;
        assert!(text.contains(expected), "{field}: {text}");
        assert!(
            text.contains(&format!("or omit {field} to use the default")),
            "{text}"
        );
    }

    let text = inspect_error(
        &mut client,
        json!({"input": format!("{{\"k\": \"{SECRET}\"")}),
    )
    .await;
    assert!(text.contains("Invalid JSON"), "{text}");
    assert!(text.contains("line 1 column"), "{text}");
    assert!(
        text.contains("Pass the document as valid JSON text"),
        "{text}"
    );
}

#[tokio::test]
async fn inspect_wide_object_with_one_path() {
    let mut client = create_client().await;
    let mut map = Map::new();
    for i in 0..50_000 {
        map.insert(format!("k{i}"), json!(SECRET));
    }
    let input = Value::Object(map).to_string();
    let (_, report) = inspect(&mut client, json!({"input": input, "max_paths": 1})).await;
    assert_eq!(pointers(&report), [""]);
    assert_eq!(report["paths"][0]["length"], 50_000);
    assert_eq!(report["complete"], false);
    assert_eq!(
        report["omitted"],
        json!({"depth": false, "paths": true, "array_sample": false, "bytes": false})
    );

    // The server parses with insertion order, so members follow the document.
    let (_, report) = inspect(&mut client, json!({"input": input, "max_paths": 3})).await;
    assert_eq!(pointers(&report), ["", "/k0", "/k1"]);
}

/// A document far larger than any budget, with escape-heavy and Unicode keys.
fn large_awkward_input() -> String {
    let mut map = Map::new();
    for i in 0..3000 {
        map.insert(
            format!("\"k\\{i:04}\" é日🙂/~\n"),
            json!([SECRET, {"x": SECRET}]),
        );
    }
    map.insert("z".repeat(70_000), json!(SECRET));
    Value::Object(map).to_string()
}

#[tokio::test]
async fn inspect_bounds_serialized_call_tool_result() {
    let mut client = create_client().await;
    let input = large_awkward_input();
    for max_bytes in [512, 16384, 65536] {
        let args = json!({
            "input": input,
            "max_depth": 32,
            "max_paths": 1000,
            "array_sample": 100,
            "max_bytes": max_bytes
        });
        let (size, report) = inspect(&mut client, args).await;
        assert!(size <= max_bytes, "{size} > {max_bytes}");
        // Budgets are used, not merely approximated by a cap on the inner JSON.
        assert!(size + 200 > max_bytes, "{size} far below {max_bytes}");
        assert_eq!(report["omitted"]["bytes"], true);
        assert_eq!(report["complete"], false);
        let n = report["paths"].as_array().unwrap().len();
        assert_eq!(report["returned_paths"], n);
        assert!(n >= 1);
        assert_eq!(report["paths"][0]["length"], 3001);
    }

    let (size, _) = inspect(&mut client, json!({ "input": input })).await;
    assert!(size <= 16384);
}

#[tokio::test]
async fn inspect_byte_budget_is_exact_for_complete_previews() {
    let mut client = create_client().await;
    let input = json!({"\"q\"": [SECRET, {"\\": null}], "é/~": {"\n": 1}, "日本": 2}).to_string();
    let mut boundary = None;
    for max_bytes in 512..=4096 {
        let args = json!({"input": input, "max_bytes": max_bytes});
        let (size, report) = inspect(&mut client, args).await;
        assert!(size <= max_bytes);
        if report["complete"] == true {
            boundary = Some((max_bytes, size));
            break;
        }
        assert_eq!(report["omitted"]["bytes"], true);
    }
    let (max_bytes, size) = boundary.expect("complete preview within 4096 bytes");
    assert!(max_bytes > 512, "fixture too small to probe the boundary");
    assert_eq!(
        size, max_bytes,
        "complete preview should fill its exact budget"
    );
}

#[tokio::test]
async fn existing_paths_and_keys_handlers_are_unchanged() {
    let mut client = create_client().await;
    let input = json!({"a": {"b": 1}}).to_string();
    let paths = client
        .call_tool_json("paths", json!({"input": input, "include_types": false}))
        .await;
    assert_eq!(
        paths,
        json!([{"path": "@"}, {"path": "a"}, {"path": "a.b"}])
    );
    let keys = client
        .call_tool_json("keys", json!({"input": input, "recursive": true}))
        .await;
    assert_eq!(keys, json!(["a", "a.b"]));
}

#[tokio::test]
async fn inspect_reports_parse_nesting_limit() {
    let mut client = create_client().await;
    let deep = |n: usize| format!("{}{}", "[".repeat(n), "]".repeat(n));

    let (_, report) = inspect(&mut client, json!({ "input": deep(127), "max_depth": 32 })).await;
    assert_eq!(report["omitted"]["depth"], json!(true), "{report}");
    assert_eq!(report["paths"].as_array().unwrap().len(), 33);

    let text = inspect_error(&mut client, json!({ "input": deep(200) })).await;
    assert!(
        text.contains("exceeds the 128-level nesting limit for parsing"),
        "{text}"
    );
    assert!(
        !text.contains("Pass the document as valid JSON text"),
        "{text}"
    );
}
