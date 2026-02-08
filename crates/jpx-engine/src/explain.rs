//! Expression explanation via AST analysis.
//!
//! Walks a parsed JMESPath AST and produces a structured breakdown
//! of each step in the expression, suitable for human or agent consumption.

use crate::JpxEngine;
use crate::error::{EngineError, Result};
use jpx_core::ast::{Ast, Comparator};
use serde::{Deserialize, Serialize};

/// Step-by-step breakdown of a JMESPath expression.
///
/// Returned by [`JpxEngine::explain`].
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let result = engine.explain("users[?age > `30`].name | sort(@)").unwrap();
///
/// assert!(!result.steps.is_empty());
/// assert!(result.functions_used.contains(&"sort".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    /// The original expression.
    pub expression: String,
    /// Ordered steps describing what the expression does.
    pub steps: Vec<ExplainStep>,
    /// All function names used in the expression.
    pub functions_used: Vec<String>,
    /// Rough complexity label: "simple", "moderate", or "complex".
    pub complexity: String,
}

/// A single step in the expression breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainStep {
    /// The AST node type (e.g. "field", "filter", "function").
    pub node_type: String,
    /// Human-readable description of what this step does.
    pub description: String,
    /// Nested children steps (for compound nodes).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ExplainStep>,
}

impl JpxEngine {
    /// Explain a JMESPath expression by walking its AST.
    ///
    /// Returns a structured breakdown with step-by-step descriptions,
    /// a list of functions used, and a complexity rating.
    ///
    /// For invalid expressions, returns an `InvalidExpression` error
    /// containing the parse error message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Valid expression
    /// let result = engine.explain("users[*].name").unwrap();
    /// assert_eq!(result.steps.len(), 1); // top-level projection
    /// assert!(result.functions_used.is_empty());
    ///
    /// // Invalid expression returns an error
    /// let err = engine.explain("users[*.name");
    /// assert!(err.is_err());
    /// ```
    pub fn explain(&self, expression: &str) -> Result<ExplainResult> {
        let ast = jpx_core::parse(expression)
            .map_err(|e| EngineError::InvalidExpression(e.to_string()))?;

        let mut functions = Vec::new();
        let steps = vec![walk_ast(&ast, &mut functions)];

        functions.sort();
        functions.dedup();

        let complexity = classify_complexity(&ast);

        Ok(ExplainResult {
            expression: expression.to_string(),
            steps,
            functions_used: functions,
            complexity,
        })
    }
}

