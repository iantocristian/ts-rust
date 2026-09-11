"""Adversarial checks for each frozen P0 output class and a native typed-tool fixture."""
import copy
import json
import lzma
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from s04_common import strict_json_loads
from s08_contracts import compare, diagnostics, ordering, state, validate
from s08_contract_oracle import requests
from s08_inventory import compact, sites
from s08_measurement_contract import measured_ratios, type_mean_ratio
from s08_oracle import ROOT
from s08_queries import action, expected_baseline


def load(name):
    path = ROOT / name
    raw = path.read_bytes()
    return strict_json_loads(lzma.decompress(raw) if path.suffix == '.xz' else raw)


class P0ProtocolTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.spec = load('tools/s08/contracts/relations.json')
        cls.request = requests(cls.spec)
        cls.observed = load('data/s08/supplemental-observations.json.xz')
        cls.text = load('tools/s08/contracts/text.json')
        cls.residuals = load('tools/s08/contracts/residuals.json')

    def validate(self, observed):
        validate(self.spec, self.request, self.text, self.residuals, observed)

    def test_observed_protocol_is_complete(self):
        self.validate(self.observed)

    def test_missing_mode_reordered_requests_and_fake_execution_fail(self):
        for mutation in (
            lambda x: x['rows'].pop(),
            lambda x: x['rows'].reverse(),
            lambda x: x['rows'][0]['groups'].pop(),
            lambda x: x['rows'][0].update(state='not_implemented'),
            lambda x: x['rows'][0].update(panic='bounds'),
            lambda x: x['rows'][0]['groups'][0]['actions'].reverse(),
            lambda x: x['rows'][0]['groups'][0]['actions'][0].update(result=1),
        ):
            altered = copy.deepcopy(self.observed)
            mutation(altered)
            with self.assertRaises(ValueError):
                self.validate(altered)

    def test_cache_counts_and_cold_repeat_state_cannot_be_forged(self):
        row = self.observed['rows'][6]['groups'][1]
        # A real generic fixture allocates during the cold relation and reuses that state.
        self.assertGreater(row['actions'][0]['after']['types_created'], row['actions'][0]['before']['types_created'])
        self.assertEqual(row['actions'][1]['before'], row['actions'][0]['after'])
        altered = copy.deepcopy(row['actions'][0]['after'])
        altered['caches']['assignable']['entries'] += 1
        with self.assertRaises(ValueError):
            state(altered)
        altered = copy.deepcopy(self.observed)
        altered['rows'][6]['groups'][1]['actions'][1]['before']['types_created'] += 1
        with self.assertRaises(ValueError):
            self.validate(altered)

    def test_complete_diagnostic_payload_and_panic_reason_are_compared(self):
        altered = copy.deepcopy(self.observed)
        diagnostic = altered['rows'][1]['groups'][1]['actions'][0]['diagnostics'][0]
        self.assertTrue(diagnostic['chain'])
        diagnostic['chain'] = []
        with self.assertRaises(ValueError):
            compare(self.observed, altered)
        altered = copy.deepcopy(self.observed)
        altered['supplemental']['residuals']['foreign-checker']['message'] = 'debug overflow'
        with self.assertRaises(ValueError):
            self.validate(altered)
        with self.assertRaises(ValueError):
            diagnostics([dict(diagnostic, category=True)])

    def test_union_shuffle_and_pairwise_outcomes_are_not_sort_claims(self):
        original = self.observed['rows'][0]['groups'][0]['union_ordering']
        self.assertTrue(original)
        altered = copy.deepcopy(original)
        altered[1]['sorted'].pop()
        with self.assertRaises(ValueError):
            ordering(altered)
        altered = copy.deepcopy(original)
        altered[1]['sorted'].reverse()
        with self.assertRaises(ValueError):
            ordering(altered)
        with self.assertRaises(ValueError):
            ordering(original[:-1])

    def test_text_bytes_and_truncation_cannot_be_normalized_away(self):
        text = self.observed['supplemental']['text']
        truncated = next(r for r in text if r['id'] == 'byte-truncation')
        self.assertTrue(truncated['truncated'])
        self.assertGreater(len(truncated['type_full_hex']), len(truncated['type_short_hex']))
        altered = copy.deepcopy(self.observed)
        altered['supplemental']['text'][0]['original_hex'] = 'replacement'.encode().hex()
        with self.assertRaises(ValueError):
            compare(self.observed, altered)
        altered['supplemental']['text'][0]['original_hex'] = 'FF'
        with self.assertRaises(ValueError):
            self.validate(altered)

    def test_node_schedule_requires_coordinates_and_never_uses_unscoped_ids(self):
        query = dict(operation='GetTypeAtLocation', file='/file.ts', kind=80, pos=1, end=2, type_id=10)
        first = action(query, {})
        query['type_id'] = 999
        self.assertEqual(first, action(query, {}))
        for changed in (dict(query, pos=True), dict(query, end=0), dict(query, flags=-1), dict(query, absent=0),
                        {k: v for k, v in query.items() if k != 'pos'}, dict(query, unexpected=1)):
            with self.assertRaises(ValueError):
                action(changed, {})

    def test_baseline_missing_disabled_and_empty_content_stay_distinct(self):
        outcomes = [expected_baseline(row) for row in (
            dict(state='content', text_hex=''), dict(state='no_content'), dict(state='disabled'))]
        self.assertEqual(len({tuple(row) for row in outcomes}), 3)
        for bad in (dict(state='missing'), dict(state='not_implemented'), dict(state='no_content', text_hex='')):
            with self.assertRaises(ValueError):
                expected_baseline(bad)

    def test_request_unknown_fields_and_duplicate_ids_fail(self):
        altered = copy.deepcopy(self.spec)
        altered['cases'].append(altered['cases'][0])
        with self.assertRaises(ValueError):
            requests(altered)
        altered = copy.deepcopy(self.spec)
        altered['options']['strict'] = 1
        with self.assertRaises(ValueError):
            requests(altered)

    def test_measurement_missing_work_and_unusable_denominators_fail(self):
        sample = dict(state='completed', workload_sha256='work', executed_queries=10,
                      elapsed_seconds=2.0, requested_bytes=100, retained_bytes=80)
        samples = [dict(sample) for _ in range(7)]
        ratios = measured_ratios(samples, [dict(sample, elapsed_seconds=1.0) for _ in range(7)], 'work', 10)
        self.assertEqual(ratios['throughput'], .5)
        for key, value in [('elapsed_seconds', 0), ('elapsed_seconds', float('nan')), ('requested_bytes', -1),
                           ('retained_bytes', -1), ('executed_queries', 9), ('state', 'skipped')]:
            bad = copy.deepcopy(samples)
            bad[0][key] = value
            with self.assertRaises(ValueError):
                measured_ratios(bad, samples, 'work', 10)
        with self.assertRaises(ValueError):
            measured_ratios(samples, [dict(sample, retained_bytes=0) for _ in range(7)], 'work', 10)
        with self.assertRaises(ValueError):
            measured_ratios(samples[:-1], samples, 'work', 10)

    def test_census_cannot_omit_unknown_backing_or_inflate_empty_capacity(self):
        complete = dict(state='complete', charged_type_storage_bytes=160, logical_retained_types=2, unassigned_allocations=[])
        self.assertEqual(type_mean_ratio(complete, complete), 1)
        for bad in (dict(complete, logical_retained_types=0), dict(complete, unassigned_allocations=['map']),
                    dict(complete, state='unavailable'), dict(complete, logical_retained_types=True)):
            with self.assertRaises(ValueError):
                type_mean_ratio(bad, complete)

    def test_unknown_typed_boundary_requires_review(self):
        from s08_audit import boundary
        with self.assertRaises(ValueError):
            boundary(dict(file='internal/checker/new.go', signature='func() *NewType', value='newHost.NewType'))


