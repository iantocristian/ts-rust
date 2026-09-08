// Access-only retained-graph census. Copy into the pinned export's internal/ast
// package; this file never changes source objects or invokes lazy accessors.
package ast

import (
	"fmt"
	"reflect"
	"runtime"
	"sort"
)

type MemoryTypeCensus struct {
	Type         string `json:"type"`
	Category     string `json:"category"`
	Count        uint64 `json:"count"`
	Size         uint64 `json:"size"`
	LogicalBytes uint64 `json:"logical_bytes"`
}

type MemorySliceCensus struct {
	Type                 string `json:"type"`
	Headers              uint64 `json:"headers"`
	NilHeaders           uint64 `json:"nil_headers"`
	LengthElements       uint64 `json:"length_elements"`
	CapacityElements     uint64 `json:"capacity_elements"`
	ElementSize          uint64 `json:"element_size"`
	VisibleCapacityUnion uint64 `json:"visible_capacity_union_bytes"`
}

type MemoryMapCensus struct {
	Type              string `json:"type"`
	Maps              uint64 `json:"maps"`
	Entries           uint64 `json:"entries"`
	EntryLogicalSize  uint64 `json:"entry_logical_size"`
	EntryLogicalBytes uint64 `json:"entry_logical_bytes"`
}

type MemoryStringCensus struct {
	Fields                   uint64 `json:"fields"`
	EmptyFields              uint64 `json:"empty_fields"`
	LogicalBytes             uint64 `json:"logical_bytes"`
	SourceBackedFields       uint64 `json:"source_backed_fields"`
	SourceBackedLogicalBytes uint64 `json:"source_backed_logical_bytes"`
	SourceVisibleUnionBytes  uint64 `json:"source_visible_union_bytes"`
	OtherVisibleUnionBytes   uint64 `json:"other_visible_union_bytes"`
}

type MemoryFileCensus struct {
	NodeKinds                    map[string]uint64   `json:"node_kinds"`
	Objects                      []MemoryTypeCensus  `json:"objects"`
	Slices                       []MemorySliceCensus `json:"slices"`
	Maps                         []MemoryMapCensus   `json:"maps"`
	Strings                      MemoryStringCensus  `json:"strings"`
	NodeHeaderContainedBytes     uint64              `json:"node_header_contained_bytes"`
	SourceTextBytes              uint64              `json:"source_text_bytes"`
	PointerIdentities            uint64              `json:"pointer_identities"`
	SkippedExternalPointerFields map[string]uint64   `json:"skipped_external_pointer_fields"`
}

type memoryPointerKey struct {
	typeOf reflect.Type
	ptr    uintptr
}

type memoryRange struct{ start, end uintptr }

type memoryObject struct {
	MemoryTypeCensus
	kindIndex []int
}

type memoryCensus struct {
	seen    map[memoryPointerKey]bool
	objects map[reflect.Type]*memoryObject
	slices  map[reflect.Type]*MemorySliceCensus
	maps    map[reflect.Type]*MemoryMapCensus
	ranges  map[reflect.Type][]memoryRange
	strings [2][]memoryRange
	source  memoryRange
	result  MemoryFileCensus
}

var memoryNodePointer = reflect.TypeOf((*Node)(nil))
var memoryNodeData = reflect.TypeOf((*nodeData)(nil)).Elem()
var memoryAstPackage = reflect.TypeOf(Node{}).PkgPath()
var memoryNodeDataIndex = func() int {
	field, exists := memoryNodePointer.Elem().FieldByName("data")
	if !exists || len(field.Index) != 1 {
		panic("Node.data is not a direct field")
	}
	return field.Index[0]
}()

