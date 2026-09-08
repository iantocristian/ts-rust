package tspath
import("encoding/hex";"encoding/json";"fmt";"os";"strings";"testing")
type s07Request struct {ID int `json:"id"`; A string `json:"a"`;B string `json:"b"`;Cwd string `json:"cwd"`;Sensitive bool `json:"sensitive"`}
func TestS07Paths(t *testing.T){
 raw,e:=os.ReadFile(os.Getenv("S07_PATH_REQUESTS"));if e!=nil{t.Fatal(e)};var requests []s07Request;if e=json.Unmarshal(raw,&requests);e!=nil{t.Fatal(e)}
 rows:=[]map[string]any{}
 decode:=func(s string)string{v,e:=hex.DecodeString(s);if e!=nil{t.Fatal(e)};return string(v)}
 text:=func(f func()string)(result any){defer func(){if p:=recover();p!=nil {message:=fmt.Sprint(p);if message!="paths must either both be absolute or both be relative"{t.Fatalf("unclassified path panic: %T: %v",p,p)};result=map[string]any{"panic":message}}}();return map[string]any{"text":hex.EncodeToString([]byte(f()))}}
 for _,r:=range requests{a,b,cwd:=decode(r.A),decode(r.B),decode(r.Cwd);o:=ComparePathsOptions{CurrentDirectory:cwd,UseCaseSensitiveFileNames:r.Sensitive};trim,ok:=TrimFilePathPrefix(a,b,r.Sensitive);components:=GetNormalizedPathComponents(a,cwd);parts:=[]string{};for _,part:=range components{parts=append(parts,hex.EncodeToString([]byte(part)))}
 rows=append(rows,map[string]any{"id":r.ID,"fold":strings.EqualFold(a,b),"relative":PathIsRelative(a),"compare":ComparePaths(a,b,o),"contains":ContainsPath(a,b,o),"trim":[]any{hex.EncodeToString([]byte(trim)),ok},"components":parts,"reconstruct":hex.EncodeToString([]byte(GetPathFromPathComponents(components))),"directory":text(func()string{return GetRelativePathFromDirectory(a,b,o)}),"file":text(func()string{return GetRelativePathFromFile(a,b,o)})})}
 data,e:=json.MarshalIndent(rows,"","  ");if e!=nil{t.Fatal(e)};data=append(data,'\n');if e=os.WriteFile(os.Getenv("S07_PATH_OUTPUT"),data,0600);e!=nil{t.Fatal(e)}
}