@unittest.skipUnless(os.environ.get('S08_NATIVE_TESTS') == '1', 'opt-in pinned Go typed-tool integration')
class NativeInventoryTests(unittest.TestCase):
    def test_direct_generic_interface_and_external_callback_edges(self):
        from s04 import go_environment, verified_upstream
        with tempfile.TemporaryDirectory(prefix='s08-inventory-test-', dir=ROOT / 'target') as tmp:
            directory = Path(tmp)
            (directory / 'go.mod').write_text('module example.test/p0\n\ngo 1.26\n')
            (directory / 'main.go').write_text('''package fixture
import "slices"
type Count = int
type Host interface { Run(int) int }
type worker struct{}
func (worker) Run(n int) int { return n+1 }
var hook = callback
var state int
func init() { register(callback) }
func register(fn func(int) int) { state=fn(2) }
func callback(n int) int { return n*2 }
func invoke(fn func(int) int, n int) int { return fn(n) }
func generic[T any](v T, fn func(T) int) int { return fn(v) }
func throughHost(h Host) int { return h.Run(4) }
func less(a,b int) int { return a-b }
func Root() int { xs:=[]int{2,1};slices.SortFunc(xs,less); return invoke(hook,1)+generic(Count(2),callback)+throughHost(worker{})+xs[0] }
''')
            request = dict(prefix='example.test/', patterns=['.'], roots=['p0::Root'], interfaces=['p0::Host'], goos='linux', goarch='amd64')
            path = directory / 'request.json'
            path.write_text(json.dumps(request))
            cmd = ['go', 'run', '-mod=readonly', str(ROOT / 'tools/s08/inventory/main.go'), '--dir', str(directory), '--request', str(path)]
            env = go_environment()
            cwd = verified_upstream() / 'tsc'
            result = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, check=True, timeout=120)
            original = strict_json_loads(result.stdout)
            document = compact(original)
            self.assertNotIn('example.test/p0.Count', json.dumps(document))
            self.assertNotIn('func(n int)', json.dumps(document['tables']['signature']))
            selected = {r['selector'] for r in document['functions']}
            self.assertTrue({'p0::Root', 'p0::callback', 'p0::worker.Run', 'p0::less', 'p0::register'} <= selected)
            self.assertEqual([m['name'] for m in document['interfaces']['p0::Host']], ['Run'])
            decoded = list(sites(document))
            self.assertTrue(any(s['kind'] == 'interface' and any('worker).Run' in t for t in s['targets']) for s in decoded))
            self.assertTrue(any(s['kind'] == 'callback_argument' and any('.less@' in t for t in s['targets']) for s in decoded))
            self.assertTrue(any('generic[int]' in name for name in document['identifiers']))
            request['roots'] = ['p0::Missing']
            path.write_text(json.dumps(request))
            failure = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, timeout=120)
            self.assertNotEqual(failure.returncode, 0)
            self.assertIn(b'missing source root', failure.stderr)


if __name__ == '__main__':
    unittest.main()
