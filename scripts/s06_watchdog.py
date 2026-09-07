"""Measured nontermination probes, separate from returned-decoder parity."""
from pathlib import Path
import time

from s04_common import strict_json_loads
from s06_process import Process
from s06_protocol import (StreamCase, canonical, exact_keys, hex_bytes, integer,
                          sha256, text, validate_request)

ROOT = Path(__file__).resolve().parents[1]


def read_manifest(path):
    raw = Path(path).read_bytes()
    value = strict_json_loads(raw)
    exact_keys(value, ('version','pin','primary_rows_contributed',
                       'returned_decoder_parity_rows_contributed','cases'), 'watchdog manifest')
    integer(value['version'],1,1,'watchdog version')
    integer(value['primary_rows_contributed'],0,0,'watchdog primary contribution')
    integer(value['returned_decoder_parity_rows_contributed'],0,0,'watchdog decoder contribution')
    text(value['pin'],'watchdog pin',empty=False)
    pin = strict_json_loads((ROOT/'data/upstream.json').read_bytes())['pin']
    if value['pin'] != pin: raise ValueError('watchdog manifest pin differs from canonical pin')
    if type(value['cases']) is not list or not value['cases']:
        raise ValueError('missing watchdog inventory')
    ids=set()
    for case in value['cases']:
        exact_keys(case,('request','begin_deadline_seconds','after_begin_deadline_seconds','expected','wire_recipe'),'watchdog case')
        request=case['request'];validate_request(request)
        if request['op'] != 'decode' or request['entrypoint'] != 'nodes' or request['primary'] is not None:
            raise ValueError('watchdog must use independent raw DecodeNodes')
        if request['id'] in ids: raise ValueError('duplicate watchdog identity')
        ids.add(request['id'])
        # Fixed bounds were reviewed for this looping, append-only Go operation.
        if type(case['begin_deadline_seconds']) is not int or case['begin_deadline_seconds'] != 10:
            raise ValueError('watchdog startup bound changed')
        if type(case['after_begin_deadline_seconds']) is not float or case['after_begin_deadline_seconds'] != 0.2:
            raise ValueError('watchdog behavior bound changed')
        for key in ('expected','wire_recipe'):text(case[key],key,empty=False)
        # Reject accidental mutation to another crashing or unrelated request.
        wire=hex_bytes(request['wire_hex'],'watchdog wire')
        words=[int.from_bytes(wire[i:i+4],'little') for i in range(0,len(wire),4)]
        if len(wire)!=128 or words[:11]!=[8<<24,0,0,0,0,0,44,44,44,44,44] or words[11:25]!=[0]*14 or words[25:]!=[27,0,0,2,1,0,0]:
            raise ValueError('watchdog wire does not identify the frozen sibling loop')
    return value,sha256(raw)


def run_watchdogs(commands, report_dir, env=None, manifest_path=ROOT/'data/s06/watchdogs.json'):
    """Return measured results; crashes and malformed streams invalidate capture.

    Launch each exact frozen request in a fresh production adapter. Only a valid
    begin followed by a live process at the deadline yields the timeout result.
    A returned frame is a measured failure, not a timeout; no timeout contributes
    to the ordinary decoder success/error or byte-parity criteria.
    """
    if set(commands) != {'oracle','rust'} or any(not command for command in commands.values()):
        raise ValueError('both watchdog runtime commands are required')
    manifest,digest=read_manifest(manifest_path)
    directory=Path(report_dir);directory.mkdir(parents=True,exist_ok=True)
    details=[]
    for ordinal,case in enumerate(manifest['cases']):
        request=case['request']
        for runtime in ('oracle','rust'):
            prefix=directory/f'{ordinal:03d}-{runtime}'
            prefix.with_suffix('.request.json').write_bytes(canonical(request)+b'\n')
            process=Process(commands[runtime],prefix.with_suffix('.stderr'),env=env,deadline=case['begin_deadline_seconds'])
            try:
                process.send(request)
                state=StreamCase(request)
                begin=process.read();state.accept(begin)
                prefix.with_suffix('.begin.json').write_bytes(canonical(begin)+b'\n')
                if begin['tag']!='begin':raise ValueError('watchdog readiness must be a begin frame')
                process.deadline=case['after_begin_deadline_seconds']
                started=time.monotonic()
                try:
                    record=process.read();state.accept(record)
                    prefix.with_suffix('.unexpected.json').write_bytes(canonical(record)+b'\n')
                    outcome='returned'
                except TimeoutError:
                    if process.buffer:
                        prefix.with_suffix('.partial').write_bytes(process.buffer)
                        raise ValueError('partial response at watchdog deadline')
                    if process.process.poll() is not None:
                        raise RuntimeError('watchdog child exited at its deadline')
                    outcome='timeout'
                detail={'id':request['id'],'runtime':runtime,'outcome':outcome,
                        'elapsed_seconds':time.monotonic()-started,'artifact_prefix':str(prefix)}
                prefix.with_suffix('.result.json').write_bytes(canonical(detail)+b'\n')
                details.append(detail)
            except (OSError,ValueError,RuntimeError,TimeoutError) as error:
                prefix.with_suffix('.invalid.txt').write_text(str(error)+'\n')
                raise RuntimeError(f'S06 watchdog capture invalid ({request["id"]}, {runtime}): {error}') from error
            finally:
                process.close()
    if len(details)!=2*len(manifest['cases']):raise ValueError('incomplete watchdog execution')
    return {'metric':all(item['outcome']=='timeout' for item in details),
            'requests':len(manifest['cases']),'runtimes':2,'manifest_sha256':digest,'cases':details}
