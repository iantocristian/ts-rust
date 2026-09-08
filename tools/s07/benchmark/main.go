// Native source-tree parser/binder measurement adapter. The compiler behavior
// comes entirely from the fresh pinned source export containing this command.
package main

import (
	"crypto/sha256"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"runtime/debug"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/binder"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/microsoft/TypeScript/tsc/internal/vfs/osvfs"
)

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
type Report struct {
	LoadedInputSHA256 string `json:"loaded_input_sha256"`
	Version           int    `json:"version"`
	Workers           int    `json:"workers"`
	Files             int    `json:"files"`
	LoadedBytes       int    `json:"loaded_bytes"`
	Nodes             int    `json:"nodes"`
	Symbols           int    `json:"symbols"`
	ParseDiagnostics  int    `json:"parse_diagnostics"`
	BindDiagnostics   int    `json:"bind_diagnostics"`
	WallTimeNs        int64  `json:"wall_time_ns"`
	AllocatedBytes    uint64 `json:"allocated_bytes"`
	StartupNs         int64  `json:"startup_ns"`
	PreloadNs         int64  `json:"preload_ns"`
	WorkerSetupNs     int64  `json:"worker_setup_ns"`
	CPUCapacity       int    `json:"cpu_capacity"`
	GoroutinesReady   int    `json:"goroutines_ready"`
	GOMAXPROCS        int    `json:"gomaxprocs"`
	GOGC              int    `json:"gogc"`
}

// The digest observes the exact immutable preload, outside the measured phase.
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

func main() {
	entered := time.Now()
	graphIndex := -1
	if len(os.Args) == 4 && strings.HasPrefix(os.Args[3], "--graph-records=") {
		var err error
		graphIndex, err = strconv.Atoi(strings.TrimPrefix(os.Args[3], "--graph-records="))
		if err != nil || graphIndex < 0 {
			panic("invalid graph index")
		}
	}
	graphMode := len(os.Args) == 4 && (os.Args[3] == "--graphs" || graphIndex >= 0)
	if len(os.Args) != 3 && !graphMode {
		panic("usage: benchmark INPUTS.json WORKERS (1 or 8) [--graphs | --graph-records=INDEX], or options")
	}
	input, err := os.Open(os.Args[1])
	if err != nil {
		panic(err)
	}
	decoder := json.NewDecoder(input)
	decoder.DisallowUnknownFields()
	var requests []Input
	if err := decoder.Decode(&requests); err != nil {
		panic(err)
	}
	input.Close()
	if graphIndex >= len(requests) {
		panic("graph index outside workload")
	}
	if len(requests) == 0 {
		panic("empty parse-and-bind workload")
	}
	if os.Args[2] == "options" {
		options := &core.CompilerOptions{Target: core.ScriptTargetESNext}
		for index := range requests {
			request := &requests[index]
			indicator := ast.GetExternalModuleIndicatorOptions(request.Filename, options, ast.SourceFileMetaData{})
			request.ScriptKind = int32(core.GetScriptKindFromFileName(request.Filename))
			request.JSX = indicator.JSX
			request.Force = indicator.Force
		}
		if err := json.NewEncoder(os.Stdout).Encode(requests); err != nil {
			panic(err)
		}
		return
	}
	workers, err := strconv.Atoi(os.Args[2])
	if err != nil || workers != 1 && workers != 8 {
		panic("worker count must be 1 or 8")
	}
	runtime.GOMAXPROCS(workers)
	debug.SetGCPercent(100)
	startupNs := time.Since(entered).Nanoseconds()
	preloadStarted := time.Now()
	inputs := make([]Loaded, len(requests))
	loadedBytes := 0
	for index, request := range requests {
		text, ok := osvfs.FS().ReadFile(request.Local)
		if !ok {
			panic("could not load workload file: " + request.Local)
		}
		inputs[index] = Loaded{text, ast.SourceFileParseOptions{FileName: request.Filename, Path: tspath.Path(request.Path), ExternalModuleIndicatorOptions: ast.ExternalModuleIndicatorOptions{JSX: request.JSX, Force: request.Force}}, core.ScriptKind(request.ScriptKind)}
		loadedBytes += len(text)
	}
	loadedInputSHA256 := loadedDigest(inputs)
	preloadNs := time.Since(preloadStarted).Nanoseconds()
	workerSetupStarted := time.Now()
	channels := make([]chan int, workers)
	results := make([][]*ast.SourceFile, workers)
	failures := make([]any, workers)
	ready := sync.WaitGroup{}
	ready.Add(workers)
	done := sync.WaitGroup{}
	done.Add(workers)
	start := make(chan struct{})
	for worker := range workers {
		channels[worker] = make(chan int, workers)
		results[worker] = make([]*ast.SourceFile, 0, (len(inputs)+workers-1)/workers)
		go func(worker int) {
			ready.Done()
			<-start
			for index := range channels[worker] {
				if failures[worker] != nil {
					continue
				}
				func() {
					defer func() {
						if failure := recover(); failure != nil {
							failures[worker] = fmt.Sprintf("file %s: %v\n%s", inputs[index].options.FileName, failure, debug.Stack())
						}
					}()
					input := inputs[index]
					file := parser.ParseSourceFile(input.options, input.text, input.kind)
					binder.BindSourceFile(file)
					results[worker] = append(results[worker], file)
				}()
			}
			done.Done()
		}(worker)
	}
	ready.Wait()
	workerSetupNs := time.Since(workerSetupStarted).Nanoseconds()
	cpuCapacity, goroutinesReady := runtime.NumCPU(), runtime.NumGoroutine()
	var before, after runtime.MemStats
	runtime.ReadMemStats(&before)
	started := time.Now()
	close(start)
	for index := range inputs {
		channels[index%workers] <- index
	}
	for _, channel := range channels {
		close(channel)
	}
	done.Wait()
	elapsed := time.Since(started)
	runtime.ReadMemStats(&after)
	if after.TotalAlloc < before.TotalAlloc {
		panic("allocation counter wrapped")
	}
	report := Report{Version: 1, LoadedInputSHA256: loadedInputSHA256, Workers: workers, LoadedBytes: loadedBytes, WallTimeNs: elapsed.Nanoseconds(), AllocatedBytes: after.TotalAlloc - before.TotalAlloc, GOMAXPROCS: runtime.GOMAXPROCS(0), GOGC: 100, StartupNs: startupNs, PreloadNs: preloadNs, WorkerSetupNs: workerSetupNs, CPUCapacity: cpuCapacity, GoroutinesReady: goroutinesReady}
	for worker, files := range results {
		if failures[worker] != nil {
			panic(failures[worker])
		}
		for _, file := range files {
			report.Files++
			report.Nodes += file.NodeCount
			report.Symbols += file.SymbolCount
			report.ParseDiagnostics += len(file.Diagnostics())
			report.BindDiagnostics += len(file.BindDiagnostics())
		}
	}
	if report.Files != len(inputs) {
		panic("workload file count mismatch")
	}
	if graphMode {
		if graphIndex >= 0 {
			writeGraphRecords(os.Stdout, results[graphIndex%workers][graphIndex/workers], graphIndex, workers)
		} else {
			for index, input := range inputs {
				writeGraphReport(os.Stdout, results[index%workers][index/workers], input, index, workers)
			}
		}
		runtime.KeepAlive(results)
		return
	}
	if err := json.NewEncoder(os.Stdout).Encode(report); err != nil {
		panic(err)
	}
	runtime.KeepAlive(results)
}
