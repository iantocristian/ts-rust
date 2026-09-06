package printer

import "strings"

// OracleEscapeStringWorker exposes the unexported escape worker (E4 fixtures).
func OracleEscapeStringWorker(s string, quote rune, flags int) string {
	var b strings.Builder
	escapeStringWorker(s, QuoteChar(quote), getLiteralTextFlags(flags), &b)
	return b.String()
}
