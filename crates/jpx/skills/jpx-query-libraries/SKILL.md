---
name: jpx-query-libraries
description: Create, validate, and run reusable named JMESPath queries in jpx .jpx libraries. Use when saving related queries, selecting a query with -Q, or checking a library before use.
---

# jpx query libraries

## Try a complete example

With `jpx` on PATH, run this POSIX shell example in a scratch directory. It
creates a library and JSON input, validates both queries, and selects each
query explicitly:

```sh
cat > users.jpx <<'JPX'
-- :name active-names
-- :desc Names of active users; expects an object with a users array
users[?active == `true`].name

-- :name all-names
-- :desc All user names in input order
users[*].name
JPX

cat > users.json <<'JSON'
{"users":[{"name":"Ada","active":true},{"name":"Lin","active":false}]}
JSON

jpx -Q users.jpx --check
jpx -c -Q users.jpx:active-names -f users.json
jpx -c -Q users.jpx --query all-names -f users.json
# Same filter directly: single quotes protect the backticks from the shell.
jpx -c 'users[?active == `true`].name' -f users.json
```

The check exits successfully and reports `All queries valid.`. The three
query results are `["Ada"]`, `["Ada","Lin"]`, and `["Ada"]`, in that order.

## Adapt the library

- Start every query with `-- :name <query-name>`. Use unique, descriptive names.
  Add an optional `-- :desc` to state the expected input and intended result.
- Put the expression on the following line. Expressions may span lines; the
  next `-- :name` starts a new query. Blank lines and `-- ` comments are ignored.
- Keep the `.jpx` extension. A plain expression file is a different format and
  does not provide named queries.
- Select one query with `-Q library.jpx:query-name -f data.json`, or use
  `-Q library.jpx --query query-name -f data.json`. Use explicit `-f` to keep
  the input path separate from expressions and library selection.
- Inspect names with `jpx -Q library.jpx --list-queries`. A library containing
  multiple queries needs a name when executed; a single query can be selected
  automatically.
- After editing, run `jpx -Q library.jpx --check`. It checks syntax and function
  names without reading input. Then run against representative JSON and verify
  the result: validation alone cannot catch data-dependent type errors.

## Preserve literals

Backticks delimit JSON literals in JMESPath: numbers, booleans, null, arrays,
objects, and double-quoted JSON strings. For a string comparison, write
`` status == `"active"` ``; a bare word inside backticks is not valid JSON.
Single-quoted JMESPath strings are raw strings, useful for function arguments.

The quoted heredoc delimiter in the example preserves backticks and dollar
signs in the file. For inline shell expressions, single-quote the entire
expression; unprotected backticks perform shell command substitution. Prefer
a query file when mixing JMESPath raw strings with shell quoting gets awkward.

## Recover from errors

If a file reports no queries, add `-- :name <query-name>` before its expression.
For an unknown name, use the available names in the error or `--list-queries`.
Rename duplicate query names. For a parse error, check literal quoting and
validate again before executing with `-f`.

## Save this skill

To keep a local copy, run from the directory where you want a `skills` folder:

```sh
mkdir -p skills/jpx-query-libraries
jpx --skills query-libraries > skills/jpx-query-libraries/SKILL.md
```

The output is this entire standalone skill, including YAML frontmatter. Place
the `jpx-query-libraries` folder in your agent's supported skill directory to
install it. No companion files or network access are required.
