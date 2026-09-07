//! Shared scanner configuration and signed source ranges.
//!
//! This dependency slice does not implement compiler options or the remaining
//! core algorithms. Integer newtypes retain Go's open numeric value domain.

/// Pinned `core.ScriptTarget` values, including None and JSON.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptTarget(pub i32);

impl ScriptTarget {
    pub const NONE: Self = Self(0);
    pub const ES5: Self = Self(1);
    pub const ES2015: Self = Self(2);
    pub const ES2016: Self = Self(3);
    pub const ES2017: Self = Self(4);
    pub const ES2018: Self = Self(5);
    pub const ES2019: Self = Self(6);
    pub const ES2020: Self = Self(7);
    pub const ES2021: Self = Self(8);
    pub const ES2022: Self = Self(9);
    pub const ES2023: Self = Self(10);
    pub const ES2024: Self = Self(11);
    pub const ES2025: Self = Self(12);
    pub const ESNEXT: Self = Self(99);
    pub const JSON: Self = Self(100);
    pub const LATEST: Self = Self::ESNEXT;
    pub const LATEST_STANDARD: Self = Self::ES2025;
}

/// Pinned `core.LanguageVariant`; unknown integers remain representable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LanguageVariant(pub i32);

impl LanguageVariant {
    pub const STANDARD: Self = Self(0);
    pub const JSX: Self = Self(1);
}

/// Go stores each position as int32, although its accessors use machine ints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextRange {
    pos: i32,
    end: i32,
}

impl TextRange {
    /// Narrows deliberately, including synthetic negative positions.
    /// port: tsc/internal/core/text.go:NewTextRange
    pub const fn new(pos: i64, end: i64) -> Self {
        Self {
            pos: pos as i32,
            end: end as i32,
        }
    }

    /// port: tsc/internal/core/text.go:TextRange.Pos
    pub fn pos(self) -> i64 {
        i64::from(self.pos)
    }

    /// port: tsc/internal/core/text.go:TextRange.End
    pub fn end(self) -> i64 {
        i64::from(self.end)
    }

    /// The subtraction wraps at int32 before Go widens it to int.
    /// port: tsc/internal/core/text.go:TextRange.Len
    pub fn len(self) -> i64 {
        i64::from(self.end.wrapping_sub(self.pos))
    }

    pub fn is_empty(self) -> bool {
        self.pos == self.end
    }
}
