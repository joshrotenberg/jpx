//! Typed transport and composition regressions for the bounded slice of #261.
//! Fixtures are in memory; these tests do not read user settings or history.

use jpx_mcp::build_router;
use serde_json::{Value, json};
use tower_mcp::{CallToolResult, TestClient, ToolBuilder};

async fn client() -> TestClient {
    let mut client = TestClient::from_router(build_router(false).unwrap());
    client.initialize().await;
    client
}

fn envelope(response: &CallToolResult) -> &Value {
    assert!(!response.is_error, "{response:?}");
    assert_eq!(response.content.len(), 1);
    let structured = response.structured_content.as_ref().unwrap();
    let text: Value = serde_json::from_str(response.first_text().unwrap()).unwrap();
    assert_eq!(&text, structured, "text-only and typed clients must agree");
    assert_eq!(structured["schema_version"], 1);
    structured
}

fn tool_error(response: &CallToolResult, guidance: &str) {
    assert!(response.is_error, "{response:?}");
    assert!(response.structured_content.is_none());
    let wire = serde_json::to_value(response).unwrap();
    assert_eq!(wire["isError"], true);
    assert!(!wire.as_object().unwrap().contains_key("structuredContent"));
    assert!(
        response.first_text().unwrap().contains(guidance),
        "{response:?}"
    );
}

// A focused validator for the output-schema vocabulary used by these tools.
// Fail on unhandled keywords so schema changes cannot silently weaken tests.
fn matches_schema(schema: &Value, value: &Value) -> bool {
    schema
        .as_object()
        .unwrap()
        .iter()
        .all(|(key, rule)| match key.as_str() {
            "type" => match rule.as_str().unwrap() {
                "object" => value.is_object(),
                "array" => value.is_array(),
                "string" => value.is_string(),
                "integer" => value.is_i64() || value.is_u64(),
                other => panic!("unhandled schema type: {other}"),
            },
            "const" => value == rule,
            "required" => rule
                .as_array()
                .unwrap()
                .iter()
                .all(|name| value.get(name.as_str().unwrap()).is_some()),
            "properties" => rule.as_object().unwrap().iter().all(|(name, property)| {
                value.get(name).is_none_or(|v| matches_schema(property, v))
            }),
            "additionalProperties" => {
                assert_eq!(rule, false);
                value.as_object().is_none_or(|object| {
                    object
                        .keys()
                        .all(|name| schema["properties"].get(name).is_some())
                })
            }
            "items" => value
                .as_array()
                .is_none_or(|items| items.iter().all(|item| matches_schema(rule, item))),
            "oneOf" => {
                rule.as_array()
                    .unwrap()
                    .iter()
                    .filter(|alternative| matches_schema(alternative, value))
                    .count()
                    == 1
            }
            "minLength" => value
                .as_str()
                .is_none_or(|s| s.chars().count() >= rule.as_u64().unwrap() as usize),
            other => panic!("unhandled schema keyword: {other}"),
        })
}

#[tokio::test]
async fn every_json_category_and_literal_strings() {
    let mut client = client().await;
    let tools = client.list_tools().await;
    let single_schema = &tools
        .iter()
        .find(|tool| tool["name"] == "evaluate_json")
        .unwrap()["outputSchema"];
    let batch_schema = &tools
        .iter()
        .find(|tool| tool["name"] == "batch_evaluate_json")
        .unwrap()["outputSchema"];
    let cases = [
        (
            json!({"nested": ["{\"x\":[1]}", "[true,null]", "\"quoted\"\\\n\r\t\u{0000} café 東京 🦀"]}),
            "nested[0]",
            json!("{\"x\":[1]}"),
        ),
        (json!([1, null, {"a": true}]), "[2].a", json!(true)),
        (
            json!("{\"looks\":\"like JSON\"}"),
            "type(@)",
            json!("string"),
        ),
        (json!("[1,2]"), "type(@)", json!("string")),
        (json!("null"), "type(@)", json!("string")),
        (json!("42"), "type(@)", json!("string")),
        (
            json!("\"quoted\"\\\n\r\t\u{0001} café 東京 🦀"),
            "type(@)",
            json!("string"),
        ),
        (json!(42), "@ == `42`", json!(true)),
        (json!(1.5), "@ > `1`", json!(true)),
        (json!(true), "!@", json!(false)),
        (json!(false), "!@", json!(true)),
        (Value::Null, "type(@)", json!("null")),
    ];
    for (input, expression, expected) in cases {
        for (expr, expected) in [("@", &input), (expression, &expected)] {
            let response = client
                .call_tool("evaluate_json", json!({"input": input, "expression": expr}))
                .await;
            let output = envelope(&response);
            assert_eq!(&output["result"], expected);
            assert!(matches_schema(single_schema, output));
        }
        let response = client
            .call_tool(
                "batch_evaluate_json",
                json!({"input": input, "expressions": ["@", expression]}),
            )
            .await;
        let output = envelope(&response);
        assert_eq!(
            output["results"],
            json!([
                {"expression": "@", "result": input},
                {"expression": expression, "result": expected}
            ])
        );
        assert!(matches_schema(batch_schema, output));
    }
}

