use crate::{NodeKind, SyntaxKind};

/// Pinned Go operator precedence values, including the invalid sentinel.
pub mod operator_precedence {
    pub const COMMA: i32 = 0;
    pub const SPREAD: i32 = 1;
    pub const YIELD: i32 = 2;
    pub const ASSIGNMENT: i32 = 3;
    pub const CONDITIONAL: i32 = 4;
    pub const LOGICAL_OR: i32 = 5;
    pub const LOGICAL_AND: i32 = 6;
    pub const BITWISE_OR: i32 = 7;
    pub const BITWISE_XOR: i32 = 8;
    pub const BITWISE_AND: i32 = 9;
    pub const EQUALITY: i32 = 10;
    pub const RELATIONAL: i32 = 11;
    pub const SHIFT: i32 = 12;
    pub const ADDITIVE: i32 = 13;
    pub const MULTIPLICATIVE: i32 = 14;
    pub const EXPONENTIATION: i32 = 15;
    pub const UNARY: i32 = 16;
    pub const UPDATE: i32 = 17;
    pub const LEFT_HAND_SIDE: i32 = 18;
    pub const OPTIONAL_CHAIN: i32 = 19;
    pub const MEMBER: i32 = 20;
    pub const PRIMARY: i32 = 21;
    pub const PARENTHESES: i32 = 22;
    pub const LOWEST: i32 = COMMA;
    pub const HIGHEST: i32 = PARENTHESES;
    pub const DISALLOW_COMMA: i32 = YIELD;
    pub const COALESCE: i32 = LOGICAL_OR;
    pub const INVALID: i32 = -1;
}

// port: tsc/internal/ast/precedence.go:GetBinaryOperatorPrecedence
pub fn get_binary_operator_precedence(operator: NodeKind) -> i32 {
    use operator_precedence as p;
    use SyntaxKind as K;
    match operator.known() {
        Some(K::QuestionQuestionToken) => p::COALESCE,
        Some(K::BarBarToken) => p::LOGICAL_OR,
        Some(K::AmpersandAmpersandToken) => p::LOGICAL_AND,
        Some(K::BarToken) => p::BITWISE_OR,
        Some(K::CaretToken) => p::BITWISE_XOR,
        Some(K::AmpersandToken) => p::BITWISE_AND,
        Some(
            K::EqualsEqualsToken
            | K::ExclamationEqualsToken
            | K::EqualsEqualsEqualsToken
            | K::ExclamationEqualsEqualsToken,
        ) => p::EQUALITY,
        Some(
            K::LessThanToken
            | K::GreaterThanToken
            | K::LessThanEqualsToken
            | K::GreaterThanEqualsToken
            | K::InstanceOfKeyword
            | K::InKeyword
            | K::AsKeyword
            | K::SatisfiesKeyword,
        ) => p::RELATIONAL,
        Some(
            K::LessThanLessThanToken
            | K::GreaterThanGreaterThanToken
            | K::GreaterThanGreaterThanGreaterThanToken,
        ) => p::SHIFT,
        Some(K::PlusToken | K::MinusToken) => p::ADDITIVE,
        Some(K::AsteriskToken | K::SlashToken | K::PercentToken) => p::MULTIPLICATIVE,
        Some(K::AsteriskAsteriskToken) => p::EXPONENTIATION,
        _ => p::INVALID,
    }
}