// This is a census of reachable logical objects and visible backing ranges,
// not an allocator census. In particular, core.Arena's retired headers do not
// expose unused slots in backing arrays kept alive by interior pointers.
// Dedupe tables exist only while one file is visited. Result values retain no
// pointers into the compiler graph. Shared globals can recur across files.
func MemoryProfileCensus(file *SourceFile) MemoryFileCensus {
	if file == nil {
		panic("memory census requires a source file")
	}
	text := reflect.ValueOf(file.text)
	c := memoryCensus{
		seen:    make(map[memoryPointerKey]bool),
		objects: make(map[reflect.Type]*memoryObject),
		slices:  make(map[reflect.Type]*MemorySliceCensus),
		maps:    make(map[reflect.Type]*MemoryMapCensus),
		ranges:  make(map[reflect.Type][]memoryRange),
		source:  memoryExtent(text.Pointer(), uint64(text.Len()), 1),
		result: MemoryFileCensus{NodeKinds: make(map[string]uint64),
			SourceTextBytes:              uint64(len(file.text)),
			SkippedExternalPointerFields: make(map[string]uint64)},
	}
	c.visit(reflect.ValueOf(file.AsNode()))
	for typ, values := range c.ranges {
		c.slices[typ].VisibleCapacityUnion = memoryUnion(values)
	}
	c.result.Strings.SourceVisibleUnionBytes = memoryUnion(c.strings[0])
	c.result.Strings.OtherVisibleUnionBytes = memoryUnion(c.strings[1])
	c.result.PointerIdentities = uint64(len(c.seen))
	for _, value := range c.objects {
		c.result.Objects = append(c.result.Objects, value.MemoryTypeCensus)
	}
	for _, value := range c.slices {
		c.result.Slices = append(c.result.Slices, *value)
	}
	for _, value := range c.maps {
		c.result.Maps = append(c.result.Maps, *value)
	}
	sort.Slice(c.result.Objects, func(i, j int) bool { return c.result.Objects[i].Type < c.result.Objects[j].Type })
	sort.Slice(c.result.Slices, func(i, j int) bool { return c.result.Slices[i].Type < c.result.Slices[j].Type })
	sort.Slice(c.result.Maps, func(i, j int) bool { return c.result.Maps[i].Type < c.result.Maps[j].Type })
	runtime.KeepAlive(file)
	return c.result
}

func memoryExtent(start uintptr, count, size uint64) memoryRange {
	if size != 0 && count > uint64(^uintptr(0))/size {
		panic("memory census range multiplication overflow")
	}
	bytes := uintptr(count * size)
	if start > ^uintptr(0)-bytes {
		panic("memory census range overflow")
	}
	return memoryRange{start, start + bytes}
}

func memoryUnion(ranges []memoryRange) uint64 {
	if len(ranges) == 0 {
		return 0
	}
	sort.Slice(ranges, func(i, j int) bool {
		if ranges[i].start != ranges[j].start {
			return ranges[i].start < ranges[j].start
		}
		return ranges[i].end < ranges[j].end
	})
	current := ranges[0]
	var size uint64
	for _, next := range ranges[1:] {
		if next.start <= current.end {
			current.end = max(current.end, next.end)
		} else {
			size += uint64(current.end - current.start)
			current = next
		}
	}
	return size + uint64(current.end-current.start)
}

