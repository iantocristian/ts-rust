// Diagnostic CPU profiling adapter for the pinned source parser and binder.
// Its queue and loaded-input identity match tools/s07/benchmark/main.go.
// Profiling and per-file timers add overhead: none of these timings or CPU
// sample shares are acceptance measurements.
package main

import (
	"context"
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

// These sums measure elapsed time inside the actual parse/bind callbacks,
// including scheduler and GC pauses. They are not per-thread CPU time, and
// concurrent workers' sums can exceed the overall phase wall time.
type WorkerTimes struct {
	Worker     int   `json:"worker"`
	Files      int   `json:"files"`
	ParseCalls int   `json:"parse_calls"`
	BindCalls  int   `json:"bind_calls"`
	ParseNs    int64 `json:"parse_ns"`
	BindNs     int64 `json:"bind_ns"`
}

type Report struct {
	Version           int           `json:"version"`
	Operation         string        `json:"operation"`
	DiagnosticOnly    bool          `json:"diagnostic_only"`
	LoadedInputSHA256 string        `json:"loaded_input_sha256"`
	Workers           int           `json:"workers"`
	Files             int           `json:"files"`
	LoadedBytes       int           `json:"loaded_bytes"`
	Nodes             int           `json:"nodes"`
	Symbols           int           `json:"symbols"`
	ParseDiagnostics  int           `json:"parse_diagnostics"`
	BindDiagnostics   int           `json:"bind_diagnostics"`
	WallTimeNs        int64         `json:"wall_time_ns"`
	ParseCalls        int           `json:"parse_calls"`
	BindCalls         int           `json:"bind_calls"`
	ParseNs           int64         `json:"parse_ns"`
	BindNs            int64         `json:"bind_ns"`
	WorkerTimes       []WorkerTimes `json:"worker_times"`
	TimingDomain      string        `json:"timing_domain"`
	ProfileScope      string        `json:"profile_scope"`
	CPUProfile        string        `json:"cpu_profile"`
	StartupNs         int64         `json:"startup_ns"`
	PreloadNs         int64         `json:"preload_ns"`
	WorkerSetupNs     int64         `json:"worker_setup_ns"`
	CPUCapacity       int           `json:"cpu_capacity"`
	GoroutinesReady   int           `json:"goroutines_ready"`
	GOMAXPROCS        int           `json:"gomaxprocs"`
	GOGC              int           `json:"gogc"`
}

// Observe exactly the same immutable preload and byte protocol as the baseline.
// This hash is calculated before profiling or releasing any worker.
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
	if len(os.Args) != 4 {
		panic("usage: cpu-profile INPUTS.json WORKERS (1 or 8) OUTPUT_PREFIX")
	}
	workers, err := strconv.Atoi(os.Args[2])
	if err != nil || workers != 1 && workers != 8 {
		panic("worker count must be 1 or 8")
	}
	if os.Args[3] == "" {
		panic("empty output prefix")
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
	var trailing any
	if err := decoder.Decode(&trailing); err != io.EOF {
		panic("workload must contain exactly one JSON value")
	}
	if err := input.Close(); err != nil {
		panic(err)
	}
	if len(requests) == 0 {
		panic("empty parse-and-bind workload")
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

	// Open both outputs before the phase. A phase failure leaves no successful
	// report at this prefix; its partial profile can still help diagnose failure.
	prefix, err := filepath.Abs(os.Args[3])
	if err != nil {
		panic(err)
	}
	if err := os.MkdirAll(filepath.Dir(prefix), 0o755); err != nil {
		panic(err)
	}
	reportFile, err := os.Create(prefix + ".report.json")
	if err != nil {
		panic(err)
	}
	defer reportFile.Close()
	profileFile, err := os.Create(prefix + ".pprof")
	if err != nil {
		panic(err)
	}
	defer profileFile.Close()

	workerSetupStarted := time.Now()
	channels := make([]chan int, workers)
	results := make([][]*ast.SourceFile, workers)
	timings := make([]WorkerTimes, workers)
	failures := make([]any, workers)
	ready := sync.WaitGroup{}
	ready.Add(workers)
	done := sync.WaitGroup{}
	done.Add(workers)
	start := make(chan struct{})
	for worker := range workers {
		channels[worker] = make(chan int, workers)
		results[worker] = make([]*ast.SourceFile, 0, (len(inputs)+workers-1)/workers)
		timings[worker].Worker = worker
		go func(worker int) {
			defer done.Done()
			ctx := context.Background()
			parseLabels := pprof.Labels("phase", "parse")
			bindLabels := pprof.Labels("phase", "bind")
			ready.Done()
			<-start
			for index := range channels[worker] {
				if failures[worker] != nil {
					// Drain after a panic so the bounded dispatcher can finish.
					continue
				}
				func() {
					completed := false
					defer func() {
						failure := recover()
						if !completed {
							failures[worker] = fmt.Sprintf("file %s: %v\n%s", inputs[index].options.FileName, failure, debug.Stack())
						}
					}()
					input := inputs[index]
					var file *ast.SourceFile
					pprof.Do(ctx, parseLabels, func(context.Context) {
						started := time.Now()
						file = parser.ParseSourceFile(input.options, input.text, input.kind)
						timings[worker].ParseNs += time.Since(started).Nanoseconds()
						timings[worker].ParseCalls++
					})
					pprof.Do(ctx, bindLabels, func(context.Context) {
						started := time.Now()
						binder.BindSourceFile(file)
						timings[worker].BindNs += time.Since(started).Nanoseconds()
						timings[worker].BindCalls++
					})
					results[worker] = append(results[worker], file)
					timings[worker].Files++
					completed = true
				}()
			}
		}(worker)
	}
	ready.Wait()
	workerSetupNs := time.Since(workerSetupStarted).Nanoseconds()
	cpuCapacity, goroutinesReady := runtime.NumCPU(), runtime.NumGoroutine()
	if err := pprof.StartCPUProfile(profileFile); err != nil {
		panic(err)
	}
	profiling := true
	defer func() {
		if profiling {
			pprof.StopCPUProfile()
		}
	}()
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
	pprof.StopCPUProfile()
	profiling = false
	if err := profileFile.Close(); err != nil {
		panic(err)
	}

	// Roots remain reachable through profile shutdown and the entire summary.
	// No GC, semantic traversal, serialization, or root release is added between
	// the worker endpoint and StopCPUProfile.
	report := Report{
		Version: 1, Operation: "parse_bind_cpu_profile", DiagnosticOnly: true,
		LoadedInputSHA256: loadedInputSHA256, Workers: workers, LoadedBytes: loadedBytes,
		WallTimeNs: elapsed.Nanoseconds(), WorkerTimes: timings,
		TimingDomain: "parse_ns and bind_ns sum per-file elapsed worker time inside the labeled callbacks, including scheduler and GC pauses; they exclude pprof label setup/restore and are not CPU time",
		ProfileScope: "CPU samples from before the worker barrier release until immediately after all workers finish; pprof.Do phase=parse surrounds ParseSourceFile and phase=bind surrounds BindSourceFile; labels include their wrapper overhead, while runtime/background work may be unlabeled",
		CPUProfile:   profileFile.Name(), GOMAXPROCS: runtime.GOMAXPROCS(0), GOGC: 100,
		StartupNs: startupNs, PreloadNs: preloadNs, WorkerSetupNs: workerSetupNs,
		CPUCapacity: cpuCapacity, GoroutinesReady: goroutinesReady,
	}
	for worker, files := range results {
		if failures[worker] != nil {
			panic(failures[worker])
		}
		expected := len(inputs) / workers
		if worker < len(inputs)%workers {
			expected++
		}
		stats := timings[worker]
		if len(files) != expected || stats.Files != expected || stats.ParseCalls != expected || stats.BindCalls != expected {
			panic(fmt.Sprintf("worker %d workload count mismatch: expected %d, retained %d, completed %+v", worker, expected, len(files), stats))
		}
		report.ParseCalls += stats.ParseCalls
		report.BindCalls += stats.BindCalls
		report.ParseNs += stats.ParseNs
		report.BindNs += stats.BindNs
		for _, file := range files {
			report.Files++
			report.Nodes += file.NodeCount
			report.Symbols += file.SymbolCount
			report.ParseDiagnostics += len(file.Diagnostics())
			report.BindDiagnostics += len(file.BindDiagnostics())
		}
	}
	if report.Files != len(inputs) || report.ParseCalls != len(inputs) || report.BindCalls != len(inputs) {
		panic("workload file count mismatch")
	}
	if err := json.NewEncoder(reportFile).Encode(report); err != nil {
		panic(err)
	}
	if err := reportFile.Close(); err != nil {
		panic(err)
	}
	if err := json.NewEncoder(os.Stdout).Encode(report); err != nil {
		panic(err)
	}
	runtime.KeepAlive(results)
}
