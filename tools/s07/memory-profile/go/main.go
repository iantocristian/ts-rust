// Diagnostic memory adapter, built inside an exact pinned Go source export.
// Profiles/census/checkpoint pauses deliberately perturb execution and cannot
// replace the ordinary allocation, timing, or lifetime-RSS acceptance samples.
package main

import (
	"bufio"
	"crypto/sha256"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"runtime/pprof"
	"sort"
	"strconv"
	"sync"
	"time"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/binder"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
)

// The pinned default is 512 KiB. This diagnostic uses 64 KiB to improve
// attribution of smaller sites while retaining statistical sampling; all
// pipeline work happens after this single early rate assignment.
const memoryProfileRate = 64 * 1024

func init() { runtime.MemProfileRate = memoryProfileRate }

type Input struct {
	Filename   string `json:"filename"`
	Path       string `json:"path"`
	Local      string `json:"local"`
	ScriptKind int32  `json:"script_kind"`
	JSX        bool   `json:"jsx"`
	Force      bool   `json:"force"`
}

type Loaded struct {
	text    string
	options ast.SourceFileParseOptions
	kind    core.ScriptKind
}

type Snapshot struct {
	Name       string           `json:"name"`
	UnixTimeNs int64            `json:"unix_time_ns"`
	Stats      runtime.MemStats `json:"mem_stats"`
}

type Profile struct {
	Name         string   `json:"name"`
	HeapPath     string   `json:"heap_path"`
	HeapSHA256   string   `json:"heap_sha256"`
	AllocsPath   string   `json:"allocs_path"`
	AllocsSHA256 string   `json:"allocs_sha256"`
	Before       Snapshot `json:"before"`
	After        Snapshot `json:"after"`
	WallTimeNs   int64    `json:"wall_time_ns"`
}

type Worker struct {
	Worker     int   `json:"worker"`
	Files      int   `json:"files"`
	ParseCalls int   `json:"parse_calls"`
	BindCalls  int   `json:"bind_calls"`
	ParseNs    int64 `json:"parse_ns"`
	BindNs     int64 `json:"bind_ns"`
}

type Census struct {
	Files                        int                  `json:"files"`
	MaximumFilePointerIdentities uint64               `json:"maximum_file_pointer_identities"`
	Categories                   ast.MemoryFileCensus `json:"categories"`
	Before                       Snapshot             `json:"before"`
	After                        Snapshot             `json:"after"`
	WallTimeNs                   int64                `json:"wall_time_ns"`
}

type Report struct {
	Version           int        `json:"version"`
	Operation         string     `json:"operation"`
	DiagnosticOnly    bool       `json:"diagnostic_only"`
	MemProfileRate    int        `json:"mem_profile_rate"`
	LoadedInputSHA256 string     `json:"loaded_input_sha256"`
	Workers           int        `json:"workers"`
	Files             int        `json:"files"`
	LoadedBytes       int        `json:"loaded_bytes"`
	Nodes             int        `json:"nodes"`
	Symbols           int        `json:"symbols"`
	ParseDiagnostics  int        `json:"parse_diagnostics"`
	BindDiagnostics   int        `json:"bind_diagnostics"`
	WallTimeNs        int64      `json:"wall_time_ns"`
	AllocatedBytes    uint64     `json:"allocated_bytes"`
	WorkerTimes       []Worker   `json:"worker_times"`
	GOMAXPROCS        int        `json:"gomaxprocs"`
	GOGC              int        `json:"gogc"`
	CPUCapacity       int        `json:"cpu_capacity"`
	GoroutinesReady   int        `json:"goroutines_ready"`
	PipelineBefore    Snapshot   `json:"pipeline_before"`
	Snapshots         []Snapshot `json:"snapshots"`
	Profiles          []Profile  `json:"profiles"`
	Census            Census     `json:"census"`
	Limitations       []string   `json:"limitations"`
}

