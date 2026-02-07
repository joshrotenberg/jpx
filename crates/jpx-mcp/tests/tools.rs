//! Integration tests for jpx-mcp tools using tower-mcp's TestClient.

use jpx_mcp::build_router;
use serde_json::json;
use tower_mcp::TestClient;

// =============================================================================
// Test helpers
// =============================================================================

async fn create_client() -> TestClient {
    let router = build_router(false).expect("Failed to build router");
    let mut client = TestClient::from_router(router);
    client.initialize().await;
    client
}

async fn create_strict_client() -> TestClient {
    let router = build_router(true).expect("Failed to build router");
    let mut client = TestClient::from_router(router);
    client.initialize().await;
    client
}

// =============================================================================
// Server initialization tests
// =============================================================================

#[tokio::test]
async fn test_initialize() {
    let router = build_router(false).expect("Failed to build router");
    let mut client = TestClient::from_router(router);

    let init = client.initialize().await;

    assert!(init.get("protocolVersion").is_some());
    assert_eq!(
        init.get("serverInfo")
            .and_then(|s| s.get("name"))
            .and_then(|n| n.as_str()),
        Some("jpx-mcp")
    );
}

#[tokio::test]
async fn test_list_tools() {
    let mut client = create_client().await;

    let tools = client.list_tools().await;

    // Should have 32 tools
    assert_eq!(tools.len(), 32, "Expected 32 tools, got {}", tools.len());

    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    // Core evaluation tools
    assert!(tool_names.contains(&"evaluate"));
    assert!(tool_names.contains(&"batch_evaluate"));
    assert!(tool_names.contains(&"validate"));
    assert!(tool_names.contains(&"evaluate_file"));

    // Introspection tools
    assert!(tool_names.contains(&"functions"));
    assert!(tool_names.contains(&"describe"));
    assert!(tool_names.contains(&"categories"));
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"similar"));

    // JSON utilities
    assert!(tool_names.contains(&"format"));
    assert!(tool_names.contains(&"diff"));
    assert!(tool_names.contains(&"patch"));
    assert!(tool_names.contains(&"merge"));
    assert!(tool_names.contains(&"keys"));
    assert!(tool_names.contains(&"paths"));
    assert!(tool_names.contains(&"stats"));

    // Query store
    assert!(tool_names.contains(&"define_query"));
    assert!(tool_names.contains(&"get_query"));
    assert!(tool_names.contains(&"delete_query"));
    assert!(tool_names.contains(&"list_queries"));
    assert!(tool_names.contains(&"run_query"));

    // Discovery
    assert!(tool_names.contains(&"register_discovery"));
    assert!(tool_names.contains(&"query_tools"));
    assert!(tool_names.contains(&"similar_tools"));
    assert!(tool_names.contains(&"unregister_discovery"));
    assert!(tool_names.contains(&"list_discovery_servers"));
    assert!(tool_names.contains(&"list_discovery_categories"));
    assert!(tool_names.contains(&"inspect_discovery_index"));
    assert!(tool_names.contains(&"get_discovery_schema"));
    assert!(tool_names.contains(&"register_tools_simple"));

    // Engine info
    assert!(tool_names.contains(&"engine_info"));
}

// =============================================================================
// Evaluation tools
// =============================================================================

#[tokio::test]
async fn test_evaluate_simple() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "evaluate",
            json!({
                "input": r#"{"name": "alice", "age": 30}"#,
                "expression": "name"
            }),
        )
        .await;

    assert!(!result.is_error);
    assert_eq!(result.first_text(), Some("\"alice\""));
}

#[tokio::test]
async fn test_evaluate_array_projection() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "evaluate",
            json!({
                "input": r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#,
                "expression": "users[*].name"
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("alice"));
    assert!(text.contains("bob"));
}

