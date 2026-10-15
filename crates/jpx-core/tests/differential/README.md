# Bounded JMESPath differential tests

`../differential.rs` compares `jpx_core::Runtime::strict()` with the existing
Rust `jmespath` 0.5 dev dependency (also used by the `vs_jmespath` bench). It is test infrastructure, not a second
specification. No Python, CLI subprocess, network service, user settings, or
history is used by the harness.

## Campaigns and controls

Run from the repository root, with locked dependencies already available:

```sh
cargo test --offline --locked -p jpx-core --no-default-features --test differential --test property_tests
cargo test --offline --locked -p jpx-core --all-features --test differential --test property_tests
cargo fmt --all -- --check
```

Ordinary tests run a smoke campaign of **32 cases per population per seed**,
with seeds **249 and 99537481**: 192 comparisons. Harness self-tests also check
normalization, replay, both generated populations, and failure handling.
Two entry points are ignored during ordinary tests: the larger campaign and
single-file replay.

Explicit larger campaign (the same budget used by the manually dispatched CI workflow):

```sh
JPX_DIFF_CASES=1024 JPX_DIFF_SEEDS=249,99537481 \
  cargo test --offline --locked -p jpx-core --no-default-features \
  --test differential differential_large -- --ignored --exact --nocapture
```

Replace `--no-default-features` with `--all-features` to exercise the feature-rich
build. `.github/workflows/differential.yml` initially runs both configurations
only via `workflow_dispatch`, with read-only repository permissions, a 20-minute
job limit, and artifact upload even on failure. Cold CI runners first fetch the
existing locked dependencies; campaign evaluation is offline.

| Control | Default | Bound |
| --- | --- | --- |
| `JPX_DIFF_CASES` | smoke: 32; large: 1024 | 1–10000 per population per seed |
| `JPX_DIFF_SEEDS` | `249,99537481` | 1–8 comma-separated decimal `u64`s |
| `JPX_DIFF_REPLAY` | unset | path to one saved JSON reproducer |

A larger run performs 6144 comparisons. Neither campaign stops at the first
mismatch: both generated populations and the real compliance pairs always run
the requested count. The test fails after writing its summary if any unallowed
divergence occurred. Generation failure is also a test failure. No shrinking or
proptest persistence is used: identity is the population, seed, and zero-based
case index. A separate ChaCha stream per population keeps their identities
independent. Case generation is reproducible with this source, lockfile, and
architecture; direct input replay does not depend on generation versions.

The populations are:

- **grammar**: shared recursive grammar (depth 6, target size 192, branch bound 8),
  paired with generated JSON. The differential entry point explicitly excludes
  JEP-18 even with `let-expr`/all features. Strict runtime restricts registered
  functions, not parser syntax. The original property population still includes
  let expressions when enabled, with the same production weights.
- **near-valid**: shared one-edit mutations of successful vendored compliance
  expressions, paired with generated JSON. Truncation, deletion, replacement,
  duplication, and an incomplete filter suffix remain in the population.
  Mutations can remain valid; errors are not discarded.
- **compliance**: sampled original expression/document pairs, including error
  cases, from every vendored JSON file. Sorted files and group/case indices are
  recorded as provenance. Corpus files are never modified.

Generated documents have recursion depth 4, target size 64, arrays of 0–7 values,
and objects of 0–4 fields. They include null, booleans, short escaped/Unicode
strings, arbitrary full-width signed/unsigned integers, numeric boundaries,
and exact quarter-unit floats. Ordered maps make generation deterministic.
Dyadic floats keep input JSON replay exact without a new serialization feature
or dependency. This is not exhaustive IEEE-754 coverage; the existing property
JSON strategy retains its arbitrary finite floats and original distributions.

## Results and direct replay

Artifacts are under `<target-dir>/tmp/due-scratch/differential/{smoke,large}/{standard,let-expr}/`.
`<target-dir>/tmp` is Cargo's `CARGO_TARGET_TMPDIR` for integration tests, so it
follows `CARGO_TARGET_DIR` when set and is the workspace `target/tmp` otherwise.
The test prints the exact artifact directory at the end of each campaign:

