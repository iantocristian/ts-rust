package checker

// Access-only bridge for the S08 P1 storage-families trace. It executes the
// original constructors on a checker whose state is what NewChecker builds
// before installing closures, and counts the live storage the way
// data/s08/type-footprint.json prescribes. The driver validates the prepared
// checker against a real NewChecker before trusting it.

import (
	"encoding/hex"
	"fmt"
	"maps"
	"math"
	"reflect"
	"runtime"
	"slices"
	"strconv"
	"unsafe"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/jsnum"
)

type S08FamiliesOptions struct {
	StrictNullChecks           bool `json:"strict_null_checks"`
	ExactOptionalPropertyTypes bool `json:"exact_optional_property_types"`
}

type S08FamiliesMember struct {
	TextHex  string `json:"text_hex"`
	Type     int    `json:"type"`
	Optional bool   `json:"optional"`
	Readonly bool   `json:"readonly"`
}

type S08FamiliesAction struct {
	Op          string              `json:"op"`
	Name        string              `json:"name,omitempty"`
	TextHex     string              `json:"text_hex,omitempty"`
	BitsHex     string              `json:"bits_hex,omitempty"`
	Negative    bool                `json:"negative,omitempty"`
	Digits      string              `json:"digits,omitempty"`
	Root        int                 `json:"root,omitempty"`
	Types       []int               `json:"types,omitempty"`
	Reduction   string              `json:"reduction,omitempty"`
	AliasSymbol int                 `json:"alias_symbol,omitempty"`
	AliasArgs   []int               `json:"alias_args,omitempty"`
	Flags       uint32              `json:"flags,omitempty"`
	CheckFlags  uint32              `json:"check_flags,omitempty"`
	Symbol      *int                `json:"symbol,omitempty"`
	Elements    []string            `json:"elements,omitempty"`
	Readonly    bool                `json:"readonly,omitempty"`
	Target      int                 `json:"target,omitempty"`
	Args        []int               `json:"args,omitempty"`
	Members     []S08FamiliesMember `json:"members,omitempty"`
	Texts       []string            `json:"texts,omitempty"`
	Parameters  []int               `json:"parameters,omitempty"`
	Return      int                 `json:"return,omitempty"`
	Type        int                 `json:"type,omitempty"`
}

// The Checker type fields NewChecker creates before initializeClosures, in
// creation order; the Rust port asserts the same ids and flags for each.
var s08FamiliesNamed = []struct {
	name string
	get  func(c *Checker) *Type
}{
	{"anyType", func(c *Checker) *Type { return c.anyType }},
	{"autoType", func(c *Checker) *Type { return c.autoType }},
	{"wildcardType", func(c *Checker) *Type { return c.wildcardType }},
	{"blockedStringType", func(c *Checker) *Type { return c.blockedStringType }},
	{"errorType", func(c *Checker) *Type { return c.errorType }},
	{"unresolvedType", func(c *Checker) *Type { return c.unresolvedType }},
	{"nonInferrableAnyType", func(c *Checker) *Type { return c.nonInferrableAnyType }},
	{"intrinsicMarkerType", func(c *Checker) *Type { return c.intrinsicMarkerType }},
	{"unknownType", func(c *Checker) *Type { return c.unknownType }},
	{"undefinedType", func(c *Checker) *Type { return c.undefinedType }},
	{"undefinedWideningType", func(c *Checker) *Type { return c.undefinedWideningType }},
	{"missingType", func(c *Checker) *Type { return c.missingType }},
	{"undefinedOrMissingType", func(c *Checker) *Type { return c.undefinedOrMissingType }},
	{"optionalType", func(c *Checker) *Type { return c.optionalType }},
	{"nullType", func(c *Checker) *Type { return c.nullType }},
	{"nullWideningType", func(c *Checker) *Type { return c.nullWideningType }},
	{"stringType", func(c *Checker) *Type { return c.stringType }},
	{"numberType", func(c *Checker) *Type { return c.numberType }},
	{"bigintType", func(c *Checker) *Type { return c.bigintType }},
	{"regularFalseType", func(c *Checker) *Type { return c.regularFalseType }},
	{"falseType", func(c *Checker) *Type { return c.falseType }},
	{"regularTrueType", func(c *Checker) *Type { return c.regularTrueType }},
	{"trueType", func(c *Checker) *Type { return c.trueType }},
	{"booleanType", func(c *Checker) *Type { return c.booleanType }},
	{"esSymbolType", func(c *Checker) *Type { return c.esSymbolType }},
	{"voidType", func(c *Checker) *Type { return c.voidType }},
	{"neverType", func(c *Checker) *Type { return c.neverType }},
	{"silentNeverType", func(c *Checker) *Type { return c.silentNeverType }},
	{"implicitNeverType", func(c *Checker) *Type { return c.implicitNeverType }},
	{"unreachableNeverType", func(c *Checker) *Type { return c.unreachableNeverType }},
	{"nonPrimitiveType", func(c *Checker) *Type { return c.nonPrimitiveType }},
	{"stringOrNumberType", func(c *Checker) *Type { return c.stringOrNumberType }},
	{"stringNumberSymbolType", func(c *Checker) *Type { return c.stringNumberSymbolType }},
	{"numberOrBigIntType", func(c *Checker) *Type { return c.numberOrBigIntType }},
	{"numericStringType", func(c *Checker) *Type { return c.numericStringType }},
	{"templateConstraintType", func(c *Checker) *Type { return c.templateConstraintType }},
	{"uniqueLiteralType", func(c *Checker) *Type { return c.uniqueLiteralType }},
	{"emptyObjectType", func(c *Checker) *Type { return c.emptyObjectType }},
	{"emptyJsxObjectType", func(c *Checker) *Type { return c.emptyJsxObjectType }},
	{"emptyFreshJsxObjectType", func(c *Checker) *Type { return c.emptyFreshJsxObjectType }},
	{"emptyTypeLiteralType", func(c *Checker) *Type { return c.emptyTypeLiteralType }},
	{"unknownEmptyObjectType", func(c *Checker) *Type { return c.unknownEmptyObjectType }},
	{"unknownUnionType", func(c *Checker) *Type { return c.unknownUnionType }},
	{"emptyGenericType", func(c *Checker) *Type { return c.emptyGenericType }},
	{"anyFunctionType", func(c *Checker) *Type { return c.anyFunctionType }},
	{"noConstraintType", func(c *Checker) *Type { return c.noConstraintType }},
	{"circularConstraintType", func(c *Checker) *Type { return c.circularConstraintType }},
	{"resolvingDefaultType", func(c *Checker) *Type { return c.resolvingDefaultType }},
	{"markerSuperType", func(c *Checker) *Type { return c.markerSuperType }},
	{"markerSubType", func(c *Checker) *Type { return c.markerSubType }},
	{"markerOtherType", func(c *Checker) *Type { return c.markerOtherType }},
	{"markerSuperTypeForCheck", func(c *Checker) *Type { return c.markerSuperTypeForCheck }},
	{"markerSubTypeForCheck", func(c *Checker) *Type { return c.markerSubTypeForCheck }},
	{"emptyStringType", func(c *Checker) *Type { return c.emptyStringType }},
	{"zeroType", func(c *Checker) *Type { return c.zeroType }},
	{"zeroBigIntType", func(c *Checker) *Type { return c.zeroBigIntType }},
	{"typeofType", func(c *Checker) *Type { return c.typeofType }},
}

