"""Sequential Go relation/ordering observations with original source algorithms."""
import json
from pathlib import Path
import subprocess

from s04 import go_environment,verified_upstream
from s04_common import strict_json_loads
from s08_baselines import replace_exact
from s08_oracle import ROOT,canonical,digest
from s08_contracts import fields, hex_bytes, relation_start_spec

SPEC='tools/s08/contracts/relations.json'
BRIDGE='tools/s08/oracle/contracts/bridge.go'
DRIVER='tools/s08/oracle/contracts/cases_test.go'
EXTRA='tools/s08/oracle/contracts/residuals.go'
TEXT='tools/s08/contracts/text.json'
RESIDUALS='tools/s08/contracts/residuals.json'


def requests(spec):
    fields(spec,'version scope options modes sequence_per_mode report_errors cases')
    if spec['version']!=1 or type(spec['cases']) is not list or not spec['cases']:raise ValueError('invalid fixture specification')
    if spec['modes']!=['identity','assignable','subtype','strict_subtype','comparable'] or spec['sequence_per_mode']!=['A-to-B-first','A-to-B-repeat','B-to-A','A-to-A'] or spec['report_errors'] is not True:raise ValueError('relation protocol changed')
    if canonical(spec['options'])!=canonical({'target':'ESNext','module':'ESNext','strict':True,'skipLibCheck':True}):raise ValueError('fixture options changed')
    result=[];seen=set()
    for case in spec['cases']:
        if set(case)-{'id','source','capabilities','expected_diagnostic_codes','files','allow_js','setup','starting_cache_entries','first_call','resolved_primitive'}:raise ValueError('unknown relation fixture field')
        relation_start_spec(case)
        if type(case['id']) is not str or type(case['source']) is not str or type(case['capabilities']) is not list or any(type(v) is not str for v in case['capabilities']):raise ValueError('malformed relation fixture')
        if type(case.get('allow_js',False)) is not bool or type(case['expected_diagnostic_codes']) is not list or any(type(v) is not int or v<=0 for v in case['expected_diagnostic_codes']):raise ValueError('invalid fixture options/diagnostics')
        if any(not p.startswith('/') or p=='/fixture.ts' or '..' in Path(p).parts or type(t) is not str for p,t in case.get('files',{}).items()):raise ValueError('invalid fixture extra file')
        if case['id'] in seen or not case['source'] or not case['capabilities']:raise ValueError('invalid relation fixture')
        seen.add(case['id'])
        actions=[{'mode':mode,'source':a,'target':b,'report_errors':True} for mode in spec['modes'] for a,b in [('A','B'),('A','B'),('B','A'),('A','A')]]
        result.append({'id':case['id'],'source_hex':case['source'].encode().hex(),'actions':actions,'allow_js':case.get('allow_js',False),'files':{p:t.encode().hex() for p,t in case.get('files',{}).items()}})
    return result


def prepare(directory):
    directory=Path(directory).resolve();directory.mkdir(parents=True,exist_ok=False)
    upstream=verified_upstream();env=go_environment()
    spec=strict_json_loads((ROOT/SPEC).read_bytes());request=requests(spec);raw=canonical(request)+b'\n'
    (directory/'requests.json').write_bytes(raw)
    relater=(upstream/'tsc/internal/checker/relater.go').read_text()
    anchor='\tresult := r.isRelatedToEx(source, target, RecursionFlagsBoth, errorNode != nil /*reportErrors*/, headMessage, IntersectionStateNone)\n'
    relater=replace_exact(relater,anchor,anchor+'\tif s08RelationObserver != nil { s08RelationObserver(c,result) }\n')
    overlay={}
    for name,content in {'relater.go':relater,'s08_contract_residuals.go':(ROOT/EXTRA).read_text(),'s08_contract_bridge.go':(ROOT/BRIDGE).read_text(),'s08_contract_cases_test.go':(ROOT/DRIVER).read_text()}.items():
        path=directory/name;path.write_text(content);overlay[str(upstream/'tsc/internal/checker'/name)]=str(path)
    (directory/'overlay.json').write_bytes(canonical({'Replace':overlay})+b'\n')
    sources={name:digest((ROOT/name).read_bytes()) for name in (SPEC,BRIDGE,DRIVER,EXTRA,TEXT,RESIDUALS,'scripts/s08_contracts.py','scripts/s08_contract_oracle.py','scripts/s04.py','scripts/s04_common.py','scripts/s04_runtime.py','scripts/s08_oracle.py','data/s04/toolchains.toml')}
    env.update(S08_TEXT_REQUESTS=str(ROOT/TEXT),S08_REQUESTS=str(directory/'requests.json'),S08_OUTPUT=str(directory/'observations.json'))
    cmd=['go','test','-mod=readonly','-trimpath','-overlay',str(directory/'overlay.json'),'./internal/checker','-run','^TestS08Contracts$','-count=1','-timeout=5m']
    (directory/'command.json').write_bytes(canonical(cmd)+b'\n')
    with (directory/'stdout').open('wb') as stdout,(directory/'stderr').open('wb') as stderr:
        run=subprocess.run(cmd,cwd=upstream/'tsc',env=env,stdout=stdout,stderr=stderr,timeout=330)
    if run.returncode:raise ValueError(f'contract observation failed; retained at {directory}')
    observed=strict_json_loads((directory/'observations.json').read_bytes())
    from s08_contracts import validate
    validate(spec,request,strict_json_loads((ROOT/TEXT).read_bytes()),strict_json_loads((ROOT/RESIDUALS).read_bytes()),observed)
    verified_upstream()
    if sources!={name:digest((ROOT/name).read_bytes()) for name in sources}:raise ValueError('contract sources changed')
    report={'version':1,'pin':strict_json_loads((ROOT/'data/upstream.json').read_bytes())['pin'],'sources':sources,'native_sources':{p:digest((upstream/'tsc'/p).read_bytes()) for p in ('internal/checker/checker.go','internal/checker/relater.go','internal/checker/utilities.go','internal/checker/mapper.go','internal/checker/printer.go','internal/printer/printer.go','go.mod','go.sum')},'runtime':{k:observed[k] for k in ('go','goos','goarch')},'request_sha256':digest(raw),'observation_sha256':digest((directory/'observations.json').read_bytes()),'cases':len(request),'relation_actions':sum(len(r['actions']) for r in request),'scope':'Original Go relation, ordering, display and diagnostics; counters are diagnostic state, not footprint/throughput evidence'}
    (directory/'report.json').write_bytes(canonical(report)+b'\n');print(json.dumps(report))
    return observed,report