#[tokio::test]
async fn test_evaluate_with_extension_function() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "evaluate",
            json!({
                "input": r#"{"name": "alice"}"#,
                "expression": "upper(name)"
            }),
        )
        .await;

    assert!(!result.is_error);
    assert_eq!(result.first_text(), Some("\"ALICE\""));
}

#[tokio::test]
async fn test_evaluate_invalid_expression() {
    let mut client = create_client().await;

    let result = client
        .call_tool_expect_error(
            "evaluate",
            json!({
                "input": r#"{"name": "alice"}"#,
                "expression": "users[*.name"  // Invalid - unclosed bracket
            }),
        )
        .await;

    assert!(result.get("code").is_some() || result.get("isError").is_some());
}

#[tokio::test]
async fn test_batch_evaluate() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "batch_evaluate",
            json!({
                "input": r#"{"a": 1, "b": 2, "c": 3}"#,
                "expressions": ["a", "b", "c", "d"]
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    // Should contain results for all expressions
    assert!(text.contains("\"result\""));
}

#[tokio::test]
async fn test_validate_valid_expression() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "validate",
            json!({
                "expression": "users[*].name | sort(@)"
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("\"valid\": true") || text.contains("\"valid\":true"));
}

#[tokio::test]
async fn test_validate_invalid_expression() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "validate",
            json!({
                "expression": "users[*.name"  // Invalid
            }),
        )
        .await;

    assert!(!result.is_error); // validate returns a result, not an error
    let text = result.first_text().unwrap();
    assert!(text.contains("\"valid\": false") || text.contains("\"valid\":false"));
}

// =============================================================================
// Introspection tools
// =============================================================================

#[tokio::test]
async fn test_categories() {
    let mut client = create_client().await;

    let result = client.call_tool("categories", json!({})).await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("string") || text.contains("String"));
    assert!(text.contains("math") || text.contains("Math"));
    assert!(text.contains("array") || text.contains("Array"));
}

#[tokio::test]
async fn test_functions_all() {
    let mut client = create_client().await;

    let result = client.call_tool("functions", json!({})).await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    // Should have many functions
    assert!(text.contains("upper"));
    assert!(text.contains("length"));
}

#[tokio::test]
async fn test_functions_filtered_by_category() {
    let mut client = create_client().await;

    let result = client
        .call_tool("functions", json!({"category": "String"}))
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("upper"));
    assert!(text.contains("lower"));
}

#[tokio::test]
async fn test_describe_function() {
    let mut client = create_client().await;

    let result = client.call_tool("describe", json!({"name": "upper"})).await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("upper"));
    assert!(text.contains("String"));
}

#[tokio::test]
async fn test_describe_unknown_function() {
    let mut client = create_client().await;

    let result = client
        .call_tool("describe", json!({"name": "nonexistent_function"}))
        .await;

    // Returns error result, not JSON-RPC error
    assert!(result.is_error);
}

#[tokio::test]
async fn test_search_functions() {
    let mut client = create_client().await;

    let result = client
        .call_tool("search", json!({"query": "hash", "limit": 10}))
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    // Should find hash-related functions
    assert!(text.contains("md5") || text.contains("sha"));
}

#[tokio::test]
async fn test_similar_functions() {
    let mut client = create_client().await;

    let result = client
        .call_tool("similar", json!({"function": "upper"}))
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    // Should find similar string functions
    assert!(text.contains("same_category") || text.contains("lower"));
}

// =============================================================================
// JSON utilities
// =============================================================================

