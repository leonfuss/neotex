/// This module defines a parser for processing LaTeX syntax.
/// It converts LaTeX source code into a sequence of syntax tokens and
/// extracts the position of LaTeX commands definitions.
use crate::SyntaxKind;
use crate::SyntaxKind::*;
use crate::{preparse::converter::Converter, utils::utils::IndexedSliceView};

use tracing::{instrument, trace};

use super::errors::PreparseError;

/// Represents the result of pre-parsing LaTeX source code (parsing-stage 1). It holds a reference to the input string,
/// the pre-parsed syntax tokens and their corresponding byte start positions, resulting pre-parse errors,
/// as well as definitions in the pre-parsed code and their positions in bytes.
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug)]
pub(crate) struct LexedStr<'source> {
    pub(super) src: &'source str,
    pub(super) tokens: Vec<SyntaxKind>,
    pub(super) start: Vec<usize>,
    pub(super) errors: Vec<PreparseError>,
    pub(super) definitions: Vec<Definition>,
}

impl<'source> LexedStr<'source> {
    /// Create a new [`LexedStr`] by pre-parsing the provided LaTeX source code string.
    ///
    /// # Arguments
    ///
    /// * `input` - The LaTeX source code string to be pre-parsed.
    ///
    /// # Returns
    ///
    /// A [`LexedStr`] containing the pre-parsed syntax tokens, error information, and definitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use parser::preparse::LexedStr;
    /// let input = "\\documentclass{article} \\begin{document} Hello, World! \\end{document}";
    /// let lexed = LexedStr::new(input);
    ///
    /// for token in lexed.syntax_tokens() {
    ///     println!("{:?}", token);
    /// }
    /// ```
    ///
    /// This function takes an input LaTeX source code string and pre-parses it, generating a [`LexedStr`]
    /// that can be used to access the pre-parsed syntax tokens, errors, and definitions.
    #[instrument(skip(input))]
    pub fn new(input: &str) -> LexedStr {
        let buf = LexedStr {
            src: input,
            tokens: Vec::new(),
            start: Vec::new(),
            errors: Vec::new(),
            definitions: Vec::new(),
        };

        let tokens = lexer::tokenize(input);

        let conv = Converter::new(buf, tokens);

        trace!("Converting LexerToken to LexedStr");
        conv.transform()
    }

    /// Returns an iterator over the pre-parsed syntax tokens in the [`LexedStr`].
    ///
    /// This method allows you to iterate over the pre-parsed syntax tokens extracted from the
    /// LaTeX source code. Syntax tokens represent the fundamental elements of the LaTeX code
    /// and are categorized into various types such as keywords, identifiers, operators, and more.
    ///
    /// # Returns
    ///
    /// An iterator yielding [`SyntaxKind`] values representing the pre-parsed syntax tokens.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use parser::preparse::LexedStr;
    /// # use parser::SyntaxKind;
    /// let input = "\\section{Title} Some text.";
    /// let lexed = LexedStr::new(input);
    ///
    /// for token in lexed.syntax_tokens() {
    ///     match token {
    ///         SyntaxKind::Command => println!("Found a command token."),
    ///         SyntaxKind::Comment => println!("Found an comment token."),
    ///         // Handle other token types as needed.
    ///         _ => {}
    ///     }
    /// }
    /// ```
    ///
    /// This method is useful for performing further analysis or processing of the syntax
    /// tokens extracted from the LaTeX source code. This method ommits the last definition (EOF)
    /// which is only inserted to make life for definition indexes a bit easier.
    pub fn syntax_tokens(&self) -> impl Iterator<Item = &SyntaxKind> + '_ {
        self.tokens.iter().filter(|&&it| it != Eof)
    }

    // TODO: Documetn
    pub fn syntax_tokens_from_index(
        &'source self,
        idx: usize,
    ) -> impl Iterator<Item = (usize, SyntaxKind)> + 'source {
        self.tokens.iter().copied().enumerate().skip(idx)
    }

    /// Returns an iterator over the definitions found in the pre-parsed LaTeX source.
    ///
    /// During preparsing the \newcommand and \newenvironment macros are identified and picked out
    /// analysed for their argument count. This should reduce later modifications in the AST tree
    /// due to changes in the argument size.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to [`Definition`] structs representing recognized LaTeX definitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use parser::preparse::LexedStr;
    /// # use parser::preparse::DefinitionKind;
    /// let input = "\\newcommand{\\mycommand}[1]{...}";
    /// let lexed = LexedStr::new(input);
    ///
    /// for def in lexed.definitions() {
    ///     match def.kind {
    ///         DefinitionKind::Def => println!("Found a LaTeX \\def command."),
    ///         DefinitionKind::Input => println!("Found a LaTeX \\input command."),
    ///         DefinitionKind::Package => println!("Found a LaTeX \\usepackage command."),
    ///         DefinitionKind::Environment => println!("Found a LaTeX \\newenvironment command."),
    ///         DefinitionKind::Command => println!("Found a LaTeX \\newcommand definition."),
    ///         // Handle other definition types as needed.
    ///         _ => {}
    ///     }
    /// }
    /// ```
    ///
    /// This method plays a crucial role in enabling subsequent parsing steps to efficiently
    /// process LaTeX source code by providing information about recognized commands and their
    /// positions.
    pub fn definitions(&self) -> impl Iterator<Item = &Definition> + '_ {
        self.definitions.iter()
    }

    /// Returns an iterator over pre-parse errors encountered during pre-parsing.
    ///
    /// Pre-parse errors represent issues or inconsistencies found in the LaTeX source code
    /// during the pre-parsing phase. These errors are essential for identifying problems
    /// early in the processing pipeline.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to [`PreparseError`] structs representing pre-parse errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // the postfix '_' is not a valid command name
    /// # use parser::preparse::LexedStr;
    /// # use parser::preparse::PreparseErrorKind;
    /// let input = "\\invalid_command_{...}";
    /// let lexed = LexedStr::new(input);
    ///
    /// for error in lexed.errors() {
    ///     match error.kind {
    ///         PreparseErrorKind::CommandOrFunctionNameMissing => println!("Command name is missing."),
    ///         PreparseErrorKind::InvalidCommandOrFunctionName => println!("Invalid command name."),
    ///         PreparseErrorKind::InvalidCommandOrFunctionNameEnding => println!("Invalid command name ending."),
    ///         // Handle other error types as needed.
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn errors(&self) -> impl Iterator<Item = &PreparseError> + '_ {
        self.errors.iter()
    }
}

