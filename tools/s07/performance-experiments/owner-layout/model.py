#!/usr/bin/env python3
"""Compiled, untimed owner-layout costs; unknown representation costs stay unknown."""
from __future__ import annotations

import argparse
from collections import Counter
from functools import lru_cache
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE.parent))
from layout_model import archived_inputs, strict_json

CAPTURE_SHA = '5a2aeab597be444b5906b675bc7ce2d20c53605c9ab6b636bfe6d45d4d43d339'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def natural(value):
    require(type(value) is int and value >= 0, 'expected nonnegative integer')
    return value


def divide(value, denominator):
    require(value >= 0 and denominator > 0 and value % denominator == 0,
            'nonintegral or negative census deconvolution')
    return value // denominator


def compile_layouts(output, rustc='rustc'):
    output.mkdir(parents=True, exist_ok=True)
    source = HERE / 'layouts.rs'
    source_files = [source, HERE.parent/'storage-pilot/text.rs', HERE.parent/'storage-pilot/text-tests.rs']
    source_identity = {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in source_files}
    binary, tests = output / 'layouts', output / 'layout-tests'
    compiler = subprocess.check_output([rustc, '-vV']).decode()
    for target, extra in ((binary, []), (tests, ['--test'])):
        subprocess.run([rustc, '--edition=2021', '-Dwarnings', *extra,
                        str(source), '-o', str(target)], check=True)
    tested = subprocess.check_output([str(tests)], stderr=subprocess.STDOUT)
    (output / 'rust-tests.txt').write_bytes(tested)
    layouts = {}
    for line in subprocess.check_output([str(binary)]).decode().splitlines():
        name, size, alignment = line.split()
        require(name not in layouts, 'duplicate compiled layout')
        layouts[name] = {'size': int(size), 'alignment': int(alignment)}
    require(source_identity == {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in source_files},
            'compiled source changed during build/tests')
    return layouts, {'rustc': compiler, 'source_sha256': sha(source.read_bytes()),
                     'source_inputs': source_identity,
                     'binary_sha256': sha(binary.read_bytes()), 'tests': tested.decode()}


def legacy_counts(census, sizes):
    """Invert mixed census rows, preserving physical (including obsolete) boxes."""
    rows = census['rows']
    result = {}
    for name, stem, width in [('symbols', 'BindResult.symbols', 112),
                              ('flows', 'FlowNodes.0', 48),
                              ('flow_lists', 'FlowLists.0', 16)]:
        payload, directory = rows[stem + '.page_payload'], rows[stem + '.page_directory']
        count, capacity = payload['elements'], payload['capacity_elements']
        require(payload['used_payload_bytes'] == count * width and
                payload['capacity_payload_bytes'] == capacity * width,
                'legacy plain row width changed')
        result[name] = {'len': count, 'capacity': capacity,
                        'directory_capacity_bytes': directory['capacity_payload_bytes']}
    payload = rows['DeclarationLists.0.page_payload']
    directory = rows['DeclarationLists.0.page_directory']
    headers = payload['containers'] - directory['elements']
    require(payload['elements'] == 2 * headers, 'declaration container rule changed')
    cells = divide(payload['used_payload_bytes'] - headers * sizes['BoxBacking'], 8)
    capacity = divide(payload['capacity_payload_bytes'] - cells * 8, sizes['BoxBacking'])
    require(payload['capacity_elements'] == capacity + headers, 'declaration capacity identity changed')
    result['declarations'] = {'len': headers, 'capacity': capacity, 'physical_cells': cells,
        'directory_capacity_bytes': directory['capacity_payload_bytes']}
    payload, directory = rows['SymbolTables.0.page_payload'], rows['SymbolTables.0.page_directory']
    headers = payload['containers'] - directory['elements']
    entries = payload['elements'] - headers
    current_header, current_entry = sizes['CurrentTableHeader'], 40
    require(payload['used_payload_bytes'] == headers * current_header + entries * current_entry,
            'table used-byte identity changed')
    capacity = divide(payload['capacity_payload_bytes'] - payload['capacity_elements'] * current_entry,
                      current_header - current_entry)
    entry_capacity = payload['capacity_elements'] - capacity
    require(headers == payload['unreported_layout_containers'] and capacity >= headers and entry_capacity >= entries,
            'table capacity identity changed')
    result['tables'] = {'len': headers, 'capacity': capacity, 'entries': entries,
        'entry_public_capacity': entry_capacity,
        'directory_capacity_bytes': directory['capacity_payload_bytes'],
        'hash_controls_and_bucket_layout': 'unreported; public capacity is not bucket count'}
    result['known_binding_directory_bytes'] = sum(result[name]['directory_capacity_bytes']
        for name in ('symbols', 'flows', 'flow_lists', 'declarations', 'tables'))
    # Keep all original row terms visible instead of folding this into a guessed other allowance.
    result['auxiliary'] = {'len': rows['core.auxiliary.page_payload']['elements'],
        'capacity': rows['core.auxiliary.page_payload']['capacity_elements'],
        'directory_capacity_bytes': rows['core.auxiliary.page_directory']['capacity_payload_bytes'],
        'node_list_headers': rows['AstStorageData.List.records']['elements'],
        'node_backing_headers': rows['AstStorageData.Nodes.records']['elements'],
        'node_backing_cells': divide(rows['AstStorageData.Nodes.owned']['used_payload_bytes'], 8),
        'node_backing_allocations': rows['AstStorageData.Nodes.owned']['allocations']}
    return result


