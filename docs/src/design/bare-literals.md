# Bare literals: compatibility decision for #221

**Recommendation: retain current semantics in both default and strict mode.**
The proposal to accept bare `null`, `true`, `false`, and numbers in non-strict
mode is not purely additive: the words already name fields, including comparison
operands. Shell quoting and query files solve the escaping problem today.
This is a recommendation for issue resolution; no new grammar is implemented.

## Executable compatibility evidence

`crates/jpx/tests/cli_bare_literals.rs` runs each case in default mode and with
`--strict`, asserting results or failure status. Both modes have the same
behavior for these expressions. Reproduce from the repository root:

```text
cargo test -p jpx --test cli_bare_literals --offline
```

For each word `W` in `null`, `true`, `false`:

| Input JSON | Expression | Result (both modes) |
| --- | --- | --- |
| `{"W":"field"}` (substitute the actual key) | `W` or `"W"` | `"field"` |
| Same object | backtick-delimited `W` | JSON null/boolean |
| Same object | `'W'` | The string spelling of the word |
| `{}` | `W` | `null` (missing field) |
| `{"value":7,"W":7}` | `value == W` | `true` |
| Same object | `value ==` backtick-delimited `W` | `false` |

These are expression results: the CLI emits no stdout for a top-level `null`
(exit 0 without `-x`). The tests also wrap null results in a multi-select list
to verify the observable JSON `[null]`.

The tests also cover nested field access and filters. For example, on
`[{"id":"field","value":7,"true":7},{"id":"literal","value":true,"true":7}]`,
`[?value == true].id` returns `["field"]`, whereas
``[?value == `true`].id`` returns `["literal"]`.
Equivalent cases distinguish bare `null` and `false` from their literals.
Limiting the proposal to comparison right-hand sides would still break these
valid field-to-field comparisons.

Missing fields make the ambiguity especially easy to miss: `value == null`
returns `true` for both `{}` and `{"value":null}`. Adding `"null":7` to the
latter object changes the result to `false`; ``value == `null` `` remains
`true`. Likewise, `value == true` on `{"value":true}` and `value == false` on
`{"value":false}` both return `false`, because the RHS field is missing.

Numbers need a separate analysis:

| Expression | Input JSON | Result (both modes) |
| --- | --- | --- |
| `0`, `42`, `-42`, `-0`, `1.5`, `-1.5`, `1e3`, `-1e3` | `{}` | Parse error, exit 1 |
| `value == 42`, `value == -42` (also decimals/exponents) | Matching numeric `value` | Parse error, exit 1 |
| ``value == `42` ``, ``value == `-42` `` (also decimals/exponents) | Matching numeric `value` | `true` |
| `[0]`, `[-1]` | `[10,20,30]` | `10`, `30` (indices) |
| `[1:3]`, `[-2:]`, `[::-1]` | `[10,20,30]` | `[20,30]`, `[20,30]`, `[30,20,10]` (slices) |
| ``[`0`]``, ``[`-1`]`` | `[10,20,30]` | `[0]`, `[-1]` (multi-select lists) |
| `` `[0]` ``, `` `[-1]` `` | `[10,20,30]` | `[0]`, `[-1]` (literal arrays) |
| `["42", "-42"]` | `{"42":"positive","-42":"negative"}` | `["positive","negative"]` (quoted fields) |

The tests cover numeric filter RHSs too. Numeric probes pass the expression
after the existing `--` option separator: without it, `jpx -n -42` fails in
argument parsing (exit 2). `jpx -n -- -42` reaches the expression parser but
still fails (exit 1). Shell quotes alone do not stop a leading minus sign from
being treated as an option. A backtick literal works: ``jpx -n '`-42`'``.

Standalone numbers currently fail rather than selecting numeric-named fields,
so a carefully scoped numbers-only extension might be additive in those value
positions. It cannot simply reinterpret all numeric tokens: `[0]` and `[-1]`
already mean indexing, not singleton literal lists. Such an extension would need
an explicit precedence rule, JSON numeric syntax/range decisions, and a separate
compatibility review. This investigation does not claim that it is impossible.

## Safe shell recipes using existing flags

These snippets are executed verbatim by the CLI regression test through
`/bin/sh`, in both default and strict mode. They use POSIX shell quoting; no
backslash before a backtick is needed inside shell single quotes.

```sh
printf '%s\n' '{"null":"field","true":"field","false":"field"}' |
  jpx -c '[`null`, `true`, `false`, `42`, `-42`]'
# [null,true,false,42,-42]
```

Single-quote the whole expression, including comparisons:

```sh
printf '%s\n' '[{"id":1,"enabled":true,"score":-2,"deleted":null},{"id":2,"enabled":false,"score":3,"deleted":null}]' |
  jpx -c '[?enabled == `true` && score >= `-2` && deleted == `null`].id'
# [1]
```

When an expression also contains JMESPath raw strings (`'ready'`), use a query
file to avoid nested shell quoting. Run this in a scratch directory: it creates
`rows.json` and `queries.jpx` (overwriting files with those names).
The quoted here-document delimiters prevent shell expansion of the contents.

```sh
cat > rows.json <<'JSON'
[{"id":1,"state":"ready","enabled":true,"score":-2,"deleted":null},{"id":2,"state":"ready","enabled":false,"score":3,"deleted":null}]
JSON
cat > queries.jpx <<'JPX'
-- :name ready
[?state == 'ready' && enabled == `true` && score >= `-2` && deleted == `null`].id
JPX
jpx -Q queries.jpx --check >/dev/null
jpx -c -Q queries.jpx:ready -f rows.json
# [1]
```

Add `--strict` to each `jpx` invocation to use strict mode. `--check` validates
the query library; it does not prove that a query selects the intended records.

## Decision and limits

Making the three words literals by default would silently change successful
queries. Requiring callers to rewrite existing fields as `"null"`, `"true"`, or
`"false"` is a migration, not an additive feature. Resolving them as fields only
when present would make literal interpretation depend on the input data.
Keeping old semantics only under `--strict` would still break existing default
mode callers. Retain the semantics and publish the recipes above.

The evidence is for this checkout's CLI default build and explicit `--strict`,
with inherited `JPX_*` settings removed and strict config explicitly disabled
before applying the flag. It is not an exhaustive grammar/number-range audit,
a test of other JMESPath implementations, or a proposed opt-in dialect.
Strict mode disables extension functions and let expressions; it does not
currently change these literal/identifier rules.

Shell regression coverage uses `/bin/sh` on Unix (skipped on non-Unix targets);
it makes no PowerShell, cmd.exe, or fish quoting claim. Shell single quotes
cannot themselves contain a single quote, hence the query-file recipe.
Direct process/API callers pass backticks as ordinary argument characters and
should not add shell quoting. Existing bare-number errors can expose parser
terms such as `Unexpected nud token` without a corrective hint; improving those
diagnostics is separate from this documentation/test-only decision.
