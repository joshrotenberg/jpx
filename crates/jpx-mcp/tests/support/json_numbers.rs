//! Shared regression cases for every JSON-text boundary. Expected bits are
//! independent of serde_json, so decoding CLI/MCP output cannot mask bad input.
use serde_json::Value;

pub fn check_json_numbers(mut parse: impl FnMut(&str) -> Value) {
    let cases = [
        ("0.9999999999999999", 0x3fef_ffff_ffff_ffff),
        ("1.0000000000000002", 0x3ff0_0000_0000_0001),
        // Halfway between adjacent binary64 integers: ties round to even.
        ("9007199254740993.0", 0x4340_0000_0000_0000),
        ("9007199254740995.0", 0x4340_0000_0000_0002),
        ("-9007199254740993.0", 0xc340_0000_0000_0000),
        ("5e-324", 0x0000_0000_0000_0001),
        ("2.2250738585072011e-308", 0x000f_ffff_ffff_ffff),
        ("2.2250738585072014e-308", 0x0010_0000_0000_0000),
    ];
    for (decimal, bits) in cases {
        let value = parse(decimal);
        assert!(value.is_f64(), "{decimal} must remain a float");
        assert_eq!(value.as_f64().unwrap().to_bits(), bits, "{decimal}");
        let encoded = serde_json::to_string(&value).unwrap();
        let roundtrip = parse(&encoded);
        assert_eq!(
            roundtrip.as_f64().unwrap().to_bits(),
            bits,
            "{decimal} round trip"
        );
    }
    for (decimal, expected) in [
        ("9007199254740993", Value::from(9_007_199_254_740_993_u64)),
        ("18446744073709551615", Value::from(u64::MAX)),
        ("-9223372036854775808", Value::from(i64::MIN)),
    ] {
        let value = parse(decimal);
        assert!(!value.is_f64(), "{decimal} must remain an integer");
        assert_eq!(value, expected);
        assert_eq!(parse(&serde_json::to_string(&value).unwrap()), expected);
    }
}
