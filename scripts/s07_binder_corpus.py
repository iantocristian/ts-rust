"""Frozen binder obligations reuse S06's clean pinned Go input expansion."""
from collections import Counter
from contextlib import ExitStack
import hashlib
import json
from pathlib import Path
import sys

from s04_common import strict_json_loads
from s06 import freeze as freeze_parser, producer_lock
from s06_corpus import request_recipe
from s06_protocol import canonical


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def primary_requests():
    from s07_binder import convert_request
    with producer_lock():
        documents, requests, changes = freeze_parser(False)
    if changes:
        raise ValueError("S06 corpus changed during binder expansion")
    return documents, [convert_request(request) for request in requests]


def inventory(requests, oracle_binary, output):
    """Observe every Go stage; process/observer errors abort the entire freeze."""
    from s07_binder import Process, panic_class
    output = Path(output); output.mkdir(parents=True, exist_ok=True)
    items=[]; reached=Counter(); panics=Counter()
    process=Process([str(oracle_binary)],output/"oracle.stderr",deadline=120)
    try:
        with (output/"outcomes.ndjson").open("wb") as file:
            for index,request in enumerate(requests):
                process.send(request); outcomes=[]; count=0
                for record in process.observations(request):
                    count+=1
                    if record["tag"]=="stage":
                        outcomes.append({key:record[key] for key in ("stage","outcome","message_hex")})
                        if record["stage"] in ("parse","bind","repeat_bind"):
                            reached[record["stage"]]+=1
                        if record["outcome"]=="panic":
                            identity=panic_class(request,record["stage"],record["message_hex"],"oracle")
                            if identity is None:
                                raise ValueError("unclassified source panic during freeze: "+json.dumps(record))
                            panics[record["stage"]]+=1
                item={"id":request["id"],"request_sha256":sha(canonical(request)),"outcomes":outcomes,"records":count}
                items.append(item);file.write(canonical(item)+b"\n")
                if index%250==0:
                    file.flush();print(f"S07 Go preflight {index+1}/{len(requests)}",file=sys.stderr,flush=True)
        process.finish();digest=process.digest.hexdigest()
    finally:
        process.close()
    return {"requests":len(requests),"reached":dict(reached),"panics":dict(panics),"raw_stream_sha256":digest,"outcomes":items}


def documents(parser,requests,supplemental,preflight):
    from s07_binder import ROOT, OPERATIONS
    recipes=[]
    for request,recipe in zip(requests,parser["requests.json"]["requests"]):
        if request["id"]!=recipe["id"]:
            raise ValueError("binder/parser recipe order drift")
        recipes.append(request_recipe(request,recipe["unit"],recipe["configuration"]))
    pin=parser["probes.json"]["pin"]
    shapes=[];compact=[]
    for item in preflight["outcomes"]:
        if item["outcomes"] not in shapes:shapes.append(item["outcomes"])
        compact.append({"id":item["id"],"request_sha256":item["request_sha256"],"records":item["records"],"shape":shapes.index(item["outcomes"])})
    preflight={**preflight,"outcome_shapes":shapes,"outcomes":compact}
    probes={"version":1,"pin":pin,"primary_rows":len(parser["cases.json"]),"primary_requests":len(requests),
        "request_sha256":sha(canonical(requests)),"parser_expansion_sha256":parser["probes.json"]["request_sha256"],
        "syntax_schema_sha256":sha((ROOT/"data/s03/schema/ast.json").read_bytes()),
        "operations":OPERATIONS,"scope":"all frozen S06 primary parser-entry requests, followed by native binding; parser-terminal outcomes retain their rows",
        "source_preflight":preflight,"supplemental_requests":len(supplemental),"supplemental_sha256":sha(canonical(supplemental)),
        "normalization":{"names":"only exact source-generated pattern@NodeID and private class #SymbolID components; raw diagnostics stay exact",
            "synthetic_flow_identity":"owner canonical FlowId plus payload discriminant; Go observer rejects multiple owners",
            "slice_aliases":"connected overlapping exported ranges, including declaration capacity; offsets relative to first visible element; inaccessible allocation boundaries excluded"}}
    return {"binder-cases.json":parser["cases.json"],"binder-requests.json":{"version":1,"pin":pin,"requests":recipes},
        "binder-supplemental.json":{"version":1,"pin":pin,"requests":supplemental},"binder-probes.json":probes}


def freeze(write=False):
    from s07_binder import ROOT,build_oracle,smoke_requests
    parser,requests=primary_requests();supplemental=smoke_requests();oracle,pin=build_oracle()
    preflight=inventory(requests,oracle,ROOT/"target/s07-binder-preflight")
    # Runtime IDs are deliberately absent from obligations. Raw stream hashes are
    # retained in the preflight artifact, not required to be stable across runs.
    raw_digest=preflight.pop("raw_stream_sha256")
    result=documents(parser,requests,supplemental,preflight)
    changes=write_manifests(ROOT/"data/s07",result,write=write)
    return result,requests,changes,raw_digest


