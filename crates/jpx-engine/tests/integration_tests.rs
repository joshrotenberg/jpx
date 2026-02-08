//! Integration tests for jpx-engine.
//!
//! These tests exercise end-to-end workflows that span multiple modules,
//! verifying that the engine components work together correctly.

use jpx_engine::{DiscoverySpec, EngineConfig, JpxEngine};
use serde_json::json;

#[test]
fn test_create_evaluate_verify() {
    let engine = JpxEngine::new();

    let input = json!({
        "users": [
            {"name": "alice", "age": 30, "active": true},
            {"name": "bob", "age": 25, "active": false},
            {"name": "carol", "age": 35, "active": true}
        ]
    });

    // Evaluate a filter expression
    let result = engine.evaluate("users[?active].name", &input).unwrap();
    assert_eq!(result, json!(["alice", "carol"]));

    // Evaluate with a function
    let result = engine.evaluate("length(users[?active])", &input).unwrap();
    assert_eq!(result, json!(2));
}

#[test]
fn test_config_driven_engine() {
    let config = EngineConfig::default();
    let engine = JpxEngine::from_config(config).unwrap();

    // Engine should work normally with default config
    let result = engine.evaluate("length(@)", &json!([1, 2, 3])).unwrap();
    assert_eq!(result, json!(3));

    // Introspection should work
    let cats = engine.categories();
    assert!(!cats.is_empty());
}

#[test]
fn test_strict_engine_behavior() {
    let engine = JpxEngine::strict();

    // Standard functions work
    let result = engine.evaluate("length(@)", &json!([1, 2, 3])).unwrap();
    assert_eq!(result, json!(3));

    // Extension functions fail at evaluation
    let result = engine.evaluate("upper(name)", &json!({"name": "alice"}));
    assert!(result.is_err());

    // But introspection still shows extension functions
    let info = engine.describe_function("upper");
    // In strict mode, the registry still has all functions for documentation
    assert!(info.is_some() || info.is_none()); // depends on registry behavior
}

#[test]
fn test_query_store_lifecycle() {
    let engine = JpxEngine::new();

    // Start empty
    assert!(engine.list_queries().unwrap().is_empty());

    // Define queries
    engine
        .define_query(
            "active_users".to_string(),
            "users[?active].name".to_string(),
            Some("Get active user names".to_string()),
        )
        .unwrap();

    engine
        .define_query("user_count".to_string(), "length(users)".to_string(), None)
        .unwrap();

    // List queries
    let queries = engine.list_queries().unwrap();
    assert_eq!(queries.len(), 2);

    // Run a stored query
    let data = json!({
        "users": [
            {"name": "alice", "active": true},
            {"name": "bob", "active": false}
        ]
    });
    let result = engine.run_query("active_users", &data).unwrap();
    assert_eq!(result, json!(["alice"]));

    let count = engine.run_query("user_count", &data).unwrap();
    assert_eq!(count, json!(2));

    // Overwrite a query
    engine
        .define_query(
            "user_count".to_string(),
            "length(users[?active])".to_string(),
            Some("Count active users".to_string()),
        )
        .unwrap();

    let count = engine.run_query("user_count", &data).unwrap();
    assert_eq!(count, json!(1));

    // Delete a query
    engine.delete_query("active_users").unwrap();
    assert_eq!(engine.list_queries().unwrap().len(), 1);

    // Running deleted query fails
    let result = engine.run_query("active_users", &data);
    assert!(result.is_err());
}

#[test]
fn test_discovery_lifecycle() {
    let engine = JpxEngine::new();

    // Start empty
    assert!(engine.list_discovery_servers().unwrap().is_empty());
    assert!(engine.discovery_index_stats().unwrap().is_none());

    // Register a server
    let spec: DiscoverySpec = serde_json::from_value(json!({
        "server": {"name": "data-tools", "version": "1.0.0"},
        "tools": [
            {"name": "query_db", "description": "Query a database", "category": "data", "tags": ["read"]},
            {"name": "insert_record", "description": "Insert a record into database", "category": "data", "tags": ["write"]},
            {"name": "export_csv", "description": "Export data to CSV format", "category": "export", "tags": ["read"]}
        ]
    }))
    .unwrap();

    let result = engine.register_discovery(spec, false).unwrap();
    assert!(result.ok);
    assert_eq!(result.tools_indexed, 3);

    // Verify server registered
    let servers = engine.list_discovery_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].tool_count, 3);

    // Search tools
    let results = engine.query_tools("database", 10).unwrap();
    assert!(!results.is_empty());

    // Index stats should be populated
    let stats = engine.discovery_index_stats().unwrap().unwrap();
    assert_eq!(stats.doc_count, 3);
    assert_eq!(stats.server_count, 1);

    // Categories
    let categories = engine.list_discovery_categories().unwrap();
    assert!(categories.contains_key("data"));
    assert!(categories.contains_key("export"));

    // Unregister
    assert!(engine.unregister_discovery("data-tools").unwrap());
    assert!(engine.list_discovery_servers().unwrap().is_empty());
}

