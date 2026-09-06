package core

import "unicode/utf8"

// DecodeRuneForOracle exposes Go's standard decoder so the fixtures record the
// (rune, size) pairs the standard-decoding helpers see.
func DecodeRuneForOracle(s string) (rune, int) {
	return utf8.DecodeRuneInString(s)
}