func loadedDigest(inputs []Loaded) string {
	digest := sha256.New()
	digest.Write([]byte("S07-loaded-inputs-v1\x00"))
	word := make([]byte, 8)
	binary.BigEndian.PutUint64(word, uint64(len(inputs)))
	digest.Write(word)
	for _, input := range inputs {
		for _, value := range []string{input.options.FileName, string(input.options.Path)} {
			binary.BigEndian.PutUint64(word, uint64(len(value)))
			digest.Write(word)
			digest.Write([]byte(value))
		}
		binary.BigEndian.PutUint32(word[:4], uint32(input.kind))
		digest.Write(word[:4])
		bits := []byte{0, 0}
		if input.options.ExternalModuleIndicatorOptions.JSX {
			bits[0] = 1
		}
		if input.options.ExternalModuleIndicatorOptions.Force {
			bits[1] = 1
		}
		digest.Write(bits)
		binary.BigEndian.PutUint64(word, uint64(len(input.text)))
		digest.Write(word)
		source := sha256.Sum256([]byte(input.text))
		digest.Write(source[:])
	}
	return fmt.Sprintf("%x", digest.Sum(nil))
}

func snapshot(name string) Snapshot {
	result := Snapshot{Name: name, UnixTimeNs: time.Now().UnixNano()}
	runtime.ReadMemStats(&result.Stats)
	return result
}

func checkpoint(value Snapshot, reader *bufio.Reader, writer *bufio.Writer) {
	if err := json.NewEncoder(writer).Encode(struct {
		Checkpoint string   `json:"checkpoint"`
		Snapshot   Snapshot `json:"snapshot"`
	}{value.Name, value}); err != nil {
		panic(err)
	}
	if err := writer.Flush(); err != nil {
		panic(err)
	}
	line, err := reader.ReadString('\n')
	if err != nil {
		panic(fmt.Sprintf("checkpoint %s resume failed: %v", value.Name, err))
	}
	if line != "\n" && line != "\r\n" {
		panic("checkpoint resume must be one empty line")
	}
}

func fileDigest(path string) string {
	file, err := os.Open(path)
	if err != nil {
		panic(err)
	}
	defer file.Close()
	digest := sha256.New()
	if _, err := io.Copy(digest, file); err != nil {
		panic(err)
	}
	return fmt.Sprintf("%x", digest.Sum(nil))
}

func profiles(prefix, name string) Profile {
	result := Profile{Name: name, HeapPath: prefix + "-" + name + ".heap.pprof",
		AllocsPath: prefix + "-" + name + ".allocs.pprof", Before: snapshot(name + "_profile_before")}
	started := time.Now()
	for _, output := range []struct{ kind, path string }{{"heap", result.HeapPath}, {"allocs", result.AllocsPath}} {
		file, err := os.Create(output.path)
		if err != nil {
			panic(err)
		}
		profile := pprof.Lookup(output.kind)
		if profile == nil {
			panic("missing runtime memory profile: " + output.kind)
		}
		if err := profile.WriteTo(file, 0); err != nil {
			file.Close()
			panic(err)
		}
		if err := file.Close(); err != nil {
			panic(err)
		}
	}
	result.HeapSHA256 = fileDigest(result.HeapPath)
	result.AllocsSHA256 = fileDigest(result.AllocsPath)
	result.WallTimeNs = time.Since(started).Nanoseconds()
	result.After = snapshot(name + "_profile_after")
	return result
}

