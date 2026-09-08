package ast

// Pinned source calls over all signed Kind values, plus explicit graph requests.
import (
 "fmt"
 "os"
 "testing"
 "github.com/microsoft/TypeScript/tsc/internal/core"
)

func TestS06UtilitiesFront(t *testing.T) {
 out,err:=os.Create(os.Getenv("S06_UTILITIES_FRONT_OUTPUT"));if err!=nil {t.Fatal(err)};defer out.Close()
 emit:=func(name string,value any){fmt.Fprintf(out,"%s\t%v\n",name,value)}
 f:=NewNodeFactory(NodeFactoryHooks{})
 for raw:=-32768;raw<=32767;raw++ {
  kind:=Kind(raw);node:=f.NewToken(kind);mask:=uint64(0)
  values:=[]bool{
   IsObjectBindingOrAssignmentElement(node),
   IsPropertyNameLiteral(node),
   IsPropertyName(node),
   IsClassElement(node),
   IsMethodOrAccessor(node),
   IsTypeElement(node),
   IsObjectLiteralElement(node),
   IsJsxChild(node),
   CanHaveSymbol(node),
   CanHaveIllegalModifiers(node),
   CanHaveModifiers(node),
   isUnaryExpressionKind(kind),
   isExpressionKind(kind),
   isDeclarationStatementKind(kind),
   isStatementKindButNotDeclarationKind(kind),
   IsBindingPattern(node),
   IsAccessor(node),
   IsMemberName(node),
   IsEntityName(node),
   IsBooleanLiteral(node),
   IsStringLiteralLike(node),
   IsStringOrNumericLiteralLike(node),
   IsAssertionExpression(node),
   IsAccessExpression(node),
   IsClassLike(node),
   IsClassOrInterfaceLike(node),
   IsJsxAttributeLike(node),
   IsFunctionExpressionOrArrowFunction(node),
   IsModuleOrEnumDeclaration(node),
   IsImportOrExportSpecifier(node),
   NodeIsSynthesized(node),
   IsModifier(node),
   IsModifierLike(node),
   IsLiteralExpression(node),
   IsDeclarationStatement(node),
   IsStatementButNotDeclaration(node),
   IsTypeNode(node),
   IsForInOrOfStatement(node),
   IsFunctionLikeDeclaration(node),
   IsFunctionLike(node),
   IsFunctionLikeOrClassStaticBlockDeclaration(node),
   IsFunctionOrSourceFile(node),
   IsInJSFile(node),
   IsInJsonFile(node),
   IsCompoundAssignment(kind),
   IsLogicalBinaryOperator(kind),
   IsLogicalOrCoalescingBinaryOperator(kind),
   IsJSDocKind(kind),
   IsAnyImportSyntax(node),
   IsImportNode(node),
   IsAnyImportOrReExport(node),
   isFunctionLikeDeclarationKind(kind),
   IsFunctionLikeKind(kind),
   IsTypeNodeKind(kind),
   IsOptionalChain(node),
   IsOptionalChainRoot(node),
  }
  for i,value:=range values {if value {mask|=uint64(1)<<i}}
  emit(fmt.Sprintf("kind/%d",raw),fmt.Sprintf("%x",mask))
 }
 emit("nil/for",IsForInOrOfStatement(nil));emit("nil/function",IsFunctionLike(nil));emit("nil/function-decl",IsFunctionLikeDeclaration(nil));emit("nil/function-static",IsFunctionLikeOrClassStaticBlockDeclaration(nil));emit("nil/js",IsInJSFile(nil));emit("nil/block",IsFunctionBlock(nil));emit("nil/object-method",IsObjectLiteralMethod(nil));emit("nil/descendant",IsNodeDescendantOf(nil,nil))
 for i,pos:=range []int{-1<<63,-1,-0,1,1<<31-1,1<<31,1<<63-1} { emit(fmt.Sprintf("position/%d",i),PositionIsSynthesized(pos)) }
 for i,pair:=range [][2]int{{-1,-1},{0,0},{0,1},{1,0},{-1,1},{1,-1},{1<<31,1<<31},{1<<32,1<<32}} {
  node:=f.NewToken(KindIdentifier);node.Loc=core.NewTextRange(pair[0],pair[1]);emit(fmt.Sprintf("range/%d",i),RangeIsSynthesized(node.Loc));emit(fmt.Sprintf("node-range/%d",i),NodeIsSynthesized(node))
 }
 flags:=[]NodeFlags{0,^NodeFlags(0)};for bit:=0;bit<32;bit++ {flags=append(flags,NodeFlags(1)<<bit)}
 for i,flag:=range flags { node:=f.NewToken(KindIdentifier);node.Flags=flag;emit(fmt.Sprintf("flags/js/%d",i),IsInJSFile(node));emit(fmt.Sprintf("flags/json/%d",i),IsInJsonFile(node)) }
 a:=f.NewToken(KindIdentifier);b:=f.NewToken(KindClassExpression);c:=f.NewSourceFile(SourceFileParseOptions{FileName:"/s06/front.ts"},"",nil,nil);a.Parent=b;b.Parent=c
 emit("ancestor/self",FindAncestor(a,func(n *Node)bool{return n==a})==a)
 emit("ancestor/class",GetContainingClass(a)==b);emit("ancestor/class-self",GetContainingClass(b)==nil)
 emit("ancestor/source",GetSourceFileOfNode(a)==c.AsSourceFile())
 emit("ancestor/kind",FindAncestorKind(a,KindClassExpression)==b)
 emit("ancestor/nil",FindAncestor(nil,func(n *Node)bool{panic("callback called")})==nil)
 many:=FindManyAncestors(a,func(n *Node)bool{return true},func(n *Node)bool{return true},func(n *Node)bool{return true});emit("ancestor/many-order",many[0]==a&&many[1]==b&&many[2]==c)
 emit("ancestor/many-empty",len(FindManyAncestors(a))==0)
 for _,value:=range []int32{-1,0,1,2,3} {emit(fmt.Sprintf("ancestor/result/%d",value),FindAncestorOrQuit(a,func(n *Node)FindAncestorResult{if n==a{return FindAncestorResult(value)};return FindAncestorTrue})==a);emit(fmt.Sprintf("ancestor/result-parent/%d",value),FindAncestorOrQuit(a,func(n *Node)FindAncestorResult{if n==a{return FindAncestorResult(value)};return FindAncestorTrue})==b)}
 emit("ancestor/from-bool/false",int32(ToFindAncestorResult(false)));emit("ancestor/from-bool/true",int32(ToFindAncestorResult(true)))
 for i,pair:=range [][2]*Node{{a,a},{a,b},{a,c},{b,a},{a,nil},{nil,a}} {emit(fmt.Sprintf("descendant/%d",i),IsNodeDescendantOf(pair[0],pair[1]))}
 paren:=f.NewToken(KindParenthesizedExpression);paren2:=f.NewToken(KindParenthesizedExpression);paren.Parent=paren2;paren2.Parent=a
 emit("walk/expression",WalkUpParenthesizedExpressions(paren)==a);emit("walk/expression-other",WalkUpParenthesizedExpressions(a)==a);emit("walk/expression-nil",WalkUpParenthesizedExpressions(nil)==nil)
 typ:=f.NewToken(KindParenthesizedType);typ.Parent=a;emit("walk/type",WalkUpParenthesizedTypes(typ)==a);emit("walk/type-nil",WalkUpParenthesizedTypes(nil)==nil)
 hidden:=f.NewToken(KindIdentifier);hidden.Flags=NodeFlagsReparsed
 emit("visible/empty",FindLastVisibleNode(nil)==nil);emit("visible/skip",FindLastVisibleNode([]*Node{a,hidden,hidden})==a);emit("visible/all-hidden",FindLastVisibleNode([]*Node{hidden})==nil)
 for i,kind:=range []Kind{KindFunctionDeclaration,KindTryStatement,KindCatchClause,KindIdentifier} {block:=f.NewToken(KindBlock);block.Parent=f.NewToken(kind);emit(fmt.Sprintf("block/function/%d",i),IsFunctionBlock(block));emit(fmt.Sprintf("block/statement/%d",i),IsStatement(block));emit(fmt.Sprintf("block/module/%d",i),IsFunctionOrModuleBlock(block))}
 for flag:=NodeFlags(0);flag<8;flag++ {
  variable:=f.NewToken(KindVariableDeclaration);decls:=f.NewToken(KindVariableDeclarationList);stmt:=f.NewToken(KindVariableStatement);variable.Parent=decls;decls.Parent=stmt;variable.Flags=NodeFlagsJavaScriptFile;decls.Flags=flag;stmt.Flags=NodeFlagsAmbient
  binding:=f.NewToken(KindBindingElement);pattern:=f.NewToken(KindObjectBindingPattern);binding.Parent=pattern;pattern.Parent=variable
  emit(fmt.Sprintf("combined/flags/%d",flag),uint32(GetCombinedNodeFlags(binding)))
  emit(fmt.Sprintf("combined/root/%d",flag),GetRootDeclaration(binding)==variable)
  emit(fmt.Sprintf("combined/walk/%d",flag),WalkUpBindingElementsAndPatterns(binding)==variable)
  emit(fmt.Sprintf("combined/await/%d",flag),IsVarAwaitUsing(binding));emit(fmt.Sprintf("combined/using/%d",flag),IsVarUsing(binding));emit(fmt.Sprintf("combined/const/%d",flag),IsVarConst(binding));emit(fmt.Sprintf("combined/let/%d",flag),IsVarLet(binding));emit(fmt.Sprintf("combined/const-like/%d",flag),IsVarConstLike(binding))
 }
 parameter:=f.NewToken(KindParameter);pattern:=f.NewToken(KindArrayBindingPattern);binding:=f.NewToken(KindBindingElement);binding.Parent=pattern;pattern.Parent=parameter;emit("binding/parameter",IsPartOfParameterDeclaration(binding))
 variable:=f.NewToken(KindVariableDeclaration);variable.Parent=f.NewToken(KindCatchClause);emit("binding/catch",IsCatchClauseVariableDeclarationOrBindingElement(variable));emit("binding/scoped",IsBlockOrCatchScoped(variable))
 panicClass:=func(name string,action func(),want string){defer func(){value:=recover();text:=fmt.Sprint(value);if text!=want{t.Fatalf("%s wrong panic %q",name,text)};emit(name,true)}();action();t.Fatalf("%s did not panic",name)}
 panicClass("panic/visible-nil",func(){FindLastVisibleNode([]*Node{nil})},"runtime error: invalid memory address or nil pointer dereference")
 panicClass("panic/binding-parent",func(){GetRootDeclaration(f.NewToken(KindBindingElement))},"runtime error: invalid memory address or nil pointer dereference")
 panicClass("panic/source-payload",func(){GetSourceFileOfNode(f.NewToken(KindSourceFile))},"interface conversion: ast.nodeData is *ast.Token, not *ast.SourceFile")
 panicClass("panic/module-payload",func(){IsGlobalScopeAugmentation(f.NewToken(KindModuleDeclaration))},"interface conversion: ast.nodeData is *ast.Token, not *ast.ModuleDeclaration")
 // New modifier, optional-chain and logical-expression families.
 name:=f.NewIdentifier("x");question:=f.NewToken(KindQuestionDotToken)
 for i,flag:=range []NodeFlags{0,NodeFlagsOptionalChain,^NodeFlags(0)} {
  for j,q:=range []*Node{nil,question} {
   node:=f.NewPropertyAccessExpression(a,q,name,0);node.Flags=flag;node.Parent=b
   emit(fmt.Sprintf("optional/chain/%d/%d",i,j),IsOptionalChain(node));emit(fmt.Sprintf("optional/root/%d/%d",i,j),IsOptionalChainRoot(node));emit(fmt.Sprintf("optional/outer/%d/%d",i,j),IsOutermostOptionalChain(node));a.Parent=node
   emit(fmt.Sprintf("optional/expression/%d/%d",i,j),IsExpressionOfOptionalChainRoot(a));a.Parent=b
  }
 }
 for i,kind:=range []Kind{KindBarBarToken,KindAmpersandAmpersandToken,KindQuestionQuestionToken,KindCommaToken,KindInstanceOfKeyword,KindEqualsToken,KindPlusEqualsToken,KindBarBarEqualsToken,KindAmpersandAmpersandEqualsToken,KindQuestionQuestionEqualsToken} {
  node:=f.NewBinaryExpression(nil,a,nil,f.NewToken(kind),name)
  emit(fmt.Sprintf("binary/logical/%d",i),IsLogicalOrCoalescingBinaryExpression(node));emit(fmt.Sprintf("binary/assignment/%d",i),IsLogicalOrCoalescingAssignmentExpression(node));emit(fmt.Sprintf("binary/coalesce/%d",i),IsNullishCoalesce(node));emit(fmt.Sprintf("binary/comma/%d",i),IsCommaExpression(node));emit(fmt.Sprintf("binary/sequence/%d",i),IsCommaSequence(node));emit(fmt.Sprintf("binary/instanceof/%d",i),IsInstanceOfExpression(node))
  wrapper:=f.NewParenthesizedExpression(f.NewPrefixUnaryExpression(KindExclamationToken,node));emit(fmt.Sprintf("binary/wrapped/%d",i),IsLogicalExpression(wrapper))
 }
 for i,operator:=range []Kind{KindPlusToken,KindMinusToken,KindExclamationToken} {for j,operand:=range []*Node{f.NewNumericLiteral("1",0),f.NewBigIntLiteral("1n",0),nil} {if j==2&&i!=2 {continue};emit(fmt.Sprintf("signed/%d/%d",i,j),IsSignedNumericLiteral(f.NewPrefixUnaryExpression(operator,operand)))}}
 for i,flag:=range []ModifierFlags{0,ModifierFlagsStatic,ModifierFlagsAccessor,ModifierFlagsPrivate,ModifierFlagsConst,^ModifierFlags(0)} {
  mods:=&ModifierList{};mods.ModifierFlags=flag
  property:=f.NewPropertyDeclaration(mods,name,nil,nil,nil)
  emit(fmt.Sprintf("modifier/static/%d",i),IsStatic(property));emit(fmt.Sprintf("modifier/accessor/%d",i),IsAutoAccessorPropertyDeclaration(property));emit(fmt.Sprintf("modifier/has/%d",i),HasSyntacticModifier(property,ModifierFlagsStatic|ModifierFlagsAccessor))
  parameter:=f.NewParameterDeclaration(mods,nil,name,nil,nil,nil);emit(fmt.Sprintf("modifier/parameter/%d",i),IsParameterPropertyDeclaration(parameter,f.NewToken(KindConstructor)))
  stmt:=f.NewVariableStatement(mods,nil);decl:=f.NewVariableDeclaration(name,nil,nil,nil);decl.Parent=stmt
  emit(fmt.Sprintf("modifier/combined/%d",i),uint32(GetCombinedModifierFlags(decl)));emit(fmt.Sprintf("modifier/const/%d",i),IsEnumConst(stmt))
 }
 for i,name:=range []*Node{f.NewIdentifier("x"),f.NewPrivateIdentifier("#x")} {emit(fmt.Sprintf("private/%d",i),IsPrivateIdentifierClassElementDeclaration(f.NewPropertyDeclaration(nil,name,nil,nil,nil)))}
 for i,expression:=range []*Node{f.NewStringLiteral("use strict",0),f.NewNoSubstitutionTemplateLiteral("use strict",0),a} {emit(fmt.Sprintf("prologue/%d",i),IsPrologueDirective(f.NewExpressionStatement(expression)))}
}
