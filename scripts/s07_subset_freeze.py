"""Join source-only eligibility with actual pinned loader dependencies."""

from collections import Counter
from pathlib import Path

from s04 import verified_upstream
from s04_common import strict_json_loads
from s07_subset import ROOT, PIN, TABLES, classify, json_bytes, sha256, table
from s07_operation_validation import validate_operation_matrix


CAPABILITIES = {
    "tokens_and_recovery": ("Declaration token/recovery handling", "checkSourceElementWorker", 2294),
    "structure_and_recovery": ("Declaration traversal and recovered nodes", "checkSourceElementWorker", 2294),
    "declaration_members": ("Member signatures, accessors and overloads", "checkInterfaceDeclaration", 5077),
    "declarations_and_merging": ("Ambient declarations and declaration merging", "checkSourceElementWorker", 2294),
    "non_generic_type_syntax": ("Type references, type operators and compound types", "getTypeFromTypeNodeWorker", 23152),
    "expressions_and_bindings": ("Declaration expressions and value/type references", "checkVariableLikeDeclaration", 5950),
    "control_flow": ("Function/declaration bodies and flow-sensitive types", "checkSourceElementWorker", 2294),
    "imports_and_exports": ("Declaration imports, aliases and reexports", "checkImportDeclaration", 5393),
    "jsdoc": ("JSDoc declarations and contained types", "getTypeFromTypeNodeWorker", 23152),
    "explicit_type_parameters": ("Generic type parameter environments and constraints", "checkTypeParameter", 2645),
    "nonempty_type_arguments": ("Generic reference instantiation and argument substitution", "getTypeFromTypeReference", 23344),
    "conditional_types": ("Conditional type construction and instantiation", "getTypeFromConditionalTypeNode", 24619),
    "infer_types": ("Inference variables in conditional types", "getTypeFromInferTypeNode", 24917),
    "mapped_types": ("Mapped type construction and instantiation", "getTypeFromMappedTypeNode", 24605),
    "indexed_access_types": ("Indexed-access type construction and relations", "getTypeFromIndexedAccessTypeNode", 23290),
    "template_literal_types": ("Template-literal type construction", "getTypeFromTemplateTypeNode", 24589),
    "import_types": ("Import-type resolution in declarations", "checkImportType", 3368),
    "decorators": ("Decorator type checking", "checkSourceElementWorker", 2294),
    "jsx": ("JSX types and library declarations", "checkSourceElementWorker", 2294),
    "transformation_only": ("Synthetic declaration forms", "checkSourceElementWorker", 2294),
}


def verify_loader(path, requests):
    path = Path(path)
    data = path.read_bytes()
    manifest = strict_json_loads(path.with_suffix(".manifest.json").read_bytes())
    if (manifest["upstream_pin"] != PIN or manifest["requests_sha256"] != sha256(json_bytes(requests))
            or manifest["observations_sha256"] != sha256(data)
            or manifest["adapter_sha256"] != sha256((ROOT / "tools/s07/program/export_test.go").read_bytes())):
        raise ValueError("program closure observation is stale or belongs to different candidate requests")
    rows = strict_json_loads(data)
    upstream = verified_upstream()
    expected_sources = {str(path.relative_to(upstream)): sha256(path.read_bytes())
                        for path in sorted((upstream / "tsc/internal/compiler").glob("*.go"))
                        if not path.name.endswith("_test.go")}
    expected_sources["tsc/internal/compiler/s06_metadata_bridge.go"] = sha256((ROOT / "scripts/s06_oracle/metadata_bridge.go").read_bytes())
    if manifest["source_sha256"] != expected_sources:
        raise ValueError("loader source fingerprint differs from pinned compiler source inventory")
    if manifest["rows"] != len(rows) or [row["ID"] for row in rows] != [row["id"] for row in requests]:
        raise ValueError("missing, extra, duplicate or reordered loader closure")
    if any(row.get("Panic") for row in rows):
        raise ValueError("source loader panic prevents a complete dependency closure; retain the row and resolve before freeze")
    return rows, manifest


