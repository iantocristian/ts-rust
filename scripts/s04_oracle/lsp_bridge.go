// Test-only access to raw coordinate conversions, with the production line map.
package lsconv

import (
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/lsp/lsproto"
	"github.com/microsoft/TypeScript/tsc/internal/spanmap"
)

type s04Script string

func (s s04Script) FileName() string { return "s04.ts" }
func (s s04Script) OriginalFileName() string { return "s04.ts" }
func (s s04Script) Text() string { return string(s) }
func (s s04Script) OriginalText() string { return string(s) }
func (s s04Script) SpanMap() *spanmap.SpanMap { return nil }

func s04Converters(text string, utf8 bool) *Converters {
	encoding := lsproto.PositionEncodingKindUTF16
	if utf8 { encoding = lsproto.PositionEncodingKindUTF8 }
	lineMap := ComputeLSPLineStarts(text)
	return NewConverters(encoding, func(string) *LSPLineMap { return lineMap })
}

func S04ToPosition(text string, line, character uint32, utf8 bool) int32 {
	return int32(s04Converters(text, utf8).lineAndCharacterToPosition(s04Script(text), lsproto.Position{Line: line, Character: character}))
}

func S04FromPosition(text string, position int32, utf8 bool) [2]uint32 {
	p := s04Converters(text, utf8).positionToLineAndCharacter(s04Script(text), core.TextPos(position))
	return [2]uint32{p.Line, p.Character}
}
