// This access-only test is overlaid into the pinned Go standard-library slices
// package. Its sorter and helper bodies remain the unmodified Go implementation.
package slices_test

import (
	"cmp"
	"encoding/json"
	"os"
	"runtime"
	"slices"
	"testing"
)

type sortCase struct {
	Name        string   `json:"name"`
	Mode        string   `json:"mode"`
	Lazy        bool     `json:"lazy"`
	Keys        []int    `json:"keys"`
	Start       int      `json:"start"`
	End         int      `json:"end"`
	Sorted      []int    `json:"sorted"`
	Trace       [][3]int `json:"trace"`
	Result      bool     `json:"result"`
	ResultIndex int      `json:"result_index"`
	Panicked    bool     `json:"panicked"`
}

func TestS06NodeIndexSortTrace(t *testing.T) {
	var cases []sortCase
	for _, n := range []int{0, 1, 2, 12, 13, 49, 50, 51, 127, 257} {
		for _, pattern := range []string{"ascending", "descending", "equal", "sawtooth", "organ-pipe", "random"} {
			keys := make([]int, n)
			random := uint64(n + 73)
			for i := range keys {
				switch pattern {
				case "ascending":
					keys[i] = i
				case "descending":
					keys[i] = n - i
				case "equal":
					keys[i] = 0
				case "sawtooth":
					keys[i] = i % 7
				case "organ-pipe":
					if i < n/2 {
						keys[i] = i
					} else {
						keys[i] = n - i
					}
				case "random":
					random ^= random << 13
					random ^= random >> 7
					random ^= random << 17
					keys[i] = int(random % uint64(n+1))
				}
			}
			for _, lazy := range []bool{false, true} {
				cases = append(cases, sortCase{Name: pattern, Mode: "sort", Lazy: lazy, Keys: keys, End: n})
			}
		}
	}
	for _, mode := range []string{"heap", "partial", "break", "partition-equal"} {
		keys := make([]int, 96)
		for i := range keys {
			keys[i] = i
		}
		switch mode {
		case "heap":
			for i := range keys {
				keys[i] = (i * 37) % 96
			}
		case "partial":
			keys[16], keys[65] = keys[65], keys[16]
			keys[65] = -4
		case "partition-equal":
			for i := range keys {
				if i < 70 {
					keys[i] = 0
				} else {
					keys[i] = 1
				}
			}
		}
		cases = append(cases, sortCase{Name: "forced-" + mode, Mode: mode, Keys: keys, Start: 8, End: 88})
	}
	cases = append(cases, sortCase{Name: "comparison-unwind", Mode: "panic", Keys: []int{3, 2, 1}, End: 3})
	for i := range cases {
		c := &cases[i]
		c.Sorted = make([]int, len(c.Keys))
		for i := range c.Sorted {
			c.Sorted[i] = i
		}
		c.Trace = make([][3]int, 0)
		assigned := map[int]int{}
		next := 0
		getID := func(key int) int {
			if id, ok := assigned[key]; ok {
				return id
			}
			next++
			assigned[key] = next
			return next
		}
		compare := func(a, b int) int {
			left, right := c.Keys[a], c.Keys[b]
			if c.Lazy {
				left = getID(left)
				right = getID(right)
			}
			result := cmp.Compare(left, right)
			c.Trace = append(c.Trace, [3]int{a, b, result})
			if c.Mode == "panic" && len(c.Trace) == 2 {
				panic("sort comparison")
			}
			return result
		}
		func() {
			defer func() {
				if value := recover(); value != nil {
					if value != "sort comparison" {
						panic(value)
					}
					c.Panicked = true
				}
			}()
			switch c.Mode {
			case "sort", "panic":
				slices.SortFunc(c.Sorted, compare)
			case "heap":
				slices.S06Heap(c.Sorted, c.Start, c.End, compare)
			case "partial":
				c.Result = slices.S06Partial(c.Sorted, c.Start, c.End, compare)
			case "break":
				slices.S06Break(c.Sorted, c.Start, c.End, compare)
			case "partition-equal":
				c.ResultIndex = slices.S06PartitionEqual(c.Sorted, c.Start, c.End, c.Start, compare)
			default:
				t.Fatal(c.Mode)
			}
		}()
	}
	out, err := os.Create(os.Getenv("S06_SORT_TRACE_OUTPUT"))
	if err != nil {
		t.Fatal(err)
	}
	defer out.Close()
	if err := json.NewEncoder(out).Encode(struct {
		Go    string     `json:"go"`
		Cases []sortCase `json:"cases"`
	}{runtime.Version(), cases}); err != nil {
		t.Fatal(err)
	}
}