- `results.jsonl`: every full input and both normalized outcomes;
- `summary.json`: seed list, per-population counts, failures and allowed count;
- `<population>-<seed>-<case_index>.json`: every divergent input and its outcomes;
- `current-case.json`: saved before evaluation, useful after an abort.

The feature directory reflects whether the let parser is compiled in; the
campaign grammar remains standard-only in either directory. Re-running a
campaign replaces results and summary; old individual reproducers are retained.
Use the current `results.jsonl`/summary to identify the current run. Concurrent
runs of the same campaign/build should use separate checkouts.

Copy the printed reproducer path into this command (absolute paths work from
any directory; this relative example assumes the repository root):

```sh
JPX_DIFF_REPLAY="$PWD/target/tmp/due-scratch/differential/large/standard/near-valid-249-123.json" \
  cargo test --offline --locked -p jpx-core --no-default-features \
  --test differential differential_replay -- --ignored --exact --nocapture
```

Use an actual emitted path, or one of the retained reproducers below. Replay
reads the full expression and document directly, recomputes both outcomes, prints
them, and **fails again if they still diverge**. It ignores saved verdicts and
outcomes; it does not regenerate preceding cases. `current-case.json` works too.
The self-test `replay_reproduces_a_real_divergence_without_generation` proves
that serialization and direct replay reproduce an actual numeric mismatch.

## Comparison and error policy

Objects compare by keys regardless of insertion order; arrays compare in order.
Numbers permit `1 == 1.0` and signed zero equality, with no epsilon. Integer pairs
compare as integers; mixed pairs convert an integral, range-checked float to
`i128`. No integer is rounded to `f64`, so adjacent large integers cannot silently
agree. This comparator is independent of both engines' equality implementations.

Reference inputs use `Variable::try_from(&serde_json::Value)`, preserving the
original `serde_json::Number`s. Reference results use fallible serde serialization;
expression references (including nested ones) are unexpected, rather than being
coerced into strings by the reference serializer.

Only typed variants normalize to these classes:

| Variants | Class |
| --- | --- |
| `ErrorReason::Parse` during compilation | syntax |
| `ErrorReason::Parse` during evaluation | unexpected failure (never agreement) |
| `InvalidSlice` | invalid-value |
| `NotEnoughArguments`, `TooManyArguments` | invalid-arity |
| `UnknownFunction` | unknown-function |
| `InvalidType`, `InvalidReturnType` | invalid-type |

Messages, offsets, and wording do not decide equivalence. Unknown jpx variants,
including recursion-limit failures, are unexpected; the reference's exhaustive
match requires review on dependency changes. Conversion/serialization failures
and unwinding panics are actionable failures, **even on both sides**. Panics are
caught around each engine, without changing the global panic hook. Process
aborts, OOM, and stack overflow cannot be caught this way; use the current-input
artifact and CI timeout to investigate them. Bounded inputs limit this risk but
do not make the harness a sandbox.

## Triage and allowlist

A green run only establishes agreement for its finite inputs. The engines may
share bugs, error-class normalization hides diagnostic differences, and the
reference is not an infallible specification oracle. In particular, reference
0.5 numeric equality still uses `f64` with tolerance. Array results of unordered object
operations can also differ without violating the specification: array order is
still compared, so such findings need explicit triage rather than silent sorting.

For each failure, retain the full artifact, reduce a copy if helpful, and check
the standard/compliance expectation and both implementations. Classify it as a
**likely jpx bug**, **likely reference bug**, or **spec ambiguity** only when there
is supporting evidence; otherwise mark it unclassified. Do not change runtime
code, remove a population, alter seeds, or narrow generation merely to obtain a
passing campaign. Runtime fixes and policy decisions are separate follow-ups.

There are two reviewed exception lists, and both require a non-blank reason.
Tests enforce the matching rules below.

`ALLOWLIST` covers a divergence where both engines returned a JSON result or a
typed error. It is deliberately empty. A future entry must include a reason and
issue/evidence, an exact expression and document, and both exact normalized
outcomes. Changes to input or outcomes do not match, and panics, unexpected
errors, and conversion failures on either side cannot be exempted through it.

