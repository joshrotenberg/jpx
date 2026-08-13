"""Type stubs for jpx._core"""

from typing import Any

__version__: str

class CompiledExpression:
    """A compiled JMESPath expression for efficient repeated searches."""

    @property
    def expression(self) -> str:
        """The expression string."""
        ...

    def search(self, data: Any) -> Any:
        """
        Search JSON data using this compiled expression.

        Args:
            data: JSON-compatible Python data (dict, list, str, int, float, bool, None)

        Returns:
            The result of evaluating the expression against the data

        Raises:
            ValueError: If evaluation fails
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class JpxEngine:
    """Full-featured JMESPath query engine with introspection, query store, and JSON utilities."""

    def __init__(self, strict: bool = False) -> None:
        """
        Create a new JpxEngine.

        Args:
            strict: If True, only standard JMESPath functions are available for evaluation.
        """
        ...

    @property
    def strict(self) -> bool:
        """Whether this engine is in strict mode."""
        ...

    # Evaluation

    def evaluate(self, expression: str, data: Any) -> Any:
        """
        Evaluate a JMESPath expression against data.

        Args:
            expression: A JMESPath expression string
            data: JSON-compatible Python data

        Returns:
            The result of evaluating the expression

        Raises:
            ValueError: If the expression is invalid or evaluation fails
        """
        ...

    def evaluate_str(self, expression: str, json_str: str) -> Any:
        """
        Evaluate a JMESPath expression against a JSON string.

        Args:
            expression: A JMESPath expression string
            json_str: A JSON string

        Returns:
            The result of evaluating the expression

        Raises:
            ValueError: If the JSON is invalid, expression is invalid, or evaluation fails
        """
        ...

    def batch_evaluate(
        self, expressions: list[str], data: Any
    ) -> list[dict[str, Any]]:
        """
        Evaluate multiple expressions against the same data.

        Args:
            expressions: A list of JMESPath expression strings
            data: JSON-compatible Python data

        Returns:
            A list of dicts with "expression", "result", and "error" keys
        """
        ...

    def validate_expression(self, expression: str) -> dict[str, Any]:
        """
        Validate a JMESPath expression without evaluating it.

        Args:
            expression: A JMESPath expression string

        Returns:
            A dict with "valid" (bool) and "error" (str | None)
        """
        ...

    def explain(self, expression: str) -> dict[str, Any]:
        """
        Explain a JMESPath expression step by step.

        Args:
            expression: A JMESPath expression string

        Returns:
            A dict with "expression", "steps", "functions_used", and "complexity"

        Raises:
            ValueError: If the expression is invalid
        """
        ...

    # Introspection

    def categories(self) -> list[str]:
        """List all available function categories."""
        ...

    def functions(self, category: str | None = None) -> list[dict[str, Any]]:
        """
        List functions, optionally filtered by category.

        Args:
            category: Optional category name (case-insensitive)

        Returns:
            A list of dicts with function details
        """
        ...

    def describe_function(self, name: str) -> dict[str, Any] | None:
        """
        Get detailed information about a function by name.

        Args:
            name: The function name

        Returns:
            A dict with function details, or None if not found
        """
        ...

    def search_functions(
        self, query: str, limit: int = 10
    ) -> list[dict[str, Any]]:
        """
        Search for functions matching a query string.

        Args:
            query: Search term (e.g., "hash", "string", "date")
            limit: Maximum number of results (default 10)

        Returns:
            A list of dicts with function details, match_type, and score
        """
        ...

    def similar_functions(self, name: str) -> dict[str, Any] | None:
        """
        Find functions similar to a given function.

        Args:
            name: Function name

        Returns:
            A dict with "same_category", "similar_signature", "related_concepts" lists,
            or None if the function is not found
        """
        ...

    # JSON utilities

    def format_json(self, json_str: str, indent: int = 2) -> str:
        """
        Format JSON with configurable indentation.

        Args:
            json_str: JSON string to format
            indent: Spaces per indent level (default 2, 0 = compact)

        Returns:
            Formatted JSON string

        Raises:
            ValueError: If the input is not valid JSON
        """
        ...

    def diff(self, source: str, target: str) -> list[dict[str, Any]]:
        """
        Generate a JSON Patch (RFC 6902) from two JSON strings.

        Args:
            source: Original JSON string
            target: Modified JSON string

        Returns:
            A list of patch operations

        Raises:
            ValueError: If either input is not valid JSON
        """
        ...

    def patch(self, json_str: str, patch_str: str) -> Any:
        """
        Apply a JSON Patch (RFC 6902) to a document.

        Args:
            json_str: JSON document string
            patch_str: JSON array of patch operations

        Returns:
            The patched document

        Raises:
            ValueError: If the JSON or patch is invalid
        """
        ...

    def merge(self, json_str: str, patch_str: str) -> Any:
        """
        Apply a JSON Merge Patch (RFC 7396) to a document.

        Args:
            json_str: JSON document string
            patch_str: JSON merge patch object

        Returns:
            The merged document

        Raises:
            ValueError: If either input is not valid JSON
        """
        ...

    def stats(self, json_str: str) -> dict[str, Any]:
        """
        Analyze JSON data and return structural statistics.

        Args:
            json_str: JSON string to analyze

        Returns:
            A dict with root_type, size_bytes, size_human, depth, and more

        Raises:
            ValueError: If the input is not valid JSON
        """
        ...

    def paths(
        self,
        json_str: str,
        include_types: bool = True,
        include_values: bool = False,
    ) -> list[dict[str, Any]]:
        """
        Extract all paths from a JSON document.

        Args:
            json_str: JSON string to analyze
            include_types: Include type info (default True)
            include_values: Include leaf values (default False)

        Returns:
            A list of dicts with "path", optionally "path_type" and "value"

        Raises:
            ValueError: If the input is not valid JSON
        """
        ...

    def keys(self, json_str: str, recursive: bool = False) -> list[str]:
        """
        Extract keys from a JSON object.

        Args:
            json_str: JSON string
            recursive: If True, extract all nested paths in dot notation (default False)

        Returns:
            A list of key strings

        Raises:
            ValueError: If the input is not valid JSON
        """
        ...

    # Query store

    def define_query(
        self,
        name: str,
        expression: str,
        description: str | None = None,
    ) -> dict[str, Any] | None:
        """
        Store a named query for reuse.

        Args:
            name: Query name
            expression: JMESPath expression string
            description: Optional description

        Returns:
            The previously stored query dict if one existed, else None

        Raises:
            ValueError: If the expression is invalid
        """
        ...

    def get_query(self, name: str) -> dict[str, Any] | None:
        """
        Get a stored query by name.

        Args:
            name: Query name

        Returns:
            A dict with "name", "expression", "description", or None if not found
        """
        ...

    def run_query(self, name: str, data: Any) -> Any:
        """
        Run a stored query against data.

        Args:
            name: Query name
            data: JSON-compatible Python data

        Returns:
            The result of evaluating the stored expression

        Raises:
            ValueError: If the query is not found or evaluation fails
        """
        ...

    def list_queries(self) -> list[dict[str, Any]]:
        """
        List all stored queries.

        Returns:
            A list of dicts with "name", "expression", "description"
        """
        ...

    def delete_query(self, name: str) -> dict[str, Any] | None:
        """
        Delete a stored query.

        Args:
            name: Query name

        Returns:
            The deleted query dict if it existed, else None
        """
        ...

    def __repr__(self) -> str: ...

# Module-level convenience functions

def search(expression: str, data: Any) -> Any:
    """
    Search JSON data using a JMESPath expression with 490+ functions.

    Args:
        expression: A JMESPath expression string
        data: JSON-compatible Python data (dict, list, str, int, float, bool, None)

    Returns:
        The result of evaluating the expression against the data

    Raises:
        ValueError: If the expression is invalid or evaluation fails

    Example:
        >>> import jpx
        >>> jpx.search("upper(name)", {"name": "alice"})
        'ALICE'
    """
    ...

def compile(expression: str) -> CompiledExpression:
    """
    Compile a JMESPath expression for repeated use.

    Args:
        expression: A JMESPath expression string

    Returns:
        A CompiledExpression object

    Raises:
        ValueError: If the expression is invalid

    Example:
        >>> import jpx
        >>> expr = jpx.compile("users[*].name")
        >>> expr.search({"users": [{"name": "alice"}]})
        ['alice']
    """
    ...

def validate(expression: str) -> dict[str, Any]:
    """
    Validate a JMESPath expression without evaluating it.

    Args:
        expression: A JMESPath expression string

    Returns:
        A dict with "valid" (bool) and "error" (str | None)

    Example:
        >>> import jpx
        >>> jpx.validate("users[*].name")
        {'valid': True, 'error': None}
    """
    ...

def list_functions(category: str | None = None) -> list[str]:
    """
    List all available function names, optionally filtered by category.

    Args:
        category: Optional category name (case-insensitive)

    Returns:
        A list of function name strings

    Example:
        >>> import jpx
        >>> "upper" in jpx.list_functions("String")
        True
    """
    ...

def list_categories() -> list[str]:
    """
    List all available function categories.

    Returns:
        A list of category name strings

    Example:
        >>> import jpx
        >>> "String" in jpx.list_categories()
        True
    """
    ...

def describe(name: str) -> dict[str, Any] | None:
    """
    Get information about a specific function.

    Args:
        name: The function name

    Returns:
        A dict with function info (name, category, description, signature, example, is_standard),
        or None if not found

    Example:
        >>> import jpx
        >>> info = jpx.describe("upper")
        >>> info["description"]
        'Convert string to uppercase'
    """
    ...