def legacy_sensitivity(counts, sizes):
    rows = []
    for symbol in ('SymbolText4', 'SymbolText8'):
        used = counts['symbols']['len'] * sizes[symbol] + counts['flows']['len'] * sizes['FlowInline']
        capacity = counts['symbols']['capacity'] * sizes[symbol] + counts['flows']['capacity'] * sizes['FlowInline']
        for name, layout in [('flow_lists', 'FlowList'), ('declarations', 'BoxBacking'), ('tables', 'ScalarTable')]:
            used += counts[name]['len'] * sizes[layout]
            capacity += counts[name]['capacity'] * sizes[layout]
        cells = counts['declarations']['physical_cells'] * sizes['Cell']
        used += cells + counts['tables']['entries'] * sizes['TableEntry']
        capacity += cells + counts['tables']['entry_public_capacity'] * sizes['TableEntry']
        rows.append({'symbol': symbol, 'flow': 'FlowInline', 'used_bytes_excluding_directories': used,
            'current_capacity_sensitivity_bytes': capacity + counts['known_binding_directory_bytes'],
            'table_buckets_status': 'not priced here; final public capacity substituted only for sensitivity',
            'allocation_traffic': None})
    return rows


@lru_cache(None)
def pages_for(count, first, maximum):
    natural(count)
    require(first > 0 and maximum >= first and not first & (first - 1) and not maximum & (maximum - 1),
            'invalid power-of-two page policy')
    capacity = pages = 0
    next_size = first
    while capacity < count and next_size < maximum:
        capacity += next_size
        pages += 1
        next_size *= 2
    if capacity < count:
        extra = (count - capacity + maximum - 1) // maximum
        capacity += extra * maximum
        pages += extra
    return capacity, pages


@lru_cache(None)
def directory_for(count, width):
    natural(count); natural(width)
    if not count:
        return 0, 0, 0
    capacity, requested, calls = 4, 4 * width, 1
    while capacity < count:
        capacity *= 2
        requested += capacity * width
        calls += 1
    return capacity * width, requested, calls


def empty_cost():
    return {'used_bytes': 0, 'capacity_bytes': 0, 'root_bytes': 0,
            'directory_live_bytes': 0, 'live_bytes': 0, 'requested_bytes': 0,
            'allocation_calls_excluding_embedded_roots': 0}


def add(*parts):
    return {key: sum(part[key] for part in parts) for key in empty_cost()}


def store(counts, width, policy, sizes):
    """Each logical arena retains a fixed compiled root, also when empty.

    Stable pages are never copied. The directory uses an explicit 4/doubling
    design policy, charging every full replacement request (not just net growth).
    Root bytes are part of the containing owner allocation; no separate call.
    """
    first, maximum = policy
    descriptor = sizes['ThinPage'] if first == maximum else sizes['FatPage']
    result = empty_cost()
    for count in counts:
        capacity, pages = pages_for(count, first, maximum)
        directory_live, directory_requests, directory_calls = directory_for(pages, descriptor)
        result['used_bytes'] += count * width
        result['capacity_bytes'] += capacity * width
        result['root_bytes'] += sizes['ThinStore']
        result['directory_live_bytes'] += directory_live
        result['requested_bytes'] += capacity * width + sizes['ThinStore'] + directory_requests
        result['allocation_calls_excluding_embedded_roots'] += pages + directory_calls
    result['live_bytes'] = result['capacity_bytes'] + result['root_bytes'] + result['directory_live_bytes']
    return result


