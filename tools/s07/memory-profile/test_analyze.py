import unittest
import analyze

class DomainAccountingTests(unittest.TestCase):
    def test_contained_go_header_and_static_strings_are_not_added(self):
        value={'objects':[{'category':'concrete_node_including_header','count':2,'logical_bytes':160}],
            'slices':[{'visible_capacity_union_bytes':24}], 'maps':[{'entry_logical_bytes':48}],
            'source_text_bytes':100,'node_header_contained_bytes':96,
            'strings':{'other_visible_union_bytes':999}}
        report=analyze.go_census(value)
        self.assertEqual(report['known_logical_payload_bytes'],332)
        self.assertEqual(report['node_header_contained_bytes_nonadditive'],96)
    def test_rust_spare_capacity_and_arc_estimate_separate(self):
        report=analyze.rust_census({'rows':{'a':{'used_payload_bytes':30,'capacity_payload_bytes':50,'shared_header_estimate_bytes':16},
            'b':{'used_payload_bytes':7,'capacity_payload_bytes':7,'shared_header_estimate_bytes':0}}})
        self.assertEqual(report['capacity_minus_used_bytes'],20)
        self.assertEqual(report['capacity_payload_bytes'],57)
        self.assertEqual(report['payload_plus_arc_estimate_bytes'],73)

if __name__=='__main__':unittest.main()
