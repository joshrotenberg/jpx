"""
jpx - JMESPath query engine with 490+ functions.

Provides both module-level convenience functions and a full-featured JpxEngine class.

Quick start:
    >>> import jpx
    >>> jpx.search("upper(name)", {"name": "alice"})
    'ALICE'

    >>> jpx.search("users[*].name | sort(@)", {"users": [{"name": "bob"}, {"name": "alice"}]})
    ['alice', 'bob']

Engine class:
    >>> engine = jpx.JpxEngine()
    >>> engine.evaluate("length(@)", [1, 2, 3])
    3
    >>> engine.search_functions("hash", limit=5)
    [...]
"""

from jpx._core import (
    CompiledExpression,
    JpxEngine,
    __version__,
    compile,
    describe,
    list_categories,
    list_functions,
    search,
    validate,
)

__all__ = [
    "search",
    "compile",
    "validate",
    "list_functions",
    "list_categories",
    "describe",
    "CompiledExpression",
    "JpxEngine",
    "__version__",
]