#[tokio::test]
async fn supported_numbers_survive_rust_json_transport() {
    let mut client = client().await;
    // These travel through actual serde JSON bytes as well as TestClient's
    // JSON-RPC service. This makes no claim about pre-transport JS rounding or
    // arbitrary decimal precision; floating-point cases are binary64 values.
    for number in [
        json!(9_007_199_254_740_993_u64),
        json!(i64::MIN),
        json!(i64::MAX),
        json!(u64::MAX),
        json!(0.0),
        json!(-0.0),
        json!(0.1),
        json!(1.5),
        json!(-1234.125),
        json!(1.0e100),
    ] {
        for (tool, expression_fields) in [
            ("evaluate_json", json!({"expression": "@"})),
            ("batch_evaluate_json", json!({"expressions": ["@"]})),
        ] {
            let mut args = expression_fields;
            args["input"] = number.clone();
            let args = serde_json::from_slice(&serde_json::to_vec(&args).unwrap()).unwrap();
            let response = client.call_tool(tool, args).await;
            let response: CallToolResult =
                serde_json::from_slice(&serde_json::to_vec(&response).unwrap()).unwrap();
            let output = envelope(&response);
            let actual = if tool == "evaluate_json" {
                &output["result"]
            } else {
                &output["results"][0]["result"]
            };
            assert_eq!(actual, &number);
            // Preserve numeric representation too, including -0.0 and exact
            // integer digits (not merely a lossy as_f64 comparison).
            assert_eq!(
                serde_json::to_string(actual).unwrap(),
                serde_json::to_string(&number).unwrap()
            );
        }
    }
}

#[tokio::test]
async fn typed_request_errors_are_corrective_and_null_is_present() {
    let mut client = client().await;
    for (tool, valid) in [
        ("evaluate_json", json!({"expression": "@", "input": null})),
        (
            "batch_evaluate_json",
            json!({"expressions": ["@"], "input": null}),
        ),
    ] {
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("input");
        let response = client.call_tool(tool, missing.clone()).await;
        tool_error(&response, "missing field `input`");
        tool_error(&response, "native JSON value");
        for field in [
            "input_text",
            "file_path",
            "unexpected",
            "expression",
            "expressions",
        ] {
            if valid.get(field).is_some() {
                continue;
            }
            // Reject both a competing mode alongside input and a replacement
            // mode without input. Legacy handlers remain permissive.
            for mut args in [valid.clone(), missing.clone()] {
                args[field] = json!("payload");
                let response = client.call_tool(tool, args).await;
                tool_error(&response, "native JSON value");
                tool_error(&response, "Remove other fields");
            }
        }
        for args in [json!({}), Value::Null, json!([]), json!(42)] {
            tool_error(
                &client.call_tool(tool, args).await,
                "Supply only required 'input'",
            );
        }
        let field = if tool == "evaluate_json" {
            "expression"
        } else {
            "expressions"
        };
        for wrong in [Value::Null, json!(42), json!({})] {
            let mut args = valid.clone();
            args[field] = wrong;
            tool_error(
                &client.call_tool(tool, args).await,
                "Supply only required 'input'",
            );
        }
        let response = client.call_tool(tool, valid).await;
        let output = envelope(&response);
        let result = if tool == "evaluate_json" {
            output.get("result")
        } else {
            output["results"][0].get("result")
        };
        assert_eq!(result, Some(&Value::Null));
    }
}

