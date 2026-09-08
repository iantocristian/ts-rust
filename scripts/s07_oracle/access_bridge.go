// Access-only methods over pinned private identity and synthetic payload fields.
package ast

func S07RawNodeID(node *Node) uint64 {
	if node == nil {
		return 0
	}
	return node.id.Load()
}
func S07RawSymbolID(symbol *Symbol) uint64 {
	if symbol == nil {
		return 0
	}
	return symbol.id.Load()
}
func S07FlowPayload(node *Node) any {
	if node == nil {
		return nil
	}
	switch data := node.data.(type) {
	case *FlowSwitchClauseData:
		return data
	case *FlowReduceLabelData:
		return data
	default:
		return node
	}
}
