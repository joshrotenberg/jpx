# Agent recipe fixtures

`recipes.json` is the canonical set of `jpx` CLI recipes that docs and skills
show to agents. `crates/jpx/tests/agent_recipes.rs` runs every entry. Entries
with `shell_args` also check that the exact command fragment remains in their
referenced docs. Other doc paths are context links, not executable assertions.
When you change a recipe in the docs, update its fixture too.

## Schema

The file is a JSON array of objects:

| Field | Required | Meaning |
|-------|----------|---------|
| `id` | yes | Unique, stable name for the recipe. |
| `outcome` | yes | What the recipe is meant to show, in one sentence. |
| `docs` | no | Repo-relative paths of the docs or skills that use it. Must exist. |
| `profile` | yes | `default`, or `default-with-core-let` for syntax enabled transitively by current core defaults. |
| `input` | yes | Exact input bytes (stdin, or the `-f` file). |
| `args` | yes | CLI arguments after `jpx`, one argv element per entry. The expression is the last element. |
| `expected` | yes | `status` (exit code), `stdout` (exact bytes), and optional `stderr_contains`. |
| `shell_args` | no | Literal POSIX command fragment, asserted present in each referenced doc and executed through a real shell. |

## How the harness runs a recipe

- `--color never -c -S` is prepended to `args` so stdout is byte-stable,
  including when dependency features change JSON object insertion order.
  Docs can omit these presentation flags.
- `JPX_*` variables are removed, `JPX_CONFIG` points at an empty file, and
  `HOME` and the working directory are a fresh temp dir. User or project
  config never leaks in.
- Each recipe runs twice: as a direct argv with `input` on stdin, and on Unix
  through `/bin/sh -c` with `input` read via `-f` from a path containing spaces
  and a single quote. `shell_args` exercises the documented quoting verbatim;
  other entries use POSIX single quoting (`'` becomes `'\''`).
- Current CLI builds enable core's `let-expr` through dependency defaults,
  including `cargo test -p jpx --no-default-features`. The let recipe requires
  success; it does not accept a parse failure as an alternative.
- Missing `stderr_contains` means stderr must be empty. Error cases check the
  specified corrective or conflict message.

## Exit codes

- `0`: success. With `-x`, the result was truthy. Empty arrays and objects
  count as truthy.
- `1`: with `-x`, the result was `false` or `null`. Also used for
  evaluation and input errors.
- `2`: argument usage errors from the parser, such as `--stream` with `-s`.
