package lsconv

import (
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/lsp/lsproto"
)

// Oracle exports for the unexported raw coordinate converters (E4 fixtures).

func OracleLineAndCharacterToPosition(c *Converters, script Script, line, character uint32) core.TextPos {
	return c.lineAndCharacterToPosition(script, lsproto.Position{Line: line, Character: character})
}

func OraclePositionToLineAndCharacter(c *Converters, script Script, position core.TextPos) lsproto.Position {
	return c.positionToLineAndCharacter(script, position)
}
