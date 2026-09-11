"""Compact, lossless action contracts derived from the accepted native capture."""
from collections import Counter
import json
import lzma
import tarfile
from pathlib import Path

from s04_common import strict_json_loads
from s07_acceptance import load_partition, select_acceptance
from s08_baselines import review_capture
from s08_oracle import ROOT, canonical, digest

OPERATIONS=('GetTypeAtLocation','GetSymbolAtLocation','TypeToTypeNode','SymbolToStringEx')
COLUMNS=('operation','file','kind','pos','end','flags','internal_flags','result_flags','absent')
ARCHIVE='tools/s08/results/acceptance-amendment/baselines.tar.xz'
MEMBER='amended-baselines-final/'


def expected_baseline(value):
    state=value['state']
    if state=='content':
        if set(value)!={'state','text_hex'}:raise ValueError('malformed content')
        raw=bytes.fromhex(value['text_hex'])
        return [state,len(raw),digest(raw)]
    if state not in ('no_content','disabled') or set(value)!={'state'}:raise ValueError('unavailable required baseline')
    return [state]


def action(query,files):
    if not {'operation','file','kind','pos','end'} <= set(query):raise ValueError('query lacks coordinates')
    if set(query)-set(COLUMNS)-{'type_id'}:raise ValueError('unknown query field')
    if query['operation'] not in OPERATIONS:raise ValueError('unknown query')
    file=query['file']
    if not isinstance(file,str) or not file:raise ValueError('invalid query file')
    if file not in files:files[file]=len(files)
    values=[OPERATIONS.index(query['operation']),files[file]]
    for key in COLUMNS[2:-1]:
        value=query.get(key,0)
        if type(value) is not int:raise ValueError('query integer expected')
        if key in ('pos','end'):
            if not -1 <= value < 2**31:raise ValueError('query coordinate out of bounds')
        elif not 0 <= value < 2**32:raise ValueError('query flag out of bounds')
        values.append(value)
    if query['end']<query['pos']:raise ValueError('reversed query range')
    absent=query.get('absent',False)
    if type(absent) is not bool:raise ValueError('query absence must be boolean')
    # Native numeric type IDs are deliberately absent: this observation does not
    # carry a checker identity, so it cannot certify cross-file ID relationships.
    values.append(absent)
    return values


def extract_capture(directory):
    """Materialize only authenticated final-capture members, never tar paths blindly."""
    directory=Path(directory);directory.mkdir(parents=True,exist_ok=False)
    wanted={'requests.json','observations.ndjson','report.json','go.stdout','go.stderr'}
    found=set()
    for archive in (ARCHIVE,ARCHIVE.replace('baselines.tar.xz','provenance.tar.xz')):
        manifest=strict_json_loads((ROOT/(archive+'.manifest.json')).read_bytes())
        if digest((ROOT/archive).read_bytes())!=manifest['archive_sha256']:
            raise ValueError('native archive digest mismatch')
        selected={name for name in manifest['files'] if name.startswith(MEMBER) and
                  (name[len(MEMBER):] in wanted or name.startswith(MEMBER+'source-snapshot/'))}
        with tarfile.open(ROOT/archive,'r:xz') as tar:
            for member in tar:
                if member.name not in selected:continue
                relative=Path(member.name[len(MEMBER):])
                if not member.isfile() or relative.is_absolute() or '..' in relative.parts or member.name in found:
                    raise ValueError('unsafe or duplicate archive member')
                content=tar.extractfile(member).read()
                record=manifest['files'][member.name]
                if digest(content)!=record['sha256'] or len(content)!=record['bytes']:
                    raise ValueError('archive member content mismatch')
                target=directory/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(content)
                found.add(member.name)
        if not selected<=found:raise ValueError('missing native archive member')
    if not {MEMBER+name for name in wanted}<=found:raise ValueError('incomplete archived capture')
    return directory


