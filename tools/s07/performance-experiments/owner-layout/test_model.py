"""Counterexamples for diagnostic costs; no workload execution or timing."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import model


class ModelTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix='s07-owner-layout-test-')
        cls.layouts, _ = model.compile_layouts(Path(cls.temp.name))
        cls.sizes = {name: row['size'] for name, row in cls.layouts.items()}
        cls.inputs, _ = model.archived_inputs()
        cls.census = cls.inputs['native/rust-1-0-census.json']
        cls.counts = model.legacy_counts(cls.census, cls.sizes)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_compiled_full_fields_and_control_niches(self):
        self.assertEqual(self.sizes['DeclarationSlice'], 16)
        self.assertEqual(self.sizes['SymbolText4'], 56)
        self.assertEqual(self.sizes['SymbolText8'], 64)
        self.assertEqual(self.sizes['FlowInline'], 28)
        self.assertEqual(self.sizes['FlowOutlined'], 20)
        self.assertEqual(self.sizes['FlowPacked'], 16)
        self.assertEqual(self.sizes['TableBucket'], self.sizes['TableEntry'])
        self.assertEqual(self.sizes['ThinStore'], 32)
        self.assertEqual(self.sizes['TextPrototypeOwner'], 112)
        self.assertGreater(self.sizes['FlowEscapeBucket'], self.sizes['FlowPacked'])

    def test_declarations_do_not_confuse_box_count_with_cell_count(self):
        row = self.counts['declarations']
        self.assertEqual((row['len'], row['capacity'], row['physical_cells']),
                         (2_446_623, 3_140_204, 2_454_685))
        self.assertEqual(row['capacity'] * 16 + row['physical_cells'] * 4, 60_062_004)
        broken = copy.deepcopy(self.census)
        broken['rows']['DeclarationLists.0.page_payload']['capacity_elements'] += 1
        with self.assertRaisesRegex(ValueError, 'capacity identity'):
            model.legacy_counts(broken, self.sizes)
        broken = copy.deepcopy(self.census)
        broken['rows']['DeclarationLists.0.page_payload']['used_payload_bytes'] += 1
        with self.assertRaisesRegex(ValueError, 'deconvolution'):
            model.legacy_counts(broken, self.sizes)

    def test_table_headers_entries_and_capacity_are_deconvolved_separately(self):
        row = self.counts['tables']
        self.assertEqual((row['len'], row['capacity'], row['entries'], row['entry_public_capacity']),
                         (644_333, 893_462, 1_988_329, 3_191_336))
        self.assertEqual(self.counts['known_binding_directory_bytes'], 17_644_960)
        broken = copy.deepcopy(self.census)
        broken['rows']['SymbolTables.0.page_payload']['used_payload_bytes'] += 8
        with self.assertRaisesRegex(ValueError, 'table used-byte'):
            model.legacy_counts(broken, self.sizes)

    def test_table_policy_charges_tiny_buckets_and_full_rehash_requests(self):
        self.assertEqual(model.table_plan(0), (0, 0, 0))
        self.assertEqual(model.table_plan(1), (2, 2, 1))
        self.assertEqual(model.table_plan(2), (4, 6, 2))
        self.assertEqual(model.table_plan(7), (8, 14, 3))
        self.assertEqual(model.table_plan(8), (16, 30, 4))
        rows = [{'tables': {'length_capacity_histogram': {'1/3': 2, '0/0': 1}}}]
        final, _ = model.table_buffers(rows, self.sizes, False)
        reserve, _ = model.table_buffers(rows, self.sizes, True)
        self.assertEqual(final['capacity_bytes'], 2 * 2 * self.sizes['TableBucket'])
        self.assertEqual(reserve['capacity_bytes'], 2 * 4 * self.sizes['TableBucket'])
        self.assertEqual(reserve['requested_bytes'], 2 * 6 * self.sizes['TableBucket'])

    def test_directory_growth_requests_include_discarded_arrays(self):
        self.assertEqual(model.directory_for(5, 8), (64, 96, 2))
        small = model.store([0, 1, 33], 12, (8, 8), self.sizes)
        # Six pages, two directories (4 and 8 entries), three embedded roots.
        self.assertEqual(small['capacity_bytes'], 48 * 12)
        self.assertEqual(small['directory_live_bytes'], (4 + 8) * 8)
        self.assertEqual(small['root_bytes'], 3 * self.sizes['ThinStore'])
        self.assertEqual(small['allocation_calls_excluding_embedded_roots'], 6 + 3)
        self.assertEqual(small['requested_bytes'] - small['live_bytes'], 4 * 8)
        for count in range(50):
            self.assertEqual(model.pages_for(count, 8, 8), (((count + 7) // 8) * 8, (count + 7) // 8))
        with self.assertRaises(ValueError): model.pages_for(1, 3, 16)
        with self.assertRaises(ValueError): model.pages_for(-1, 2, 16)

    def test_physical_box_buffers_keep_nil_tail_and_zero_allocations(self):
        costs = model.exact_buffers([{'0': 2, '1': 3, '8': 1}], 4)
        self.assertEqual(costs['live_bytes'], 44)
        self.assertEqual(costs['allocation_calls_excluding_embedded_roots'], 4)
        for bad in ({'01': 1}, {'1/0': 1}, {'1': True}, {'1': 0}):
            with self.assertRaises(ValueError): model.parse_histogram(bad)
        with self.assertRaises(ValueError): model.parse_histogram({'2/1': 1}, True)

    def test_sparse_runtime_ids_are_not_free_when_none_assigned(self):
        result = model.side_table([0, 0, 1], self.sizes['RuntimeBucket'], self.sizes['RuntimeTable'])
        self.assertEqual(result['root_bytes'], 3 * self.sizes['RuntimeTable'])
        self.assertEqual(result['capacity_bytes'], 2 * self.sizes['RuntimeBucket'])
        self.assertEqual(result['allocation_calls_excluding_embedded_roots'], 1)

    @staticmethod
    def owner_row():
        return {'arenas': {name: {'len': 1} for name in ('symbols','flows','flow_lists','declarations','tables','core_aux')},
            'flow_data': {'None': 0, 'Ast': 0, 'SwitchClause': 1, 'ReduceLabel': 0},
            'symbol_runtime_ids_assigned': 1,
            'declarations': {'physical_length_histogram': {'2': 1}},
            'tables': {'length_capacity_histogram': {'1/3': 1}},
            'aux_variants': {'List': 1, 'Nodes': 1, 'Text': 1},
            'node_lists': {'distinct_descriptors': 1},
            'node_backings': {'physical_length_histogram': {'2': 1}}}

    def test_outlined_flow_charges_synthetic_roots_pages_and_missing_counts_fail(self):
        rows = [self.owner_row()]
        candidate = model.binding_candidate(rows, (8,8), self.sizes, 'SymbolText4', 'FlowOutlined', False)
        synthetic = candidate['components']['synthetic_SwitchClause']
        self.assertEqual(synthetic['capacity_bytes'], 8 * self.sizes['SwitchClause'])
        self.assertEqual(candidate['components']['synthetic_ReduceLabel']['root_bytes'], self.sizes['ThinStore'])
        del rows[0]['flow_data']['ReduceLabel']
        with self.assertRaisesRegex(ValueError, 'synthetic payload'):
            model.binding_candidate(rows, (8,8), self.sizes, 'SymbolText4', 'FlowOutlined', False)

    def test_packed_flow_never_claims_unobserved_zero_escapes(self):
        row = self.owner_row()
        candidate = model.binding_candidate([row], (8,8), self.sizes, 'SymbolText4', 'FlowPacked', False)
        self.assertIn('unknown', candidate['escape_status'])
        self.assertEqual(candidate['components']['flow_escape_table_roots']['live_bytes'], self.sizes['FlowEscapeTable'])
        self.assertEqual(candidate['additional_all_flows_escape_bound']['live_bytes'], 2 * self.sizes['FlowEscapeBucket'])
        self.assertGreater(candidate['components']['synthetic_SwitchClause']['live_bytes'], 0)

    def test_auxiliary_routing_cost_is_conditional_not_silently_removed(self):
        rows = [self.owner_row()]
        full = model.auxiliary_candidate(rows, (8,8), self.sizes, False, self.census, 'original_global_slots')
        typed = model.auxiliary_candidate(rows, (8,8), self.sizes, False, self.census, 'per_kind_arena_ids')
        self.assertEqual(full['totals']['live_bytes'] - typed['totals']['live_bytes'],
                         full['components']['auxiliary_locator']['live_bytes'])
        self.assertIn('reject foreign', typed['routing_obligation'])
        indirect = model.auxiliary_candidate(rows, (8,8), self.sizes, True, self.census, 'per_kind_arena_ids')
        self.assertGreater(indirect['components']['distinct_slice_descriptors']['live_bytes'], 0)
        self.assertEqual(indirect['slice_interning_traffic'], 'unpriced')


if __name__ == '__main__': unittest.main()
