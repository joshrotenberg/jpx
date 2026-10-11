# JSON workflow evaluation corpus

A synthetic, deterministic corpus of routine JSON tasks (part of #264). Each
case runs through the jpx CLI and through a concise Python and a direct
JavaScript baseline. All three read the **same fixture file**, so no approach
pays for fetching, parsing or transport that the others skip.

This is a **deterministic fixture check, not an agent replay or plugin trial**.
It measures correctness, subprocess invocations and bytes. Model tokens,
retries and agent elapsed time are not measured and are always reported as
`null`. Nothing here supports a claim of model token or time savings.

## Run

```bash
cargo build -p jpx
python3 evaluations/json-workflows/run.py --jpx target/debug/jpx
python3 evaluations/json-workflows/run.py --jpx target/debug/jpx --json /tmp/jpx-workflows.json
python3 evaluations/json-workflows/run.py --jpx target/debug/jpx --case large-id
```

You need Python 3.8+ and `node` 16.6+. Node is found on `PATH`, or you can pass
it with `--node`. A missing node is a setup error. Pass `--allow-missing-node`
to label the JS runs `unrun` instead; when no case ran in all three
implementations, the cost totals are `null`. An explicit `--node` or `--jpx`
that doesn't point to an executable file is always a setup error. The runner
uses only the standard library, fetches nothing and has no dependencies.

The runner isolates every subprocess:
- stdin is `/dev/null`.
- `HOME` and `XDG_CONFIG_HOME` point to a temporary directory.
- `JPX_CONFIG` points to an empty file there.
- `JPX_*`, `NODE_OPTIONS` and `NODE_PATH` are removed from the environment.
- If a `jpx.toml` exists in this directory or any parent, the runner refuses to
  start. jpx merges that file from the working directory regardless of
  `JPX_CONFIG`.

Exit status:
- `0`: every run met its expectation, and every known failure reproduced its
  recorded wrong result.
- `1`: an unexpected failure **or a stale known failure**.
- `2`: a setup or corpus error. The runner prints the corrective action, with
  no traceback. This covers a malformed or invalid corpus, a `--case` that
  matches nothing, a `--require-fixed` tag that isn't in the corpus, and a
  missing or non-executable binary.

Unit tests for the runner (no jpx or node needed):

```bash
python3 -m unittest discover -s evaluations/json-workflows -p 'test_*.py'
```

## Outcomes

| Outcome | Meaning |
|---|---|
| `pass` | The run matched its expectation |
| `fail` | Unexpected mismatch (the runner exits 1) |
| `xfail` | A known failure that reproduced its exact recorded `known_result`; it's listed with its problems, never shown as a pass |
| `stale` | A known failure that now meets its expectation (the runner exits 1 until the marker is removed) |

A known failure that produces any other result counts as `fail`: a different
wrong value, a crash or a timeout.
| `unrun` | Not executed, with a reason (not applicable, deferred or tool missing) |

`meets_intent` is checked separately against the case's intended outcome. A
run can `pass` a labelled jpx difference (documented behavior) and still have
`meets_intent: false`.

Known failures:

- `#259`: jpx compares numbers as `f64`, so ids above 2^53 compare equal
  (`large-id-compare`, `large-id-filter`). Once the fix lands, run with
  `--require-fixed '#259'` to require the corrected results.
  In default mode they are reported as `stale` until you remove the marker.
- `js-number-precision`: plain `JSON.parse` rounds integers above 2^53.
- `js-undefined-not-error`: `x.length` on a number gives `undefined`, not an error.

## Labelled jpx differences

These are deliberate behaviors. Each one has its own expectation and is labelled
in the output: `null_omission`, `forgiving_json_decode`, `tolerant_streaming`,
`chained_cli_expressions`, `false_null_exit_status`. Their definitions are in
`cases.json` under `differences`. Where a working translation exists, it is a
separate case (`null-preserving-map`, `exit-status-boolean`,
`multiple-queries-combined`).

## Case format (`cases.json`)

Each case has the following fields:

- `id`, `category` and `intent`
- `data`: the fixture path, relative to this directory. It's passed to jpx as
  `-f` and to the baselines as `argv`. `extra_data` is optional.
- `profile`: `standard` (JMESPath spec only), `extensions` (jpx extension
  functions) or `strict` (tests strict mode itself). The runner adds
  `--strict` to jpx runs for `standard` and `strict`, which proves that those
  cases need no extensions. Don't put `--strict` in `args` yourself.
- `preferred`: `jpx`, `script` or `either`. The two `control-*` cases are tasks
  where ordinary scripting should stay the preferred choice.
- `expect`: the intended outcome. It uses at most one of `json` (a single
  stdout value), `lines` (one JSON value per line) or `stdout` (the exact
  text). With none of them, stdout must be empty. It can also set `status` (an
  int or `"nonzero"`, default `0`) and `stderr_contains`.
- `jpx`: `{"args": [...]}`. `python` and `js`: `{"code": "..."}`. Each can add
  an optional `expect` override, `difference` and `note`. A `known_failure` tag
  must come with a `known_result`, which records the exact wrong output in the
  same expectation format. A path can also be `{"unrun": "reason"}`.

Number comparison is by exact value, so `5.0 == 5` but
`9007199254740992 != 9007199254740993`. Booleans never equal numbers, and the
order of object keys is ignored.

Baselines get a small shared prelude: `P` is the data path, `J()` loads JSON,
`L()` loads NDJSON and `out(v)` prints JSON. The prelude counts toward
`harness_argv_bytes`.

## Metrics per run

- `invocations`
- `harness_argv_bytes`: the UTF-8 byte sum of the harness arguments, excluding
  the executable path and separators. It is **not** the command text an agent
  would author.
- `fixture_bytes`: the size of the fixture files on disk. It is **not** the
  number of bytes the process actually read.
- `stdout_bytes`, `stderr_bytes` and `status`
- `process_ms`: subprocess wall time, not agent time

Totals compare implementations only over the cases that all three ran. None of
these numbers is a token or agent-cost measurement.

## Remaining under #264 (not in this slice)

- MCP translations (`evaluate`, `batch_evaluate`, query store), including the
  costs of first use versus warm repeated queries, discovery and document
  import. These show as `mcp: unrun`.
- jq comparisons, and jpx translations for the deferred controls (multi-file
  join, document update and write-back).
- The installed-plugin trial in a real host (CLI routing, MCP chaining,
  discovery, error recovery, restart, non-jpx fallback).
- A bounded local opt-in trial (about 50 operations), measured token, retry and
  latency data, and the go/adjust/remove decision.

The `JSON workflows` CI job builds the CLI, runs the runner's unit tests and
the corpus, and uploads its JSON report. When #259 lands, remove its known
failure markers and recorded wrong results together; the stale-result check
deliberately makes that update visible. To evaluate the fix before merging it,
run with `--require-fixed '#259'`.
