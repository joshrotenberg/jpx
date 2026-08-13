"""Tests for jpx Python bindings."""

import jpx
import pytest
from jpx import CompiledExpression, JpxEngine

# =============================================================================
# Module-level convenience functions
# =============================================================================


class TestSearch:
    def test_simple_field(self):
        assert jpx.search("name", {"name": "alice"}) == "alice"

    def test_nested_field(self):
        data = {"user": {"name": "alice"}}
        assert jpx.search("user.name", data) == "alice"

    def test_array_projection(self):
        data = {"users": [{"name": "alice"}, {"name": "bob"}]}
        assert jpx.search("users[*].name", data) == ["alice", "bob"]

    def test_extension_function(self):
        assert jpx.search("upper(name)", {"name": "alice"}) == "ALICE"

    def test_length(self):
        assert jpx.search("length(@)", [1, 2, 3]) == 3

    def test_null_result(self):
        assert jpx.search("missing", {"name": "alice"}) is None

    def test_filter(self):
        data = {"items": [1, 2, 3, 4, 5]}
        result = jpx.search("items[?@ > `3`]", data)
        assert result == [4, 5]

    def test_multi_select(self):
        data = {"name": "alice", "age": 30}
        result = jpx.search("{n: name, a: age}", data)
        assert result == {"n": "alice", "a": 30}

    def test_pipe(self):
        data = {"users": [{"name": "bob"}, {"name": "alice"}]}
        result = jpx.search("users[*].name | sort(@)", data)
        assert result == ["alice", "bob"]

    def test_invalid_expression(self):
        with pytest.raises(ValueError, match="Invalid expression"):
            jpx.search("invalid[", {"a": 1})

    def test_types(self):
        data = {
            "string": "hello",
            "number": 42,
            "float": 3.14,
            "bool": True,
            "null": None,
            "array": [1, 2],
            "object": {"a": 1},
        }
        assert jpx.search("string", data) == "hello"
        assert jpx.search("number", data) == 42
        assert jpx.search("float", data) == 3.14
        assert jpx.search("bool", data) is True
        assert jpx.search("null", data) is None
        assert jpx.search("array", data) == [1, 2]
        assert jpx.search("object", data) == {"a": 1}


class TestCompile:
    def test_compile_and_search(self):
        expr = jpx.compile("users[*].name")
        result = expr.search({"users": [{"name": "alice"}, {"name": "bob"}]})
        assert result == ["alice", "bob"]

    def test_reuse(self):
        expr = jpx.compile("length(@)")
        assert expr.search([1, 2, 3]) == 3
        assert expr.search([1]) == 1
        assert expr.search([]) == 0

    def test_expression_property(self):
        expr = jpx.compile("users[*].name")
        assert expr.expression == "users[*].name"

    def test_repr(self):
        expr = jpx.compile("name")
        assert "name" in repr(expr)

    def test_str(self):
        expr = jpx.compile("name")
        assert str(expr) == "name"

    def test_invalid(self):
        with pytest.raises(ValueError, match="Invalid expression"):
            jpx.compile("invalid[")

    def test_isinstance(self):
        expr = jpx.compile("name")
        assert isinstance(expr, CompiledExpression)


class TestValidate:
    def test_valid(self):
        result = jpx.validate("users[*].name")
        assert result["valid"] is True
        assert result["error"] is None

    def test_invalid(self):
        result = jpx.validate("users[*.name")
        assert result["valid"] is False
        assert result["error"] is not None
        assert isinstance(result["error"], str)

    def test_complex_valid(self):
        result = jpx.validate("users[?age > `30`].name | sort(@)")
        assert result["valid"] is True


class TestListFunctions:
    def test_all(self):
        funcs = jpx.list_functions()
        assert len(funcs) > 100

    def test_by_category(self):
        funcs = jpx.list_functions("String")
        assert "upper" in funcs
        assert "lower" in funcs

    def test_empty_category(self):
        funcs = jpx.list_functions("NonexistentCategory")
        assert len(funcs) == 0


class TestListCategories:
    def test_returns_list(self):
        cats = jpx.list_categories()
        assert isinstance(cats, list)
        assert len(cats) > 5

    def test_contains_common(self):
        cats = jpx.list_categories()
        assert "String" in cats
        assert "Math" in cats
        assert "Array" in cats


class TestDescribe:
    def test_existing(self):
        info = jpx.describe("upper")
        assert info is not None
        assert info["name"] == "upper"
        assert "description" in info
        assert "signature" in info

    def test_nonexistent(self):
        assert jpx.describe("nonexistent_function") is None


class TestVersion:
    def test_version_exists(self):
        assert isinstance(jpx.__version__, str)
        assert len(jpx.__version__) > 0


# =============================================================================
# JpxEngine class
# =============================================================================


class TestJpxEngineCreation:
    def test_default(self):
        engine = JpxEngine()
        assert engine.strict is False

    def test_strict(self):
        engine = JpxEngine(strict=True)
        assert engine.strict is True

    def test_repr(self):
        assert "JpxEngine()" == repr(JpxEngine())
        assert "strict=True" in repr(JpxEngine(strict=True))


