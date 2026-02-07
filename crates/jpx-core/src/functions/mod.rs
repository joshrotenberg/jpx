//! JMESPath functions.

use std::fmt;

use serde_json::{Number, Value};

use crate::ast::Ast;
use crate::interpreter::{SearchResult, interpret};
use crate::value_ext::{JmespathType, ValueExt};
use crate::{Context, ErrorReason, JmespathError, RuntimeError, get_expref_id};

/// Represents a JMESPath function.
pub trait Function: Sync + Send {
    /// Evaluates the function against a slice of Value arguments.
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult;
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

macro_rules! arg {
    (any) => (ArgumentType::Any);
    (null) => (ArgumentType::Null);
    (string) => (ArgumentType::String);
    (bool) => (ArgumentType::Bool);
    (number) => (ArgumentType::Number);
    (object) => (ArgumentType::Object);
    (expref) => (ArgumentType::Expref);
    (array_number) => (ArgumentType::TypedArray(Box::new(ArgumentType::Number)));
    (array_string) => (ArgumentType::TypedArray(Box::new(ArgumentType::String)));
    (array) => (ArgumentType::Array);
    ($($x:ident) | *) => (ArgumentType::Union(vec![$(arg!($x)), *]));
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
        self.validate_arity(args.len(), ctx)?;
        if let Some(ref variadic) = self.variadic {
            for (k, v) in args.iter().enumerate() {
                let validator = self.inputs.get(k).unwrap_or(variadic);
                self.validate_arg(ctx, k, v, validator)?;
            }
        } else {
            for (k, v) in args.iter().enumerate() {
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
fn number_value(n: f64) -> Value {
    Number::from_f64(n).map_or(Value::Null, Value::Number)
}

macro_rules! defn {
    ($name:ident, $args:expr, $variadic:expr) => {
        pub struct $name {
            signature: Signature,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    signature: Signature::new($args, $variadic),
                }
            }
        }
    };
}

// ---- 26 Built-in JMESPath Functions ----

defn!(AbsFn, vec![arg!(number)], None);

impl Function for AbsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.abs()))
    }
}

defn!(AvgFn, vec![arg!(array_number)], None);

impl Function for AvgFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let arr = args[0].as_array().unwrap();
        if arr.is_empty() {
            return Ok(Value::Null);
        }
        let sum: f64 = arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).sum();
        Ok(number_value(sum / arr.len() as f64))
    }
}

defn!(CeilFn, vec![arg!(number)], None);

impl Function for CeilFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.ceil()))
    }
}

defn!(ContainsFn, vec![arg!(string | array), arg!(any)], None);

impl Function for ContainsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        match &args[0] {
            Value::Array(arr) => Ok(Value::Bool(arr.contains(&args[1]))),
            Value::String(s) => match args[1].as_str() {
                Some(needle) => Ok(Value::Bool(s.contains(needle))),
                None => Ok(Value::Bool(false)),
            },
            _ => unreachable!(),
        }
    }
}

defn!(EndsWithFn, vec![arg!(string), arg!(string)], None);

impl Function for EndsWithFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let subject = args[0].as_str().unwrap();
        let search = args[1].as_str().unwrap();
        Ok(Value::Bool(subject.ends_with(search)))
    }
}

defn!(FloorFn, vec![arg!(number)], None);

impl Function for FloorFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.floor()))
    }
}

defn!(JoinFn, vec![arg!(string), arg!(array_string)], None);

impl Function for JoinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let glue = args[0].as_str().unwrap();
        let values = args[1].as_array().unwrap();
        let result: String = values
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect::<Vec<String>>()
            .join(glue);
        Ok(Value::String(result))
    }
}

defn!(KeysFn, vec![arg!(object)], None);

impl Function for KeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let obj = args[0].as_object().unwrap();
        let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
        Ok(Value::Array(keys))
    }
}

defn!(LengthFn, vec![arg!(array | object | string)], None);

impl Function for LengthFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let len = match &args[0] {
            Value::Array(a) => a.len(),
            Value::Object(m) => m.len(),
            Value::String(s) => s.chars().count(),
            _ => unreachable!(),
        };
        Ok(Value::Number(Number::from(len)))
    }
}

defn!(MapFn, vec![arg!(expref), arg!(array)], None);

impl Function for MapFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let ast = get_expref_ast(&args[0], ctx).ok_or_else(|| {
            JmespathError::new("", 0, ErrorReason::Parse("Expected expref".to_owned()))
        })?;
        // Clone the AST since we need to borrow ctx mutably
        let ast = ast.clone();
        let values = args[1].as_array().unwrap();
        let mut results = vec![];
        for value in values {
            results.push(interpret(value, &ast, ctx)?);
        }
        Ok(Value::Array(results))
    }
}

defn!(MaxFn, vec![arg!(array_string | array_number)], None);

impl Function for MaxFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let values = args[0].as_array().unwrap();
        if values.is_empty() {
            return Ok(Value::Null);
        }
        let result = values
            .iter()
            .skip(1)
            .fold(values[0].clone(), |acc, item| max_value(acc, item.clone()));
        Ok(result)
    }
}

