pub use crate::cursor::Cursor;

use self::TokenKind::*;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, len: usize) -> Token {
        Token { kind, len }
    }
}

/// Enum representing common lexical types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    CommandIdent,

    // Whitespace
    Whitespace, // ' '
    Newline,
    Comment,  // %....\n
    AComment, // %% ... \n - Annotated Comment

    Number, // (0-9)+
    Word,
    AWord, // ASCII Word

    // Braces
    OpenBrace,    // '{'
    CloseBrace,   // '}'
    OpenBracket,  // '['
    CloseBracket, // ']'
    OpenParen,    // '('
    CloseParen,   // ')'

    // Special Symbols
    Star,       // '*'
    NumSign,    // '#'
    Carret,     // '^'
    Less,       // '<'
    Greater,    // '>'
    Underscore, // '_'
    Apostrophe, // "'"
    Slash,      // '/'
    Tilde,      // '~'
    Comma,      // ','
    Semicolon,  // ';'
    Ampersand,  // '&'
    Equal,      // '='
    Pipe,       // '|'
    Colon,      // ':'
    Dollar,     // '$'
    Minus,      // '-'
    Plus,       // '+'
    Dot,        // '.'
    At,         // '@'

    Question, // '?'
    Bang,     // '!'

    Unknown, // only if grapheme segmentation failed

    // End of Input
    Eof,
}

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
pub fn is_whitespace(c: char) -> bool {
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

pub fn is_command_name(c: char) -> bool {
    c == '@' || c.is_ascii_alphabetic()
}

/// Checks newline character for a singel char. This does
/// of course not check for '\r\n'
pub fn is_newline(c: char) -> bool {
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

pub fn is_multi_char_newline(first: char, second: char) -> bool {
    matches!((first, second), ('\u{000D}', '\u{000A}'))
}

impl Cursor<'_> {
    pub fn advance_token(&mut self) -> Token {
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