/// Recursively walk the AST and produce explain steps.
fn walk_ast(node: &Ast, functions: &mut Vec<String>) -> ExplainStep {
    match node {
        Ast::Identity { .. } => ExplainStep {
            node_type: "identity".into(),
            description: "Reference the current node (@)".into(),
            children: vec![],
        },
        Ast::Field { name, .. } => ExplainStep {
            node_type: "field".into(),
            description: format!("Select the '{}' field", name),
            children: vec![],
        },
        Ast::Index { idx, .. } => ExplainStep {
            node_type: "index".into(),
            description: if *idx < 0 {
                format!("Select element at index {} (from end)", idx)
            } else {
                format!("Select element at index {}", idx)
            },
            children: vec![],
        },
        Ast::Slice {
            start, stop, step, ..
        } => {
            let start_s = start.map_or(String::new(), |s| s.to_string());
            let stop_s = stop.map_or(String::new(), |s| s.to_string());
            let desc = if *step == 1 {
                format!("Slice array [{}:{}]", start_s, stop_s)
            } else {
                format!("Slice array [{}:{}:{}]", start_s, stop_s, step)
            };
            ExplainStep {
                node_type: "slice".into(),
                description: desc,
                children: vec![],
            }
        }
        Ast::Subexpr { lhs, rhs, .. } => {
            let left = walk_ast(lhs, functions);
            let right = walk_ast(rhs, functions);

            // Check if this is a pipe expression (rhs starts at a new scope)
            // Subexpr is the general a.b form; show both sides
            ExplainStep {
                node_type: "subexpression".into(),
                description: "Chain two expressions (left.right)".into(),
                children: vec![left, right],
            }
        }
        Ast::Projection { lhs, rhs, .. } => {
            let source = walk_ast(lhs, functions);
            let project = walk_ast(rhs, functions);
            ExplainStep {
                node_type: "projection".into(),
                description: "Project: evaluate right side for each element of left side".into(),
                children: vec![source, project],
            }
        }
        Ast::Function { name, args, .. } => {
            functions.push(name.clone());
            let arg_steps: Vec<ExplainStep> = args.iter().map(|a| walk_ast(a, functions)).collect();
            let desc = if args.is_empty() {
                format!("Call function {}()", name)
            } else {
                format!("Call function {}() with {} argument(s)", name, args.len())
            };
            ExplainStep {
                node_type: "function".into(),
                description: desc,
                children: arg_steps,
            }
        }
        Ast::Literal { value, .. } => {
            let json = serde_json::to_string(value).unwrap_or_else(|_| "?".into());
            ExplainStep {
                node_type: "literal".into(),
                description: format!("Literal value: {}", json),
                children: vec![],
            }
        }
        Ast::Comparison {
            comparator,
            lhs,
            rhs,
            ..
        } => {
            let op = match comparator {
                Comparator::Equal => "==",
                Comparator::NotEqual => "!=",
                Comparator::LessThan => "<",
                Comparator::LessThanEqual => "<=",
                Comparator::GreaterThan => ">",
                Comparator::GreaterThanEqual => ">=",
            };
            let left = walk_ast(lhs, functions);
            let right = walk_ast(rhs, functions);
            ExplainStep {
                node_type: "comparison".into(),
                description: format!("Compare using {}", op),
                children: vec![left, right],
            }
        }
        Ast::And { lhs, rhs, .. } => {
            let left = walk_ast(lhs, functions);
            let right = walk_ast(rhs, functions);
            ExplainStep {
                node_type: "and".into(),
                description: "Logical AND: both sides must be truthy".into(),
                children: vec![left, right],
            }
        }
        Ast::Or { lhs, rhs, .. } => {
            let left = walk_ast(lhs, functions);
            let right = walk_ast(rhs, functions);
            ExplainStep {
                node_type: "or".into(),
                description: "Logical OR: return left if truthy, else right".into(),
                children: vec![left, right],
            }
        }
        Ast::Not { node, .. } => {
            let inner = walk_ast(node, functions);
            ExplainStep {
                node_type: "not".into(),
                description: "Logical NOT: negate the result".into(),
                children: vec![inner],
            }
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            let pred = walk_ast(predicate, functions);
            let body = walk_ast(then, functions);
            ExplainStep {
                node_type: "filter".into(),
                description: "Filter elements matching a condition".into(),
                children: vec![pred, body],
            }
        }
        Ast::Flatten { node, .. } => {
            let inner = walk_ast(node, functions);
            ExplainStep {
                node_type: "flatten".into(),
                description: "Flatten nested arrays by one level".into(),
                children: vec![inner],
            }
        }
        Ast::ObjectValues { node, .. } => {
            let inner = walk_ast(node, functions);
            ExplainStep {
                node_type: "object_values".into(),
                description: "Extract all values from an object".into(),
                children: vec![inner],
            }
        }
        Ast::MultiList { elements, .. } => {
            let children: Vec<ExplainStep> =
                elements.iter().map(|e| walk_ast(e, functions)).collect();
            ExplainStep {
                node_type: "multi_select_list".into(),
                description: format!("Create a list of {} evaluated expressions", elements.len()),
                children,
            }
        }
        Ast::MultiHash { elements, .. } => {
            let children: Vec<ExplainStep> = elements
                .iter()
                .map(|kvp| {
                    let mut step = walk_ast(&kvp.value, functions);
                    step.description = format!("'{}': {}", kvp.key, step.description);
                    step
                })
                .collect();
            ExplainStep {
                node_type: "multi_select_hash".into(),
                description: format!("Create an object with {} key(s)", elements.len()),
                children,
            }
        }
        Ast::Expref { ast, .. } => {
            let inner = walk_ast(ast, functions);
            ExplainStep {
                node_type: "expression_reference".into(),
                description: "Pass expression as argument (used by sort_by, map, etc.)".into(),
                children: vec![inner],
            }
        }
        Ast::VariableRef { name, .. } => ExplainStep {
            node_type: "variable_ref".into(),
            description: format!("Reference variable ${}", name),
            children: vec![],
        },
        Ast::Let { bindings, expr, .. } => {
            let mut children: Vec<ExplainStep> = bindings
                .iter()
                .map(|(name, ast)| {
                    let mut step = walk_ast(ast, functions);
                    step.description = format!("${} = {}", name, step.description);
                    step
                })
                .collect();
            children.push(walk_ast(expr, functions));
            ExplainStep {
                node_type: "let".into(),
                description: format!("Bind {} variable(s) and evaluate body", bindings.len()),
                children,
            }
        }
    }
}

