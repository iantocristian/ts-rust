//! Pinned binary-layout metadata for the source-file encoder.
//!
//! These tables describe the protocol and its generated field layouts. They do
//! not implement serialization or the inventoried custom codecs; those are S06.

use ts_ast::SyntaxKind;

mod generated;

pub use generated::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DataType {
    Children = NODE_DATA_TYPE_CHILDREN,
    String = NODE_DATA_TYPE_STRING,
    Extended = NODE_DATA_TYPE_EXTENDED_DATA,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildType {
    Node,
    NodeList,
    RawNodeList,
    ModifierList,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChildProperty {
    pub name: &'static str,
    pub child_type: ChildType,
    pub optional: bool,
}

/// An automatically packed field, relative to the start of common data (bit 24).
/// Empty `kind_values` means a boolean. For an optional kind union, encoded zero
/// means absent and the first listed kind has encoded value one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommonDataField {
    pub name: &'static str,
    pub bit_position: u8,
    pub bit_width: u8,
    pub optional: bool,
    pub kind_values: &'static [SyntaxKind],
}

#[derive(Debug)]
pub struct NodeLayout {
    pub name: &'static str,
    pub kinds: &'static [SyntaxKind],
    pub data_type: DataType,
    pub children: &'static [ChildProperty],
    pub common_data: &'static [CommonDataField],
    pub text_member: Option<&'static str>,
    pub hand_written_common_data: bool,
}

impl NodeLayout {
    /// A custom codec must supply extended data or the complete common-data word.
    pub fn requires_custom_codec(&self) -> bool {
        self.data_type == DataType::Extended || self.hand_written_common_data
    }

    /// Only ordinary one-child nodes encode that child's index directly.
    pub fn single_child(&self) -> Option<&ChildProperty> {
        if self.data_type != DataType::Extended && self.children.len() == 1 {
            self.children.first()
        } else {
            None
        }
    }
}