def library_obligations(result, loader):
    syntax_table = table("syntax-table.json", "kind")
    source_lines = (verified_upstream() / "tsc/internal/checker/checker.go").read_text().splitlines()
    for _, function, line in CAPABILITIES.values():
        if line < 1 or line > len(source_lines) or not source_lines[line-1].startswith(f"func (c *Checker) {function}("):
            raise ValueError(f"checker capability anchor does not name the pinned function: {function}:{line}")
    known = {row["filename"]: row for row in result["libraries"]}
    variants = {v["id"]: (case, v) for case in result["cases"] for v in case["variants"]}
    observed_libraries = {}
    for row in loader:
        for file in row["Files"]:
            if not file["Lib"] and file["Name"] not in known:
                continue
            key = (file["Name"], file["SHA256"])
            if key in observed_libraries:
                continue
            if file["Name"] in known:
                source = known[file["Name"]]
                text = bytes.fromhex(source["text_hex"])
                syntax = source["syntax"]
                anchor = source["path"]
            else:
                # Source-case package replacements retain the exact virtual
                # unit and byte coordinate instead of inventing a bundled path.
                case, variant = variants[row["ID"]]
                inputs = case["source"]["variants"][variant["configuration"]]["inputs"]
                found = next((u for u in inputs if u["filename"] == file["Name"]), None)
                if found is None or found["source_sha256"] != file["SHA256"]:
                    raise ValueError("loaded declaration library has no source syntax witness: " + file["Name"])
                syntax = next(u["syntax"] for u in variant["source_syntax"] if u["unit"] == found["unit"])
                text = None
                anchor = case["source"]["path"] + "#unit=" + str(found["unit"])
            if text is not None and (sha256(text) != file["SHA256"] or len(text) != file["Bytes"]):
                raise ValueError("loaded library bytes differ from source syntax witness")
            observed_libraries[key] = (row["ID"], file, syntax, anchor, text)
    obligations = []
    for (name, digest), (witness, file, syntax, anchor, text) in sorted(observed_libraries.items()):
        family_kinds = {}
        for kind in syntax["kinds"]:
            family = syntax_table[kind]["family"]
            family_kinds.setdefault(family, []).append(kind)
        if syntax["nonempty_type_arguments"]:
            family_kinds["nonempty_type_arguments"] = []
        for family, kinds in sorted(family_kinds.items()):
            if family not in CAPABILITIES:
                raise ValueError("library feature lacks checker obligation: " + family)
            capability, function, line = CAPABILITIES[family]
            positions = ([syntax["first_positions"][kind] for kind in kinds]
                         if kinds else syntax["nonempty_type_arguments"])
            pos = min(positions)
            obligations.append({
                "id": name + "#" + digest[:16] + "/" + family,
                "owner_sprint": "S08", "status": "planned", "family": family,
                "required_checker_capability": capability,
                "input_witness": witness, "library": name, "source_sha256": digest,
                "declaration_anchor": {"path": anchor, "byte_position": pos,
                                       "line": text[:max(pos,0)].count(b"\n") + 1 if text is not None else None},
                "pinned_source_anchor": {"path": "tsc/internal/checker/checker.go", "function": function, "line": line},
                "syntax_kinds": sorted(kinds), "expected_consumers": ["E2", "E7", "E8"],
                "measurement": "loaded declaration syntax; checker operation is required on use, not observed execution",
            })
    return {"version": 1, "pin": PIN, "scope": "all syntax families in actual loaded declaration/library closure",
            "library_identities": len(observed_libraries), "obligations": obligations}


