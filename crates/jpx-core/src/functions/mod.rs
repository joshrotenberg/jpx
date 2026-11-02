//! JMESPath functions.

use std::{
    borrow::{Borrow, Cow},
    fmt,
};

use serde_json::{Number, Value};

use crate::ast::Ast;
use crate::interpreter::{SearchResult, interpret};
use crate::value_ext::{JmespathType, ValueExt, compare_numbers, values_equal};
use crate::{Context, ErrorReason, JmespathError, RuntimeError, get_expref_id};

/// Represents a JMESPath function.
pub trait Function: Sync + Send {
    /// Evaluates the function against a slice of Value arguments.
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult;
}

/// Internal opt-in for functions that can inspect arguments without copying them.
/// The public `Function` interface remains available for direct callers.
pub(crate) trait BorrowedFunction: Function {
    fn evaluate_borrowed(&self, args: &[Cow<'_, Value>], ctx: &mut Context<'_>) -> SearchResult;
}

/// Function argument types used when validating.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArgumentType {
    Any,
    Null,
    String,
    Number,
    Bool,
    Object,
    Array,
    Expref,
    /// Each element of the array must match the provided type.
    TypedArray(Box<ArgumentType>),
    /// Accepts one of a number of `ArgumentType`s.
    Union(Vec<ArgumentType>),
}

impl ArgumentType {
    /// Returns true/false if the value is valid for the type.
    pub fn is_valid(&self, value: &Value) -> bool {
        use self::ArgumentType::*;
        match *self {
            Any => true,
            Null if value.is_null() => true,
            String if value.is_string() => true,
            Number if value.is_number() => true,
            Object if value.is_object() && !value.is_expref() => true,
            Bool if value.is_boolean() => true,
            Expref if value.is_expref() => true,
            Array if value.is_array() => true,
            TypedArray(ref t) if value.is_array() => {
                if let Some(array) = value.as_array() {
                    array.iter().all(|v| t.is_valid(v))
                } else {
                    false
                }
            }
            Union(ref types) => types.iter().any(|t| t.is_valid(value)),
            _ => false,
        }
    }
}

impl fmt::Display for ArgumentType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use self::ArgumentType::*;
        match *self {
            Any => write!(fmt, "any"),
            String => write!(fmt, "string"),
            Number => write!(fmt, "number"),
            Bool => write!(fmt, "boolean"),
            Array => write!(fmt, "array"),
            Object => write!(fmt, "object"),
            Null => write!(fmt, "null"),
            Expref => write!(fmt, "expref"),
            TypedArray(ref t) => write!(fmt, "array[{t}]"),
            Union(ref types) => {
                let str_value = types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join("|");
                write!(fmt, "{str_value}")
            }
        }
    }
}

#[macro_export]
macro_rules! arg {
    (any) => ($crate::functions::ArgumentType::Any);
    (null) => ($crate::functions::ArgumentType::Null);
    (string) => ($crate::functions::ArgumentType::String);
    (bool) => ($crate::functions::ArgumentType::Bool);
    (number) => ($crate::functions::ArgumentType::Number);
    (object) => ($crate::functions::ArgumentType::Object);
    (expref) => ($crate::functions::ArgumentType::Expref);
    (array_number) => ($crate::functions::ArgumentType::TypedArray(Box::new($crate::functions::ArgumentType::Number)));
    (array_string) => ($crate::functions::ArgumentType::TypedArray(Box::new($crate::functions::ArgumentType::String)));
    (array) => ($crate::functions::ArgumentType::Array);
    ($($x:ident) | *) => ($crate::functions::ArgumentType::Union(vec![$($crate::arg!($x)), *]));
}

type InvokedFunction = dyn Fn(&[Value], &mut Context<'_>) -> SearchResult + Sync + Send;

/// Custom function that allows the creation of runtime functions with signature validation.
pub struct CustomFunction {
    signature: Signature,
    f: Box<InvokedFunction>,
}

impl CustomFunction {
    /// Creates a new custom function.
    pub fn new(fn_signature: Signature, f: Box<InvokedFunction>) -> CustomFunction {
        CustomFunction {
            signature: fn_signature,
            f,
        }
    }
}

impl Function for CustomFunction {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        (self.f)(args, ctx)
    }
}

