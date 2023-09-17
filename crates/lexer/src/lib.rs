//! Low level Latex/Tex Parser
//!
//! Lexer is the first preperation step of converting LaTeX code,
//! to a loseless syntax tree.
//!
//! This crate is very much inspired by 'rustc_lexer'[<https://github.com/rust-lang/rust/tree/master/compiler/rustc_lexer>]

mod cursor;
pub mod token;
pub use token::tokenize;
pub use token::Token;
pub use token::TokenKind;

#[cfg(test)]
mod tests;