def documents(observations, loader_observations):
    operation_matrix = validate_operation_matrix()
    result = classify(observations)
    loader, manifest = verify_loader(loader_observations, result["requests"])
    obligations = library_obligations(result, loader)
    by_id = {row["ID"]: row for row in loader}
    file_table, file_ids = [], {}
    for case in result["cases"]:
        for variant in case["variants"]:
            if variant["disposition"] != "eligible":
                continue
            observed = by_id[variant["id"]]
            ordered, libraries = [], []
            for file in observed["Files"]:
                key = json_bytes(file)
                if key not in file_ids:
                    file_ids[key] = len(file_table)
                    file_table.append(file)
                fid = file_ids[key]
                ordered.append(fid)
                if file["Lib"]:
                    libraries.append(fid)
            variant["effective_lib_closure"] = libraries
            variant["dependency_closure"] = ordered
            variant["loader_observation"] = {key: value for key, value in observed.items() if key not in {"ID", "Files"}}
    rule = result["rule"]
    rule["classifier_inputs"]["scripts/s07_subset_freeze.py"] = sha256((ROOT / "scripts/s07_subset_freeze.py").read_bytes())
    rule["classifier_inputs"]["scripts/s07_operation_validation.py"] = sha256((ROOT / "scripts/s07_operation_validation.py").read_bytes())
    rule["provenance"] = {
        "syntax": strict_json_loads(Path(observations).with_suffix(".provenance.json").read_bytes()),
        "loader": manifest,
    }
    rule["dependency_counts"] = {"programs": len(loader), "ordered_files": sum(len(r["Files"]) for r in loader),
                                 "distinct_file_observations": len(file_table), "library_identities": obligations["library_identities"],
                                 "checker_obligations": len(obligations["obligations"]),
                                 "checker_families": dict(Counter(o["family"] for o in obligations["obligations"]))}
    rule["operation_matrix"] = {**operation_matrix, "E2": {"frozen_subset": "S07", "checker_parity": "S08 planned"},
                                "E7": {"selection_and_dependencies": "same frozen variant IDs and ordered closure", "execution": "later sprint"},
                                "E8": {"selection_and_dependencies": "same frozen variant IDs and ordered closure", "execution": "later sprint"}}
    rule["mandatory_fixtures"] = strict_json_loads((TABLES / "mandatory-fixtures.json").read_bytes())
    rule["tables"]["mandatory-fixtures.json"] = sha256((TABLES / "mandatory-fixtures.json").read_bytes())
    eligible = {v["id"] for c in result["cases"] for v in c["variants"] if v["disposition"] == "eligible"}
    if not rule["counts"]["eligible_malformed_variants"]:
        raise ValueError("malformed-input stress family is empty")
    for fixture in rule["mandatory_fixtures"]["rows"]:
        if fixture.get("primary_case") and not any(i.startswith(fixture["primary_case"]+"#configuration=") for i in eligible):
            raise ValueError("mandatory stress family has no eligible witness: " + fixture["id"])
    return {"subset-rule.json": rule,
            "subset.json": {"version": 1, "pin": PIN, "cases": result["cases"], "file_observations": file_table},
            "checker-obligations.json": obligations}


def prepare_review(observations, loader_observations, destination):
    docs = documents(observations, loader_observations)
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    for name, value in docs.items():
        (destination / name.replace(".json", ".candidate.json")).write_bytes(json_bytes(value))
    payload = {"version": 1, "pin": PIN,
               "candidate_sha256": {name: sha256(json_bytes(value)) for name, value in docs.items()},
               "counts": docs["subset-rule.json"]["counts"],
               "by_feature": docs["subset-rule.json"]["by_feature"],
               "dependency_counts": docs["subset-rule.json"]["dependency_counts"]}
    (destination / "review-payload.json").write_bytes(json_bytes(payload))
    return payload


def freeze(observations, loader_observations, review_path, write=False):
    docs = documents(observations, loader_observations)
    review = strict_json_loads(Path(review_path).read_bytes())
    expected = {name: sha256(json_bytes(value)) for name, value in docs.items()}
    if (review.get("pin") != PIN or review.get("candidate_sha256") != expected
            or review.get("status") != "accepted" or not review.get("reviewer")
            or type(review.get("findings")) is not list or review.get("required_program_operations_reviewed") is not True):
        raise ValueError("subset requires independent review of these exact candidates and required program operation scope")
    docs["subset-rule.json"]["state"] = "frozen"
    docs["subset-rule.json"]["review_sha256"] = sha256(json_bytes(review))
    docs["subset-review.json"] = review
    differences = []
    for name, value in docs.items():
        path = ROOT / "data/s07" / name
        if not path.exists() or path.read_bytes() != json_bytes(value):
            differences.append(name)
    if differences and not write:
        raise ValueError("reviewed subset manifest differs: " + ", ".join(differences))
    if write:
        for name, value in docs.items():
            (ROOT / "data/s07" / name).write_bytes(json_bytes(value))
    return {"frozen_subset": True, "eligible_variants": docs["subset-rule.json"]["counts"]["eligible_variants"]}