def embedded(byte_count):
    value = empty_cost()
    value.update(root_bytes=byte_count, live_bytes=byte_count, requested_bytes=byte_count)
    return value


def exact_buffers(histograms, width):
    result = empty_cost()
    for histogram in histograms:
        for length, frequency in parse_histogram(histogram):
            result['used_bytes'] += length * frequency * width
            result['allocation_calls_excluding_embedded_roots'] += frequency if length else 0
    result['capacity_bytes'] = result['live_bytes'] = result['requested_bytes'] = result['used_bytes']
    return result


def parse_histogram(histogram, pair=False):
    require(type(histogram) is dict, 'histogram must be object')
    result = []
    for key, frequency in histogram.items():
        require(type(key) is str and type(frequency) is int and frequency > 0, 'invalid histogram entry')
        components = key.split('/')
        require(len(components) == (2 if pair else 1) and all(s.isdecimal() and str(int(s)) == s for s in components),
                'invalid histogram key')
        values = tuple(map(int, components))
        require(not pair or values[0] <= values[1], 'histogram length exceeds capacity')
        result.append((*values, frequency))
    return result


@lru_cache(None)
def table_plan(entries):
    """Proposed scalar map: 2 then doubling, floor(7*buckets/8) occupancy."""
    natural(entries)
    if not entries:
        return 0, 0, 0
    buckets, requested, calls = 2, 2, 1
    while entries > buckets * 7 // 8:
        buckets *= 2
        requested += buckets
        calls += 1
    return buckets, requested, calls


def table_buffers(rows, sizes, preserve_public_capacity):
    result, summary = empty_cost(), Counter()
    for row in rows:
        for length, capacity, frequency in parse_histogram(row['tables']['length_capacity_histogram'], True):
            target = capacity if preserve_public_capacity else length
            buckets, requested_buckets, calls = table_plan(target)
            result['used_bytes'] += frequency * length * sizes['TableEntry']
            result['capacity_bytes'] += frequency * buckets * sizes['TableBucket']
            result['requested_bytes'] += frequency * requested_buckets * sizes['TableBucket']
            result['allocation_calls_excluding_embedded_roots'] += frequency * calls
            summary['entries'] += frequency * length
            summary['old_public_capacity'] += frequency * capacity
            summary['new_buckets'] += frequency * buckets
            summary['tables'] += frequency
    result['live_bytes'] = result['capacity_bytes']
    return result, dict(summary)


def side_table(counts, bucket_width, root_width):
    """One fixed header per owner, then complete 7/8-load bucket growth."""
    result = empty_cost()
    for count in counts:
        buckets, requested, calls = table_plan(count)
        result['used_bytes'] += count * bucket_width
        result['capacity_bytes'] += buckets * bucket_width
        result['root_bytes'] += root_width
        result['requested_bytes'] += requested * bucket_width + root_width
        result['allocation_calls_excluding_embedded_roots'] += calls
    result['live_bytes'] = result['capacity_bytes'] + result['root_bytes']
    return result


def load_capture(capture, capture_sha, build, build_sha):
    """Verify immutable build/capture with their producer before reading records."""
    manifest_data = (capture / 'manifest.json').read_bytes()
    require(sha(manifest_data) == capture_sha, 'capture manifest hash changed')
    subprocess.run([sys.executable, str(HERE.parent / 'owner-census/probe.py'), 'replay',
                    '--build', str(build), '--build-sha', build_sha, '--output', str(capture)],
                   check=True, stdout=subprocess.DEVNULL)
    manifest = strict_json(manifest_data)
    raw = (capture / 'owners.ndjson').read_bytes()
    require(sha(raw) == manifest['raw_sha256'], 'owner records changed')
    rows = [strict_json(line) for line in raw.splitlines()]
    return rows, {'capture_manifest_sha256': capture_sha, 'build_manifest_sha256': build_sha,
        'raw_sha256': sha(raw), 'baseline_manifest_sha256': manifest['baseline_manifest_sha256'],
        'files': len(rows), 'kind': manifest['kind'], 'diagnostic_only': manifest['diagnostic_only']}


