"""Frozen physical membership and authoritative Go preprocessing observations."""

from collections import Counter
import copy
import json
from pathlib import Path

from s04_common import command, strict_json_loads
from s06_protocol import canonical, exact_keys, hex_bytes, integer, parser_request, sha256, text

CASE_ROOTS = ("tsc/testdata/tests/cases/compiler/", "tsc/testdata/tests/cases/conformance/")
LIB_ROOT = "tsc/internal/bundled/libs/"
EXPECTED = {"physical_cases": 12721, "libraries": 108, "primary_rows": 12829,
            "extracted_units": 17264, "extracted_bytes": 7528194, "library_bytes": 3785075,
            "script_kinds": {"0": 29, "1": 1360, "2": 16, "3": 14746, "4": 472, "6": 641},
            "multiunit_cases": 2078, "symlink_cases": 25, "explicit_cwd_cases": 50}
LEGACY_PATHS = {CASE_ROOTS[0] + name for name in (
    "moduleNoneDynamicImport.ts", "moduleNoneErrors.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile1.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile2.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile3.ts",
    "requireOfJsonFileWithModuleEmitNone.ts", "requireOfJsonFileWithModuleNodeResolutionEmitNone.ts",
)}


def membership(upstream, pin):
    paths = command(["git", "ls-tree", "-r", "--name-only", pin, *CASE_ROOTS, LIB_ROOT], cwd=upstream).decode().splitlines()
    cases = [p for p in paths if p.startswith(CASE_ROOTS) and p.endswith((".ts", ".tsx", ".js", ".jsx"))]
    libraries = [p for p in paths if p.startswith(LIB_ROOT) and p.endswith(".d.ts")]
    if len(cases) != EXPECTED["physical_cases"] or len(libraries) != EXPECTED["libraries"]:
        raise ValueError("S06 physical membership differs from the reviewed denominator")
    return sorted(cases), sorted(libraries)


def primary_id(path):
    for prefix in CASE_ROOTS:
        if path.startswith(prefix):
            return path.removeprefix("tsc/testdata/tests/cases/")
    if path.startswith(LIB_ROOT):
        return "lib/" + path.removeprefix(LIB_ROOT)
    raise ValueError(f"not an S06 primary path: {path}")


def read_export(path, physical, libraries):
    records = []
    with Path(path).open("rb") as stream:
        while line := stream.readline(64 * 1024 * 1024 + 1):
            if len(line) > 64 * 1024 * 1024 or not line.endswith(b"\n"):
                raise ValueError("S06 export record exceeds preflight bound")
            record = strict_json_loads(line)
            validate_export_record(record)
            records.append(record)
    wanted = physical + libraries
    actual = [record.get("path") for record in records]
    if actual != wanted or len(set(actual)) != len(actual):
        raise ValueError("S06 export has missing, extra, reordered or duplicate primary rows")
    return records


def string_map(value, context):
    if type(value) is not dict:
        raise ValueError(f"{context}: expected string map")
    for key, item in value.items():
        text(key, context)
        text(item, context)


def array(value, context):
    if type(value) is not list:
        raise ValueError(f"{context}: expected array")


def boolean(value, context):
    if type(value) is not bool:
        raise ValueError(f"{context}: expected boolean")


def digest(value, context):
    if len(hex_bytes(value, context)) != 32:
        raise ValueError(f"{context}: expected sha256")


def diagnostics(values):
    array(values, "diagnostics")
    for value in values:
        exact_keys(value, {"code", "start", "end"}, "diagnostic")
        for key in value:
            integer(value[key], -(2**63), 2**63-1, "diagnostic integer")