def write_manifests(directory, documents, *, write=False):
    # One explicit obligation per line, with deduplicated complete stage shapes.
    # No outcome or physical row is removed by this serialization choice.
    encoded={}
    for name,value in documents.items():
        if name=="binder-cases.json":
            raw=b"[\n"+b",\n".join(b"  "+canonical(item) for item in value)+b"\n]\n"
        elif name in ("binder-requests.json","binder-supplemental.json"):
            header={key:item for key,item in value.items() if key!="requests"}
            raw=canonical(header)[:-1]+b',"requests":[\n'+b",\n".join(b"  "+canonical(item) for item in value["requests"])+b"\n]}\n"
        else:
            raw=canonical(value)+b"\n"
        encoded[name]=raw
    directory=Path(directory)
    changed=[name for name,raw in encoded.items() if not (directory/name).exists() or (directory/name).read_bytes()!=raw]
    if changed and not write:raise ValueError("S07 frozen binder manifest drift: "+", ".join(changed))
    if write:
        directory.mkdir(parents=True,exist_ok=True)
        for name,raw in encoded.items():(directory/name).write_bytes(raw)
    return changed


def validate_results(requests,results):
    expected=[request["id"] for request in requests]
    if [item["id"] for item in results]!=expected or len(set(expected))!=len(expected):
        raise ValueError("missing, duplicate, extra or reordered binder result/variant")
    rows={request["primary"]:True for request in requests if request["primary"] is not None}
    for request,result in zip(requests,results):
        if result["primary"]!=request["primary"]:
            raise ValueError("changed primary-row mapping")
        if request["primary"] is not None:
            rows[request["primary"]]&=result["equal"]
    return rows


def capture(documents,requests,oracle_binary,rust_binary,output,*,prefix=None):
    from s07_binder import Process,compare,ROOT
    output=Path(output);output.mkdir(parents=True,exist_ok=True)
    all_requests=requests+documents["binder-supplemental.json"]["requests"]
    selected=[request for request in all_requests if prefix is None or request["id"].startswith(prefix)]
    if not selected:raise ValueError("empty diagnostic subset")
    results=[]
    with ExitStack() as cleanup:
        processes={}
        for runtime,binary in (("oracle",oracle_binary),("rust",rust_binary)):
            process=Process([str(binary)],output/(runtime+".stderr"),deadline=120);cleanup.callback(process.close);processes[runtime]=process
        with (output/"requests.ndjson").open("wb") as outcomes,(output/"failures.ndjson").open("wb") as failures:
            for index,request in enumerate(selected):
                for process in processes.values():process.send(request)
                try: result=compare(request,**processes)
                except (ValueError,RuntimeError,OSError) as error:raise RuntimeError(f"invalid S07 capture at {request['id']}: {error}") from error
                if not result["equal"]:
                    failures.write(canonical({"request":request,"failure":result["first_difference"]})+b"\n");failures.flush()
                    print("S07 mismatch "+request["id"]+": "+json.dumps(result["first_difference"])[:1000],file=sys.stderr,flush=True)
                outcomes.write(canonical(result)+b"\n");results.append(result)
                if index%250==0:outcomes.flush();print(f"S07 compared {index+1}/{len(selected)}",file=sys.stderr,flush=True)
        for process in processes.values():process.finish()
        digests={runtime:process.digest.hexdigest() for runtime,process in processes.items()}
    if prefix is not None:return {"diagnostic_subset":True,"requests":len(results),"failed_requests":sum(not item["equal"] for item in results),"stream_sha256":digests}
    rows=validate_results(requests,results[:len(requests)])
    expected=documents["binder-cases.json"]
    if set(rows)!=set(expected):raise ValueError("frozen binder row membership changed")
    expected_preflight=documents["binder-probes.json"]["source_preflight"]["outcomes"]
    reached=Counter()
    if len(expected_preflight)!=len(requests):raise ValueError("missing frozen source outcome")
    for result,expected in zip(results,expected_preflight):
        oracle=[{key:stage[key] for key in ("stage","outcome","message_hex")} for stage in result["stages"]["oracle"]]
        if result["id"]!=expected["id"] or oracle!=documents["binder-probes.json"]["source_preflight"]["outcome_shapes"][expected["shape"]]:
            raise ValueError("Go outcome drift from independent frozen preflight: "+result["id"])
        if result["records"]["oracle"]!=expected["records"]:
            raise ValueError("Go graph record count drift from independent preflight: "+result["id"])
        for runtime in ("oracle","rust"):
            reached[runtime]+=any(stage["stage"]=="bind" for stage in result["stages"][runtime])
    if reached["oracle"]!=documents["binder-probes.json"]["source_preflight"]["reached"].get("bind",0):raise ValueError("Go reached-bind denominator changed")
    return {"scope":"frozen-primary-and-separate-supplemental","requests":len(results),"primary_requests":len(requests),
        "primary_rows":len(rows),"passed_rows":sum(rows.values()),"parity":sum(rows.values())/len(rows),"failed_requests":sum(not item["equal"] for item in results),
        "reached_bind":dict(reached),"supplemental":{"requests":len(results)-len(requests),"passed":sum(item["equal"] for item in results[len(requests):]),"parity":sum(item["equal"] for item in results[len(requests):])/(len(results)-len(requests))},
        "stream_sha256":digests,"tests":rows,
        "raw_exact_requests":sum(item["raw_exact"] for item in results),
        "qualified_name_observations":{runtime:sum(len(item["qualified_name_observations"][runtime]) for item in results) for runtime in ("oracle","rust")}}