// S08FamiliesNamed observes the named types of any checker: id, flags, object flags.
func S08FamiliesNamed(c *Checker) map[string]any {
	named := map[string]any{}
	for _, field := range s08FamiliesNamed {
		t := field.get(c)
		named[field.name] = map[string]any{"id": t.id, "flags": t.flags, "object_flags": t.objectFlags}
	}
	return named
}

// S08FamiliesCounts reports the checker's creation counters.
func S08FamiliesCounts(c *Checker) map[string]any {
	return map[string]any{"types": c.TypeCount, "symbols": c.SymbolCount, "signatures": c.SignatureCount}
}

// S08FamiliesReplica prepares a checker the way NewChecker does before it
// installs closures, resolvers and initializeChecker. The statements are the
// original constructor's, in its order; the driver proves that by comparing
// every named type with a real NewChecker over an empty program.
func S08FamiliesReplica(options S08FamiliesOptions) *Checker {
	c := &Checker{}
	c.fileIndexMap = map[*ast.SourceFile]int{}
	c.compareSymbols = c.compareSymbolsWorker
	c.compareSymbolChains = c.compareSymbolChainsWorker
	c.strictNullChecks = options.StrictNullChecks
	c.exactOptionalPropertyTypes = options.ExactOptionalPropertyTypes
	c.arrayVariances = []VarianceFlags{VarianceFlagsCovariant}
	c.globals = make(ast.SymbolTable)
	c.stringLiteralTypes = make(map[string]*Type)
	c.numberLiteralTypes = make(map[jsnum.Number]*Type)
	c.bigintLiteralTypes = make(map[jsnum.PseudoBigInt]*Type)
	c.enumLiteralTypes = make(map[EnumLiteralKey]*Type)
	c.enumNaNLiteralTypes = make(map[*ast.Symbol]*Type)
	c.indexedAccessTypes = make(map[CacheHashKey]*Type)
	c.templateLiteralTypes = make(map[CacheHashKey]*Type)
	c.stringMappingTypes = make(map[StringMappingKey]*Type)
	c.uniqueESSymbolTypes = make(map[*ast.Symbol]*Type)
	c.thisExpandoKinds = make(map[*ast.Symbol]thisAssignmentDeclarationKind)
	c.thisExpandoLocations = make(map[*ast.Symbol]*ast.Node)
	c.subtypeReductionCache = make(map[CacheHashKey][]*Type)
	c.cachedTypes = make(map[CachedTypeKey]*Type)
	c.cachedSignatures = make(map[CachedSignatureKey]*Signature)
	c.undefinedProperties = make(map[string]*ast.Symbol)
	c.narrowedTypes = make(map[NarrowedTypeKey]*Type)
	c.assignmentReducedTypes = make(map[AssignmentReducedKey]*Type)
	c.discriminatedContextualTypes = make(map[DiscriminatedContextualTypeKey]*Type)
	c.instantiationExpressionTypes = make(map[InstantiationExpressionKey]*Type)
	c.substitutionTypes = make(map[SubstitutionTypeKey]*Type)
	c.reverseMappedCache = make(map[ReverseMappedTypeKey]*Type)
	c.reverseHomomorphicMappedCache = make(map[ReverseMappedTypeKey]*Type)
	c.iterationTypesCache = make(map[IterationTypesKey]IterationTypes)
	c.undefinedSymbol = c.newSymbol(ast.SymbolFlagsProperty, "undefined")
	c.argumentsSymbol = c.newSymbol(ast.SymbolFlagsProperty, "arguments")
	c.requireSymbol = c.newSymbol(ast.SymbolFlagsProperty, "require")
	c.unknownSymbol = c.newSymbol(ast.SymbolFlagsProperty, "unknown")
	c.unresolvedSymbols = make(map[string]*ast.Symbol)
	c.errorTypes = make(map[CacheHashKey]*Type)
	c.moduleSymbols = make(map[*ast.Node]*ast.Symbol)
	c.globalThisSymbol = c.newSymbolEx(ast.SymbolFlagsModule, "globalThis", ast.CheckFlagsReadonly)
	c.globalThisSymbol.Exports = c.globals
	c.globals[c.globalThisSymbol.Name] = c.globalThisSymbol
	c.tupleTypes = make(map[CacheHashKey]*Type)
	c.unionTypes = make(map[CacheHashKey]*Type)
	c.unionOfUnionTypes = make(map[UnionOfUnionKey]*Type)
	c.intersectionTypes = make(map[CacheHashKey]*Type)
	c.propertiesTypes = make(map[PropertiesTypesKey]*Type)
	c.mergedSymbols = make(map[*ast.Symbol]*ast.Symbol)
	c.patternForType = make(map[*Type]*ast.Node)
	c.contextFreeTypes = make(map[*ast.Node]*Type)
	c.anyType = c.newIntrinsicType(TypeFlagsAny, "any")
	c.autoType = c.newIntrinsicTypeEx(TypeFlagsAny, "any", ObjectFlagsNonInferrableType)
	c.wildcardType = c.newIntrinsicType(TypeFlagsAny, "any")
	c.blockedStringType = c.newIntrinsicType(TypeFlagsAny, "any")
	c.errorType = c.newIntrinsicType(TypeFlagsAny, "error")
	c.unresolvedType = c.newIntrinsicType(TypeFlagsAny, "unresolved")
	c.nonInferrableAnyType = c.newIntrinsicTypeEx(TypeFlagsAny, "any", ObjectFlagsContainsWideningType)
	c.intrinsicMarkerType = c.newIntrinsicType(TypeFlagsAny, "intrinsic")
	c.unknownType = c.newIntrinsicType(TypeFlagsUnknown, "unknown")
	c.undefinedType = c.newIntrinsicType(TypeFlagsUndefined, "undefined")
	c.undefinedWideningType = c.createWideningType(c.undefinedType)
	c.missingType = c.newIntrinsicType(TypeFlagsUndefined, "undefined")
	c.undefinedOrMissingType = core.IfElse(c.exactOptionalPropertyTypes, c.missingType, c.undefinedType)
	c.optionalType = c.newIntrinsicType(TypeFlagsUndefined, "undefined")
	c.nullType = c.newIntrinsicType(TypeFlagsNull, "null")
	c.nullWideningType = c.createWideningType(c.nullType)
	c.stringType = c.newIntrinsicType(TypeFlagsString, "string")
	c.numberType = c.newIntrinsicType(TypeFlagsNumber, "number")
	c.bigintType = c.newIntrinsicType(TypeFlagsBigInt, "bigint")
	c.regularFalseType = c.newLiteralType(TypeFlagsBooleanLiteral, false, nil)
	c.falseType = c.newLiteralType(TypeFlagsBooleanLiteral, false, c.regularFalseType)
	c.regularFalseType.AsLiteralType().freshType = c.falseType
	c.falseType.AsLiteralType().freshType = c.falseType
	c.regularTrueType = c.newLiteralType(TypeFlagsBooleanLiteral, true, nil)
	c.trueType = c.newLiteralType(TypeFlagsBooleanLiteral, true, c.regularTrueType)
	c.regularTrueType.AsLiteralType().freshType = c.trueType
	c.trueType.AsLiteralType().freshType = c.trueType
	c.booleanType = c.getUnionType([]*Type{c.regularFalseType, c.regularTrueType})
	c.esSymbolType = c.newIntrinsicType(TypeFlagsESSymbol, "symbol")
	c.voidType = c.newIntrinsicType(TypeFlagsVoid, "void")
	c.neverType = c.newIntrinsicType(TypeFlagsNever, "never")
	c.silentNeverType = c.newIntrinsicTypeEx(TypeFlagsNever, "never", ObjectFlagsNonInferrableType)
	c.implicitNeverType = c.newIntrinsicType(TypeFlagsNever, "never")
	c.unreachableNeverType = c.newIntrinsicType(TypeFlagsNever, "never")
	c.nonPrimitiveType = c.newIntrinsicType(TypeFlagsNonPrimitive, "object")
	c.stringOrNumberType = c.getUnionType([]*Type{c.stringType, c.numberType})
	c.stringNumberSymbolType = c.getUnionType([]*Type{c.stringType, c.numberType, c.esSymbolType})
	c.numberOrBigIntType = c.getUnionType([]*Type{c.numberType, c.bigintType})
	c.numericStringType = c.getTemplateLiteralType([]string{"", ""}, []*Type{c.numberType})
	c.templateConstraintType = c.getUnionType([]*Type{c.stringType, c.numberType, c.booleanType, c.bigintType, c.nullType, c.undefinedType})
	c.uniqueLiteralType = c.newIntrinsicType(TypeFlagsNever, "never")
	c.emptyObjectType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.emptyJsxObjectType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.emptyFreshJsxObjectType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.emptyTypeLiteralType = c.newAnonymousType(c.newSymbol(ast.SymbolFlagsTypeLiteral, ast.InternalSymbolNameType), nil, nil, nil, nil)
	c.unknownEmptyObjectType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.unknownUnionType = c.createUnknownUnionType()
	c.emptyGenericType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.emptyGenericType.AsObjectType().instantiations = make(map[CacheHashKey]*Type)
	c.anyFunctionType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.anyFunctionType.objectFlags |= ObjectFlagsNonInferrableType
	c.noConstraintType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.circularConstraintType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.resolvingDefaultType = c.newAnonymousType(nil, nil, nil, nil, nil)
	c.markerSuperType = c.newTypeParameter(nil)
	c.markerSubType = c.newTypeParameter(nil)
	c.markerSubType.AsTypeParameter().constraint = c.markerSuperType
	c.markerOtherType = c.newTypeParameter(nil)
	c.markerSuperTypeForCheck = c.newTypeParameter(nil)
	c.markerSubTypeForCheck = c.newTypeParameter(nil)
	c.markerSubTypeForCheck.AsTypeParameter().constraint = c.markerSuperTypeForCheck
	c.noTypePredicate = &TypePredicate{kind: TypePredicateKindIdentifier, parameterIndex: 0, parameterName: "<<unresolved>>", t: c.anyType}
	c.anySignature = c.newSignature(SignatureFlagsNone, nil, nil, nil, nil, c.anyType, nil, 0)
	c.unknownSignature = c.newSignature(SignatureFlagsNone, nil, nil, nil, nil, c.errorType, nil, 0)
	c.resolvingSignature = c.newSignature(SignatureFlagsNone, nil, nil, nil, nil, c.anyType, nil, 0)
	c.silentNeverSignature = c.newSignature(SignatureFlagsNone, nil, nil, nil, nil, c.silentNeverType, nil, 0)
	c.cachedArgumentsReferenced = make(map[*ast.Node]bool)
	c.enumNumberIndexInfo = &IndexInfo{keyType: c.numberType, valueType: c.stringType, isReadonly: true}
	c.anyBaseTypeIndexInfo = &IndexInfo{keyType: c.stringType, valueType: c.anyType, isReadonly: false}
	c.emptyStringType = c.getStringLiteralType("")
	c.zeroType = c.getNumberLiteralType(0)
	c.zeroBigIntType = c.getBigIntLiteralType(jsnum.PseudoBigInt{})
	c.typeofType = c.getUnionType(core.Map(slices.Sorted(maps.Keys(typeofNEFacts)), c.getStringLiteralType))
	return c
}