def load_archived_capture(directory):
    manifest_data = (directory/'manifest.json').read_bytes()
    envelope = strict_json(manifest_data)
    require(envelope['capture_manifest_sha256'] == CAPTURE_SHA, 'unexpected archived capture identity')
    subprocess.run([sys.executable, str(HERE.parent/'owner-census/replay.py'), str(directory)],
                   check=True, stdout=subprocess.DEVNULL)
    raw = gzip.decompress((directory/'owners.ndjson.gz').read_bytes())
    provenance = strict_json(gzip.decompress((directory/'provenance.json.gz').read_bytes()))
    capture = provenance['capture_manifest']
    require(sha(raw) == capture['raw_sha256'], 'archived owner census changed')
    rows = [strict_json(line) for line in raw.splitlines()]
    return rows, {'archive_manifest_sha256': sha(manifest_data),
        'capture_manifest_sha256': CAPTURE_SHA, 'build_manifest_sha256': capture['build_manifest_sha256'],
        'raw_sha256': sha(raw), 'baseline_manifest_sha256': capture['baseline_manifest_sha256'],
        'files': len(rows), 'kind': capture['kind'], 'diagnostic_only': True}


def reconcile_owner(rows, counts):
    """Assert which old aggregate quantities the new physical capture reproduces."""
    for name in ('symbols', 'flows', 'flow_lists', 'declarations', 'tables'):
        for field in ('len', 'capacity'):
            require(sum(row['arenas'][name][field] for row in rows) == counts[name][field],
                    f'new census differs from old physical {name}.{field}')
    for arena, histogram, expected in [
        ('declarations', 'physical_length_histogram', counts['declarations']['physical_cells']),
        ('node_backings', 'physical_length_histogram', counts['auxiliary']['node_backing_cells'])]:
        require(sum(length * frequency for row in rows for length, frequency in parse_histogram(row[arena][histogram])) == expected,
                f'physical cells differ: {arena}')
    table_count = sum(freq for row in rows for _, _, freq in parse_histogram(row['tables']['length_capacity_histogram'], True))
    require(table_count == counts['tables']['len'], 'table histogram differs from old count')
    require(sum(count for row in rows for count in row['flow_data'].values()) == counts['flows']['len'],
            'missing flow payload discrimination')


def binding_candidate(rows, policy, sizes, symbol, flow, table_capacity):
    fields = {'symbols': symbol, 'flows': flow, 'flow_lists': 'FlowList',
              'declarations': 'BoxBacking', 'tables': 'ScalarTable'}
    components = {name: store([r['arenas'][name]['len'] for r in rows], sizes[layout], policy, sizes)
                  for name, layout in fields.items()}
    if flow in ('FlowOutlined', 'FlowPacked'):
        for variant, layout in [('SwitchClause', 'SwitchClause'), ('ReduceLabel', 'ReduceLabel')]:
            require(all(variant in r['flow_data'] for r in rows), 'synthetic payload counts unavailable')
            components['synthetic_' + variant] = store([r['flow_data'][variant] for r in rows], sizes[layout], policy, sizes)
    if symbol.endswith('WithoutRuntimeId'):
        components['symbol_runtime_ids'] = side_table([r['symbol_runtime_ids_assigned'] for r in rows], sizes['RuntimeBucket'], sizes['RuntimeTable'])
    if flow == 'FlowPacked':
        # The capture records physical variant counts, not every stored link's
        # arena. Charge an empty escape table header, then report an additional
        # complete all-flows fallback bound below; zero escapes is NOT observed.
        components['flow_escape_table_roots'] = side_table([0] * len(rows), sizes['FlowEscapeBucket'], sizes['FlowEscapeTable'])
    components['declaration_cells'] = exact_buffers([r['declarations']['physical_length_histogram'] for r in rows], sizes['Cell'])
    components['table_buckets'], buckets = table_buffers(rows, sizes, table_capacity)
    components['owner_ledger'] = embedded(len(rows) * sizes['OwnerLedger'])
    result = {'symbol': symbol, 'flow': flow, 'page_policy': list(policy),
            'table_target': 'old_public_capacity' if table_capacity else 'endpoint_length',
            'components': components, 'totals': add(*components.values()), 'table_inventory': buckets}
    if flow == 'FlowPacked':
        full = side_table([r['arenas']['flows']['len'] for r in rows], sizes['FlowEscapeBucket'], sizes['FlowEscapeTable'])
        roots = components['flow_escape_table_roots']
        result['escape_status'] = 'unknown link-domain counts; zero-escape subtotal is conditional, not measured'
        result['additional_all_flows_escape_bound'] = {key: full[key] - roots[key] for key in full}
        result['synthetic_capacity_status'] = 'all observed synthetic rows remain charged even in the full escape bound; no assumed elision'
    return result