#[tokio::test]
async fn test_format_json() {
    let mut client = create_client().await;

    let result = client
        .call_tool("format", json!({"input": r#"{"a":1,"b":2}"#, "indent": 2}))
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains('\n')); // Should be formatted
}

#[tokio::test]
async fn test_format_compact() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "format",
            json!({"input": r#"{"a": 1, "b": 2}"#, "indent": 0}),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(!text.contains('\n')); // Should be compact
}

#[tokio::test]
async fn test_diff() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "diff",
            json!({
                "source": r#"{"a": 1}"#,
                "target": r#"{"a": 2}"#
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("replace") || text.contains("op"));
}

#[tokio::test]
async fn test_patch() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "patch",
            json!({
                "input": r#"{"a": 1}"#,
                "patch": r#"[{"op": "replace", "path": "/a", "value": 2}]"#
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("2"));
}

#[tokio::test]
async fn test_merge() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "merge",
            json!({
                "input": r#"{"a": 1, "b": 2}"#,
                "patch": r#"{"b": 3, "c": 4}"#
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("\"a\""));
    assert!(text.contains("\"b\": 3") || text.contains("\"b\":3"));
    assert!(text.contains("\"c\""));
}

#[tokio::test]
async fn test_keys_top_level() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "keys",
            json!({
                "input": r#"{"a": 1, "b": {"c": 2}}"#,
                "recursive": false
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("a"));
    assert!(text.contains("b"));
}

#[tokio::test]
async fn test_keys_recursive() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "keys",
            json!({
                "input": r#"{"a": 1, "b": {"c": 2}}"#,
                "recursive": true
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("b.c"));
}

#[tokio::test]
async fn test_paths() {
    let mut client = create_client().await;

    let result = client
        .call_tool(
            "paths",
            json!({
                "input": r#"{"user": {"name": "alice"}}"#,
                "include_types": true
            }),
        )
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("user.name"));
    assert!(text.contains("string"));
}

#[tokio::test]
async fn test_stats() {
    let mut client = create_client().await;

    let result = client
        .call_tool("stats", json!({"input": r#"[1, 2, 3, 4, 5]"#}))
        .await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("array"));
    assert!(text.contains("5") || text.contains("length"));
}

// =============================================================================
// Query store
// =============================================================================

#[tokio::test]
async fn test_query_store_lifecycle() {
    let mut client = create_client().await;

    // Define a query
    let result = client
        .call_tool(
            "define_query",
            json!({
                "name": "count_items",
                "expression": "length(@)",
                "description": "Count items in array"
            }),
        )
        .await;
    assert!(!result.is_error);

    // Get the query
    let result = client
        .call_tool("get_query", json!({"name": "count_items"}))
        .await;
    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("length(@)"));

    // List queries
    let result = client.call_tool("list_queries", json!({})).await;
    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("count_items"));

    // Run the query
    let result = client
        .call_tool(
            "run_query",
            json!({
                "name": "count_items",
                "input": "[1, 2, 3, 4, 5]"
            }),
        )
        .await;
    assert!(!result.is_error);
    assert_eq!(result.first_text(), Some("5"));

    // Delete the query
    let result = client
        .call_tool("delete_query", json!({"name": "count_items"}))
        .await;
    assert!(!result.is_error);

    // Verify it's gone
    let result = client
        .call_tool("get_query", json!({"name": "count_items"}))
        .await;
    assert!(result.is_error);
}

#[tokio::test]
async fn test_define_invalid_query() {
    let mut client = create_client().await;

    // Invalid expression should cause an error (either JSON-RPC error or tool error)
    let result = client
        .call_tool_expect_error(
            "define_query",
            json!({
                "name": "bad_query",
                "expression": "users[*.name"  // Invalid
            }),
        )
        .await;

    // call_tool_expect_error returns if isError is true OR if there's a JSON-RPC error
    // Either way proves the error was handled
    assert!(
        result.get("code").is_some()
            || result.get("message").is_some()
            || result.get("content").is_some()
    );
}

// =============================================================================
// Discovery
// =============================================================================

#[tokio::test]
async fn test_discovery_lifecycle() {
    let mut client = create_client().await;

    // Register a server
    let result = client
        .call_tool(
            "register_tools_simple",
            json!({
                "server_name": "test-server",
                "version": "1.0.0",
                "tools": [
                    {"name": "test_tool", "description": "A test tool", "tags": ["test"]}
                ]
            }),
        )
        .await;
    assert!(!result.is_error);

    // List servers
    let result = client.call_tool("list_discovery_servers", json!({})).await;
    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("test-server"));

    // Query tools
    let result = client
        .call_tool("query_tools", json!({"query": "test", "top_k": 10}))
        .await;
    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("test_tool"));

    // Unregister
    let result = client
        .call_tool(
            "unregister_discovery",
            json!({"server_name": "test-server"}),
        )
        .await;
    assert!(!result.is_error);

    // Verify it's gone
    let result = client.call_tool("list_discovery_servers", json!({})).await;
    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(!text.contains("test-server") || text.contains("[]"));
}