/// Rate the expression complexity based on AST depth and features used.
fn classify_complexity(ast: &Ast) -> String {
    let depth = ast_depth(ast);
    let func_count = count_functions(ast);
    let has_filter = uses_filter(ast);

    if depth <= 2 && func_count == 0 && !has_filter {
        "simple".into()
    } else if depth <= 5 && func_count <= 2 {
        "moderate".into()
    } else {
        "complex".into()
    }
}

fn ast_depth(node: &Ast) -> usize {
    match node {
        Ast::Identity { .. }
        | Ast::Field { .. }
        | Ast::Index { .. }
        | Ast::Slice { .. }
        | Ast::Literal { .. } => 1,
        Ast::Subexpr { lhs, rhs, .. }
        | Ast::Projection { lhs, rhs, .. }
        | Ast::And { lhs, rhs, .. }
        | Ast::Or { lhs, rhs, .. }
        | Ast::Comparison { lhs, rhs, .. } => 1 + ast_depth(lhs).max(ast_depth(rhs)),
        Ast::Condition {
            predicate, then, ..
        } => 1 + ast_depth(predicate).max(ast_depth(then)),
        Ast::Not { node, .. } | Ast::Flatten { node, .. } | Ast::ObjectValues { node, .. } => {
            1 + ast_depth(node)
        }
        Ast::Function { args, .. } => 1 + args.iter().map(ast_depth).max().unwrap_or(0),
        Ast::MultiList { elements, .. } => 1 + elements.iter().map(ast_depth).max().unwrap_or(0),
        Ast::MultiHash { elements, .. } => {
            1 + elements
                .iter()
                .map(|kvp| ast_depth(&kvp.value))
                .max()
                .unwrap_or(0)
        }
        Ast::Expref { ast, .. } => 1 + ast_depth(ast),
        Ast::VariableRef { .. } => 1,
        Ast::Let { bindings, expr, .. } => {
            let binding_depth = bindings
                .iter()
                .map(|(_, ast)| ast_depth(ast))
                .max()
                .unwrap_or(0);
            1 + binding_depth.max(ast_depth(expr))
        }
    }
}

fn count_functions(node: &Ast) -> usize {
    match node {
        Ast::Identity { .. }
        | Ast::Field { .. }
        | Ast::Index { .. }
        | Ast::Slice { .. }
        | Ast::Literal { .. } => 0,
        Ast::Subexpr { lhs, rhs, .. }
        | Ast::Projection { lhs, rhs, .. }
        | Ast::And { lhs, rhs, .. }
        | Ast::Or { lhs, rhs, .. }
        | Ast::Comparison { lhs, rhs, .. } => count_functions(lhs) + count_functions(rhs),
        Ast::Condition {
            predicate, then, ..
        } => count_functions(predicate) + count_functions(then),
        Ast::Not { node, .. } | Ast::Flatten { node, .. } | Ast::ObjectValues { node, .. } => {
            count_functions(node)
        }
        Ast::Function { args, .. } => 1 + args.iter().map(count_functions).sum::<usize>(),
        Ast::MultiList { elements, .. } => elements.iter().map(count_functions).sum(),
        Ast::MultiHash { elements, .. } => {
            elements.iter().map(|kvp| count_functions(&kvp.value)).sum()
        }
        Ast::Expref { ast, .. } => count_functions(ast),
        Ast::VariableRef { .. } => 0,
        Ast::Let { bindings, expr, .. } => {
            bindings
                .iter()
                .map(|(_, ast)| count_functions(ast))
                .sum::<usize>()
                + count_functions(expr)
        }
    }
}

