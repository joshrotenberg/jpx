//! Capability metadata must describe the engine that handles tool calls,
//! including when Cargo enables core syntax through dependency defaults.

use jpx_mcp::build_router;
use serde_json::{Value, json};
use tower_mcp::TestClient;

const LET_EXPRESSION: &str = "let $value = value in $value";

async fn assert_let_capability(strict: bool) {
    let mut client = TestClient::from_router(build_router(strict).unwrap());
    client.initialize().await;

    let evaluated = client
        .call_tool(
            "evaluate",
            json!({"expression": LET_EXPRESSION, "input": r#"{"value": 42}"#}),
        )
        .await;
    assert_eq!(evaluated.is_error, strict);
    if strict {
        assert!(evaluated.first_text().unwrap().contains("strict mode"));
    } else {
        assert_eq!(evaluated.first_text(), Some("42"));
    }

    let result = client.call_tool("engine_info", json!({})).await;
    assert!(!result.is_error);
    let info: Value = serde_json::from_str(result.first_text().unwrap()).unwrap();
    assert_eq!(info["strict_mode"], strict);
    assert_eq!(
        info["let_expressions"], !evaluated.is_error,
        "engine_info must agree with actual let-expression evaluation"
    );
}

#[tokio::test]
async fn normal_engine_reports_let_capability() {
    assert_let_capability(false).await;
}

#[tokio::test]
async fn strict_engine_reports_let_capability() {
    assert_let_capability(true).await;
}