type s08Root struct {
	t    *Type
	sym  *ast.Symbol
	sig  *Signature
	node *ast.Node
}

// S08FamiliesRoots holds the results of a trace until they are observed.
type S08FamiliesRoots struct {
	roots []s08Root
}

// S08FamiliesExecute runs the frozen actions with the original constructors;
// observation is separate so allocation traffic can be measured around this call.
func S08FamiliesExecute(c *Checker, actions []S08FamiliesAction) *S08FamiliesRoots {
	roots := make([]s08Root, 0, len(actions))
	typeRoot := func(i int) *Type {
		if roots[i].t == nil {
			panic("trace root is not a type")
		}
		return roots[i].t
	}
	symbolRoot := func(i int) *ast.Symbol {
		if roots[i].sym == nil {
			panic("trace root is not a symbol")
		}
		return roots[i].sym
	}
	typeRoots := func(indexes []int) []*Type {
		types := make([]*Type, 0, len(indexes))
		for _, i := range indexes {
			types = append(types, typeRoot(i))
		}
		return types
	}
	decode := func(text string) string {
		value, err := hex.DecodeString(text)
		if err != nil {
			panic(err)
		}
		return string(value)
	}
	for _, action := range actions {
		var root s08Root
		switch action.Op {
		case "builtin":
			for _, field := range s08FamiliesNamed {
				if field.name == action.Name {
					root.t = field.get(c)
				}
			}
			if root.t == nil {
				panic("unknown builtin " + action.Name)
			}
		case "string":
			root.t = c.getStringLiteralType(decode(action.TextHex))
		case "number":
			bits, err := strconv.ParseUint(action.BitsHex, 16, 64)
			if err != nil {
				panic(err)
			}
			root.t = c.getNumberLiteralType(jsnum.Number(math.Float64frombits(bits)))
		case "bigint":
			root.t = c.getBigIntLiteralType(jsnum.NewPseudoBigInt(action.Digits, action.Negative))
		case "fresh":
			root.t = c.getFreshTypeOfLiteralType(typeRoot(action.Root))
		case "regular":
			root.t = c.getRegularTypeOfLiteralType(typeRoot(action.Root))
		case "union":
			reduction := UnionReductionLiteral
			switch action.Reduction {
			case "literal":
			case "none":
				reduction = UnionReductionNone
			default:
				panic("unsupported union reduction")
			}
			root.t = c.getUnionTypeEx(typeRoots(action.Types), reduction, nil, nil)
		case "union_alias":
			alias := &TypeAlias{symbol: symbolRoot(action.AliasSymbol), typeArguments: typeRoots(action.AliasArgs)}
			root.t = c.getUnionTypeEx(typeRoots(action.Types), UnionReductionLiteral, alias, nil)
		case "symbol":
			root.sym = c.newSymbolEx(ast.SymbolFlags(action.Flags), decode(action.TextHex), ast.CheckFlags(action.CheckFlags))
		case "type_parameter":
			var symbol *ast.Symbol
			if action.Symbol != nil {
				symbol = symbolRoot(*action.Symbol)
			}
			root.t = c.newTypeParameter(symbol)
		case "tuple_target":
			infos := make([]TupleElementInfo, 0, len(action.Elements))
			for _, element := range action.Elements {
				var flags ElementFlags
				switch element {
				case "required":
					flags = ElementFlagsRequired
				case "optional":
					flags = ElementFlagsOptional
				case "rest":
					flags = ElementFlagsRest
				case "variadic":
					flags = ElementFlagsVariadic
				default:
					panic("unknown element kind")
				}
				infos = append(infos, TupleElementInfo{flags: flags})
			}
			root.t = c.getTupleTargetType(infos, action.Readonly)
		case "tuple":
			root.t = c.createTupleType(typeRoots(action.Types))
		case "reference":
			root.t = c.createTypeReference(typeRoot(action.Target), typeRoots(action.Args))
		case "anonymous":
			var symbol *ast.Symbol
			if action.Symbol != nil {
				symbol = symbolRoot(*action.Symbol)
			}
			var members ast.SymbolTable
			if len(action.Members) != 0 {
				members = make(ast.SymbolTable, len(action.Members))
				for _, member := range action.Members {
					flags := ast.SymbolFlagsProperty
					if member.Optional {
						flags |= ast.SymbolFlagsOptional
					}
					var checkFlags ast.CheckFlags
					if member.Readonly {
						checkFlags = ast.CheckFlagsReadonly
					}
					property := c.newSymbolEx(flags, decode(member.TextHex), checkFlags)
					c.valueSymbolLinks.Get(property).resolvedType = typeRoot(member.Type)
					members[property.Name] = property
				}
			}
			root.t = c.newAnonymousType(symbol, members, nil, nil, nil)
		case "template":
			texts := make([]string, 0, len(action.Texts))
			for _, text := range action.Texts {
				texts = append(texts, decode(text))
			}
			root.t = c.getTemplateLiteralType(texts, typeRoots(action.Types))
		case "call_signature":
			var parameters []*ast.Symbol
			for _, i := range action.Parameters {
				parameters = append(parameters, symbolRoot(i))
			}
			root.sig = c.newCallSignature(nil, nil, parameters, typeRoot(action.Return))
		case "synthetic_expression":
			root.node = c.factory.NewSyntheticExpression(typeRoot(action.Type), false, nil)
		default:
			panic("unknown action " + action.Op)
		}
		roots = append(roots, root)
	}
	return &S08FamiliesRoots{roots: roots}
}

