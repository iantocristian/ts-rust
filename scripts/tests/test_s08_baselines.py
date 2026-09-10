"""Native baseline failures and missing pulls cannot certify S08 parity."""
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from s08_baselines import canonical, digest, replace_exact, review_capture, validate_observations


class BaselineObservationContract(unittest.TestCase):
    def fixture(self):
        variant={"id":"case#0","options":{},"harness_options":{"NoTypesAndSymbols":False},"dependency_closure":[0]}
        request={"id":"case#0","raw_sha256":"raw","loaded_sha256":"loaded"}
        query={"operation":"GetTypeAtLocation","file":"a.ts","kind":1,"pos":0,"end":1}
        result={**request,"state":"executed","options":{},"harness_options":variant["harness_options"],
                "files":[{"name":"a.ts","path":"a.ts","bytes":1,"sha256":"source"}],
                "types":{"state":"content","text_hex":"78"},"symbols":{"state":"no_content"},
                "errors":{"state":"no_content"},"queries":[query],"pre_diagnostics":[],"post_diagnostics":[]}
        files=[{"Name":"a.ts","Path":"a.ts","Bytes":1,"SHA256":"source"}]
        return [({},variant)],[request],[result],files

    def test_complete_native_observation_has_no_drift(self):
        self.assertEqual(validate_observations(*self.fixture()),[])

    def test_empty_content_is_an_explicit_baseline_not_no_content(self):
        args=self.fixture()
        args[2][0]["types"]={"state":"content","text_hex":""}
        self.assertEqual(validate_observations(*args),[])

    def test_missing_duplicate_and_wrong_input_fail(self):
        rows,requests,results,files=self.fixture()
        for changed in ([],results*2,[dict(results[0],id="other")],[dict(results[0],raw_sha256="wrong")]):
            with self.assertRaises(ValueError):
                validate_observations(rows,requests,changed,files)

    def test_native_failure_is_retained_as_failure_not_an_empty_baseline(self):
        rows,requests,results,files=self.fixture()
        results=[{**requests[0],"state":"upstream_failed","queries":[]}]
        self.assertEqual(validate_observations(rows,requests,results,files),[])
        self.assertEqual(results[0]["state"],"upstream_failed")
        for state in ("not_executed","pass",True):
            with self.assertRaises(ValueError):
                validate_observations(rows,requests,[dict(results[0],state=state)],files)

    def test_equal_diagnostic_counts_do_not_hide_payload_changes(self):
        rows,requests,results,files=self.fixture()
        results[0]["pre_diagnostics"]=[{"code":100,"text_hex":"61"}]
        results[0]["post_diagnostics"]=[{"code":100,"text_hex":"62"}]
        self.assertEqual(validate_observations(rows,requests,results,files),[{"id":"case#0","field":"pre_post_diagnostics"}])

    def test_skipped_rows_still_compare_observed_inputs_options_and_diagnostics(self):
        rows,requests,results,files=self.fixture()
        results[0].update(state="upstream_skipped",options={"changed":True},post_diagnostics=[{"code":100}])
        self.assertEqual(validate_observations(rows,requests,results,files),[
            {"id":"case#0","field":"options"}, {"id":"case#0","field":"pre_post_diagnostics"}])
        results[0]["raw_sha256"]="wrong"
        with self.assertRaises(ValueError): validate_observations(rows,requests,results,files)

    def test_executed_rows_cannot_hide_panics_or_missing_observations(self):
        args=self.fixture()
        args[2][0]["panic"]="bounds"
        with self.assertRaises(ValueError): validate_observations(*args)
        for key in ("options","harness_options","pre_diagnostics","post_diagnostics"):
            args=self.fixture()
            del args[2][0][key]
            with self.assertRaises(ValueError): validate_observations(*args)

    def test_loaded_source_identity_is_compared_even_with_equal_byte_counts(self):
        rows,requests,results,files=self.fixture()
        results[0]["files"][0]["sha256"]="same-length-edit"
        self.assertEqual(validate_observations(rows,requests,results,files),[{"id":"case#0","field":"loaded_files"}])

    def test_baseline_enablement_and_query_shapes_are_not_truthiness_checks(self):
        for key,value in (("types",{"state":"disabled"}),("errors",{"state":"disabled"}),
                          ("symbols",{"state":"no_content","text_hex":""}),
                          ("types",{"state":"content","text_hex":"invalid"})):
            args=self.fixture()
            args[2][0][key]=value
            with self.assertRaises(ValueError): validate_observations(*args)
        for changes in ({"operation":"unsupported"},{"kind":True},{"pos":None},{"absent":1},{"file":""}):
            args=self.fixture()
            args[2][0]["queries"][0].update(changes)
            with self.assertRaises(ValueError): validate_observations(*args)

    def test_observation_hooks_require_exact_source_anchors(self):
        self.assertEqual(replace_exact("a + b","a","x"),"x + b")
        for source in ("none","a + a"):
            with self.assertRaises(ValueError): replace_exact(source,"a","x")

    def test_review_rejects_tampered_evidence_and_partial_inventory(self):
        rows,requests,results,files=self.fixture()
        for changed in (None,"requests.json","observations.ndjson","source-snapshot/data/s07/subset.json","partial"):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary)
                capture=root/"capture"
                subset=canonical({"file_observations":files})
                inputs={"data/s07/subset.json":digest(subset)}
                for path in (root/"data/s07/subset.json",capture/"source-snapshot/data/s07/subset.json"):
                    path.parent.mkdir(parents=True,exist_ok=True)
                    path.write_bytes(subset)
                request_raw=canonical([] if changed=="partial" else requests)
                observation_raw=canonical(results[0])+b"\n"
                (capture/"requests.json").write_bytes(request_raw)
                (capture/"observations.ndjson").write_bytes(observation_raw)
                (capture/"report.json").write_bytes(canonical({"pin":"pin","source_inputs":inputs,
                    "request_sha256":digest(request_raw),"observation_sha256":digest(observation_raw)}))
                if changed not in (None,"partial"):
                    (capture/changed).write_bytes(b"tampered")
                with patch("s08_baselines.ROOT",root), patch("s08_baselines.requests_from_subset",return_value=(rows,requests)), patch("builtins.print"):
                    if changed:
                        with self.assertRaises(ValueError): review_capture(capture,root/"inventory.json")
                    else:
                        review_capture(capture,root/"inventory.json")
                        self.assertTrue((root/"inventory.json").exists())
                        with self.assertRaises(FileExistsError): review_capture(capture,root/"inventory.json")
