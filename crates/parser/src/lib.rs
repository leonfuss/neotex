#![doc(
    html_logo_url = "https://gist.githubusercontent.com/leonfuss/9247909d6237cb406439944fe22405a4/raw/02853019c2e0187bfb518dba1052ed5338c144c8/logo.svg"
)]
#![doc(
    html_favicon_url = "https://gist.githubusercontent.com/leonfuss/fa44f11267796b352edaa121675cd6f2/raw/ae237aa701a5d087c89af313299858f5e1c1144f/favicon.svg"
)]
#![warn(missing_docs)]

//! **Part of [NeoTeX](../neotex/index.html)**
/// The `preparse` module serves as the initial step in the parsing pipeline for LaTeX source code.
///
/// Its primary purpose is to perform a preliminary analysis and conversion of Lexer tokens into a
/// more structured form of syntax tokens. This conversion allows for a more organized
/// representation of the LaTeX source code.
///
/// Key functionalities of the `preparse` module include:
///
/// 1. **Lexer Token Conversion**: Conversion of Lexer tokens into pre-parsed syntax tokens.
/// 2. **Syntax Token Grouping**: Grouping of consecutive Lexer tokens into single syntax tokens,
///    simplifying subsequent parsing steps.
/// 3. **Identification of Special Tokens**: Recognition and extraction of special tokens, such as
///    macro and environment definitions, package imports, and external file input commands.
/// 4. **Error Handling**: Detection and marking of pre-parse errors in the source code.
///
/// This module acts as a crucial bridge between the Lexer and the core parsing stages, preparing
/// the LaTeX source code for more in-depth analysis. It enhances code structure, identifies
/// special tokens, and captures error information, all contributing to a more robust and
/// efficient parsing process.
pub mod preprocessor;

/// LaTeX Syntax Tokens
pub mod syntax;

pub mod expansion;

mod lexer;
mod parser;
mod utils;

pub use syntax::SyntaxKind;
