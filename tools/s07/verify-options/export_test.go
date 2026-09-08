package compiler
import("encoding/hex";"encoding/json";"fmt";"os";"sort";"testing"
"github.com/microsoft/TypeScript/tsc/internal/bundled"
"github.com/microsoft/TypeScript/tsc/internal/tsoptions"
"github.com/microsoft/TypeScript/tsc/internal/tspath"
"github.com/microsoft/TypeScript/tsc/internal/vfs/vfstest")
type s07OptionRequest struct{s07Request;ConfigName string `json:"config_name"`;ConfigText string `json:"config_text"`}
func TestS07VerifyOptions(t *testing.T){
 raw,e:=os.ReadFile(os.Getenv("S07_VERIFY_REQUESTS"));if e!=nil{t.Fatal(e)};var requests []s07OptionRequest;if e=json.Unmarshal(raw,&requests);e!=nil{t.Fatal(e)};rows:=[]map[string]any{}
 for _,r:=range requests {files:=map[string]any{};for name,value:=range r.Files{bytes,e:=hex.DecodeString(value);if e!=nil{t.Fatal(e)};files[name]=bytes};for name,target:=range r.Symlinks{files[name]=vfstest.Symlink(target)}
 fs:=bundled.WrapFS(vfstest.FromMap(files,r.CaseSensitive));host:=NewCompilerHost(r.Cwd,fs,bundled.LibPath(),nil,nil,nil);compare:=tspath.ComparePathsOptions{CurrentDirectory:r.Cwd,UseCaseSensitiveFileNames:r.CaseSensitive};config:=tsoptions.NewParsedCommandLine(&r.Options,r.Roots,nil,compare)
 if r.ConfigName!=""{raw,e:=hex.DecodeString(r.ConfigText);if e!=nil{t.Fatal(e)};config.ConfigFile=tsoptions.NewTsconfigSourceFileFromFilePath(r.ConfigName,tspath.ToPath(r.ConfigName,r.Cwd,r.CaseSensitive),string(raw))}
 opts:=ProgramOptions{Host:host,Config:config,SkipModuleResolution:r.SkipModuleResolution};loaded:=processAllProgramFiles(opts,true);p:=&Program{opts:opts,processedFiles:loaded,comparePathsOptions:compare};before:=len(loaded.includeProcessor.processingDiagnostics)
 func(){defer func(){if value:=recover();value!=nil{t.Fatalf("%s unclassified verify panic: %T: %v",r.ID,value,value)}}();p.verifyCompilerOptions()}()
 diagnostics:=[]s07Diagnostic{};for _,diagnostic:=range p.programDiagnostics{diagnostics=append(diagnostics,s07Diag(diagnostic))}
 includes:=[]map[string]any{};for _,diagnostic:=range p.includeProcessor.processingDiagnostics[before:]{if diagnostic.kind!=processingDiagnosticKindExplainingFileInclude{t.Fatal("unexpected option include kind")};d:=diagnostic.data.(*includeExplainingDiagnostic);args:=[]string{};for _,value:=range d.args{args=append(args,fmt.Sprint(value))};includes=append(includes,map[string]any{"file":string(d.file),"code":d.message.Code(),"args":args})}
 blocked:=[]string{};for name:=range p.hasEmitBlockingDiagnostics.Keys(){blocked=append(blocked,string(name))};sort.Strings(blocked)
 rows=append(rows,map[string]any{"id":r.ID,"diagnostics":diagnostics,"includes":includes,"blocked":blocked})}
 data,e:=json.MarshalIndent(rows,"","  ");if e!=nil{t.Fatal(e)};data=append(data,'\n');if e=os.WriteFile(os.Getenv("S07_VERIFY_OUTPUT"),data,0600);e!=nil{t.Fatal(e)}
}
