use lexer::{
    Token,
    TokenKind,
};

use super::converter::Converter;
use super::errors::PreparseErrorKind::*;
use crate::syntax::Trivia;
use crate::SyntaxKind;
use crate::SyntaxKind::*;

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

impl<I> Converter<'_, I>
where
    I: Iterator<Item = Token>,
{
    pub(super) fn convert(&mut self) {
        while !self.is_eof() {
            self.token();
        }
        self.add_token(Eof)
    }

    fn token(&mut self) {
        match self.advance() {
            TokenKind::CommandIdent => self.command(),
            TokenKind::Newline => self.newline(),
            TokenKind::Less if self.at(TokenKind::Equal) => {
                self.advance_with_token(LessEq)
            }
            TokenKind::Greater if self.at(TokenKind::Equal) => {
                self.advance_with_token(GreaterEq)
            }
            TokenKind::Equal if self.at(TokenKind::Equal) => {
                self.advance_with_token(Comparison)
            }
            TokenKind::Minus if self.at(TokenKind::Greater) => {
                self.advance_with_token(RightArrow)
            }
            TokenKind::Dollar => self.add_token(MathDelimiter),
            TokenKind::Number => self.number(),
            TokenKind::Dot => self.dot(),
            TokenKind::DoubleApostrophe => self.string(),
            TokenKind::NumSign => self.raw(),
            token => {
                let kind = self.basic_token_kind(token);
                self.add_token(kind);
            }
        }
    }

    fn command(&mut self) {
        let mut valid_multichar = false;

        match self.peek_first() {
            TokenKind::AWord | TokenKind::Underscore => {
                valid_multichar = true;
            }
            TokenKind::At => return self.variable(),
            TokenKind::Colon if self.at_second(TokenKind::Colon) => {
                return self.path_begin();
            }

            c if c.is_trivia() => self.advance_with_error(CommandNameMissing),
            TokenKind::Eof => self.advance_with_error(CommandNameMissing),
            TokenKind::Word => self.advance_with_error(InvalidCommandName),

            _ => {}
        };

        self.advance();

        // only continue if command started with multi-char syntax
        if !valid_multichar || self.is_eof() {
            self.add_command();
        } else {
            self.command_inner(false);
        }
    }

    fn path_begin(&mut self) {
        debug_assert!(self.at(TokenKind::Colon));
        debug_assert!(self.at_second(TokenKind::Colon));

        self.advance_by(2);
        self.add_token(PathSeperator);

        self.command_inner(true)
    }

    fn command_inner(&mut self, mut after_path_spec: bool) {
        while !self.is_eof() {
            match self.peek_first() {
                TokenKind::AWord | TokenKind::Number => {}
                TokenKind::Underscore if self.is_valid_in_cmd() => {}
                TokenKind::Colon if after_path_spec => {
                    return self.advance_with_error(InvalidCommandName);
                }
                TokenKind::Colon if self.peek_second() == TokenKind::Colon => {
                    self.add_token(Namespace);
                    self.advance_by(2);
                    self.add_token(PathSeperator);
                    return self.command_inner(true);
                }
                TokenKind::At => return self.variable(),
                TokenKind::Star => {
                    self.advance();
                    return self.add_command();
                }
                _ => break,
            }

            after_path_spec = false;
            self.advance();
        }

        self.add_command()
    }

    fn is_valid_in_cmd(&self) -> bool {
        matches!(
            self.peek_second(),
            TokenKind::AWord
                | TokenKind::Number
                | TokenKind::Underscore
                | TokenKind::Colon
        )
    }

    fn variable(&mut self) {
        debug_assert!(self.at(TokenKind::At));
        self.advance();

        match self.advance() {
            TokenKind::AWord => {}
            TokenKind::Underscore if self.is_valid_in_cmd() => {}
            _ => self.add_error(InvalidVariableName),
        }

        while !self.is_eof() {
            match self.peek_first() {
                TokenKind::AWord | TokenKind::Number => {}
                TokenKind::Underscore if self.is_valid_in_cmd() => {}
                _ => break,
            }

            self.advance();
        }

        self.add_token(Variable);
    }

    fn newline(&mut self) {
        if !self.at(TokenKind::Newline) {
            return self.add_token(Newline);
        }

        while !self.is_eof() {
            if !self.at(TokenKind::Newline) {
                if !(self.at(TokenKind::Whitespace)
                    && self.at_sec(TokenKind::Newline))
                {
                    break;
                }
            }
            self.advance();
        }

        self.add_token(Break)
    }

    fn number(&mut self) {
        if self.at(TokenKind::Dot) && self.at_sec(TokenKind::Number) {
            self.advance_by(2);
            self.add_token(Float);
        } else {
            self.add_token(Number)
        }
    }

    fn dot(&mut self) {
        if self.at(TokenKind::Number) {
            self.advance();
            self.add_token(Float)
        } else {
            self.add_token(Dot)
        }
    }

    fn string(&mut self) {
        while !self.is_eof() {
            if self.at(TokenKind::DoubleApostrophe) {
                self.advance();
                break;
            }
            self.advance();
        }
        self.add_token(String);
    }

    fn raw(&mut self) {
        if self.at(TokenKind::Greater) && self.at_sec(TokenKind::Greater) {
            self.advance_by(2);
            self.add_token(RawDelimiter);
        } else if self.at(TokenKind::AWord) || self.at(TokenKind::Number) {
            self.macro_expansion_token()
        } else {
            self.add_token(NumSign)
        }
    }

    fn macro_expansion_token(&mut self) {
        if self.eat(TokenKind::Number) {
            self.add_token(SimpleMacroExpansionToken);
            return;
        }

        while !self.is_eof() {
            match self.peek_first() {
                TokenKind::AWord | TokenKind::Number => {}
                TokenKind::Underscore if self.is_valid_in_cmd() => {}
                _ => break,
            }
            self.advance();
        }
        self.add_token(ComplexMacroExpansionToken)
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
            SingleApostrophe,
            DoubleApostrophe,
            Slash,
            Tilde,
            Comma,
            Semicolon,
            Ampersand,
            Pipe,
            Colon,
            Plus,
            Minus,
            Question,
            At,
            Equal,
            Less,
            Greater,
            Bang,
            Eof
        )
    }
}
