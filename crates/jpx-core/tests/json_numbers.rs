mod support {
    pub mod json_numbers;
}

#[test]
fn json_literals_are_correctly_rounded() {
    support::json_numbers::check_json_numbers(|input| {
        jpx_core::compile(&format!("`{input}`"))
            .unwrap()
            .search(&serde_json::Value::Null)
            .unwrap()
    });
}
