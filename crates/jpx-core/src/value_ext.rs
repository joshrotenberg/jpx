//! Extension trait for `serde_json::Value` providing JMESPath operations.
//!
//! This replaces the 2,719-line `variable.rs` in jmespath.rs with ~200 lines
//! by leveraging `Value`'s built-in API.

use serde_json::Value;

use crate::ast::Comparator;

/// JMESPath type names used in error messages and the `type()` function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JmespathType {
    Null,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Expref,
}

impl std::fmt::Display for JmespathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JmespathType::Null => write!(f, "null"),
            JmespathType::String => write!(f, "string"),
            JmespathType::Number => write!(f, "number"),
            JmespathType::Boolean => write!(f, "boolean"),
            JmespathType::Array => write!(f, "array"),
            JmespathType::Object => write!(f, "object"),
            JmespathType::Expref => write!(f, "expref"),
        }
    }
}

/// Extension trait providing JMESPath operations on `serde_json::Value`.
pub trait ValueExt {
    /// Returns the JMESPath type name.
    fn jmespath_type(&self) -> JmespathType;

    /// Returns true if the value is "truthy" per the JMESPath spec.
    ///
    /// - `null` is falsy
    /// - `false` is falsy
    /// - `""` (empty string) is falsy
    /// - `[]` (empty array) is falsy
    /// - `{}` (empty object) is falsy
    /// - Everything else is truthy
    fn is_truthy(&self) -> bool;

    /// Extracts a named field from an object; returns `Value::Null` if not found.
    fn get_field(&self, name: &str) -> Value;

    /// Extracts an element by positive index; returns `Value::Null` if out of range.
    fn get_index(&self, idx: usize) -> Value;

    /// Extracts an element by negative index (counting from end).
    fn get_negative_index(&self, idx: usize) -> Value;

    /// Borrows a named field from an object; `None` if absent or not an object.
    ///
    /// The borrowing counterpart to [`ValueExt::get_field`]. The interpreter uses
    /// these `_ref` accessors so that selecting a field does not deep-copy the
    /// subtree it selects; the copy happens once, at the end, on the result.
    fn get_field_ref(&self, name: &str) -> Option<&Value>;

    /// Borrows an element by positive index; `None` if out of range or not an array.
    fn get_index_ref(&self, idx: usize) -> Option<&Value>;

    /// Borrows an element by negative index (counting from end).
    fn get_negative_index_ref(&self, idx: usize) -> Option<&Value>;

    /// Slices an array with start/stop/step, per the JMESPath spec.
    /// Returns `None` if the value is not an array.
    fn slice(&self, start: Option<i32>, stop: Option<i32>, step: i32) -> Option<Vec<Value>>;

    /// Compares two values using a JMESPath comparator.
    /// Returns `None` if the comparison is not valid for these types.
    fn compare(&self, comparator: &Comparator, other: &Value) -> Option<bool>;

    /// Returns `true` if this is an expref sentinel.
    fn is_expref(&self) -> bool;
}

impl ValueExt for Value {
    fn jmespath_type(&self) -> JmespathType {
        if self.is_expref() {
            return JmespathType::Expref;
        }
        match self {
            Value::Null => JmespathType::Null,
            Value::Bool(_) => JmespathType::Boolean,
            Value::Number(_) => JmespathType::Number,
            Value::String(_) => JmespathType::String,
            Value::Array(_) => JmespathType::Array,
            Value::Object(_) => JmespathType::Object,
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Number(_) => true,
        }
    }

    fn get_field(&self, name: &str) -> Value {
        self.get_field_ref(name).cloned().unwrap_or(Value::Null)
    }

    fn get_index(&self, idx: usize) -> Value {
        self.get_index_ref(idx).cloned().unwrap_or(Value::Null)
    }

    fn get_negative_index(&self, idx: usize) -> Value {
        self.get_negative_index_ref(idx)
            .cloned()
            .unwrap_or(Value::Null)
    }

