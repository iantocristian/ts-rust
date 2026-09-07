// Access-only exporter. Copy under the tooling checkout's tsc/ directory so
// Go's internal-package rule permits importing the pinned encoder constants.
package main

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/microsoft/TypeScript/tsc/internal/api/encoder"
)

func main() {
	if len(os.Args) != 2 {
		panic("expected encoder constants output path")
	}
	constants := map[string]uint32{
		"ProtocolVersion":            uint32(encoder.ProtocolVersion),
		"NodeOffsetKind":             encoder.NodeOffsetKind,
		"NodeOffsetPos":              encoder.NodeOffsetPos,
		"NodeOffsetEnd":              encoder.NodeOffsetEnd,
		"NodeOffsetNext":             encoder.NodeOffsetNext,
		"NodeOffsetParent":           encoder.NodeOffsetParent,
		"NodeOffsetData":             encoder.NodeOffsetData,
		"NodeOffsetFlags":            encoder.NodeOffsetFlags,
		"NodeSize":                   encoder.NodeSize,
		"NodeDataTypeChildren":       encoder.NodeDataTypeChildren,
		"NodeDataTypeString":         encoder.NodeDataTypeString,
		"NodeDataTypeExtendedData":   encoder.NodeDataTypeExtendedData,
		"NodeDataTypeMask":           encoder.NodeDataTypeMask,
		"NodeDataChildMask":          encoder.NodeDataChildMask,
		"NodeDataStringIndexMask":    encoder.NodeDataStringIndexMask,
		"SyntaxKindNodeList":         encoder.SyntaxKindNodeList,
		"HeaderOffsetMetadata":       encoder.HeaderOffsetMetadata,
		"HeaderOffsetHashLo0":        encoder.HeaderOffsetHashLo0,
		"HeaderOffsetHashLo1":        encoder.HeaderOffsetHashLo1,
		"HeaderOffsetHashHi0":        encoder.HeaderOffsetHashHi0,
		"HeaderOffsetHashHi1":        encoder.HeaderOffsetHashHi1,
		"HeaderOffsetParseOptions":   encoder.HeaderOffsetParseOptions,
		"HeaderOffsetStringOffsets":  encoder.HeaderOffsetStringOffsets,
		"HeaderOffsetStringData":     encoder.HeaderOffsetStringData,
		"HeaderOffsetExtendedData":   encoder.HeaderOffsetExtendedData,
		"HeaderOffsetStructuredData": encoder.HeaderOffsetStructuredData,
		"HeaderOffsetNodes":          encoder.HeaderOffsetNodes,
		"HeaderSize":                 encoder.HeaderSize,
	}
	data, err := json.MarshalIndent(struct {
		Version   int               `json:"version"`
		Constants map[string]uint32 `json:"constants"`
	}{Version: 1, Constants: constants}, "", "  ")
	if err != nil {
		panic(err)
	}
	if err := os.WriteFile(os.Args[1], append(data, '\n'), 0o644); err != nil {
		panic(err)
	}
	fmt.Printf("exported %d encoder constants\n", len(constants))
}
