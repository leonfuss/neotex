use self::TokenKind::*;
use crate::cursor::Cursor;

/// Represents a token in LaTeX source code.
///
/// A `Token` includes information about the type of the token (kind) and its length in bytes.
#[derive(Debug)]
pub struct Token {
    /// The kind or type of the token.
    pub kind: TokenKind,
    /// The length of the token in bytes.
    pub len: usize,
}

impl Token {
    /// Creates a new `Token` with the specified kind and length.
    fn new(kind: TokenKind, len: usize) -> Token {
        Token { kind, len }
    }
}

/// Enum representing common lexical types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifies a LaTeX command, typically starting with a backslash ('\').
    CommandIdent,
    // Whitespace
    /// Represents a whitespace character (e.g., space or tab).
    Whitespace, // ' '
    /// Represents a newline character.
    Newline,
    /// Represents a regular comment line (starting with '%').
    Comment, // %....\n
    /// Represents an annotated comment line (starting with '%%').
    AComment, // %% ... \n - Annotated Comment
    // Numbers and Words
    /// Represents a sequence of digits forming a number.
    Number,
    /// Represents a sequence of characters forming a word.
    Word,
    /// Represents a sequence of ASCII characters forming an ASCII word.
    AWord,
    // Braces
    /// Represents an opening brace symbol ('{').
    OpenBrace,
    /// Represents a closing brace symbol ('}').
    CloseBrace,
    /// Represents an opening bracket symbol ('[').
    OpenBracket,
    /// Represents a closing bracket symbol (']').
    CloseBracket,
    /// Represents an opening parenthesis symbol ('(').
    OpenParen,
    /// Represents a closing parenthesis symbol (')').
    CloseParen,
    // Special Symbols
    /// Represents the asterisk symbol ('*').
    Star,
    /// Represents the number sign or hash symbol ('#').
    NumSign,
    /// Represents the caret symbol ('^').
    Carret,
    /// Represents the less-than symbol ('<').
    Less,
    /// Represents the greater-than symbol ('>').
    Greater,
    /// Represents the underscore symbol ('_').
    Underscore,
    /// Represents the apostrophe symbol ("'").
    Apostrophe,
    /// Represents the slash symbol ('/').
    Slash,
    /// Represents the tilde symbol ('~').
    Tilde,
    /// Represents the comma symbol (',').
    Comma,
    /// Represents the semicolon symbol (';').
    Semicolon,
    /// Represents the ampersand symbol ('&').
    Ampersand,
    /// Represents the equal symbol ('=').
    Equal,
    /// Represents the pipe symbol ('|').
    Pipe,
    /// Represents the colon symbol (':').
    Colon,
    /// Represents the dollar symbol ('$').
    Dollar,
    /// Represents the minus symbol ('-').
    Minus,
    /// Represents the plus symbol ('+').
    Plus,
    /// Represents the dot symbol ('.').
    Dot,
    /// Represents the at symbol ('@').
    At,
    // Other Variants
    /// Represents a question mark symbol ('?').
    Question,
    /// Represents an exclamation mark symbol ('!').
    Bang,
    /// Represents an unknown or unrecognized token. This happens only if grapheme segmentation
    /// failed
    Unknown,

    /// Represents the end of input or the last token.
    Eof,
}

/// Tokenizes a LaTeX source code string into an iterator of `Token` objects.
///
/// This function takes an input string containing LaTeX source code and returns an iterator
/// that produces individual `Token` objects. It respects UTF-8 encoding and grapheme clusters,
/// ensuring correct tokenization of multi-byte characters.
///
/// # Example
///
/// ```rust
/// let latex_code = "\\documentclass{article}";
/// let tokens: Vec<_> = tokenize(latex_code).collect();
///
/// assert_eq!(tokens.len(), 5);
/// ```
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.advance_token();
        if token.kind != TokenKind::Eof {
            Some(token)
        } else {
            None
        }
    })
}

/// Checks single chars for whitespace characters. This does not
/// include newline characters. To check for newlines please use 'is_newline'
fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'   // \t
        | '\u{0020}' // space
        | '\u{00A0}' // no-break space
        | '\u{1680}' // ogham space mark
        | '\u{202F}' // narrow no-break space
        | '\u{3000}' // ideographic space

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK
    )
}

