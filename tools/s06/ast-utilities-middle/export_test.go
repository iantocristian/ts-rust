package ast

import (
    "bytes"
    "encoding/binary"
    "fmt"
    "os"
    "path/filepath"
    "testing"
    "github.com/microsoft/TypeScript/tsc/internal/core"
)

func TestS06MiddleUtilities(t *testing.T) {
    directory := os.Getenv("S06_MIDDLE_OUTPUT")
    if directory == "" { t.Fatal("missing output directory") }
    var kinds, behaviors bytes.Buffer
    for raw := -32768; raw <= 32767; raw++ {
        node := &Node{Kind: Kind(raw)}
        observations := []bool{
            IsJSDocLinkLike(node), IsJSDocTag(node), IsQuestionToken(node), IsJSDocNode(node),
            IsPropertyAccessOrQualifiedName(node), IsBreakOrContinueStatement(node), IsParameterLike(node),
            NodeHasKind(node,KindIdentifier), IsContextualKeyword(node.Kind), IsParameterPropertyModifier(node.Kind),
            HasTypeArguments(node), IsTypeReferenceType(node), IsVariableLike(node), IsVariableParameterOrProperty(node),
            IsObjectTypeDeclaration(node), IsTypeKeywordToken(node), IsResolutionModeOverrideHost(node),
            IsStringTextContainingNode(node), IsTemplateLiteralKind(node.Kind), IsTemplateLiteralToken(node),
            IsLateVisibilityPaintedStatement(node), IsJsxOpeningLikeElement(node), IsCallOrNewExpression(node),
            IsTrivia(node.Kind), hasComment(node.Kind), IsDeclarationBindingElement(node), IsJsxCallLike(node),
        }
        var mask uint32
        for index,value := range observations { if value { mask |= 1 << index } }
        if err := binary.Write(&kinds,binary.LittleEndian,mask); err != nil { t.Fatal(err) }
    }
    emit := func(name string, values ...int64) {
        fmt.Fprint(&behaviors,name)
        for _,value := range values { fmt.Fprintf(&behaviors,"\t%d",value) }
        fmt.Fprintln(&behaviors)
    }
    flag := func(value bool) int64 { if value { return 1 }; return 0 }
    f := NewNodeFactory(NodeFactoryHooks{})
    emit("nullable",flag(IsQuestionToken(nil)),flag(NodeHasKind(nil,KindUnknown)),flag(IsResolutionModeOverrideHost(nil)),flag(IsPlainJSFile(nil,core.TSUnknown)))
    for index,flags := range []ModifierFlags{0,ModifierFlags(0xffffffff),ModifierFlagsPublic|ModifierFlagsAbstract|ModifierFlagsReadonly|ModifierFlagsAsync} {
        var order []int64
        CreateModifiersFromModifierFlags(flags,func(kind Kind)*Node { order=append(order,int64(kind));return f.NewToken(kind) })
        emit(fmt.Sprintf("modifiers/%d",index),order...)
    }
    nilModifiers := CreateModifiersFromModifierFlags(ModifierFlags(0xffffffff),func(Kind)*Node{return nil})
    allNil := true
    for _,node := range nilModifiers {allNil = allNil && node == nil}
    emit("modifiers/nil",int64(len(nilModifiers)),flag(allNil))
    unexpectedModifier := func(Kind)*Node { panic("unexpected modifier callback") }
    emit("modifiers/empty",flag(CreateModifiersFromModifierFlags(0,unexpectedModifier)==nil),flag(CreateModifiersFromModifierFlags(ModifierFlagsDeprecated,unexpectedModifier)==nil))
    for index,flags := range []TokenFlags{0,TokenFlagsUnterminated,TokenFlags(-1)} {
        numeric := f.NewNumericLiteral("1",flags)
        template := f.NewNoSubstitutionTemplateLiteral("x",flags)
        head := f.NewTemplateHead("x","x",flags)
        emit(fmt.Sprintf("literal/%d",index),flag(IsUnterminatedLiteral(numeric)),flag(IsUnterminatedLiteral(template)),flag(IsUnterminatedLiteral(head)))
    }
    ordinary := f.NewIdentifier("ordinary")
    space := f.NewJsxText(" ",true)
    word := f.NewJsxText("x",false)
    empty := f.NewJsxExpression(nil,nil)
    filled := f.NewJsxExpression(nil,ordinary)
    children := []*Node{ordinary,space,word,empty,filled}
    filtered := GetSemanticJsxChildren(children)
    var retained []int64
    for _,node := range filtered { for index,original := range children { if node==original { retained=append(retained,int64(index));break } } }
    emit("jsx/filter",retained...)
    kept := []*Node{ordinary,word,filled}
    same := GetSemanticJsxChildren(kept)
    emit("jsx/identity",flag(&same[0]==&kept[0]),flag(IsWhitespaceOnlyJsxText(space)),flag(IsWhitespaceOnlyJsxText(word)),flag(IsNonWhitespaceToken(space)),flag(IsNonWhitespaceToken(word)))
    discarded := GetSemanticJsxChildren([]*Node{space})
    emit("jsx/empty",flag(GetSemanticJsxChildren(nil)==nil),flag(GetSemanticJsxChildren(make([]*Node,0))==nil),flag(discarded==nil),int64(len(discarded)))
    label := f.NewIdentifier("label")
    statement := f.NewLabeledStatement(label,f.NewEmptyStatement()); label.Parent=statement
    emit("label/statement",flag(IsLabelName(label)),flag(IsLabelOfLabeledStatement(label)),flag(IsJumpStatementTarget(label)))
    jump := f.NewBreakStatement(label);label.Parent=jump
    emit("label/jump",flag(IsLabelName(label)),flag(IsLabelOfLabeledStatement(label)),flag(IsJumpStatementTarget(label)))
    base,name := f.NewIdentifier("base"),f.NewIdentifier("name")
    access := f.NewPropertyAccessExpression(base,nil,name,0);name.Parent=access; base.Parent=access
    emit("access/property",flag(IsRightSideOfPropertyAccess(name)),flag(IsRightSideOfPropertyAccess(base)),flag(IsRightSideOfQualifiedNameOrPropertyAccess(name)),flag(ClimbPastPropertyAccess(name)==access),flag(GetFirstIdentifier(access)==base),flag(GetLeftmostAccessExpression(access)==base))
    element := f.NewElementAccessExpression(base,nil,name,0);name.Parent=element
    emit("access/element",flag(IsArgumentExpressionOfElementAccess(name)),flag(climbPastPropertyOrElementAccess(name)==element))
    qualified := f.NewQualifiedName(base,name);name.Parent=qualified
    emit("access/qualified",flag(IsRightSideOfQualifiedNameOrPropertyAccess(name)),flag(GetFirstIdentifier(qualified)==base))
    require := f.NewIdentifier("require");literal := f.NewStringLiteral("package",0)
    arguments := f.NewNodeList([]*Node{literal})
    call := f.NewCallExpression(require,nil,nil,arguments,0)
    emit("call/require",flag(IsRequireCall(call,true)),flag(IsRequireCall(call,false)),flag(IsSuperCall(call)),flag(IsCallLikeExpression(call)),flag(IsCallLikeOrFunctionLikeExpression(call)),flag(GetInvokedExpression(call)==require),flag(selectExpressionOfCallOrNewExpressionOrDecorator(call)==require))
    arguments.Nodes[0]=ordinary
    emit("call/nonliteral",flag(IsRequireCall(call,true)),flag(IsRequireCall(call,false)))
    call.AsCallExpression().Expression=f.NewToken(KindSuperKeyword)
    emit("call/super",flag(IsSuperCall(call)),flag(IsRequireCall(call,false)))
    constName := f.NewIdentifier("const");typ := f.NewTypeReferenceNode(constName,nil)
    assertion := f.NewAsExpression(ordinary,typ)
    emit("const/assertion",flag(IsConstTypeReference(typ)),flag(IsConstAssertion(assertion)),flag(IsConstAssertion(ordinary)))
    typ.AsTypeReferenceNode().TypeArguments=f.NewNodeList([]*Node{ordinary})
    emit("const/arguments",flag(IsConstTypeReference(typ)),flag(IsConstAssertion(assertion)))
    comment := f.NewJSDocText([]string{"comment"});list := f.NewNodeList([]*Node{comment});doc := f.NewJSDoc(list,nil);comment.Parent=doc
    alternate := f.NewNodeList(list.Nodes)
    emit("jsdoc/identity",flag(IsJSDocSingleCommentNode(doc)),flag(IsJSDocSingleCommentNodeList(list)),flag(IsJSDocSingleCommentNodeList(alternate)),flag(IsJSDocSingleCommentNodeComment(comment)),flag(IsJSDocSingleCommentNodeList(nil)),flag(IsJSDocSingleCommentNodeComment(nil)))
    list.Nodes=append(list.Nodes,f.NewJSDocText([]string{"second"}))
    emit("jsdoc/multiple",flag(IsJSDocSingleCommentNode(doc)),flag(IsJSDocSingleCommentNodeComment(comment)))
    left,right := f.NewIdentifier("a"),f.NewIdentifier("b")
    left.Loc=core.NewTextRange(-2147483648,2147483647);right.Loc=core.NewTextRange(2147483647,-2147483648)
    emit("positions/extreme",int64(CompareNodePositions(left,right)),int64(CompareNodePositions(right,left)))
    left.Loc=core.NewTextRange(1,2);right.Loc=core.NewTextRange(1,4);other:=f.NewIdentifier("c");other.Loc=left.Loc
    emit("positions/search",int64(IndexOfNode([]*Node{left,other,right},other)),int64(IndexOfNode([]*Node{left,other,right},right)),int64(IndexOfNode(nil,left)),int64(IndexOfNode(nil,nil)))
    for name,data := range map[string][]byte{"ast-utilities-middle-kinds.bin":kinds.Bytes(),"ast-utilities-middle-behaviors.tsv":behaviors.Bytes()} {
        if err:=os.WriteFile(filepath.Join(directory,name),data,0600);err!=nil {t.Fatal(err)}
    }
}
