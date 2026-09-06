// Package oracle re-exports the internal file decoder for the E4 oracle command.
// It is overlaid as a new package under internal/vfs, which may import
// internal/vfs/internal without the import cycle a wrapper inside vfs would create.
package oracle

import "github.com/microsoft/TypeScript/tsc/internal/vfs/internal"

func DecodeBytes(s string) (string, bool) {
	return internal.OracleDecodeBytes(s)
}
