#[path = "support/json_numbers.rs"]
mod json_numbers;

#[test]
fn engine_json_text_is_correctly_rounded() {
    let engine = jpx_engine::JpxEngine::new();
    json_numbers::check_json_numbers(|input| engine.evaluate_str("@", input).unwrap());
}
