import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[4]/'scripts'))
from validate import validate_record,validate_capture,aggregate


def fixture():
    arenas={k:dict(len=1,capacity=2,pages=1,directory_capacity=4,element_bytes=8) for k in ('core_nodes','core_aux','symbols','tables','declarations','flows','flow_lists')}
    arenas['core_aux']['len']=2
    return dict(version=1,index=0,bound_in_place=True,core_shapes={'Identifier':1},node_runtime_ids_by_shape={},symbol_runtime_ids_assigned=0,
        identifiers=dict(shapes={'Identifier':1},classes={'byte_mismatch':1},selected_bytes=1,suffix_selected_bytes=0,fallback_selected_bytes=1,fallback_unique_values=1,fallback_unique_selected_bytes=1,fallback_length_histogram={'1':1}),
        arenas=arenas,aux_variants={'Nodes':1,'List':1},source_metadata_variants={},text_slice_elements=0,metadata_text_slice_elements=0,
        node_lists=dict(exposed_length_histogram={'0':1},distinct_descriptors=1,nil=0,missing=0,allocated_empty=1,nonzero_start=0),
        node_backings=dict(physical_length_histogram={'0':1},physical_lengths_in_aux_order=[0],nonnull_elements=0),
        declarations=dict(physical_length_histogram={'0':1},nonnull_elements=0,not_referenced_by_physical_symbols=0,distinct_symbol_descriptors=1,symbol_length_capacity_histogram={'0/0':1}),
        tables=dict(length_capacity_histogram={'1/3':1}),flow_data={'None':1,'Ast':0,'SwitchClause':0,'ReduceLabel':0},
        binding_maps=dict(overlay={'len':0,'capacity':0},bindings={'len':0,'capacity':0},flow_slots=[0,0,0,0,0,0]),lazy_slots_before=[0,0,0,0],lazy_slots_after=[0,0,0,0])


class Contracts(unittest.TestCase):
    def test_cross_checks_observed_physical_storage(self):
        row=fixture();self.assertEqual(validate_record(row,0),row)
        totals=aggregate([row,row]);self.assertEqual(totals['arenas']['core_aux']['len'],4)
        self.assertEqual(totals['arenas']['core_aux']['element_bytes'],8)
    def test_lost_backing_order_and_changed_lengths_are_rejected(self):
        for mutate in [lambda r:r['node_backings'].update(physical_lengths_in_aux_order=[]),lambda r:r['node_backings'].update(physical_lengths_in_aux_order=[1]),lambda r:r['node_backings'].update(nonnull_elements=1)]:
            row=fixture();mutate(row)
            with self.assertRaises(ValueError):validate_record(row,0)
    def test_runtime_observer_and_lazy_mutations_cannot_pass(self):
        for mutate in [lambda r:r.update(node_runtime_ids_by_shape={'Identifier':2}),lambda r:r.update(symbol_runtime_ids_assigned=2),lambda r:r.update(lazy_slots_after=[1,0,1,0])]:
            row=fixture();mutate(row)
            with self.assertRaises(ValueError):validate_record(row,0)
    def test_table_capacity_and_physical_declaration_mismatches_fail(self):
        for mutate in [lambda r:r['tables'].update(length_capacity_histogram={'4/3':1}),lambda r:r['declarations'].update(physical_length_histogram={}),lambda r:r['flow_data'].update(Ast=1)]:
            row=fixture();mutate(row)
            with self.assertRaises(ValueError):validate_record(row,0)
    def test_missing_identifier_fallback_or_byte_mismatch_fails(self):
        for mutate in [lambda r:r['identifiers']['classes'].clear(),lambda r:r['identifiers'].update(fallback_selected_bytes=2),lambda r:r['identifiers'].update(fallback_unique_values=2)]:
            row=fixture();mutate(row)
            with self.assertRaises(ValueError):validate_record(row,0)
    def test_booleans_and_unknown_fields_are_not_counts(self):
        for mutate in [lambda r:r['arenas']['symbols'].update(len=True),lambda r:r.update(unexpected=1),lambda r:r.update(version=True)]:
            row=fixture();mutate(row)
            with self.assertRaises(ValueError):validate_record(row,0)
    def test_missing_duplicate_reordered_and_changed_input_capture_fails(self):
        expected=dict(files=1,loaded_bytes=1,nodes=1,symbols=1,parse_diagnostics=0,bind_diagnostics=0,loaded_input_sha256='a'*64)
        child=dict(version=1,diagnostic_only=True,runtime='rust',workers=1,domain='untimed',**expected,bound_in_place_files=1,fallback_files=0)
        raw=json.dumps(fixture()).encode()+b'\n'
        self.assertEqual(len(validate_capture(raw,child,expected)),1)
        for wrong in [b'',raw+raw,json.dumps({**fixture(),'index':1}).encode()]:
            with self.assertRaises(ValueError):validate_capture(wrong,child,expected)
        with self.assertRaises(ValueError):validate_capture(raw,{**child,'loaded_input_sha256':'b'*64},expected)
        with self.assertRaises(ValueError):validate_capture(raw,{**child,'version':True},expected)
        with self.assertRaises(ValueError):validate_capture(raw.replace(b'"index": 0',b'"index": 0, "index": 0'),child,expected)

if __name__=='__main__': unittest.main()
