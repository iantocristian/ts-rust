// Access-only export of pinned scanner tables. No scanner algorithm is replaced.
package scanner

import "slices"

func S05Tables() map[string]any {
	keywords := map[string]int{}
	for name, kind := range textToKeyword {
		keywords[name] = int(kind)
	}
	tokens := map[string]int{}
	for name, kind := range textToToken {
		tokens[name] = int(kind)
	}
	binary := []string{}
	for name := range binaryUnicodeProperties.Keys() {
		binary = append(binary, name)
	}
	strings := []string{}
	for name := range binaryUnicodePropertiesOfStrings.Keys() {
		strings = append(strings, name)
	}
	values := map[string][]string{}
	for property, set := range valuesOfNonBinaryUnicodeProperties {
		names := []string{}
		for name := range set.Keys() {
			names = append(names, name)
		}
		slices.Sort(names)
		values[property] = names
	}
	slices.Sort(binary)
	slices.Sort(strings)
	return map[string]any{"keywords": keywords, "tokens": tokens,
		"non_binary": nonBinaryUnicodeProperties, "binary": binary,
		"strings": strings, "values": values}
}

func S05NormalizeJSDoc(text string) string { return normalizeJSDocTypeSourceText(text) }
