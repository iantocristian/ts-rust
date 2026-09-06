package main

import "unicode/utf8"

// decodeStandard is Go's own UTF-8 decoder, kept behind one name so the probe
// values state which decoder produced them.
func decodeStandard(s string) (rune, int) {
	return utf8.DecodeRuneInString(s)
}
