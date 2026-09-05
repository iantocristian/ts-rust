"""Fail-closed coverage for the bootstrap evidence commands (no network/builds)."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "tracking-bootstrap.py"
SPEC = importlib.util.spec_from_file_location("tracking_bootstrap", SCRIPT)
bootstrap = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bootstrap)


def git(root, *args):
    return subprocess.check_output(
        ["git", "-C", str(root), *args], stderr=subprocess.DEVNULL, text=True
    ).strip()


class UpstreamChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="tracking-bootstrap-test-")
        self.addCleanup(self.temp.cleanup)
        base = Path(self.temp.name)
        self.root = base / "workspace"
        self.source = base / "origin"
        for repo in (self.root, self.source):
            repo.mkdir()
            git(repo, "init", "--quiet")
            git(repo, "config", "user.name", "Fixture")
            git(repo, "config", "user.email", "fixture@example.invalid")
        compiler = self.source / "tsc/cmd/tsc"
        compiler.mkdir(parents=True)
        (self.source / "tsc/go.mod").write_text("module fixture\n")
        (compiler / "main.go").write_text("package main\nfunc main() {}\n")
        git(self.source, "add", ".")
        git(self.source, "-c", "commit.gpgsign=false", "commit", "--quiet", "-m", "fixture")
        self.pin = git(self.source, "rev-parse", "HEAD")
        (self.root / "data").mkdir()
        self.manifest = self.root / "data/upstream.json"
        self.manifest.write_text(json.dumps({"schema_version": 2, "pin": self.pin}))
        git(self.root, "-c", "protocol.file.allow=always", "submodule", "add", "--quiet", str(self.source), "upstream")
        self.upstream = self.root / "upstream"
        git(self.root, "config", "--file", ".gitmodules", "submodule.upstream.url", bootstrap.CANONICAL_UPSTREAM + ".git")
        git(self.root, "add", ".gitmodules")

    def test_registered_clean_submodule_at_full_pin_passes(self):
        self.assertEqual(bootstrap.verify_upstream(self.root), self.upstream)

    def test_missing_initialization_does_not_create_it(self):
        git(self.root, "submodule", "deinit", "--force", "upstream")
        with self.assertRaisesRegex(ValueError, "not initialized"):
            bootstrap.verify_upstream(self.root)
        self.assertFalse((self.upstream / ".git").exists())

    def test_unregistered_checkout_cannot_pass(self):
        git(self.root, "update-index", "--force-remove", "upstream")
        with self.assertRaisesRegex(ValueError, "registered gitlink"):
            bootstrap.verify_upstream(self.root)

    def test_wrong_gitlink_pin_is_rejected(self):
        other = "f" * 40 if self.pin != "f" * 40 else "e" * 40
        git(self.root, "update-index", "--cacheinfo", "160000", other, "upstream")
        with self.assertRaisesRegex(ValueError, "registered gitlink"):
            bootstrap.verify_upstream(self.root)

    def test_checkout_commit_must_match_registered_pin(self):
        git(self.upstream, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false", "commit", "--allow-empty", "--quiet", "-m", "different commit")
        with self.assertRaisesRegex(ValueError, "checkout does not match"):
            bootstrap.verify_upstream(self.root)

    def test_short_pin_is_rejected(self):
        self.manifest.write_text(json.dumps({"schema_version": 2, "pin": self.pin[:10]}))
        with self.assertRaisesRegex(ValueError, "full lowercase commit"):
            bootstrap.verify_upstream(self.root)

    def test_wrong_url_is_rejected(self):
        git(self.root, "config", "--file", ".gitmodules", "submodule.upstream.url", "https://example.invalid/TypeScript.git")
        with self.assertRaisesRegex(ValueError, "canonical Microsoft"):
            bootstrap.verify_upstream(self.root)

    def test_dirty_tracked_file_is_rejected(self):
        (self.upstream / "tsc/go.mod").write_text("module changed\n")
        with self.assertRaisesRegex(ValueError, "tracked or untracked changes"):
            bootstrap.verify_upstream(self.root)

    def test_untracked_file_is_rejected(self):
        (self.upstream / "extra.go").write_text("package extra\n")
        with self.assertRaisesRegex(ValueError, "tracked or untracked changes"):
            bootstrap.verify_upstream(self.root)

    def test_oracle_build_and_version_smoke_produce_metrics(self):
        commands = []

        def run(command, **kwargs):
            commands.append((command, kwargs))
            return subprocess.CompletedProcess(command, 0, stdout="Version 7.0.0-dev\n" if command[-1] == "--version" else None)

        # check_output internally calls subprocess.run, so capture the two
        # non-Git commands and let real run handle all provenance queries.
        real_run = subprocess.run

        def selective_run(command, **kwargs):
            if command[0] == "git":
                kwargs["stderr"] = subprocess.DEVNULL
                return real_run(command, **kwargs)
            return run(command, **kwargs)

        with patch.object(bootstrap.subprocess, "run", side_effect=selective_run), contextlib.redirect_stdout(io.StringIO()) as output, contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(bootstrap.oracle(self.root), {"build": True, "smoke": True})
        self.assertEqual(output.getvalue(), "")
        self.assertEqual(commands[0][0][:3], ["go", "build", "-mod=readonly"])
        self.assertEqual(commands[0][1]["env"]["CGO_ENABLED"], "0")
        self.assertEqual(commands[1][0][-1], "--version")
        self.assertFalse(Path(commands[1][0][0]).parent.exists())

    def test_failed_build_cannot_report_success(self):
        with patch.object(bootstrap, "verify_upstream", return_value=self.upstream), patch.object(bootstrap.subprocess, "run", side_effect=subprocess.CalledProcessError(1, ["go", "build"])):
            with self.assertRaises(subprocess.CalledProcessError):
                bootstrap.oracle(self.root)

    def test_arbitrary_successful_executable_is_not_version_smoke(self):
        results = [subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 0, stdout="hello\n")]
        with patch.object(bootstrap, "verify_upstream", return_value=self.upstream), patch.object(bootstrap.subprocess, "run", side_effect=results), contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaisesRegex(ValueError, "TypeScript version"):
                bootstrap.oracle(self.root)


class OutputChecks(unittest.TestCase):
    def test_main_stdout_is_metrics_json_only(self):
        with patch("sys.argv", [str(SCRIPT), "workspace"]), patch.object(bootstrap, "workspace", return_value={"build": True}), contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(bootstrap.main(), 0)
        self.assertEqual(json.loads(output.getvalue()), {"metrics": {"build": True}})

    def test_failure_emits_no_success_metrics(self):
        with patch("sys.argv", [str(SCRIPT), "oracle"]), patch.object(bootstrap, "oracle", side_effect=ValueError("missing submodule")), contextlib.redirect_stdout(io.StringIO()) as output, contextlib.redirect_stderr(io.StringIO()) as errors:
            self.assertEqual(bootstrap.main(), 1)
        self.assertEqual(output.getvalue(), "")
        self.assertIn("missing submodule", errors.getvalue())


if __name__ == "__main__":
    unittest.main()
