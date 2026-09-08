"""Adversarial tests use actual Go graph records as the unmodified control."""
import copy
import json
import subprocess
from pathlib import Path


def run(oracle_binary, rust_binary, directory):
    from s07_binder import (Process, Stream, GraphValidator, canonical, comparable,
                            convert_request, first_difference, panic_class)
    from s06_protocol import parser_request
    directory = Path(directory); directory.mkdir(parents=True, exist_ok=True)
    request = convert_request(parser_request("s07/protocol/control", None,
        b"function f(x: number): number; function f(x) { if(x) { x = 1; } else { x = 2; } return x; }", "/control.ts", "/control.ts", 3))
    process = Process([str(oracle_binary)], directory/"control.stderr")
    try:
        process.send(request); control = list(process.observations(request)); process.finish()
    finally:
        process.close()
    tested = []

    def replay(records):
        stream = Stream(request)
        for record in records:
            stream.accept(record)
        if not stream.ended:
            raise ValueError("no terminal end")

    def reject(name, mutate):
        records = copy.deepcopy(control); mutate(records)
        try:
            replay(records)
        except (ValueError, KeyError, IndexError, TypeError):
            tested.append(name)
        else:
            raise AssertionError("accepted protocol corruption: " + name)

    replay(control)
    reject("missing observation", lambda records: records.pop(4))
    reject("duplicate observation", lambda records: records.insert(4, copy.deepcopy(records[4])))
    reject("extra terminal record", lambda records: records.append(copy.deepcopy(records[-1])))
    reject("missing terminal end", lambda records: records.pop())
    reject("reordered observations", lambda records: records.__setitem__(slice(4,6), records[4:6][::-1]))
    reject("unknown stage", lambda records: records[1].__setitem__("stage", "invented"))
    reject("boolean protocol version", lambda records: records[0].__setitem__("version", True))
    reject("missing graph field", lambda records: next(r for r in records if r.get("kind")=="flow")["value"].pop("antecedent"))
    reject("observer panic laundering", lambda records: next(r for r in records if r["tag"]=="stage" and r["stage"]=="parsed_graph").update(outcome="panic",message_hex=b"invented".hex()))

    # Compare changed source observations directly. This does not depend on
    # repeated-bind validation to notice a wrong edge or declaration identity.
    mutations = {
        "deleted flow edge": ("flow", lambda value: value.__setitem__("antecedent", 0), lambda value: value["antecedent"] != 0),
        "changed graph alias target": ("flow_list", lambda value: value.__setitem__("flow", 0), lambda value: value["flow"] != 0),
        "changed declaration identity": ("declarations", lambda value: value["values"].__setitem__(0,0), lambda value: bool(value["values"])),
        "changed backing alias": ("nodes", lambda value: value.__setitem__("backing_start",value["backing_start"]+1), lambda value: bool(value["values"])),
    }
    for name,(kind,mutation,predicate) in mutations.items():
        changed=copy.deepcopy(control)
        candidate=next(r for r in changed if r.get("stage")=="bound_graph" and r.get("kind")==kind and predicate(r["value"]))
        mutation(candidate["value"])
        if first_difference(comparable(control),comparable(changed)) is None:
            raise AssertionError("comparison ignored "+name)
        tested.append(name)
    for runtime in ("oracle","rust"):
        if panic_class(request,"bind",b"manufactured matching panic".hex(),runtime) is not None:
            raise AssertionError("unclassified panic was accepted")
    tested.append("manufactured matching panic")
    declarations=next(r for r in control if r.get("stage")=="bound_graph" and r.get("kind")=="declarations" and len(r["value"]["values"])>1)
    reordered=copy.deepcopy(declarations);reordered["value"]["values"].reverse()
    if first_difference(declarations,reordered) is None:raise AssertionError("reordered declarations ignored")
    tested.append("reordered declarations")
    from s07_binder_corpus import validate_results
    requests=[{**request,"primary":"row","id":"first"},{**request,"primary":"row","id":"second"}]
    try:validate_results(requests,[{"id":"first","primary":"row","equal":True}])
    except ValueError:tested.append("dropped required variant")
    else:raise AssertionError("missing variant passed its primary row")

    for runtime,binary in (("oracle",oracle_binary),("rust",rust_binary)):
        raw=canonical(request)
        inputs={"truncated request":raw,"duplicate request":raw+b"\n"+raw+b"\n",
            "duplicate JSON key":raw.replace(b'"version":1',b'"version":1,"version":1')+b"\n",
            "fractional version":raw.replace(b'"version":1',b'"version":1.0')+b"\n",
            "invalid scalar":raw.replace(b'"id":"s07/protocol/control"',b'"id":"\\ud800"')+b"\n"}
        for name,source in inputs.items():
            result=subprocess.run([str(binary)],input=source,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=10)
            (directory/(runtime+"-"+name.replace(" ","_")+".stderr")).write_bytes(result.stderr)
            if result.returncode==0:raise AssertionError(runtime+" accepted "+name)
            tested.append(runtime+": "+name)
    # Only the two verified identity spellings are substitutable; literal names,
    # identity targets, delimiter bytes and diagnostic arguments remain exact.
    qualified={"raw_hex":b"\xfe#12@#x".hex(),"identity":{"kind":"symbol","ref":1,"prefix_hex":"fe23","suffix_hex":b"@#x".hex()}}
    renumbered={**qualified,"raw_hex":b"\xfe#13@#x".hex()}
    if comparable(qualified)!=comparable(renumbered):raise AssertionError("verified runtime IDs were not canonicalized")
    changed={**renumbered,"identity":{**qualified["identity"],"ref":2}}
    if comparable(qualified)==comparable(changed):raise AssertionError("different symbol identity was hidden")
    for left,right in (({"raw_hex":qualified["raw_hex"],"identity":None},{"raw_hex":renumbered["raw_hex"],"identity":None}),
                       ({"args_hex":[qualified["raw_hex"]]},{"args_hex":[renumbered["raw_hex"]]})):
        if comparable(left)==comparable(right):raise AssertionError("literal/diagnostic bytes were normalized")
    tested.extend(["bounded identity-number substitution","identity target retained","literal name exactness","diagnostic argument exactness"])
    import sys
    # A live child that stalls after the begin frame must hit the absolute
    # request deadline and be reaped. It cannot become a native source panic.
    child = "import json,sys,time; r=json.loads(sys.stdin.readline()); print(json.dumps(dict(version=1,id=r['id'],tag='begin',op='bind')),flush=True); time.sleep(1)"
    process=Process([sys.executable,"-c",child],directory/"deadline.stderr",deadline=0.05)
    try:
        process.send(request)
        try:list(process.observations(request))
        except TimeoutError:tested.append("absolute request deadline")
        else:raise AssertionError("stalled source request escaped its deadline")
    finally:process.close()
    tested.append("deadline child cleanup")
    if process.process.poll() is None:raise AssertionError("deadline leaked its child")
    return {"tests":len(tested),"passed":tested}