/// Represents the different types of LaTeX definitions.
///
/// The `DefinitionKind` enum categorizes LaTeX definitions into distinct types, including
/// package input, includes, imports, new command definitions, new environment definitions,
/// and LaTeX `\def` commands.
///
/// This categorization allows for easy identification and processing of different types of
/// LaTeX constructs within the pre-parsed source code.
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum DefinitionKind {
    /// Represents  LaTeX `\newcommand`.
    Macro,
    /// Represents LaTeX `\def`.
    Def,
    /// Represents LaTeX `\newenvironment`.
    Environment,
}

/// Represents a LaTeX definition found in the pre-parsed source code.
///
/// A [`Definition`] encapsulates information about a recognized LaTeX definition, including
/// its type and its index position in the token vector
///
/// # Fields
///
/// - `kind`: The type or category of the LaTeX definition, represented by a [`DefinitionKind`].
///
/// - `idx`: The index of the definition within the pre-parsed source code token vector.
///          See [`LexedStr`] for more
///
/// # Examples
///
/// ```rust
/// # use parser::preparse::LexedStr;
/// # let input = "asdfasdf";
/// let lexed = LexedStr::new(input);
///
/// for def in lexed.definitions(){
///    match def {
///       // do something with the definitions
///       _ => {}
///    }
/// }
/// ```
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug)]
pub(crate) struct Definition {
    /// The type or category of the LaTeX definition.
    pub kind: DefinitionKind,
    /// The index of the definition within the pre-parsed source code tokens.
    pub idx: usize,
}

impl Definition {
    /// Creates a new [`Definition`] instance with the specified kind and index.
    ///
    /// # Parameters
    ///
    /// - `kind`: The type or category of the LaTeX definition, represented by a [`DefinitionKind`].
    ///
    /// - `idx`: The position or index of the definition within the pre-parsed source code tokens
    ///          of [`LexedStr`].
    ///
    /// # Returns
    ///
    /// A [`Definition`] instance with the provided kind and index.
    ///
    pub fn new(kind: DefinitionKind, idx: usize) -> Definition {
        Definition { kind, idx }
    }
}

impl<'source> IndexedSliceView<'source> for LexedStr<'source> {
    #[inline]
    fn as_str(&self) -> &'source str {
        self.src
    }

    #[inline]
    fn start_of_index(&self, idx: usize) -> usize {
        *self.start.get(idx).or_else(|| self.start.last()).unwrap()
    }

    /// Returns the number of pre-parsed syntax tokens in the `LexedStr`.
    ///
    /// This method provides the count of pre-parsed syntax tokens extracted from the LaTeX
    /// source code. It can be used to determine the size or length of the pre-parsed code.
    ///
    /// # Returns
    ///
    /// The number of pre-parsed syntax tokens.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use parser::preparse::LexedStr;
    ///
    /// let input = "\\section{Title} Some text.";
    /// let lexed = LexedStr::new(input);
    ///
    /// let token_count = lexed.len();
    /// println!("Number of tokens: {}", token_count);
    /// ```
    #[inline]
    fn len(&self) -> usize {
        self.tokens.len()
    }
}
