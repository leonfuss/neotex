use lexer::TokenKind;

/// The Kind of a Syntax Node
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    /// Command tokens
    Command,
    /// Called if it could not be determined if token is function or macro
    CommandOrFunction,
    /// FunctionToken
    Function,

    // Whitespace and formatting tokens
    /// Whitespace token (e.g., ' ')
    Whitespace,
    /// Newline token (e.g., '\n')
    Newline,
    /// Multiple Newline tokens
    Break,

    // Comment tokens
    /// Comment token (e.g., '% Comment')
    Comment,
    /// Annotated Comment token (e.g., '%% Annotated Comment')
    AComment,

    // Textual content tokens
    /// Unicode-Word token (e.g., 'wòad')
    Word,
    /// ASCII-Word token (e.g., 'ASCII')
    AWord,
    /// Number token (e.g., '123')
    Number,

    // Delimiter tokens
    /// Open brace token (e.g., '{')
    OpenBrace,
    /// Close brace token (e.g., '}')
    CloseBrace,
    /// Open bracket token (e.g., '[')
    OpenBracket,
    /// Close bracket token (e.g., ']')
    CloseBracket,
    /// Open parenthesis token (e.g., '(')
    OpenParen,
    /// Close parenthesis token (e.g., ')')
    CloseParen,

    // Operator tokens
    /// Star token (e.g., '*')
    Star,
    /// Number sign token (e.g., '#')
    NumSign,
    /// Carret token (e.g., '^')
    Carret,
    /// Less than token (e.g., '<')
    Less,
    /// Less than or equal token (e.g., '<=')
    LessEq,
    /// Greater than token (e.g., '>')
    Greater,
    /// Greater than or equal token (e.g., '>=')
    GreaterEq,
    /// Underscore token (e.g., '_')
    Underscore,
    /// Apostrophe token (e.g., '\'')
    Apostrophe,
    /// Slash token (e.g., '/')
    Slash,
    /// Tilde token (e.g., '~')
    Tilde,
    /// Comma token (e.g., ',')
    Comma,
    /// Semicolon token (e.g., ';')
    Semicolon,
    /// Ampersand token (e.g., '&')
    Ampersand,
    /// Equal token (e.g., '=')
    Equal,
    /// Pipe token (e.g., '|')
    Pipe,
    /// Colon token (e.g., ':')
    Colon,
    /// Minus token (e.g., '-')
    Minus,
    /// Plus token (e.g., '+')
    Plus,
    /// Dot token (e.g., '.')
    Dot,
    /// Question mark token (e.g., '?')
    Question,
    /// Exclamation mark token (e.g., '!')
    Bang,
    /// Not equal token (e.g., '!=')
    NotEq,
    /// At token (e.g., '@')
    At,
    /// Left arrow token (e.g., '<-')
    LeftArrow, // <-
    /// Right arrow token (e.g., '->')
    RightArrow, // ->

    // Math delimiter token
    /// Math delimiter token (e.g., '$')
    MathDelimiter, // $

    // Document structure tokens
    /// ROOT token
    ROOT,
    /// PREAMBLE token
    PREAMBLE,
    /// DOCUMENT token
    DOCUMENT,

    // Modes
    /// CODE mode token
    CODE,
    /// TEXT mode token
    TEXT,
    /// MATH mode token
    MATH,

    // Grouping tokens
    /// BLOCK grouping token (e.g., '{...}')
    BLOCK,
    /// OPTIONBLOCK grouping token (e.g., '[...]')
    OPTIONBLOCK,

    // Fixed identifier
    /// The modul or package identifier in between '::'
    Namespace,
    /// module path seperator
    PathSeperator, // '::' in function and variable calls
    /// Function declaration
    FunctionIdentifier, // \fn
    /// Module Declaration
    Module, // \mod
    /// Variable declaration
    Variable, // \@<name> only with underscore and ASCII-Word
    /// usage decleration. eg. \use ::sdaf::asdfa
    Use,

    /// BeginGroup token (e.g., '\begin')
    BeginGroup, // \begin
    /// EndGroup token (e.g., '\end')
    EndGroup, // \end
    /// DocClass token (e.g., '\usepackage')
    DocClass, // \usepackage
    /// FileInput token (e.g., '\import', '\include', '\input')
    FileInput, // \import, \include, \input
    /// PackageInput token (e.g., '\usepackage')
    PackageInput, // \usepackage
    /// ProvidesPackage token (e.g., '\ProvidesPackage')
    ProvidesPackage, // \ProvidesPackage
    /// NeedsTeXFormat token (e.g., '\NeedsTeXFormat')
    NeedsTeXFormat, // \NeedsTeXFormat
    /// End-of-file token
    Eof,

    /// Error token
    Error,
}

pub(crate) trait Trivia {
    fn is_trivia(&self) -> bool;
}

impl Trivia for SyntaxKind {
    #[inline(always)]
    fn is_trivia(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Whitespace | SyntaxKind::Newline | SyntaxKind::Comment
        )
    }
}

impl Trivia for TokenKind {
    #[inline(always)]
    fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace | TokenKind::Newline | TokenKind::Comment | TokenKind::AComment
        )
    }
}
