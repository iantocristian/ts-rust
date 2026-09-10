//! Checker-local identities. Upstream numbers types and signatures per checker
//! from 1 (`Checker.TypeCount`, `SignatureCount`); the numbers are not unique
//! across checkers, which is why they never leave an operation scope on their
//! own (ADR 0007, plan §4.1). Retention and cross-checker validation are the
//! owner's job; these values only index this checker's stores.

use std::num::NonZeroU32;

macro_rules! local_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name(NonZeroU32);
        impl $name {
            /// The upstream numeric id; only for display, ordering fallbacks and
            /// diagnostics, never for reconstructing a handle elsewhere.
            pub fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

local_id!(
    TypeId,
    "A type in one checker's type store (`checker.TypeId`)."
);
local_id!(
    SignatureId,
    "A signature in one checker's signature store (`checker.SignatureId`)."
);

impl TypeId {
    pub(crate) fn new(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self)
    }
    pub(crate) fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

impl SignatureId {
    /// Minted by the signature store, which arrives with the P3 signatures
    /// family; until then only tests construct one.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self)
    }
}