/// Normal closures can be used as functions.
impl<F> Function for F
where
    F: Send + Sync + Fn(&[Value], &mut Context<'_>) -> SearchResult,
{
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        (self)(args, ctx)
    }
}

/// Represents a function's signature.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Signature {
    pub inputs: Vec<ArgumentType>,
    pub variadic: Option<ArgumentType>,
}

impl Signature {
    /// Creates a new Signature struct.
    pub fn new(inputs: Vec<ArgumentType>, variadic: Option<ArgumentType>) -> Signature {
        Signature { inputs, variadic }
    }

    /// Validates the arity of a function.
    pub fn validate_arity(&self, actual: usize, ctx: &Context<'_>) -> Result<(), JmespathError> {
        let expected = self.inputs.len();
        if self.variadic.is_some() {
            if actual >= expected {
                Ok(())
            } else {
                let reason =
                    ErrorReason::Runtime(RuntimeError::NotEnoughArguments { expected, actual });
                Err(JmespathError::from_ctx(ctx, reason))
            }
        } else if actual == expected {
            Ok(())
        } else if actual < expected {
            let reason =
                ErrorReason::Runtime(RuntimeError::NotEnoughArguments { expected, actual });
            Err(JmespathError::from_ctx(ctx, reason))
        } else {
            let reason = ErrorReason::Runtime(RuntimeError::TooManyArguments { expected, actual });
            Err(JmespathError::from_ctx(ctx, reason))
        }
    }

    /// Validates the provided function arguments against the signature.
    pub fn validate(&self, args: &[Value], ctx: &Context<'_>) -> Result<(), JmespathError> {
        self.validate_values(args.iter(), ctx)
    }

    pub(crate) fn validate_values<'a>(
        &self,
        args: impl ExactSizeIterator<Item = &'a Value>,
        ctx: &Context<'_>,
    ) -> Result<(), JmespathError> {
        self.validate_arity(args.len(), ctx)?;
        if let Some(ref variadic) = self.variadic {
            for (k, v) in args.enumerate() {
                let validator = self.inputs.get(k).unwrap_or(variadic);
                self.validate_arg(ctx, k, v, validator)?;
            }
        } else {
            for (k, v) in args.enumerate() {
                self.validate_arg(ctx, k, v, &self.inputs[k])?;
            }
        }
        Ok(())
    }

    fn validate_arg(
        &self,
        ctx: &Context<'_>,
        position: usize,
        value: &Value,
        validator: &ArgumentType,
    ) -> Result<(), JmespathError> {
        if validator.is_valid(value) {
            Ok(())
        } else {
            let reason = ErrorReason::Runtime(RuntimeError::InvalidType {
                expected: validator.to_string(),
                actual: value.jmespath_type().to_string(),
                position,
            });
            Err(JmespathError::from_ctx(ctx, reason))
        }
    }
}

/// Helper to extract an expref AST from a function argument.
fn get_expref_ast<'a>(value: &Value, ctx: &'a Context<'_>) -> Option<&'a Ast> {
    get_expref_id(value).and_then(|id| ctx.get_expref(id))
}

/// Helper to create a number Value from f64, returning Null on NaN/Inf.
pub fn number_value(n: f64) -> Value {
    Number::from_f64(n).map_or(Value::Null, Value::Number)
}

/// Applies an integral rounding operation (`ceil`, `floor`, `round`). Integer
/// input is already integral and is returned unchanged, without an `f64`
/// round trip that would lose precision on large values.
pub(crate) fn rounded_value(value: &Value, op: fn(f64) -> f64) -> Value {
    if exact_int(value).is_some() {
        return value.clone();
    }
    integral_value(op(value.as_f64().unwrap_or(0.0)))
}

/// Helper to create a number Value from an f64 that is integral by
/// construction (`ceil`, `floor`, `round`). Whole values inside the `i64`
/// range become integers so they print without a trailing `.0`; anything else
/// falls back to [`number_value`].
pub(crate) fn integral_value(n: f64) -> Value {
    // -2^63 and 2^63 are exactly representable; `as i64` is exact in between.
    const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;
    if n.fract() == 0.0 && (-TWO_POW_63..TWO_POW_63).contains(&n) {
        Value::from(n as i64)
    } else {
        number_value(n)
    }
}