class TestJpxEngineEvaluation:
    def test_evaluate(self):
        engine = JpxEngine()
        result = engine.evaluate("name", {"name": "alice"})
        assert result == "alice"

    def test_evaluate_extension(self):
        engine = JpxEngine()
        result = engine.evaluate("upper(name)", {"name": "alice"})
        assert result == "ALICE"

    def test_evaluate_str(self):
        engine = JpxEngine()
        result = engine.evaluate_str("length(@)", "[1, 2, 3]")
        assert result == 3

    def test_evaluate_str_invalid_json(self):
        engine = JpxEngine()
        with pytest.raises(ValueError, match="Invalid JSON"):
            engine.evaluate_str("@", "not json")

    def test_batch_evaluate(self):
        engine = JpxEngine()
        data = {"a": 1, "b": 2}
        results = engine.batch_evaluate(["a", "b", "c"], data)
        assert len(results) == 3
        assert results[0]["expression"] == "a"
        assert results[0]["result"] == 1
        assert results[0]["error"] is None
        assert results[1]["result"] == 2
        assert results[2]["result"] is None  # null for missing field

    def test_batch_evaluate_with_errors(self):
        engine = JpxEngine()
        results = engine.batch_evaluate(["a", "invalid["], {"a": 1})
        assert len(results) == 2
        assert results[0]["result"] == 1
        assert results[0]["error"] is None
        assert results[1]["result"] is None
        assert results[1]["error"] is not None

    def test_validate_expression(self):
        engine = JpxEngine()
        valid = engine.validate_expression("users[*].name")
        assert valid["valid"] is True
        assert valid["error"] is None

        invalid = engine.validate_expression("users[*.name")
        assert invalid["valid"] is False
        assert invalid["error"] is not None

    def test_explain(self):
        engine = JpxEngine()
        result = engine.explain("users[*].name | sort(@)")
        assert result["expression"] == "users[*].name | sort(@)"
        assert "sort" in result["functions_used"]
        assert isinstance(result["steps"], list)
        assert result["complexity"] in ("simple", "moderate", "complex")

    def test_explain_invalid(self):
        engine = JpxEngine()
        with pytest.raises(ValueError, match="Invalid expression"):
            engine.explain("invalid[")

    def test_strict_rejects_extensions(self):
        engine = JpxEngine(strict=True)
        with pytest.raises(ValueError, match="Unknown function: upper"):
            engine.evaluate("upper(name)", {"name": "alice"})


class TestJpxEngineIntrospection:
    def test_categories(self):
        engine = JpxEngine()
        cats = engine.categories()
        assert "String" in cats
        assert "Math" in cats

    def test_functions_all(self):
        engine = JpxEngine()
        funcs = engine.functions()
        assert isinstance(funcs, list)
        assert len(funcs) > 100

    def test_functions_by_category(self):
        engine = JpxEngine()
        funcs = engine.functions(category="String")
        assert isinstance(funcs, list)
        assert all(f["category"] == "String" for f in funcs)
        assert any(f["name"] == "upper" for f in funcs)

    def test_describe_function(self):
        engine = JpxEngine()
        info = engine.describe_function("upper")
        assert info is not None
        assert info["name"] == "upper"
        assert info["category"] == "String"

    def test_describe_function_nonexistent(self):
        engine = JpxEngine()
        assert engine.describe_function("nonexistent") is None

    def test_search_functions(self):
        engine = JpxEngine()
        results = engine.search_functions("hash", limit=5)
        assert isinstance(results, list)
        assert len(results) <= 5

    def test_similar_functions(self):
        engine = JpxEngine()
        result = engine.similar_functions("upper")
        assert result is not None
        assert "same_category" in result
        assert isinstance(result["same_category"], list)

    def test_similar_functions_nonexistent(self):
        engine = JpxEngine()
        assert engine.similar_functions("nonexistent") is None