def validate_export_record(record):
    if type(record) is not dict or record.get("kind") not in ("case", "library"):
        raise ValueError("invalid export record kind")
    if record["kind"] == "library":
        exact_keys(record, {"kind", "path", "filename", "text_hex"}, "library")
        text(record["filename"], "library filename", empty=False)
        hex_bytes(record["text_hex"], "library text")
        text(record["path"], "library path", empty=False)
        return
    exact_keys(record, {"kind", "path", "raw_sha256", "loaded_sha256", "loaded_bytes", "units",
                        "symlinks", "current_directory", "global_options", "raw_settings",
                        "configurations", "variants", "config_diagnostics", "legacy_projection"}, "case")
    text(record["path"], "case path", empty=False)
    text(record["current_directory"], "current directory")
    for key in ("raw_sha256", "loaded_sha256"):
        digest(record[key], key)
    integer(record["loaded_bytes"], 0, 2**31-1, "loaded_bytes")
    for key in ("symlinks", "global_options", "raw_settings"):
        string_map(record[key], key)
    diagnostics(record["config_diagnostics"])
    array(record["units"], "units")
    for unit in record["units"]:
        exact_keys(unit, {"name", "text_hex", "script_kind", "file_options"}, "unit")
        text(unit["name"], "unit name")
        hex_bytes(unit["text_hex"], "unit text")
        integer(unit["script_kind"], -(2**31), 2**31-1, "unit kind")
        string_map(unit["file_options"], "file options")
    array(record["configurations"], "configurations")
    for config in record["configurations"]:
        string_map(config, "configuration")
    configs = [canonical(config) for config in record["configurations"]]
    if configs != sorted(set(configs)):
        raise ValueError("configurations are duplicated or not canonical")
    array(record["variants"], "variants")
    for variant in record["variants"]:
        exact_keys(variant, {"configuration", "current_directory", "case_sensitive", "parse_settings",
                             "other_settings", "inputs", "reads", "option_diagnostics"}, "variant")
        ci = integer(variant["configuration"], 0, len(configs)-1, "configuration index")
        text(variant["current_directory"], "variant directory", empty=False)
        boolean(variant["case_sensitive"], "case sensitivity")
        for key in ("parse_settings", "other_settings"):
            string_map(variant[key], key)
        if variant["parse_settings"].keys() & variant["other_settings"].keys() or (variant["parse_settings"] | variant["other_settings"]) != record["configurations"][ci]:
            raise ValueError("option projection lost or changed a configuration setting")
        diagnostics(variant["option_diagnostics"])
        array(variant["inputs"], "parser inputs")
        for item in variant["inputs"]:
            exact_keys(item, {"unit", "filename", "path", "script_kind", "route", "text_hex", "jsx", "force", "metadata"}, "parser input")
            ui = integer(item["unit"], 0, len(record["units"])-1, "unit ordinal")
            if type(item["script_kind"]) is not int or item["script_kind"] != record["units"][ui]["script_kind"]:
                raise ValueError("final parser kind differs from extracted eligibility")
            if item["route"] not in ("virtual_file", "initial_config_direct"):
                raise ValueError("unknown unit loading route")
            parser_request("validation", None, hex_bytes(item["text_hex"], "parser source"),
                           item["filename"], item["path"], item["script_kind"], item["jsx"], item["force"])
            exact_keys(item["metadata"], {"PackageJsonType", "PackageJsonDirectory", "ImpliedNodeFormat"}, "source metadata")
            text(item["metadata"]["PackageJsonType"], "package type")
            text(item["metadata"]["PackageJsonDirectory"], "package directory")
            integer(item["metadata"]["ImpliedNodeFormat"], -(2**31), 2**31-1, "implied format")
        array(variant["reads"], "metadata reads")
        for read in variant["reads"]:
            exact_keys(read, {"path", "exists", "text_sha256"}, "metadata read")
            text(read["path"], "metadata path", empty=False)
            boolean(read["exists"], "metadata exists")
            digest(read["text_sha256"], "metadata hash")
    legacy = record["legacy_projection"]
    if legacy is not None:
        exact_keys(legacy, {"policy", "original_settings", "rejected_helper", "rejected_message"}, "legacy projection")
        string_map(legacy["original_settings"], "legacy original")
        if record["path"] not in LEGACY_PATHS or legacy["original_settings"] != record["raw_settings"] or legacy["policy"] != "legacy-module-none-parser-input-v1" or record["raw_settings"].get("module") != "none":
            raise ValueError("legacy projection differs from reviewed policy")
        for variant in record["variants"]:
            if [d["code"] for d in variant["option_diagnostics"]] != [6046]:
                raise ValueError("legacy projection omitted its rejected-option diagnostic")
    elif record["path"] in LEGACY_PATHS:
        raise ValueError("reviewed legacy case lost its explicit disposition")


