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
        match self {
            Value::Object(map) => map.get(name).cloned().unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }

    fn get_index(&self, idx: usize) -> Value {
        match self {
            Value::Array(arr) => arr.get(idx).cloned().unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }

    fn get_negative_index(&self, idx: usize) -> Value {
        match self {
            Value::Array(arr) => {
                if idx > arr.len() {
                    Value::Null
                } else {
                    arr.get(arr.len() - idx).cloned().unwrap_or(Value::Null)
                }
            }
            _ => Value::Null,
        }
    }

    fn slice(&self, start: Option<i32>, stop: Option<i32>, step: i32) -> Option<Vec<Value>> {
        let arr = self.as_array()?;
        let len = arr.len() as i32;
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
        matches!(self, Value::Object(map) if map.contains_key("__jpx_expref__"))
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

/// Equality comparison per JMESPath spec.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // Compare as f64 to handle integer/float comparison
            a.as_f64().zip(b.as_f64()).is_some_and(|(af, bf)| af == bf)
        }
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
        (Value::Number(a), Value::Number(b)) => {
            let af = a.as_f64()?;
            let bf = b.as_f64()?;
            af.partial_cmp(&bf)
        }
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
