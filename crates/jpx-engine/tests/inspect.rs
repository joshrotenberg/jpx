//! Public API boundary tests for bounded structural inspection.

use jpx_engine::{EngineError, InspectLimits, JpxEngine, inspect_value};
use serde_json::json;

const SECRET: &str = "SECRET-LEAF-VALUE-5a91";

#[test]
fn new_rejects_every_out_of_range_limit_with_guidance() {
    let cases = [
        (
            InspectLimits::new(33, 100, 3, 16384),
            "max_depth must be a whole number from 0 to 32 (default 3); got 33",
        ),
        (
            InspectLimits::new(usize::MAX, 100, 3, 16384),
            "max_depth must be a whole number from 0 to 32",
        ),
        (
            InspectLimits::new(3, 0, 3, 16384),
            "max_paths must be a whole number from 1 to 1000 (default 100); got 0",
        ),
        (
            InspectLimits::new(3, 1001, 3, 16384),
            "max_paths must be a whole number from 1 to 1000 (default 100); got 1001",
        ),
        (
            InspectLimits::new(3, 100, 101, 16384),
            "array_sample must be a whole number from 0 to 100 (default 3); got 101",
        ),
        (
            InspectLimits::new(3, 100, 3, 0),
            "max_bytes must be a whole number from 512 to 65536 (default 16384); got 0",
        ),
        (
            InspectLimits::new(3, 100, 3, 511),
            "max_bytes must be a whole number from 512 to 65536 (default 16384); got 511",
        ),
        (
            InspectLimits::new(3, 100, 3, 65537),
            "max_bytes must be a whole number from 512 to 65536 (default 16384); got 65537",
        ),
    ];
    for (result, expected) in cases {
        let err = result.unwrap_err();
        assert!(matches!(err, EngineError::InvalidLimit(_)), "{err:?}");
        let text = err.to_string();
        assert!(text.contains(expected), "{text}");
        assert!(
            text.contains("Pass a value in that range or omit"),
            "{text}"
        );
    }
}

#[test]
fn new_accepts_range_edges() {
    let low = InspectLimits::new(0, 1, 0, 512).unwrap();
    assert_eq!(
        (
            low.max_depth(),
            low.max_paths(),
            low.array_sample(),
            low.max_bytes()
        ),
        (0, 1, 0, 512)
    );
    let high = InspectLimits::new(32, 1000, 100, 65536).unwrap();
    assert_eq!(
        (
            high.max_depth(),
            high.max_paths(),
            high.array_sample(),
            high.max_bytes()
        ),
        (32, 1000, 100, 65536)
    );
}

#[test]
fn minimum_limits_on_nonempty_documents_do_not_panic() {
    let limits = InspectLimits::new(0, 1, 0, 512).unwrap();
    for doc in [json!({"a": SECRET}), json!([SECRET]), json!(SECRET)] {
        let report = inspect_value(&doc, &limits, 0).unwrap();
        assert_eq!(report.returned_paths, 1);
        assert!(!serde_json::to_string(&report).unwrap().contains(SECRET));
    }
    let limits = InspectLimits::new(3, 1, 3, 512).unwrap();
    let report = inspect_value(&json!({"a": 1}), &limits, 0).unwrap();
    assert!(!report.complete);
    assert!(report.omitted.paths);

    let err = inspect_value(&json!({"a": 1}), &limits, usize::MAX).unwrap_err();
    assert!(err.to_string().contains("Increase max_bytes"), "{err}");
}

#[test]
fn malformed_json_input_gets_corrective_guidance() {
    let engine = JpxEngine::new();
    for input in [
        format!("{{\"k\": \"{SECRET}\""),
        "not json".to_string(),
        String::new(),
    ] {
        let err = engine
            .inspect(&input, &InspectLimits::default(), 0)
            .unwrap_err();
        assert!(matches!(err, EngineError::InvalidJson(_)), "{err:?}");
        let text = err.to_string();
        assert!(text.contains("line 1 column"), "{text}");
        assert!(
            text.contains("Pass the document as valid JSON text"),
            "{text}"
        );
        assert!(!text.contains(SECRET), "{text}");
    }
}