def unknown_reason(physical, unit):
    if "compiler/contentMapper" in physical:
        return "mapper_native_asset: external transformation is outside the direct parser corpus"
    return "ancillary_asset: upstream GetScriptKindFromFileName returns Unknown; retained without coercion"


def freeze_records(records, upstream, pin):
    corpus, recipes, requests = [], [], []
    counts = Counter()
    kinds = Counter()
    routes = Counter()
    legacy_paths = set()
    for original in records:
        validate_export_record(original)
        record = copy.deepcopy(original)
        path = record["path"]
        primary = primary_id(path)
        raw = (upstream / path).read_bytes()
        if record["kind"] == "library":
            exact_keys(record, {"kind", "path", "filename", "text_hex"}, "library export")
            source = hex_bytes(record.pop("text_hex"), "library text")
            if source != raw or record["filename"] != "bundled:///libs/" + path.removeprefix(LIB_ROOT):
                raise ValueError("bundled source/path differs from pinned Git bytes")
            request = parser_request(primary + "/unit/0/config/0", primary, source, record["filename"], record["filename"])
            record.update(id=primary, source_sha256=sha256(source), source_bytes=len(source),
                          route="bundled_embedded", script_kind=3, jsx=False, force=False)
            counts["libraries"] += 1
            counts["library_bytes"] += len(source)
            requests.append(request)
            recipes.append(request_recipe(request, 0, 0))
        elif record["kind"] == "case":
            if record["raw_sha256"] != sha256(raw):
                raise ValueError(f"raw case hash changed: {path}")
            counts["physical_cases"] += 1
            if record["legacy_projection"] is not None:
                legacy_paths.add(path)
                counts["legacy_variants"] += len(record["variants"])
                counts["legacy_requests"] += sum(len(v["inputs"]) for v in record["variants"])
            counts["multiunit_cases"] += len(record["units"]) > 1
            counts["symlink_cases"] += bool(record["symlinks"])
            counts["explicit_cwd_cases"] += bool(record["current_directory"])
            record["id"] = primary
            for unit in record["units"]:
                source = hex_bytes(unit.pop("text_hex"), "extracted unit")
                unit.update(extracted_sha256=sha256(source), extracted_bytes=len(source),
                            eligibility="direct_parser" if unit["script_kind"] else unknown_reason(path, unit))
                counts["extracted_units"] += 1
                counts["extracted_bytes"] += len(source)
                kinds[str(unit["script_kind"])] += 1
            if not record["configurations"] or len(record["variants"]) != len(record["configurations"]):
                raise ValueError("empty or incomplete configuration expansion")
            for ci, variant in enumerate(record["variants"]):
                if variant["configuration"] != ci or not variant["inputs"]:
                    raise ValueError("missing configuration or empty parser obligation")
                eligible = [i for i, unit in enumerate(record["units"]) if unit["script_kind"]]
                if [item["unit"] for item in variant["inputs"]] != eligible:
                    raise ValueError("eligible virtual unit omitted, reordered or duplicated")
                for item in variant["inputs"]:
                    source = hex_bytes(item.pop("text_hex"), "final parser text")
                    ui = integer(item["unit"], 0, len(record["units"])-1, "unit")
                    request = parser_request(f"{primary}/unit/{ui}/config/{ci}", primary, source,
                                             item["filename"], item["path"], item["script_kind"], item["jsx"], item["force"])
                    item.update(source_sha256=sha256(source), source_bytes=len(source), request_sha256=sha256(canonical(request)))
                    routes[item["route"]] += 1
                    requests.append(request)
                    recipes.append(request_recipe(request, ui, ci))
            counts["option_variants"] += len(record["variants"])
        else:
            raise ValueError("unknown primary export kind")
        corpus.append(record)
    counts["primary_rows"] = len(corpus)
    actual = {key: counts[key] for key in EXPECTED if key != "script_kinds"}
    actual["script_kinds"] = dict(sorted(kinds.items()))
    if actual != EXPECTED:
        raise ValueError(f"S06 independent planning totals differ: expected {EXPECTED}; got {actual}")
    if legacy_paths != LEGACY_PATHS or counts["legacy_variants"] != 13 or counts["legacy_requests"] != 23:
        raise ValueError("reviewed seven-case legacy projection coverage changed")
    if len(set(request["id"] for request in requests)) != len(requests):
        raise ValueError("duplicate request ID")
    probes = {"version": 1, "pin": pin, "planning_totals": actual,
              "primary_requests": len(requests), "option_variants": counts["option_variants"],
              "routes": dict(sorted(routes.items())), "request_sha256": sha256(canonical(requests)),
              "max_request_bytes": max(len(canonical(request)) for request in requests),
              "legacy_projection": {"policy": "legacy-module-none-parser-input-v1", "cases": 7, "option_variants": 13, "parser_requests": 23, "native_diagnostic": 6046},
              "option_projection": {"direct_fields": ["target", "module", "moduledetection", "moduleresolution", "jsx", "usecasesensitivefilenames"],
                                    "other_settings": "retained verbatim; currentdirectory supplies the virtual cwd, filename/directives supply units; other compiler/emit/checker settings do not enter the direct parser API",
                                    "base_options": "pinned initial tsconfig parsing, without Program/project-reference loading or mapper execution"},
              "scope": "direct parser inputs; no compiler skips, content mapper execution, project loading or Rust option-validation claim"}
    return {"corpus.json": {"version": 1, "pin": pin, "cases": corpus},
            "cases.json": [record["id"] for record in corpus],
            "requests.json": {"version": 1, "pin": pin, "requests": recipes},
            "probes.json": probes}, requests