// S08FamiliesObserve records every result and returns the type roots the census retains.
func S08FamiliesObserve(c *Checker, executed *S08FamiliesRoots) ([]map[string]any, []*Type) {
	roots := executed.roots
	observations := make([]map[string]any, 0, len(roots))
	var typeRootsOut []*Type
	for _, root := range roots {
		switch {
		case root.sym != nil:
			observations = append(observations, map[string]any{"kind": "symbol", "flags": root.sym.Flags, "check_flags": root.sym.CheckFlags, "name_hex": hex.EncodeToString([]byte(root.sym.Name))})
		case root.sig != nil:
			var ret uint32
			if root.sig.resolvedReturnType != nil {
				ret = uint32(root.sig.resolvedReturnType.id)
			}
			observations = append(observations, map[string]any{"kind": "signature", "id": root.sig.id, "return": ret, "min_argument_count": root.sig.minArgumentCount,
				"parameter_count": len(root.sig.parameters), "declaration_is_function_type": root.sig.declaration != nil && root.sig.declaration.Kind == ast.KindFunctionType})
		case root.node != nil:
			var t uint32
			if typ, ok := root.node.AsSyntheticExpression().Type.(*Type); ok {
				t = uint32(typ.id)
			}
			observations = append(observations, map[string]any{"kind": "node", "is_synthetic_expression": root.node.Kind == ast.KindSyntheticExpression, "type": t})
		default:
			observations = append(observations, s08ObserveType(c, root.t))
			typeRootsOut = append(typeRootsOut, root.t)
		}
	}
	return observations, typeRootsOut
}

