"""Construction inputs frozen before codec comparison; no expected Rust outcomes."""
LITERAL_KINDS = (
    'StringLiteral', 'NumericLiteral', 'BigIntLiteral', 'RegularExpressionLiteral',
    'NoSubstitutionTemplateLiteral', 'TemplateHead', 'TemplateMiddle', 'TemplateTail',
    'Identifier', 'PrivateIdentifier', 'JsxText', 'JSDocText', 'JSDocLink', 'JSDocLinkPlain', 'JSDocLinkCode',
)
CODEC_SCENARIOS = tuple('literal/'+kind for kind in LITERAL_KINDS) + (
    'synthetic-expression', 'raw-kind', 'source-strings', 'source-metadata', 'source-empty-span', 'msgpack-boundaries',
)
CODEC_STAGES = ['encode', 'decode', 'decoded_tree', 'reencode']


def document(pin):
    cases=[]
    for scenario in CODEC_SCENARIOS:
        if scenario.startswith('literal/'):
            steps=[
                'New factory; construct '+scenario[8:]+' with text hex 61f09f9880eda080ff, rawText hex 7261775c6e5c7544383030 where applicable, flags=-1 where applicable.',
                'JSDoc text is the three raw string pieces 61, empty, f09f9880eda080ff. JSDocLink kinds have Identifier("name") child. JsxText containsOnlyTriviaWhiteSpaces=true.',
                'Set root loc=(-1,2), flags=2779096485. EncodeNode(root,nil). DecodeNodes returned bytes; walk public ForEachChild; EncodeNode(decoded,nil).',
            ]
        elif scenario=='synthetic-expression':
            steps=['NewToken(KindSyntheticExpression), then EncodeNode(root,nil); kind dispatch reaches the explicit codec panic before payload inspection. No checker-owned constructor is credited.']
        elif scenario=='raw-kind':
            steps=['NewToken(Kind(-1)), loc=(-1,2147483647), flags=4294967295; EncodeNode(root,nil), DecodeNodes, child walk and reencode if returned.']
        else:
            steps=[
                'New factory; SourceFile text hex 27f09f988027206120610a, filename=/s06/codec.ts, path=/s06/codec.ts; jsx=true, force=true. Statements: StringLiteral(emoji,flags=1024) loc=(0,6); Identifier(a) loc=(7,8); distinct Identifier(a) loc=(9,10); EOF loc=(11,11). List loc=(0,10); source loc=(0,11).',
                'SourceFile node/text counts use factory counters, scriptKind=3, languageVariant=1, hash hi=0123456789abcdef/lo=fedcba9876543210. EncodeSourceFile; DecodeNodes; public child walk; EncodeSourceFile(decoded).',
            ]
            if scenario in ('source-metadata','source-empty-span'):
                steps += [
                    'ModuleAugmentations=[first identifier, separately allocated missing Identifier, nil]; imports=[string literal, separately allocated missing StringLiteral, nil]; ExternalModuleIndicator=source itself. Nil metadata entries intentionally encode zero indices. Ambient names=[empty,repeat,repeat,raw ff].',
                    'ReferencedFiles=[loc(1,5),fileName raw ff,resolutionMode=-1,preserve=true]; TypeReferenceDirectives=[loc(-1,128),fileName=types,resolutionMode=99,preserve=false]; LibReferenceDirectives=[loc(0,11),fileName=lib,resolutionMode=0,preserve=true].',
                    'ContentMapper=mapper, virtualFileName=/s06/codec.virtual.ts, originalText hex 58f09f9880590a. Supplemental files /s06/one.ts and /s06/two.ts; canonical=/s06/canonical.ts; all are separately allocated empty SourceFiles.',
                    'Diagnostic directives: original(1,5),virtual(1,5),policy=1,unusedCode=-1; original(-1,128),virtual(-1,256),policy=255,unusedCode=65536. Unused text/source fields are populated but omitted by wire codec.',
                    ('SpanMap allocated empty, distinct from nil.' if scenario=='source-empty-span' else
                     'SpanMap segments supplied reversed then Go New sorts: virtual(7,10),original(5,6),kind=2,features=0; virtual(1,5),original(1,5),kind=0,features=1048575. Rust freezes the resulting sorted records, without claiming spanmap algorithms.'),
                ]
            elif scenario=='msgpack-boundaries':
                steps += [
                    'AmbientModuleNames array length=65536, empty strings except first eight: ASCII x repeated lengths 0,31,32,255,256,65535,65536,1. This crosses fixed/str8/str16/str32 and array16/array32 boundaries.',
                    'ReferencedFiles array length=16: positions use [0,127,128,255,256,65535,65536,-1] cyclically, end=pos; resolutionMode same, preserve alternates. Filename lengths reuse first eight ambient names. TypeReferenceDirectives uses first15 references, LibReferenceDirectives is empty.',
                ]
        cases.append({'request':{'version':1,'id':'codec/'+scenario,'primary':None,'op':'codec','scenario':scenario},'steps':steps,'stages':CODEC_STAGES})
    return {'version':1,'pin':pin,'cases':cases}
