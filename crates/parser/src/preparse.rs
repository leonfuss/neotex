/// This module defines a parser for processing LaTeX syntax.
/// It converts LaTeX source code into a sequence of syntax tokens and
/// extracts the position of LaTeX commands definitions.
use crate::SyntaxKind;
use crate::SyntaxKind::*;
use lexer::{Token, TokenKind};

use tracing::{instrument, trace};

/// A macro shorthand for matching lexed TokenKinds and converting them into SyntaxKind,
/// if the lex and syntax token name correspond. Only usefull if one-to-one conversion
/// is possible.
macro_rules! token_to_syntax {
    ( $var:expr; $($element:ident),+ ) => {
        {
            match $var {
                $(
                TokenKind::$element => $element,
                )*
                _ => unreachable!(),
            }
        }
    };
}

/// A macro for defining peek methods to check if the next token matches a given symbol.
macro_rules! is_peek {
    ($name:ident; $symbol:expr) => {
        #[inline(always)]
        fn $name(&self) -> bool {
            if let Some(t) = self.peek() {
                t == $symbol
            } else {
                false
            }
        }
    };
}

/// macro shorthand for pathspec seperators
macro_rules! name_seperator {
    ($name:ident, $func:ident, $err:expr) => {
        fn $name(&mut self) {
            assert_eq!(self.peek(), Some(TokenKind::Colon));
            self.bump();

            match self.peek() {
                Some(TokenKind::Colon) => {}
                _ => {
                    self.bump();
                    self.add_error_token(PreparseErrorKind::InvalidPathSpecSeperator);
                }
            }

            match self.peek() {
                Some(TokenKind::Word) | Some(TokenKind::Underscore) => {
                    self.$func();
                }
                _ => {
                    self.bump();
                    self.add_error_token($err)
                }
            }
        }
    };
}

/// Represents the result of pre-parsing LaTeX source code (parsing-stage 1). It holds a reference to the input string,
/// the pre-parsed syntax tokens and their corresponding byte start positions, resulting pre-parse errors,
/// as well as definitions in the pre-parsed code and their positions in bytes.
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug)]
pub(crate) struct LexedStr<'source> {
    src: &'source str,
    tokens: Vec<SyntaxKind>,
    start: Vec<usize>,
    errors: Vec<PreparseError>,
    definitions: Vec<Definition>,
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

        let token_iter = lexer::tokenize(input);

        let conv = Converter::new(token_iter, buf);

        trace!("Converting LexerToken to LexedStr");
        conv.convert()
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
    /// tokens extracted from the LaTeX source code.
    pub fn syntax_tokens(&self) -> impl Iterator<Item = &SyntaxKind> + '_ {
        self.tokens.iter()
    }

    /// Returns an iterator over the definitions found in the pre-parsed LaTeX source.
    ///
    /// During pre-parsing, this method identifies and marks specific LaTeX commands, including
    /// `\def`, `\import`, `\include`, `\usepackage`, `\newenvironment`, and `\newcommand`. These
    /// markings are essential for subsequent parsing steps, such as file lookup for imported files
    /// and extraction of argument spcific information on newly defined commands.
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
    ///         DefinitionKind::Include => println!("Found a LaTeX \\include command."),
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
    ///         PreparseErrorKind::CommandNameMissing => println!("Command name is missing."),
    ///         PreparseErrorKind::InvalidCommandName => println!("Invalid command name."),
    ///         PreparseErrorKind::InvalidCommandNameEnding => println!("Invalid command name ending."),
    ///         // Handle other error types as needed.
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn errors(&self) -> impl Iterator<Item = &PreparseError> + '_ {
        self.errors.iter()
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
    pub fn len(&self) -> usize {
        self.tokens.len() - 1
    }
}