def prepare(capture,directory):
    capture=Path(capture).resolve();directory=Path(directory).resolve();directory.mkdir(parents=True,exist_ok=False)
    archive_manifest=strict_json_loads((ROOT/(ARCHIVE+'.manifest.json')).read_bytes())
    if digest((ROOT/ARCHIVE).read_bytes())!=archive_manifest['archive_sha256']:raise ValueError('baseline archive changed')
    for name in ('requests.json','observations.ndjson','report.json'):
        if digest((capture/name).read_bytes())!=archive_manifest['files'][MEMBER+name]['sha256']:raise ValueError('capture differs from archived final capture: '+name)
    review_capture(capture,directory/'capture-review.json')
    if strict_json_loads((directory/'capture-review.json').read_bytes())['mismatches']:
        raise ValueError('native baseline capture has unresolved contract mismatches')
    subset=strict_json_loads((ROOT/'data/s07/subset.json').read_bytes());partition=load_partition(subset)
    phases=strict_json_loads((ROOT/'data/s08/baseline-requests.json').read_bytes())
    phase_by_id={r['id']:(i,r) for i,r in enumerate(phases['requests'])};requests=[];counts=Counter();info=Counter()
    with (capture/'observations.ndjson').open() as source:
        for line in source:
            row=strict_json_loads(line);phase_index,phase=phase_by_id[row['id']]
            if row['acceptance_tier']!=phase['acceptance_tier']:raise ValueError('tier changed')
            if row['acceptance_tier']=='informational':
                info[row['state']]+=1;continue
            if row['state']!='executed':raise ValueError('required native contract unavailable: '+row['id'])
            files={};queries=[action(q,files) for q in row['queries']]
            counts.update(q['operation'] for q in row['queries'])
            if row['pre_diagnostics']!=row['post_diagnostics']:raise ValueError('pre/post diagnostics changed')
            requests.append({'id':row['id'],'phase_request':phase_index,'files':list(files),
                'queries':queries,'baselines':{k:expected_baseline(row[k]) for k in ('types','symbols','errors')},
                'diagnostics':{'count':len(row['pre_diagnostics']),'sha256':digest(canonical(row['pre_diagnostics']))}})
    if select_acceptance(requests,partition['variants'])!=requests:raise ValueError('wrong required query inventory')
    document={'version':1,'operations':OPERATIONS,'columns':COLUMNS,'requests':requests}
    raw=canonical(document)+b'\n';compressed=lzma.compress(raw,preset=3)
    (directory/'query-schedule.json.xz').write_bytes(compressed)
    manifest={'version':1,'pin':subset['pin'],'acceptance_variants':len(requests),'query_operations':dict(counts),
        'informational_states':dict(info),'query_schedule':{'storage':'Derived cache under target; authenticated existing archive is the committed authority','format':'canonical JSON, optional xz cache; projection version 1','decoded_sha256':digest(raw),'decoded_bytes':len(raw)},
        'sources':{p:digest((ROOT/p).read_bytes()) for p in ('scripts/s08_queries.py','scripts/s08_baselines.py','scripts/s07_acceptance.py','data/s08/baseline-requests.json','data/s07/e2-acceptance.json')},
        'authority':{'archive':ARCHIVE,'sha256':archive_manifest['archive_sha256'],'member_prefix':MEMBER},
        'execution':'Port the pinned baseline walker and verify its ordered actions, including reparsed nodes. A range locator is an observation, not a unique NodeId or permission to skip traversal.',
        'identities':'Raw native type IDs remain in the archive only; ownership/identity assertions belong to owner-scoped supplemental fixtures.',
        'comparison':'Exact ordered action columns and baseline outcome/bytes. Hashes index archived bytes; a mismatch must expose actual and expected bytes. NoContent, disabled and empty content are distinct. Informational results never affect E2.'}
    (directory/'query-contract.json').write_bytes(canonical(manifest)+b'\n')
    print(json.dumps({'variants':len(requests),'queries':sum(counts.values()),'compressed_bytes':len(compressed)}))
    return manifest