#[tokio::test]
async fn single_failures_and_ordered_mixed_batch() {
    let mut client = client().await;
    for expression in ["[", "length(`42`)", "unknown_function(@)"] {
        let response = client
            .call_tool(
                "evaluate_json",
                json!({"input": {"x": 7}, "expression": expression}),
            )
            .await;
        tool_error(&response, "use 'validate' for syntax");
        tool_error(&response, "'describe' for function signatures");
    }
    let response = client
        .call_tool(
            "batch_evaluate_json",
            json!({"input": {"x": 7}, "expressions": ["x", "missing", "[", "x"]}),
        )
        .await;
    let output = envelope(&response);
    let results = output["results"].as_array().unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(results[0], json!({"expression": "x", "result": 7}));
    assert_eq!(results[1], json!({"expression": "missing", "result": null}));
    assert_eq!(results[2]["expression"], "[");
    assert!(results[2].get("result").is_none());
    assert!(!results[2]["error"].as_str().unwrap().is_empty());
    assert_eq!(results[3], results[0]);
    let tools = client.list_tools().await;
    let schema = &tools
        .iter()
        .find(|tool| tool["name"] == "batch_evaluate_json")
        .unwrap()["outputSchema"];
    assert!(matches_schema(schema, output));
}

#[tokio::test]
async fn empty_and_boundary_batch_counts() {
    let mut client = client().await;
    for count in [0, 1000, 1001] {
        let response = client
            .call_tool(
                "batch_evaluate_json",
                json!({"input": null, "expressions": vec!["@"; count]}),
            )
            .await;
        if count > 1000 {
            tool_error(&response, "Split expressions into batches of at most 1000");
        } else {
            let output = envelope(&response);
            assert_eq!(
                output["results"],
                json!(vec![json!({"expression": "@", "result": null}); count])
            );
        }
    }
}

#[tokio::test]
async fn compact_utf8_bytes_accept_exact_limit_and_reject_one_more() {
    const LIMIT: usize = 50 * 1024 * 1024;
    let mut client = client().await;
    // Includes multibyte UTF-8, quotes, backslashes and control escapes. Counting
    // chars or the unescaped string's bytes would incorrectly accept LIMIT+1.
    let suffix = "é東京🦀\"\\\n\r\t\u{0000}";
    let serialized_suffix_bytes = serde_json::to_vec(suffix).unwrap().len();
    for tool in ["evaluate_json", "batch_evaluate_json"] {
        for extra in [0, 1] {
            let mut text = "a".repeat(LIMIT - serialized_suffix_bytes + extra);
            text.push_str(suffix);
            assert!(text.len() < LIMIT);
            let input = Value::String(text);
            assert_eq!(serde_json::to_vec(&input).unwrap().len(), LIMIT + extra);
            let mut args = if tool == "evaluate_json" {
                json!({"expression": "type(@)"})
            } else {
                json!({"expressions": ["type(@)"]})
            };
            args["input"] = input;
            let response = client.call_tool(tool, args).await;
            if extra == 0 {
                let output = envelope(&response);
                let result = if tool == "evaluate_json" {
                    &output["result"]
                } else {
                    &output["results"][0]["result"]
                };
                assert_eq!(result, "string");
            } else {
                tool_error(&response, "52428800 bytes");
                tool_error(&response, "Reduce or split the input");
            }
        }
    }
}

#[tokio::test]
async fn advertised_schemas_enforce_version_and_exclusive_batch_outcomes() {
    let mut client = client().await;
    let tools = client.list_tools().await;
    for (tool, field) in [
        ("evaluate_json", "expression"),
        ("batch_evaluate_json", "expressions"),
    ] {
        let metadata = tools
            .iter()
            .find(|metadata| metadata["name"] == tool)
            .unwrap();
        let input_schema = &metadata["inputSchema"];
        assert_eq!(input_schema["type"], "object");
        assert_eq!(input_schema["additionalProperties"], false);
        let required = input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&json!("input")));
        assert!(required.contains(&json!(field)));
        assert_eq!(input_schema["properties"].as_object().unwrap().len(), 2);
        assert!(
            input_schema["properties"]["input"].get("type").is_none(),
            "native input accepts every JSON type"
        );
        let schema = &metadata["outputSchema"];
        assert_eq!(schema["type"], "object");
        let valid = if tool == "evaluate_json" {
            json!({"schema_version": 1, "result": null})
        } else {
            json!({"schema_version": 1, "results": []})
        };
        assert!(matches_schema(schema, &valid));
        for bad in [
            Value::Null,
            json!([]),
            json!({}),
            json!({"schema_version": 1}),
        ] {
            assert!(!matches_schema(schema, &bad));
        }
        let mut wrong_version = valid.clone();
        wrong_version["schema_version"] = json!(2);
        assert!(!matches_schema(schema, &wrong_version));
        let mut missing_version = valid;
        missing_version
            .as_object_mut()
            .unwrap()
            .remove("schema_version");
        assert!(!matches_schema(schema, &missing_version));
        if tool == "batch_evaluate_json" {
            assert_eq!(input_schema["properties"]["expressions"]["maxItems"], 1000);
            for bad_entry in [
                json!({"expression": "@", "result": null, "error": "failed"}),
                json!({"expression": "@"}),
                json!({"expression": "@", "error": ""}),
                json!({"expression": "@", "error": null}),
                json!({"result": 7}),
            ] {
                assert!(!matches_schema(
                    schema,
                    &json!({"schema_version": 1, "results": [bad_entry]})
                ));
            }
        }
    }
    // Output schemas and structuredContent are additive only on the new tools.
    assert_eq!(
        tools
            .iter()
            .filter(|tool| tool.get("outputSchema").is_some())
            .count(),
        2
    );
}