func main() {
	if len(os.Args) != 4 {
		panic("usage: memory-profile INPUTS.json WORKERS (1 or 8) OUTPUT_PREFIX")
	}
	workers, err := strconv.Atoi(os.Args[2])
	if err != nil || workers != 1 && workers != 8 {
		panic("worker count must be 1 or 8")
	}
	if os.Args[3] == "" {
		panic("empty output prefix")
	}
	file, err := os.Open(os.Args[1])
	if err != nil {
		panic(err)
	}
	decoder := json.NewDecoder(file)
	decoder.DisallowUnknownFields()
	var requests []Input
	if err := decoder.Decode(&requests); err != nil {
		panic(err)
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		panic("workload must contain exactly one JSON value")
	}
	if err := file.Close(); err != nil {
		panic(err)
	}
	if len(requests) == 0 {
		panic("empty parse-and-bind workload")
	}
	runtime.GOMAXPROCS(workers)
	debug.SetGCPercent(100)
	inputs := make([]Loaded, len(requests))
	loadedBytes := 0
	for index, request := range requests {
		text, ok := osvfs.FS().ReadFile(request.Local)
		if !ok {
			panic("could not load workload file: " + request.Local)
		}
		inputs[index] = Loaded{text, ast.SourceFileParseOptions{FileName: request.Filename, Path: tspath.Path(request.Path),
			ExternalModuleIndicatorOptions: ast.ExternalModuleIndicatorOptions{JSX: request.JSX, Force: request.Force}}, core.ScriptKind(request.ScriptKind)}
		loadedBytes += len(text)
	}
	identity := loadedDigest(inputs)
	prefix, err := filepath.Abs(os.Args[3])
	if err != nil {
		panic(err)
	}
	if err := os.MkdirAll(filepath.Dir(prefix), 0o755); err != nil {
		panic(err)
	}
	output, err := os.Create(prefix + "-report.json")
	if err != nil {
		panic(err)
	}
	defer output.Close()
	reader, writer := bufio.NewReader(os.Stdin), bufio.NewWriterSize(os.Stdout, 64*1024)
	channels := make([]chan int, workers)
	results := make([][]*ast.SourceFile, workers)
	statistics := make([]Worker, workers)
	failures := make([]any, workers)
	ready, done := sync.WaitGroup{}, sync.WaitGroup{}
	ready.Add(workers)
	done.Add(workers)
	start := make(chan struct{})
	for worker := range workers {
		channels[worker] = make(chan int, workers)
		results[worker] = make([]*ast.SourceFile, 0, (len(inputs)+workers-1)/workers)
		statistics[worker].Worker = worker
		go func(worker int) {
			defer done.Done()
			ready.Done()
			<-start
			for index := range channels[worker] {
				if failures[worker] != nil {
					continue
				}
				func() {
					complete := false
					defer func() {
						failure := recover()
						if !complete {
							failures[worker] = fmt.Sprintf("file %s: %v\n%s", inputs[index].options.FileName, failure, debug.Stack())
						}
					}()
					input := inputs[index]
					started := time.Now()
					file := parser.ParseSourceFile(input.options, input.text, input.kind)
					statistics[worker].ParseNs += time.Since(started).Nanoseconds()
					statistics[worker].ParseCalls++
					started = time.Now()
					binder.BindSourceFile(file)
					statistics[worker].BindNs += time.Since(started).Nanoseconds()
					statistics[worker].BindCalls++
					results[worker] = append(results[worker], file)
					statistics[worker].Files++
					complete = true
				}()
			}
		}(worker)
	}
	ready.Wait()
	report := Report{Version: 1, Operation: "parse_bind_memory_profile", DiagnosticOnly: true,
		MemProfileRate: memoryProfileRate, LoadedInputSHA256: identity, Workers: workers, LoadedBytes: loadedBytes,
		GOMAXPROCS: runtime.GOMAXPROCS(0), GOGC: 100, CPUCapacity: runtime.NumCPU(), GoroutinesReady: runtime.NumGoroutine(),
		WorkerTimes: statistics, Snapshots: make([]Snapshot, 0, 5), Profiles: make([]Profile, 0, 4)}
	report.Profiles = append(report.Profiles, profiles(prefix, "pre_pipeline"))
	before := snapshot("pre_pipeline")
	report.Snapshots = append(report.Snapshots, before)
	checkpoint(before, reader, writer)
	// Reset after all pre-pipeline profile/checkpoint serialization. Exact
	// TotalAlloc delta therefore excludes that diagnostic preparation.
	report.PipelineBefore = snapshot("pipeline_before_barrier")
	started := time.Now()
	close(start)
	for index := range inputs {
		channels[index%workers] <- index
	}
	for _, channel := range channels {
		close(channel)
	}
	done.Wait()
	report.WallTimeNs = time.Since(started).Nanoseconds()
	endpoint := snapshot("retained_endpoint")
	report.Snapshots = append(report.Snapshots, endpoint)
	checkpoint(endpoint, reader, writer)
	for worker, files := range results {
		if failures[worker] != nil {
			panic(failures[worker])
		}
		expected := len(inputs) / workers
		if worker < len(inputs)%workers {
			expected++
		}
		stats := statistics[worker]
		if len(files) != expected || stats.Files != expected || stats.ParseCalls != expected || stats.BindCalls != expected {
			panic(fmt.Sprintf("worker %d workload count mismatch", worker))
		}
		for _, file := range files {
			report.Files++
			report.Nodes += file.NodeCount
			report.Symbols += file.SymbolCount
			report.ParseDiagnostics += len(file.Diagnostics())
			report.BindDiagnostics += len(file.BindDiagnostics())
		}
	}
	if report.Files != len(inputs) || endpoint.Stats.TotalAlloc < report.PipelineBefore.Stats.TotalAlloc {
		panic("workload/accounting mismatch")
	}
	report.AllocatedBytes = endpoint.Stats.TotalAlloc - report.PipelineBefore.Stats.TotalAlloc
	report.Profiles = append(report.Profiles, profiles(prefix, "retained_endpoint"))
	// The pinned runtime documents memory profiles as up to two GC cycles old.
	// These deliberate cycles are separate from the ordinary retained endpoint.
	runtime.GC()
	runtime.GC()
	retainedGC := snapshot("retained_after_gc")
	report.Snapshots = append(report.Snapshots, retainedGC)
	checkpoint(retainedGC, reader, writer)
	report.Profiles = append(report.Profiles, profiles(prefix, "retained_after_gc"))
	report.Census = census(results)
	runtime.KeepAlive(inputs)
	runtime.KeepAlive(results)
	inputs, results, requests, channels, failures = nil, nil, nil, nil, nil
	retired := snapshot("post_retirement")
	report.Snapshots = append(report.Snapshots, retired)
	checkpoint(retired, reader, writer)
	runtime.GC()
	runtime.GC()
	retiredGC := snapshot("post_retirement_after_gc")
	report.Snapshots = append(report.Snapshots, retiredGC)
	checkpoint(retiredGC, reader, writer)
	report.Profiles = append(report.Profiles, profiles(prefix, "post_retirement_after_gc"))
	report.Limitations = []string{
		"Diagnostic only: 64KiB sampled heap/alloc profiles, checkpoint pauses and explicit GCs perturb execution; not acceptance allocation/RSS/time measurements.",
		"Exact pipeline allocated_bytes is MemStats.TotalAlloc delta after pre-pipeline profile/checkpoint overhead and before endpoint serialization. Sampled alloc_space estimates use allocation stacks and may differ; no forced reconciliation.",
		"Heap and allocs profiles contain the same four sample types with different defaults. Ordinary endpoint profiles can lag up to two GC cycles. The separate retained_after_gc profiles follow two requested GC cycles while roots remain live.",
		"Checkpoint MemStats precedes its own JSON serialization and pause; parent OS snapshots occur afterward. Snapshot/profile/checkpoint bookkeeping remains visible in whole-process heap data.",
		"Census runs after retained heap snapshots. Its measured allocations and temporary pointer maps must not be attributed to compiler work or interpreted as compiler peak RSS.",
		"Census counts reachable logical objects, including metadata and synthetic flow nodes, not all historically allocated factory nodes/symbols or every element kept alive by an arena backing array.",
		"Concrete Go AST object sizes include the embedded Node header and binding fields. node_header_contained_bytes is a contained subcategory and must not be added to concrete object bytes.",
		"Slice capacity ranges are unioned within each file and element type. Arena backing slack outside visible slices, map buckets/control bytes/spare capacity, size-class rounding and allocator/GC metadata remain unmeasured by the census.",
		"String counts include descriptors' selected bytes, not necessarily separate allocations. Source-backed aliases and per-file visible range unions are reported; other ranges include read-only/global strings and are not a heap-byte subtotal. Shared globals can recur across files.",
		"Post-retirement snapshots release compiler roots and preload buffers; they still include runtime/profiler/report metadata and allocator/OS footprint caused by prior census scratch allocations. The first snapshot requests no GC; the second follows two explicit GCs, which do not require immediate OS page release. Automatic collections may also occur.",
	}
	if err := json.NewEncoder(output).Encode(report); err != nil {
		panic(err)
	}
	if err := output.Close(); err != nil {
		panic(err)
	}
	if err := json.NewEncoder(writer).Encode(map[string]any{"complete": true, "diagnostic_only": true,
		"report": prefix + "-report.json", "report_sha256": fileDigest(prefix + "-report.json"),
		"files": report.Files, "loaded_input_sha256": identity}); err != nil {
		panic(err)
	}
	if err := writer.Flush(); err != nil {
		panic(err)
	}
}

