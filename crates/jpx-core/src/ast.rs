//! JMESPath Abstract Syntax Tree.

use serde_json::Value;

/// Represents a JMESPath AST node.
#[derive(Clone, Debug, PartialEq)]
pub enum Ast {
    /// Compares two nodes using a comparator (e.g., `foo == bar`).
    Comparison {
        offset: usize,
        comparator: Comparator,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    /// Returns the RHS if the predicate yields a truthy value.
    Condition {
        offset: usize,
        predicate: Box<Ast>,
        then: Box<Ast>,
    },
    /// Returns the current node.
    Identity { offset: usize },
    /// Expression reference (used by `sort_by`, `max_by`, etc.).
    Expref { offset: usize, ast: Box<Ast> },
    /// Flattens nested arrays.
    Flatten { offset: usize, node: Box<Ast> },
    /// Calls a function with the given arguments.
    Function {
        offset: usize,
        name: String,
        args: Vec<Ast>,
    },
    /// Extracts a field from an object.
    Field { offset: usize, name: String },
    /// Accesses an element by index.
    Index { offset: usize, idx: i32 },
    /// A literal JSON value.
    Literal { offset: usize, value: Value },
    /// Multi-select list (e.g., `[foo, bar]`).
    MultiList { offset: usize, elements: Vec<Ast> },
    /// Multi-select hash (e.g., `{foo: bar, baz: qux}`).
    MultiHash {
        offset: usize,
        elements: Vec<KeyValuePair>,
    },
    /// Negation (e.g., `!expr`).
    Not { offset: usize, node: Box<Ast> },
    /// Projects the RHS over each element of the LHS array.
    Projection {
        offset: usize,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    /// Converts an object into an array of its values.
    ObjectValues { offset: usize, node: Box<Ast> },
    /// Logical AND.
    And {
        offset: usize,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    /// Logical OR.
    Or {
        offset: usize,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    /// Slices an array (e.g., `[0:5:2]`).
    Slice {
        offset: usize,
        start: Option<i32>,
        stop: Option<i32>,
        step: i32,
    },
    /// Sub-expression (e.g., `foo.bar`).
    Subexpr {
        offset: usize,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    /// JEP-18: Variable reference (e.g., `$name`).
    #[cfg(feature = "let-expr")]
    VariableRef { offset: usize, name: String },
    /// JEP-18: Let expression (e.g., `let $x = expr in body`).
    #[cfg(feature = "let-expr")]
    Let {
        offset: usize,
        bindings: Vec<(String, Ast)>,
        expr: Box<Ast>,
    },
}

/// A key-value pair used in multi-select hash expressions.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: Ast,
}

/// Comparison operators used in JMESPath expressions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Comparator {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
}