func s08Id(t *Type) uint32 {
	if t == nil {
		return 0
	}
	return uint32(t.id)
}

func s08Ids(types []*Type) []uint32 {
	ids := make([]uint32, 0, len(types))
	for _, t := range types {
		ids = append(ids, s08Id(t))
	}
	return ids
}

func s08Properties(c *Checker, properties []*ast.Symbol) [][2]any {
	out := make([][2]any, 0, len(properties))
	for _, property := range properties {
		var resolved uint32
		if links := c.valueSymbolLinks.TryGet(property); links != nil {
			resolved = s08Id(links.resolvedType)
		}
		out = append(out, [2]any{hex.EncodeToString([]byte(property.Name)), resolved})
	}
	return out
}

func s08ObserveType(c *Checker, t *Type) map[string]any {
	row := map[string]any{"kind": "type", "id": t.id, "flags": t.flags, "object_flags": t.objectFlags}
	if t.symbol != nil {
		row["symbol_name_hex"] = hex.EncodeToString([]byte(t.symbol.Name))
	} else {
		row["symbol_name_hex"] = nil
	}
	if t.alias != nil {
		row["alias_symbol_name_hex"] = hex.EncodeToString([]byte(t.alias.symbol.Name))
		row["alias_args"] = s08Ids(t.alias.typeArguments)
	} else {
		row["alias_symbol_name_hex"] = nil
		row["alias_args"] = []uint32{}
	}
	switch data := t.data.(type) {
	case *IntrinsicType:
		row["payload"] = "intrinsic"
		row["name_hex"] = hex.EncodeToString([]byte(data.intrinsicName))
	case *LiteralType:
		row["payload"] = "literal"
		switch value := data.value.(type) {
		case string:
			row["value"] = "string:" + hex.EncodeToString([]byte(value))
		case jsnum.Number:
			row["value"] = fmt.Sprintf("number:%016x", math.Float64bits(float64(value)))
		case bool:
			row["value"] = fmt.Sprintf("boolean:%t", value)
		case jsnum.PseudoBigInt:
			row["value"] = "bigint:" + value.String()
		default:
			row["value"] = "enum:computed"
		}
		row["fresh"] = s08Id(data.freshType)
		row["regular"] = s08Id(data.regularType)
	case *UnionType:
		row["payload"] = "union"
		row["types"] = s08Ids(data.types)
		row["origin"] = s08Id(data.origin)
	case *TypeParameter:
		row["payload"] = "type_parameter"
		row["constraint"] = s08Id(data.constraint)
		row["is_this_type"] = data.isThisType
	case *TemplateLiteralType:
		row["payload"] = "template_literal"
		texts := make([]string, 0, len(data.texts))
		for _, text := range data.texts {
			texts = append(texts, hex.EncodeToString([]byte(text)))
		}
		row["texts"] = texts
		row["types"] = s08Ids(data.types)
	case *ObjectType, *TypeReference, *InterfaceType, *TupleType:
		object := t.AsObjectType()
		row["target"] = s08Id(object.target)
		row["properties"] = s08Properties(c, object.properties)
		if reference := t.AsTypeReference(); reference != nil {
			row["type_arguments"] = s08Ids(reference.resolvedTypeArguments)
		} else {
			row["type_arguments"] = []uint32{}
		}
		if intf := t.AsInterfaceType(); intf != nil {
			row["this_type"] = s08Id(intf.thisType)
			row["type_parameters"] = s08Ids(intf.TypeParameters())
			members := [][2]any{}
			for name, symbol := range intf.declaredMembers {
				var resolved uint32
				if links := c.valueSymbolLinks.TryGet(symbol); links != nil {
					resolved = s08Id(links.resolvedType)
				}
				members = append(members, [2]any{hex.EncodeToString([]byte(name)), resolved})
			}
			slices.SortFunc(members, func(a, b [2]any) int {
				if c := compareStrings(a[0].(string), b[0].(string)); c != 0 {
					return c
				}
				return int(a[1].(uint32)) - int(b[1].(uint32))
			})
			row["declared_members"] = members
		}
		switch data := t.data.(type) {
		case *ObjectType:
			row["payload"] = "anonymous"
		case *TypeReference:
			row["payload"] = "reference"
		case *InterfaceType:
			row["payload"] = "interface"
		case *TupleType:
			row["payload"] = "tuple"
			row["element_flags"] = data.ElementFlags()
			row["min_length"] = data.minLength
			row["fixed_length"] = data.fixedLength
			row["combined_flags"] = data.combinedFlags
			row["readonly"] = data.readonly
		}
	default:
		panic(fmt.Sprintf("unexpected payload %T", t.data))
	}
	return row
}

func compareStrings(a, b string) int {
	switch {
	case a < b:
		return -1
	case a > b:
		return 1
	}
	return 0
}

