#!/usr/bin/env python3
"""Unit tests for run.py (stdlib unittest; no jpx or node needed).

Run: python3 -m unittest discover -s evaluations/json-workflows -p 'test_*.py'
"""

import contextlib
import copy
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("jpx_workflow_runner", HERE / "run.py")
runner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(runner)

CORPUS = json.loads((HERE / "cases.json").read_text())
INTENDED = {"json": {"equal": False, "greater": True}}
KNOWN = {"args": ["@"], "known_failure": "#259", "known_result": {"json": {"equal": True, "greater": False}}}


def judge(out, err="", status=0, required=(), spec=KNOWN, timed_out=False):
    return runner.judge(spec, INTENDED, out, err, status, set(required), timed_out)[0]


def main(*argv):
    """Run the CLI entry point; return (exit status, stdout, stderr)."""
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = runner.main(list(argv))
    return code, out.getvalue(), err.getvalue()


class Classification(unittest.TestCase):
    def test_known_failure_reproducing_recorded_result_is_xfail(self):
        self.assertEqual(judge('{"equal":true,"greater":false}\n'), "xfail")

    def test_known_failure_with_a_different_wrong_result_fails(self):
        self.assertEqual(judge('{"equal":null,"greater":false}\n'), "fail")

    def test_known_failure_that_crashes_or_times_out_fails(self):
        self.assertEqual(judge("", "panic", 101), "fail")
        self.assertEqual(judge("", timed_out=True), "fail")

    def test_known_failure_meeting_the_intent_is_stale(self):
        self.assertEqual(judge('{"equal":false,"greater":true}\n'), "stale")

    def test_required_fix_turns_known_failure_into_ordinary_expectation(self):
        self.assertEqual(judge('{"equal":true,"greater":false}\n', required=["#259"]), "fail")
        self.assertEqual(judge('{"equal":false,"greater":true}\n', required=["#259"]), "pass")

    def test_unrelated_known_tag_does_not_unmark(self):
        self.assertEqual(judge('{"equal":true,"greater":false}\n', required=["other"]), "xfail")

    def test_difference_passes_its_own_expectation_but_not_the_intent(self):
        spec = {"args": ["@"], "difference": "null_omission", "expect": {"json": [1]}}
        outcome, _, meets_intent = runner.judge(spec, {"json": [None, 1]}, "[1]\n", "", 0, set())
        self.assertEqual((outcome, meets_intent), ("pass", False))

    def test_exact_stdout_expectation(self):
        self.assertEqual(runner.check({"stdout": "undefined\n"}, "undefined\n", "", 0), [])
        self.assertTrue(runner.check({"stdout": "undefined\n"}, "undefined", "", 0))

    def test_large_integers_and_booleans_are_compared_exactly(self):
        self.assertFalse(runner.same(9007199254740993, 9007199254740992.0))
        self.assertFalse(runner.same(True, 1))
        self.assertTrue(runner.same(5, 5.0))


class Validation(unittest.TestCase):
    def errors_after(self, mutate):
        corpus = copy.deepcopy(CORPUS)
        mutate({c["id"]: c for c in corpus["cases"]})
        return runner.validate(corpus)

    def test_shipped_corpus_is_valid_and_large_enough(self):
        self.assertEqual(runner.validate(CORPUS), [])
        self.assertGreaterEqual(len(CORPUS["cases"]), 20)

    def test_every_known_failure_records_its_wrong_result(self):
        errors = self.errors_after(lambda c: c["large-id-compare"]["jpx"].pop("known_result"))
        self.assertTrue(any("known_result" in e for e in errors), errors)

    def test_rejects_ambiguous_expectation(self):
        errors = self.errors_after(lambda c: c["ci-select"]["expect"].update(lines=[1]))
        self.assertTrue(any("at most one" in e for e in errors), errors)

    def test_rejects_translation_in_wrong_field(self):
        errors = self.errors_after(lambda c: c["ci-select"].update(jpx={"code": "status"}))
        self.assertTrue(any("ci-select.jpx" in e for e in errors), errors)

    def test_rejects_unknown_profile_and_explicit_strict(self):
        def mutate(c):
            c["ci-select"]["profile"] = "fancy"
            c["ci-failed-jobs"]["jpx"]["args"].insert(0, "--strict")
        errors = self.errors_after(mutate)
        self.assertTrue(any("profile" in e for e in errors), errors)
        self.assertTrue(any("--strict" in e for e in errors), errors)

    def test_standard_profile_runs_jpx_strict(self):
        case = next(c for c in CORPUS["cases"] if c["profile"] == "standard" and "args" in c["jpx"])
        self.assertIn("--strict", runner.command("jpx", case["jpx"], case, {"jpx": "jpx"}))


class SetupErrors(unittest.TestCase):
    def assert_setup_error(self, argv, hint):
        code, _, err = main(*argv)
        self.assertEqual(code, 2, err)
        self.assertIn(hint, err)
        self.assertNotIn("Traceback", err)

    def test_malformed_corpus(self):
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            f.write("{")
        self.addCleanup(os.unlink, f.name)
        self.assert_setup_error(["--cases", f.name, "--jpx", sys.executable], "not valid JSON")

    def test_case_filter_without_hits(self):
        self.assert_setup_error(["--case", "no-such-case", "--jpx", sys.executable], "Valid ids")

    def test_unknown_required_tag(self):
        self.assert_setup_error(["--require-fixed", "#999", "--jpx", sys.executable], "Known tags")

    def test_jpx_directory_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            self.assert_setup_error(["--jpx", d], "cargo build -p jpx")

    @unittest.skipIf(os.name == "nt", "POSIX execute bit")
    def test_non_executable_jpx_is_rejected(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            pass
        self.addCleanup(os.unlink, f.name)
        self.assert_setup_error(["--jpx", f.name], "not an executable file")

    def test_explicit_invalid_node_fails_even_when_missing_node_is_allowed(self):
        self.assert_setup_error(["--jpx", sys.executable, "--node", "/nonexistent/node",
                                 "--allow-missing-node"], "--node")

    def test_missing_node_fails_by_default(self):
        with mock.patch.object(runner.shutil, "which", return_value=None):
            self.assert_setup_error(["--jpx", sys.executable], "--allow-missing-node")

    def test_allowed_missing_node_is_labelled_unrun_without_cost_totals(self):
        with tempfile.TemporaryDirectory() as d, \
                mock.patch.object(runner.shutil, "which", return_value=None):
            report_path = Path(d, "report.json")
            code, _, err = main("--jpx", sys.executable, "--allow-missing-node",
                                "--case", "control-update-document", "--json", str(report_path))
            report = json.loads(report_path.read_text())
        self.assertEqual(code, 0, err)
        js = report["totals"]["js"]
        self.assertEqual((js["unrun"], js["pass"]), (1, 0))
        self.assertEqual(report["totals"]["python"]["pass"], 1)
        self.assertEqual(js["common_cases"], 0)
        self.assertIsNone(js["harness_argv_bytes"])


if __name__ == "__main__":
    unittest.main()
