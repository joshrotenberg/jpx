//! Integration tests for jpx CLI

use std::io::Write;
use std::process::Command;

fn jpx_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jpx"))
}

fn run_query(json: &str, query: &str) -> String {
    let mut child = jpx_cmd()
        .arg(query)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn jpx");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(json.as_bytes())
        .expect("Failed to write to stdin");

    let output = child.wait_with_output().expect("Failed to wait on jpx");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_with_args(args: &[&str], stdin_data: &str) -> std::process::Output {
    let mut child = jpx_cmd()
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn jpx");

    if !stdin_data.is_empty() {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin_data.as_bytes())
            .expect("Failed to write to stdin");
    }
    drop(child.stdin.take()); // Close stdin

    child.wait_with_output().expect("Failed to wait on jpx")
}

fn run_query_with_file(file: &str, query: &str) -> String {
    let testdata = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/");
    let path = format!("{}{}", testdata, file);

    let output = jpx_cmd()
        .arg("-f")
        .arg(&path)
        .arg(query)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn jpx")
        .wait_with_output()
        .expect("Failed to wait on jpx");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

mod basic_queries {
    use super::*;

    #[test]
    fn test_simple_field_access() {
        let result = run_query(r#"{"name": "Alice", "age": 30}"#, "name");
        assert_eq!(result, r#""Alice""#);
    }

    #[test]
    fn test_nested_field_access() {
        let result = run_query(r#"{"user": {"name": "Bob"}}"#, "user.name");
        assert_eq!(result, r#""Bob""#);
    }

    #[test]
    fn test_array_index() {
        let result = run_query(r#"[1, 2, 3]"#, "[1]");
        assert_eq!(result, "2");
    }

    #[test]
    fn test_array_projection() {
        let result = run_query(r#"[{"a": 1}, {"a": 2}]"#, "[*].a");
        assert_eq!(result, "[\n  1,\n  2\n]");
    }

    #[test]
    fn test_filter_expression() {
        let result = run_query(r#"[{"age": 20}, {"age": 30}]"#, "[?age > `25`]");
        assert_eq!(result, "[\n  {\n    \"age\": 30\n  }\n]");
    }
}

mod extension_functions {
    use super::*;

    #[test]
    fn test_unique() {
        let result = run_query(r#"[1, 2, 2, 3, 3, 3]"#, "unique(@)");
        assert_eq!(result, "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_flatten_deep() {
        let result = run_query(r#"[[1, [2, [3]]]]"#, "flatten_deep(@)");
        assert_eq!(result, "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_split() {
        // split(string, delimiter) - splits the string by delimiter
        let result = run_query(r#""a,b,c""#, "split(@, ',')");
        assert_eq!(result, "[\n  \"a\",\n  \"b\",\n  \"c\"\n]");
    }

    #[test]
    fn test_trim() {
        let result = run_query(r#""  hello  ""#, "trim(@)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_now() {
        // now() returns a number (Unix timestamp)
        let result = run_query(r#"null"#, "type(now())");
        assert_eq!(result, r#""number""#);
    }

    #[test]
    fn test_from_items() {
        // from_items converts [[key, value], ...] pairs to object
        let result = run_query(r#"[["a", 1]]"#, "from_items(@)");
        assert_eq!(result, "{\n  \"a\": 1\n}");
    }

    #[test]
    fn test_items() {
        // items converts object to [[key, value], ...] pairs
        let result = run_query(r#"{"a": 1}"#, "items(@)");
        assert!(result.contains("\"a\""));
        assert!(result.contains("1"));
    }

    #[test]
    fn test_group_by_expr() {
        // group_by_expr('expression', array)
        let result = run_query(
            r#"[{"role": "admin", "name": "Alice"}, {"role": "user", "name": "Bob"}, {"role": "admin", "name": "Carol"}]"#,
            "group_by_expr('role', @)",
        );
        assert!(result.contains("\"admin\""));
        assert!(result.contains("\"user\""));
    }

    #[test]
    fn test_map_values() {
        let result = run_query(r#"{"a": 1, "b": 2}"#, "map_values('multiply(@, `2`)', @)");
        assert!(result.contains("\"a\": 2"));
        assert!(result.contains("\"b\": 4"));
    }

    #[test]
    fn test_reduce_expr() {
        // reduce_expr(expression, array, initial)
        let result = run_query(
            r#"[1, 2, 3, 4, 5]"#,
            "reduce_expr('add(accumulator, current)', @, `0`)",
        );
        assert_eq!(result, "15.0");
    }

    #[test]
    fn test_filter_expr() {
        // filter_expr('expression', array)
        let result = run_query(r#"[1, 2, 3, 4, 5]"#, "filter_expr('@ > `3`', @)");
        assert_eq!(result, "[\n  4,\n  5\n]");
    }
}

mod file_operations {
    use super::*;

    #[test]
    fn test_load_users_file() {
        let result = run_query_with_file("users.json", "length(@)");
        assert_eq!(result, "5");
    }

    #[test]
    fn test_users_filter() {
        let result =
            run_query_with_file("users.json", "[?department == 'Engineering'] | length(@)");
        assert_eq!(result, "3");
    }

    #[test]
    fn test_users_projection() {
        let result = run_query_with_file("users.json", "[*].name | sort(@)");
        assert!(result.contains("Alice Johnson"));
        assert!(result.contains("Eva Martinez"));
    }

    #[test]
    fn test_users_aggregation() {
        let result = run_query_with_file("users.json", "avg([*].salary)");
        assert_eq!(result, "95000.0");
    }

    #[test]
    fn test_users_group_by() {
        let result = run_query_with_file(
            "users.json",
            "group_by_expr('department', @) | map_values('length(@)', @)",
        );
        assert!(result.contains("\"Engineering\": 3"));
        assert!(result.contains("\"Marketing\": 1"));
        assert!(result.contains("\"Sales\": 1"));
    }
}

mod cli_options {
    use super::*;

    #[test]
    fn test_compact_output() {
        let mut child = jpx_cmd()
            .arg("-c")
            .arg("[*].a")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"[{\"a\": 1}, {\"a\": 2}]")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(result, "[1,2]");
    }

    #[test]
    fn test_raw_output() {
        let mut child = jpx_cmd()
            .arg("-r")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"\"hello world\"")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_list_functions() {
        // Use --list-category instead of --list
        let output = jpx_cmd()
            .arg("--list-category")
            .arg("array")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let result = String::from_utf8_lossy(&output.stdout);
        assert!(result.contains("unique"));
        assert!(result.contains("flatten"));
    }

    #[test]
    fn test_describe_function() {
        let output = jpx_cmd()
            .arg("--describe")
            .arg("unique")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let result = String::from_utf8_lossy(&output.stdout);
        assert!(result.contains("unique"));
        assert!(result.contains("array"));
    }

    #[test]
    fn test_version() {
        let output = jpx_cmd()
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let result = String::from_utf8_lossy(&output.stdout);
        assert!(result.contains("jpx"));
    }
}

mod path_functions {
    use super::*;

    #[test]
    fn test_get_path_dot_notation() {
        let result = run_query(r#"{"a": {"b": {"c": 42}}}"#, "get_path(@, `\"a.b.c\"`)");
        assert_eq!(result, "42");
    }

    #[test]
    fn test_get_path_with_default() {
        let result = run_query(r#"{"a": 1}"#, "get_path(@, `\"a.b.c\"`, `\"missing\"`)");
        assert_eq!(result, r#""missing""#);
    }

    #[test]
    fn test_get_path_array_index() {
        let result = run_query(
            r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#,
            "get_path(@, `\"users.0.name\"`)",
        );
        assert_eq!(result, r#""alice""#);
    }

    #[test]
    fn test_has_path_exists() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "has_path(@, `\"a.b\"`)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_has_path_missing() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "has_path(@, `\"a.c\"`)");
        assert_eq!(result, "false");
    }

    #[test]
    fn test_has_path_array_index() {
        let result = run_query(r#"{"items": [1, 2, 3]}"#, "has_path(@, `\"items.1\"`)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_set_path_dot_notation() {
        let result = run_query(r#"{"a": {}}"#, "set_path(@, `\"a.b\"`, `99`)");
        assert!(result.contains("\"b\": 99"));
    }

    #[test]
    fn test_set_path_creates_nested() {
        let result = run_query(r#"{}"#, "set_path(@, `\"a.b.c\"`, `\"deep\"`)");
        assert!(result.contains("\"c\": \"deep\""));
    }

    #[test]
    fn test_set_path_array_index() {
        let result = run_query(
            r#"{"items": [1, 2, 3]}"#,
            "set_path(@, `\"items.1\"`, `99`)",
        );
        assert!(result.contains("99"));
    }

    #[test]
    fn test_delete_path_dot_notation() {
        let result = run_query(r#"{"a": {"b": 1, "c": 2}}"#, "delete_path(@, `\"a.b\"`)");
        assert!(!result.contains("\"b\":"));
        assert!(result.contains("\"c\": 2"));
    }

    #[test]
    fn test_delete_path_array_index() {
        let result = run_query(r#"{"items": [1, 2, 3]}"#, "delete_path(@, `\"items.1\"`)");
        // After deleting index 1 (value 2), should have [1, 3]
        assert!(result.contains("1"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_get_alias_works() {
        // get and get_path should be equivalent
        let result1 = run_query(r#"{"a": {"b": 1}}"#, "get(@, `\"a.b\"`)");
        let result2 = run_query(r#"{"a": {"b": 1}}"#, "get_path(@, `\"a.b\"`)");
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_has_alias_works() {
        // has and has_path should be equivalent
        let result1 = run_query(r#"{"a": {"b": 1}}"#, "has(@, `\"a.b\"`)");
        let result2 = run_query(r#"{"a": {"b": 1}}"#, "has_path(@, `\"a.b\"`)");
        assert_eq!(result1, result2);
    }
}

mod object_functions {
    use super::*;

    #[test]
    fn test_pick() {
        let result = run_query(r#"{"a": 1, "b": 2, "c": 3}"#, "pick(@, `[\"a\", \"c\"]`)");
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 3"));
        assert!(!result.contains("\"b\""));
    }

    #[test]
    fn test_omit() {
        let result = run_query(r#"{"a": 1, "b": 2, "c": 3}"#, "omit(@, `[\"a\", \"c\"]`)");
        assert!(!result.contains("\"a\""));
        assert!(result.contains("\"b\": 2"));
        assert!(!result.contains("\"c\""));
    }

    #[test]
    fn test_deep_merge() {
        let result = run_query(
            r#"[{"a": {"b": 1}}, {"a": {"c": 2}}]"#,
            "deep_merge([0], [1])",
        );
        assert!(result.contains("\"b\": 1"));
        assert!(result.contains("\"c\": 2"));
    }

    #[test]
    fn test_defaults() {
        let result = run_query(r#"{"a": 1}"#, "defaults(@, `{\"a\": 99, \"b\": 2}`)");
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"b\": 2"));
    }

    #[test]
    fn test_rename_keys() {
        let result = run_query(
            r#"{"old_name": 1}"#,
            "rename_keys(@, `{\"old_name\": \"new_name\"}`)",
        );
        assert!(result.contains("\"new_name\": 1"));
        assert!(!result.contains("\"old_name\""));
    }

    #[test]
    fn test_invert() {
        let result = run_query(r#"{"a": "x", "b": "y"}"#, "invert(@)");
        assert!(result.contains("\"x\": \"a\""));
        assert!(result.contains("\"y\": \"b\""));
    }
}

mod type_functions {
    use super::*;

    #[test]
    fn test_to_boolean_string_true() {
        let result = run_query(r#""true""#, "to_boolean(@)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_to_boolean_string_yes() {
        let result = run_query(r#""yes""#, "to_boolean(@)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_to_boolean_number() {
        let result = run_query(r#"1"#, "to_boolean(@)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_to_boolean_zero() {
        let result = run_query(r#"0"#, "to_boolean(@)");
        assert_eq!(result, "false");
    }

    #[test]
    fn test_parse_numbers() {
        let result = run_query(r#"{"count": "42", "name": "alice"}"#, "parse_numbers(@)");
        assert!(result.contains("\"count\": 42"));
        assert!(result.contains("\"name\": \"alice\""));
    }

    #[test]
    fn test_parse_booleans() {
        let result = run_query(
            r#"{"active": "true", "name": "alice"}"#,
            "parse_booleans(@)",
        );
        assert!(result.contains("\"active\": true"));
        assert!(result.contains("\"name\": \"alice\""));
    }

    #[test]
    fn test_auto_parse() {
        let result = run_query(
            r#"{"num": "42", "bool": "true", "nil": "null", "str": "hello"}"#,
            "auto_parse(@)",
        );
        assert!(result.contains("\"num\": 42"));
        assert!(result.contains("\"bool\": true"));
        assert!(result.contains("\"nil\": null"));
        assert!(result.contains("\"str\": \"hello\""));
    }

    #[test]
    fn test_type_of() {
        assert_eq!(run_query(r#""hello""#, "type_of(@)"), r#""string""#);
        assert_eq!(run_query(r#"42"#, "type_of(@)"), r#""number""#);
        assert_eq!(run_query(r#"true"#, "type_of(@)"), r#""boolean""#);
        assert_eq!(run_query(r#"[1,2]"#, "type_of(@)"), r#""array""#);
        assert_eq!(run_query(r#"{"a":1}"#, "type_of(@)"), r#""object""#);
    }

    #[test]
    fn test_is_string() {
        assert_eq!(run_query(r#""hello""#, "is_string(@)"), "true");
        assert_eq!(run_query(r#"42"#, "is_string(@)"), "false");
    }

    #[test]
    fn test_is_number() {
        assert_eq!(run_query(r#"42"#, "is_number(@)"), "true");
        assert_eq!(run_query(r#""42""#, "is_number(@)"), "false");
    }

    #[test]
    fn test_is_empty() {
        assert_eq!(run_query(r#"[]"#, "is_empty(@)"), "true");
        assert_eq!(run_query(r#"{}"#, "is_empty(@)"), "true");
        assert_eq!(run_query(r#""""#, "is_empty(@)"), "true");
        assert_eq!(run_query(r#"[1]"#, "is_empty(@)"), "false");
    }
}

mod cleanup_functions {
    use super::*;

    #[test]
    fn test_remove_nulls() {
        let result = run_query(r#"{"a": 1, "b": null, "c": 2}"#, "remove_nulls(@)");
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 2"));
        assert!(!result.contains("null"));
    }

    #[test]
    fn test_remove_empty() {
        let result = run_query(
            r#"{"a": "", "b": "hello", "c": [], "d": [1]}"#,
            "remove_empty(@)",
        );
        assert!(result.contains("\"b\": \"hello\""));
        assert!(result.contains("\"d\""));
    }

    #[test]
    fn test_remove_empty_strings() {
        let result = run_query(
            r#"{"name": "alice", "bio": "", "city": "nyc"}"#,
            "remove_empty_strings(@)",
        );
        assert!(result.contains("\"name\": \"alice\""));
        assert!(result.contains("\"city\": \"nyc\""));
        assert!(!result.contains("\"bio\""));
    }
}

mod array_functions {
    use super::*;

    #[test]
    fn test_index_by_simple() {
        let result = run_query(
            r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#,
            "index_by(@, 'id')",
        );
        assert!(result.contains("\"a\":"));
        assert!(result.contains("\"b\":"));
    }

    #[test]
    fn test_index_at() {
        assert_eq!(run_query(r#"[1, 2, 3]"#, "index_at(@, `0`)"), "1");
        assert_eq!(run_query(r#"[1, 2, 3]"#, "index_at(@, `-1`)"), "3");
    }

    #[test]
    fn test_find_index() {
        assert_eq!(run_query(r#"[1, 2, 3]"#, "find_index(@, `2`)"), "1");
    }

    #[test]
    fn test_zip() {
        let result = run_query(r#"{"a": [1, 2], "b": ["x", "y"]}"#, "zip(a, b)");
        assert!(result.contains("1"));
        assert!(result.contains("\"x\""));
    }

    #[test]
    fn test_intersection() {
        let result = run_query(r#"[[1, 2, 3], [2, 3, 4]]"#, "intersection([0], [1])");
        assert!(result.contains("2"));
        assert!(result.contains("3"));
        assert!(!result.contains("1"));
        assert!(!result.contains("4"));
    }

    #[test]
    fn test_difference() {
        let result = run_query(r#"[[1, 2, 3], [2, 3, 4]]"#, "difference([0], [1])");
        assert!(result.contains("1"));
        assert!(!result.contains("2"));
    }
}

mod flatten_unflatten {
    use super::*;

    #[test]
    fn test_flatten_keys() {
        let result = run_query(r#"{"a": {"b": {"c": 1}}}"#, "flatten_keys(@)");
        assert!(result.contains("\"a.b.c\": 1"));
    }

    #[test]
    fn test_flatten_alias() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "flatten(@)");
        assert!(result.contains("\"a.b\": 1"));
    }

    #[test]
    fn test_unflatten_keys() {
        let result = run_query(r#"{"a.b.c": 1, "a.b.d": 2}"#, "unflatten_keys(@)");
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\""));
        assert!(result.contains("\"c\": 1"));
        assert!(result.contains("\"d\": 2"));
    }

    #[test]
    fn test_unflatten_alias() {
        let result = run_query(r#"{"a.b": 1}"#, "unflatten(@)");
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\": 1"));
    }

    #[test]
    fn test_flatten_array() {
        let result = run_query(r#"{"a": [1, 2]}"#, "flatten_array(@)");
        assert!(result.contains("\"a.0\": 1"));
        assert!(result.contains("\"a.1\": 2"));
    }
}

mod key_functions {
    use super::*;

    #[test]
    fn test_pluck_deep() {
        let result = run_query(
            r#"{"users": [{"id": 1}, {"id": 2}], "meta": {"id": 99}}"#,
            "pluck_deep(@, 'id')",
        );
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("99"));
    }

    #[test]
    fn test_paths_to() {
        let result = run_query(r#"{"a": {"id": 1}, "b": {"id": 2}}"#, "paths_to(@, 'id')");
        assert!(result.contains("a.id"));
        assert!(result.contains("b.id"));
    }

    #[test]
    fn test_camel_keys() {
        let result = run_query(r#"{"hello_world": 1, "foo_bar": 2}"#, "camel_keys(@)");
        assert!(result.contains("helloWorld"));
        assert!(result.contains("fooBar"));
    }

    #[test]
    fn test_snake_keys() {
        let result = run_query(r#"{"helloWorld": 1, "fooBar": 2}"#, "snake_keys(@)");
        assert!(result.contains("hello_world"));
        assert!(result.contains("foo_bar"));
    }

    #[test]
    fn test_kebab_keys() {
        let result = run_query(r#"{"helloWorld": 1}"#, "kebab_keys(@)");
        assert!(result.contains("hello-world"));
    }

    #[test]
    fn test_leaves() {
        let result = run_query(r#"{"a": 1, "b": {"c": 2, "d": 3}}"#, "leaves(@)");
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_paths() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "paths(@)");
        assert!(result.contains("/a"));
        assert!(result.contains("/a/b"));
    }
}

mod redaction {
    use super::*;

    #[test]
    fn test_mask_default() {
        let result = run_query(r#""4111111111111111""#, "mask(@)");
        assert!(result.contains("*"));
        assert!(result.contains("1111")); // shows last 4 by default
    }

    #[test]
    fn test_mask_custom_visible() {
        let result = run_query(r#""secret123""#, "mask(@, `3`)");
        // Should show last 3 characters
        assert!(result.contains("123"));
        assert!(result.contains("*"));
    }

    #[test]
    fn test_redact() {
        let result = run_query(
            r#"{"password": "secret", "name": "alice"}"#,
            "redact(@, `[\"password\"]`)",
        );
        assert!(result.contains("\"name\": \"alice\""));
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("secret"));
    }

    #[test]
    fn test_redact_keys_pattern() {
        let result = run_query(
            r#"{"password": "secret", "api_key": "xyz", "name": "alice"}"#,
            "redact_keys(@, 'password|api_key')",
        );
        assert!(result.contains("\"name\": \"alice\""));
        assert!(!result.contains("secret"));
        assert!(!result.contains("xyz"));
    }
}

mod math_stats {
    use super::*;

    #[test]
    fn test_quartiles() {
        let result = run_query(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#, "quartiles(@)");
        assert!(result.contains("q1"));
        assert!(result.contains("q2"));
        assert!(result.contains("q3"));
    }

    #[test]
    fn test_outliers_iqr() {
        let result = run_query(r#"[1, 2, 3, 4, 5, 100]"#, "outliers_iqr(@)");
        // 100 should be detected as an outlier
        assert!(result.contains("100"));
    }

    #[test]
    fn test_outliers_zscore() {
        let result = run_query(r#"[10, 10, 10, 10, 10, 100]"#, "outliers_zscore(@)");
        // 100 should be detected as an outlier
        assert!(result.contains("100"));
    }

    #[test]
    fn test_percentile() {
        let result = run_query(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#, "percentile(@, `50`)");
        // 50th percentile (median) should be around 5.5
        assert!(result.contains("5"));
    }

    #[test]
    fn test_median() {
        let result = run_query(r#"[1, 2, 3, 4, 5]"#, "median(@)");
        assert!(result.starts_with("3")); // Could be 3 or 3.0
    }

    #[test]
    fn test_stddev() {
        let result = run_query(r#"[2, 4, 4, 4, 5, 5, 7, 9]"#, "stddev(@)");
        assert!(result.contains("2")); // stddev is 2
    }

    #[test]
    fn test_variance() {
        let result = run_query(r#"[2, 4, 4, 4, 5, 5, 7, 9]"#, "variance(@)");
        assert!(result.contains("4")); // variance is 4
    }
}

mod string_functions {
    use super::*;

    #[test]
    fn test_upper() {
        assert_eq!(run_query(r#""hello""#, "upper(@)"), r#""HELLO""#);
    }

    #[test]
    fn test_lower() {
        assert_eq!(run_query(r#""HELLO""#, "lower(@)"), r#""hello""#);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(
            run_query(r#""hello world""#, "capitalize(@)"),
            r#""Hello world""#
        );
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            run_query(r#""hello world""#, "replace(@, 'world', 'there')"),
            r#""hello there""#
        );
    }

    #[test]
    fn test_pad_left() {
        let result = run_query(r#""42""#, "pad_left(@, `5`, '0')");
        assert_eq!(result, r#""00042""#);
    }

    #[test]
    fn test_pad_right() {
        let result = run_query(r#""hi""#, "pad_right(@, `5`, '.')");
        assert_eq!(result, r#""hi...""#);
    }

    #[test]
    fn test_repeat() {
        assert_eq!(run_query(r#""ab""#, "repeat(@, `3`)"), r#""ababab""#);
    }

    #[test]
    fn test_substr() {
        let result = run_query(r#""hello world""#, "substr(@, `0`, `5`)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_slice() {
        let result = run_query(r#""hello world""#, "slice(@, `0`, `5`)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_is_blank() {
        assert_eq!(run_query(r#""   ""#, "is_blank(@)"), "true");
        assert_eq!(run_query(r#""hello""#, "is_blank(@)"), "false");
    }

    #[test]
    fn test_ltrimstr() {
        assert_eq!(run_query(r#""foobar""#, "ltrimstr(@, 'foo')"), r#""bar""#);
    }

    #[test]
    fn test_rtrimstr() {
        assert_eq!(run_query(r#""foobar""#, "rtrimstr(@, 'bar')"), r#""foo""#);
    }
}

mod encoding_functions {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let result = run_query(r#""hello""#, "base64_encode(@)");
        assert_eq!(result, r#""aGVsbG8=""#);
    }

    #[test]
    fn test_base64_decode() {
        let result = run_query(r#""aGVsbG8=""#, "base64_decode(@)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_hex_encode() {
        let result = run_query(r#""hello""#, "hex_encode(@)");
        assert_eq!(result, r#""68656c6c6f""#);
    }

    #[test]
    fn test_hex_decode() {
        let result = run_query(r#""68656c6c6f""#, "hex_decode(@)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_url_encode() {
        let result = run_query(r#""hello world""#, "url_encode(@)");
        assert!(result.contains("hello%20world") || result.contains("hello+world"));
    }

    #[test]
    fn test_url_decode() {
        let result = run_query(r#""hello%20world""#, "url_decode(@)");
        assert_eq!(result, r#""hello world""#);
    }
}

mod hash_functions {
    use super::*;

    #[test]
    fn test_md5() {
        let result = run_query(r#""hello""#, "md5(@)");
        assert_eq!(result, r#""5d41402abc4b2a76b9719d911017c592""#);
    }

    #[test]
    fn test_sha256() {
        let result = run_query(r#""hello""#, "sha256(@)");
        // SHA256 of "hello"
        assert!(result.len() > 60); // SHA256 hex is 64 chars + quotes
    }
}

mod datetime_functions {
    use super::*;

    #[test]
    fn test_now() {
        let result = run_query(r#"null"#, "type(now())");
        assert_eq!(result, r#""number""#);
    }

    #[test]
    fn test_format_date() {
        let result = run_query(r#"0"#, "format_date(@, '%Y')");
        assert_eq!(result, r#""1970""#);
    }

    #[test]
    fn test_date_add() {
        // Add 1 day (86400 seconds) to epoch
        let result = run_query(r#"0"#, "date_add(@, `1`, 'day')");
        assert!(result.starts_with("86400")); // Could be 86400 or 86400.0
    }
}

mod regex_functions {
    use super::*;

    #[test]
    fn test_regex_match() {
        assert_eq!(run_query(r#""hello123""#, "regex_match(@, '\\d+')"), "true");
        assert_eq!(run_query(r#""hello""#, "regex_match(@, '\\d+')"), "false");
    }

    #[test]
    fn test_regex_replace() {
        let result = run_query(r#""hello123world456""#, "regex_replace(@, '\\d+', 'X')");
        assert_eq!(result, r#""helloXworldX""#);
    }

    #[test]
    fn test_regex_extract() {
        let result = run_query(r#""hello123""#, "regex_extract(@, '(\\d+)')");
        assert!(result.contains("123"));
    }
}

mod validation_functions {
    use super::*;

    #[test]
    fn test_is_email() {
        assert_eq!(run_query(r#""test@example.com""#, "is_email(@)"), "true");
        assert_eq!(run_query(r#""not-an-email""#, "is_email(@)"), "false");
    }

    #[test]
    fn test_is_url() {
        assert_eq!(run_query(r#""https://example.com""#, "is_url(@)"), "true");
        assert_eq!(run_query(r#""not a url""#, "is_url(@)"), "false");
    }

    #[test]
    fn test_is_ipv4() {
        assert_eq!(run_query(r#""192.168.1.1""#, "is_ipv4(@)"), "true");
        assert_eq!(run_query(r#""999.999.999.999""#, "is_ipv4(@)"), "false");
    }

    #[test]
    fn test_is_uuid() {
        assert_eq!(
            run_query(r#""550e8400-e29b-41d4-a716-446655440000""#, "is_uuid(@)"),
            "true"
        );
        assert_eq!(run_query(r#""not-a-uuid""#, "is_uuid(@)"), "false");
    }
}

mod cli_bench {
    use super::*;

    fn run_with_flags(json: &str, flags: &[&str]) -> (String, String) {
        let mut cmd = jpx_cmd();
        for flag in flags {
            cmd.arg(flag);
        }
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(json.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        (
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    }

    #[test]
    fn test_bench_basic() {
        let (stdout, _) = run_with_flags(
            r#"[{"name": "alice"}, {"name": "bob"}]"#,
            &["--bench=10", "--warmup=2", "[*].name"],
        );
        assert!(stdout.contains("BENCHMARK"));
        assert!(stdout.contains("Expression:"));
        assert!(stdout.contains("Mean:"));
        assert!(stdout.contains("Median:"));
        assert!(stdout.contains("Throughput:"));
    }

    #[test]
    fn test_bench_shows_iterations() {
        let (stdout, _) = run_with_flags(r#"{"value": 42}"#, &["--bench=25", "value"]);
        assert!(stdout.contains("25"));
        assert!(stdout.contains("Iterations:"));
    }

    #[test]
    fn test_bench_shows_distribution() {
        let (stdout, _) = run_with_flags(r#"[1, 2, 3]"#, &["--bench=20", "@"]);
        // Distribution should appear for >= 10 iterations
        assert!(stdout.contains("Distribution"));
    }

    #[test]
    fn test_bench_pipeline() {
        let (stdout, _) = run_with_flags(r#"[3, 1, 2]"#, &["--bench=10", "@", "sort(@)"]);
        // Pipeline expressions should be joined with |
        assert!(stdout.contains("@ | sort(@)"));
    }
}

mod cli_paths {
    use super::*;

    fn run_with_flags(json: &str, flags: &[&str]) -> String {
        let mut cmd = jpx_cmd();
        for flag in flags {
            cmd.arg(flag);
        }
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(json.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[test]
    fn test_paths_basic() {
        let result = run_with_flags(r#"{"name": "alice", "age": 30}"#, &["--paths"]);
        assert!(result.contains("."));
        assert!(result.contains("name"));
        assert!(result.contains("age"));
    }

    #[test]
    fn test_paths_nested() {
        let result = run_with_flags(r#"{"user": {"name": "alice"}}"#, &["--paths"]);
        assert!(result.contains("user"));
        assert!(result.contains("user.name"));
    }

    #[test]
    fn test_paths_array() {
        let result = run_with_flags(r#"{"items": [1, 2, 3]}"#, &["--paths"]);
        assert!(result.contains("items"));
        assert!(result.contains("items[0]"));
        assert!(result.contains("items[1]"));
        assert!(result.contains("items[2]"));
    }

    #[test]
    fn test_paths_with_types() {
        let result = run_with_flags(r#"{"name": "alice", "age": 30}"#, &["--paths", "--types"]);
        assert!(result.contains("string"));
        assert!(result.contains("number"));
        assert!(result.contains("object"));
    }

    #[test]
    fn test_paths_with_values() {
        let result = run_with_flags(r#"{"name": "alice"}"#, &["--paths", "--values"]);
        assert!(result.contains("\"alice\""));
    }

    #[test]
    fn test_paths_with_types_and_values() {
        let result = run_with_flags(r#"{"count": 42}"#, &["--paths", "--types", "--values"]);
        assert!(result.contains("number"));
        assert!(result.contains("42"));
    }
}

mod cli_table {
    use super::*;

    fn run_with_flags(json: &str, flags: &[&str]) -> String {
        let mut cmd = jpx_cmd();
        for flag in flags {
            cmd.arg(flag);
        }
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(json.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[test]
    fn test_table_basic() {
        let result = run_with_flags(
            r#"[{"name": "alice", "age": 30}]"#,
            &["@", "--table", "--color", "never"],
        );
        assert!(result.contains("name"));
        assert!(result.contains("age"));
        assert!(result.contains("alice"));
        assert!(result.contains("30"));
    }

    #[test]
    fn test_table_multiple_rows() {
        let result = run_with_flags(
            r#"[{"name": "alice"}, {"name": "bob"}]"#,
            &["@", "--table", "--color", "never"],
        );
        assert!(result.contains("alice"));
        assert!(result.contains("bob"));
    }

    #[test]
    fn test_table_ascii_style() {
        let result = run_with_flags(
            r#"[{"name": "alice"}]"#,
            &["@", "--table", "--table-style", "ascii"],
        );
        assert!(result.contains("+"));
        assert!(result.contains("-"));
        assert!(result.contains("|"));
    }

    #[test]
    fn test_table_markdown_style() {
        let result = run_with_flags(
            r#"[{"name": "alice"}]"#,
            &["@", "--table", "--table-style", "markdown"],
        );
        // Markdown tables use | for columns
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 2);
        assert!(lines[0].contains("|"));
        // Second line should be the separator with dashes
        assert!(lines[1].contains("-"));
    }

    #[test]
    fn test_table_with_expression() {
        let result = run_with_flags(
            r#"[{"user": {"name": "alice"}}, {"user": {"name": "bob"}}]"#,
            &["[*].user", "--table", "--color", "never"],
        );
        assert!(result.contains("alice"));
        assert!(result.contains("bob"));
    }

    #[test]
    fn test_table_short_flag() {
        let result = run_with_flags(r#"[{"name": "alice"}]"#, &["@", "-t", "--color", "never"]);
        assert!(result.contains("name"));
        assert!(result.contains("alice"));
    }
}

mod output_formats {
    use super::*;

    fn run_query_with_flag(json: &str, query: &str, flag: &str) -> String {
        let mut child = jpx_cmd()
            .arg(flag)
            .arg(query)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(json.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[test]
    fn test_yaml_output_object() {
        let result = run_query_with_flag(r#"{"name": "alice", "age": 30}"#, "@", "--yaml");
        assert!(result.contains("name: alice"));
        assert!(result.contains("age: 30"));
    }

    #[test]
    fn test_yaml_output_array() {
        let result = run_query_with_flag(r#"[{"name": "alice"}, {"name": "bob"}]"#, "@", "--yaml");
        assert!(result.contains("- name: alice"));
        assert!(result.contains("- name: bob"));
    }

    #[test]
    fn test_yaml_output_nested() {
        let result = run_query_with_flag(
            r#"{"user": {"name": "alice", "skills": ["rust", "python"]}}"#,
            "@",
            "--yaml",
        );
        assert!(result.contains("user:"));
        assert!(result.contains("name: alice"));
        assert!(result.contains("- rust"));
        assert!(result.contains("- python"));
    }

    #[test]
    fn test_toml_output_object() {
        let result = run_query_with_flag(r#"{"name": "alice", "age": 30}"#, "@", "--toml");
        assert!(result.contains("name = \"alice\""));
        assert!(result.contains("age = 30"));
    }

    #[test]
    fn test_toml_output_nested() {
        let result = run_query_with_flag(
            r#"{"server": {"host": "localhost", "port": 8080}}"#,
            "@",
            "--toml",
        );
        assert!(result.contains("[server]"));
        assert!(result.contains("host = \"localhost\""));
        assert!(result.contains("port = 8080"));
    }

    #[test]
    fn test_csv_output_array_of_objects() {
        let result = run_query_with_flag(
            r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]"#,
            "@",
            "--csv",
        );
        // Should have header and two data rows
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        // Header should contain both column names
        assert!(lines[0].contains("name"));
        assert!(lines[0].contains("age"));
        // Data rows
        assert!(result.contains("alice"));
        assert!(result.contains("bob"));
    }

    #[test]
    fn test_csv_output_nested_flattening() {
        let result = run_query_with_flag(
            r#"[{"name": "alice", "address": {"city": "NYC", "zip": "10001"}}]"#,
            "@",
            "--csv",
        );
        // Nested keys should be flattened with dot notation
        assert!(result.contains("address.city"));
        assert!(result.contains("address.zip"));
        assert!(result.contains("NYC"));
        assert!(result.contains("10001"));
    }

    #[test]
    fn test_csv_output_array_in_cell() {
        let result = run_query_with_flag(
            r#"[{"name": "alice", "tags": ["dev", "admin"]}]"#,
            "@",
            "--csv",
        );
        // Arrays should be JSON-encoded in cells (CSV escapes quotes as "")
        assert!(result.contains("alice"));
        assert!(result.contains("dev") && result.contains("admin"));
    }

    #[test]
    fn test_tsv_output_array_of_objects() {
        let result = run_query_with_flag(
            r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]"#,
            "@",
            "--tsv",
        );
        // Should be tab-separated
        assert!(result.contains('\t'));
        assert!(result.contains("alice"));
        assert!(result.contains("bob"));
    }

    #[test]
    fn test_lines_output_array() {
        let result = run_query_with_flag(r#"[1, 2, 3, "hello"]"#, "@", "--lines");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "2");
        assert_eq!(lines[2], "3");
        assert_eq!(lines[3], "\"hello\"");
    }

    #[test]
    fn test_lines_output_array_of_objects() {
        let result = run_query_with_flag(r#"[{"id": 1}, {"id": 2}]"#, "@", "--lines");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"id\":1") || lines[0].contains("\"id\": 1"));
        assert!(lines[1].contains("\"id\":2") || lines[1].contains("\"id\": 2"));
    }

    #[test]
    fn test_lines_output_with_expression() {
        let result = run_query_with_flag(
            r#"[{"name": "alice"}, {"name": "bob"}]"#,
            "[*].name",
            "--lines",
        );
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "\"alice\"");
        assert_eq!(lines[1], "\"bob\"");
    }

    #[test]
    fn test_yaml_short_flag() {
        let result = run_query_with_flag(r#"{"name": "alice"}"#, "@", "-y");
        assert!(result.contains("name: alice"));
    }

    #[test]
    fn test_lines_short_flag() {
        let result = run_query_with_flag(r#"[1, 2, 3]"#, "@", "-l");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_csv_single_object() {
        let result = run_query_with_flag(r#"{"name": "alice", "age": 30}"#, "@", "--csv");
        // Single object should be output as single-row table
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 data row
        assert!(result.contains("alice"));
    }

    #[test]
    fn test_csv_array_of_primitives() {
        let result = run_query_with_flag(r#"[1, 2, 3]"#, "@", "--csv");
        // Array of primitives - single column, no header
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_invalid_json() {
        let mut child = jpx_cmd()
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"not valid json")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(!output.status.success());
    }

    #[test]
    fn test_invalid_query() {
        let mut child = jpx_cmd()
            .arg("[[[invalid")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{}")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(!output.status.success());
    }
}

mod cli_similar {
    use super::*;

    fn run_similar(func_name: &str) -> (String, String) {
        let output = jpx_cmd()
            .arg("--similar")
            .arg(func_name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait on jpx");

        (
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    }

    #[test]
    fn test_similar_basic() {
        let (stdout, _) = run_similar("upper");
        assert!(stdout.contains("Functions similar to"));
        assert!(stdout.contains("Same category"));
        assert!(stdout.contains("string"));
    }

    #[test]
    fn test_similar_shows_signature_matches() {
        let (stdout, _) = run_similar("upper");
        // upper has signature string -> string, should find similar signatures
        assert!(stdout.contains("Similar signature"));
    }

    #[test]
    fn test_similar_shows_related_concepts() {
        let (stdout, _) = run_similar("sort_by");
        // sort_by should match related sorting/ordering functions
        assert!(stdout.contains("Related concepts") || stdout.contains("Same category"));
    }

    #[test]
    fn test_similar_unknown_function() {
        let output = jpx_cmd()
            .arg("--similar")
            .arg("not_a_real_function")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait on jpx");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Unknown function"));
    }
}

mod cli_stream {
    use super::*;

    fn run_stream(input: &str, query: &str) -> String {
        let mut child = jpx_cmd()
            .arg("--stream")
            .arg(query)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn run_stream_raw(input: &str, query: &str) -> String {
        let mut child = jpx_cmd()
            .arg("--stream")
            .arg("-r")
            .arg(query)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[test]
    fn test_stream_basic() {
        let input = r#"{"id":1}
{"id":2}
{"id":3}"#;
        let result = run_stream(input, "id");
        assert_eq!(result, "1\n2\n3");
    }

    #[test]
    fn test_stream_with_raw_output() {
        let input = r#"{"name":"alice"}
{"name":"bob"}
{"name":"charlie"}"#;
        let result = run_stream_raw(input, "name");
        assert_eq!(result, "alice\nbob\ncharlie");
    }

    #[test]
    fn test_stream_skips_empty_lines() {
        let input = r#"{"id":1}

{"id":2}

{"id":3}"#;
        let result = run_stream(input, "id");
        assert_eq!(result, "1\n2\n3");
    }

    #[test]
    fn test_stream_skips_null_results() {
        let input = r#"{"id":1,"name":"alice"}
{"id":2}
{"id":3,"name":"charlie"}"#;
        let result = run_stream(input, "name");
        // Second line has no "name" field, result is null, should be skipped
        assert_eq!(result, "\"alice\"\n\"charlie\"");
    }

    #[test]
    fn test_stream_with_nested_fields() {
        let input = r#"{"user":{"name":"alice"}}
{"user":{"name":"bob"}}"#;
        let result = run_stream_raw(input, "user.name");
        assert_eq!(result, "alice\nbob");
    }

    #[test]
    fn test_stream_with_array_access() {
        let input = r#"{"items":[1,2,3]}
{"items":[4,5,6]}"#;
        let result = run_stream(input, "items[0]");
        assert_eq!(result, "1\n4");
    }

    #[test]
    fn test_stream_with_expression_function() {
        let input = r#"{"name":"alice"}
{"name":"bob"}"#;
        let result = run_stream_raw(input, "upper(name)");
        assert_eq!(result, "ALICE\nBOB");
    }

    #[test]
    fn test_stream_alias_each() {
        let input = r#"{"id":1}
{"id":2}"#;
        let mut child = jpx_cmd()
            .arg("--each") // Use alias instead of --stream
            .arg("id")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(result, "1\n2");
    }

    #[test]
    fn test_stream_conflicts_with_slurp() {
        let output = jpx_cmd()
            .arg("--stream")
            .arg("--slurp")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait on jpx");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("cannot be used with"));
    }

    #[test]
    fn test_stream_verbose_shows_line_count() {
        let input = r#"{"id":1}
{"id":2}
{"id":3}"#;
        let mut child = jpx_cmd()
            .arg("--stream")
            .arg("-v")
            .arg("id")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output().expect("Failed to wait on jpx");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Processed 3 lines"));
    }
}

// ============================================================================
// CLI Debug Flag Tests
// ============================================================================

mod cli_debug {
    use super::*;

    #[test]
    fn test_debug_shows_version() {
        let mut child = jpx_cmd()
            .arg("--debug")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"null")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Version:"), "Debug should show version");
    }

    #[test]
    fn test_debug_shows_environment() {
        let mut child = jpx_cmd()
            .arg("--debug")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"null")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Environment:"),
            "Debug should show environment section"
        );
        assert!(
            stderr.contains("JPX_VERBOSE"),
            "Debug should list JPX_VERBOSE env var"
        );
    }

    #[test]
    fn test_debug_shows_expressions() {
        let mut child = jpx_cmd()
            .arg("--debug")
            .arg("name")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{\"name\": \"test\"}")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Expressions:"),
            "Debug should show expressions section"
        );
        assert!(stderr.contains("name"), "Debug should show the expression");
    }

    #[test]
    fn test_debug_shows_functions_count() {
        let mut child = jpx_cmd()
            .arg("--debug")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"null")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Functions:"),
            "Debug should show functions section"
        );
        assert!(
            stderr.contains("registered:"),
            "Debug should show registered function count"
        );
    }
}

// ============================================================================
// CLI Stats Flag Tests
// ============================================================================

mod cli_stats {
    use super::*;

    #[test]
    fn test_stats_array() {
        let mut child = jpx_cmd()
            .arg("--stats")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"[1, 2, 3, 4, 5]")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("array"), "Stats should show type as array");
        assert!(
            stdout.contains("5") || stdout.contains("Length"),
            "Stats should show length"
        );
    }

    #[test]
    fn test_stats_object() {
        let mut child = jpx_cmd()
            .arg("--stats")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{\"name\": \"alice\", \"age\": 30}")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("object"),
            "Stats should show type as object"
        );
        assert!(
            stdout.contains("2") || stdout.contains("Keys"),
            "Stats should show key count"
        );
    }

    #[test]
    fn test_stats_array_of_objects() {
        let mut child = jpx_cmd()
            .arg("--stats")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"[{\"name\": \"alice\"}, {\"name\": \"bob\"}]")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("array"), "Stats should show type as array");
        // Should show field analysis for arrays of objects
        assert!(
            stdout.contains("name") || stdout.contains("Field"),
            "Stats should analyze fields"
        );
    }

    #[test]
    fn test_stats_shows_depth() {
        let mut child = jpx_cmd()
            .arg("--stats")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{\"a\": {\"b\": {\"c\": 1}}}")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Depth") || stdout.contains("depth") || stdout.contains("level"),
            "Stats should show depth"
        );
    }
}

// ============================================================================
// CLI Search Flag Tests
// ============================================================================

mod cli_search {
    use super::*;

    #[test]
    fn test_search_finds_functions() {
        let output = jpx_cmd()
            .arg("--search")
            .arg("upper")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("upper"),
            "Search should find 'upper' function"
        );
    }

    #[test]
    fn test_search_by_category() {
        let output = jpx_cmd()
            .arg("--search")
            .arg("hash")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should find hash-related functions like md5, sha256
        assert!(
            stdout.contains("md5") || stdout.contains("sha") || stdout.contains("hash"),
            "Search should find hash functions"
        );
    }

    #[test]
    fn test_search_no_results() {
        let output = jpx_cmd()
            .arg("--search")
            .arg("xyznonexistent123")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("No functions found") || stdout.contains("0"),
            "Search should indicate no results"
        );
    }

    #[test]
    fn test_search_by_description() {
        let output = jpx_cmd()
            .arg("--search")
            .arg("distance")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should find functions with "distance" in description (like geo functions or levenshtein)
        assert!(
            stdout.contains("geo")
                || stdout.contains("levenshtein")
                || stdout.contains("haversine"),
            "Search should find functions by description"
        );
    }
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

mod cli_env_vars {
    use super::*;

    #[test]
    fn test_jpx_raw_env_var() {
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .env("JPX_RAW", "1")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"\"hello\"")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // With JPX_RAW=1, string should be output without quotes
        assert_eq!(stdout, "hello");
    }

    #[test]
    fn test_jpx_compact_env_var() {
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .env("JPX_COMPACT", "1")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{\"a\": 1, \"b\": 2}")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Compact output should be on one line with no extra spaces
        assert!(
            !stdout.contains('\n'),
            "Compact output should be single line"
        );
    }

    #[test]
    fn test_cli_flag_overrides_env_var() {
        // Set JPX_RAW=1 via env, then use -r explicitly too (should work the same)
        // This tests that explicit CLI flags work alongside env vars
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .env("JPX_RAW", "1")
            .arg("-r") // Explicit flag alongside env var
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run jpx");

        use std::io::Write;
        child.stdin.take().unwrap().write_all(b"\"test\"").unwrap();

        let output = child.wait_with_output().expect("Failed to wait");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // With raw mode, string output should not have quotes
        assert_eq!(stdout.trim(), "test");
    }
}

mod cli_null_input {
    use super::*;

    #[test]
    fn test_null_input_basic() {
        let output = run_with_args(&["--null-input", "now()"], "");
        assert!(output.status.success());
        // now() returns a Unix timestamp (float)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let timestamp: f64 = stdout.trim().parse().expect("Should be a number");
        assert!(timestamp > 1700000000.0); // After Nov 2023
    }

    #[test]
    fn test_null_input_short_flag() {
        let output = run_with_args(&["-n", "`42`"], "");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "42");
    }

    #[test]
    fn test_null_input_with_literal() {
        let output = run_with_args(&["-n", "`{\"key\": \"value\"}`"], "");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("key"));
        assert!(stdout.contains("value"));
    }

    #[test]
    fn test_null_input_ignores_stdin() {
        // Even with stdin provided, -n should use null
        // When result is null, jpx outputs nothing (empty)
        let output = run_with_args(&["-n", "@"], r#"{"ignored": true}"#);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Null input means @ evaluates to null, which produces empty output
        assert!(stdout.trim().is_empty() || stdout.trim() == "null");
    }
}

mod cli_slurp {
    use super::*;

    #[test]
    fn test_slurp_multiple_json_values() {
        let output = run_with_args(&["--slurp", "@"], "1\n2\n3");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should collect into array
        assert!(stdout.contains("["));
        assert!(stdout.contains("1"));
        assert!(stdout.contains("2"));
        assert!(stdout.contains("3"));
    }

    #[test]
    fn test_slurp_short_flag() {
        let output = run_with_args(&["-s", "length(@)"], "1\n2\n3");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "3");
    }

    #[test]
    fn test_slurp_objects() {
        let output = run_with_args(
            &["-s", "[*].name"],
            "{\"name\": \"alice\"}\n{\"name\": \"bob\"}",
        );
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("alice"));
        assert!(stdout.contains("bob"));
    }

    #[test]
    fn test_slurp_with_expression() {
        let output = run_with_args(&["-s", "sum(@)"], "10\n20\n30");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // sum() returns float
        assert!(stdout.trim() == "60" || stdout.trim() == "60.0");
    }
}

mod cli_explain {
    use super::*;

    #[test]
    fn test_explain_simple_expression() {
        // --explain doesn't need stdin, it just parses the expression
        let output = run_with_args(&["--explain", "name"], "");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show AST information
        assert!(stdout.contains("Field") || stdout.contains("field") || stdout.contains("name"));
    }

    #[test]
    fn test_explain_function_call() {
        let output = run_with_args(&["--explain", "length(@)"], "");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("length") || stdout.contains("Function"));
    }

    #[test]
    fn test_explain_complex_expression() {
        let output = run_with_args(&["--explain", "items[?price > `100`].name"], "");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should parse without error and show structure
        assert!(stdout.len() > 10);
    }

    #[test]
    fn test_explain_pipeline() {
        let output = run_with_args(&["--explain", "items", "sort(@)"], "");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("sort") || stdout.contains("items"));
    }
}

mod cli_diff_patch_merge {
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_diff_generates_patch() {
        let mut source = NamedTempFile::new().unwrap();
        let mut target = NamedTempFile::new().unwrap();

        writeln!(source, r#"{{"name": "alice", "age": 30}}"#).unwrap();
        writeln!(target, r#"{{"name": "alice", "age": 31}}"#).unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("--diff")
            .arg(source.path())
            .arg(target.path())
            .output()
            .expect("Failed to run jpx");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should generate RFC 6902 JSON Patch
        assert!(stdout.contains("op"));
        assert!(stdout.contains("replace") || stdout.contains("add") || stdout.contains("remove"));
        assert!(stdout.contains("age") || stdout.contains("/age"));
    }

    #[test]
    fn test_diff_identical_files() {
        let mut source = NamedTempFile::new().unwrap();
        let mut target = NamedTempFile::new().unwrap();

        writeln!(source, r#"{{"name": "alice"}}"#).unwrap();
        writeln!(target, r#"{{"name": "alice"}}"#).unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("--diff")
            .arg(source.path())
            .arg(target.path())
            .output()
            .expect("Failed to run jpx");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Empty patch for identical files
        assert!(stdout.trim() == "[]");
    }

    #[test]
    fn test_patch_applies_operations() {
        let mut doc = NamedTempFile::new().unwrap();
        let mut patch = NamedTempFile::new().unwrap();

        writeln!(doc, r#"{{"name": "alice", "age": 30}}"#).unwrap();
        writeln!(
            patch,
            r#"[{{"op": "replace", "path": "/age", "value": 31}}]"#
        )
        .unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("--patch")
            .arg(patch.path())
            .arg("-f")
            .arg(doc.path())
            .output()
            .expect("Failed to run jpx");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("31"));
        assert!(stdout.contains("alice"));
    }

    #[test]
    fn test_patch_add_operation() {
        let mut doc = NamedTempFile::new().unwrap();
        let mut patch = NamedTempFile::new().unwrap();

        writeln!(doc, r#"{{"name": "alice"}}"#).unwrap();
        writeln!(
            patch,
            r#"[{{"op": "add", "path": "/email", "value": "alice@example.com"}}]"#
        )
        .unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("--patch")
            .arg(patch.path())
            .arg("-f")
            .arg(doc.path())
            .output()
            .expect("Failed to run jpx");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("email"));
        assert!(stdout.contains("alice@example.com"));
    }

    #[test]
    fn test_merge_patch_basic() {
        let mut doc = NamedTempFile::new().unwrap();
        let mut merge = NamedTempFile::new().unwrap();

        writeln!(doc, r#"{{"name": "alice", "age": 30}}"#).unwrap();
        writeln!(merge, r#"{{"age": 31, "city": "NYC"}}"#).unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("--merge")
            .arg(merge.path())
            .arg("-f")
            .arg(doc.path())
            .output()
            .expect("Failed to run jpx");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("alice"));
        assert!(stdout.contains("31"));
        assert!(stdout.contains("NYC"));
    }

    #[test]
    fn test_merge_patch_removes_null() {
        let mut doc = NamedTempFile::new().unwrap();
        let mut merge = NamedTempFile::new().unwrap();

        writeln!(doc, r#"{{"name": "alice", "age": 30, "temp": "data"}}"#).unwrap();
        writeln!(merge, r#"{{"temp": null}}"#).unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("--merge")
            .arg(merge.path())
            .arg("-f")
            .arg(doc.path())
            .output()
            .expect("Failed to run jpx");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("alice"));
        assert!(stdout.contains("30"));
        // temp should be removed
        assert!(!stdout.contains("temp"));
    }
}

mod cli_query_file {
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_query_file_basic() {
        let mut query_file = NamedTempFile::new().unwrap();
        writeln!(query_file, "name").unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("-Q")
            .arg(query_file.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run jpx");

        use std::io::Write;
        let mut child = output;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(br#"{"name": "alice"}"#)
            .unwrap();

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("alice"));
    }

    #[test]
    fn test_query_file_complex_expression() {
        let mut query_file = NamedTempFile::new().unwrap();
        writeln!(query_file, "items[?price > `50`].name | sort(@)").unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jpx"))
            .arg("-Q")
            .arg(query_file.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run jpx");

        let mut child = output;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                br#"{"items": [{"name": "a", "price": 100}, {"name": "b", "price": 30}, {"name": "c", "price": 75}]}"#,
            )
            .unwrap();

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("a"));
        assert!(stdout.contains("c"));
        assert!(!stdout.contains("b")); // price 30 < 50
    }
}