`REFERENCE_FAILURES` covers a reviewed failure of the reference engine itself,
which the harness reports as `unexpected` and which `ALLOWLIST` cannot cover.
An entry is keyed by the exact expression, the exact reference `unexpected`
message (the failure kind, such as a parse error during search), and the exact
jpx outcome, which must be a JSON result or a typed error. The document is not
part of the key. jpx-side panics, jpx-side unexpected errors, and conversion
failures or panics on either side are never covered. The reason must name the
reference bug and its evidence.

There are no broad regex or error-class exemptions. Keep a reproducer and
regression alongside any approved exception; never silently add an exception
after a red run. To deliberately review a finding: classify it with evidence as
described above, add the narrowest matching entry with its reason, keep the
reproducer under `reproducers/`, and add a self-test that replays it and checks
the entry covers exactly that outcome. Covered divergences are still written as
reproducers and counted as `allowlisted` in `summary.json`; they do not fail the
campaign. Remove an entry once the reference or jpx changes so that it no longer
diverges; the self-test fails when that happens.

## Retained findings

The initial 6,144-case campaign (1,024 cases per population per seed, seeds
249 and 99537481) found five divergent cases in two groups. All five full inputs
are committed under `reproducers/`; saved outcomes are deliberately omitted so
replay always computes current behavior. `ALLOWLIST` remains empty;
`REFERENCE_FAILURES` holds one reviewed entry for the `avg(empty_list)` pair.

| Reproducer | Classification and evidence |
| --- | --- |
| `compliance-249-322.json`, `compliance-249-872.json` | Reviewed reference bug, covered by `REFERENCE_FAILURES`: `avg(empty_list)` returns null in jpx, but the reference computes 0/0 and returns a Parse error during search. This is present in `jmespath` 0.3 and still present in 0.5. The harness reports it as unexpected, not syntax. The [specification](https://jmespath.org/specification.html#avg) and vendored `functions.json` case both require null. |
| `grammar-99537481-147.json`, `grammar-99537481-184.json`, `grammar-99537481-557.json` | Likely jpx strict-conformance gap: string ordering produces a boolean in jpx and null in the reference. The [ordering specification](https://jmespath.org/specification.html#ordering-operators) limits these operators to numbers. Existing string comparison code establishes behavior, not an approved exception to strict semantics. |

Replay any row by selecting its filename; for example:

```sh
JPX_DIFF_REPLAY="$PWD/crates/jpx-core/tests/differential/reproducers/compliance-249-322.json" \
  cargo test --offline --locked -p jpx-core --no-default-features \
  --test differential differential_replay -- --ignored --exact --nocapture

JPX_DIFF_REPLAY="$PWD/crates/jpx-core/tests/differential/reproducers/grammar-99537481-147.json" \
  cargo test --offline --locked -p jpx-core --all-features \
  --test differential differential_replay -- --ignored --exact --nocapture
```

The `compliance-*` replay passes because its reviewed `REFERENCE_FAILURES` entry
covers it. The `grammar-*` replays and a manually dispatched larger campaign
currently exit nonzero: each configuration reports 3 divergences and 2
allowlisted cases out of 6,144 comparisons. The normal smoke suite is green.
This is a known initial state, not evidence that either implementation is fully
conformant.
The workflow starts as manual-only; its failing test exit is preserved, and full
logs and reproducers are uploaded even on failure. Track follow-up work in
[issue #249](https://github.com/joshrotenberg/jpx/issues/249). Before enabling a
schedule, resolve or deliberately review the exact discrepancies documented above
in separate follow-up work (the string-ordering cases either by a jpx strict-mode
fix or by reviewed exact `ALLOWLIST` entries, since both outcomes are JSON), confirm that both configurations pass the unchanged
1,024-case-per-population campaign with seeds 249 and 99537481, and validate
failure reporting and artifact handling. Enable the schedule only in a separately
reviewed PR. Runtime/reference corrections are separate from this readiness patch.
No generator, seed, comparator or allowlist was weakened to hide the findings.
The sampled compliance population supplements the existing exhaustive vendored
suite; it does not replace it or promise every corpus pair appears in one run.
