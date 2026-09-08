package ast

// Access-only source utility observations. Every request is defined here before
// comparing Rust, including the entire signed Kind domain and concrete graphs.
import (
    "fmt"
    "os"
    "testing"
)

func TestS06UtilitiesTail(t *testing.T) {
    out, err := os.Create(os.Getenv("S06_UTILITIES_TAIL_OUTPUT")); if err != nil { t.Fatal(err) }; defer out.Close()
    emit := func(name string, result bool) { fmt.Fprintf(out, "%s\t%t\n",name,result) }
    f:=NewNodeFactory(NodeFactoryHooks{})
    for raw:=-32768;raw<=32767;raw++ {
        kind:=Kind(raw);node:=f.NewToken(kind)
        mask:=0
        for i,v:=range []bool{IsImportOrImportEqualsDeclaration(node),HasInferredType(node),IsKeyword(kind),IsNonContextualKeyword(kind),IsExpandoPropertyDeclaration(node)} { if v { mask|=1<<i } }
        fmt.Fprintf(out,"kind/%d\t%d\n",raw,mask)
    }
    one:=f.NewIdentifier("x"); other:=f.NewIdentifier("x"); different:=f.NewIdentifier("y")
    list:=func(nodes ...*Node)*NodeList { return &NodeList{Nodes:nodes} }
    for i,nodes:=range [][]*Node{nil,{}, {one}} {
        var items *NodeList; if i!=0 {items=list(nodes...)}
        emit(fmt.Sprintf("empty/object/%d",i),IsEmptyObjectLiteral(f.NewObjectLiteralExpression(items,false)))
        emit(fmt.Sprintf("empty/array/%d",i),IsEmptyArrayLiteral(f.NewArrayLiteralExpression(items,false)))
    }
    emit("empty/other",IsEmptyObjectLiteral(one)||IsEmptyArrayLiteral(one))
    rest:=f.NewToken(KindDotDotDotToken)
    for i,node:=range []*Node{f.NewParameterDeclaration(nil,rest,one,nil,nil,nil),f.NewBindingElement(rest,nil,one,nil),f.NewSpreadElement(one),f.NewSpreadAssignment(one),one} {
        expected:=rest; if i==2||i==3 {expected=node}; if i==4 {expected=nil}
        emit(fmt.Sprintf("rest/indicator/%d",i),GetRestIndicatorOfBindingOrAssignmentElement(node)==expected)
    }
    doc:=f.NewJSDoc(nil,nil); doc.Parent=one
    link:=f.NewJSDocNameReference(other);link.Parent=doc;other.Parent=link
    emit("jsdoc/context/unflagged",IsJSDocNameReferenceContext(other))
    other.Flags|=NodeFlagsJSDoc
    emit("jsdoc/context/flagged",IsJSDocNameReferenceContext(other))
    emit("jsdoc/root",GetJSDocRoot(other)==doc)
    emit("jsdoc/host",GetJSDocHost(other)==one)
    emit("jsdoc/root/self",GetJSDocRoot(doc)==nil)
    emit("jsdoc/host/absent",GetJSDocHost(one)==nil)
    for i,kind:=range []Kind{KindPropertyAssignment,KindExportAssignment,KindPropertyDeclaration,KindVariableDeclaration,KindSatisfiesExpression,KindReturnStatement,KindVariableStatement,KindExpressionStatement,KindIdentifier} {
        parent:=f.NewToken(kind);different.Parent=parent
        emit(fmt.Sprintf("jsdoc/next/%d",i),GetNextJSDocCommentLocation(different)==parent)
    }
    decls:=f.NewVariableDeclarationList(list(one,different),0);one.Parent=decls;different.Parent=decls
    emit("jsdoc/next/first",GetNextJSDocCommentLocation(one)==decls)
    emit("jsdoc/next/second",GetNextJSDocCommentLocation(different)==nil)
    num:=f.NewNumericLiteral("1",0);big:=f.NewBigIntLiteral("1n",0);truth:=f.NewToken(KindTrueKeyword)
    for i,operator:=range []Kind{KindPlusToken,KindMinusToken,KindExclamationToken} {
        for j,operand:=range []*Node{num,big,truth} {
            node:=f.NewPrefixUnaryExpression(operator,operand)
            for _,include:=range []bool{false,true} { emit(fmt.Sprintf("primitive/%d/%d/%t",i,j,include),IsPrimitiveLiteralValue(node,include)) }
        }
    }
    for _,kind:=range []Kind{KindTrueKeyword,KindFalseKeyword,KindNumericLiteral,KindStringLiteral,KindNoSubstitutionTemplateLiteral,KindBigIntLiteral,KindNullKeyword} {
        for _,include:=range []bool{false,true} { emit(fmt.Sprintf("primitive/kind/%d/%t",kind,include),IsPrimitiveLiteralValue(f.NewToken(kind),include)) }
    }
    for i,name:=range []string{"Infinity","-Infinity","NaN","+Infinity","nan","Infinity\x00","\xff"} { emit(fmt.Sprintf("number-name/%d",i),IsInfinityOrNaNString(name)) }
    for i,node:=range []*Node{nil,f.NewArrayTypeNode(one),f.NewTypeReferenceNode(one,nil),f.NewTypeReferenceNode(one,list()),f.NewTypeReferenceNode(one,list(other,different)),one} {
        expected:=(*Node)(nil);if i==1 {expected=one};if i==4 {expected=other}
        emit(fmt.Sprintf("rest/type/%d",i),GetRestParameterElementType(node)==expected)
    }
    this:=f.NewToken(KindThisKeyword)
    for i,pair:=range [][2]*Node{{one,other},{one,different},{one,truth},{this,this},{f.NewJsxNamespacedName(one,other),f.NewJsxNamespacedName(other,one)},{f.NewJsxNamespacedName(one,other),f.NewJsxNamespacedName(other,different)},{f.NewPropertyAccessExpression(one,nil,other,0),f.NewPropertyAccessExpression(other,nil,one,0)},{f.NewPropertyAccessExpression(one,nil,other,0),f.NewPropertyAccessExpression(different,nil,one,0)}} { emit(fmt.Sprintf("tag/equal/%d",i),TagNamesAreEquivalent(pair[0],pair[1])) }
    func(){defer func(){value:=recover();emit("tag/unhandled",value=="Unhandled case in TagNamesAreEquivalent")}();TagNamesAreEquivalent(num,num)}()
    tag:=f.NewJSDocUnknownTag(one,nil);one.Parent=tag;other.Parent=tag
    emit("tag/name/yes",IsTagName(one));emit("tag/name/no",IsTagName(other));emit("tag/name/root",IsTagName(tag))
    access:=f.NewElementAccessExpression(truth,nil,one,0);one.Parent=access;other.Parent=access
    emit("argument/nil",isArgumentOfElementAccessExpression(nil));emit("argument/yes",isArgumentOfElementAccessExpression(one));emit("argument/no",isArgumentOfElementAccessExpression(other));emit("argument/root",isArgumentOfElementAccessExpression(access))
    emit("expando/nil",IsExpandoPropertyDeclaration(nil))
    super:=f.NewToken(KindSuperKeyword)
    for i,node:=range []*Node{f.NewPropertyAccessExpression(super,nil,one,0),f.NewElementAccessExpression(super,nil,one,0),f.NewPropertyAccessExpression(truth,nil,one,0),one} {emit(fmt.Sprintf("super/%d",i),IsSuperProperty(node))}
    proto:=f.NewIdentifier("__proto__");protoString:=f.NewStringLiteral("__proto__",0)
    for i,node:=range []*Node{proto,protoString,f.NewNumericLiteral("__proto__",0),f.NewNoSubstitutionTemplateLiteral("__proto__",0),f.NewComputedPropertyName(proto),one} {emit(fmt.Sprintf("proto/%d",i),IsProtoSetter(node))}
    for i,node:=range []*Node{f.NewLiteralTypeNode(protoString),f.NewLiteralTypeNode(f.NewNoSubstitutionTemplateLiteral("x",0)),f.NewLiteralTypeNode(num),one} {emit(fmt.Sprintf("literal-type/%d",i),IsStringLiteralLikeType(node))}
    for i,node:=range []*Node{f.NewPropertyAssignment(nil,proto,nil,nil,one),f.NewPropertyAssignment(nil,one,nil,nil,nil),f.NewShorthandPropertyAssignment(nil,one,nil,nil,nil,nil),f.NewShorthandPropertyAssignment(nil,one,nil,nil,nil,truth),f.NewVariableDeclaration(one,nil,nil,truth),f.NewVariableDeclaration(protoString,nil,nil,truth),f.NewParameterDeclaration(nil,nil,one,nil,nil,truth),f.NewParameterDeclaration(nil,rest,one,nil,nil,truth),f.NewBindingElement(nil,nil,one,truth),f.NewBindingElement(rest,nil,one,truth),f.NewPropertyDeclaration(nil,one,nil,nil,truth),f.NewPropertyDeclaration(nil,one,nil,nil,nil),f.NewBinaryExpression(nil,one,nil,f.NewToken(KindEqualsToken),truth),f.NewBinaryExpression(nil,one,nil,f.NewToken(KindPlusToken),truth),f.NewToken(KindExportAssignment),one} {emit(fmt.Sprintf("named/%d",i),IsNamedEvaluationSource(node))}
}