fn uses_filter(node: &Ast) -> bool {
    match node {
        Ast::Condition { .. } => true,
        Ast::Identity { .. }
        | Ast::Field { .. }
        | Ast::Index { .. }
        | Ast::Slice { .. }
        | Ast::Literal { .. } => false,
        Ast::Subexpr { lhs, rhs, .. }
        | Ast::Projection { lhs, rhs, .. }
        | Ast::And { lhs, rhs, .. }
        | Ast::Or { lhs, rhs, .. }
        | Ast::Comparison { lhs, rhs, .. } => uses_filter(lhs) || uses_filter(rhs),
        Ast::Not { node, .. } | Ast::Flatten { node, .. } | Ast::ObjectValues { node, .. } => {
            uses_filter(node)
        }
        Ast::Function { args, .. } => args.iter().any(uses_filter),
        Ast::MultiList { elements, .. } => elements.iter().any(uses_filter),
        Ast::MultiHash { elements, .. } => elements.iter().any(|kvp| uses_filter(&kvp.value)),
        Ast::Expref { ast, .. } => uses_filter(ast),
        Ast::VariableRef { .. } => false,
        Ast::Let { bindings, expr, .. } => {
            bindings.iter().any(|(_, ast)| uses_filter(ast)) || uses_filter(expr)
        }
    }
}

/// Returns `true` if the AST contains any `Let` or `VariableRef` nodes.
///
/// Used by strict mode to reject JEP-18 let expressions, which are not
/// part of the standard JMESPath specification.
pub fn has_let_nodes(node: &Ast) -> bool {
    match node {
        Ast::VariableRef { .. } | Ast::Let { .. } => true,
        Ast::Identity { .. }
        | Ast::Field { .. }
        | Ast::Index { .. }
        | Ast::Slice { .. }
        | Ast::Literal { .. } => false,
        Ast::Subexpr { lhs, rhs, .. }
        | Ast::Projection { lhs, rhs, .. }
        | Ast::And { lhs, rhs, .. }
        | Ast::Or { lhs, rhs, .. }
        | Ast::Comparison { lhs, rhs, .. } => has_let_nodes(lhs) || has_let_nodes(rhs),
        Ast::Condition {
            predicate, then, ..
        } => has_let_nodes(predicate) || has_let_nodes(then),
        Ast::Not { node, .. } | Ast::Flatten { node, .. } | Ast::ObjectValues { node, .. } => {
            has_let_nodes(node)
        }
        Ast::Function { args, .. } => args.iter().any(has_let_nodes),
        Ast::MultiList { elements, .. } => elements.iter().any(has_let_nodes),
        Ast::MultiHash { elements, .. } => elements.iter().any(|kvp| has_let_nodes(&kvp.value)),
        Ast::Expref { ast, .. } => has_let_nodes(ast),
    }
}

/// Collect all function names referenced in the AST.
///
/// Walks the entire AST and returns a deduplicated, sorted list of
/// every function name that appears in a `Function` node. Useful for
/// checking whether an expression uses extension functions.
pub fn collect_function_names(node: &Ast) -> Vec<String> {
    let mut names = Vec::new();
    collect_functions_recursive(node, &mut names);
    names.sort();
    names.dedup();
    names
}

