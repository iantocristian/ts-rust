"""Counterexamples for S03 provenance, client parity and codec coverage claims."""

import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
SPEC = importlib.util.spec_from_file_location("s03", Path(__file__).resolve().parents[1] / "s03.py")
s03 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(s03)
PIN = "1" * 40


class CodecInventoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.inventory = {
            "schemaVersion": 1,
            "mappings": [{"goType": "api.Custom", "fixtureIds": ["custom"]}],
            "ordinaryFieldFixtures": ["ordinary"],
        }
        self.api = {"schemaVersion": 1, "specialMappings": [{"id": "api.Custom"}]}
        self.wire = {
            "schemaVersion": 1,
            "fixtures": [{"id": "custom"}, {"id": "ordinary"}],
        }
        (self.root / "data/s03").mkdir(parents=True)

    def validate(self):
        (self.root / "data/s03/api-special-codecs.json").write_text(json.dumps(self.inventory))
        s03.validate_codecs(self.root, self.api, self.wire)

    def test_complete_inventory_has_a_fixture_for_every_claim(self):
        self.validate()

    def test_missing_or_extra_codec_is_rejected(self):
        for mappings in [[], [{"id": "api.Other"}], [{"id": "api.Custom"}, {"id": "api.New"}]]:
            with self.subTest(mappings=mappings):
                self.api["specialMappings"] = mappings
                with self.assertRaises(ValueError):
                    self.validate()

    def test_duplicate_declarations_and_duplicate_observations_are_rejected(self):
        self.inventory["mappings"].append(copy.deepcopy(self.inventory["mappings"][0]))
        with self.assertRaises(ValueError):
            self.validate()
        self.inventory["mappings"].pop()
        self.api["specialMappings"].append(copy.deepcopy(self.api["specialMappings"][0]))
        with self.assertRaises(ValueError):
            self.validate()

    def test_missing_or_duplicate_wire_fixture_is_rejected(self):
        for fixtures in [[], [{"id": "ordinary"}], [{"id": "custom"}],
                         [{"id": "ordinary"}, {"id": "custom"}, {"id": "custom"}]]:
            with self.subTest(fixtures=fixtures):
                self.wire["fixtures"] = fixtures
                with self.assertRaises(ValueError):
                    self.validate()

    def test_codec_cannot_drop_its_fixture_obligation(self):
        self.inventory["mappings"][0]["fixtureIds"] = []
        with self.assertRaises(ValueError):
            self.validate()

    def test_ordinary_field_coverage_cannot_be_empty(self):
        self.inventory["ordinaryFieldFixtures"] = []
        with self.assertRaises(ValueError):
            self.validate()

    def test_ordinary_field_fixture_ids_cannot_be_duplicated(self):
        self.inventory["ordinaryFieldFixtures"] = ["ordinary", "ordinary"]
        with self.assertRaises(ValueError):
            self.validate()


class ClientParityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tooling = self.root / "tooling"
        self.expected = [f"{s03.CLIENT}/api/proto.generated.ts", f"{s03.CLIENT}/enums/example.ts"]
        self.originals = {self.expected[0]: b"proto\r\n", self.expected[1]: b"enum\r\n"}
        for path, data in self.originals.items():
            output = self.tooling / path
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(data)
        self.control = self.tooling / s03.CLIENT / "api/control.ts"
        self.control.write_bytes(self.originals[self.expected[0]])

    def compare(self):
        def original(args, cwd, env=None):
            self.assertEqual(args[:2], ["git", "show"])
            self.assertEqual(cwd, self.root / "upstream")
            self.assertTrue(args[2].startswith(PIN + ":"))
            return self.originals[args[2].split(":", 1)[1]]
        with patch.object(s03, "run", side_effect=original):
            return s03.compare_client(self.root, self.tooling, PIN, self.expected, self.control)

    def test_equal_bytes_and_complete_inventory_pass(self):
        identical, hashes = self.compare()
        self.assertTrue(identical)
        self.assertEqual([item["path"] for item in hashes], self.expected)
        self.assertTrue(all(len(item["sha256"]) == 64 for item in hashes))

    def test_enum_omission_is_not_hidden_by_equal_remaining_bytes(self):
        (self.tooling / self.expected[1]).unlink()
        self.assertFalse(self.compare()[0])

    def test_restored_baseline_is_removed_before_a_generator_can_omit_an_output(self):
        self.assertTrue(self.compare()[0])  # A reset worktree alone is not evidence.
        s03.clear_enum_outputs(self.tooling, self.expected)
        self.assertFalse((self.tooling / self.expected[1]).exists())
        self.assertTrue((self.tooling / self.expected[0]).exists())
        self.assertFalse(self.compare()[0])  # Simulate a generator producing no enum.
        (self.tooling / self.expected[1]).write_bytes(self.originals[self.expected[1]])
        self.assertTrue(self.compare()[0])

    def test_enum_cleanup_cannot_delete_files_outside_its_output_scope(self):
        protected = self.tooling / "package.json"
        protected.write_text("keep")
        with self.assertRaisesRegex(ValueError, "unexpected pinned enum"):
            s03.clear_enum_outputs(self.tooling, [self.expected[0], "package.json"])
        self.assertEqual(protected.read_text(), "keep")

    def test_extra_enum_output_is_rejected(self):
        (self.tooling / s03.CLIENT / "enums/unexpected.ts").write_text("extra")
        self.assertFalse(self.compare()[0])

    def test_enum_and_proto_byte_changes_fail_independently(self):
        for path in self.expected:
            with self.subTest(path=path):
                output = self.tooling / path
                output.write_bytes(self.originals[path].replace(b"\r\n", b"\n"))
                self.assertFalse(self.compare()[0])
                output.write_bytes(self.originals[path])

    def test_export_renderer_must_also_match_generator_control(self):
        self.control.write_bytes(b"wrong control\r\n")
        self.assertFalse(self.compare()[0])

    def test_client_inventory_comes_from_git_pin_and_rejects_empty_enum_set(self):
        with patch.object(s03, "run", return_value=b"packages/typescript/src/enums/example.ts\nREADME.md\n") as run:
            self.assertEqual(s03.client_inventory(self.root, PIN), self.expected)
            self.assertIn(PIN, run.call_args.args[0])
        with patch.object(s03, "run", return_value=b"README.md\n"):
            with self.assertRaisesRegex(ValueError, "empty pinned enum"):
                s03.client_inventory(self.root, PIN)


class ProvenanceAndRuntimeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "upstream").mkdir()
        (self.root / "upstream/.git").write_text("gitdir: elsewhere")

    def check(self, head=PIN, link=PIN, status=b""):
        outputs = [head.encode(), f"160000 commit {link}\tupstream\n".encode(), status]
        with patch.object(s03, "run", side_effect=outputs):
            s03.check_upstream(self.root, PIN)

    def test_clean_head_gitlink_and_ledger_must_agree(self):
        self.check()
        with self.assertRaisesRegex(ValueError, "must agree"):
            self.check(head="2" * 40)
        with self.assertRaisesRegex(ValueError, "must agree"):
            self.check(link="2" * 40)

    def test_dirty_or_untracked_canonical_source_is_rejected(self):
        for dirty in (b" M tools/gen-proto/main.go\n", b"?? unexpected.go\n"):
            with self.subTest(status=dirty):
                with self.assertRaisesRegex(ValueError, "must be clean"):
                    self.check(status=dirty)

    def test_uninitialized_or_malformed_pin_fails_before_running_tools(self):
        with patch.object(s03, "run") as run:
            with self.assertRaisesRegex(ValueError, "invalid upstream pin"):
                s03.check_upstream(self.root, "HEAD")
            (self.root / "upstream/.git").unlink()
            with self.assertRaisesRegex(ValueError, "submodule update --init upstream"):
                s03.check_upstream(self.root, PIN)
            run.assert_not_called()

    def package(self, manager="npm@11.17.0+sha512-integrity"):
        (self.root / "upstream/package.json").write_text(json.dumps({
            "volta": {"node": "24.20.0", "npm": "11.17.0"}, "packageManager": manager,
        }))

    def test_integrity_suffix_is_accepted_and_effective_toolchains_are_checked(self):
        self.package()
        with patch.object(s03, "load_toolchains", return_value={"go": "go1.27.1"}), \
                patch.object(s03, "run", side_effect=[b"v24.20.0\n", b"11.17.0\n", b"go1.27.1\n"]), \
                patch.dict(os.environ, {"GOTOOLCHAIN": "auto", "GOFLAGS": "-race", "NODE_OPTIONS": "--inspect"}):
            env = s03.version_environment(self.root, self.root / "tooling")
        self.assertEqual(env["GOTOOLCHAIN"], "local")
        self.assertNotIn("-race", env["GOFLAGS"])
        self.assertNotIn("NODE_OPTIONS", env)

    def test_disagreeing_npm_pin_and_wrong_node_version_are_rejected(self):
        self.package("npm@11.16.0")
        with self.assertRaisesRegex(ValueError, "pins disagree"):
            s03.version_environment(self.root, self.root / "tooling")
        self.package()
        with patch.object(s03, "run", return_value=b"v24.15.0\n"):
            with self.assertRaisesRegex(ValueError, "requires node"):
                s03.version_environment(self.root, self.root / "tooling")

    def test_missing_tool_and_failed_subprocess_cannot_return_success(self):
        with patch("s04_common.subprocess.run", side_effect=FileNotFoundError("missing node")):
            with self.assertRaises(FileNotFoundError):
                s03.run(["node", "--version"], self.root)
        failure = subprocess.CompletedProcess(["go"], returncode=2, stdout=b"", stderr=b"")
        with patch("s04_common.subprocess.run", return_value=failure):
            with self.assertRaisesRegex(RuntimeError, "command exited 2"):
                s03.run(["go", "version"], self.root)

    def test_cli_retains_a_diagnostic_and_fails_on_missing_tools(self):
        stderr, stdout = io.StringIO(), io.StringIO()
        with patch.object(s03, "ROOT", self.root), \
                patch.object(s03, "prepare", side_effect=FileNotFoundError("missing pinned node")), \
                patch.object(sys, "argv", ["s03.py", "prepare", "--output", str(self.root / "stage"), "--pin", PIN]), \
                patch.object(sys, "stderr", stderr), patch.object(sys, "stdout", stdout):
            self.assertEqual(s03.main(), 1)
        self.assertIn("S03 generation failed: missing pinned node", stderr.getvalue())
        self.assertEqual(stdout.getvalue(), "")


class HandwrittenEnumInventoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.enum_dir = self.root / "upstream" / s03.CLIENT / "enums"
        self.enum_dir.mkdir(parents=True)
        (self.root / "data/s03").mkdir(parents=True)
        self.inventory = {"version": 1, "enums": [{"name": "Authored", "file": "authored.enum.ts"}]}
        self.expected = [f"{s03.CLIENT}/api/proto.generated.ts", f"{s03.CLIENT}/enums/authored.enum.ts",
                         f"{s03.CLIENT}/enums/authored.ts", f"{s03.CLIENT}/enums/generated.ts"]
        for name in ("authored.enum.ts", "authored.ts", "generated.ts"):
            (self.enum_dir / name).write_text("authored source")

    def inputs(self):
        (self.root / "data/s03/handwritten-enums.json").write_text(json.dumps(self.inventory))
        return s03.handwritten_inputs(self.root, self.expected)

    def test_both_source_files_survive_while_generated_output_is_cleared(self):
        inputs = self.inputs()
        self.assertEqual(inputs, set(self.expected[1:3]))
        s03.clear_enum_outputs(self.root / "upstream", self.expected, inputs)
        self.assertTrue((self.enum_dir / "authored.enum.ts").exists())
        self.assertTrue((self.enum_dir / "authored.ts").exists())
        self.assertFalse((self.enum_dir / "generated.ts").exists())

    def test_generated_header_cannot_be_relabeled_as_a_source_input(self):
        for name in ("authored.enum.ts", "authored.ts"):
            with self.subTest(name=name):
                (self.enum_dir / name).write_text("// Code generated by Herebyfile.mjs; DO NOT EDIT.")
                with self.assertRaisesRegex(ValueError, "cannot be declared a handwritten"):
                    self.inputs()
                (self.enum_dir / name).write_text("authored source")

    def test_source_inventory_requires_the_complete_pair_in_the_pin(self):
        self.expected.remove(f"{s03.CLIENT}/enums/authored.ts")
        with self.assertRaisesRegex(ValueError, "does not match the pin"):
            self.inputs()

    def test_duplicate_or_outside_source_paths_are_rejected(self):
        self.inventory["enums"].append(copy.deepcopy(self.inventory["enums"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate handwritten"):
            self.inputs()
        self.inventory["enums"] = [{"name": "Unsafe", "file": "../../package.enum.ts"}]
        with self.assertRaisesRegex(ValueError, "invalid handwritten"):
            self.inputs()


if __name__ == "__main__":
    unittest.main()