/// Returns the exact value of an integer-stored number. Floats (including
/// whole ones such as `5.0`) and non-numbers return `None`.
pub(crate) fn exact_int(value: &Value) -> Option<i128> {
    let n = value.as_number()?;
    n.as_i64()
        .map(i128::from)
        .or_else(|| n.as_u64().map(i128::from))
}

/// Builds an integer number from an exact result, or `None` when the result
/// fits neither `i64` nor `u64` and the caller should fall back to `f64`.
pub(crate) fn int_value(n: i128) -> Option<Value> {
    i64::try_from(n)
        .map(Value::from)
        .or_else(|_| u64::try_from(n).map(Value::from))
        .ok()
}

/// Sums numbers exactly when every one is integer-stored and the total fits
/// `i64` or `u64`. Returns `None` otherwise so the caller can sum as `f64`.
pub(crate) fn exact_int_sum<'a, I>(values: I) -> Option<Value>
where
    I: IntoIterator<Item = &'a Value>,
{
    let total = values
        .into_iter()
        .try_fold(0i128, |acc, v| acc.checked_add(exact_int(v)?))?;
    int_value(total)
}

/// Creates a JmespathError for a custom runtime error message.
pub fn custom_error(ctx: &Context<'_>, message: &str) -> JmespathError {
    JmespathError::from_ctx(ctx, ErrorReason::Parse(message.to_owned()))
}

/// Creates a JmespathError for an invalid argument type at a given position.
pub fn invalid_type_error(
    ctx: &Context<'_>,
    position: usize,
    expected: &str,
    actual: &Value,
) -> JmespathError {
    JmespathError::from_ctx(
        ctx,
        ErrorReason::Runtime(RuntimeError::InvalidType {
            expected: expected.to_owned(),
            actual: actual.jmespath_type().to_string(),
            position,
        }),
    )
}

#[macro_export]
macro_rules! defn {
    ($name:ident, $args:expr, $variadic:expr) => {
        pub struct $name {
            signature: $crate::functions::Signature,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    signature: $crate::functions::Signature::new($args, $variadic),
                }
            }
        }
    };
}

// ---- 26 Built-in JMESPath Functions ----
//
// Each built-in is written once, generic over `A: Borrow<Value>`, so the same
// body serves the public `Function::evaluate(&[Value])` and the interpreter's
// borrowed dispatch (`&[Cow<Value>]`) without copying arguments on either path.

/// Implements `Function` and `BorrowedFunction` for a function whose body is an
/// inherent `call<A: Borrow<Value>>(&self, args: &[A], ctx)` method. Arguments
/// are validated against the signature before `call` runs.
macro_rules! borrowed_function {
    ($name:ident) => {
        impl $crate::functions::Function for $name {
            fn evaluate(
                &self,
                args: &[serde_json::Value],
                ctx: &mut $crate::Context<'_>,
            ) -> $crate::interpreter::SearchResult {
                self.signature.validate(args, ctx)?;
                self.call(args, ctx)
            }
        }

        impl $crate::functions::BorrowedFunction for $name {
            fn evaluate_borrowed(
                &self,
                args: &[std::borrow::Cow<'_, serde_json::Value>],
                ctx: &mut $crate::Context<'_>,
            ) -> $crate::interpreter::SearchResult {
                self.signature
                    .validate_values(args.iter().map(std::borrow::Cow::as_ref), ctx)?;
                self.call(args, ctx)
            }
        }
    };
}
// Unused when no extension category is compiled in.
#[allow(unused_imports)]
pub(crate) use borrowed_function;

/// The `i`th argument as a reference, whichever way the arguments are held.
#[inline]
pub(crate) fn arg<A: Borrow<Value>>(args: &[A], i: usize) -> &Value {
    args[i].borrow()
}

defn!(AbsFn, vec![arg!(number)], None);
borrowed_function!(AbsFn);

impl AbsFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        Ok(abs_value(arg(args, 0)))
    }
}

/// Absolute value, kept as an integer for integer input.
pub(crate) fn abs_value(value: &Value) -> Value {
    if let Some(abs) = exact_int(value).and_then(|i| int_value(i.abs())) {
        return abs;
    }
    number_value(value.as_f64().unwrap_or(0.0).abs())
}

