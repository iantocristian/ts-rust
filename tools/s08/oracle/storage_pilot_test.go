package checker

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"runtime"
	"testing"
	"unsafe"
)

// Actual constructor calls, not a transcription of the Rust representation.
// Input text and the empty Checker are prepared before the allocation interval.
func TestS08StoragePilot(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("S08_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	var requests []struct {
		Op      string `json:"op"`
		Flags   uint32 `json:"flags"`
		TextHex string `json:"text_hex"`
		Root    int    `json:"root"`
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&requests); err != nil {
		t.Fatal(err)
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		t.Fatalf("trailing input: %v", err)
	}
	names := make([]string, len(requests))
	for i, request := range requests {
		value, err := hex.DecodeString(request.TextHex)
		if err != nil {
			t.Fatal(err)
		}
		names[i] = string(value)
		switch request.Op {
		case "intrinsic", "string":
		case "fresh":
			if request.Root < 0 || request.Root >= i {
				t.Fatal("invalid fresh root")
			}
		default:
			t.Fatal("unknown constructor operation")
		}
	}
	c := &Checker{}
	runtime.GC()
	var before, allocated, retained runtime.MemStats
	runtime.ReadMemStats(&before)
	c.stringLiteralTypes = make(map[string]*Type)
	roots := make([]*Type, 0, len(requests))
	for i, request := range requests {
		var value *Type
		switch request.Op {
		case "intrinsic":
			value = c.newIntrinsicTypeEx(TypeFlags(request.Flags), names[i], ObjectFlagsNone)
		case "string":
			value = c.getStringLiteralType(names[i])
		case "fresh":
			if roots[request.Root].flags != TypeFlagsStringLiteral {
				t.Fatal("pilot only supports fresh string types")
			}
			value = c.getFreshTypeOfLiteralType(roots[request.Root])
		}
		roots = append(roots, value)
	}
	runtime.ReadMemStats(&allocated)
	runtime.GC()
	runtime.ReadMemStats(&retained)
	runtime.KeepAlive(c)
	runtime.KeepAlive(roots)
	runtime.KeepAlive(names)
	// The decoded request backing and decoder buffers must survive both endpoints.
	// Keeping only len(requests) live lets GC reclaim the backing during the interval.
	runtime.KeepAlive(requests)
	runtime.KeepAlive(decoder)
	runtime.KeepAlive(raw)
	observations := make([]map[string]any, 0, len(roots))
	for _, value := range roots {
		row := map[string]any{"id": value.id, "flags": value.flags, "regular": 0, "fresh": 0}
		switch data := value.data.(type) {
		case *IntrinsicType:
			row["kind"] = "intrinsic"
			row["text_hex"] = hex.EncodeToString([]byte(data.intrinsicName))
		case *LiteralType:
			row["kind"] = "string"
			row["text_hex"] = hex.EncodeToString([]byte(data.value.(string)))
			row["regular"] = data.regularType.id
			if data.freshType != nil {
				row["fresh"] = data.freshType.id
			}
		default:
			t.Fatal("unexpected type payload")
		}
		observations = append(observations, row)
	}
	sum := sha256.Sum256(raw)
	output, err := json.Marshal(map[string]any{
		"request_sha256": hex.EncodeToString(sum[:]), "go": runtime.Version(), "goos": runtime.GOOS, "goarch": runtime.GOARCH,
		"roots": observations,
		"census": map[string]any{"type_records": c.TypeCount, "string_cache_entries": len(c.stringLiteralTypes),
			"intrinsic_record_bytes": unsafe.Sizeof(IntrinsicType{}), "string_record_bytes": unsafe.Sizeof(LiteralType{})},
		"allocator": map[string]any{"requested_bytes": allocated.TotalAlloc - before.TotalAlloc,
			"live_before": before.HeapAlloc, "live_after": retained.HeapAlloc,
			"retained_delta": int64(retained.HeapAlloc) - int64(before.HeapAlloc)},
	})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("S08_OUTPUT"), output, 0600); err != nil {
		t.Fatal(err)
	}
}