def auxiliary_candidate(rows, policy, sizes, indirect, census, routing):
    aux = lambda name: [r['aux_variants'].get(name, 0) for r in rows]
    components = {
        'node_list_headers': store(aux('List'), sizes['IndirectNodeList' if indirect else 'NodeList'], policy, sizes),
        'node_backing_headers': store(aux('Nodes'), sizes['BoxBacking'], policy, sizes),
        'node_backing_cells': exact_buffers([r['node_backings']['physical_length_histogram'] for r in rows], sizes['Cell']),
        # Keep non-list records at their current full enum width. No unpriced
        # SourceMetadata / BTree / diagnostic / JsString change is claimed.
        'other_original_records': store([sum(n for name, n in r['aux_variants'].items() if name not in ('List', 'Nodes')) for r in rows], 40, policy, sizes),
    }
    if routing == 'original_global_slots':
        components['auxiliary_locator'] = store([r['arenas']['core_aux']['len'] for r in rows], sizes['AuxiliaryLocator'], policy, sizes)
    else:
        require(routing == 'per_kind_arena_ids', 'unknown auxiliary routing mode')
    if indirect:
        components['distinct_slice_descriptors'] = store([r['node_lists']['distinct_descriptors'] for r in rows], sizes['NodeSlice'], policy, sizes)
    return {'node_list_layout': 'IndirectNodeList' if indirect else 'NodeList',
            'page_policy': list(policy), 'routing': routing,
            'routing_obligation': 'per-kind arena identities require checked dispatch among named stores, preserve backing/header aliasing and reject foreign/unpublished IDs; roots already charge arena_id' if routing == 'per_kind_arena_ids' else 'charged row+kind locator preserves the original mixed auxiliary slot domain',
            'components': components, 'totals': add(*components.values()),
            'retained_owned_children_bytes': sum(row['capacity_payload_bytes'] for name, row in census['rows'].items()
                 if name.endswith('.owned') and (name.startswith('SourceMetadataData.') or name in ('AstStorageData.Text.owned','AstStorageData.SourceFiles.owned'))),
            'retained_owned_children_status': 'added separately to owner totals; existing BTree private layout remains unreported',
            'slice_interning_traffic': 'unpriced' if indirect else 'not needed'}


