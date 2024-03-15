#[derive(Debug)]
pub(crate) enum PreparseErrorKind {
    /// Indicates that a command name is missing, typically occurring when a command
    /// identifier ('\\') is not followed by a valid command name but by whitespace,
    /// newline, or the end of the file.
    CommandNameMissing,
    /// Indicates an invalid command name, often triggered by a command identifier
    /// followed by a non-ASCII sequence, which is not allowed for commands.
    InvalidCommandName,
    /// Indicates an invalid variable name. Variable names are only allowed to consist of
    /// ASCII-Words and Underscores after the Variable identifier at ('@').
    InvalidVariableName,
}

#[derive(Debug)]
pub(crate) struct PreparseError {
    pub kind: PreparseErrorKind,
    /// The index in the source code where the error occurred.
    pub idx: usize,
}

impl PreparseError {
    pub fn new(kind: PreparseErrorKind, idx: usize) -> PreparseError {
        PreparseError { kind, idx }
    }
}
