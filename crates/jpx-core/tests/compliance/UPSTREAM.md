# Compliance suite provenance

These files are vendored verbatim from the JMESPath specification test suite.

| | |
|---|---|
| Source | https://github.com/jmespath/jmespath.test |
| Path | `tests/` |
| Pinned commit | `53abcc37901891cf4308fcd910eab287416c4609` |
| Files | 16 |
| Cases | 908 |

Do not hand-edit these files. A previous local edit silently dropped both `?`
characters from a `filters.json` benchmark expression, and the divergence was
only found by diffing against upstream (see #237).

`build.rs` generates one test per case, skipping cases that carry a `bench` key.

## Refreshing

Bump `SHA`, run, and commit the diff along with an updated commit hash and case
count in the table above.

```bash
SHA=53abcc37901891cf4308fcd910eab287416c4609
for f in basic benchmarks boolean current escape filters functions identifiers \
         indices literal multiselect pipe slice syntax unicode wildcard; do
  curl -fsS -o "crates/jpx-core/tests/compliance/$f.json" \
    "https://raw.githubusercontent.com/jmespath/jmespath.test/$SHA/tests/$f.json"
done
```

Because the files are pinned and unmodified, each refresh is a reviewable diff
rather than an archaeology exercise.
