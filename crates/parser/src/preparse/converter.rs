use std::cell::Cell;
use std::ops::Range;

use lexer::Token;
use lexer::TokenKind;

use super::errors::PreparseError;
use super::errors::PreparseErrorKind;
use super::lexed_str::DefinitionKind;
use super::lexed_str::LexedStr;
use super::lexed_str::SyntaxToken;
use crate::utils::peek::DualPeekIterator;
use crate::utils::peek::Iterutils;
use crate::SyntaxKind;
use crate::SyntaxKind::*;

pub(super) struct Converter<'source, I>
where
    I: Iterator,
    I::Item: std::fmt::Debug,
{
    iter: DualPeekIterator<I>,
    pub lexed: LexedStr<'source>,
    position: Position,
    fuel: Cell<u32>,
}

#[derive(Debug, Default)]
struct Position {
    position: usize,
    offset: usize,
}

// infrastructure
impl<'source, I> Converter<'source, I>
where
    I: Iterator<Item = Token>,
{
    pub(crate) fn new(
        lexed: LexedStr<'source>,
        tokens: I,
    ) -> Converter<'source, I> {
        Converter {
            lexed,
            iter: tokens.peek_two(),
            position: Position::default(),
            fuel: 256.into(),
        }
    }

    pub(crate) fn transform(mut self) -> LexedStr<'source> {
        self.convert();
        self.lexed
    }

    pub(super) fn is_eof(&self) -> bool {
        self.peek_first() == TokenKind::Eof
    }

    pub(super) fn advance(&mut self) -> TokenKind {
        self.fuel.set(256);
        let (kind, len) = self
            .iter
            .next()
            .map_or((TokenKind::Eof, 0), |it| (it.kind, it.len));
        self.position.advance_by(len);
        kind
    }

    pub(super) fn advance_by(&mut self, step: usize) -> TokenKind {
        debug_assert!(0 < step);
        let mut last = TokenKind::Eof;
        for _ in 0..step {
            last = self.advance()
        }
        last
    }

    pub(super) fn peek_first(&self) -> TokenKind {
        self.fuel.set(self.fuel.get() - 1);
        self.iter.peek_first().map_or(TokenKind::Eof, |it| it.kind)
    }

    pub(super) fn peek_second(&self) -> TokenKind {
        self.fuel.set(self.fuel.get() - 1);
        self.iter.peek_second().map_or(TokenKind::Eof, |it| it.kind)
    }

    pub(super) fn at(&self, kind: TokenKind) -> bool {
        self.peek_first() == kind
    }

    pub(super) fn at_second(&self, kind: TokenKind) -> bool {
        self.peek_second() == kind
    }

    pub(super) fn at_sec(&self, kind: TokenKind) -> bool {
        self.peek_second() == kind
    }

    pub(super) fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(super) fn advance_with_token(&mut self, token: SyntaxKind) {
        self.advance();
        self.add_token(token);
    }

    pub(super) fn advance_with_error(&mut self, err: PreparseErrorKind) {
        self.advance();
        self.add_error(err);
    }

    pub(super) fn add_error(&mut self, err: PreparseErrorKind) {
        let start = self.position.reset_to_start();
        self.lexed.add_error(err, start);
    }

    pub(super) fn add_token(&mut self, token: SyntaxKind) {
        let start = self.position.reset_to_start();
        self.lexed.add_token(token, start);
    }

    pub(super) fn add_command(&mut self) {
        let range = self.position.current_range();
        let slice = &self.lexed.src[range];

        let kind = match slice {
            "\\def" => return self.add_definition(Def, DefinitionKind::Def),
            "\\newcommand" => {
                return self.add_definition(NewCommand, DefinitionKind::Macro);
            }
            "\\newenvironment" => {
                return self
                    .add_definition(NewEnv, DefinitionKind::Environment);
            }
            "\\input" => Input,
            "\\usepackage" => UsePackage,
            "\\use" => Use,
            "\\fn" => FunctionIdent,
            "\\pub" => Pub,
            "\\let" => Let,
            _ => Command,
        };

        self.add_token(kind);
    }

    fn add_definition(&mut self, token: SyntaxKind, def: DefinitionKind) {
        let start = self.position.reset_to_start();
        self.lexed.add_definition(token, def, start)
    }
}

impl LexedStr<'_> {
    fn add_token(&mut self, token: SyntaxKind, start: usize) {
        self.tokens.push(SyntaxToken::Token(token));
        self.start.push(start);
    }

    fn add_error(&mut self, err_kind: PreparseErrorKind, start: usize) {
        self.add_token(Error, start);
        let err = PreparseError::new(err_kind, self.last_idx());
        self.errors.push(err)
    }

    fn add_definition(
        &mut self,
        token: SyntaxKind,
        def_kind: DefinitionKind,
        start: usize,
    ) {
        self.add_token(token, start);
        let def = super::lexed_str::Definition::new(def_kind, self.last_idx());
        self.definitions.push(def);
    }

    fn last_idx(&self) -> usize {
        self.tokens.len() - 1
    }
}

impl Position {
    fn advance_by(&mut self, steps: usize) {
        self.position += steps;
        self.offset += steps;
    }

    fn reset_to_start(&mut self) -> usize {
        let ret = self.position - self.offset;
        self.offset = 0;
        ret
    }

    fn current_range(&self) -> Range<usize> {
        let start = self.position - self.offset;
        start..self.position
    }
}
