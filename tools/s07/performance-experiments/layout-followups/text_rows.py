#!/usr/bin/env python3
"""Price four-byte identifier words in the unchanged physical page census."""
import copy
import gzip
import json
from pathlib import Path

import mixed_rows as mixed


def report():
    rows, original, recorded, inputs = mixed.inputs()
    mixed_path = mixed.HERE / "mixed-rows-result.json.gz"
    control = mixed.previous.strict_json_loads(gzip.decompress(mixed_path.read_bytes()))
    if mixed.sha((mixed.HERE / "mixed_rows.rs").read_bytes()) != control["provenance"]["compilation"]["source_sha256"]:
        raise ValueError("compiled scalar layouts changed")
    if inputs != control["provenance"]["historical_inputs_sha256"]:
        raise ValueError("mixed-row inputs differ")
    model = copy.deepcopy(original)
    counts = {}
    for name in ("Identifier", "PrivateIdentifier"):
        shape = model["shape_inventory"][name]
        if shape["fields"] != [["text", "JsString"]] or shape["facts"] or shape["syntax_layout"] != {"size": 8, "alignment": 4}:
            raise ValueError("identifier field contract changed")
        shape["syntax_layout"]["size"] -= 4
        shape["bound_layout"]["size"] -= 4
        counts[name] = sum(row["core_shapes"].get(name, 0) for row in rows)
    layouts = control["layouts"]
    sizes = recorded["directory_layouts"]
    header = mixed.previous.lean_header_costs(rows, 32, 32, sizes, "Thin")
    variants = []
    for label, use_mixed in (("all_atomic", False), ("mixed", True)):
        grouped, classes, counts_by_class = mixed.group_shapes(rows, model["shape_inventory"], layouts, use_mixed)
        for page_size in (8, 16):
            costs, active = mixed.previous.page_costs(grouped, classes, page_size, page_size, sizes, sizes["ThinPage"]["size"])
            directory = mixed.previous.lean_directory_costs("optional_one_or_many", costs, active, sizes, "Thin", "owner", len(classes))
            variants.append({"family": label, "page_rows": page_size, "header_rows": 32,
                "class_counts": counts_by_class,
                "live_bytes": header["capacity"] + header["directory_live"] + costs["capacity_bound"] + directory["live"],
                "request_bytes": header["capacity"] + header["directory_requests"] + costs["capacity_bound"] + directory["requests"],
                "allocation_calls": header["pages"] + header["directory_allocations"] + costs["payload_pages"] + directory["allocations_excluding_embedded_root"]})
    # Keep the historical leading ordinary typed-page policy in this comparison.
    costs, active = mixed.previous.page_costs(rows, model["shape_inventory"], 4, 4, sizes, sizes["ThinPage"]["size"])
    directory = mixed.previous.lean_directory_costs("optional_one_or_many", costs, active, sizes, "Thin", "owner", len(model["shape_inventory"]))
    typed = {"family": "typed", "page_rows": 4, "header_rows": 32,
        "live_bytes": header["capacity"] + header["directory_live"] + costs["capacity_bound"] + directory["live"],
        "request_bytes": header["capacity"] + header["directory_requests"] + costs["capacity_bound"] + directory["requests"],
        "allocation_calls": header["pages"] + header["directory_allocations"] + costs["payload_pages"] + directory["allocations_excluding_embedded_root"]}
    variants.append(typed)
    used_saving = sum(counts.values()) * 4
    expected_used = control["families"]["mixed_rows"]["used_bound_payload_bytes"] - used_saving
    if any(value["class_counts"]["used_bound_payload_bytes"] != expected_used for value in variants if "class_counts" in value):
        raise ValueError("text-word saving lost or double counted")
    return {"version": 1, "diagnostic_only": True, "cpu": "unmeasured", "physical_identifiers": counts,
        "used_payload_saving_bytes": used_saving, "used_bound_payload_bytes": expected_used, "variants": variants,
        "provenance": {"script_sha256": mixed.sha(Path(__file__).read_bytes()), "historical_inputs": inputs,
            "mixed_rows_model_sha256": mixed.sha(mixed_path.read_bytes()), "compiled_layout_provenance": control["provenance"]["compilation"]},
        "limitations": ["Four-byte identifier words are a conditional representation candidate; factory staging, text/escape pools and owner metadata are not priced here.",
            "The total includes modeled headers, bound payload capacity and directories only; runtime identities, full-range links and remaining owner storage are excluded.",
            "Page policies are fixed existing controls, not a new exhaustive optimum search; no CPU, RSS or native allocation measurement is made."]}


if __name__ == "__main__":
    value = report()
    destination = mixed.HERE / "text-rows-result.json.gz"
    destination.write_bytes(gzip.compress((json.dumps(value, indent=2, sort_keys=True) + "\n").encode(), mtime=0))
    print(json.dumps({"used_payload_saving_bytes": value["used_payload_saving_bytes"], "variants": [
        {k: v for k, v in row.items() if k != "class_counts"} for row in value["variants"]]}, indent=2))