defn!(AvgFn, vec![arg!(array_number)], None);
borrowed_function!(AvgFn);

impl AvgFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let arr = arg(args, 0).as_array().unwrap();
        if arr.is_empty() {
            return Ok(Value::Null);
        }
        let sum: f64 = arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).sum();
        Ok(number_value(sum / arr.len() as f64))
    }
}

defn!(CeilFn, vec![arg!(number)], None);
borrowed_function!(CeilFn);

impl CeilFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        Ok(rounded_value(arg(args, 0), f64::ceil))
    }
}

defn!(ContainsFn, vec![arg!(string | array), arg!(any)], None);
borrowed_function!(ContainsFn);

impl ContainsFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let needle = arg(args, 1);
        match arg(args, 0) {
            Value::Array(arr) => Ok(Value::Bool(
                arr.iter().any(|item| values_equal(item, needle)),
            )),
            Value::String(s) => match needle.as_str() {
                Some(needle) => Ok(Value::Bool(s.contains(needle))),
                None => Ok(Value::Bool(false)),
            },
            _ => unreachable!(),
        }
    }
}

defn!(EndsWithFn, vec![arg!(string), arg!(string)], None);
borrowed_function!(EndsWithFn);

impl EndsWithFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let subject = arg(args, 0).as_str().unwrap();
        let search = arg(args, 1).as_str().unwrap();
        Ok(Value::Bool(subject.ends_with(search)))
    }
}

defn!(FloorFn, vec![arg!(number)], None);
borrowed_function!(FloorFn);

impl FloorFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        Ok(rounded_value(arg(args, 0), f64::floor))
    }
}

defn!(JoinFn, vec![arg!(string), arg!(array_string)], None);
borrowed_function!(JoinFn);

impl JoinFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let glue = arg(args, 0).as_str().unwrap();
        let values = arg(args, 1).as_array().unwrap();
        let result: String = values
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<&str>>()
            .join(glue);
        Ok(Value::String(result))
    }
}

defn!(KeysFn, vec![arg!(object)], None);
borrowed_function!(KeysFn);

impl KeysFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let obj = arg(args, 0).as_object().unwrap();
        let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
        Ok(Value::Array(keys))
    }
}

defn!(LengthFn, vec![arg!(array | object | string)], None);
borrowed_function!(LengthFn);

impl LengthFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let len = match arg(args, 0) {
            Value::Array(a) => a.len(),
            Value::Object(m) => m.len(),
            Value::String(s) => s.chars().count(),
            _ => unreachable!(),
        };
        Ok(Value::Number(Number::from(len)))
    }
}

defn!(MapFn, vec![arg!(expref), arg!(array)], None);
borrowed_function!(MapFn);

impl MapFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], ctx: &mut Context<'_>) -> SearchResult {
        let ast = get_expref_ast(arg(args, 0), ctx).ok_or_else(|| {
            JmespathError::new("", 0, ErrorReason::Parse("Expected expref".to_owned()))
        })?;
        // Clone the AST since we need to borrow ctx mutably
        let ast = ast.clone();
        let values = arg(args, 1).as_array().unwrap();
        let mut results = Vec::with_capacity(values.len());
        for value in values {
            results.push(interpret(value, &ast, ctx)?);
        }
        Ok(Value::Array(results))
    }
}

defn!(MaxFn, vec![arg!(array_string | array_number)], None);
borrowed_function!(MaxFn);

impl MaxFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let values = arg(args, 0).as_array().unwrap();
        Ok(fold_extreme(values, |best, item| {
            compare_values(best, item).is_ge()
        }))
    }
}

defn!(MinFn, vec![arg!(array_string | array_number)], None);
borrowed_function!(MinFn);

impl MinFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let values = arg(args, 0).as_array().unwrap();
        Ok(fold_extreme(values, |best, item| {
            compare_values(best, item).is_le()
        }))
    }
}

defn!(MaxByFn, vec![arg!(array), arg!(expref)], None);
borrowed_function!(MaxByFn);

impl MaxByFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], ctx: &mut Context<'_>) -> SearchResult {
        min_max_by(ctx, arg(args, 0), arg(args, 1), |a, b| {
            compare_values(a, b).is_gt()
        })
    }
}

defn!(MinByFn, vec![arg!(array), arg!(expref)], None);
borrowed_function!(MinByFn);

