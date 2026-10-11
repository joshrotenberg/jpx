//! The single-expression recipes use the same fixtures in CLI and MCP builds.

use jpx_mcp::build_router;
use serde_json::{Value, json};
use tower_mcp::TestClient;

#[tokio::test]
async fn single_expression_recipes_match_mcp() {
    let recipes: Vec<Value> = serde_json::from_str(include_str!(
        "../../jpx/tests/fixtures/agent_recipes/recipes.json"
    ))
    .unwrap();
    let mut client = TestClient::from_router(build_router(false).unwrap());
    client.initialize().await;
    let mut checked_let = false;
    for recipe in recipes {
        let args = recipe["args"].as_array().unwrap();
        if args.len() != 1 {
            continue;
        }
        let result = client
            .call_tool(
                "evaluate",
                json!({"expression": args[0], "input": recipe["input"]}),
            )
            .await;
        if recipe["expected"]["status"] != 0 {
            assert!(result.is_error, "recipe {} should fail", recipe["id"]);
            // MCP exposes an error result; the CLI's suggestion formatting is
            // specific to stderr and is not a shared protocol contract.
            let function = args[0].as_str().unwrap().split('(').next().unwrap();
            assert!(
                result.first_text().unwrap().contains(function),
                "recipe {} must identify the unknown function {function}",
                recipe["id"]
            );
            continue;
        }
        assert!(
            !result.is_error,
            "{}: {:?}",
            recipe["id"],
            result.first_text()
        );
        let actual: Value = serde_json::from_str(result.first_text().unwrap()).unwrap();
        let expected: Value =
            serde_json::from_str(recipe["expected"]["stdout"].as_str().unwrap()).unwrap();
        assert_eq!(actual, expected, "recipe {}", recipe["id"]);
        checked_let |= recipe["profile"] == "default-with-core-let";
    }
    assert!(
        checked_let,
        "shared corpus must exercise the default let profile"
    );
}
