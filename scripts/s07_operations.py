#!/usr/bin/env python3
"""Source-typed S07 operation dependencies, independent of Rust success outcomes."""
import argparse,collections,csv,json,os,re,sys,tempfile,tomllib
from pathlib import Path
from s07_inventory import ROOT,GO_INSPECTOR,command,upstream_pin,normalized_receiver,sha,write_or_check

PACKAGES=('compiler','tsoptions','module','packagejson','tspath','core','bundled','collections','vfs/vfsmatch','vfs/internal','json','semver','ast')
TRAVERSE_FILES={'ast': {'tsc/internal/ast/diagnostic.go'}}
SPECS=[
 dict(id='program.load', roots=['compiler/fileloader.go:processAllProgramFiles','compiler/filesparser.go:filesParser.getProcessedFiles'],supported_inputs=['Every ordered root/options/virtual-host variant selected by the frozen E2 source rule, including missing roots and recovered syntax','All bundled library/dependency closure files; libraries are not syntax-filtered','Single-thread source discovery order, minimum JS dependency depth, package identity redirects, complete include reasons'],observations=['program-observations: Files including order, text hashes, metadata, library/external flags, Imports; Missing; Resolutions; TypeResolutions; Diagnostics; Trace','run.program.subset_loads; E2,E7,E8 dependency boundary'],unsupported_calls=['Project-reference program execution','External content mapper process execution','Checker diagnostics/emission']),
 dict(id='program.option_verification',roots=['compiler/program.go:Program.verifyCompilerOptions','compiler/program.go:Program.GetProgramDiagnostics','compiler/includeprocessor.go:includeProcessor.getDiagnostics','compiler/processingDiagnostic.go:processingDiagnostic.createDiagnosticExplainingFile'],supported_inputs=['All source-selected effective compiler options, including invalid combinations retained as diagnostics','Original config AST ranges, source root set and complete inclusion reason graph'],observations=['config-options observations: option_diagnostics complete payloads and chains','Program retained OptionVerification and GetProgramDiagnostics output including source sort/equality and related-info merging'],unsupported_calls=['Project-reference builds and emit execution','External content mapper project diagnostics callbacks: collectContentMapperOptionDiagnostics requires excluded external mapper execution; static manifest/config diagnostics remain included']),
 dict(id='config.parse',roots=['tsoptions/tsconfigparsing.go:ParseJsonSourceFileConfigFileContent','tsoptions/parsinghelpers.go:ParseCompilerOptions'],supported_inputs=['Original physical config bytes, source harness directives/defaults and exact selected variants','extends packages/files, include/exclude/files wildcards, paths, rootDirs, type acquisition and invalid values','contentMappers declaration/extension/manifest validation without execution'],observations=['config-options actual output: options, root_file_names, config_raw, config_diagnostics, option_diagnostics, compile_on_save','config mapper and compact JSON direct original-Go fixtures'],unsupported_calls=['CLI build/watch execution','External mapper process execution','Raw-object ParseJsonConfigFileContent entry point: selected source harness/config and extends requests supply parsed JSON source files to ParseJsonSourceFileConfigFileContent']),
 dict(id='module.resolve',roots=['module/resolver.go:Resolver.ResolveModuleName','module/resolver.go:Resolver.ResolveTypeReferenceDirective','module/resolver.go:GetAutomaticTypeDirectiveNames','module/resolver.go:Resolver.GetPackageScopeForPath','module/util.go:GetResolutionDiagnostic'],supported_inputs=['Source Node16/NodeNext/Bundler mode and feature rules; package exports/imports/self-name, conditions, typesVersions, package IDs/peers','Relative/absolute/rootDirs/paths/moduleSuffixes; typeRoots/types; symlink/case-sensitive and insensitive immutable VFS','Successful and unsuccessful lookup with typed source trace arguments; nil/empty options/maps; malformed package JSON'],observations=['program module/type result tables and trace sequence','module-trace direct fixtures including package-directory spelling/cache identity','package-json typed fields and semver direct fixtures'],unsupported_calls=['Nonempty automatic typings acquisition cache host','Project-reference redirects','Module entrypoint enumeration for language-service auto-imports']),
 dict(id='module.config_and_mapper',roots=['module/resolver.go:ResolveConfig','tsoptions/contentmappers.go:resolveContentMapperManifest'],supported_inputs=['Fresh JSON-only NodeNext/CJS config lookup','Bundler package-directory-only mapper lookup, including @types fallback and package manifest errors'],observations=['config-resolver direct original-Go results','config-mapper original declaration, manifest and diagnostic payload fixtures'],unsupported_calls=['Running mapper exec argv']),
 dict(id='host.bundled_reads',roots=['bundled/bundled.go:WrapFS','bundled/bundled.go:LibPath','bundled/embed.go:wrappedFS.ReadFile','bundled/embed.go:wrappedFS.FileExists','bundled/embed.go:wrappedFS.DirectoryExists','bundled/embed.go:wrappedFS.GetAccessibleEntries','bundled/embed.go:wrappedFS.Realpath'],supported_inputs=['Embedded source library names/bytes; explicit immutable overlay host','ReadFile physical BOM decoding versus already-loaded bytes; FileExists, DirectoryExists, entries, realpath and case policy'],observations=['All 108 bundled source byte hashes and original program closure','VFS immutable/write rejection and retained snapshot edit tests'],unsupported_calls=['Filesystem writes/appends/removes/timestamp mutation','Live unsnapshotted OS host','Bundled executable/plugin resources']),
]
EXCLUDED_FILES={'compiler/checkerpool.go':'S08 checker construction/lease and diagnostics are not S07 loader execution','compiler/projectreferencedtsfakinghost.go':'Project-reference execution is excluded by the source feature rule','compiler/projectreferencefilemapper.go':'Project-reference mappings are absent under the source feature rule; empty mapping accessors are represented by the no-reference host domain','compiler/projectreferenceparser.go':'Project-reference execution is excluded; config references are still validated','tsoptions/showconfig.go':'showConfig command output is not used by the source-selected loader','tsoptions/parsedbuildcommandline.go':'Build command execution is outside the source-selected config boundary'}
EXCLUDED_IDS={'compiler/program.go:Program.initCheckerPool':'Checker pool construction belongs to S08','compiler/program.go:Program.collectContentMapperOptionDiagnostics':'External mapper project execution is excluded; static declaration/manifest validation remains required','compiler/fileloader.go:fileLoader.addProjectReferenceTasks':'No project-reference execution in frozen eligible domain','compiler/fileloader.go:fileLoader.parseContentMappedFile':'External mapper process execution is outside S07','module/resolver.go:Resolver.tryResolveFromTypingsLocation':'No automatic typings acquisition cache location is supplied in the selected source host'}



