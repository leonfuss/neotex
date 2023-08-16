pub use crate::cursor::Cursor;

use self::TokenKind::*;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    fn new(kind: TokenKind, len: u32) -> Token {
        Token { kind, len }
    }
}

/// Enum representing common lexical types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Whitespace
    Whitespace, // ' '
    Newline,

    // Braces
    OpenBrace,    // '{'
    CloseBrace,   // '}'
    OpenBracket,  // '['
    CloseBracket, // ']'
    OpenParen,    // '('
    CloseParen,   // ')'

    // CommandIdentifier
    BackSlash, // '\'

    // Special Symbols
    Star,       // '*'
    NumSign,    // '#'
    Carret,     // '^'
    Underscore, // '_'
    Lt,         // '<'
    Gt,         // '>'
    Apostrophe, // "'"
    Slash,      // '/'
    Tilde,      // '~'
    Percent,    // '%'
    Comma,      // ','
    Semicolon,  // ';'
    Ampersand,  // '&'
    Eq,         // '='
    Pipe,       // '|'
    Colon,      // ':'
    Dollar,     // '$'
    At,         // '@'
    Minus,      // '-'
    Plus,       // '+'

    // Math / Sentence End
    Dot, // '.'

    // Sentence End
    Question, // '?'
    Bang,     // '!'

    // Alpha-Numerical
    Number,    // '0-9'
    AsciiWord, // '([A-Z][a-z])*' seperated by all other charactersn
    Word,      // rest inculding all not "tex-command"-valid characters

    // Unknow value
    Unknown,

    // End of Input
    Eof,
}

impl TokenKind {
    pub fn is_empty_char(&self) -> bool {
        *self == Whitespace || *self == Newline
    }

    pub fn is_valid_macro_name(&self) -> bool {
        !(self.is_empty_char() || *self == Word || *self == Eof)
    }
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
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{0020}' // space

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK
    )
}

/// Checks
pub fn is_newline(first: char, second: char) -> bool {
    let is_single_char_newline_character = matches!(
        first,
        '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        // NEXT LINE from latin1
        | '\u{0085}'
        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    );

    return is_single_char_newline_character | is_multi_char_newline(first, second);
}

pub fn is_multi_char_newline(first: char, second: char) -> bool {
    match (first, second) {
        ('\u{000D}', '\u{000A}') => true,
        _ => false,
    }
}

impl Cursor<'_> {
    pub fn advance_token(&mut self) -> Token {
        let first_char = match self.bump() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, 0),
        };

        let token_kind = match first_char {
            c if is_whitespace(c) => self.whitespace(),
            c if is_newline(c, self.first()) => self.newline(c),

            '0'..='9' => Number,

            '\u{005C}' => BackSlash,

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
            '@' => At,
            '#' => NumSign,
            '~' => Tilde,
            '$' => Dollar,
            '?' => Question,
            '!' => Bang,
            '&' => Ampersand,
            '=' => Eq,
            '<' => Lt,
            '>' => Gt,
            '-' => Minus,
            '+' => Plus,
            '|' => Pipe,
            '^' => Carret,
            '%' => Percent,
            '\'' => Apostrophe,
            '_' => Underscore,
            '/' => Slash,

            c if c.is_ascii_alphabetic() => self.ascii_word(),
            c if c.is_alphabetic() => self.word(),
            _ => Unknown,
        };

        let res = Token::new(token_kind, self.pos_within_token());
        self.reset_pos_within_token();
        res
    }

    fn whitespace(&mut self) -> TokenKind {
        self.eat_while(is_whitespace);
        Whitespace
    }

    fn newline(&mut self, current: char) -> TokenKind {
        if is_multi_char_newline(current, self.first()) {
            self.bump();
        }
        loop {
            match (self.first(), self.second()) {
                (f, s) if is_multi_char_newline(f, s) => {
                    self.bump();
                    self.bump()
                }
                (f, s) if is_newline(f, s) => self.bump(),
                (_, _) => break,
            };
        }
        Newline
    }

    // Numbers are treated as whole blocks. It remains to be seen if this is
    // the most efficient choice. It depends on how often the numberblocks must be
    // split up again. - eg. when used as macro options
    // fn number(&mut self) -> TokenKind {
    //     loop {
    //         match self.first() {
    //             '0'..='9' => self.bump(),
    //             _ => break,
    //         };
    //     }
    //     Number
    // }

    // We treat ascii-words differently to allow easy building of marcros.
    // If an ascii-word is captured after a backslash, the ascii-word is the
    // macro name. The input is invalid if its is any other multichar characters.
    fn ascii_word(&mut self) -> TokenKind {
        loop {
            match self.first() {
                c if c.is_ascii_alphabetic() => self.bump(),
                _ => break,
            };
        }
        AsciiWord
    }

    fn word(&mut self) -> TokenKind {
        loop {
            match self.first() {
                c if c.is_ascii_alphabetic() => break,
                c if c.is_alphabetic() => self.bump(),
                _ => break,
            };
        }
        Word
    }
}