    fn get_field_ref(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(map) => map.get(name),
            _ => None,
        }
    }

    fn get_index_ref(&self, idx: usize) -> Option<&Value> {
        match self {
            Value::Array(arr) => arr.get(idx),
            _ => None,
        }
    }

    fn get_negative_index_ref(&self, idx: usize) -> Option<&Value> {
        match self {
            Value::Array(arr) => {
                if idx > arr.len() {
                    None
                } else {
                    arr.get(arr.len() - idx)
                }
            }
            _ => None,
        }
    }

    fn slice(&self, start: Option<i32>, stop: Option<i32>, step: i32) -> Option<Vec<Value>> {
        let arr = self.as_array()?;
        // Slice endpoints are i32 (matching the AST); clamp rather than letting
        // `arr.len() as i32` wrap negative for arrays larger than i32::MAX (which
        // would then panic on `arr[i as usize]`). Such arrays are unrealistic but
        // the cast must not be unchecked.
        let len = i32::try_from(arr.len()).unwrap_or(i32::MAX);
        if len == 0 {
            return Some(vec![]);
        }

        let a: i32 = match start {
            Some(s) => adjust_slice_endpoint(len, s, step),
            _ if step < 0 => len - 1,
            _ => 0,
        };
        let b: i32 = match stop {
            Some(s) => adjust_slice_endpoint(len, s, step),
            _ if step < 0 => -1,
            _ => len,
        };

        let mut result = Vec::new();
        let mut i = a;
        if step > 0 {
            while i < b {
                result.push(arr[i as usize].clone());
                i += step;
            }
        } else {
            while i > b {
                result.push(arr[i as usize].clone());
                i += step;
            }
        }
        Some(result)
    }

    fn compare(&self, comparator: &Comparator, other: &Value) -> Option<bool> {
        match comparator {
            Comparator::Equal => Some(values_equal(self, other)),
            Comparator::NotEqual => Some(!values_equal(self, other)),
            Comparator::LessThan => compare_ordered(self, other).map(|o| o.is_lt()),
            Comparator::LessThanEqual => compare_ordered(self, other).map(|o| o.is_le()),
            Comparator::GreaterThan => compare_ordered(self, other).map(|o| o.is_gt()),
            Comparator::GreaterThanEqual => compare_ordered(self, other).map(|o| o.is_ge()),
        }
    }

    fn is_expref(&self) -> bool {
        matches!(self, Value::Object(map) if map.contains_key(crate::EXPREF_KEY.as_str()))
    }
}

/// Adjusts a slice endpoint per the JMESPath spec, accounting for step direction.
fn adjust_slice_endpoint(len: i32, mut endpoint: i32, step: i32) -> i32 {
    if endpoint < 0 {
        endpoint += len;
        if endpoint >= 0 {
            endpoint
        } else if step < 0 {
            -1
        } else {
            0
        }
    } else if endpoint < len {
        endpoint
    } else if step < 0 {
        len - 1
    } else {
        len
    }
}

/// A JSON number widened to a form that can be compared without loss.
///
/// Every integer `serde_json` stores (`i64` or `u64`) fits exactly in `i128`;
/// everything else is an `f64`.
#[derive(Clone, Copy)]
enum ExactNumber {
    Int(i128),
    Float(f64),
}

impl ExactNumber {
    fn from_number(n: &serde_json::Number) -> Option<Self> {
        if let Some(i) = n.as_i64() {
            Some(ExactNumber::Int(i128::from(i)))
        } else if let Some(u) = n.as_u64() {
            Some(ExactNumber::Int(i128::from(u)))
        } else {
            n.as_f64().map(ExactNumber::Float)
        }
    }
}

/// Compares an integer to a float by mathematical value, without rounding the
/// integer to `f64`. Returns `None` only when `f` is NaN.
fn compare_int_float(i: i128, f: f64) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if f.is_nan() {
        return None;
    }
    // 2^127 is exactly representable; anything at or beyond it (including
    // infinities) lies outside the i128 range, so the integer is on one side.
    const TWO_POW_127: f64 = (1u128 << 127) as f64;
    if f >= TWO_POW_127 {
        return Some(Ordering::Less);
    }
    if f < -TWO_POW_127 {
        return Some(Ordering::Greater);
    }
    // `trunc` is exact and now within [-2^127, 2^127), so the cast is exact
    // (no saturation). `-0.0` truncates to 0.
    let whole = f.trunc();
    match i.cmp(&(whole as i128)) {
        Ordering::Equal => {
            // Same integer part: the fractional part (exact) decides.
            let frac = f - whole;
            Some(if frac > 0.0 {
                Ordering::Less
            } else if frac < 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            })
        }
        other => Some(other),
    }
}