def function_references(inspector):
    """Named callback values are dependencies even without a direct call node."""
    inspector = inspector.replace('Interface bool `json:"interface"`', 'Interface bool `json:"interface"`\n Reference bool `json:"reference"`')
    before = r"""
            directFunctions := map[*ast.Ident]bool{}
            ast.Inspect(body, func(node ast.Node) bool {
                call, ok := node.(*ast.CallExpr)
                if !ok { return true }
                expr := call.Fun
                if x, ok := expr.(*ast.IndexExpr); ok { expr = x.X }
                if x, ok := expr.(*ast.IndexListExpr); ok { expr = x.X }
                switch x := expr.(type) {
                case *ast.Ident: directFunctions[x] = true
                case *ast.SelectorExpr: directFunctions[x.Sel] = true
                }
                return true
            })
"""
    inspector = inspector.replace('\t\t\tast.Inspect(body, func(n ast.Node) bool {', before + '\t\t\tast.Inspect(body, func(n ast.Node) bool {')
    references = r"""
                if id, ok := n.(*ast.Ident); ok && !directFunctions[id] {
                    if f, ok := info.Uses[id].(*types.Func); ok {
                        pkg, rec := "", ""
                        isInterface := false
                        if f.Pkg() != nil { pkg = f.Pkg().Path() }
                        if r := f.Type().(*types.Signature).Recv(); r != nil {
                            _, isInterface = r.Type().Underlying().(*types.Interface)
                            rec = types.TypeString(r.Type(), func(*types.Package) string { return "" })
                        }
                        calls = append(calls, Call{Anchor: anchor(id), Package: pkg, Receiver: rec,
                            Interface: isInterface, Reference: true, Name: f.Name(), Expression: render(id)})
                    }
                }
"""
    return inspector.replace('\t\t\t\tif call, ok := n.(*ast.CallExpr); ok {', references + '\t\t\t\tif call, ok := n.(*ast.CallExpr); ok {')