impl MinByFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], ctx: &mut Context<'_>) -> SearchResult {
        min_max_by(ctx, arg(args, 0), arg(args, 1), |a, b| {
            compare_values(a, b).is_lt()
        })
    }
}

defn!(MergeFn, vec![arg!(object)], Some(arg!(object)));
borrowed_function!(MergeFn);

impl MergeFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let mut result = serde_json::Map::new();
        for arg in args {
            if let Some(obj) = arg.borrow().as_object() {
                result.extend(obj.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }
        Ok(Value::Object(result))
    }
}

defn!(NotNullFn, vec![arg!(any)], Some(arg!(any)));
borrowed_function!(NotNullFn);

impl NotNullFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        Ok(args
            .iter()
            .map(Borrow::borrow)
            .find(|arg: &&Value| !arg.is_null())
            .cloned()
            .unwrap_or(Value::Null))
    }
}

defn!(ReverseFn, vec![arg!(array | string)], None);
borrowed_function!(ReverseFn);

impl ReverseFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        match arg(args, 0) {
            Value::Array(arr) => Ok(Value::Array(arr.iter().rev().cloned().collect())),
            Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
            _ => unreachable!(),
        }
    }
}

defn!(SortFn, vec![arg!(array_string | array_number)], None);
borrowed_function!(SortFn);

impl SortFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let mut values = arg(args, 0).as_array().unwrap().clone();
        values.sort_by(compare_values);
        Ok(Value::Array(values))
    }
}

defn!(SortByFn, vec![arg!(array), arg!(expref)], None);
borrowed_function!(SortByFn);

impl SortByFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], ctx: &mut Context<'_>) -> SearchResult {
        let vals = arg(args, 0).as_array().unwrap();
        if vals.is_empty() {
            return Ok(Value::Array(vec![]));
        }
        let ast = get_expref_ast(arg(args, 1), ctx)
            .ok_or_else(|| {
                JmespathError::new("", 0, ErrorReason::Parse("Expected expref".to_owned()))
            })?
            .clone();

        // Map each value to (original, mapped_key)
        let first_value = interpret(&vals[0], &ast, ctx)?;
        let first_type = first_value.jmespath_type();
        if first_type != JmespathType::String && first_type != JmespathType::Number {
            let reason = ErrorReason::Runtime(RuntimeError::InvalidReturnType {
                expected: "expression->string|expression->number".to_owned(),
                actual: first_type.to_string(),
                position: 1,
                invocation: 1,
            });
            return Err(JmespathError::from_ctx(ctx, reason));
        }

        // Sort references and copy each element once, into the result.
        let mut mapped: Vec<(&Value, Value)> = Vec::with_capacity(vals.len());
        mapped.push((&vals[0], first_value));
        for (invocation, v) in vals.iter().enumerate().skip(1) {
            let mapped_value = interpret(v, &ast, ctx)?;
            if mapped_value.jmespath_type() != first_type {
                return Err(JmespathError::from_ctx(
                    ctx,
                    ErrorReason::Runtime(RuntimeError::InvalidReturnType {
                        expected: format!("expression->{first_type}"),
                        actual: mapped_value.jmespath_type().to_string(),
                        position: 1,
                        invocation,
                    }),
                ));
            }
            mapped.push((v, mapped_value));
        }
        mapped.sort_by(|a, b| compare_values(&a.1, &b.1));
        let result = mapped.into_iter().map(|(orig, _)| orig.clone()).collect();
        Ok(Value::Array(result))
    }
}

defn!(StartsWithFn, vec![arg!(string), arg!(string)], None);
borrowed_function!(StartsWithFn);

impl StartsWithFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let subject = arg(args, 0).as_str().unwrap();
        let search = arg(args, 1).as_str().unwrap();
        Ok(Value::Bool(subject.starts_with(search)))
    }
}

defn!(SumFn, vec![arg!(array_number)], None);
borrowed_function!(SumFn);

impl SumFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let arr = arg(args, 0).as_array().unwrap();
        if let Some(sum) = exact_int_sum(arr) {
            return Ok(sum);
        }
        let sum: f64 = arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).sum();
        Ok(number_value(sum))
    }
}

defn!(ToArrayFn, vec![arg!(any)], None);
borrowed_function!(ToArrayFn);

