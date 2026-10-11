#!/usr/bin/env python3
"""Deterministic JSON workflow corpus runner (stdlib only).

Runs each case in cases.json through the jpx CLI and through concise Python and
direct JavaScript (node) baselines that read the same fixture file. This is a
deterministic fixture check, not an agent replay: model tokens, retries and
agent elapsed time are not measured and are always reported as null.

Exit status: 0 when every run met its expectation (known failures reproduced
their recorded wrong result), 1 on an unexpected failure or a stale known
failure, 2 on setup or corpus errors.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
IMPLS = ("jpx", "python", "js")
PY_PRELUDE = (
    "import json,sys\nP=sys.argv[1]\nJ=lambda:json.load(open(P))\n"
    "L=lambda:[json.loads(l) for l in open(P) if l.strip()]\n"
    "out=lambda v:print(json.dumps(v))\n"
)
JS_PRELUDE = (
    "const fs=require('fs'),P=process.argv.at(-1),"
    "J=()=>JSON.parse(fs.readFileSync(P,'utf8')),"
    "L=()=>fs.readFileSync(P,'utf8').split('\\n').filter(s=>s.trim()).map(s=>JSON.parse(s)),"
    "out=v=>console.log(JSON.stringify(v));\n"
)
UNMEASURED = {"model_tokens": None, "retries": None, "agent_elapsed_ms": None}
CASE_KEYS = {"id", "category", "intent", "data", "profile", "preferred", "expect", *IMPLS}
SPEC_KEYS = {"args", "code", "expect", "difference", "known_failure", "known_result", "note", "unrun"}
EXPECT_KEYS = {"json", "lines", "stdout", "status", "stderr_contains"}
STDOUT_KINDS = ("json", "lines", "stdout")
PROFILES = {"standard", "extensions", "strict"}
# Profiles whose jpx runs get --strict, proving they need no extension functions.
STRICT_PROFILES = {"standard", "strict"}
COST_KEYS = ("invocations", "harness_argv_bytes", "stdout_bytes", "stderr_bytes")
TIMEOUT = 30


class SetupError(Exception):
    """A setup or corpus problem; reported with its corrective action and exit status 2."""


def same(a, b):
    """JSON equality that keeps bool distinct from number and compares numbers by exact value."""
    if isinstance(a, bool) or isinstance(b, bool):
        return type(a) is type(b) and a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a == b
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(map(same, a, b))
    return type(a) is type(b) and a == b


def check(expect, out, err, status):
    """Return a list of mismatches between an expectation and a finished process."""
    problems = []
    want = expect.get("status", 0)
    if (status == 0) if want == "nonzero" else (status != want):
        problems.append(f"exit status {status}, expected {want}")
    shown = out.strip()[:160]
    try:
        if "json" in expect:
            if not same(json.loads(out), expect["json"]):
                problems.append(f"stdout {shown!r} != {json.dumps(expect['json'])}")
        elif "lines" in expect:
            got = [json.loads(line) for line in out.splitlines() if line.strip()]
            if not same(got, expect["lines"]):
                problems.append(f"stdout lines {shown!r} != {json.dumps(expect['lines'])}")
        elif "stdout" in expect:
            if out != expect["stdout"]:
                problems.append(f"stdout {out[:160]!r} != {expect['stdout']!r}")
        elif out.strip():
            problems.append(f"unexpected stdout {shown!r}")
    except ValueError:
        problems.append(f"stdout is not the expected JSON shape: {shown!r}")
    for needle in expect.get("stderr_contains", []):
        if needle not in err:
            problems.append(f"stderr lacks {needle!r}")
    return problems


def judge(spec, case_expect, out, err, status, required, timed_out=False):
    """Classify one finished run; return (outcome, problems, meets_intent).

    A known failure is an xfail only when it reproduces its recorded known_result.
    Any other mismatch (crash, timeout, different wrong value) is a fail, and a
    known failure that now meets its expectation is stale.
    """
    if timed_out:
        return "fail", [f"timed out after {TIMEOUT}s"], False
    problems = check(spec.get("expect", case_expect), out, err, status)
    meets_intent = not check(case_expect, out, err, status)
    known = spec.get("known_failure")
    if not known or known in required:
        return ("fail" if problems else "pass"), problems, meets_intent
    if not problems:
        return "stale", [f"known failure {known} now passes; remove the marker or "
                         f"run with --require-fixed '{known}'"], meets_intent
    known_problems = check(spec["known_result"], out, err, status)
    if known_problems:
        return "fail", problems + [f"does not reproduce known_result for {known}: "
                                   + "; ".join(known_problems)], meets_intent
    return "xfail", problems, meets_intent


def validate_expect(expect, where, errors):
    if not isinstance(expect, dict):
        errors.append(f"{where}: expectation must be an object")
        return
    if extra := expect.keys() - EXPECT_KEYS:
        errors.append(f"{where}: unknown expectation keys {sorted(extra)}; use {sorted(EXPECT_KEYS)}")
    if sum(k in expect for k in STDOUT_KINDS) > 1:
        errors.append(f"{where}: use at most one of json, lines, stdout")
    status = expect.get("status", 0)
    if status != "nonzero" and (type(status) is not int):
        errors.append(f"{where}: status must be an integer or \"nonzero\"")
    if "lines" in expect and not isinstance(expect["lines"], list):
        errors.append(f"{where}: lines must be a list of JSON values")
    if "stdout" in expect and not isinstance(expect["stdout"], str):
        errors.append(f"{where}: stdout must be the exact output string")
    needles = expect.get("stderr_contains", [])
    if not (isinstance(needles, list) and all(isinstance(s, str) for s in needles)):
        errors.append(f"{where}: stderr_contains must be a list of strings")


def validate_spec(case, impl, differences, errors):
    where, spec = f"{case['id']}.{impl}", case[impl]
    if not isinstance(spec, dict):
        errors.append(f"{where}: must be an object")
        return
    if extra := spec.keys() - SPEC_KEYS:
        errors.append(f"{where}: unknown keys {sorted(extra)}; use {sorted(SPEC_KEYS)}")
    if "unrun" in spec:
        if not isinstance(spec["unrun"], str) or len(spec) > 1:
            errors.append(f"{where}: 'unrun' takes only a reason string")
        return
    want, other = ("args", "code") if impl == "jpx" else ("code", "args")
    if other in spec:
        errors.append(f"{where}: use '{want}', not '{other}'")
    if impl == "jpx":
        args = spec.get("args")
        if not (isinstance(args, list) and args and all(isinstance(a, str) for a in args)):
            errors.append(f"{where}: 'args' must be a non-empty list of strings")
        elif "--strict" in args:
            errors.append(f"{where}: remove '--strict'; the runner adds it for the "
                          f"{sorted(STRICT_PROFILES)} profiles")
    elif not (isinstance(spec.get("code"), str) and spec["code"].strip()):
        errors.append(f"{where}: 'code' must be a non-empty string")
    if "expect" in spec:
        validate_expect(spec["expect"], f"{where}.expect", errors)
    diff = spec.get("difference")
    if diff is not None and diff not in differences:
        errors.append(f"{where}: unknown difference {diff!r}; use one of {sorted(differences)}")
    if diff is not None and "expect" not in spec:
        errors.append(f"{where}: a labelled difference needs its own expect")
    known = spec.get("known_failure")
    if known is not None:
        if not (isinstance(known, str) and known):
            errors.append(f"{where}: known_failure must be a non-empty tag such as '#259'")
        if "known_result" not in spec:
            errors.append(f"{where}: known_failure needs a known_result recording the exact wrong output")
        else:
            validate_expect(spec["known_result"], f"{where}.known_result", errors)
            if spec["known_result"] == spec.get("expect", case["expect"]):
                errors.append(f"{where}: known_result must differ from the expectation")
    elif "known_result" in spec:
        errors.append(f"{where}: known_result needs a known_failure tag")


def validate(corpus):
    if not (isinstance(corpus, dict) and isinstance(corpus.get("cases"), list)
            and isinstance(corpus.get("differences"), dict)):
        return ["the corpus must be an object with a 'differences' object and a 'cases' list"]
    errors, seen = [], set()
    cases = corpus["cases"]
    if len(cases) < 20:
        errors.append(f"corpus has {len(cases)} cases; at least 20 are required")
    for n, case in enumerate(cases):
        if not isinstance(case, dict):
            errors.append(f"case #{n}: must be an object")
            continue
        cid = case.get("id", f"#{n}")
        if missing := CASE_KEYS - case.keys():
            errors.append(f"{cid}: missing {sorted(missing)}")
            continue
        if cid in seen:
            errors.append(f"{cid}: duplicate id")
        seen.add(cid)
        if case["profile"] not in PROFILES:
            errors.append(f"{cid}: profile {case['profile']!r} must be one of {sorted(PROFILES)}")
        extra = case.get("extra_data", [])
        if not isinstance(extra, list):
            errors.append(f"{cid}: extra_data must be a list of fixture paths")
            extra = []
        for path in [case["data"], *extra]:
            if not (isinstance(path, str) and (HERE / path).is_file()):
                errors.append(f"{cid}: missing fixture {path!r} (paths are relative to {HERE.name}/)")
        validate_expect(case["expect"], cid, errors)
        for impl in IMPLS:
            validate_spec(case, impl, corpus["differences"], errors)
    return errors


def load_corpus(path):
    try:
        corpus = json.loads(Path(path).read_text())
    except OSError as e:
        raise SetupError(f"cannot read corpus {path}: {e.strerror}. Pass --cases with an existing file.")
    except ValueError as e:
        raise SetupError(f"corpus {path} is not valid JSON: {e}. Fix the syntax at that position.")
    if errors := validate(corpus):
        raise SetupError("invalid corpus; fix these entries:\n  " + "\n  ".join(errors))
    return corpus


def select_cases(cases, filters):
    for f in filters:
        if not any(f in c["id"] for c in cases):
            raise SetupError(f"--case {f!r} matches no case id. Valid ids: "
                             + ", ".join(c["id"] for c in cases))
    return [c for c in cases if not filters or any(f in c["id"] for f in filters)]


def check_required(cases, required):
    tags = {c[i].get("known_failure") for c in cases for i in IMPLS} - {None}
    if unknown := set(required) - tags:
        raise SetupError(f"--require-fixed {', '.join(sorted(unknown))} matches no known_failure tag. "
                         f"Known tags: {', '.join(sorted(tags))}")


def resolve_exe(value, name, hint):
    """Resolve a path or PATH name to an executable file, or raise a corrective SetupError."""
    is_path = os.sep in value or (os.altsep and os.altsep in value)
    found = value if is_path else shutil.which(value)
    if not found or not Path(found).exists():
        raise SetupError(f"{name} not found: {value!r}. {hint}")
    if not Path(found).is_file() or not os.access(found, os.X_OK):
        raise SetupError(f"{name} at {value!r} is not an executable file. {hint}")
    return str(Path(found).resolve())


def find_project_config():
    for d in (HERE, *HERE.parents):
        if (d / "jpx.toml").exists():
            return d / "jpx.toml"
    return None


def command(impl, spec, case, tools):
    if impl == "jpx":
        strict = ["--strict"] if case["profile"] in STRICT_PROFILES else []
        return [tools["jpx"], "--color", "never", "-c", *strict, *spec["args"], "-f", case["data"]]
    if impl == "python":
        return [sys.executable, "-I", "-c", PY_PRELUDE + spec["code"], case["data"]]
    return [tools["js"], "-e", JS_PRELUDE + spec["code"], case["data"]]


def run_one(impl, case, tools, env, required):
    spec = case[impl]
    labels = {k: spec[k] for k in ("difference", "known_failure", "note") if k in spec}
    if "unrun" in spec or not tools.get(impl):
        reason = spec.get("unrun") or f"{impl} executable not available (--allow-missing-node)"
        return {"outcome": "unrun", "reason": reason, **labels, **UNMEASURED}
    cmd = command(impl, spec, case, tools)
    started = time.perf_counter()
    try:
        proc = subprocess.run(cmd, cwd=HERE, env=env, stdin=subprocess.DEVNULL,
                              capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        proc = None
    except OSError as e:
        raise SetupError(f"could not execute {cmd[0]} for {case['id']}: {e.strerror}. "
                         "Check the binary passed with --jpx/--node.")
    elapsed = round((time.perf_counter() - started) * 1000, 1)
    if proc is None:
        outcome, problems, meets_intent = judge(spec, case["expect"], "", "", None, required, True)
    else:
        outcome, problems, meets_intent = judge(
            spec, case["expect"], proc.stdout.decode("utf-8", "replace"),
            proc.stderr.decode("utf-8", "replace"), proc.returncode, required)
    data = [case["data"], *case.get("extra_data", [])]
    return {
        "outcome": outcome,
        "meets_intent": meets_intent,
        **labels,
        "problems": problems,
        "status": proc and proc.returncode,
        "invocations": 1,
        "harness_argv_bytes": sum(len(a.encode()) for a in cmd[1:]),
        "fixture_bytes": sum((HERE / p).stat().st_size for p in data),
        "stdout_bytes": proc and len(proc.stdout),
        "stderr_bytes": proc and len(proc.stderr),
        "process_ms": elapsed,
        **UNMEASURED,
    }


def version(cmd):
    try:
        return subprocess.run(cmd, capture_output=True, text=True, stdin=subprocess.DEVNULL,
                              timeout=10).stdout.strip() or None
    except (OSError, subprocess.TimeoutExpired):
        return None


def totals(results):
    """Per-implementation outcome counts, plus costs over cases every implementation ran."""
    common = [r for r in results if all(r["results"][i]["outcome"] != "unrun" for i in IMPLS)]
    summary = {}
    for impl in IMPLS:
        runs = [r["results"][impl] for r in results]
        t = {k: sum(x["outcome"] == k for x in runs) for k in ("pass", "fail", "xfail", "stale", "unrun")}
        t["meets_intent"] = sum(bool(x.get("meets_intent")) for x in runs)
        t["common_cases"] = len(common)
        shared = [r["results"][impl] for r in common]
        if shared:
            t["common_meets_intent"] = sum(bool(x["meets_intent"]) for x in shared)
            t.update({k: sum(x[k] or 0 for x in shared) for k in COST_KEYS})
        else:
            t.update(dict.fromkeys(("common_meets_intent", *COST_KEYS)))
        summary[impl] = t
    return summary


def resolve_tools(args):
    tools = {
        "jpx": resolve_exe(args.jpx or os.environ.get("JPX_BIN") or "jpx", "jpx",
                           "Build it with 'cargo build -p jpx' and pass --jpx target/debug/jpx."),
        "python": sys.executable,
    }
    if args.node:
        tools["js"] = resolve_exe(args.node, "node", "Pass --node with the path to an executable node binary.")
    elif shutil.which("node"):
        tools["js"] = resolve_exe("node", "node", "Pass --node with the path to an executable node binary.")
    elif args.allow_missing_node:
        tools["js"] = None
    else:
        raise SetupError("node not found on PATH; the JS baseline is required. Install node 16.6+, "
                         "pass --node PATH, or pass --allow-missing-node to label JS runs unrun.")
    return tools


def run(args):
    corpus = load_corpus(args.cases)
    cases = select_cases(corpus["cases"], args.case)
    check_required(corpus["cases"], args.require_fixed)
    if config := find_project_config():
        raise SetupError(f"{config} would be merged into every jpx run (project config is discovered "
                         "from the working directory). Move it aside and rerun.")
    tools = resolve_tools(args)

    with tempfile.TemporaryDirectory() as home:
        # Isolate from user/home config, JPX_* defaults and node preload settings.
        empty_config = Path(home, "jpx-config.toml")
        empty_config.write_text("")
        env = {k: v for k, v in os.environ.items()
               if not k.startswith("JPX_") and k not in ("NODE_OPTIONS", "NODE_PATH")}
        env.update(HOME=home, XDG_CONFIG_HOME=home, NO_COLOR="1", JPX_CONFIG=str(empty_config))
        required = set(args.require_fixed)
        results = [{
            "id": c["id"], "category": c["category"], "profile": c["profile"],
            "preferred": c["preferred"], "data": c["data"],
            "results": {i: run_one(i, c, tools, env, required) for i in IMPLS},
        } for c in cases]

    report = {
        "kind": "deterministic-fixture-check",
        "agent_replay": None,
        "tools": {"jpx": version([tools["jpx"], "--version"]),
                  "python": sys.version.split()[0],
                  "js": version([tools["js"], "--version"]) if tools["js"] else None},
        "mcp": "unrun: MCP translations are deferred",
        "required_fixed": args.require_fixed,
        "unmeasured": UNMEASURED,
        "metric_notes": {
            "harness_argv_bytes": "UTF-8 byte sum of the harness arguments, excluding the executable "
                                  "and separators; not the command text an agent would author",
            "fixture_bytes": "size of the fixture files on disk, not bytes the process actually read",
        },
        "differences": corpus["differences"],
        "totals": totals(results),
        "cases": results,
    }
    if args.json:
        try:
            Path(args.json).write_text(json.dumps(report, indent=2) + "\n")
        except OSError as e:
            raise SetupError(f"cannot write report {args.json}: {e.strerror}. Pass --json with a writable path.")

    bad = 0
    for r in results:
        cells = []
        for impl in IMPLS:
            res = r["results"][impl]
            tag = res.get("known_failure") or res.get("difference") or ""
            cells.append(f"{impl}={res['outcome']}" + (f"[{tag}]" if tag else ""))
            bad += res["outcome"] in ("fail", "stale")
        print(f"{r['id']:<34} " + "  ".join(cells))
        for impl in IMPLS:
            res = r["results"][impl]
            if res["outcome"] in ("fail", "stale", "xfail"):
                for problem in res["problems"]:
                    print(f"    {impl} {res['outcome']}: {problem}")
    print("\nkind: deterministic fixture check (model tokens, retries, agent elapsed: not measured)")
    print("mcp: unrun (deferred)")
    na = lambda v: "n/a" if v is None else v
    for impl, t in report["totals"].items():
        print(f"{impl:<7} pass={t['pass']} fail={t['fail']} xfail={t['xfail']} stale={t['stale']} "
              f"unrun={t['unrun']} meets_intent={t['meets_intent']}  "
              f"common({t['common_cases']}): meets_intent={na(t['common_meets_intent'])} "
              f"harness_argv={na(t['harness_argv_bytes'])}B stdout={na(t['stdout_bytes'])}B "
              f"stderr={na(t['stderr_bytes'])}B")
    return 1 if bad else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--jpx", help="jpx binary (default: $JPX_BIN, then jpx on PATH)")
    parser.add_argument("--node", help="node binary (default: node on PATH; required unless --allow-missing-node)")
    parser.add_argument("--allow-missing-node", action="store_true",
                        help="if node is not on PATH, label JS runs unrun instead of failing")
    parser.add_argument("--require-fixed", action="append", default=[], metavar="TAG",
                        help="treat known failures with TAG (e.g. '#259') as ordinary expectations")
    parser.add_argument("--case", action="append", default=[], help="run only cases whose id contains this")
    parser.add_argument("--cases", default=str(HERE / "cases.json"), help=argparse.SUPPRESS)
    parser.add_argument("--json", metavar="PATH", help="write the full report as JSON")
    args = parser.parse_args(argv)
    try:
        return run(args)
    except SetupError as e:
        print(f"setup error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
