"""Reconcile one sealed backing diagnostic; never launch a workload."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools/s07/performance-experiments'))
import runner
from s04_common import strict_json_loads


def summarize(observation):
    d, report = observation['diagnostic'], observation['report']
    r = dict(zip(d['retained_fields'], d['retained'], strict=True))
    t = dict(zip(d['traffic_fields'], d['traffic'], strict=True))
    range_size, compact_size, full_size = d['element_bytes']
    assert r['owners'] == report['files'] == d['bound_in_place_files'] and d['fallback_files'] == 0
    assert r['name_ranges_len'] == r['name_hash_entries']
    assert r['name_ranges_logical_bytes'] == r['name_ranges_len'] * range_size
    assert r['name_ranges_allocation_bytes'] == r['name_ranges_capacity'] * range_size
    assert r['table_arena_logical_rows'] == r['compact_tables'] + r['full_tables']
    assert sum(r[key] for key in ('tables_0', 'tables_1', 'tables_2', 'tables_3_to_4', 'tables_5_to_8', 'tables_9_to_16', 'tables_17_plus')) == r['table_arena_logical_rows']
    names = []
    for family, retained, logical in (
        ('name_bytes', r['name_bytes_capacity'], r['name_bytes_len']),
        ('name_ranges', r['name_ranges_allocation_bytes'], r['name_ranges_logical_bytes']),
        ('name_hash', r['name_hash_allocation_bytes'], r['name_hash_entries'] * 8),
        ('compact_hash', r['compact_allocation_bytes'], r['compact_entries'] * compact_size),
        ('full_hash', r['full_allocation_bytes'], r['full_entries'] * full_size),
    ):
        requests, replaced = t[family + '_requests'], t[family + '_replaced']
        conversion = t['compact_bytes_released_on_conversion'] if family == 'compact_hash' else 0
        other_release = requests - replaced - conversion - retained
        assert retained >= logical and requests >= retained and other_release >= 0
        names.append({'family': family, 'requested_bytes': requests, 'replaced_bytes': replaced,
                      'release_bytes': conversion, 'unclassified_release_bytes': other_release,
                      'retained_backing_bytes': retained, 'logical_element_bytes': logical,
                      'allocation_calls': t[family + '_allocations']})

    families, phases = [], []
    field_count, family_count = len(d['backing_fields']), len(d['backing_families'])
    assert field_count == 4 and family_count == 19
    for phase_index, phase in enumerate(d['backing_phases']):
        rows = []
        for family_index, family in enumerate(d['backing_families']):
            at = (phase_index * family_count + family_index) * field_count
            row = dict(zip(d['backing_fields'], d['backing_traffic'][at:at + field_count], strict=True))
            row['family'] = family
            row['net_live_change'] = row['requested_bytes'] - row['replaced_bytes'] - row['release_bytes']
            rows.append(row)
        totals = {field: sum(row[field] for row in rows) for field in (*d['backing_fields'], 'net_live_change')}
        phases.append({'phase': phase, 'families': rows, 'totals': totals})
    for family in d['backing_families']:
        rows = [next(row for row in phase['families'] if row['family'] == family) for phase in phases]
        row = {field: sum(part[field] for part in rows) for field in d['backing_fields']}
        row['family'] = family
        row['retained_backing_bytes'] = row['requested_bytes'] - row['replaced_bytes'] - row['release_bytes']
        assert row['retained_backing_bytes'] >= 0, family
        families.append(row)
    eager = next(row for row in families if row['family'] == 'eager_parser_list_buffers')
    assert eager['retained_backing_bytes'] == 0, 'all eager parse list buffers are consumed or dropped before endpoint'
    all_rows = [*families, *names]
    totals = {field: sum(row[field] for row in all_rows) for field in (
        'requested_bytes', 'replaced_bytes', 'release_bytes', 'retained_backing_bytes', 'allocation_calls')}
    totals['unclassified_release_bytes'] = sum(row['unclassified_release_bytes'] for row in names)
    assert totals['requested_bytes'] == (totals['retained_backing_bytes'] + totals['replaced_bytes'] +
                                         totals['release_bytes'] + totals['unclassified_release_bytes'])
    m = d['memory']
    requested = m['total_endpoint'] - m['total_before']
    live = m['live_endpoint'] - m['live_before']
    assert requested == report['allocated_bytes'] and requested >= live
    residual = {'requested_bytes': requested - totals['requested_bytes'],
                'net_live_change': live - totals['retained_backing_bytes']}
    residual['freed_or_superseded'] = residual['requested_bytes'] - residual['net_live_change']
    assert residual['requested_bytes'] >= 0 and residual['freed_or_superseded'] >= 0
    process_phases = []
    for index, name in enumerate(('parse', 'bind_publish')):
        p = d['phase_process_windows'][index * 4:(index + 1) * 4]
        phase_live = p[1] - p[2]
        process_phases.append({'phase': name, 'requested_bytes': p[0], 'net_live_change': phase_live,
                               'freed_or_superseded': p[0] - phase_live, 'files': p[3]})
    unscoped = {'requested_bytes': requested - sum(p['requested_bytes'] for p in process_phases),
                'net_live_change': live - sum(p['net_live_change'] for p in process_phases)}
    unscoped['freed_or_superseded'] = unscoped['requested_bytes'] - unscoped['net_live_change']
    assert unscoped['requested_bytes'] >= 0
    directory_rows = [row for row in families if row['family'].endswith('_directories')]
    directory_totals = {field: sum(row[field] for row in directory_rows) for field in (
        'requested_bytes', 'replaced_bytes', 'release_bytes', 'retained_backing_bytes', 'allocation_calls')}
    return {'diagnostic_only': True,
            'work': {key: report[key] for key in ('files', 'loaded_bytes', 'nodes', 'symbols', 'parse_diagnostics', 'bind_diagnostics', 'loaded_input_sha256')},
            'backing_families': families, 'name_table_families': names, 'observed_totals': totals,
            'pipeline': {'requested_bytes': requested, 'net_live_change': live, 'freed_or_superseded': requested - live},
            'residual_outside_observed_families': residual, 'family_events_by_phase': phases,
            'process_phase_windows': process_phases, 'process_outside_phase_windows': unscoped,
            'directory_totals': directory_totals,
            'traffic_ranking': sorted(all_rows, key=lambda row: row['replaced_bytes'] + row['release_bytes'] + row.get('unclassified_release_bytes', 0), reverse=True),
            'limits': [
                'Retained backing from events is requested minus superseded and released requested layouts; allocator resident bytes and RSS are not measured.',
                'Phase family events label when allocation/release happened; a parse allocation can remain owned during binding or be released there.',
                'Process phase windows can include simultaneous main-thread allocation/deallocation, explicitly distinguished from thread-local family event phases.',
                'Edge pages combine syntax and declaration edges; other arenas include declaration headers and table records. Name-table bucket backing is disjoint from its record arena.',
                'Eager parser list buffers exclude caller-owned append suffixes and default/lazy Vec-to-Box transfers.',
                'Text entry/free-slot vectors exclude nested JsString bytes, Arc storage and exceptional maps.',
                'No cap backtrace attribution assigns the residual to a guessed family. Name/table events have no per-phase tags.',
                'Replaced capacity is a conditional avoidable-traffic ceiling, not a saving demonstrated by a replacement representation or a CPU/RSS gate result.'
            ]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('capture_manifest_sha256')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    source = ROOT / 'target/s07-bis/allocation-traffic-capture'
    assert runner.digest(source / 'manifest.json') == args.capture_manifest_sha256
    manifest = strict_json_loads((source / 'manifest.json').read_bytes())
    assert manifest['kind'] == 'allocation-traffic-capture' and manifest['invocations'] == 1
    assert runner.inventory(source) == manifest['inventory']
    result = summarize(strict_json_loads((source / 'observations.json').read_bytes()))
    result['provenance'] = {'capture_manifest_sha256': args.capture_manifest_sha256,
                            'build_manifest_sha256': manifest['build_manifest_sha256'],
                            'observations_sha256': runner.digest(source / 'observations.json'),
                            'summarizer_sha256': runner.digest(__file__)}
    runner.write_json(args.output, result)
    print(json.dumps({key: result[key] for key in ('pipeline', 'observed_totals', 'residual_outside_observed_families', 'process_phase_windows', 'process_outside_phase_windows', 'directory_totals')}, indent=2))

if __name__ == '__main__':
    main()
