"""Failure-path checks for the exact benchmark graph prerequisite consumer."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

import s07_benchmark_graph as graph


class PrerequisiteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.frozen=json.loads((graph.ROOT/"data/s07/bindworkload-probes.json").read_bytes())
        cls.source={"sha256":"a"*64,"files":{"production.rs":"b"*64}}
        cls.binaries={"go":"c"*64,"rust":"d"*64,"rust_allocation":"e"*64}
        cls.configuration={"/external/.cargo/config.toml":"f"*64}
        cls.report={"version":1,"diagnostic_subset":False,"source_stable":True,"parity":1,"files":13094,
            "source_fingerprint":cls.source,"source_fingerprint_after":cls.source,
            "cargo_configuration":cls.configuration,"cargo_configuration_after":cls.configuration,
            "binary_sha256":{"oracle":cls.binaries["go"],"rust":cls.binaries["rust"]},
            "binary_sha256_after":{"oracle":cls.binaries["go"],"rust":cls.binaries["rust"]},
            **{key:cls.frozen[key] for key in ("expected_scalars","input_sha256","options_sha256","workload_sha256")},
            "runs":[{"workers":workers,"files":13094,"passed_files":13094,"parity":1,
                "expected_scalars":cls.frozen["expected_scalars"],"rust_scalars":cls.frozen["expected_scalars"],
                "results":[{"index":index,"equal":True,"raw_exact":False,"first_difference":None} for index in range(13094)]} for workers in (1,8)]}

    def validate(self,report):
        return graph.validate_measurement_prerequisite(report,self.source,self.binaries,self.configuration)

    def test_complete_two_mode_control(self):
        from s07_benchmark_inputs import loaded_input_digest
        self.assertEqual(self.validate(self.report),{**self.frozen["expected_scalars"], "loaded_input_sha256":loaded_input_digest(self.frozen["requests"])})

    def test_rejects_stale_partial_or_manufactured_aggregate(self):
        mutations={
            "diagnostic":lambda r:r.update(diagnostic_subset=True),
            "unstable":lambda r:r.update(source_stable=False),
            "stale_source":lambda r:r["source_fingerprint"].update(sha256="f"*64),
            "stale_source_after":lambda r:r.update(source_fingerprint_after={}),
            "missing_configuration":lambda r:r.pop("cargo_configuration"),
            "missing_configuration_after":lambda r:r.pop("cargo_configuration_after"),
            "stale_configuration":lambda r:r.update(cargo_configuration={}),
            "changed_configuration_after":lambda r:r.update(cargo_configuration_after={}),
            "stale_binary":lambda r:r["binary_sha256"].update(rust="f"*64),
            "changed_binary_after":lambda r:r["binary_sha256_after"].update(oracle="f"*64),
            "missing_mode":lambda r:r["runs"].pop(),
            "duplicate_mode":lambda r:r["runs"][1].update(workers=1),
            "reordered_modes":lambda r:r["runs"].reverse(),
            "missing_file":lambda r:r["runs"][0]["results"].pop(),
            "duplicated_file":lambda r:r["runs"][0]["results"][1].update(index=0),
            "failed_file":lambda r:r["runs"][1]["results"][8000].update(equal=False),
            "hidden_difference":lambda r:r["runs"][0]["results"][7].update(first_difference={"path":"flow"}),
            "false_scalar":lambda r:r["runs"][0].update(rust_scalars={}),
            "boolean_version":lambda r:r.update(version=True),
            "boolean_worker_mode":lambda r:r["runs"][0].update(workers=True),
            "boolean_parity":lambda r:r.update(parity=True),
            "boolean_file_index":lambda r:r["runs"][0]["results"][1].update(index=True),
            "changed_input":lambda r:r.update(input_sha256="f"*64),
        }
        for name,mutate in mutations.items():
            with self.subTest(name=name):
                report=copy.deepcopy(self.report);mutate(report)
                with self.assertRaises(ValueError):self.validate(report)

    def test_allocation_binary_cannot_replace_normal_binary(self):
        binaries={**self.binaries,"rust":self.binaries["rust_allocation"]}
        with self.assertRaises(ValueError):graph.validate_measurement_prerequisite(self.report,self.source,binaries)

    def test_legacy_diagnostic_protocol_does_not_qualify_native_capture(self):
        report=copy.deepcopy(self.report)
        del report["cargo_configuration"];del report["cargo_configuration_after"]
        graph.validate_measurement_prerequisite(report,self.source,self.binaries)
        with self.assertRaisesRegex(ValueError,"Cargo configuration"):
            self.validate(report)

    def test_missing_frozen_inventory_fails_closed(self):
        with patch.object(Path,"read_bytes",side_effect=FileNotFoundError):
            with self.assertRaises(FileNotFoundError):self.validate(self.report)

    def test_cached_binary_cannot_freeze_or_publish(self):
        for args in (("capture","--no-build"),("freeze","--no-build","--write-manifest"),("freeze","--no-build","--diagnostic","--write-manifest")):
            result=subprocess.run([sys.executable,str(graph.ROOT/"scripts/s07_benchmark_graph.py"),*args],capture_output=True)
            self.assertNotEqual(result.returncode,0)
            self.assertTrue(b"diagnostic" in result.stderr)


if __name__=="__main__":unittest.main()
