"""Native baseline failures and missing pulls cannot certify S08 parity."""
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from s08_baselines import (LEGACY_ARCHIVE, LEGACY_MEMBER, canonical, digest, replace_exact,
                           review_capture, review_legacy_failures, validate_observations)


class BaselineObservationContract(unittest.TestCase):
    def fixture(self):
        variant={"id":"case#0","options":{},"harness_options":{"NoTypesAndSymbols":False},"dependency_closure":[0],
                 "option_outcome":"accepted","option_diagnostics":[]}
        request={"id":"case#0","raw_sha256":"raw","loaded_sha256":"loaded","acceptance_tier":"acceptance"}
        query={"operation":"GetTypeAtLocation","file":"a.ts","kind":1,"pos":0,"end":1}
        result={**request,"state":"executed","execution_stage":"complete","options":{},"harness_options":variant["harness_options"],
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

    def test_execution_cannot_promote_an_informational_case_into_acceptance(self):
        args=self.fixture()
        args[1][0]["acceptance_tier"]="informational"
        with self.assertRaises(ValueError):validate_observations(*args)

    def test_missing_duplicate_and_wrong_input_fail(self):
        rows,requests,results,files=self.fixture()
        for changed in ([],results*2,[dict(results[0],id="other")],[dict(results[0],raw_sha256="wrong")]):
            with self.assertRaises(ValueError):
                validate_observations(rows,requests,changed,files)

    def test_native_failure_is_retained_as_failure_not_an_empty_baseline(self):
        rows,requests,results,files=self.fixture()
        rows[0][1].update(option_outcome="rejected",option_diagnostics=[{"code":5023}])
        results=[{**requests[0],"state":"upstream_failed","queries":[],"execution_stage":"native_options",
                  "failure":{"stage":"native_options","reason":"option_rejected","message":"Unknown compiler option 'removed'."}}]
        self.assertEqual(validate_observations(rows,requests,results,files),[])
        self.assertEqual(results[0]["state"],"upstream_failed")
        for state in ("not_executed","pass",True):
            with self.assertRaises(ValueError):
                validate_observations(rows,requests,[dict(results[0],state=state)],files)

    def test_rejected_options_do_not_excuse_missing_phase_or_harness_failures(self):
        for tier in ("acceptance","informational"):
            rows,requests,results,files=self.fixture()
            requests[0]["acceptance_tier"]=tier
            rows[0][1].update(option_outcome="rejected",option_diagnostics=[{"code":5023}])
            for state in ("upstream_failed","harness_failed"):
                failed={**requests[0],"state":state,"queries":[]}
                with self.subTest(tier=tier,state=state), self.assertRaises(ValueError):
                    validate_observations(rows,requests,[failed],files)
            for stage,reason in (("harness_input","option_rejected"),("native_compile","option_rejected"),
                                 ("native_options","source_digest")):
                failed={**requests[0],"state":"upstream_failed","execution_stage":stage,"queries":[],
                        "failure":{"stage":stage,"reason":reason,"message":"failed"}}
                with self.subTest(stage=stage,reason=reason), self.assertRaises(ValueError):
                    validate_observations(rows,requests,[failed],files)

    def test_native_option_rejection_requires_frozen_rejection_and_diagnostics(self):
        for outcome,diagnostics in (("accepted",[]),("rejected",[]),("accepted",[{"code":5023}])):
            rows,requests,_,files=self.fixture()
            rows[0][1].update(option_outcome=outcome,option_diagnostics=diagnostics)
            failed={**requests[0],"state":"upstream_failed","execution_stage":"native_options","queries":[],
                    "failure":{"stage":"native_options","reason":"option_rejected","message":"rejected"}}
            with self.subTest(outcome=outcome,diagnostics=diagnostics), self.assertRaises(ValueError):
                validate_observations(rows,requests,[failed],files)

    def test_native_panics_and_assertions_remain_unresolved_even_on_rejected_informational_rows(self):
        rows,requests,_,files=self.fixture()
        rows[0][1].update(option_outcome="rejected",option_diagnostics=[{"code":5023}])
        requests[0]["acceptance_tier"]="informational"
        for reason in ("native_panic","native_assertion"):
            failed={**requests[0],"state":"upstream_failed","execution_stage":"native_options","queries":[],
                    "failure":{"stage":"native_options","reason":reason,"message":"bounds"}}
            if reason=="native_panic":failed["panic"]="bounds"
            self.assertEqual(validate_observations(rows,requests,[failed],files),
                             [{"id":"case#0","field":"unexpected_native_failure"}])
            if reason=="native_panic":
                del failed["panic"]
                with self.assertRaises(ValueError):validate_observations(rows,requests,[failed],files)

    def test_equal_diagnostic_counts_do_not_hide_payload_changes(self):
        rows,requests,results,files=self.fixture()
        results[0]["pre_diagnostics"]=[{"code":100,"text_hex":"61"}]
        results[0]["post_diagnostics"]=[{"code":100,"text_hex":"62"}]
        self.assertEqual(validate_observations(rows,requests,results,files),[{"id":"case#0","field":"pre_post_diagnostics"}])

    def test_skipped_rows_still_compare_observed_inputs_options_and_diagnostics(self):
        rows,requests,results,files=self.fixture()
        results[0].update(state="upstream_skipped",execution_stage="native_option_guard",options={"changed":True},post_diagnostics=[{"code":100}])
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
                (capture/"report.json").write_bytes(canonical({"version":3,"pin":"pin","source_inputs":inputs,
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

    def test_legacy_failures_require_authenticated_native_log_and_matching_rejection_site(self):
        for changed in (None,"tampered","wrong_site","harness_error","missing_failure","wrong_option","accepted"):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary);capture=root/"capture";capture.mkdir()
                rows,requests,_,_=self.fixture()
                rows[0][1].update(option_outcome="rejected",option_diagnostics=[{"code":5023}])
                requests[0]["settings"]={"removed":"true"}
                observed=[{**requests[0],"state":"upstream_failed","queries":[]}]
                detail="        harnessutil.go:313: Unknown compiler option 'removed'."
                if changed=="wrong_site":detail=detail.replace(":313:",":999:")
                if changed=="harness_error":detail="        s08_baselines_test.go:160: configured name drift"
                if changed=="wrong_option":detail=detail.replace("'removed'","'other'")
                if changed=="accepted":rows[0][1]["option_outcome"]="accepted"
                body=f"    --- FAIL: TestS08Baselines/case#0 (0.00s)\n{detail}\n"
                if changed=="missing_failure":body=""
                stdout="--- FAIL: TestS08Baselines (1.00s)\n"+body+"FAIL\nFAIL\tgithub.com/microsoft/TypeScript/tsc/internal/testrunner\t1.000s\nFAIL\n"
                payloads={"report.json":b"{}","requests.json":canonical(requests),
                          "observations.ndjson":canonical(observed[0])+b"\n","go.stdout":stdout.encode(),"go.stderr":b""}
                archive=root/LEGACY_ARCHIVE;archive.parent.mkdir(parents=True);archive.write_bytes(b"authenticated fixture archive")
                manifest={"archive_sha256":digest(archive.read_bytes()),"files":{}}
                for name,raw in payloads.items():
                    (capture/name).write_bytes(raw)
                    manifest["files"][LEGACY_MEMBER+name]={"sha256":digest(raw),"bytes":len(raw)}
                Path(str(archive)+".manifest.json").write_bytes(canonical(manifest))
                if changed=="tampered":(capture/"go.stdout").write_bytes(b"changed")
                with patch("s08_baselines.ROOT",root):
                    if changed:
                        with self.assertRaises(ValueError):review_legacy_failures(capture,rows,requests,observed)
                    else:
                        failures,classification=review_legacy_failures(capture,rows,requests,observed)
                        self.assertEqual(failures["case#0"]["reason"],"option_rejected")
                        self.assertEqual(classification["kind"],"authenticated_legacy_native_log")
