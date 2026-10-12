//! Typed composition is additive: legacy tools keep their JSON-text contracts.

use super::{MAX_BATCH_EXPRESSIONS, MAX_INLINE_JSON_BYTES};
use jpx_engine::JpxEngine;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{io, sync::Arc};
use tower_mcp::{CallToolResult, Error, Tool, ToolBuilder};

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct EvaluateJsonParams {
    /// Required native JSON value. Strings are literal; explicit null is valid.
    // Value's default deserializer accepts an omitted field as null. A field
    // deserializer forces serde to reject omission before deserializing Value.
    #[serde(deserialize_with = "Value::deserialize")]
    #[schemars(required)]
    input: Value,
    /// JMESPath expression to evaluate.
    expression: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct BatchEvaluateJsonParams {
    /// Required native JSON value. Strings are literal; explicit null is valid.
    #[serde(deserialize_with = "Value::deserialize")]
    #[schemars(required)]
    input: Value,
    /// JMESPath expressions, evaluated independently in order (at most 1000).
    #[schemars(length(max = MAX_BATCH_EXPRESSIONS))]
    expressions: Vec<String>,
}

fn params<T: DeserializeOwned>(arguments: Value, expression_field: &str) -> Result<T, Error> {
    // Validate here so every invalid request gets the corrective typed-input
    // guidance, including missing input, extra fields, and wrong field types.
    let invalid = |reason: String| {
        Error::tool(format!(
            "Invalid typed JSON request: {reason}. Supply only required 'input' as a native JSON value \
             (object, array, string, number, boolean, or explicit null) and '{expression_field}'. \
             Strings are literal, not encoded JSON. Remove other fields such as 'input_text' or \
             'file_path'; use 'evaluate'/'batch_evaluate' for JSON text or 'evaluate_file' for a file."
        ))
    };
    // serde maps a sequence onto struct fields by position, so check the
    // container first: only a named-field object matches the input schema.
    if !arguments.is_object() {
        return Err(invalid(
            "arguments must be a JSON object with named fields, not positional values".into(),
        ));
    }
    serde_json::from_value(arguments).map_err(|e| invalid(e.to_string()))
}

/// Count the actual compact JSON bytes without allocating a serialized input
/// buffer. Stop at the ceiling. Transport has *already* allocated the Value;
/// this is neither a transport-memory limit nor an evaluation/output budget.
fn ensure_input_size(input: &Value) -> Result<(), Error> {
    struct ByteCounter(usize);
    impl io::Write for ByteCounter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() > MAX_INLINE_JSON_BYTES - self.0 {
                return Err(io::Error::other("typed input exceeds byte limit"));
            }
            self.0 += bytes.len();
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    serde_json::to_writer(ByteCounter(0), input).map_err(|e| {
        Error::tool(format!(
            "Cannot serialize input within the maximum size of {MAX_INLINE_JSON_BYTES} bytes \
             (50 MiB) of compact UTF-8 JSON: {e}. Reduce or split the input; quotes and JSON \
             escapes count toward this limit."
        ))
    })
}

fn evaluation_error(error: impl std::fmt::Display) -> String {
    format!(
        "{error} Check the expression and input types; use 'validate' for syntax and \
         'describe' for function signatures."
    )
}

fn envelope_schema(field: &str, value_schema: Value) -> Value {
    json!({
        "type": "object",
        "properties": {
            "schema_version": {"type": "integer", "const": 1},
            (field): value_schema
        },
        "required": ["schema_version", field],
        "additionalProperties": false
    })
}

// An empty schema permits every JSON value, including arrays, scalars and null.
fn batch_results_schema() -> Value {
    json!({
        "type": "array",
        "items": {
            "oneOf": [
                {
                    "type": "object",
                    "properties": {"expression": {"type": "string"}, "result": {}},
                    "required": ["expression", "result"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "expression": {"type": "string"},
                        "error": {"type": "string", "minLength": 1}
                    },
                    "required": ["expression", "error"],
                    "additionalProperties": false
                }
            ]
        }
    })
}

pub(super) fn evaluate_tool(engine: Arc<JpxEngine>) -> Tool {
    ToolBuilder::new("evaluate_json")
        .title("Evaluate Native JSON")
        .description("Evaluate a JMESPath expression against required native JSON input (any JSON value, including null; strings stay literal). Accepts only input and expression. Input is limited to 50 MiB of compact serialized UTF-8 JSON. Returns {schema_version: 1, result: <value>} as structuredContent and equivalent JSON text. Use evaluate for encoded JSON text. No output-size bound.")
        .read_only()
        .input_schema(schemars::schema_for!(EvaluateJsonParams).into())
        .output_schema(envelope_schema("result", json!({})))
        .handler(move |arguments: Value| {
            let engine = engine.clone();
            async move {
                let params: EvaluateJsonParams = params(arguments, "expression")?;
                ensure_input_size(&params.input)?;
                let result = engine.evaluate(&params.expression, &params.input)
                    .map_err(|e| Error::tool(evaluation_error(e)))?;
                // Move the result into its envelope; CallToolResult::json adds
                // the equivalent text representation for text-only clients.
                Ok(CallToolResult::json(Value::Object([
                    ("schema_version".into(), json!(1)),
                    ("result".into(), result),
                ].into_iter().collect())))
            }
        })
        .build()
}

pub(super) fn batch_evaluate_tool(engine: Arc<JpxEngine>) -> Tool {
    ToolBuilder::new("batch_evaluate_json")
        .title("Batch Evaluate Native JSON")
        .description("Evaluate up to 1000 JMESPath expressions against required native JSON input (any JSON value, including null; strings stay literal). Accepts only input and expressions. Input is limited to 50 MiB of compact serialized UTF-8 JSON. Returns {schema_version: 1, results: [...]} as structuredContent and equivalent JSON text. Ordered entries have expression and exactly one of result or error; mixed batches succeed. No output-size bound.")
        .read_only()
        .input_schema(schemars::schema_for!(BatchEvaluateJsonParams).into())
        .output_schema(envelope_schema("results", batch_results_schema()))
        .handler(move |arguments: Value| {
            let engine = engine.clone();
            async move {
                let params: BatchEvaluateJsonParams = params(arguments, "expressions")?;
                if params.expressions.len() > MAX_BATCH_EXPRESSIONS {
                    return Err(Error::tool(format!(
                        "Too many expressions: {} (maximum {MAX_BATCH_EXPRESSIONS}). \
                         Split expressions into batches of at most {MAX_BATCH_EXPRESSIONS}.",
                        params.expressions.len()
                    )));
                }
                ensure_input_size(&params.input)?;
                let mut batch = engine.batch_evaluate(&params.expressions, &params.input);
                for entry in &mut batch.results {
                    if let Some(error) = entry.error.take() {
                        entry.error = Some(evaluation_error(error));
                    }
                }
                Ok(CallToolResult::json(json!({"schema_version": 1, "results": batch.results})))
            }
        })
        .build()
}
