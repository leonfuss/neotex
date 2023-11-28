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
    InvalidCommandEnding,
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
    /// Specifies an unexpected pathspec module name. A pathspec module is the name between '::'s
    /// in a pathspec to a module of function
    InvalidPathSpecModuleName,
    /// Specifies an missing pathspec module name
    PathSpecModuleNameMissing,
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
    pub kind: PreparseErrorKind,
    /// The index of the source code tokens where the error occurred.
    pub idx: usize,
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
