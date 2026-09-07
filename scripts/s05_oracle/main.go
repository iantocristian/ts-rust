// S05 invokes the pinned scanner. Bytes use hex, never JSON's UTF-8 repair.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"unicode"

	"github.com/microsoft/TypeScript/tsc/internal/scanner"
	"github.com/microsoft/TypeScript/tsc/internal/stringutil"
)

func main() {
	if len(os.Args) == 2 && os.Args[1] == "--tables" {
		fold := [][2]int32{}
		for ch := rune(0); ch <= unicode.MaxRune; ch++ {
			if next := unicode.SimpleFold(ch); next != ch {
				fold = append(fold, [2]int32{ch, next})
			}
		}
		if err := json.NewEncoder(os.Stdout).Encode(map[string]any{"version": 1,
			"scanner": scanner.S05Tables(), "identifier": stringutil.S05IdentifierTables(),
			"simple_fold": fold, "go_unicode_version": unicode.Version}); err != nil {
			panic(err)
		}
		return
	}
	if err := serve(os.Stdin, os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, "S05 oracle:", err)
		os.Exit(1)
	}
}