/// Compares two JSON numbers by exact mathematical value.
///
/// Contract:
/// - integer vs integer compares exactly across the full `i64` and `u64` ranges
///   (e.g. 2^53 and 2^53 + 1 are distinct; `u64::MAX - 1 < u64::MAX`);
/// - integer vs float compares the integer's true value against the float's
///   true value, so `1 == 1.0`, `0 == -0.0`, and `2^53 + 1 > 9007199254740992.0`;
/// - float vs float uses IEEE ordering (`-0.0 == 0.0`).
///
/// Returns `None` only if a NaN is involved, which JSON input cannot produce.
pub(crate) fn compare_numbers(
    a: &serde_json::Number,
    b: &serde_json::Number,
) -> Option<std::cmp::Ordering> {
    match (ExactNumber::from_number(a)?, ExactNumber::from_number(b)?) {
        (ExactNumber::Int(x), ExactNumber::Int(y)) => Some(x.cmp(&y)),
        (ExactNumber::Int(x), ExactNumber::Float(y)) => compare_int_float(x, y),
        (ExactNumber::Float(x), ExactNumber::Int(y)) => {
            compare_int_float(y, x).map(std::cmp::Ordering::reverse)
        }
        (ExactNumber::Float(x), ExactNumber::Float(y)) => x.partial_cmp(&y),
    }
}

/// Equality comparison per JMESPath spec.
pub(crate) fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => compare_numbers(a, b).is_some_and(|o| o.is_eq()),
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|bv| values_equal(v, bv)))
        }
        _ => false,
    }
}