func census(files [][]*ast.SourceFile) Census {
	result := Census{Before: snapshot("census_before")}
	started := time.Now()
	objects := make(map[string]ast.MemoryTypeCensus)
	slices := make(map[string]ast.MemorySliceCensus)
	maps := make(map[string]ast.MemoryMapCensus)
	result.Categories.NodeKinds = make(map[string]uint64)
	result.Categories.SkippedExternalPointerFields = make(map[string]uint64)
	for _, group := range files {
		for _, file := range group {
			current := ast.MemoryProfileCensus(file)
			result.Files++
			result.MaximumFilePointerIdentities = max(result.MaximumFilePointerIdentities, current.PointerIdentities)
			result.Categories.PointerIdentities += current.PointerIdentities
			result.Categories.SourceTextBytes += current.SourceTextBytes
			result.Categories.NodeHeaderContainedBytes += current.NodeHeaderContainedBytes
			for name, count := range current.NodeKinds {
				result.Categories.NodeKinds[name] += count
			}
			for name, count := range current.SkippedExternalPointerFields {
				result.Categories.SkippedExternalPointerFields[name] += count
			}
			for _, item := range current.Objects {
				old, exists := objects[item.Type]
				if exists && (old.Size != item.Size || old.Category != item.Category) {
					panic("census type layout changed")
				}
				item.Count += old.Count
				item.LogicalBytes += old.LogicalBytes
				objects[item.Type] = item
			}
			for _, item := range current.Slices {
				old := slices[item.Type]
				item.Headers += old.Headers
				item.NilHeaders += old.NilHeaders
				item.LengthElements += old.LengthElements
				item.CapacityElements += old.CapacityElements
				item.VisibleCapacityUnion += old.VisibleCapacityUnion
				slices[item.Type] = item
			}
			for _, item := range current.Maps {
				old := maps[item.Type]
				item.Maps += old.Maps
				item.Entries += old.Entries
				item.EntryLogicalBytes += old.EntryLogicalBytes
				maps[item.Type] = item
			}
			strings := &result.Categories.Strings
			strings.Fields += current.Strings.Fields
			strings.EmptyFields += current.Strings.EmptyFields
			strings.LogicalBytes += current.Strings.LogicalBytes
			strings.SourceBackedFields += current.Strings.SourceBackedFields
			strings.SourceBackedLogicalBytes += current.Strings.SourceBackedLogicalBytes
			strings.SourceVisibleUnionBytes += current.Strings.SourceVisibleUnionBytes
			strings.OtherVisibleUnionBytes += current.Strings.OtherVisibleUnionBytes
		}
	}
	for _, item := range objects {
		result.Categories.Objects = append(result.Categories.Objects, item)
	}
	for _, item := range slices {
		result.Categories.Slices = append(result.Categories.Slices, item)
	}
	for _, item := range maps {
		result.Categories.Maps = append(result.Categories.Maps, item)
	}
	sort.Slice(result.Categories.Objects, func(i, j int) bool { return result.Categories.Objects[i].Type < result.Categories.Objects[j].Type })
	sort.Slice(result.Categories.Slices, func(i, j int) bool { return result.Categories.Slices[i].Type < result.Categories.Slices[j].Type })
	sort.Slice(result.Categories.Maps, func(i, j int) bool { return result.Categories.Maps[i].Type < result.Categories.Maps[j].Type })
	result.WallTimeNs = time.Since(started).Nanoseconds()
	result.After = snapshot("census_after")
	return result
}
