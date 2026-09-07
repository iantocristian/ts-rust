// Access-only export of pinned Unicode identifier ranges.
package stringutil

import "unicode"

func S05IdentifierTables() map[string][][3]uint32 {
	rows := func(table *unicode.RangeTable) [][3]uint32 {
		result := make([][3]uint32, 0, len(table.R16)+len(table.R32))
		for _, row := range table.R16 {
			result = append(result, [3]uint32{uint32(row.Lo), uint32(row.Hi), uint32(row.Stride)})
		}
		for _, row := range table.R32 {
			result = append(result, [3]uint32{row.Lo, row.Hi, row.Stride})
		}
		return result
	}
	return map[string][][3]uint32{"start": rows(unicodeESNextIdentifierStart), "part": rows(unicodeESNextIdentifierPart)}
}
