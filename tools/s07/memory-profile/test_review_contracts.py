"""Independent counterexamples for source binding and checkpoint acceptance."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import capture


def memory(live=4,total=10,peak=6):
    return {'live_requested_bytes':live,'total_requested_bytes':total,'peak_requested_bytes':peak}


def go_snapshot(name):
    stats={key:0 for key in capture.GO_MEM_INTS}
    stats.update({'PauseNs':[0]*256,'PauseEnd':[0]*256,'GCCPUFraction':0.0,
                  'EnableGC':True,'DebugGC':False,'BySize':[]})
    return {'name':name,'unix_time_ns':1,'mem_stats':stats}


class BuildIdentityContracts(unittest.TestCase):
    def test_current_source_cannot_bless_stale_tools_or_replaced_binary(self):
        with tempfile.TemporaryDirectory() as temp:
            binary=Path(temp)/'adapter';binary.write_bytes(b'original native adapter')
            current_source={'sha256':'a'*64,'files':[]}
            current_tools={'tools/adapter.rs':'b'*64}
            manifest={'schema':1,'diagnostic_only':True,'source_fingerprint':current_source,
                      'tool_inputs':current_tools,'cargo_configuration':{},'rust':{'binary':str(binary),'sha256':capture.build.sha(binary)}}
            with patch.object(capture,'source_fingerprint',return_value=current_source),patch.object(capture.build,'tool_inputs',return_value=current_tools),patch.object(capture.build,'cargo_configuration_paths',return_value=[]),patch.object(capture,'native_environment',return_value={}):
                capture.validate_build(manifest,['rust'])
                stale=copy.deepcopy(manifest);stale['tool_inputs']['tools/adapter.rs']='c'*64
                with self.assertRaisesRegex(ValueError,'tool inputs changed'):
                    capture.validate_build(stale,['rust'])
                stale=copy.deepcopy(manifest);stale['source_fingerprint']['sha256']='d'*64
                with self.assertRaisesRegex(ValueError,'compiler source changed'):
                    capture.validate_build(stale,['rust'])
                binary.write_bytes(b'replacement adapter')
                with self.assertRaisesRegex(ValueError,'binary changed'):
                    capture.validate_build(manifest,['rust'])

    def test_false_schema_and_missing_selected_runtime_fail(self):
        manifest={'schema':True,'diagnostic_only':True}
        with self.assertRaisesRegex(ValueError,'schema'):
            capture.validate_build(manifest,['rust'])
        with patch.object(capture,'source_fingerprint',return_value={}),patch.object(capture.build,'tool_inputs',return_value={}),patch.object(capture.build,'cargo_configuration_paths',return_value=[]),patch.object(capture,'native_environment',return_value={}):
            manifest={'schema':1,'diagnostic_only':True,'source_fingerprint':{},'tool_inputs':{},'cargo_configuration':{}}
            with self.assertRaisesRegex(ValueError,'missing built adapter'):
                capture.validate_build(manifest,['go'])

    def test_external_cargo_config_drift_is_not_hidden_by_current_source_hash(self):
        with tempfile.TemporaryDirectory() as temp:
            config=Path(temp)/'config.toml';config.write_text('[build]\njobs=1\n')
            binary=Path(temp)/'adapter';binary.write_bytes(b'adapter')
            manifest={'schema':1,'diagnostic_only':True,'source_fingerprint':{},'tool_inputs':{},
                      'cargo_configuration':{str(config):capture.build.sha(config)},
                      'rust':{'binary':str(binary),'sha256':capture.build.sha(binary)}}
            with patch.object(capture,'source_fingerprint',return_value={}),patch.object(capture.build,'tool_inputs',return_value={}),patch.object(capture.build,'cargo_configuration_paths',return_value=[config]),patch.object(capture,'native_environment',return_value={}):
                capture.validate_build(manifest,['rust'])
                config.write_text('[build]\njobs=8\n')
                with self.assertRaisesRegex(ValueError,'Cargo configuration changed'):
                    capture.validate_build(manifest,['rust'])


class ProtocolContracts(unittest.TestCase):
    def test_empty_false_duplicate_early_and_extra_completion_records_fail(self):
        count=len(capture.CHECKPOINTS['rust'])
        self.assertEqual(capture.validate_protocol_record({'complete':True},'rust',count,False),'complete')
        for value,n,complete in [({},count,False),({'complete':False},count,False),
                ({'complete':True},count-1,False),({'complete':True},count,True),
                ({'complete':True,'ignored':1},count,False)]:
            with self.subTest(value=value,n=n,complete=complete),self.assertRaises(ValueError):
                capture.validate_protocol_record(value,'rust',n,complete)

    def test_checkpoints_after_completion_wrong_order_and_boolean_bytes_fail(self):
        value={'checkpoint':'pre_pipeline','memory':memory()}
        self.assertEqual(capture.validate_protocol_record(value,'rust',0,False),'checkpoint')
        for changed,n,complete in [(value,0,True),(value,1,False),
                ({**value,'memory':{**memory(),'live_requested_bytes':True}},0,False),
                ({**value,'memory':memory(live=8,peak=6)},0,False)]:
            with self.subTest(n=n,complete=complete),self.assertRaises(ValueError):
                capture.validate_protocol_record(changed,'rust',n,complete)

    def test_go_terminal_digest_fields_are_allowed_but_strictly_typed(self):
        value={'complete':True,'diagnostic_only':True,'files':1,'loaded_input_sha256':'a'*64,
               'report':'/tmp/go-report.json','report_sha256':'b'*64}
        count=len(capture.CHECKPOINTS['go'])
        self.assertEqual(capture.validate_protocol_record(value,'go',count,False),'complete')
        for change in ({'files':True},{'report_sha256':'bad'},{'diagnostic_only':False}):
            with self.subTest(change=change),self.assertRaises(ValueError):
                capture.validate_protocol_record({**value,**change},'go',count,False)

    def test_go_checkpoint_heap_arithmetic_and_pause_ring_are_validated(self):
        value={'checkpoint':'pre_pipeline','snapshot':go_snapshot('pre_pipeline')}
        capture.validate_protocol_record(value,'go',0,False)
        for key,item in [('HeapObjects',1),('HeapReleased',1),('PauseNs',[0]),('NumGC',True)]:
            changed=copy.deepcopy(value);changed['snapshot']['mem_stats'][key]=item
            with self.subTest(key=key),self.assertRaises(ValueError):
                capture.validate_protocol_record(changed,'go',0,False)


class ReportContracts(unittest.TestCase):
    def rust(self):
        before=memory(live=2,total=3,peak=2);after=memory(live=6,total=9,peak=7)
        times={'parse_ns':1,'publish_ns':2,'bind_ns':3,'parse_allocated_bytes':4,
               'publish_allocated_bytes':0,'bind_allocated_bytes':2,
               'parse_live_growth_bytes':5,'publish_live_growth_bytes':0,'bind_live_growth_bytes':-1}
        report={'version':1,'runtime':'rust','pre_pipeline':before,'retained_endpoint':after,
                'pipeline_allocated_bytes':6,'pipeline_live_growth_bytes':4,
                'pipeline_superseded_or_freed_requested_bytes':2,'pipeline_wall_ns':6,
                'worker_times':[times.copy()],'elapsed_worker_totals':times.copy(),
                'timer_domain':'elapsed','allocation_domain':'requested',
                'census_cost':{'before':after,'after':after,'wall_ns':1}}
        checkpoints=[{'adapter':{'memory':before}},{'adapter':{'memory':after}}]
        return report,checkpoints

    def test_allocator_delta_total_and_retained_snapshot_identity_are_cross_checked(self):
        report,points=self.rust();capture.validate_report(report,'rust',1,points)
        for mutate in [lambda r:r.update(pipeline_allocated_bytes=7),
                       lambda r:r.update(version=True),
                       lambda r:r['worker_times'][0].update(bind_live_growth_bytes=True),
                       lambda r:r['elapsed_worker_totals'].update(bind_ns=4),
                       lambda r:r['retained_endpoint'].update(peak_requested_bytes=8)]:
            changed=copy.deepcopy(report);mutate(changed)
            with self.assertRaises(ValueError): capture.validate_report(changed,'rust',1,points)

    def test_census_counts_include_overlays_but_cannot_charge_record_observations(self):
        def row(elements,bytes=0):
            return {'containers':1,'allocations':int(bytes>0),'elements':elements,'capacity_elements':elements,
                    'used_payload_bytes':bytes,'capacity_payload_bytes':bytes,'shared_references':0,
                    'shared_header_estimate_bytes':0,'unreported_layout_containers':0}
        census={'schema':1,'domain':'retained-owned-storage-census','limitations':['lower bounds'],
                'rows':{'core.nodes.page_payload':row(2,160),'BindResult.nodes':row(1,88),
                        'NodeData.Token.records':row(3),'core.auxiliary.page_payload':row(1,40),
                        'AstStorageData.List.records':row(1),'BindResult.symbols.page_payload':row(1,112),
                        'SourceFileState.diagnostics':row(0),'BindResult.diagnostics':row(0)}}
        expected={'nodes':2,'symbols':1,'parse_diagnostics':0,'bind_diagnostics':0}
        capture.validate_census(census,expected)
        wrong=copy.deepcopy(census);wrong['rows']['NodeData.Token.records']=row(4)
        with self.assertRaisesRegex(ValueError,'variants do not reconcile'):
            capture.validate_census(wrong,expected)
        wrong=copy.deepcopy(census);wrong['rows']['NodeData.Token.records']=row(3,1)
        with self.assertRaisesRegex(ValueError,'double charges'):
            capture.validate_census(wrong,expected)
        with self.assertRaisesRegex(ValueError,'frozen work differs'):
            capture.validate_census(census,{**expected,'nodes':3})


class SitesContracts(unittest.TestCase):
    def fixture(self):
        site={'name':'arena.push','source_file':'crates/arena.rs','source_line':12}
        inventory={'schema':1,'operation':'staged-safe-allocation-scopes','sites':[site]}
        rows=[]
        for phase,count in [('unscoped',0),('preload',1),('parse',2),('publish',2),('bind',2),('retire',1)]:
            for kind,name,path,line,observations in [
                ('inclusive_site',site['name'],site['source_file'],site['source_line'],0),
                ('phase','phase','adapter phase guard',0,count),
                ('selected_union','selected source regions','dynamic outermost source-site union',0,0)]:
                rows.append({'phase':phase,'scope_kind':kind,'name':name,'source_file':path,'source_line':line,
                             'requested_bytes':0,'allocation_calls':0,'observations':observations})
        for row in rows:
            if row['phase']=='parse':
                row.update(requested_bytes=4,allocation_calls=1)
                if row['scope_kind']!='phase':row['observations']=1
        return rows,inventory,{'files':2}

    def test_missing_tampered_and_unqualified_site_mass_fails(self):
        rows,inventory,report=self.fixture();capture.validate_sites(rows,inventory,report)
        for change in ['missing','source','uint','observations','union','duplicate']:
            wrong=copy.deepcopy(rows)
            index=next(i for i,row in enumerate(wrong) if row['phase']=='parse' and row['scope_kind']=='selected_union')
            if change=='missing':wrong.pop()
            elif change=='source':wrong[0]['source_line']=99
            elif change=='uint':wrong[index]['requested_bytes']=True
            elif change=='observations':wrong[index]['observations']=0
            elif change=='union':wrong[index]['requested_bytes']=5
            elif change=='duplicate':wrong[-1]=wrong[0].copy()
            with self.subTest(change=change),self.assertRaises(ValueError):
                capture.validate_sites(wrong,inventory,report)

    def test_unscoped_has_no_fabricated_phase_total_and_recursive_rows_may_overlap(self):
        rows,inventory,report=self.fixture()
        for row in rows:
            if row['phase']=='unscoped' and row['scope_kind']!='phase':
                row.update(requested_bytes=7,allocation_calls=1,observations=1)
            if row['phase']=='parse' and row['scope_kind']=='inclusive_site':
                row.update(requested_bytes=8,allocation_calls=2,observations=2)
        capture.validate_sites(rows,inventory,report)

    def test_sites_artifact_is_required_and_hashed_by_sites_manifest(self):
        rows,inventory,report=self.fixture()
        with tempfile.TemporaryDirectory() as temp:
            prefix=Path(temp)/'rust';census=Path(str(prefix)+'-census.json');census.write_text('{}')
            with patch.object(capture,'validate_census'):
                with self.assertRaises(FileNotFoundError):
                    capture.validate_artifacts('rust',prefix,report,{'sites_manifest':inventory})
                sites=Path(str(prefix)+'-sites.json');sites.write_text(json.dumps(rows))
                actual=capture.validate_artifacts('rust',prefix,report,{'sites_manifest':inventory})
                self.assertEqual(actual['sites']['sha256'],capture.build.sha(sites))
                with self.assertRaisesRegex(ValueError,'unexpected allocation-site'):
                    capture.validate_artifacts('rust',prefix,report,{})


if __name__=='__main__': unittest.main()