def make_report(layouts, compiler, inputs, input_hashes, rows=None, provenance=None):
    sizes = {name: value['size'] for name, value in layouts.items()}
    census = inputs['native/rust-1-0-census.json']
    counts = legacy_counts(census, sizes)
    report = {'version': 1, 'kind': 's07_bis_owner_layout_diagnostic', 'diagnostic_only': True,
        'compiled_layouts': layouts, 'compiler': compiler, 'historical_input_sha256': input_hashes,
        'old_physical_counts': counts, 'current_capacity_sensitivities': legacy_sensitivity(counts, sizes),
        'owner_census': provenance, 'binding_candidates': [], 'auxiliary_candidates': [],
        'not_priced': [
            'Owner-local to full-range foreign/lazy identity escape representation, validation and retention.',
            'Symbol name text identity pool: canonical byte equality, malformed bytes, generated names, interner capacity, fallback and construction traffic.',
            'Symbol runtime IDs are inline in conservative candidates; sparse symbol/node side tables are separately priced but synchronization, race/ID semantics and getter CPU remain unproved.',
            'SourceFile, parser/binder temporaries, diagnostic storage, preserved binding maps until field migration, and private hash/BTree/allocator layout.',
            'Native endpoint residual: old 257480101 bytes after reported census and estimated Arc headers is not assumed eliminated.',
            'Declaration/parser temporary Vec growth before boxed backings; exact-box requests below price only final physical boxes.',
            'Table transient peak length/deletion history and semantic/CPU effects of scalar hashing; endpoint-length policy is conditional.',
            'Outlined/packed flow synthetic transitions can leave obsolete payload slots or require free-list traffic; endpoint variant counts alone do not measure this history.',
            'Drop/panic/counter-ledger proofs, spare page initialization work, insertion-time mapping, address stability consumers and CPU/RSS are unmeasured.'],
        'accounting': {'scope': 'replacement binding arenas and core auxiliary records, not complete owner budget',
            'requests': 'full modeled requests from zero for stable pages, directory growth, table growth and final physical backings; excludes explicitly unpriced builder traffic',
            'roots': 'fixed compiled store roots are charged once per file per arena; symbols/flow group includes one owner ledger, auxiliary group shares that ledger',
            'acceptance': 'No tracker metric or CPU/memory gate is emitted; row allowances are provisional allocations, not independent rejection gates.'}}
    if rows is not None:
        reconcile_owner(rows, counts)
        report['reconciled_with_old_census'] = True
        report['observed_owner_details'] = {
            'node_runtime_ids_assigned': sum(sum(r['node_runtime_ids_by_shape'].values()) for r in rows),
            'symbol_runtime_ids_assigned': sum(r['symbol_runtime_ids_assigned'] for r in rows),
            'flow_data': dict(sum((Counter(r['flow_data']) for r in rows), Counter())),
            'node_distinct_slice_descriptors': sum(r['node_lists']['distinct_descriptors'] for r in rows),
            'declaration_distinct_symbol_descriptors': sum(r['declarations']['distinct_symbol_descriptors'] for r in rows),
            'obsolete_declaration_backings': sum(r['declarations']['not_referenced_by_physical_symbols'] for r in rows)}
        report['node_runtime_side_table'] = side_table([sum(r['node_runtime_ids_by_shape'].values()) for r in rows], sizes['RuntimeBucket'], sizes['RuntimeTable'])
        report['node_runtime_side_table_status'] = 'candidate with checked core-local keys; add once to whole-owner model, not included in binding/auxiliary subtotals'
        report['text_prototype_owner_metadata'] = embedded(len(rows) * sizes['TextPrototypeOwner'])
        report['text_prototype_owner_metadata_status'] = 'compiled actual storage-pilot TextStore; includes inline Arc handle, two Vec headers and extended-map header even with no pool entries; heap source refcount header, source allocation, pools and canonical name interner are additional'
        policies = [(2, 256)] + [(n, n) for n in (8, 16, 32, 64, 128)]
        for policy in policies:
            for symbol in ('SymbolText4', 'SymbolText8', 'SymbolText4WithoutRuntimeId', 'SymbolText8WithoutRuntimeId'):
                for flow in ('FlowInline', 'FlowOutlined', 'FlowPacked'):
                    for table_capacity in (False, True):
                        report['binding_candidates'].append(binding_candidate(rows, policy, sizes, symbol, flow, table_capacity))
            for indirect in (False, True):
                for routing in ('original_global_slots', 'per_kind_arena_ids'):
                    report['auxiliary_candidates'].append(auxiliary_candidate(rows, policy, sizes, indirect, census, routing))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-output', type=Path, default=ROOT/'target/s07-bis/owner-layout')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rustc', default='rustc')
    parser.add_argument('--archive', type=Path, default=HERE.parent/'owner-census/results/2026-09-08')
    parser.add_argument('--capture', type=Path)
    parser.add_argument('--capture-sha')
    parser.add_argument('--census-build', type=Path)
    parser.add_argument('--census-build-sha')
    args = parser.parse_args()
    source_files = (HERE/'model.py', HERE/'layouts.rs', HERE.parent/'layout_model.py',
         HERE.parent/'storage-pilot/text.rs', HERE.parent/'storage-pilot/text-tests.rs',
         HERE.parent/'owner-census/replay.py', HERE.parent/'owner-census/validate.py',
         HERE.parent/'owner-census/receipt.py',
         ROOT/'scripts/s04_common.py')
    source_identity = {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in source_files}
    capture_args = [args.capture, args.capture_sha, args.census_build, args.census_build_sha]
    require(all(capture_args) or not any(capture_args), 'provide all four owner census provenance arguments')
    layouts, compiler = compile_layouts(args.build_output, args.rustc)
    inputs, hashes = archived_inputs()
    rows, provenance = (load_capture(*capture_args) if args.capture else load_archived_capture(args.archive))
    report = make_report(layouts, compiler, inputs, hashes, rows, provenance)
    require(source_identity == {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in source_files},
            'model source changed during generation')
    report['model_inputs'] = source_identity
    data = (json.dumps(report, indent=2, sort_keys=True) + '\n').encode()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(gzip.compress(data, mtime=0) if args.output.suffix == '.gz' else data)
    print(json.dumps({'output': str(args.output), 'sha256': sha(args.output.read_bytes()),
                      'compiled_contract_tests_passed': True, 'owner_rows': len(rows) if rows else None}))


if __name__ == '__main__':
    main()
