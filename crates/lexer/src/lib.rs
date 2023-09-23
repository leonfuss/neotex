#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://gist.githubusercontent.com/leonfuss/9247909d6237cb406439944fe22405a4/raw/02853019c2e0187bfb518dba1052ed5338c144c8/logo.svg"
)]
#![doc(
    html_favicon_url = "https://gist.githubusercontent.com/leonfuss/fa44f11267796b352edaa121675cd6f2/raw/ae237aa701a5d087c89af313299858f5e1c1144f/favicon.svg"
)]

//! The `lexer` crate is responsible for the initial processing of LaTeX source code,
//! converting it into a format suitable for subsequent parsing. It serves as the 0th
//! step in the parsing pipeline, taking raw LaTeX code and transforming it into a
//! series of tokens, each representing a distinct element or command in the source.
//!
//! This tokenization process is essential for several reasons:
//!
//! 1. It breaks down complex LaTeX code into smaller, manageable units, making it
//!    easier to analyze and understand.
//!
//! 2. It identifies and categorizes different types of tokens, such as commands,
//!    words, symbols, and whitespace, providing structured information for further
//!    parsing and interpretation.
//!
//! Overall, the `lexer` crate plays a critical role in preparing LaTeX source code for
//! parsing and ensures that subsequent stages of the parsing pipeline can operate
//! efficiently and accurately.
//!
//! Inspired by [rustc_lexer](<https://github.com/rust-lang/rust/tree/master/compiler/rustc_lexer>)
//!
//! **Part of [NeoTeX](../neotex/index.html)**

/// Curser module - only visible to allow conditional integration tests
pub mod cursor;
/// This module defines the core functionality for tokenizing LaTeX source code.
/// Tokenization is the process of breaking down LaTeX code into individual tokens,
/// such as commands, words, and symbols, making it easier for further processing.
///
/// The module provides a [`Token`] struct representing a token with its kind and length,
/// as well as the [`TokenKind`] enum, which enumerates the various types of tokens that can be encountered.
/// Additionally, the `tokenize` function is provided to tokenize a LaTeX source code string
/// into an iterator of [`Token`] instances.
///
/// Users of this module can tokenize LaTeX code and work with individual tokens for parsing,
/// analysis, or other processing tasks.
pub mod token;

pub use token::tokenize;
pub use token::Token;
pub use token::TokenKind;

#[cfg(test)]
mod tests;