/// Represents the different types of LaTeX definitions.
///
/// The `DefinitionKind` enum categorizes LaTeX definitions into distinct types, including
/// package imports, includes, imports, new command definitions, new environment definitions,
/// and LaTeX `\def` commands.
///
/// This categorization allows for easy identification and processing of different types of
/// LaTeX constructs within the pre-parsed source code.
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum DefinitionKind {
    /// Represents LaTeX `\package` command.
    Package,
    /// Represents  LaTeX `\include` command.
    Include,
    /// Represents  LaTeX '\input' command.
    Input,
    /// Represents  LaTeX `\newcommand`.
    Command,
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
    #[cfg(feature = "integration-tests")]
    pub kind: DefinitionKind,
    #[cfg(not(feature = "integration-tests"))]
    kind: DefinitionKind,
    /// The index of the definition within the pre-parsed source code tokens.
    idx: usize,
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

/// Represents the different types of pre-parse errors in LaTeX source code.
///
/// The `PreparseErrorKind` enum categorizes pre-parse errors into distinct types, providing
/// information about the specific issue encountered during pre-parsing. These error types
/// help identify problems early in the processing pipeline.
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug)]
pub(crate) enum PreparseErrorKind {
    /// Indicates that a command name is missing, typically occurring when a command
    /// identifier ('\\') is not followed by a valid command name but by whitespace,
    /// newline, or the end of the file.
    CommandNameMissing,
    /// Indicates an invalid command name, often triggered by a command identifier
    /// followed by a non-ASCII sequence, which is not allowed for commands.
    InvalidCommandName,
    /// Signifies an issue where a command name ends with a colon (':') or an underscore ('_'),
    /// both of which are only allowed as prefix or midsection characters of a command but not
    /// as postfix characters.
    InvalidCommandNameEnding,
    /// Signifies, that a multichar command started with a number. Numbers are only allowed in
    /// midsection and postfix position.
    InvalidCommandPrefix,
    /// Indicates that a variable name after the '@' identifier is missing
    VariableNameMissing,
    /// Indicates an invalid variable name. Variable names are only allowed to consist of
    /// ASCII-Words and Underscores after the Variable identifier at ('@').
    InvalidVariableName,
    /// Signifies an issue where a variable name ends with an underscore ('_')., which is only
    /// allowed as prefix or midsection character but not as postfix character.
    InvalidVariableNameEnding,
    /// Signifies an invalid pathspec seperator. Pathspec seperator must be exacltly '::'
    InvalidPathSpecSeperator,
    /// Signifies and invalid function name. Functions consist of ASCII-Words, Numbers and
    /// Underscores. Numbers must not occcur as prefix, while in contrast Underscores are not
    /// allowed to occur at postfix position
    InvalidFunctionName,
}

/// Represents a pre-parse error encountered during the pre-parsing of LaTeX source code.
///
/// A [`PreparseError`] encapsulates information about a specific pre-parse error, including
/// its type and the position in the source code where the error occurred.
///
/// # Fields
///
/// - `kind`: The type or category of the pre-parse error, represented by a [`PreparseErrorKind`].
///
/// - `idx`: The index of the source code tokens where the error occurred. See [`LexedStr`]
///          for more.
///
/// # Examples
///
/// ```rust
/// # use parser::preparse::LexedStr;
/// # let input = "asdfasdf";
/// let lexed = LexedStr::new(input);
///
/// for err in lexed.errors(){
///    match err {
///       // do something with the errors
///       _ => {}
///    }
/// }
/// ```
#[cfg_attr(feature = "integration-tests", visibility::make(pub))]
#[derive(Debug)]
pub(crate) struct PreparseError {
    /// The type or category of the pre-parse error.
    #[cfg(feature = "integration-tests")]
    pub kind: PreparseErrorKind,
    #[cfg(not(feature = "integration-tests"))]
    kind: PreparseErrorKind,
    /// The index of the source code tokens where the error occurred.
    idx: usize,
}

impl PreparseError {
    /// Creates a new [`PreparseError`] instance with the specified kind and index.
    ///
    /// # Parameters
    ///
    /// - `kind`: The type or category of the pre-parse error, represented by a [`PreparseErrorKind`].
    ///
    /// - `idx`: The position or index in the source code where the error occurred.
    ///
    /// # Returns
    ///
    /// A [`PreparseError`] instance with the provided kind and index.
    pub fn new(kind: PreparseErrorKind, idx: usize) -> PreparseError {
        PreparseError { kind, idx }
    }
}

/// A utility struct responsible for converting Lexer tokens into pre-parsed syntax tokens.
///
/// The [`Converter`] struct plays a crucial role in the pre-parsing process of LaTeX source code.
/// It takes Lexer tokens as input and converts them into pre-parsed syntax tokens, marking
/// recognized LaTeX constructs, errors, and definitions. The primary goal of this struct is
/// to prepare the source code for further parsing and analysis.
///
/// This struct encapsulates the logic for identifying and processing LaTeX commands, errors,
/// and definitions, but the specific implementation details are considered internal and may
/// be subject to change.
#[derive(Debug)]
struct Converter<'source, I: Iterator<Item = Token>> {
    lexed: LexedStr<'source>,
    token_iter: I,
    first_peek_item: Option<Token>,
    second_peek_item: Option<Token>,
    position: usize,
    token_size: usize,
}

