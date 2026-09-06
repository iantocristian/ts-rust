// Test-only access to the leaf escaping worker, without constructing printer AST nodes.
package printer

import "strings"

func S04Escape(text string, quote int, flags int) string {
	var output strings.Builder
	escapeStringWorker(text, QuoteChar(quote), getLiteralTextFlags(flags), &output)
	return output.String()
}