// Structural census, following data/s08/type-footprint.json: each allocation
// once, with its capacity; strings and slices by backing identity; maps by the
// retained bytes of an identically typed and sized replica.

type s08Census struct {
	families map[string]*s08Family
	seen     map[uintptr]bool
	tables   map[uintptr]bool
}

type s08Family struct {
	Count int64 `json:"count"`
	Bytes int64 `json:"bytes"`
}

func (census *s08Census) add(family string, count, bytes int64) {
	entry := census.families[family]
	if entry == nil {
		entry = &s08Family{}
		census.families[family] = entry
	}
	entry.Count += count
	entry.Bytes += bytes
}

func (census *s08Census) slice(family string, data unsafe.Pointer, capacity, elem int) {
	if capacity == 0 || data == nil {
		return
	}
	key := uintptr(data)
	if census.seen[key] {
		return
	}
	census.seen[key] = true
	census.add(family, 0, int64(capacity*elem))
}

func (census *s08Census) text(family string, s string) {
	if len(s) == 0 {
		return
	}
	key := uintptr(unsafe.Pointer(unsafe.StringData(s)))
	if census.seen[key] {
		return
	}
	census.seen[key] = true
	census.add(family, 0, int64(len(s)))
}

func typeSlice[T any](census *s08Census, family string, s []T) {
	var zero T
	census.slice(family, unsafe.Pointer(unsafe.SliceData(s)), cap(s), int(unsafe.Sizeof(zero)))
}

// Every family both runtimes report, so an absent family is a mismatch, not a zero.
var s08FamilyNames = []string{"type_records", "intrinsic", "literal", "unique_es_symbol", "anonymous", "reference", "interface", "tuple", "union", "intersection", "type_parameter", "template_literal", "alias", "type_lists", "type_caches",
	"symbols", "symbol_tables", "signatures", "index_infos", "type_predicates", "value_symbol_links", "synthetic_expression_links", "checker_ast"}

// arenaCapacity replays core.Arena's chunk growth for count records of T and
// returns the slots allocated across every chunk, including the size-class
// rounding slices.Grow applies; the chunks stay reachable through the records
// they hold even after the arena moves on.
func arenaCapacity[T any](count int) int {
	total := 0
	for _, chunk := range arenaChunks[T](count) {
		total += chunk
	}
	return total
}

// arenaChunks lists the chunk capacities core.Arena allocates for count records.
func arenaChunks[T any](count int) []int {
	var arena core.Arena[T]
	data := unexportedField(&arena, "data")
	chunks := []int{}
	lastCap := 0
	for range count {
		arena.New()
		if c := data.Cap(); c != lastCap {
			chunks = append(chunks, c)
			lastCap = c
		}
	}
	return chunks
}

// unexportedField reads a field of another package's struct for observation.
func unexportedField(ptr any, name string) reflect.Value {
	field := reflect.ValueOf(ptr).Elem().FieldByName(name)
	if !field.IsValid() {
		panic("missing field " + name)
	}
	return reflect.NewAt(field.Type(), unsafe.Pointer(field.UnsafeAddr())).Elem()
}

// mapBytes is what the runtime allocates for a map of the same type sized for
// the same entry count: the header and its groups, measured as allocation
// traffic while an identically typed replica is filled from pre-collected keys
// and values. Sizing by hint avoids growth garbage, and reading keys first keeps
// reflection's own copies out of the interval, so the number is deterministic.
func mapBytes(m any) int64 {
	v := reflect.ValueOf(m)
	if v.Kind() != reflect.Map || v.IsNil() {
		return 0
	}
	keys := make([]reflect.Value, 0, v.Len())
	values := make([]reflect.Value, 0, v.Len())
	iter := v.MapRange()
	for iter.Next() {
		keys = append(keys, iter.Key())
		values = append(values, iter.Value())
	}
	var before, after runtime.MemStats
	runtime.ReadMemStats(&before)
	replica := reflect.MakeMapWithSize(v.Type(), len(keys))
	for i := range keys {
		replica.SetMapIndex(keys[i], values[i])
	}
	runtime.ReadMemStats(&after)
	runtime.KeepAlive(replica)
	return int64(after.TotalAlloc - before.TotalAlloc)
}

func (census *s08Census) symbolTable(family string, table ast.SymbolTable) {
	if table == nil {
		return
	}
	key := reflect.ValueOf(table).Pointer()
	if census.tables[key] {
		return
	}
	census.tables[key] = true
	census.add(family, 1, mapBytes(table))
}

func (census *s08Census) cache(m any, keyBytes int64) {
	v := reflect.ValueOf(m)
	census.add("type_caches", int64(v.Len()), mapBytes(m)+keyBytes)
}