#[tokio::test]
async fn legacy_evaluation_text_shapes_and_permissive_fields_stay_unchanged() {
    let mut client = client().await;
    let response = client
        .call_tool(
            "evaluate",
            json!({"input": "{\"x\":1}", "expression": "x", "input_text": "ignored"}),
        )
        .await;
    assert!(!response.is_error);
    assert!(response.structured_content.is_none());
    assert_eq!(response.first_text(), Some("1"));
    let response = client
        .call_tool(
            "batch_evaluate",
            json!({"input": "{\"x\":1}", "expressions": ["x", "missing"], "file_path": "ignored"}),
        )
        .await;
    assert!(!response.is_error);
    assert!(response.structured_content.is_none());
    assert_eq!(
        response.first_text(),
        Some(
            "{\n  \"results\": [\n    {\n      \"expression\": \"x\",\n      \"result\": 1\n    },\n    {\n      \"expression\": \"missing\",\n      \"result\": null\n    }\n  ]\n}"
        )
    );
}

#[tokio::test]
async fn source_tool_to_typed_evaluation_to_consumer_and_wire_size() {
    // This fixture source tool produces an actual structured value. The
    // application passes it directly between tools and consumes the result;
    // no model transcription, payload files, or stringify/parse convention.
    let source = ToolBuilder::new("fixture_source")
        .handler(|_: Value| async {
            Ok(CallToolResult::json(json!({"records": [
                {"id": 9_007_199_254_740_993_u64, "active": true, "label": "café \"quoted\"\\\n"},
                {"id": 2, "active": false, "label": "ignore"}
            ]})))
        })
        .build();
    let router = build_router(false).unwrap().tool(source);
    let mut client = TestClient::from_router(router);
    client.initialize().await;
    let source_response = client.call_tool("fixture_source", json!({})).await;
    assert!(!source_response.is_error);
    let input = source_response.structured_content.unwrap();
    let mut args = json!({"expression": "records[?active].{id: id, label: label}"});
    args["input"] = input;
    let response = client.call_tool("evaluate_json", args).await;
    assert!(!response.is_error);
    let consumed = &response.structured_content.as_ref().unwrap()["result"];
    assert_eq!(consumed[0]["id"].as_u64(), Some(9_007_199_254_740_993));
    assert_eq!(consumed[0]["label"], "café \"quoted\"\\\n");
    assert_eq!(consumed.as_array().unwrap().len(), 1);

    // Account for the *whole* CallToolResult: text content is serialized again
    // with JSON escaping, in addition to structuredContent. JSON-RPC framing
    // adds bytes beyond this, illustrated with a numeric request id of 1.
    envelope(&response);
    let result_bytes = serde_json::to_vec(&response).unwrap().len();
    let text_bytes = response.first_text().unwrap().len();
    let structured_bytes = serde_json::to_vec(response.structured_content.as_ref().unwrap())
        .unwrap()
        .len();
    assert!(result_bytes > text_bytes + structured_bytes);
    let framed_bytes = serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": 1, "result": response}))
        .unwrap()
        .len();
    let framing_bytes = framed_bytes - result_bytes;
    assert_eq!(framing_bytes, 34);
    println!(
        "typed composition: CallToolResult={result_bytes} bytes, text={text_bytes} bytes, structuredContent={structured_bytes} bytes; JSON-RPC framing (id=1) adds {framing_bytes} bytes"
    );
}