/// Ordering comparison per JMESPath spec - only numbers and strings.
fn compare_ordered(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => compare_numbers(a, b),
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_truthy() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::String("".into()).is_truthy());
        assert!(Value::String("hi".into()).is_truthy());
        assert!(!json!([]).is_truthy());
        assert!(json!([1]).is_truthy());
        assert!(!json!({}).is_truthy());
        assert!(json!({"a": 1}).is_truthy());
        assert!(json!(0).is_truthy());
        assert!(json!(42).is_truthy());
    }

    #[test]
    fn test_get_field() {
        let obj = json!({"foo": "bar"});
        assert_eq!(obj.get_field("foo"), json!("bar"));
        assert_eq!(obj.get_field("missing"), Value::Null);
        assert_eq!(Value::Null.get_field("x"), Value::Null);
    }

    #[test]
    fn test_get_index() {
        let arr = json!([10, 20, 30]);
        assert_eq!(arr.get_index(0), json!(10));
        assert_eq!(arr.get_index(2), json!(30));
        assert_eq!(arr.get_index(5), Value::Null);
    }

    #[test]
    fn test_get_negative_index() {
        let arr = json!([10, 20, 30]);
        assert_eq!(arr.get_negative_index(1), json!(30));
        assert_eq!(arr.get_negative_index(3), json!(10));
        assert_eq!(arr.get_negative_index(4), Value::Null);
    }

    #[test]
    fn test_slice() {
        let arr = json!([0, 1, 2, 3, 4, 5]);
        assert_eq!(
            arr.slice(Some(1), Some(4), 1),
            Some(vec![json!(1), json!(2), json!(3)])
        );
        assert_eq!(
            arr.slice(None, None, 2),
            Some(vec![json!(0), json!(2), json!(4)])
        );
        assert_eq!(
            arr.slice(None, None, -1),
            Some(vec![
                json!(5),
                json!(4),
                json!(3),
                json!(2),
                json!(1),
                json!(0)
            ])
        );
    }

    #[test]
    fn test_compare() {
        assert_eq!(json!(1).compare(&Comparator::Equal, &json!(1)), Some(true));
        assert_eq!(
            json!(1).compare(&Comparator::LessThan, &json!(2)),
            Some(true)
        );
        assert_eq!(
            json!("a").compare(&Comparator::GreaterThan, &json!("b")),
            Some(false)
        );
        // Mixed types return None for ordering
        assert_eq!(json!(1).compare(&Comparator::LessThan, &json!("a")), None);
        // But equal/notequal work across types
        assert_eq!(
            json!(1).compare(&Comparator::Equal, &json!("a")),
            Some(false)
        );
    }

    /// Parses a number from exact JSON text (never via a rounded f64).
    fn num(text: &str) -> serde_json::Number {
        serde_json::from_str(text).expect("valid JSON number")
    }

    /// Asserts `a <ord> b` and the mirrored `b <reverse ord> a`.
    fn assert_cmp(a: &str, b: &str, expected: std::cmp::Ordering) {
        assert_eq!(
            compare_numbers(&num(a), &num(b)),
            Some(expected),
            "{a} vs {b}"
        );
        assert_eq!(
            compare_numbers(&num(b), &num(a)),
            Some(expected.reverse()),
            "{b} vs {a}"
        );
    }

    #[test]
    fn test_compare_numbers_exact_integers() {
        use std::cmp::Ordering::*;
        // Around 2^53, where f64 stops representing every integer.
        assert_cmp("9007199254740992", "9007199254740993", Less);
        assert_cmp("-9007199254740992", "-9007199254740993", Greater);
        assert_cmp("9007199254740993", "9007199254740993", Equal);
        // i64 limits, and the i64/u64 boundary.
        assert_cmp("9223372036854775806", "9223372036854775807", Less);
        assert_cmp("9223372036854775807", "9223372036854775808", Less);
        assert_cmp("-9223372036854775808", "-9223372036854775807", Less);
        // u64 limits.
        assert_cmp("18446744073709551614", "18446744073709551615", Less);
        // Negative signed vs large unsigned.
        assert_cmp("-1", "18446744073709551615", Less);
        assert_cmp("-9223372036854775808", "18446744073709551615", Less);
    }

    #[test]
    fn test_compare_numbers_mixed_int_float() {
        use std::cmp::Ordering::*;
        assert_cmp("1", "1.0", Equal);
        assert_cmp("-1", "-1.0", Equal);
        assert_cmp("0", "-0.0", Equal);
        assert_cmp("0", "0.0", Equal);
        assert_cmp("1", "1.5", Less);
        assert_cmp("2", "1.5", Greater);
        assert_cmp("-1", "-0.5", Less);
        assert_cmp("-1", "-1.5", Greater);
        // 9007199254740992.0 is exactly 2^53; the integer above it must not
        // round down to it.
        assert_cmp("9007199254740992", "9007199254740992.0", Equal);
        assert_cmp("9007199254740993", "9007199254740992.0", Greater);
        assert_cmp("9007199254740991", "9007199254740992.0", Less);
        // 2^64 as a float: a saturating `as u64` cast would wrongly make this Equal.
        assert_cmp("18446744073709551615", "18446744073709551616.0", Less);
        // -2^63 as a float is exactly i64::MIN.
        assert_cmp("-9223372036854775808", "-9223372036854775808.0", Equal);
        assert_cmp("-9223372036854775807", "-9223372036854775808.0", Greater);
        // Far outside every integer range.
        assert_cmp("18446744073709551615", "1e300", Less);
        assert_cmp("-9223372036854775808", "-1e300", Greater);
    }

    #[test]
    fn test_compare_numbers_floats() {
        use std::cmp::Ordering::*;
        assert_cmp("1.25", "1.5", Less);
        assert_cmp("-0.0", "0.0", Equal);
    }

    #[test]
    fn test_values_equal_exact_nested() {
        let a: Value = serde_json::from_str(r#"{"ids":[9007199254740992]}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"ids":[9007199254740993]}"#).unwrap();
        let c: Value = serde_json::from_str(r#"{"ids":[9007199254740992.0]}"#).unwrap();
        assert!(!values_equal(&a, &b));
        assert!(!values_equal(&b, &a));
        assert!(values_equal(&a, &c));
        assert!(values_equal(&c, &a));
    }

    #[test]
    fn test_jmespath_type() {
        assert_eq!(Value::Null.jmespath_type(), JmespathType::Null);
        assert_eq!(json!(true).jmespath_type(), JmespathType::Boolean);
        assert_eq!(json!(42).jmespath_type(), JmespathType::Number);
        assert_eq!(json!("s").jmespath_type(), JmespathType::String);
        assert_eq!(json!([]).jmespath_type(), JmespathType::Array);
        assert_eq!(json!({}).jmespath_type(), JmespathType::Object);
    }
}