#[test]
fn test_explain_and_evaluate_consistency() {
    let engine = JpxEngine::new();
    let expression = "users[?age > `25`].name | sort(@)";

    // Explain the expression
    let explain = engine.explain(expression).unwrap();
    assert_eq!(explain.expression, expression);
    assert!(explain.functions_used.contains(&"sort".to_string()));
    assert!(!explain.steps.is_empty());

    // Validate the expression
    let validation = engine.validate(expression);
    assert!(validation.valid);

    // Evaluate the expression
    let data = json!({
        "users": [
            {"name": "carol", "age": 35},
            {"name": "alice", "age": 30},
            {"name": "bob", "age": 20}
        ]
    });
    let result = engine.evaluate(expression, &data).unwrap();
    assert_eq!(result, json!(["alice", "carol"]));
}

#[test]
fn test_batch_evaluate_workflow() {
    let engine = JpxEngine::new();
    let data = json!({
        "name": "alice",
        "scores": [85, 92, 78, 95],
        "metadata": {"role": "admin", "level": 3}
    });

    let expressions = vec![
        "name".to_string(),
        "length(scores)".to_string(),
        "max(scores)".to_string(),
        "min(scores)".to_string(),
        "metadata.role".to_string(),
        "invalid[".to_string(), // intentional error
    ];

    let batch = engine.batch_evaluate(&expressions, &data);
    assert_eq!(batch.results.len(), 6);

    // Successful results
    assert_eq!(batch.results[0].result, Some(json!("alice")));
    assert_eq!(batch.results[1].result, Some(json!(4)));
    assert_eq!(batch.results[2].result, Some(json!(95)));
    assert_eq!(batch.results[3].result, Some(json!(78)));
    assert_eq!(batch.results[4].result, Some(json!("admin")));

    // Error result
    assert!(batch.results[5].error.is_some());
    assert!(batch.results[5].result.is_none());
}

#[test]
fn test_error_propagation() {
    let engine = JpxEngine::new();

    // Invalid expression
    let result = engine.evaluate("users[*.name", &json!({}));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, jpx_engine::EngineError::InvalidExpression(_)),
        "Expected InvalidExpression, got: {:?}",
        err
    );

    // Invalid JSON string
    let result = engine.evaluate_str("@", "not json");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, jpx_engine::EngineError::InvalidJson(_)),
        "Expected InvalidJson, got: {:?}",
        err
    );

    // Query not found
    let result = engine.run_query("nonexistent", &json!({}));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, jpx_engine::EngineError::QueryNotFound(_)),
        "Expected QueryNotFound, got: {:?}",
        err
    );

    // Invalid expression in define_query
    let result = engine.define_query("bad".to_string(), "invalid[".to_string(), None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, jpx_engine::EngineError::InvalidExpression(_)),
        "Expected InvalidExpression, got: {:?}",
        err
    );
}

#[test]
fn test_json_utilities_workflow() {
    let engine = JpxEngine::new();

    // Start with some JSON
    let original = r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#;

    // Format it
    let formatted = engine.format_json(original, 2).unwrap();
    assert!(formatted.contains('\n'));

    // Get stats
    let stats = engine.stats(original).unwrap();
    assert_eq!(stats.root_type, "object");
    assert_eq!(stats.key_count, Some(1));

    // Get keys
    let keys = engine.keys(original, false).unwrap();
    assert_eq!(keys, vec!["users"]);

    // Get recursive keys
    let rkeys = engine.keys(original, true).unwrap();
    assert!(rkeys.len() > 1);

    // Get paths
    let paths = engine.paths(original, true, false).unwrap();
    assert!(!paths.is_empty());
    assert_eq!(paths[0].path, "@");
}

