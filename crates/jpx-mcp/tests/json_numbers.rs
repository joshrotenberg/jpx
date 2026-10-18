#[path = "support/json_numbers.rs"]
mod json_numbers;

#[test]
fn mcp_json_text_is_correctly_rounded() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let router = jpx_mcp::build_router(false).unwrap();
    let mut client = tower_mcp::TestClient::from_router(router);
    runtime.block_on(client.initialize());
    for tool in ["evaluate", "batch_evaluate"] {
        json_numbers::check_json_numbers(|input| {
            let params = if tool == "batch_evaluate" {
                serde_json::json!({"input": input, "expressions": ["@"]})
            } else {
                serde_json::json!({"input": input, "expression": "@"})
            };
            let result = runtime.block_on(client.call_tool(tool, params));
            assert!(!result.is_error, "{tool}: {input}");
            let value: serde_json::Value =
                serde_json::from_str(result.first_text().unwrap()).unwrap();
            if tool == "batch_evaluate" {
                value["results"][0]["result"].clone()
            } else {
                value
            }
        });
    }
}