impl ToArrayFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let value = arg(args, 0);
        match value {
            Value::Array(_) => Ok(value.clone()),
            _ => Ok(Value::Array(vec![value.clone()])),
        }
    }
}

defn!(ToNumberFn, vec![arg!(any)], None);
borrowed_function!(ToNumberFn);

impl ToNumberFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let value = arg(args, 0);
        match value {
            Value::Number(_) => Ok(value.clone()),
            Value::String(s) => match serde_json::from_str::<Value>(s) {
                Ok(Value::Number(n)) => Ok(Value::Number(n)),
                _ => Ok(Value::Null),
            },
            _ => Ok(Value::Null),
        }
    }
}

defn!(
    ToStringFn,
    vec![arg!(object | array | bool | number | string | null)],
    None
);
borrowed_function!(ToStringFn);

impl ToStringFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let value = arg(args, 0);
        match value {
            Value::String(_) => Ok(value.clone()),
            other => Ok(Value::String(other.to_string())),
        }
    }
}

defn!(TypeFn, vec![arg!(any)], None);
borrowed_function!(TypeFn);

impl TypeFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        Ok(Value::String(arg(args, 0).jmespath_type().to_string()))
    }
}

defn!(ValuesFn, vec![arg!(object)], None);
borrowed_function!(ValuesFn);

impl ValuesFn {
    fn call<A: Borrow<Value>>(&self, args: &[A], _ctx: &mut Context<'_>) -> SearchResult {
        let map = arg(args, 0).as_object().unwrap();
        Ok(Value::Array(map.values().cloned().collect()))
    }
}

// ---- Helper functions ----

/// Compare two Values for sorting. Numbers compare by exact numeric value
/// (see [`compare_numbers`]), strings lexicographically.
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            compare_numbers(a, b).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::String(a), Value::String(b)) => a.cmp(b),
        _ => std::cmp::Ordering::Equal,
    }
}

/// The element `keep` prefers over every later one, compared by reference and
/// copied once. `keep(best, item)` returns true to keep the current best, so
/// ties resolve to the earliest element, as the previous fold did.
fn fold_extreme(values: &[Value], keep: impl Fn(&Value, &Value) -> bool) -> Value {
    let mut iter = values.iter();
    let Some(mut best) = iter.next() else {
        return Value::Null;
    };
    for item in iter {
        if !keep(best, item) {
            best = item;
        }
    }
    best.clone()
}

/// Shared implementation for max_by and min_by.
fn min_max_by(
    ctx: &mut Context<'_>,
    array_arg: &Value,
    expref_arg: &Value,
    is_better: fn(&Value, &Value) -> bool,
) -> SearchResult {
    let vals = array_arg.as_array().ok_or_else(|| {
        JmespathError::new("", 0, ErrorReason::Parse("Expected array".to_owned()))
    })?;
    if vals.is_empty() {
        return Ok(Value::Null);
    }
    let ast = get_expref_ast(expref_arg, ctx)
        .ok_or_else(|| JmespathError::new("", 0, ErrorReason::Parse("Expected expref".to_owned())))?
        .clone();

    let initial = interpret(&vals[0], &ast, ctx)?;
    let entered_type = initial.jmespath_type();
    if entered_type != JmespathType::String && entered_type != JmespathType::Number {
        return Err(JmespathError::from_ctx(
            ctx,
            ErrorReason::Runtime(RuntimeError::InvalidReturnType {
                expected: "expression->number|expression->string".to_owned(),
                actual: entered_type.to_string(),
                position: 1,
                invocation: 1,
            }),
        ));
    }

    let mut candidate = (&vals[0], initial);
    for (invocation, v) in vals.iter().enumerate().skip(1) {
        let mapped = interpret(v, &ast, ctx)?;
        if mapped.jmespath_type() != entered_type {
            return Err(JmespathError::from_ctx(
                ctx,
                ErrorReason::Runtime(RuntimeError::InvalidReturnType {
                    expected: format!("expression->{entered_type}"),
                    actual: mapped.jmespath_type().to_string(),
                    position: 1,
                    invocation,
                }),
            ));
        }
        if is_better(&mapped, &candidate.1) {
            candidate = (v, mapped);
        }
    }
    Ok(candidate.0.clone())
}