/// Checks newline character for a singel char. This does
/// of course not check for '\r\n'
fn is_newline(c: char) -> bool {
    matches!(
        c,
        '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0085}' // NEXT LINE from latin1

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

impl Cursor<'_> {
    fn advance_token(&mut self) -> Token {
        match self.bump() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, 0),
        };

        let token_kind = self.token_kind();

        let res = Token::new(token_kind, self.token_len());
        self.reset_token_len();
        res
    }

    fn token_kind(&mut self) -> TokenKind {
        let cluster = self.buf();

        // sort into ascii and non_ascii
        match cluster.len() {
            0 => return Unknown,
            1 => {}
            2 => return self.word_or_newline(),
            _ => return self.word(),
        };

        let ascii_char = cluster[0];

        match ascii_char {
            c if is_newline(c) => Newline,
            c if is_whitespace(c) => self.whitespace(),

            '%' => self.comment(),
            '\u{005C}' => CommandIdent,
            '0'..='9' => self.number(),

            '{' => OpenBrace,
            '}' => CloseBrace,
            '(' => OpenParen,
            ')' => CloseParen,
            '[' => OpenBracket,
            ']' => CloseBracket,

            '*' => Star,
            '.' => Dot,
            ';' => Semicolon,
            ',' => Comma,
            ':' => Colon,
            '#' => NumSign,
            '~' => Tilde,
            '$' => Dollar,
            '?' => Question,
            '!' => Bang,
            '&' => Ampersand,
            '=' => Equal,
            '<' => Less,
            '>' => Greater,
            '-' => Minus,
            '+' => Plus,
            '|' => Pipe,
            '^' => Carret,
            '\'' => Apostrophe,
            '_' => Underscore,
            '/' => Slash,
            '@' => At,

            c if c.is_ascii() => self.ascii_word(),
            _ => self.word(),
        }
    }

    fn whitespace(&mut self) -> TokenKind {
        self.eat_while(|c| matches!(c.len(), 1 if is_whitespace(c[0])));
        Whitespace
    }

    fn word_or_newline(&mut self) -> TokenKind {
        let cluster = self.buf();
        assert!(cluster.len() == 2);

        match cluster[0] {
            '\u{000D}' => Newline,
            _ => self.word(),
        }
    }

    fn comment(&mut self) -> TokenKind {
        let mut comment_ty = Comment;
        let mut buf: Vec<char>;

        loop {
            match self.first() {
                None => break,
                Some(c) => buf = c.chars().collect(),
            };

            match buf.len() {
                0 => break,
                1 if buf[0] == '%' => {
                    comment_ty = AComment;
                    self.bump()
                }
                1 if is_newline(buf[0]) => {
                    self.bump();
                    break;
                }
                2 if buf[0] == '\u{000D}' => {
                    self.bump();
                    break;
                }
                _ => self.bump(),
            };
        }
        comment_ty
    }

    // Numbers are treated as whole blocks.
    fn number(&mut self) -> TokenKind {
        let mut buf: Vec<char>;
        loop {
            match self.first() {
                None => break,
                Some(c) => buf = c.chars().collect(),
            }

            if buf.is_empty() {
                break;
            }

            match buf[0] {
                '0'..='9' => self.bump(),
                _ => break,
            };
        }
        Number
    }

    fn word(&mut self) -> TokenKind {
        let mut buf: Vec<char>;

        loop {
            match self.first() {
                None => break,
                Some(c) => buf = c.chars().collect(),
            };

            match buf.len() {
                0 => break,
                1 if buf[0].is_whitespace() => break,
                1 if buf[0].is_ascii() => break,
                _ => self.bump(),
            };
        }
        Word
    }
    fn ascii_word(&mut self) -> TokenKind {
        let mut buf: Vec<char>;

        loop {
            match self.first() {
                None => break,
                Some(c) => buf = c.chars().collect(),
            };

            match buf.len() {
                0 => break,
                1 if buf[0].is_whitespace() => break,
                1 if buf[0].is_ascii_alphabetic() => self.bump(),
                _ => break,
            };
        }
        AWord
    }
}