fn collect_functions_recursive(node: &Ast, names: &mut Vec<String>) {
    match node {
        Ast::Identity { .. }
        | Ast::Field { .. }
        | Ast::Index { .. }
        | Ast::Slice { .. }
        | Ast::Literal { .. }
        | Ast::VariableRef { .. } => {}
        Ast::Subexpr { lhs, rhs, .. }
        | Ast::Projection { lhs, rhs, .. }
        | Ast::And { lhs, rhs, .. }
        | Ast::Or { lhs, rhs, .. }
        | Ast::Comparison { lhs, rhs, .. } => {
            collect_functions_recursive(lhs, names);
            collect_functions_recursive(rhs, names);
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            collect_functions_recursive(predicate, names);
            collect_functions_recursive(then, names);
        }
        Ast::Not { node, .. } | Ast::Flatten { node, .. } | Ast::ObjectValues { node, .. } => {
            collect_functions_recursive(node, names);
        }
        Ast::Function { name, args, .. } => {
            names.push(name.clone());
            for arg in args {
                collect_functions_recursive(arg, names);
            }
        }
        Ast::MultiList { elements, .. } => {
            for e in elements {
                collect_functions_recursive(e, names);
            }
        }
        Ast::MultiHash { elements, .. } => {
            for kvp in elements {
                collect_functions_recursive(&kvp.value, names);
            }
        }
        Ast::Expref { ast, .. } => collect_functions_recursive(ast, names),
        Ast::Let { bindings, expr, .. } => {
            for (_, ast) in bindings {
                collect_functions_recursive(ast, names);
            }
            collect_functions_recursive(expr, names);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> JpxEngine {
        JpxEngine::new()
    }

    #[test]
    fn test_simple_field() {
        let result = engine().explain("name").unwrap();
        assert_eq!(result.steps[0].node_type, "field");
        assert!(result.steps[0].description.contains("name"));
        assert!(result.functions_used.is_empty());
        assert_eq!(result.complexity, "simple");
    }

    #[test]
    fn test_filter_expression() {
        let result = engine().explain("users[?age > `30`]").unwrap();
        // Top-level is a projection with a filter condition
        assert!(!result.steps.is_empty());
        assert_eq!(result.complexity, "moderate");
    }

    #[test]
    fn test_projection() {
        let result = engine().explain("users[*].name").unwrap();
        assert_eq!(result.steps[0].node_type, "projection");
        assert!(result.functions_used.is_empty());
    }

    #[test]
    fn test_pipe_with_function() {
        let result = engine().explain("users[*].name | sort(@)").unwrap();
        assert!(result.functions_used.contains(&"sort".to_string()));
        assert_eq!(result.complexity, "moderate");
    }

    #[test]
    fn test_multi_select() {
        let result = engine().explain("{name: name, age: age}").unwrap();
        assert_eq!(result.steps[0].node_type, "multi_select_hash");
        assert_eq!(result.steps[0].children.len(), 2);
    }

    #[test]
    fn test_complex_expression() {
        let result = engine()
            .explain("users[?active].addresses[*].city | sort(@) | join(', ', @)")
            .unwrap();
        assert!(result.functions_used.contains(&"sort".to_string()));
        assert!(result.functions_used.contains(&"join".to_string()));
        assert_eq!(result.complexity, "complex");
    }

    #[test]
    fn test_invalid_expression() {
        let err = engine().explain("users[*.name");
        assert!(err.is_err());
    }

    #[test]
    fn test_identity() {
        let result = engine().explain("@").unwrap();
        assert_eq!(result.steps[0].node_type, "identity");
        assert_eq!(result.complexity, "simple");
    }

    #[test]
    fn test_index() {
        let result = engine().explain("[0]").unwrap();
        assert_eq!(result.steps[0].node_type, "index");
    }

    #[test]
    fn test_flatten() {
        let result = engine().explain("items[]").unwrap();
        // flatten wraps a field access
        assert!(!result.steps.is_empty());
    }

    /// Recursively search for a node_type anywhere in the step tree.
    fn contains_node_type(step: &ExplainStep, target: &str) -> bool {
        if step.node_type == target {
            return true;
        }
        step.children.iter().any(|c| contains_node_type(c, target))
    }

    // --- Missing AST node coverage ---

    #[test]
    fn test_variable_ref() {
        let result = engine().explain("let $x = name in $x").unwrap();
        // The let body references $x, which produces a variable_ref node
        assert!(
            result
                .steps
                .iter()
                .any(|s| contains_node_type(s, "variable_ref"))
        );
    }

    #[test]
    fn test_let_expression() {
        let result = engine().explain("let $x = name in upper($x)").unwrap();
        let top = &result.steps[0];
        assert_eq!(top.node_type, "let");
        assert!(top.description.contains("1 variable"));
        // Children: one binding + the body
        assert_eq!(top.children.len(), 2);
        assert!(result.functions_used.contains(&"upper".to_string()));
    }

    #[test]
    fn test_expref() {
        let result = engine().explain("sort_by(users, &age)").unwrap();
        assert!(
            result
                .steps
                .iter()
                .any(|s| contains_node_type(s, "expression_reference"))
        );
        assert!(result.functions_used.contains(&"sort_by".to_string()));
    }

    #[test]
    fn test_object_values() {
        // `*` on its own produces ObjectValues
        let result = engine().explain("*").unwrap();
        assert!(
            result
                .steps
                .iter()
                .any(|s| contains_node_type(s, "object_values"))
        );
    }

    #[test]
    fn test_not_expression() {
        let result = engine().explain("!active").unwrap();
        assert_eq!(result.steps[0].node_type, "not");
        assert!(result.steps[0].description.contains("NOT"));
        assert_eq!(result.steps[0].children.len(), 1);
    }

    #[test]
    fn test_and_expression() {
        let result = engine().explain("a && b").unwrap();
        assert_eq!(result.steps[0].node_type, "and");
        assert!(result.steps[0].description.contains("AND"));
        assert_eq!(result.steps[0].children.len(), 2);
        assert_eq!(result.steps[0].children[0].node_type, "field");
        assert_eq!(result.steps[0].children[1].node_type, "field");
    }

    #[test]
    fn test_or_expression() {
        let result = engine().explain("a || b").unwrap();
        assert_eq!(result.steps[0].node_type, "or");
        assert!(result.steps[0].description.contains("OR"));
        assert_eq!(result.steps[0].children.len(), 2);
        assert_eq!(result.steps[0].children[0].node_type, "field");
        assert_eq!(result.steps[0].children[1].node_type, "field");
    }

    // --- has_let_nodes tests ---

    #[test]
    fn test_has_let_nodes_simple_field() {
        let ast = jpx_core::parse("foo.bar").unwrap();
        assert!(!has_let_nodes(&ast));
    }

    #[test]
    fn test_has_let_nodes_with_let() {
        let ast = jpx_core::parse("let $x = name in $x").unwrap();
        assert!(has_let_nodes(&ast));
    }

    #[test]
    fn test_has_let_nodes_nested_in_function() {
        let ast = jpx_core::parse("length(people)").unwrap();
        assert!(!has_let_nodes(&ast));
    }

    #[test]
    fn test_has_let_nodes_variable_in_filter() {
        let ast = jpx_core::parse("let $min = `30` in people[?age > $min]").unwrap();
        assert!(has_let_nodes(&ast));
    }

    // --- Complexity boundary tests ---

    #[test]
    fn test_complexity_simple_field() {
        let result = engine().explain("name").unwrap();
        assert_eq!(result.complexity, "simple");
    }

    #[test]
    fn test_complexity_simple_identity() {
        let result = engine().explain("@").unwrap();
        assert_eq!(result.complexity, "simple");
    }

    #[test]
    fn test_complexity_moderate_with_function() {
        let result = engine().explain("length(@)").unwrap();
        // Has a function (count=1) but low depth, so "moderate"
        assert_eq!(result.complexity, "moderate");
        assert!(result.functions_used.contains(&"length".to_string()));
    }

    #[test]
    fn test_complexity_moderate_with_filter() {
        let result = engine().explain("users[?active]").unwrap();
        // Has a filter condition, which makes it at least moderate
        assert_eq!(result.complexity, "moderate");
    }

    #[test]
    fn test_complexity_complex_multi_function() {
        // Three functions pushes func_count > 2, triggering "complex"
        let result = engine().explain("sort(keys(@)) | join(', ', @)").unwrap();
        assert_eq!(result.functions_used.len(), 3);
        assert_eq!(result.complexity, "complex");
    }

    // --- Other tests ---

    #[test]
    fn test_explain_comparison_operators() {
        let result = engine().explain("a > b").unwrap();
        assert_eq!(result.steps[0].node_type, "comparison");
        assert!(result.steps[0].description.contains(">"));
        assert_eq!(result.steps[0].children.len(), 2);
    }

    #[test]
    fn test_explain_slice() {
        let result = engine().explain("items[1:3]").unwrap();
        assert!(result.steps.iter().any(|s| contains_node_type(s, "slice")));
    }

    #[test]
    fn test_explain_literal() {
        let result = engine().explain("`42`").unwrap();
        assert_eq!(result.steps[0].node_type, "literal");
        assert!(result.steps[0].description.contains("42"));
    }

    // --- collect_function_names tests ---

    #[test]
    fn test_collect_function_names_empty() {
        let ast = jpx_core::parse("foo.bar").unwrap();
        assert!(collect_function_names(&ast).is_empty());
    }

    #[test]
    fn test_collect_function_names_standard() {
        let ast = jpx_core::parse("length(sort(@))").unwrap();
        let names = collect_function_names(&ast);
        assert_eq!(names, vec!["length", "sort"]);
    }

    #[test]
    fn test_collect_function_names_nested() {
        let ast = jpx_core::parse("users[?contains(name, 'a')] | sort_by(@, &age) | [0]").unwrap();
        let names = collect_function_names(&ast);
        assert_eq!(names, vec!["contains", "sort_by"]);
    }
}
