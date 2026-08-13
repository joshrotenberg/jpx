# Query Store

The query store lets you save, retrieve, and execute named JMESPath queries while one server process is running.

## Workflow

### Define and iterate
```
define_query("users-by-role", "group_by(@, &role)")
run_query("users-by-role", <user data>)
# Refine...
define_query("users-by-role", "group_by([?active], &role)")
run_query("users-by-role", <user data>)
```

### Build a library of queries
```
define_query("active-users", "[?status == 'active']", "Filter to active users only")
define_query("top-scores", "sort_by(@, &score) | reverse(@) | [:10]", "Top 10 by score")
define_query("error-rate", "length([?level == 'error']) / length(@)", "Error rate")
```

### Review stored queries
```
list_queries -> shows all names, expressions, and descriptions
get_query("active-users") -> returns expression and description
```

### Clean up
```
delete_query("old-query")
```

## Notes

- Queries are process-scoped, shared across connected clients, and don't persist across server restarts
- Expressions are validated on `define_query` -- invalid syntax is rejected
- `define_query` with an existing name overwrites the previous query
- `run_query` parses the input JSON and applies the stored expression
- Descriptions are optional but help when reviewing stored queries with `list_queries`
