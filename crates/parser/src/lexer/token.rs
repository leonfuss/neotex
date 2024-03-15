use super::infra::Tombstone;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum LexToken {
    /// Backslash character ('\\')
    CommandIdent,
    /// Command name, such as 'newcommand', 'renewcommand', etc. after a backslash.
    Command,
    /// '@' character after a backslash.
    VariableIdent,
    /// A variable name, such as 'jobname', 'year', etc. after a backslash and '@'.
    Variable,

    /// Space or tab characters
    Whitespace,
    /// A newline character.
    Newline,
    /// A break token consisting of at least two newline characters.
    Break,
    /// A line comment beginning with '%' and ending with the end of line.
    Comment,
    /// A numeric literal composed of digits and optional '_' separators.
    Integer,
    /// A float consisting of a sequence of digits, an optional decimal point, and an optional exponent.
    Float,
    /// A unit of measure, such as 'pt', 'cm', 'in', etc.
    Unit,
    /// A word composed of ascii alphabetic characters.
    AWord,
    /// A word composed of unicode alphabetic characters.
    UWord,
    /// An opening curly brace ('{').
    OpenBrace,
    /// A closing curly brace ('}').
    CloseBrace,
    /// An opening square bracket ('[').
    OpenBracket,
    /// A closing square bracket (']').
    CloseBracket,
    /// An opening parenthesis ('(').
    OpenParen,
    /// A closing parenthesis (')').
    CloseParen,
    /// An asterisk ('*').
    Star,
    /// A hash ('#') character.
    NumSign,
    /// A caret ('^') character.
    Carret,
    /// A less-than ('<') character.
    Less,
    /// A greater-than ('>') character.
    Greater,
    /// An underscore ('_') character.
    Underscore,
    /// A single quote ('\'') character.
    SingleApostrophe,
    /// A double quote ('"') character.
    DoubleApostrophe,
    /// A forward slash ('/') character.
    Slash,
    /// A tilde ('~') character.
    Tilde,
    /// A comma (',') character.
    Comma,
    /// A semicolon (';') character.
    Semicolon,
    /// An ampersand ('&') character.
    Ampersand,
    /// An equals ('=') character.
    Equal,
    /// A pipe ('|') character.
    Pipe,
    /// A colon (':') character.
    Colon,
    /// A dollar ('$') character.
    Dollar,
    /// A hyphen-minus ('-') character.
    Minus,
    /// A plus ('+') character.
    Plus,
    /// A period ('.') character.
    Period,
    /// An at ('@') character.
    At,
    /// A question mark ('?') character.
    Question,
    /// An exclamation mark ('!') character.
    Bang,

    // composite tokens
    /// '$$'
    MathDisplay,
    /// '::'
    PathSeparator,
    /// '->'
    RightArrow,
    /// '<-'
    LeftArrow,
    /// '=>'
    LessEqual,
    /// '>='
    GreaterEqual,
    /// '!='
    NotEqual,
    /// '=='
    DoubleEqual,
    /// '+='
    PlusEqual,
    /// '-='
    MinusEqual,
    /// '*='
    MulEqual,
    /// '/='
    DivEqual,

    /// All other non unicode alphabetic characters.
    Symbol,

    /// Unicode Escaped character sequence. eg. '\u{1F4A9}'.
    /// Note: May be not valid. In event of EOF it will still report as UnicodeEscape to prevent macro expansion
    /// and allow for error handling by the parser.
    UnicodeEscape,

    /// A hash ('#') character followed by Unicode xid_start and xid_continue characters except the underscore.
    MacroParameter,

    /// A token representing the end of the input stream.
    Eof,
}

impl Tombstone for LexToken {
    fn tombstone() -> LexToken {
        Self::Eof
    }
}