defn!(MinFn, vec![arg!(array_string | array_number)], None);

impl Function for MinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let values = args[0].as_array().unwrap();
        if values.is_empty() {
            return Ok(Value::Null);
        }
        let result = values
            .iter()
            .skip(1)
            .fold(values[0].clone(), |acc, item| min_value(acc, item.clone()));
        Ok(result)
    }
}

defn!(MaxByFn, vec![arg!(array), arg!(expref)], None);

impl Function for MaxByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        min_max_by(ctx, &args[0], &args[1], |a, b| compare_values(a, b).is_gt())
    }
}

defn!(MinByFn, vec![arg!(array), arg!(expref)], None);

impl Function for MinByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        min_max_by(ctx, &args[0], &args[1], |a, b| compare_values(a, b).is_lt())
    }
}

defn!(MergeFn, vec![arg!(object)], Some(arg!(object)));

impl Function for MergeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let mut result = serde_json::Map::new();
        for arg in args {
            if let Some(obj) = arg.as_object() {
                result.extend(obj.clone());
            }
        }
        Ok(Value::Object(result))
    }
}

defn!(NotNullFn, vec![arg!(any)], Some(arg!(any)));

impl Function for NotNullFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        for arg in args {
            if !arg.is_null() {
                return Ok(arg.clone());
            }
        }
        Ok(Value::Null)
    }
}

defn!(ReverseFn, vec![arg!(array | string)], None);

impl Function for ReverseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        match &args[0] {
            Value::Array(arr) => {
                let mut reversed = arr.clone();
                reversed.reverse();
                Ok(Value::Array(reversed))
            }
            Value::String(s) => {
                let reversed: String = s.chars().rev().collect();
                Ok(Value::String(reversed))
            }
            _ => unreachable!(),
        }
    }
}

defn!(SortFn, vec![arg!(array_string | array_number)], None);

impl Function for SortFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let mut values = args[0].as_array().unwrap().clone();
        values.sort_by(compare_values);
        Ok(Value::Array(values))
    }
}

defn!(SortByFn, vec![arg!(array), arg!(expref)], None);

impl Function for SortByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let vals = args[0].as_array().unwrap();
        if vals.is_empty() {
            return Ok(Value::Array(vec![]));
        }
        let ast = get_expref_ast(&args[1], ctx)
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

        let mut mapped: Vec<(Value, Value)> = vec![(vals[0].clone(), first_value)];
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
            mapped.push((v.clone(), mapped_value));
        }
        mapped.sort_by(|a, b| compare_values(&a.1, &b.1));
        let result = mapped.into_iter().map(|(orig, _)| orig).collect();
        Ok(Value::Array(result))
    }
}

defn!(StartsWithFn, vec![arg!(string), arg!(string)], None);

impl Function for StartsWithFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let subject = args[0].as_str().unwrap();
        let search = args[1].as_str().unwrap();
        Ok(Value::Bool(subject.starts_with(search)))
    }
}

defn!(SumFn, vec![arg!(array_number)], None);

impl Function for SumFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let sum: f64 = args[0]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0))
            .sum();
        Ok(number_value(sum))
    }
}

defn!(ToArrayFn, vec![arg!(any)], None);

impl Function for ToArrayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        match &args[0] {
            Value::Array(_) => Ok(args[0].clone()),
            _ => Ok(Value::Array(vec![args[0].clone()])),
        }
    }
}

defn!(ToNumberFn, vec![arg!(any)], None);

impl Function for ToNumberFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        match &args[0] {
            Value::Number(_) => Ok(args[0].clone()),
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

impl Function for ToStringFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        match &args[0] {
            Value::String(_) => Ok(args[0].clone()),
            other => Ok(Value::String(other.to_string())),
        }
    }
}

defn!(TypeFn, vec![arg!(any)], None);

impl Function for TypeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(Value::String(args[0].jmespath_type().to_string()))
    }
}

defn!(ValuesFn, vec![arg!(object)], None);

impl Function for ValuesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let map = args[0].as_object().unwrap();
        Ok(Value::Array(map.values().cloned().collect()))
    }
}

// ---- Helper functions ----

/// Compare two Values for sorting. Numbers compare numerically, strings lexicographically.
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let af = a.as_f64().unwrap_or(0.0);
            let bf = b.as_f64().unwrap_or(0.0);
            af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::String(a), Value::String(b)) => a.cmp(b),
        _ => std::cmp::Ordering::Equal,
    }
}

fn max_value(a: Value, b: Value) -> Value {
    if compare_values(&a, &b).is_ge() { a } else { b }
}

fn min_value(a: Value, b: Value) -> Value {
    if compare_values(&a, &b).is_le() { a } else { b }
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

    let mut candidate = (vals[0].clone(), initial);
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
            candidate = (v.clone(), mapped);
        }
    }
    Ok(candidate.0)
}
