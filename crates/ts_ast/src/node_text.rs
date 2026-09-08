//! Node.Text keeps stored bytes borrowed; only Go's concatenation paths allocate.
use crate::{AstView, Node, NodeData, NodeId, NodeRead, SyntaxKind as K, TextSliceRead};
use std::ops::Deref;
use ts_arena::Error;

pub struct NodeText<'a>(TextStorage<'a>);
enum TextStorage<'a> {
    Empty,
    Node(NodeRead<'a>),
    Fragment(TextSliceRead<'a>),
    Joined(Vec<u8>),
}
impl NodeText<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            TextStorage::Empty => &[],
            TextStorage::Node(node) => stored_text(node),
            TextStorage::Fragment(parts) => parts[0].as_bytes(),
            TextStorage::Joined(bytes) => bytes,
        }
    }
}
impl Deref for NodeText<'_> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

macro_rules! bytes_field {
    ($node:expr, $variant:ident, $field:ident) => {
        match $node.data() {
            NodeData::$variant(data) => data.$field.as_bytes(),
            _ => panic!(
                "interface conversion: ast.nodeData is *ast.{}, not *ast.{}",
                $node.data().name(),
                stringify!($variant)
            ),
        }
    };
}

fn stored_text(node: &Node) -> &[u8] {
    match node.kind().known() {
        Some(K::Identifier) => bytes_field!(node, Identifier, text),
        Some(K::PrivateIdentifier) => bytes_field!(node, PrivateIdentifier, text),
        Some(K::StringLiteral) => bytes_field!(node, StringLiteral, text),
        Some(K::NumericLiteral) => bytes_field!(node, NumericLiteral, text),
        Some(K::BigIntLiteral) => bytes_field!(node, BigIntLiteral, text),
        Some(K::NoSubstitutionTemplateLiteral) => {
            bytes_field!(node, NoSubstitutionTemplateLiteral, text)
        }
        Some(K::TemplateHead) => bytes_field!(node, TemplateHead, text),
        Some(K::TemplateMiddle) => bytes_field!(node, TemplateMiddle, text),
        Some(K::TemplateTail) => bytes_field!(node, TemplateTail, text),
        Some(K::RegularExpressionLiteral) => bytes_field!(node, RegularExpressionLiteral, text),
        _ => panic!("Unhandled case in Node.Text: *ast.{}", node.data().name()),
    }
}

impl Node {
    /// port: tsc/internal/ast/ast.go:Node.RawText
    pub fn raw_text(&self) -> &[u8] {
        match self.kind().known() {
            Some(K::TemplateHead) => bytes_field!(self, TemplateHead, raw_text),
            Some(K::TemplateMiddle) => bytes_field!(self, TemplateMiddle, raw_text),
            Some(K::TemplateTail) => bytes_field!(self, TemplateTail, raw_text),
            _ => panic!("Unhandled case in Node.RawText: {}", self.kind()),
        }
    }
}

impl<'a> AstView<'a> {
    /// port: tsc/internal/ast/ast.go:Node.Text
    pub fn node_text(self, id: NodeId) -> Result<NodeText<'a>, Error> {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.node_text_worker(id))
    }
    fn node_text_worker(self, mut id: NodeId) -> Result<NodeText<'a>, Error> {
        loop {
            let node = self.node(id)?;
            match node.kind().known() {
                Some(K::MetaProperty) => {
                    let NodeData::MetaProperty(data) = node.data() else {
                        panic!(
                            "interface conversion: ast.nodeData is *ast.{}, not *ast.MetaProperty",
                            node.data().name()
                        );
                    };
                    id = data
                        .name
                        .expect("runtime error: invalid memory address or nil pointer dereference");
                }
                Some(K::JsxNamespacedName) => {
                    let NodeData::JsxNamespacedName(data) = node.data() else {
                        panic!(
                            "interface conversion: ast.nodeData is *ast.{}, not *ast.JsxNamespacedName",
                            node.data().name()
                        );
                    };
                    let namespace = self.node_text(data.namespace.expect(
                        "runtime error: invalid memory address or nil pointer dereference",
                    ))?;
                    let name = self.node_text(data.name.expect(
                        "runtime error: invalid memory address or nil pointer dereference",
                    ))?;
                    let mut bytes = Vec::with_capacity(namespace.len() + 1 + name.len());
                    bytes.extend_from_slice(&namespace);
                    bytes.push(b':');
                    bytes.extend_from_slice(&name);
                    return Ok(NodeText(TextStorage::Joined(bytes)));
                }
                Some(
                    kind @ (K::JSDocText | K::JSDocLink | K::JSDocLinkCode | K::JSDocLinkPlain),
                ) => {
                    let text = match (kind, node.data()) {
                        (K::JSDocText, NodeData::JSDocText(data)) => data.text,
                        (K::JSDocLink, NodeData::JSDocLink(data)) => data.text,
                        (K::JSDocLinkCode, NodeData::JSDocLinkCode(data)) => data.text,
                        (K::JSDocLinkPlain, NodeData::JSDocLinkPlain(data)) => data.text,
                        _ => panic!(
                            "interface conversion: ast.nodeData is *ast.{}, not *ast.{kind:?}",
                            node.data().name()
                        ),
                    };
                    let parts = self.text_slice(text)?;
                    return Ok(NodeText(match parts.len() {
                        0 => TextStorage::Empty,
                        1 => TextStorage::Fragment(parts),
                        _ => {
                            let len = parts
                                .iter()
                                .try_fold(0_usize, |total, part| total.checked_add(part.len()))
                                .expect("strings: Join output length overflow");
                            if len == 0 {
                                TextStorage::Empty
                            } else {
                                let mut bytes = Vec::with_capacity(len);
                                for part in parts.iter() {
                                    bytes.extend_from_slice(part.as_bytes());
                                }
                                TextStorage::Joined(bytes)
                            }
                        }
                    }));
                }
                _ => {
                    // Validate the selected payload now, even if the returned
                    // borrowed bytes are never subsequently inspected.
                    stored_text(&node);
                    return Ok(NodeText(TextStorage::Node(node)));
                }
            }
        }
    }
}
