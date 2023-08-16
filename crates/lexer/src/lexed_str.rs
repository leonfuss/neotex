use crate::{
    syntax::SyntaxKind::{self, *},
    token::tokenize,
    token::TokenKind,
};

#[derive(Debug)]
pub struct LexedStr<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<u32>,
    error: Vec<LexError>,
}

#[derive(Debug)]
struct LexError {
    ty: LexErrorKind,
    token: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    UnknownChar,
    InvalidCommandName,
    MissingCommandName,
}

impl<'a> LexedStr<'a> {
    pub fn new(text: &'a str) -> LexedStr<'a> {
        let mut conv = Converter::new(text);

        for token in tokenize(&text[conv.offset..]) {
            match (&conv.active_syntax, token.kind) {
                (MultiTokenSyntax::None, TokenKind::BackSlash) => {
                    conv.begin_macro();
                    continue;
                }
                (MultiTokenSyntax::None, TokenKind::Percent) => {
                    conv.begin_comment();
                    continue;
                }
                (MultiTokenSyntax::Comment(o), TokenKind::Newline) => {
                    let offset = token.len + o;
                    let token_text = &text[conv.offset..][..offset as usize];
                    conv.push(COMMENT, token_text.len(), None);
                    conv.end_multi_token_syntax();
                }
                (MultiTokenSyntax::Comment(_), _) => {
                    conv.extend_multi_token(token.len);
                    continue;
                }
                (MultiTokenSyntax::Command(o), t) if t.is_valid_macro_name() => {
                    let offset = token.len + o;
                    let token_text = &text[conv.offset..][..offset as usize];
                    conv.push(COMMAND, token_text.len(), None);
                    conv.end_multi_token_syntax();
                }
                (MultiTokenSyntax::Command(o), t) if t.is_empty_char() => {
                    let token_text = &text[conv.offset..][..*o as usize];
                    conv.push(
                        UNKNOWN,
                        token_text.len(),
                        Some(LexErrorKind::MissingCommandName),
                    );
                    conv.end_multi_token_syntax();

                    let token_text = &text[conv.offset..][..token.len as usize];
                    conv.extend_token(token.kind, token_text);
                }
                (MultiTokenSyntax::Command(o), _) => {
                    let offset = token.len + o;
                    let token_text = &text[conv.offset..][..offset as usize];
                    conv.push(
                        UNKNOWN,
                        token_text.len(),
                        Some(LexErrorKind::InvalidCommandName),
                    );
                    conv.end_multi_token_syntax();
                }
                (MultiTokenSyntax::None, _) => {
                    let token_text = &text[conv.offset..][..token.len as usize];
                    conv.extend_token(token.kind, token_text);
                }
            };
        }

        match conv.active_syntax {
            MultiTokenSyntax::Comment(o) => {
                let token_text = &text[conv.offset..][..o as usize];
                conv.push(COMMENT, token_text.len(), None);
            }
            MultiTokenSyntax::Command(o) => {
                let token_text = &text[conv.offset..][..o as usize];
                conv.push(
                    UNKNOWN,
                    token_text.len(),
                    Some(LexErrorKind::MissingCommandName),
                );
            }
            _ => {}
        }

        conv.finalize_with_eof()
    }

    pub fn as_str(&self) -> &str {
        self.text
    }

    pub fn token(&self) -> impl Iterator<Item = &SyntaxKind> + '_ {
        self.kind.iter()
    }

    pub fn len(&self) -> usize {
        self.kind.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn errors(&self) -> impl Iterator<Item = (usize, &LexErrorKind)> + '_ {
        self.error.iter().map(|it| (it.token as usize, &it.ty))
    }

    fn push(&mut self, kind: SyntaxKind, offset: usize) {
        self.kind.push(kind);
        self.start.push(offset as u32);
    }
}

struct Converter<'a> {
    res: LexedStr<'a>,
    offset: usize,
    active_syntax: MultiTokenSyntax,
}

#[derive(PartialEq)]
enum MultiTokenSyntax {
    Comment(u32),
    Command(u32),
    None,
}

impl<'a> Converter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            res: LexedStr {
                text,
                kind: Vec::new(),
                start: Vec::new(),
                error: Vec::new(),
            },
            offset: 0,
            active_syntax: MultiTokenSyntax::None,
        }
    }

    fn begin_comment(&mut self) {
        // start with offset one for %
        self.active_syntax = MultiTokenSyntax::Comment(1);
    }

    fn begin_macro(&mut self) {
        // start with offset one for backslash
        self.active_syntax = MultiTokenSyntax::Command(1);
    }

    fn extend_multi_token(&mut self, len: u32) {
        self.active_syntax = match self.active_syntax {
            MultiTokenSyntax::Comment(o) => MultiTokenSyntax::Comment(o + len),
            MultiTokenSyntax::Command(o) => MultiTokenSyntax::Command(o + len),
            MultiTokenSyntax::None => MultiTokenSyntax::None,
        }
    }

    fn end_multi_token_syntax(&mut self) {
        self.active_syntax = MultiTokenSyntax::None;
    }

    fn finalize_with_eof(mut self) -> LexedStr<'a> {
        self.res.push(EOF, self.offset);
        self.res
    }

    fn push(&mut self, kind: SyntaxKind, len: usize, err: Option<LexErrorKind>) {
        self.res.push(kind, self.offset);
        self.offset += len;

        if let Some(err) = err {
            let token = self.res.len() as u32;
            self.res.error.push(LexError { ty: err, token });
        }
    }

    fn extend_token(&mut self, kind: TokenKind, token_text: &str) {
        let mut err = None;

        let syntax = match kind {
            TokenKind::Unknown => {
                err = Some(LexErrorKind::UnknownChar);
                UNKNOWN
            }
            TokenKind::Whitespace => WHITESPACE,
            TokenKind::Newline => NEWLINE,
            TokenKind::OpenBrace => OBRACE,
            TokenKind::CloseBrace => CBRACE,
            TokenKind::OpenBracket => OBRACKET,
            TokenKind::CloseBracket => CBRACKET,
            TokenKind::OpenParen => OPAREN,
            TokenKind::CloseParen => CPAREN,
            TokenKind::Star => STAR,
            TokenKind::NumSign => NUMSIGN,
            TokenKind::Carret => CARRET,
            TokenKind::Underscore => UNDERSCORE,
            TokenKind::Lt => LT,
            TokenKind::Gt => GT,
            TokenKind::Apostrophe => APOSTROPE,
            TokenKind::Slash => SLASH,
            TokenKind::Tilde => TILDE,
            TokenKind::Comma => COMMA,
            TokenKind::Semicolon => SEMICOLON,
            TokenKind::Ampersand => AMPERSAND,
            TokenKind::Eq => EQ,
            TokenKind::Pipe => PIPE,
            TokenKind::Colon => COLON,
            TokenKind::Dollar => DOLLAR,
            TokenKind::At => AT,
            TokenKind::Minus => MINUS,
            TokenKind::Plus => PLUS,
            TokenKind::Dot => DOT,
            TokenKind::Question => QUESTION,
            TokenKind::Bang => BANG,
            TokenKind::Number => NUMBER,
            TokenKind::AsciiWord => ASCIIWORD,
            TokenKind::Word => WORD,
            TokenKind::Eof => EOF,

            // Can only be Backslash or Percent. Both are handeled above
            _ => unreachable!(),
        };

        self.push(syntax, token_text.len(), err);
    }
}