#[tokio::test]
async fn test_get_discovery_schema() {
    let mut client = create_client().await;

    let result = client.call_tool("get_discovery_schema", json!({})).await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("server") || text.contains("tools"));
}

#[tokio::test]
async fn test_inspect_discovery_index_empty() {
    let mut client = create_client().await;

    let result = client.call_tool("inspect_discovery_index", json!({})).await;

    assert!(!result.is_error);
    // Should work even when empty
}

// =============================================================================
// Engine info
// =============================================================================

#[tokio::test]
async fn test_engine_info() {
    let mut client = create_client().await;

    let result = client.call_tool("engine_info", json!({})).await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("jpx-mcp"));
    assert!(text.contains("version"));
    assert!(text.contains("function_count"));
    assert!(text.contains("strict_mode"));
}

#[tokio::test]
async fn test_engine_info_strict_mode() {
    let mut client = create_strict_client().await;

    let result = client.call_tool("engine_info", json!({})).await;

    assert!(!result.is_error);
    let text = result.first_text().unwrap();
    assert!(text.contains("\"strict_mode\": true") || text.contains("\"strict_mode\":true"));
}

// =============================================================================
// Strict mode tests
// =============================================================================

#[tokio::test]
async fn test_strict_mode_standard_functions_work() {
    let mut client = create_strict_client().await;

    let result = client
        .call_tool(
            "evaluate",
            json!({
                "input": r#"[1, 2, 3]"#,
                "expression": "length(@)"
            }),
        )
        .await;

    assert!(!result.is_error);
    assert_eq!(result.first_text(), Some("3"));
}

#[tokio::test]
async fn test_strict_mode_extension_functions_fail() {
    let mut client = create_strict_client().await;

    let result = client
        .call_tool_expect_error(
            "evaluate",
            json!({
                "input": r#"{"name": "alice"}"#,
                "expression": "upper(name)"  // Extension function
            }),
        )
        .await;

    // Should fail because upper() is an extension
    // Result is either JSON-RPC error (code/message) or tool error (isError/content)
    assert!(
        result.get("code").is_some()
            || result.get("message").is_some()
            || result.get("isError").is_some()
    );
}

// =============================================================================
// Error handling
// =============================================================================

#[tokio::test]
async fn test_missing_required_param() {
    let mut client = create_client().await;

    let result = client.call_tool_expect_error("evaluate", json!({})).await;

    // Either JSON-RPC error or tool error
    assert!(result.get("code").is_some() || result.get("isError").is_some());
}

#[tokio::test]
async fn test_invalid_json_input() {
    let mut client = create_client().await;

    let result = client
        .call_tool_expect_error(
            "evaluate",
            json!({
                "input": "not valid json {",
                "expression": "foo"
            }),
        )
        .await;

    // Either JSON-RPC error or tool error
    assert!(
        result.get("code").is_some()
            || result.get("message").is_some()
            || result.get("isError").is_some()
    );
}

#[tokio::test]
async fn test_unknown_tool() {
    let mut client = create_client().await;

    let result = client
        .call_tool_expect_error("nonexistent_tool", json!({}))
        .await;

    assert!(result.get("code").is_some());
}