def request_recipe(request, unit, config):
    recipe = {key: value for key, value in request.items() if key != "source_hex"}
    recipe.update(unit=unit, configuration=config, source_sha256=sha256(bytes.fromhex(request["source_hex"])),
                  source_bytes=len(request["source_hex"])//2, request_sha256=sha256(canonical(request)))
    return recipe


def manifest_changes(directory, documents, *, write=False):
    """Compare every owned manifest before publishing any reviewed replacements."""
    directory = Path(directory)
    differences = []
    encoded = {name: manifest_bytes(name, value) for name, value in documents.items()}
    for name, content in encoded.items():
        path = directory / name
        if not path.exists() or path.read_bytes() != content:
            differences.append(name)
    if differences and not write:
        raise ValueError("S06 frozen manifest drift: " + ", ".join(differences) + "; review before --write-manifest")
    if write:
        directory.mkdir(parents=True, exist_ok=True)
        for name, content in encoded.items():
            (directory / name).write_bytes(content)
    return differences


def manifest_bytes(name, value):
    """One complete case/request per line keeps the explicit corpus diff bounded."""
    field = {"corpus.json": "cases", "requests.json": "requests"}.get(name)
    if field:
        header = {key: item for key, item in value.items() if key != field}
        prefix = canonical(header)[:-1] + b',"' + field.encode() + b'":[\n'
        return prefix + b",\n".join(b"  " + canonical(item) for item in value[field]) + b"\n]}\n"
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=True).encode()+b"\n"
