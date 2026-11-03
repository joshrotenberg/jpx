---
name: jpx-shell-quoting
description: Write jpx commands that survive the shell. Use when an expression contains backticks, $variables, quotes, or hyphenated keys, when passing shell values into a query, or when a filter unexpectedly returns [] or nothing.
---

# Quote jpx expressions in the shell

JMESPath uses backticks for JSON literals and `$name` for variables. A POSIX
shell treats both specially inside double quotes, so the expression jpx
receives can differ from the one you wrote, and the result is usually `[]` or
nothing rather than an error.

Run the examples in order in one scratch directory. Each `text` block shows
exactly what the example prints.

## See what the shell passes

`printf` shows the argument jpx would get. Inside double quotes the backticks
run `true` as a command, and `$role` expands to a shell value:

```sh
role=dev
printf '%s\n' "users[?active == `true`].name" "users[?role == $role].name"
```

```text
users[?active == ].name
users[?role == dev].name
```

## Single-quote the whole expression

Single quotes pass everything through unchanged:

```sh
cat > users.json <<'JSON'
{"users":[{"name":"Ada","active":true,"role":"admin","age":36},
 {"name":"Lin","active":false,"role":"dev","age":29},
 {"name":"O'Brien","active":true,"role":"dev","age":41}]}
JSON
jpx -c 'users[?active == `true`].name' -f users.json
jpx -c 'users[?active == true].name' -f users.json
```

```text
["Ada","O'Brien"]
[]
```

The second filter shows a separate trap: a bare `true` is a field named
`true`, not the boolean, so the comparison matches nothing. Write literals in
backticks: `` `true` ``, `` `30` ``, `` `null` ``.

## Pass shell values with --arg and --argjson

Do not splice shell variables into the expression. `--arg NAME VALUE` binds
`$NAME` to a string; `--argjson NAME VALUE` parses the value as JSON, so use
it for numbers, booleans and arrays. A number passed with `--arg` is a string,
and comparing a number with a string matches nothing:

```sh
role=dev
min=30
jpx -c --arg role "$role" --argjson min "$min" \
  'users[?role == $role && age > $min].name' -f users.json
jpx -c --arg min "$min" 'users[?age > $min].name' -f users.json
```

```text
["O'Brien"]
[]
```

## Write strings inside single quotes

For a fixed string, use a backtick JSON string, which needs no shell escaping.
For a value that contains an apostrophe, pass it with `--arg`. A JMESPath raw
string (`'Ada'`) inside a single-quoted command needs `'\''` around it, so
avoid it:

```sh
jpx -c 'users[?role == `"dev"`].name' -f users.json
jpx -c --arg who "O'Brien" 'users[?name == $who].age' -f users.json
```

```text
["Lin","O'Brien"]
[41]
```

## Quote keys with special characters

A key with a hyphen, a dot or a leading digit, such as an HTTP header name,
needs a double-quoted identifier. Inside a single-quoted expression those
double quotes reach jpx unchanged:

```sh
printf '%s\n' '{"headers":{"x-request-id":"r-17","content-type":"text/plain"}}' |
  jpx -r 'headers."x-request-id"'
```

```text
r-17
```

## Move long expressions into a file

When an expression mixes raw strings, literals and variables, put it in a
`.jpx` query file instead. Quote the heredoc delimiter (`<<'JPX'`) so the shell
leaves the body alone. `jpx --skills query-libraries` covers the format.

```sh
cat > checks.jpx <<'JPX'
-- :name senior-admins
users[?role == 'admin' && age > `30`].name
JPX
jpx -c -Q checks.jpx:senior-admins -f users.json
```

```text
["Ada"]
```

## Save this skill

To keep a local copy, run from the directory where you want a `skills` folder:

```sh
mkdir -p skills/jpx-shell-quoting
jpx --skills shell-quoting > skills/jpx-shell-quoting/SKILL.md
```

The output is this entire standalone skill, including YAML frontmatter. Place
the `jpx-shell-quoting` folder in your agent's supported skill directory to
install it. No companion files or network access are required.
