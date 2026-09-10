//! Safe, isolated construction prototypes; not the compiler's storage backend.
#![forbid(unsafe_code)]

pub mod chunks;
pub mod lists;
mod pages;
#[allow(dead_code)] // This isolated text prototype has no production caller yet.
mod text;