class TestJpxEngineJsonUtils:
    def test_format_json(self):
        engine = JpxEngine()
        result = engine.format_json('{"a":1,"b":2}', indent=2)
        assert "\n" in result
        assert '"a"' in result

    def test_format_json_compact(self):
        engine = JpxEngine()
        result = engine.format_json('{"a":  1, "b": 2}', indent=0)
        assert "\n" not in result

    def test_format_json_invalid(self):
        engine = JpxEngine()
        with pytest.raises(ValueError, match="Invalid JSON"):
            engine.format_json("not json")

    def test_diff(self):
        engine = JpxEngine()
        patch = engine.diff('{"a": 1}', '{"a": 2}')
        assert isinstance(patch, list)
        assert len(patch) > 0

    def test_diff_identical(self):
        engine = JpxEngine()
        patch = engine.diff('{"a": 1}', '{"a": 1}')
        assert patch == []

    def test_patch(self):
        engine = JpxEngine()
        result = engine.patch(
            '{"a": 1}',
            '[{"op": "replace", "path": "/a", "value": 2}]',
        )
        assert result == {"a": 2}

    def test_merge(self):
        engine = JpxEngine()
        result = engine.merge('{"a": 1, "b": 2}', '{"b": 3, "c": 4}')
        assert result == {"a": 1, "b": 3, "c": 4}

    def test_stats(self):
        engine = JpxEngine()
        result = engine.stats("[1, 2, 3]")
        assert result["root_type"] == "array"
        assert result["length"] == 3
        assert result["depth"] == 1
        assert "size_bytes" in result
        assert "size_human" in result

    def test_stats_object(self):
        engine = JpxEngine()
        result = engine.stats('{"a": 1, "b": 2}')
        assert result["root_type"] == "object"
        assert result["key_count"] == 2

    def test_paths(self):
        engine = JpxEngine()
        result = engine.paths('{"user": {"name": "alice"}}')
        assert isinstance(result, list)
        paths = [p["path"] for p in result]
        assert "user.name" in paths

    def test_paths_with_values(self):
        engine = JpxEngine()
        result = engine.paths('{"name": "alice"}', include_types=True, include_values=True)
        name_path = next(p for p in result if p["path"] == "name")
        assert name_path["path_type"] == "string"
        assert name_path["value"] == "alice"

    def test_keys(self):
        engine = JpxEngine()
        result = engine.keys('{"b": 1, "a": 2}')
        assert sorted(result) == ["a", "b"]

    def test_keys_recursive(self):
        engine = JpxEngine()
        result = engine.keys('{"a": {"b": {"c": 1}}}', recursive=True)
        assert "a" in result
        assert "a.b" in result
        assert "a.b.c" in result


class TestJpxEngineQueryStore:
    def test_define_and_get(self):
        engine = JpxEngine()
        prev = engine.define_query("count", "length(@)")
        assert prev is None

        query = engine.get_query("count")
        assert query is not None
        assert query["name"] == "count"
        assert query["expression"] == "length(@)"

    def test_define_with_description(self):
        engine = JpxEngine()
        engine.define_query("count", "length(@)", description="Count elements")
        query = engine.get_query("count")
        assert query["description"] == "Count elements"

    def test_define_overwrites(self):
        engine = JpxEngine()
        engine.define_query("q", "length(@)")
        prev = engine.define_query("q", "keys(@)")
        assert prev is not None
        assert prev["expression"] == "length(@)"

    def test_run_query(self):
        engine = JpxEngine()
        engine.define_query("count", "length(@)")
        result = engine.run_query("count", [1, 2, 3])
        assert result == 3

    def test_run_query_not_found(self):
        engine = JpxEngine()
        with pytest.raises(ValueError, match="Query not found: nonexistent"):
            engine.run_query("nonexistent", [1, 2, 3])

    def test_list_queries(self):
        engine = JpxEngine()
        engine.define_query("a", "length(@)")
        engine.define_query("b", "keys(@)")
        queries = engine.list_queries()
        assert len(queries) == 2
        names = [q["name"] for q in queries]
        assert "a" in names
        assert "b" in names

    def test_delete_query(self):
        engine = JpxEngine()
        engine.define_query("count", "length(@)")
        deleted = engine.delete_query("count")
        assert deleted is not None
        assert deleted["name"] == "count"
        assert engine.get_query("count") is None

    def test_delete_nonexistent(self):
        engine = JpxEngine()
        assert engine.delete_query("nonexistent") is None

    def test_define_invalid_expression(self):
        engine = JpxEngine()
        with pytest.raises(ValueError, match="Invalid expression"):
            engine.define_query("bad", "invalid[")


# =============================================================================
# Numeric fidelity (#193)
# =============================================================================


class TestNumericFidelity:
    """Large integers must round-trip exactly, not degrade to floats."""

    def test_int_above_i64_roundtrips(self):
        # 2**63 exceeds i64::MAX; must stay an exact int, not become a float.
        n = 2**63
        result = jpx.search("@", n)
        assert result == n
        assert isinstance(result, int)

    def test_u64_max_roundtrips(self):
        n = 2**64 - 1
        result = jpx.search("@", n)
        assert result == n
        assert isinstance(result, int)

    def test_int_beyond_u64_raises(self):
        # Too large for 64-bit JSON integers: error rather than silently lossy.
        with pytest.raises(ValueError, match="exceeds 64-bit range"):
            jpx.search("@", 2**64)

    def test_regular_int_and_float_unaffected(self):
        assert jpx.search("@", 42) == 42
        assert isinstance(jpx.search("@", 42), int)
        assert jpx.search("@", 3.14) == 3.14
        assert isinstance(jpx.search("@", 3.14), float)

    def test_large_int_in_nested_structure(self):
        data = {"id": 2**63, "vals": [2**64 - 1, 1]}
        assert jpx.search("id", data) == 2**63
        assert jpx.search("vals[0]", data) == 2**64 - 1