func (c *memoryCensus) visit(value reflect.Value) {
	if !value.IsValid() {
		return
	}
	switch value.Kind() {
	case reflect.Pointer:
		if value.IsNil() {
			return
		}
		key := memoryPointerKey{value.Type(), value.Pointer()}
		if c.seen[key] {
			return
		}
		c.seen[key] = true
		if len(c.seen) > 8_000_000 {
			panic("memory census exceeds per-file pointer bound")
		}
		if value.Type() == memoryNodePointer {
			// Node is embedded in the concrete payload; it is not a separate
			// allocation. Visit data to count the enclosing concrete object once.
			c.visit(value.Elem().Field(memoryNodeDataIndex))
			return
		}
		typ := value.Type().Elem()
		if typ.PkgPath() != memoryAstPackage {
			// For example, Diagnostic.message points at global diagnostic
			// metadata, not a compiler-owned heap allocation.
			c.result.SkippedExternalPointerFields[value.Type().String()]++
			return
		}
		object := c.objects[typ]
		if object == nil {
			category := "other_ast_object"
			if value.Type().Implements(memoryNodeData) {
				category = "concrete_node_including_header"
			} else {
				switch typ.Name() {
				case "Symbol":
					category = "symbol"
				case "FlowNode":
					category = "flow_node"
				case "FlowList":
					category = "flow_list"
				case "NodeList", "ModifierList":
					category = "node_list"
				case "Diagnostic":
					category = "diagnostic"
				}
			}
			object = &memoryObject{MemoryTypeCensus: MemoryTypeCensus{Type: typ.String(), Category: category, Size: uint64(typ.Size())}}
			if category == "concrete_node_including_header" {
				kind, exists := typ.FieldByName("Kind")
				if !exists {
					panic("concrete node has no embedded Kind")
				}
				object.kindIndex = kind.Index
			}
			c.objects[typ] = object
		}
		object.Count++
		object.LogicalBytes += object.Size
		if object.Category == "concrete_node_including_header" {
			c.result.NodeHeaderContainedBytes += uint64(memoryNodePointer.Elem().Size())
			kind := value.Elem().FieldByIndex(object.kindIndex)
			c.result.NodeKinds[Kind(kind.Int()).String()]++
		}
		c.visit(value.Elem())
	case reflect.Interface:
		if !value.IsNil() {
			c.visit(value.Elem())
		}
	case reflect.Struct:
		for index := range value.NumField() {
			c.visit(value.Field(index))
		}
	case reflect.Array:
		for index := range value.Len() {
			c.visit(value.Index(index))
		}
	case reflect.Slice:
		typ := value.Type()
		entry := c.slices[typ]
		if entry == nil {
			entry = &MemorySliceCensus{Type: typ.String(), ElementSize: uint64(typ.Elem().Size())}
			c.slices[typ] = entry
		}
		entry.Headers++
		if value.IsNil() {
			entry.NilHeaders++
			return
		}
		entry.LengthElements += uint64(value.Len())
		entry.CapacityElements += uint64(value.Cap())
		if value.Cap() != 0 && entry.ElementSize != 0 {
			c.ranges[typ] = append(c.ranges[typ], memoryExtent(value.Pointer(), uint64(value.Cap()), entry.ElementSize))
		}
		for index := range value.Len() {
			c.visit(value.Index(index))
		}
	case reflect.Map:
		if value.IsNil() {
			return
		}
		typ := value.Type()
		key := memoryPointerKey{typ, value.Pointer()}
		if c.seen[key] {
			return
		}
		c.seen[key] = true
		entry := c.maps[typ]
		if entry == nil {
			entry = &MemoryMapCensus{Type: typ.String(), EntryLogicalSize: uint64(typ.Key().Size() + typ.Elem().Size())}
			c.maps[typ] = entry
		}
		entry.Maps++
		entry.Entries += uint64(value.Len())
		entry.EntryLogicalBytes += uint64(value.Len()) * entry.EntryLogicalSize
		iterator := value.MapRange()
		for iterator.Next() {
			c.visit(iterator.Key())
			c.visit(iterator.Value())
		}
	case reflect.String:
		entry := &c.result.Strings
		entry.Fields++
		entry.LogicalBytes += uint64(value.Len())
		if value.Len() == 0 {
			entry.EmptyFields++
			return
		}
		span := memoryExtent(value.Pointer(), uint64(value.Len()), 1)
		bucket := 1
		if span.start >= c.source.start && span.end <= c.source.end {
			bucket = 0
			entry.SourceBackedFields++
			entry.SourceBackedLogicalBytes += uint64(value.Len())
		}
		c.strings[bucket] = append(c.strings[bucket], span)
	case reflect.Bool, reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64,
		reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64, reflect.Uintptr,
		reflect.Float32, reflect.Float64, reflect.Complex64, reflect.Complex128:
		// Inline scalar storage is already included in its enclosing object.
	case reflect.Func, reflect.Chan, reflect.UnsafePointer:
		if !value.IsNil() {
			panic(fmt.Sprintf("unsupported retained census field %s", value.Type()))
		}
	default:
		panic(fmt.Sprintf("unsupported retained census kind %s", value.Kind()))
	}
}
