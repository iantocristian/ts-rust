// Frozen construction scripts; all codec behavior calls the unchanged pin.
package main

import (
	"encoding/hex"
	"github.com/microsoft/TypeScript/tsc/internal/api/encoder"
	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/parser"
	"github.com/microsoft/TypeScript/tsc/internal/spanmap"
	"github.com/microsoft/TypeScript/tsc/internal/tspath"
	"github.com/zeebo/xxh3"
	"strings"
)

var literalScenarios = []string{"StringLiteral", "NumericLiteral", "BigIntLiteral", "RegularExpressionLiteral", "NoSubstitutionTemplateLiteral", "TemplateHead", "TemplateMiddle", "TemplateTail", "Identifier", "PrivateIdentifier", "JsxText", "JSDocText", "JSDocLink", "JSDocLinkPlain", "JSDocLinkCode"}

func codecScenario(name string) bool {
	for _, literal := range literalScenarios {
		if name == "literal/"+literal {
			return true
		}
	}
	switch name {
	case "synthetic-expression", "raw-kind", "source-strings", "source-metadata", "source-empty-span", "msgpack-boundaries", "parsed-js/typedef", "parsed-js/import":
		return true
	}
	return false
}
func codecLiteral(f *ast.NodeFactory, name string) *ast.Node {
	text := "a😀\xed\xa0\x80\xff"
	raw := `raw\n\uD800`
	flags := ast.TokenFlags(-1)
	switch name {
	case "StringLiteral":
		return f.NewStringLiteral(text, flags)
	case "NumericLiteral":
		return f.NewNumericLiteral(text, flags)
	case "BigIntLiteral":
		return f.NewBigIntLiteral(text, flags)
	case "RegularExpressionLiteral":
		return f.NewRegularExpressionLiteral(text, flags)
	case "NoSubstitutionTemplateLiteral":
		return f.NewNoSubstitutionTemplateLiteral(text, flags)
	case "TemplateHead":
		return f.NewTemplateHead(text, raw, flags)
	case "TemplateMiddle":
		return f.NewTemplateMiddle(text, raw, flags)
	case "TemplateTail":
		return f.NewTemplateTail(text, raw, flags)
	case "Identifier":
		return f.NewIdentifier(text)
	case "PrivateIdentifier":
		return f.NewPrivateIdentifier(text)
	case "JsxText":
		return f.NewJsxText(text, true)
	case "JSDocText":
		return f.NewJSDocText([]string{"a", "", text[1:]})
	case "JSDocLink":
		return f.NewJSDocLink(f.NewIdentifier("name"), []string{"a", "", text[1:]})
	case "JSDocLinkPlain":
		return f.NewJSDocLinkPlain(f.NewIdentifier("name"), []string{"a", "", text[1:]})
	case "JSDocLinkCode":
		return f.NewJSDocLinkCode(f.NewIdentifier("name"), []string{"a", "", text[1:]})
	}
	panic("validated literal missing")
}
func codecSource(f *ast.NodeFactory, name string) *ast.Node {
	text := "'😀' a a\n"
	lit := f.NewStringLiteral("😀", 1024)
	lit.Loc = core.NewTextRange(0, 6)
	a := f.NewIdentifier("a")
	a.Loc = core.NewTextRange(7, 8)
	b := f.NewIdentifier("a")
	b.Loc = core.NewTextRange(9, 10)
	eof := f.NewToken(ast.KindEndOfFile)
	eof.Loc = core.NewTextRange(11, 11)
	list := f.NewNodeList([]*ast.Node{lit, a, b})
	list.Loc = core.NewTextRange(0, 10)
	opts := ast.SourceFileParseOptions{FileName: "/s06/codec.ts", Path: tspath.Path("/s06/codec.ts"), ExternalModuleIndicatorOptions: ast.ExternalModuleIndicatorOptions{JSX: true, Force: true}}
	root := f.NewSourceFile(opts, text, list, eof)
	root.Loc = core.NewTextRange(0, 11)
	sf := root.AsSourceFile()
	sf.NodeCount = f.NodeCount()
	sf.TextCount = f.TextCount()
	sf.ScriptKind = 3
	sf.LanguageVariant = 1
	sf.Hash = xxh3.Uint128{Hi: 0x0123456789abcdef, Lo: 0xfedcba9876543210}
	if name == "source-metadata" || name == "source-empty-span" {
		sf.ModuleAugmentations = []*ast.Node{a, f.NewIdentifier("missing"), nil}
		ast.S06SetImports(sf, []*ast.Node{lit, f.NewStringLiteral("missing", 0), nil})
		sf.ExternalModuleIndicator = root
		sf.AmbientModuleNames = []string{"", "repeat", "repeat", "\xff"}
		sf.ReferencedFiles = []*ast.FileReference{{TextRange: core.NewTextRange(1, 5), FileName: "\xff", ResolutionMode: -1, Preserve: true}}
		sf.TypeReferenceDirectives = []*ast.FileReference{{TextRange: core.NewTextRange(-1, 128), FileName: "types", ResolutionMode: 99}}
		sf.LibReferenceDirectives = []*ast.FileReference{{TextRange: core.NewTextRange(0, 11), FileName: "lib", Preserve: true}}
		other := func(name string) *ast.SourceFile {
			return f.NewSourceFile(ast.SourceFileParseOptions{FileName: name}, "", nil, nil).AsSourceFile()
		}
		segments := []spanmap.Segment{{VirtualStart: 7, VirtualEnd: 10, OriginalStart: 5, OriginalEnd: 6, Kind: 2, Features: 0}, {VirtualStart: 1, VirtualEnd: 5, OriginalStart: 1, OriginalEnd: 5, Kind: 0, Features: spanmap.FeatureAll}}
		if name == "source-empty-span" {
			segments = []spanmap.Segment{}
		}
		sf.SetContentMapperInfo(ast.ContentMapperSourceFileInfo{ContentMapper: "mapper", VirtualFileName: "/s06/codec.virtual.ts", OriginalText: "X😀Y\n", SpanMap: spanmap.New(segments), SupplementalSourceFiles: []*ast.SourceFile{other("/s06/one.ts"), other("/s06/two.ts")}, CanonicalSourceFile: other("/s06/canonical.ts"), DiagnosticDirectives: []ast.MappedDiagnosticDirective{
			{OriginalRange: core.NewTextRange(1, 5), VirtualRange: core.NewTextRange(1, 5), Policy: 1, UnusedCode: -1, UnusedMessageText: "not serialized", Source: "not serialized"},
			{OriginalRange: core.NewTextRange(-1, 128), VirtualRange: core.NewTextRange(-1, 256), Policy: 255, UnusedCode: 65536, UnusedMessageText: "not serialized", Source: "not serialized"},
		}})
	} else if name == "msgpack-boundaries" {
		lengths := []int{0, 31, 32, 255, 256, 65535, 65536, 1}
		sf.AmbientModuleNames = make([]string, 65536)
		for i, n := range lengths {
			sf.AmbientModuleNames[i] = strings.Repeat("x", n)
		}
		positions := []int{0, 127, 128, 255, 256, 65535, 65536, -1}
		for i := 0; i < 16; i++ {
			j := i % 8
			p := positions[j]
			sf.ReferencedFiles = append(sf.ReferencedFiles, &ast.FileReference{TextRange: core.NewTextRange(p, p), FileName: sf.AmbientModuleNames[j], ResolutionMode: core.ResolutionMode(p), Preserve: i%2 == 0})
		}
		sf.TypeReferenceDirectives = sf.ReferencedFiles[:15]
	}
	return root
}
func codecBytes(s *session, stage string, raw []byte) {
	s.observe(stage, "bytes", map[string]any{"length": len(raw)})
	for offset := 0; offset < len(raw); offset += 65536 {
		end := min(offset+65536, len(raw))
		s.observe(stage, "chunk", map[string]any{"offset": offset, "hex": hex.EncodeToString(raw[offset:end])})
	}
}
func executeCodec(s *session, r request) {
	var encoded []byte
	if !s.stage("encode", func() error {
		f := ast.NewNodeFactory(ast.NodeFactoryHooks{})
		name := r.data["scenario"].(string)
		var root *ast.Node
		var sf *ast.SourceFile
		switch {
		case strings.HasPrefix(name, "parsed-js/"):
			text := "/** @typedef {number} Foo */ const x=0;"
			if name == "parsed-js/import" {
				text = "/** @import {Foo} from \"bar\" */ const x=0;"
			}
			sf = parser.ParseSourceFile(ast.SourceFileParseOptions{FileName: "/s06/codec.js", Path: "/s06/codec.js"}, text, core.ScriptKindJS)
			root = sf.AsNode()
		case strings.HasPrefix(name, "literal/"):
			root = codecLiteral(f, name[8:])
			root.Loc = core.NewTextRange(-1, 2)
			root.Flags = 0xa5a5a5a5
		case name == "synthetic-expression":
			root = f.NewToken(ast.KindSyntheticExpression)
		case name == "raw-kind":
			root = f.NewToken(ast.Kind(-1))
			root.Loc = core.NewTextRange(-1, 2147483647)
			root.Flags = 0xffffffff
		default:
			root = codecSource(f, name)
			sf = root.AsSourceFile()
		}
		var err error
		if sf == nil {
			encoded, _, err = encoder.EncodeNode(root, nil)
		} else {
			encoded, _, err = encoder.EncodeSourceFile(sf)
		}
		if err == nil {
			codecBytes(s, "encode", encoded)
		}
		return err
	}) {
		return
	}
	var root *ast.Node
	if !s.stage("decode", func() error {
		var err error
		root, err = encoder.DecodeNodes(encoded)
		if err != nil {
			return err
		}
		var node any
		if root != nil {
			node = map[string]any{"kind": int16(root.Kind), "pos": root.Pos(), "end": root.End(), "flags": uint32(root.Flags)}
		}
		s.observe("decode", "root", map[string]any{"node": node})
		return nil
	}) {
		return
	}
	if !s.stage("decoded_tree", func() error { emitDecoded(s, root); return nil }) {
		return
	}
	s.stage("reencode", func() error {
		var raw []byte
		var err error
		if root != nil && root.Kind == ast.KindSourceFile {
			raw, _, err = encoder.EncodeSourceFile(root.AsSourceFile())
		} else {
			raw, _, err = encoder.EncodeNode(root, nil)
		}
		if err == nil {
			codecBytes(s, "reencode", raw)
		}
		return err
	})
}