impl<'source, I: Iterator<Item = Token>> Converter<'source, I> {
    /// Creates a new [`Converter`] instance.
    pub(crate) fn new(mut iter: I, lexed: LexedStr) -> Converter<I> {
        let peek_first = iter.next();
        let peek_second = iter.next();
        Converter {
            lexed,
            token_iter: iter,
            first_peek_item: peek_first,
            second_peek_item: peek_second,
            position: 0,
            token_size: 0,
        }
    }

    // === Core Conversion Logic ===

    /// Converts Lexer tokens into pre-parsed syntax tokens.
    pub(crate) fn convert(mut self) -> LexedStr<'source> {
        while let Some(token) = self.bump() {
            self.token(token);
        }

        self.lexed
    }

    /// Handles the pre-parsing of a Lexer token.
    #[instrument(skip(self))]
    fn token(&mut self, token: TokenKind) {
        trace!(?token, "match new token");
        match token {
            TokenKind::CommandIdent => self.command_or_var(),
            TokenKind::Newline => self.newline_or_break(),
            TokenKind::Less if self.is_peek_eq() => self.add_token(LessEq),
            TokenKind::Less if self.is_peek_minus() => self.add_token(LeftArrow),
            TokenKind::Greater if self.is_peek_eq() => self.add_token(GreaterEq),
            TokenKind::Bang if self.is_peek_eq() => self.add_token(NotEq),
            TokenKind::Minus if self.is_peek_greater() => self.add_token(RightArrow),
            TokenKind::Dollar => self.add_token(MathDelimiter),
            _ => {
                let syntax_kind = self.basic_token_kind(token);
                self.add_token(syntax_kind)
            }
        }
    }

    // === Token Processing Helper ===

    /// Peeks at the next Lexer token.
    #[inline(always)]
    fn peek(&self) -> Option<TokenKind> {
        self.first_peek_item.as_ref().map(|it| it.kind)
    }

    /// Peeks at the second Lexer token.
    #[inline(always)]
    fn peek_second(&self) -> Option<TokenKind> {
        self.second_peek_item.as_ref().map(|it| it.kind)
    }

    /// Advances to the next Lexer token.
    #[inline]
    #[instrument(
        skip(self),
        fields(self.lexed.tokens.len = ?self.lexed.tokens.len())
    )]
    fn bump(&mut self) -> Option<TokenKind> {
        if let Some(Token { kind, len }) = self.first_peek_item {
            self.position += len;
            self.token_size += len;
            // advanced peek by one
            self.first_peek_item = std::mem::take(&mut self.second_peek_item);
            self.second_peek_item = self.token_iter.next();

            trace!(?kind, "bump token to");

            return Some(kind);
        }
        None
    }

    // === Preparsed Syntax-Token Handling ===

    /// Adds a pre-parsed syntax token. Do not call for [`Error`] or [`Command`] directly!
    /// Prefer `self.add_error_token()` and `self.add_command_token()`.
    #[inline]
    fn add_token(&mut self, syntax_kind: SyntaxKind) {
        self.lexed.tokens.push(syntax_kind);
        self.lexed.start.push(self.position - self.token_size);
        self.reset_token_size();
    }

    /// Resets the current token size to zero.
    fn reset_token_size(&mut self) {
        self.token_size = 0
    }

    /// Adds a pre-parsed syntax token for a pre-parse error.
    fn add_error_token(&mut self, err: PreparseErrorKind) {
        self.add_token(Error);

        let idx = self.lexed.tokens.len();
        let err = PreparseError::new(err, idx);

        self.lexed.errors.push(err);
    }

    /// Adds a pre-parsed syntax token for a recognized LaTeX command.
    #[instrument(
        skip(self),
        fields(self.lexed.tokens.len = ?self.lexed.tokens.len())
    )]
    fn add_command_token(&mut self, is_command: bool) {
        trace!("adding command token");

        let begin = self.position - self.token_size;
        let end = self.position;
        let command_name = &self.lexed.src[begin..end];

        let mut is_command_or_function = false;
        match command_name {
            "\\def" => self.add_definition(DefinitionKind::Def),
            "\\newcommand" => self.add_definition(DefinitionKind::Command),
            "\\newenvironment" => self.add_definition(DefinitionKind::Environment),
            "\\usepackage" => self.add_definition(DefinitionKind::Package),
            "\\input" => self.add_definition(DefinitionKind::Input),
            "\\include" => self.add_definition(DefinitionKind::Include),
            "\\mod" => self.add_token(Module),
            "\\fn" => self.add_token(FunctionIdentifier),
            _ => is_command_or_function = true,
        }

        if is_command || !is_command_or_function {
            self.add_token(Command);
        } else {
            self.add_token(CommandOrFunction)
        }
    }

    // === Command, Variable and Definition Processing ===

    /// Handles the pre-parsing of a LaTeX command.
    #[instrument(
        skip(self),
        fields(self.lexed.tokens.len = ?self.lexed.tokens.len())
    )]
    fn command_or_var(&mut self) {
        trace!("begin command matching");

        let mut is_number = false;
        // used to determine later on if name is valid;
        let mut is_valid_function_name = false;
        // matching simple error cases
        match self.peek() {
            None
            | Some(TokenKind::Whitespace)
            | Some(TokenKind::Newline)
            | Some(TokenKind::Comment)
            | Some(TokenKind::AComment) => {
                return self.add_error_token(PreparseErrorKind::CommandNameMissing)
            }
            Some(TokenKind::Word) => {
                // eating erroneous token
                self.bump();
                return self.add_error_token(PreparseErrorKind::InvalidCommandName);
            }
            Some(TokenKind::At) => {
                self.bump();
                return self.variable();
            }
            Some(TokenKind::Number) => is_number = true,
            Some(TokenKind::AWord) | Some(TokenKind::Underscore) => is_valid_function_name = true,
            _ => {}
        };

        // single char and multichar commands remaining
        trace!("first command token valid");
        self.bump();

        // peek into second command token
        let Some(peek_second_token) = self.peek() else {
            return self.add_command_token(!is_valid_function_name);
        };

        match peek_second_token {
            TokenKind::AWord | TokenKind::Number | TokenKind::Underscore | TokenKind::Colon
                if is_number =>
            {
                // do not include '*'. We consider \8* valid command syntax.
                self.bump();
                self.add_error_token(PreparseErrorKind::InvalidCommandPrefix)
            }
            TokenKind::AWord | TokenKind::Number => {}
            TokenKind::Colon if self.peek_second() == Some(TokenKind::Colon) => {
                self.add_token(Function);
                return self.function_name_seperator();
            }
            TokenKind::Colon if self.is_valid_in_command_char() => {
                is_valid_function_name = false;
            }
            TokenKind::Underscore if self.is_valid_in_command_char() => {}
            TokenKind::Colon | TokenKind::Underscore => {
                // eating erroneous token
                self.bump();
                return self.add_error_token(PreparseErrorKind::InvalidCommandNameEnding);
            }
            TokenKind::Star => {
                // '*' terminates a command immediatly
                self.bump();
                return self.add_command_token(!is_valid_function_name);
            }
            _ => {
                // single token command
                return self.add_command_token(!is_valid_function_name);
            }
        };

        // advance by one token
        self.bump();

        // ending after second token with Number or AWord
        if self.peek().is_none() {
            return self.add_command_token(!is_valid_function_name);
        }

        trace!("second command token valid");

        // matching third Token and onwards
        while let Some(token) = self.peek() {
            match token {
                TokenKind::AWord | TokenKind::Number => {}
                TokenKind::Colon if self.peek_second() == Some(TokenKind::Colon) => {
                    self.add_token(Function);
                    return self.function_name_seperator();
                }
                TokenKind::Colon if self.is_valid_in_command_char() => {
                    is_valid_function_name = false;
                }
                TokenKind::Underscore if self.is_valid_in_command_char() => {}
                TokenKind::Colon | TokenKind::Underscore => {
                    // eating erroneous token
                    self.bump();
                    return self.add_error_token(PreparseErrorKind::InvalidCommandNameEnding);
                }
                TokenKind::Star => {
                    // '*' terminates command immediatly
                    self.bump();
                    break;
                }
                _ => break,
            }
            self.bump();
        }

        self.add_command_token(!is_valid_function_name);
    }

    /// Match a NeoTeX variable declaration. Variable name consist of an '\' followed by an '@'
    /// followed by a combination of [`AWord`] and [`Underscore`]. Command names can not end with
    /// an [`Underscore`]. The '@' must be bumped before calling this function.
    fn variable(&mut self) {
        let Some(token) = self.peek() else {
            return self.add_error_token(PreparseErrorKind::VariableNameMissing);
        };

        match token {
            TokenKind::Whitespace
            | TokenKind::Newline
            | TokenKind::Comment
            | TokenKind::AComment => {
                return self.add_error_token(PreparseErrorKind::VariableNameMissing);
            }
            TokenKind::AWord => {}
            TokenKind::Underscore if self.is_valid_in_variable_char() => {}
            TokenKind::Colon => self.variable_name_seperator(),
            _ => {
                self.bump();
                return self.add_error_token(PreparseErrorKind::InvalidVariableName);
            }
        }

        self.bump();

        while let Some(token) = self.peek() {
            match token {
                TokenKind::Word | TokenKind::Number => {}
                TokenKind::Underscore if self.is_valid_in_command_char() => {}
                TokenKind::Colon => {
                    self.add_token(Variable);
                    // seperator errors get handled by name_seperator()
                    return self.variable_name_seperator();
                }
                _ => break,
            }
            self.add_token(Variable)
        }
    }

    /// Handle functions. Should only be called from `self.function_name_seperator()`
    fn function(&mut self) {
        assert!(matches!(
            self.peek(),
            Some(TokenKind::AWord) | Some(TokenKind::Underscore)
        ));
        self.bump();

        while let Some(token) = self.peek() {
            match token {
                TokenKind::Underscore if self.is_valid_in_variable_char() => {}
                TokenKind::AWord | TokenKind::Number => {}
                TokenKind::Colon => {
                    self.add_token(Function);
                    // seperator errors get handled by name_seperator()
                    return self.function_name_seperator();
                }
                _ => break,
            }
            self.bump();
        }
        self.add_token(Function)
    }

    name_seperator!(
        function_name_seperator,
        function,
        PreparseErrorKind::InvalidFunctionName
    );

    name_seperator!(
        variable_name_seperator,
        variable,
        PreparseErrorKind::InvalidVariableName
    );

    /// Helper for checking if a current character is valid within a LaTeX command name based on
    /// next character.
    fn is_valid_in_command_char(&self) -> bool {
        if let Some(token) = self.peek_second() {
            matches!(
                token,
                TokenKind::AWord | TokenKind::Number | TokenKind::Underscore | TokenKind::Colon
            )
        } else {
            false
        }
    }

    /// Helper for checking if a churrent character is valid within a LaTeX command name based on
    /// next character.
    fn is_valid_in_variable_char(&self) -> bool {
        if let Some(token) = self.peek_second() {
            matches!(
                token,
                TokenKind::AWord | TokenKind::Number | TokenKind::Underscore
            )
        } else {
            false
        }
    }

    /// Adds a new definition to the pre-parsed source.
    fn add_definition(&mut self, kind: DefinitionKind) {
        let idx = self.lexed.tokens.len();
        let def = Definition::new(kind, idx);

        self.lexed.definitions.push(def);
    }

    // === Simple Token Processing Functions ===

    /// Handles the pre-parsing of newline or break tokens.
    fn newline_or_break(&mut self) {
        let token: SyntaxKind;
        if let Some(TokenKind::Newline) = self.peek() {
            while let Some(TokenKind::Newline) = self.peek() {
                self.bump();
            }
            token = Break;
        } else {
            token = Newline;
        }
        self.add_token(token);
    }

    /// Determines the pre-parsed syntax token kind for a basic Lexer token. Only 1-to-1 conversion
    /// occur here.
    fn basic_token_kind(&self, token: TokenKind) -> SyntaxKind {
        token_to_syntax!(token;
            Whitespace,
            Word,
            AWord,
            Comment,
            AComment,
            Number,
            OpenBrace,
            CloseBrace,
            OpenBracket,
            CloseBracket,
            OpenParen,
            CloseParen,
            Star,
            NumSign,
            Carret,
            Underscore,
            Apostrophe,
            Slash,
            Tilde,
            Comma,
            Semicolon,
            Ampersand,
            Pipe,
            Colon,
            Plus,
            Minus,
            Dot,
            Question,
            At,
            Equal,
            Less,
            Greater,
            Bang
        )
    }

    // === Helper ===

    // peek for '>'
    is_peek!(is_peek_greater; TokenKind::Greater);

    // peek for '-'
    is_peek!(is_peek_minus; TokenKind::Minus);

    // peek for '='
    is_peek!(is_peek_eq; TokenKind::Equal);
}