// S08FamiliesCensus counts the storage reachable from the checker's named types,
// interning caches and the retained trace roots.
func S08FamiliesCensus(c *Checker, roots []*Type) map[string]any {
	census := &s08Census{families: map[string]*s08Family{}, seen: map[uintptr]bool{}, tables: map[uintptr]bool{}}
	for _, name := range s08FamilyNames {
		census.add(name, 0, 0)
	}
	visited := map[*Type]bool{}
	var stack []*Type
	for _, field := range s08FamiliesNamed {
		stack = append(stack, field.get(c))
	}
	stack = append(stack, roots...)
	for _, t := range c.stringLiteralTypes {
		stack = append(stack, t)
	}
	for _, t := range c.numberLiteralTypes {
		stack = append(stack, t)
	}
	if c.nanType != nil {
		stack = append(stack, c.nanType)
	}
	for _, t := range c.bigintLiteralTypes {
		stack = append(stack, t)
	}
	for _, m := range []map[CacheHashKey]*Type{c.unionTypes, c.tupleTypes, c.intersectionTypes, c.templateLiteralTypes} {
		for _, t := range m {
			stack = append(stack, t)
		}
	}
	for _, t := range c.unionOfUnionTypes {
		stack = append(stack, t)
	}
	symbols := map[*ast.Symbol]bool{}
	signatures := map[*Signature]bool{}
	for len(stack) != 0 {
		t := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		if t == nil || visited[t] {
			continue
		}
		visited[t] = true
		if t.alias != nil {
			census.add("alias", 1, int64(unsafe.Sizeof(TypeAlias{})))
			typeSlice(census, "type_lists", t.alias.typeArguments)
			stack = append(stack, t.alias.typeArguments...)
		}
		structured := func(family string, data *StructuredType) {
			census.symbolTable("symbol_tables", data.members)
			typeSlice(census, family, data.properties)
			typeSlice(census, family, data.signatures)
			typeSlice(census, family, data.indexInfos)
			for _, property := range data.properties {
				symbols[property] = true
				if links := c.valueSymbolLinks.TryGet(property); links != nil {
					stack = append(stack, links.resolvedType, links.writeType, links.nameType, links.containingType)
				}
			}
			for _, signature := range data.signatures {
				signatures[signature] = true
			}
			for _, info := range data.indexInfos {
				stack = append(stack, info.keyType, info.valueType)
			}
			stack = append(stack, data.resolvedBaseConstraint, data.objectTypeWithoutAbstractConstructSignatures)
		}
		object := func(family string, data *ObjectType) {
			structured(family, &data.StructuredType)
			stack = append(stack, data.target)
			if data.instantiations != nil {
				census.cache(data.instantiations, 0)
				for _, t := range data.instantiations {
					stack = append(stack, t)
				}
			}
		}
		reference := func(family string, data *TypeReference) {
			object(family, &data.ObjectType)
			typeSlice(census, "type_lists", data.resolvedTypeArguments)
			stack = append(stack, data.resolvedTypeArguments...)
		}
		intf := func(family string, data *InterfaceType) {
			reference(family, &data.TypeReference)
			typeSlice(census, "type_lists", data.allTypeParameters)
			typeSlice(census, "type_lists", data.resolvedBaseTypes)
			typeSlice(census, family, data.declaredCallSignatures)
			typeSlice(census, family, data.declaredConstructSignatures)
			typeSlice(census, family, data.declaredIndexInfos)
			census.symbolTable("symbol_tables", data.declaredMembers)
			for _, symbol := range data.declaredMembers {
				symbols[symbol] = true
				if links := c.valueSymbolLinks.TryGet(symbol); links != nil {
					stack = append(stack, links.resolvedType)
				}
			}
			stack = append(stack, data.allTypeParameters...)
			stack = append(stack, data.resolvedBaseTypes...)
			stack = append(stack, data.thisType, data.resolvedBaseConstructorType)
		}
		if t.symbol != nil {
			symbols[t.symbol] = true
		}
		switch data := t.data.(type) {
		case *IntrinsicType:
			census.add("intrinsic", 1, int64(unsafe.Sizeof(IntrinsicType{})))
			census.text("intrinsic", data.intrinsicName)
		case *LiteralType:
			census.add("literal", 1, int64(unsafe.Sizeof(LiteralType{})))
			switch value := data.value.(type) {
			case string:
				// The interface boxes a string header; its bytes are shared with the intern key.
				census.add("literal", 0, int64(unsafe.Sizeof(value)))
				census.text("literal", value)
			case jsnum.Number:
				census.add("literal", 0, int64(unsafe.Sizeof(value)))
			case jsnum.PseudoBigInt:
				census.add("literal", 0, int64(unsafe.Sizeof(value)))
				census.text("literal", value.Base10Value)
			case bool:
				// Booleans box to the runtime's static table.
			}
			stack = append(stack, data.freshType, data.regularType)
		case *UniqueESSymbolType:
			census.add("unique_es_symbol", 1, int64(unsafe.Sizeof(UniqueESSymbolType{})))
			census.text("unique_es_symbol", data.name)
		case *ObjectType:
			census.add("anonymous", 1, int64(unsafe.Sizeof(ObjectType{})))
			object("anonymous", data)
		case *TypeReference:
			census.add("reference", 1, int64(unsafe.Sizeof(TypeReference{})))
			reference("reference", data)
		case *InterfaceType:
			census.add("interface", 1, int64(unsafe.Sizeof(InterfaceType{})))
			intf("interface", data)
		case *TupleType:
			census.add("tuple", 1, int64(unsafe.Sizeof(TupleType{})))
			intf("tuple", &data.InterfaceType)
			typeSlice(census, "tuple", data.elementInfos)
		case *UnionType:
			census.add("union", 1, int64(unsafe.Sizeof(UnionType{})))
			structured("union", &data.StructuredType)
			typeSlice(census, "type_lists", data.types)
			typeSlice(census, "union", data.resolvedProperties)
			census.symbolTable("union", data.propertyCache)
			census.symbolTable("union", data.propertyCacheWithoutFunctionPropertyAugment)
			census.text("union", data.keyPropertyName)
			if data.constituentMap != nil {
				census.add("union", 0, mapBytes(data.constituentMap))
			}
			stack = append(stack, data.types...)
			stack = append(stack, data.origin, data.regularType, data.resolvedReducedType)
		case *IntersectionType:
			census.add("intersection", 1, int64(unsafe.Sizeof(IntersectionType{})))
			structured("union", &data.StructuredType)
			typeSlice(census, "type_lists", data.types)
			stack = append(stack, data.types...)
			stack = append(stack, data.resolvedApparentType, data.uniqueLiteralFilledInstantiation)
		case *TypeParameter:
			census.add("type_parameter", 1, int64(unsafe.Sizeof(TypeParameter{})))
			stack = append(stack, data.constraint, data.target, data.resolvedDefaultType, data.resolvedBaseConstraint)
		case *TemplateLiteralType:
			census.add("template_literal", 1, int64(unsafe.Sizeof(TemplateLiteralType{})))
			typeSlice(census, "template_literal", data.texts)
			for _, text := range data.texts {
				census.text("template_literal", text)
			}
			typeSlice(census, "type_lists", data.types)
			stack = append(stack, data.types...)
			stack = append(stack, data.resolvedBaseConstraint)
		default:
			panic(fmt.Sprintf("census: unexpected payload %T", t.data))
		}
	}
	keyBytes := func(m any) int64 {
		var total int64
		iter := reflect.ValueOf(m).MapRange()
		for iter.Next() {
			key := iter.Key()
			switch key.Kind() {
			case reflect.String:
				// Shared with the literal's boxed value, charged there.
			case reflect.Struct:
				if bigint, ok := key.Interface().(jsnum.PseudoBigInt); ok {
					census.text("type_caches", bigint.Base10Value)
				}
				if unionKey, ok := key.Interface().(UnionOfUnionKey); ok {
					_ = unionKey
				}
			}
		}
		return total
	}
	for _, m := range []any{c.stringLiteralTypes, c.numberLiteralTypes, c.bigintLiteralTypes, c.unionTypes, c.unionOfUnionTypes, c.tupleTypes, c.intersectionTypes, c.templateLiteralTypes} {
		census.cache(m, keyBytes(m))
	}
	// Storage beside the type families. The core arenas and paged stores keep
	// their backing in unexported fields of another package; they are read
	// through reflection, never written.
	// The arenas' data slices hold only the current chunk; every earlier chunk
	// stays reachable through the records in it, so the census replays the
	// growth for the checker's own counters.
	census.add("symbols", int64(c.SymbolCount), int64(arenaCapacity[ast.Symbol](int(c.SymbolCount)))*int64(unsafe.Sizeof(ast.Symbol{})))
	for symbol := range symbols {
		census.text("symbols", symbol.Name)
	}
	for _, symbol := range []*ast.Symbol{c.undefinedSymbol, c.argumentsSymbol, c.requireSymbol, c.unknownSymbol, c.globalThisSymbol} {
		census.text("symbols", symbol.Name)
	}
	census.symbolTable("symbol_tables", c.globals)
	census.add("signatures", int64(c.SignatureCount), int64(arenaCapacity[Signature](int(c.SignatureCount)))*int64(unsafe.Sizeof(Signature{})))
	for signature := range signatures {
		typeSlice(census, "type_lists", signature.typeParameters)
		typeSlice(census, "signatures", signature.parameters)
	}
	for _, signature := range []*Signature{c.anySignature, c.unknownSignature, c.resolvingSignature, c.silentNeverSignature} {
		typeSlice(census, "type_lists", signature.typeParameters)
		typeSlice(census, "signatures", signature.parameters)
	}
	indexInfoData := unexportedField(&c.indexInfoArena, "data")
	census.add("index_infos", int64(indexInfoData.Len())+2, int64(arenaCapacity[IndexInfo](indexInfoData.Len()))*int64(unsafe.Sizeof(IndexInfo{}))+2*int64(unsafe.Sizeof(IndexInfo{})))
	census.add("type_predicates", 1, int64(unsafe.Sizeof(TypePredicate{})))
	census.text("type_predicates", c.noTypePredicate.parameterName)
	pageList := unexportedField(&c.valueSymbolLinks.store, "pageList")
	pageMap := unexportedField(&c.valueSymbolLinks.store, "pageMap")
	linkPages, links := 0, 0
	for i := range pageList.Len() {
		page := pageList.Index(i)
		if page.IsNil() {
			continue
		}
		linkPages++
		page = page.Elem()
		for j := range page.Len() {
			if !page.Index(j).IsNil() {
				links++
			}
		}
	}
	var zeroLink *ValueSymbolLinks
	census.add("value_symbol_links", int64(links),
		int64(pageList.Cap())*int64(unsafe.Sizeof(zeroLink))+
			int64(linkPages)*256*int64(unsafe.Sizeof(zeroLink))+ // core.pageSize
			mapBytes(pageMap.Interface())+
			int64(arenaCapacity[ValueSymbolLinks](links))*int64(unsafe.Sizeof(ValueSymbolLinks{})))
	// Go embeds a synthetic expression's type in the node; there is no side table.
	census.add("synthetic_expression_links", 0, 0)
	// The node factory's arenas are not counted; the family is named as unavailable.
	census.add("checker_ast", 0, 0)
	var typeStorage, total int64
	typeFamilies := map[string]bool{"type_records": true, "intrinsic": true, "literal": true, "unique_es_symbol": true, "anonymous": true, "reference": true, "interface": true, "tuple": true, "union": true, "intersection": true, "type_parameter": true, "template_literal": true, "alias": true, "type_lists": true, "type_caches": true}
	for name, family := range census.families {
		total += family.Bytes
		if typeFamilies[name] {
			typeStorage += family.Bytes
		}
	}
	// Go has no separate common record; the header is inside each payload struct.
	census.add("type_records", int64(len(visited)), 0)
	return map[string]any{
		"families": census.families, "type_storage_bytes": typeStorage, "checker_bytes": total,
		"types": map[string]any{"created": c.TypeCount, "reachable": len(visited), "unreachable_occupied": 0},
		"unavailable": []string{"checker_ast"},
		"symbols_reachable": len(symbols),
		"arena_chunks": map[string]any{
			"symbols":            arenaChunks[ast.Symbol](int(c.SymbolCount)),
			"signatures":         arenaChunks[Signature](int(c.SignatureCount)),
			"value_symbol_links": arenaChunks[ValueSymbolLinks](links),
		},
		"record_sizes": map[string]any{
			"Type": unsafe.Sizeof(Type{}), "IntrinsicType": unsafe.Sizeof(IntrinsicType{}), "LiteralType": unsafe.Sizeof(LiteralType{}),
			"ObjectType": unsafe.Sizeof(ObjectType{}), "TypeReference": unsafe.Sizeof(TypeReference{}), "InterfaceType": unsafe.Sizeof(InterfaceType{}),
			"TupleType": unsafe.Sizeof(TupleType{}), "UnionType": unsafe.Sizeof(UnionType{}), "TypeParameter": unsafe.Sizeof(TypeParameter{}),
			"TemplateLiteralType": unsafe.Sizeof(TemplateLiteralType{}), "TypeAlias": unsafe.Sizeof(TypeAlias{}), "Signature": unsafe.Sizeof(Signature{}),
			"IndexInfo": unsafe.Sizeof(IndexInfo{}), "Symbol": unsafe.Sizeof(ast.Symbol{}), "ValueSymbolLinks": unsafe.Sizeof(ValueSymbolLinks{}),
		},
	}
}
