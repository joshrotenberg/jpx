use jpx_core::ast::{Ast, Comparator};
use serde_json::Value;

/// Describe a Value for verbose output
pub(crate) fn describe_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => format!("bool ({})", b),
        Value::Number(n) => format!("number ({})", n),
        Value::String(s) => {
            if s.len() > 50 {
                format!("string ({} chars)", s.len())
            } else {
                format!("string \"{}\"", s)
            }
        }
        Value::Array(arr) => format!("array ({} items)", arr.len()),
        Value::Object(obj) => format!("object ({} keys)", obj.len()),
    }
}

/// Print AST in a human-readable tree format
pub(crate) fn print_ast(node: &Ast, indent: usize) {
    let prefix = "  ".repeat(indent);
    let connector = if indent > 0 { "├─ " } else { "" };

    match node {
        Ast::Identity { .. } => {
            println!("{}{}@ (current node)", prefix, connector);
        }
        Ast::Field { name, .. } => {
            println!("{}{}Field: {}", prefix, connector, name);
        }
        Ast::Index { idx, .. } => {
            println!("{}{}Index: [{}]", prefix, connector, idx);
        }
        Ast::Slice {
            start, stop, step, ..
        } => {
            let start_str = start.map_or("".to_string(), |s| s.to_string());
            let stop_str = stop.map_or("".to_string(), |s| s.to_string());
            if *step == 1 {
                println!("{}{}Slice: [{}:{}]", prefix, connector, start_str, stop_str);
            } else {
                println!(
                    "{}{}Slice: [{}:{}:{}]",
                    prefix, connector, start_str, stop_str, step
                );
            }
        }
        Ast::Subexpr { lhs, rhs, .. } => {
            println!("{}{}Subexpression (a.b):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Projection { lhs, rhs, .. } => {
            println!("{}{}Projection (map over array):", prefix, connector);
            println!("{}  source:", prefix);
            print_ast(lhs, indent + 2);
            println!("{}  project:", prefix);
            print_ast(rhs, indent + 2);
        }
        Ast::Function { name, args, .. } => {
            if args.is_empty() {
                println!("{}{}Function: {}()", prefix, connector, name);
            } else {
                println!("{}{}Function: {}", prefix, connector, name);
                for (i, arg) in args.iter().enumerate() {
                    println!("{}  arg {}:", prefix, i + 1);
                    print_ast(arg, indent + 2);
                }
            }
        }
        Ast::Literal { value, .. } => {
            let json = serde_json::to_string(value).unwrap_or_else(|_| "?".to_string());
            println!("{}{}Literal: `{}`", prefix, connector, json);
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
            println!("{}{}Comparison: {}", prefix, connector, op);
            println!("{}  left:", prefix);
            print_ast(lhs, indent + 2);
            println!("{}  right:", prefix);
            print_ast(rhs, indent + 2);
        }
        Ast::And { lhs, rhs, .. } => {
            println!("{}{}And (&&):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Or { lhs, rhs, .. } => {
            println!("{}{}Or (||):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Not { node, .. } => {
            println!("{}{}Not (!):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            println!("{}{}Filter condition ([?...]):", prefix, connector);
            println!("{}  predicate:", prefix);
            print_ast(predicate, indent + 2);
            println!("{}  then:", prefix);
            print_ast(then, indent + 2);
        }
        Ast::Flatten { node, .. } => {
            println!("{}{}Flatten ([]):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::ObjectValues { node, .. } => {
            println!("{}{}Object values (*):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::MultiList { elements, .. } => {
            println!(
                "{}{}Multi-select list ({} elements):",
                prefix,
                connector,
                elements.len()
            );
            for (i, elem) in elements.iter().enumerate() {
                println!("{}  [{}]:", prefix, i);
                print_ast(elem, indent + 2);
            }
        }
        Ast::MultiHash { elements, .. } => {
            println!(
                "{}{}Multi-select hash ({} keys):",
                prefix,
                connector,
                elements.len()
            );
            for kvp in elements {
                println!("{}  {}:", prefix, kvp.key);
                print_ast(&kvp.value, indent + 2);
            }
        }
        Ast::Expref { ast, .. } => {
            println!("{}{}Expression reference (&):", prefix, connector);
            print_ast(ast, indent + 1);
        }
        Ast::VariableRef { name, .. } => {
            println!("{}{}Variable: ${}", prefix, connector, name);
        }
        Ast::Let { bindings, expr, .. } => {
            println!(
                "{}{}Let ({} binding(s)):",
                prefix,
                connector,
                bindings.len()
            );
            for (name, binding_ast) in bindings {
                println!("{}  ${} =", prefix, name);
                print_ast(binding_ast, indent + 2);
            }
            println!("{}  in:", prefix);
            print_ast(expr, indent + 2);
        }
    }
}