def create():
    pin,version=upstream_pin()
    ledger={row['go']:row for row in tomllib.loads((ROOT/'PORTS.toml').read_text())['file']}
    rows=list(csv.DictReader((line for line in (ROOT/'data/go-functions.tsv').read_text().splitlines() if not line.startswith('#')),delimiter='\t'))
    by_id={row['id']:row for row in rows};by_name=collections.defaultdict(list);by_file=collections.defaultdict(list)
    for row in rows:by_name[(row['package'],row['receiver'],row['name'])].append(row);by_file[row['file']].append(row)
    observations=[];active_files=set()
    with tempfile.TemporaryDirectory(prefix='s07-operation-inventory-') as temp:
        temp=Path(temp)
        (temp/'packages.json').write_bytes(command(['go','list','-export','-deps','-json',*['./internal/'+pkg for pkg in PACKAGES]],ROOT/'upstream/tsc'))
        packages=(temp/'packages.json').read_text(); decoder=json.JSONDecoder()
        while packages.strip():
            package,end=decoder.raw_decode(packages.lstrip());packages=packages.lstrip()[end:]
            prefix=package['ImportPath'].removeprefix('github.com/microsoft/TypeScript/tsc/')
            active_files.update('tsc/'+prefix+'/'+name for name in package.get('GoFiles',[]))
        inspector=GO_INSPECTOR.replace('strings.HasSuffix(p.ImportPath, "/internal/binder")','strings.HasSuffix(p.ImportPath, "/internal/"+os.Args[2])').replace('"tsc/internal/binder/" + filepath.Base(a.Filename)','"tsc/internal/"+os.Args[2]+"/" + filepath.Base(a.Filename)')
        inspector=inspector.replace('Dynamic    bool   `json:"dynamic"`','Dynamic    bool   `json:"dynamic"`\n Interface bool `json:"interface"`').replace('rec := ""','rec := ""\n isInterface := false').replace('if r := f.Type().(*types.Signature).Recv(); r != nil {','if r := f.Type().(*types.Signature).Recv(); r != nil {\n _, isInterface = r.Type().Underlying().(*types.Interface)').replace('Package: pkg, Receiver: rec','Package: pkg, Interface: isInterface, Receiver: rec')
        inspector = function_references(inspector)
        (temp/'inspect.go').write_text(inspector)
        for pkg in PACKAGES:
            print('source package',pkg,file=sys.stderr,flush=True)
            observed=json.loads(command(['go','run',str(temp/'inspect.go'),str(temp/'packages.json'),pkg],ROOT/'upstream/tsc'))
            observations.extend(observed['calls'])
    graph=collections.defaultdict(list);initializers=[]
    for call in observations:
        owners=[row for row in by_file[call['file']] if int(row['start'])<=call['line']<=int(row['end'])]
        if len(owners)>1:raise ValueError('ambiguous caller source anchor')
        call['caller']=owners[0]['id'] if owners else None
        package=call['package'].removeprefix('github.com/microsoft/TypeScript/tsc/')
        candidates=[row for row in by_name[(package,normalized_receiver(call['receiver']),call['name'])] if row['file'] in active_files]
        if len(candidates)>1:raise ValueError('ambiguous callee: '+str(call))
        call['callee']=candidates[0]['id'] if candidates else None
        if call['caller'] is None:initializers.append(call)
        else:graph[call['caller']].append(call)
    def excluded(identifier):
        short=identifier.removeprefix('tsc/internal/');file=short.split(':')[0]
        return EXCLUDED_IDS.get(short) or EXCLUDED_FILES.get(file)
    operations=[];required=set();exclusions={};calls=[];boundaries=[];dynamic=[];unresolved=[]
    for spec in SPECS:
        roots=['tsc/internal/'+identifier for identifier in spec['roots']]
        for root in roots:
            if root not in by_id:raise ValueError('unknown operation root: '+root)
        seen=set();pending=roots[:];omitted={}
        while pending:
            identifier=pending.pop()
            if identifier in seen:continue
            why=excluded(identifier)
            if why:omitted[identifier]=why;continue
            seen.add(identifier)
            for call in graph[identifier]:
                call={**call,'operation':spec['id']};callee=call['callee']
                if call['dynamic']:
                    dynamic.append(call);continue
                if callee:
                    calls.append(call)
                    package = by_id[callee]['package'].removeprefix('internal/')
                    if package in PACKAGES and (package not in TRAVERSE_FILES or by_id[callee]['file'] in TRAVERSE_FILES[package]):
                        pending.append(callee)
                    else:boundaries.append({**call,'boundary':'existing source API package outside operation traversal'})
                else:
                    if call['package'].startswith('github.com/microsoft/TypeScript/tsc/') and not call['interface']:
                        unresolved.append(call)
                    boundary = 'interface dispatch through operation host or retained value' if call['interface'] else 'standard library or external dependency'
                    boundaries.append({**call,'boundary':boundary})
        required |= seen;exclusions.update(omitted)
        operations.append({key:value for key,value in spec.items() if key!='roots'}|{'source_roots':roots,'required_source_functions':sorted(i for i in seen if ledger[by_id[i]['file']]['kind']=='source'),'required_generated_functions':sorted(i for i in seen if ledger[by_id[i]['file']]['kind']=='generated'),'excluded_source_functions':[{'id':key,'reason':value} for key,value in sorted(omitted.items())]})
    mappings=collections.defaultdict(list)
    for path in sorted((ROOT/'crates').rglob('*.rs')):
        for line,text in enumerate(path.read_text().splitlines(),1):
            for identifier in re.findall(r'\bport:\s+(tsc/\S+)',text):mappings[identifier].append({'file':str(path.relative_to(ROOT)),'line':line})
    functions={}
    for identifier in sorted(required|exclusions.keys()):
        row=by_id[identifier];functions[identifier]={'file':row['file'],'start_line':int(row['start']),'end_line':int(row['end']),'kind':ledger.get(row['file'],{}).get('kind','unknown'),'source_sha256':sha((ROOT/'upstream'/row['file']).read_bytes()),'rust_mappings':mappings[identifier],'mapping_status':'source_marker_present' if mappings[identifier] else 'no_source_marker'}
    for call in initializers:
        if not call['dynamic'] and call['callee'] is None and call['package'].startswith('github.com/microsoft/TypeScript/tsc/') and not call['interface']:
            unresolved.append(call)
    inputs=['scripts/s07_operations.py','scripts/s07_inventory.py','data/go-functions.tsv','data/s04/toolchains.toml']
    source_files=sorted(path for path in active_files if path.startswith('tsc/internal/') and path.removeprefix('tsc/internal/').rsplit('/',1)[0] in PACKAGES)
    return {'schema':1,'generator_inputs':{path:sha((ROOT/path).read_bytes()) for path in inputs},'source_files':{path:sha((ROOT/'upstream'/path).read_bytes()) for path in source_files},'audited_packages':list(PACKAGES),'traversal_file_limits':{key:sorted(value) for key,value in TRAVERSE_FILES.items()},'package_initializer_calls':initializers,'construction_boundary_policy':'Package initializers and their anonymous function bodies are inventoried separately with exact typed call anchors. They are not attributed to an operation function or counted as SOURCE coverage. Source tables/cached table builders are required data dependencies; the reviewer must assess these constructor boundaries alongside dynamic operation callbacks.','upstream_pin':pin,'go_toolchain':version,'selection':'Required operation roots come from pinned source entry points and the reviewed S07 domain. Rust results never select functions or requests. Static calls and named function references are Go type-resolved; other function-value callbacks and external interfaces remain explicit boundaries. This is not a function parity metric.','operations':operations,'functions':functions,'transitive_calls':calls,'external_boundaries':boundaries,'dynamic_calls':dynamic,'unresolved_calls':unresolved, 'dynamic_boundary_policy':'Function-value calls are not treated as statically resolved. Host methods are covered by immutable VFS fixtures; collection predicates/comparators/visitors execute through their enclosing required source function. Checker/content-mapper/project-reference callback domains remain explicitly excluded.'}

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--write',action='store_true');args=parser.parse_args()
    document=create();write_or_check(ROOT/'data/s07/operations.json',(json.dumps(document,indent=2,sort_keys=True)+'\n').encode(),args.write)
    print(json.dumps({'operations':len(document['operations']),'functions':len(document['functions']),'calls':len(document['transitive_calls']),'dynamic_calls':len(document['dynamic_calls'])}))
if __name__=='__main__':main()
