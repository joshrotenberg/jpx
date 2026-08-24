use proptest::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;

fn identifier() -> BoxedStrategy<String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,7}".boxed()
}

/// Generate bounded, grammar-directed JMESPath expressions.
///
/// Every recursive branch corresponds to a parser production. The depth is
/// intentionally well below the parser's 128-level guard so this population
/// explores grammar interactions rather than mostly exercising that guard.
pub fn grammar_expression() -> BoxedStrategy<String> {
    let atom = prop_oneof![
        Just("@".to_string()),
        identifier(),
        "[a-zA-Z0-9 _-]{0,12}".prop_map(|text| format!("'{text}'")),
        any::<i16>().prop_map(|number| format!("`{number}`")),
        prop_oneof![Just("true"), Just("false"), Just("null")]
            .prop_map(|value| format!("`{value}`")),
        "[a-zA-Z0-9 _-]{0,12}"
            .prop_map(|text| format!("`{}`", serde_json::to_string(&text).unwrap())),
        (-8i16..8).prop_map(|index| format!("[{index}]")),
        Just("*".to_string()),
    ];

    atom.prop_recursive(6, 192, 8, |inner| {
        let binary = (inner.clone(), inner.clone());
        let field = identifier();
        let standard_productions = prop_oneof![
            // Subexpressions and pipes.
            (inner.clone(), field.clone()).prop_map(|(left, right)| format!("{left}.{right}")),
            binary
                .clone()
                .prop_map(|(left, right)| format!("({left}) | ({right})")),
            // Comparators, logical operators, and prefix not.
            (
                binary.clone(),
                prop::sample::select(vec!["==", "!=", "<", "<=", ">", ">="])
            )
                .prop_map(|((left, right), op)| format!("({left}) {op} ({right})")),
            (binary.clone(), prop::sample::select(vec!["&&", "||"]))
                .prop_map(|((left, right), op)| format!("({left}) {op} ({right})")),
            inner.clone().prop_map(|expr| format!("!({expr})")),
            // Bracket expressions: indices, slices, flatten, filters, and projections.
            (inner.clone(), -8i16..8).prop_map(|(expr, index)| format!("{expr}[{index}]")),
            (
                inner.clone(),
                prop::option::of(-8i16..8),
                prop::option::of(-8i16..8),
                prop_oneof![
                    Just(None),
                    (-4i16..0).prop_map(Some),
                    (1i16..5).prop_map(Some)
                ],
            )
                .prop_map(|(expr, start, stop, step)| {
                    let start = start.map_or_else(String::new, |n| n.to_string());
                    let stop = stop.map_or_else(String::new, |n| n.to_string());
                    let step = step.map_or_else(String::new, |n| format!(":{n}"));
                    format!("{expr}[{start}:{stop}{step}]")
                }),
            inner.clone().prop_map(|expr| format!("{expr}[]")),
            (inner.clone(), field.clone()).prop_map(|(expr, field)| format!("{expr}[*].{field}")),
            (inner.clone(), field.clone()).prop_map(|(expr, field)| format!("{expr}.*.{field}")),
            (inner.clone(), inner.clone())
                .prop_map(|(expr, predicate)| format!("{expr}[?{predicate}]")),
            // Multiselects.
            binary
                .clone()
                .prop_map(|(left, right)| format!("[{left}, {right}]")),
            binary
                .clone()
                .prop_map(|(left, right)| format!("{{left: {left}, right: {right}}}")),
            // Functions, including an expression-reference argument.
            inner.clone().prop_map(|expr| format!("length({expr})")),
            binary
                .clone()
                .prop_map(|(left, right)| format!("contains({left}, {right})")),
            (inner.clone(), field).prop_map(|(expr, field)| format!("sort_by({expr}, &{field})")),
        ];

        #[cfg(feature = "let-expr")]
        {
            prop_oneof![
                standard_productions,
                // JEP-18 let binding and variable reference.
                binary.prop_map(|(bound, body)| format!("let $x = {bound} in {body}")),
            ]
        }

        #[cfg(not(feature = "let-expr"))]
        standard_productions
    })
    .boxed()
}

/// One known-valid expression for every production family used above.
pub fn grammar_smoke_expressions() -> Vec<&'static str> {
    let expressions = vec![
        "@",
        "field",
        "'raw string'",
        "`\"json string\"`",
        "foo.bar",
        "foo | bar",
        "foo == `1`",
        "foo && bar",
        "foo || bar",
        "!foo",
        "items[0]",
        "items[1:4:2]",
        "items[]",
        "items[*].name",
        "object.*.name",
        "items[?score >= `10`]",
        "[name, age]",
        "{name: name, age: age}",
        "length(items)",
        "contains(tags, 'rust')",
        "sort_by(items, &name)",
    ];
    #[cfg(feature = "let-expr")]
    {
        let mut expressions = expressions;
        expressions.push("let $x = foo in $x.bar");
        expressions
    }
    #[cfg(not(feature = "let-expr"))]
    {
        expressions
    }
}

/// Read all successful expressions from the vendored compliance suite.
pub fn compliance_expressions() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/compliance");
    let mut paths = fs::read_dir(root)
        .expect("read vendored compliance directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut expressions = Vec::new();
    for path in paths {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(groups) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        let Some(groups) = groups.as_array() else {
            continue;
        };
        for group in groups {
            let Some(cases) = group.get("cases").and_then(Value::as_array) else {
                continue;
            };
            for case in cases {
                if case.get("error").is_none()
                    && let Some(expression) = case.get("expression").and_then(Value::as_str)
                {
                    expressions.push(expression.to_string());
                }
            }
        }
    }
    expressions.sort();
    expressions.dedup();
    assert!(
        !expressions.is_empty(),
        "vendored compliance suite must contain successful expressions"
    );
    expressions
}

/// Mutate a known-valid compliance expression by one small edit.
pub fn near_valid_expression() -> BoxedStrategy<String> {
    (
        prop::sample::select(compliance_expressions()),
        0usize..5,
        any::<usize>(),
    )
        .prop_map(|(expression, mutation, seed)| {
            let mut chars = expression.chars().collect::<Vec<_>>();
            if chars.is_empty() {
                return "]".to_string();
            }
            let index = seed % chars.len();
            match mutation {
                0 => chars.truncate(index),
                1 => {
                    chars.remove(index);
                }
                2 => chars[index] = [']', '}', ')', '|', ','][seed % 5],
                3 => {
                    let ch = chars[index];
                    chars.insert(index, ch);
                }
                _ => chars.extend(['[', '?']),
            }
            chars.into_iter().collect()
        })
        .boxed()
}
