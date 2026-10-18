//! MCP regression tests for exact numeric comparison (#259).
//!
//! Inputs travel as JSON *text* (the `input` string or a file), which is the
//! exact path: structured MCP arguments may already have been rounded by the
//! client before jpx sees them.

use jpx_mcp::build_router;
use serde_json::{Value, json};
use tower_mcp::TestClient;

async fn create_client() -> TestClient {
    let router = build_router(false).expect("Failed to build router");
    let mut client = TestClient::from_router(router);
    client.initialize().await;
    client
}

fn exact(text: &str) -> Value {
    serde_json::from_str(text).expect("fixture should be valid JSON")
}

async fn evaluate(client: &mut TestClient, expression: &str, input: &str) -> Value {
    let result = client
        .call_tool(
            "evaluate",
            json!({ "input": input, "expression": expression }),
        )
        .await;
    assert!(
        !result.is_error,
        "evaluate failed: {:?}",
        result.first_text()
    );
    exact(result.first_text().expect("result should have text"))
}

#[tokio::test]
async fn evaluate_keeps_adjacent_integers_distinct() {
    let mut client = create_client().await;
    assert_eq!(
        evaluate(
            &mut client,
            "{equal: a == b, less: a < b}",
            r#"{"a":9007199254740992,"b":9007199254740993}"#,
        )
        .await,
        exact(r#"{"equal":false,"less":true}"#)
    );
}

#[tokio::test]
async fn evaluate_filter_selects_only_the_exact_id() {
    let mut client = create_client().await;
    assert_eq!(
        evaluate(
            &mut client,
            "[?@ == `9007199254740992`]",
            "[9007199254740992,9007199254740993]",
        )
        .await,
        exact("[9007199254740992]")
    );
    assert_eq!(
        evaluate(
            &mut client,
            "{sorted: sort(@), max: max(@)}",
            "[18446744073709551615,18446744073709551614]",
        )
        .await,
        exact(
            r#"{"sorted":[18446744073709551614,18446744073709551615],"max":18446744073709551615}"#
        )
    );
}

#[tokio::test]
async fn evaluate_file_keeps_adjacent_integers_distinct() {
    let mut client = create_client().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ids.json");
    std::fs::write(
        &path,
        r#"[{"id":9007199254740992,"n":"a"},{"id":9007199254740993,"n":"b"}]"#,
    )
    .unwrap();

    let result = client
        .call_tool(
            "evaluate_file",
            json!({
                "file_path": path.to_str().unwrap(),
                "expression": "[?id == `9007199254740993`].n"
            }),
        )
        .await;

    assert!(
        !result.is_error,
        "evaluate_file failed: {:?}",
        result.first_text()
    );
    assert_eq!(
        exact(result.first_text().expect("result should have text")),
        exact(r#"["b"]"#)
    );
}
