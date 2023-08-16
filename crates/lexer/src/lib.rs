//! Low level Latex/Tex Parser
//!
//! Lexer is the first preperation step of converting latex code,
//! to a loseless syntax tree.
//!
//! This crate is very much inspired by 'rustc_lexer'[<https://github.com/rust-lang/rust/tree/master/compiler/rustc_lexer>]

mod cursor;
mod token;

pub mod lexed_str;
pub mod syntax;

#[cfg(test)]
mod tests;
