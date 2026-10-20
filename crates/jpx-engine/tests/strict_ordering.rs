use jpx_engine::{EngineBuilder, JpxEngine};
use serde_json::json;

#[test]
fn direct_and_configured_engines_use_the_selected_comparison_mode() {
    for strict in [false, true] {
        for engine in [
            JpxEngine::with_options(strict),
            EngineBuilder::new().strict(strict).build().unwrap(),
        ] {
            let input = json!({"left": "a", "right": "b"});
            let expression = "[left < right, left <= right, left > right, left >= right]";
            assert!(engine.validate(expression).valid);
            assert_eq!(
                engine.evaluate(expression, &input).unwrap(),
                if strict {
                    json!([null, null, null, null])
                } else {
                    json!([true, true, false, false])
                }
            );
            assert_eq!(
                engine
                    .evaluate(expression, &json!({"left": 1, "right": 2}))
                    .unwrap(),
                json!([true, true, false, false])
            );
            let rows = json!([{"id":"string","a":"a","b":"b"},{"id":"number","a":1,"b":2}]);
            assert_eq!(
                engine.evaluate("[?a < b].id", &rows).unwrap(),
                if strict {
                    json!(["number"])
                } else {
                    json!(["string", "number"])
                }
            );
        }
    }
}
