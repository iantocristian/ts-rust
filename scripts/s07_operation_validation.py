"""Fail-closed validation of the reviewed S07 source operation boundary."""
import collections
import csv
import json
import tomllib
from pathlib import Path

from s04_common import strict_json_loads
from s07_inventory import ROOT, sha
from s07_operations import PACKAGES, SPECS, TRAVERSE_FILES, create


def validate_document(document, *, regenerate=True):
    ledger = tomllib.loads((ROOT / 'PORTS.toml').read_text())
    if document.get('schema') != 1 or document.get('upstream_pin') != ledger['pin']:
        raise ValueError('operation matrix schema or source pin differs')
    expected_inputs = ('scripts/s07_operations.py', 'scripts/s07_inventory.py',
                       'data/go-functions.tsv', 'data/s04/toolchains.toml')
    if document.get('generator_inputs') != {p: sha((ROOT / p).read_bytes()) for p in expected_inputs}:
        raise ValueError('operation matrix generator inputs are stale or incomplete')
    if document.get('audited_packages') != list(PACKAGES):
        raise ValueError('operation matrix audited package domain differs')
    if document.get('traversal_file_limits') != {key: sorted(value) for key, value in TRAVERSE_FILES.items()}:
        raise ValueError('operation source traversal boundary differs')
    for key in ('dynamic_boundary_policy', 'construction_boundary_policy', 'selection'):
        if not isinstance(document.get(key), str) or not document[key]:
            raise ValueError('missing operation boundary policy: ' + key)
    if document.get('unresolved_calls') != []:
        raise ValueError('unresolved source operation calls prevent freeze')
    rows = csv.DictReader((line for line in (ROOT / 'data/go-functions.tsv').read_text().splitlines()
                           if not line.startswith('#')), delimiter='\t')
    functions = {row['id']: row for row in rows}
    kinds = {row['go']: row['kind'] for row in ledger['file']}
    source_files = document.get('source_files')
    if not isinstance(source_files, dict) or not source_files:
        raise ValueError('missing source operation file fingerprints')
    for path, expected in source_files.items():
        if path not in kinds or sha((ROOT / 'upstream' / path).read_bytes()) != expected:
            raise ValueError('stale or unknown operation source file: ' + path)
    actual_functions = document.get('functions')
    if not isinstance(actual_functions, dict) or not actual_functions:
        raise ValueError('missing source function definitions')
    for identifier, definition in actual_functions.items():
        source = functions.get(identifier)
        if source is None:
            raise ValueError('unknown source function: ' + identifier)
        expected = {'file': source['file'], 'start_line': int(source['start']),
                    'end_line': int(source['end']), 'kind': kinds[source['file']],
                    'source_sha256': sha((ROOT / 'upstream' / source['file']).read_bytes())}
        if any(definition.get(key) != value for key, value in expected.items()):
            raise ValueError('source function anchor or kind differs: ' + identifier)
        if definition.get('mapping_status') != ('source_marker_present' if definition.get('rust_mappings') else 'no_source_marker'):
            raise ValueError('inconsistent Rust mapping metadata: ' + identifier)
    operations = document.get('operations')
    if not isinstance(operations, list) or [o.get('id') for o in operations] != [s['id'] for s in SPECS]:
        raise ValueError('missing, duplicate, extra or reordered required operations')
    edges = collections.defaultdict(list)
    external = set()
    for family in ('transitive_calls', 'external_boundaries', 'dynamic_calls'):
        values = document.get(family)
        if not isinstance(values, list):
            raise ValueError('missing operation call family: ' + family)
        for call in values:
            caller = functions.get(call.get('caller'))
            if (caller is None or call.get('file') != caller['file']
                    or type(call.get('line')) is not int or type(call.get('end_line')) is not int
                    or not int(caller['start']) <= call['line'] <= call['end_line'] <= int(caller['end'])
                    or type(call.get('column')) is not int or call['column'] < 1
                    or type(call.get('end_column')) is not int or call['end_column'] < 1):
                raise ValueError('call is outside its source function')
            if family == 'transitive_calls':
                if call.get('dynamic') or call.get('callee') not in functions:
                    raise ValueError('unresolved static operation callee')
                edges[call['operation']].append(call)
            elif family == 'external_boundaries':
                if not call.get('boundary'):
                    raise ValueError('unclassified external operation boundary')
                if (not call.get('callee') and call.get('package', '').startswith('github.com/microsoft/TypeScript/tsc/')
                        and not call.get('interface')):
                    raise ValueError('project call cannot be classified as an external boundary')
                external.add((call.get('operation'), call['caller'], call.get('callee')))
            elif not call.get('dynamic'):
                raise ValueError('non-dynamic call in callback boundary inventory')
    owned = set()
    summaries = []
    for operation, spec in zip(operations, SPECS):
        identifier = operation['id']
        for field in ('supported_inputs', 'observations', 'unsupported_calls'):
            if operation.get(field) != spec[field]:
                raise ValueError('operation scope differs from source-selected specification: ' + identifier)
        roots = ['tsc/internal/' + value for value in spec['roots']]
        if operation.get('source_roots') != roots:
            raise ValueError('operation roots differ: ' + identifier)
        source = operation.get('required_source_functions')
        generated = operation.get('required_generated_functions')
        for values, kind in ((source, 'source'), (generated, 'generated')):
            if not isinstance(values, list) or values != sorted(set(values)):
                raise ValueError('operation function IDs must be unique and sorted')
            for value in values:
                if value not in actual_functions or actual_functions[value]['kind'] != kind:
                    raise ValueError('SOURCE/generated function denominator differs: ' + value)
        required = set(source + generated)
        exclusions = operation.get('excluded_source_functions')
        if not isinstance(exclusions, list) or any(not value.get('reason') for value in exclusions):
            raise ValueError('missing excluded source function reason')
        omitted = {value['id'] for value in exclusions}
        if len(omitted) != len(exclusions) or not omitted <= actual_functions.keys() or required & omitted:
            raise ValueError('invalid excluded function set')
        graph = collections.defaultdict(list)
        for call in edges[identifier]:
            if call['caller'] not in required:
                raise ValueError('call caller is outside its declared required operation')
            callee = call['callee']
            if callee not in required | omitted and (identifier, call['caller'], callee) not in external:
                raise ValueError('missing transitive callee or explicit boundary: ' + callee)
            if callee in required | omitted:
                graph[call['caller']].append(callee)
        reachable = set()
        pending = roots[:]
        while pending:
            value = pending.pop()
            if value in reachable:
                continue
            reachable.add(value)
            if value not in omitted:
                pending.extend(graph[value])
        if reachable != required | omitted:
            raise ValueError('declared source function set differs from rooted static closure: ' + identifier)
        for family in ('dynamic_calls', 'external_boundaries'):
            if any(c['caller'] not in required for c in document[family] if c.get('operation') == identifier):
                raise ValueError('boundary caller is outside required operation')
        owned |= required | omitted
        summaries.append({'id': identifier, 'required_source_functions': len(source),
                          'required_generated_functions': len(generated), 'excluded_functions': len(omitted)})
    if owned != actual_functions.keys():
        raise ValueError('orphaned operation function metadata')
    known_operations = {s['id'] for s in SPECS}
    if any(c.get('operation') not in known_operations for family in ('transitive_calls', 'external_boundaries', 'dynamic_calls')
           for c in document[family]):
        raise ValueError('call references an unknown operation')
    constructors = document.get('package_initializer_calls')
    if not isinstance(constructors, list):
        raise ValueError('missing constructor boundary inventory')
    for call in constructors:
        if call.get('caller') is not None or call.get('file') not in source_files:
            raise ValueError('invalid package constructor anchor')
        if (not call.get('dynamic') and call.get('callee') is None
                and call.get('package', '').startswith('github.com/microsoft/TypeScript/tsc/') and not call.get('interface')):
            raise ValueError('unresolved package constructor dependency')
    # Regeneration catches an omitted call/file/function even when a edited
    # document makes all its own counts and closure edges internally consistent.
    if regenerate and create() != document:
        raise ValueError('operation inventory differs from current pinned source and Rust mapping locations')
    return summaries


def validate_operation_matrix():
    path = ROOT / 'data/s07/operations.json'
    document = strict_json_loads(path.read_bytes())
    summaries = validate_document(document)
    return {'path': str(path.relative_to(ROOT)), 'sha256': sha(path.read_bytes()),
            'operations': summaries, 'dynamic_calls': len(document['dynamic_calls']),
            'package_initializer_calls': len(document['package_initializer_calls']),
            'scope': 'Source static call closure plus separately reviewed callback/interface/construction boundaries; not function parity'}