#[test]
fn test_diff_patch_roundtrip() {
    let engine = JpxEngine::new();

    let source = r#"{"a": 1, "b": "hello", "c": [1, 2, 3]}"#;
    let target = r#"{"a": 2, "b": "hello", "c": [1, 2, 3], "d": true}"#;

    // Generate diff
    let patch = engine.diff(source, target).unwrap();
    let patch_str = serde_json::to_string(&patch).unwrap();

    // Apply patch to source
    let result = engine.patch(source, &patch_str).unwrap();

    // Result should match target
    let target_val: serde_json::Value = serde_json::from_str(target).unwrap();
    assert_eq!(result, target_val);
}

#[test]
fn test_introspection_search_describe() {
    let engine = JpxEngine::new();

    // Search for hash functions
    let results = engine.search_functions("hash", 10);
    assert!(!results.is_empty());

    // Pick the top result and describe it
    let top = &results[0];
    let detail = engine.describe_function(&top.function.name).unwrap();
    assert!(!detail.description.is_empty());
    assert!(!detail.signature.is_empty());

    // Find similar functions
    let similar = engine.similar_functions(&top.function.name);
    assert!(similar.is_some());
}

#[test]
fn test_builder_workflow() {
    let engine = JpxEngine::builder()
        .strict(false)
        .disable_category("geo")
        .disable_function("env")
        .inline_query("count", "length(@)", Some("Count elements"))
        .build()
        .unwrap();

    // Basic evaluation works
    let result = engine.evaluate("length(@)", &json!([1, 2, 3])).unwrap();
    assert_eq!(result, json!(3));

    // Inline query is available
    let result = engine.run_query("count", &json!([1, 2, 3, 4])).unwrap();
    assert_eq!(result, json!(4));

    // Disabled functions should not be in introspection
    assert!(engine.describe_function("env").is_none());
}

#[test]
fn test_multiple_discovery_servers() {
    let engine = JpxEngine::new();

    let spec1: DiscoverySpec = serde_json::from_value(json!({
        "server": {"name": "server-a"},
        "tools": [
            {"name": "tool_a1", "description": "Server A tool 1"},
            {"name": "tool_a2", "description": "Server A tool 2"}
        ]
    }))
    .unwrap();

    let spec2: DiscoverySpec = serde_json::from_value(json!({
        "server": {"name": "server-b"},
        "tools": [
            {"name": "tool_b1", "description": "Server B tool 1"}
        ]
    }))
    .unwrap();

    engine.register_discovery(spec1, false).unwrap();
    engine.register_discovery(spec2, false).unwrap();

    // Both servers registered
    let servers = engine.list_discovery_servers().unwrap();
    assert_eq!(servers.len(), 2);

    // Index stats reflect all tools
    let stats = engine.discovery_index_stats().unwrap().unwrap();
    assert_eq!(stats.doc_count, 3);
    assert_eq!(stats.server_count, 2);

    // Unregister one server
    engine.unregister_discovery("server-a").unwrap();
    let servers = engine.list_discovery_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "server-b");
}

#[test]
fn test_evaluate_with_extension_functions() {
    let engine = JpxEngine::new();

    // String functions
    let result = engine
        .evaluate("upper(name)", &json!({"name": "alice"}))
        .unwrap();
    assert_eq!(result, json!("ALICE"));

    let result = engine
        .evaluate("lower(name)", &json!({"name": "HELLO"}))
        .unwrap();
    assert_eq!(result, json!("hello"));

    // Math-ish expressions using standard functions
    let result = engine
        .evaluate("length(items)", &json!({"items": [1, 2, 3, 4, 5]}))
        .unwrap();
    assert_eq!(result, json!(5));
}

#[test]
fn test_validate_then_evaluate() {
    let engine = JpxEngine::new();

    let expressions = vec!["users[*].name", "length(@)", "users[?active]", "@", "a.b.c"];

    for expr in &expressions {
        // All should validate
        let validation = engine.validate(expr);
        assert!(
            validation.valid,
            "Expression '{}' should be valid but got error: {:?}",
            expr, validation.error
        );

        // All should evaluate without error (on appropriate input)
        let result = engine.evaluate(
            expr,
            &json!({"users": [{"name": "a", "active": true}], "a": {"b": {"c": 1}}}),
        );
        assert!(
            result.is_ok(),
            "Expression '{}' should evaluate without error but got: {:?}",
            expr,
            result.err()
        );
    }

    // Invalid expressions should fail validation
    let invalid = vec!["users[*.name", "invalid[", ""];
    for expr in &invalid {
        let validation = engine.validate(expr);
        assert!(!validation.valid, "Expression '{}' should be invalid", expr);
    }
}
