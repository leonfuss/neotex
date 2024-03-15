use std::str::FromStr;

use crate::{
    preparse::{IndexRange, LexerIter},
    syntax::Trivia,
    utils::utils::{Lock, Marker},
    SyntaxKind,
    SyntaxKind::*,
};

use super::errors::{ResolverErrorKind, ResolverResult};

/// Try to resolve the given definition, but abort immediately if an error occurs.
/// Error reporting is minimal, as comprehensive analysis is done one the AST is constructed.
pub(crate) struct ResExpParser<'source> {
    pub(super) iter: LexerIter<'source>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OptionalKind {
    Required,
    OptionalBlockIdent,
    OptionalBlock,
}

impl<'s> ResExpParser<'s> {
    pub(super) fn new(iter: LexerIter<'s>) -> ResExpParser<'s> {
        ResExpParser { iter }
    }

    pub(super) fn advance(&mut self) -> Result<SyntaxKind, ResolverErrorKind> {
        self.iter.next().ok_or(ResolverErrorKind::EofReached).copied()
    }

    pub(super) fn advance_to(&mut self, kind: SyntaxKind) -> ResolverResult {
        self.iter.advance_while(|token| token != kind);
        (self.peek() != Eof).then(|| ()).ok_or(ResolverErrorKind::EofReached)
    }

    pub(super) fn peek(&self) -> SyntaxKind {
        self.iter.peek().unwrap_or(Eof)
    }

    pub(super) fn peek_second(&self) -> SyntaxKind {
        self.iter.peek_nth(2).unwrap_or(Eof)
    }

    pub(super) fn peek_skip_trivia(&mut self) -> SyntaxKind {
        let mut n = 1;
        while self.iter.peek_nth(n).unwrap_or(Eof).is_trivia() {
            n += 1;
        }
        self.iter.peek_nth(n).unwrap_or(Eof)
    }

    // Compare the next token with the given one.
    pub(super) fn at(&self, kind: SyntaxKind) -> bool {
        self.peek() == kind
    }

    pub(super) fn expect(&mut self, kind: SyntaxKind, optional: bool) -> ResolverResult {
        let peek = self.peek();
        if self.peek() == kind {
            self.advance()?;
            Ok(())
        } else if optional {
            self.advance()?;
            Ok(())
        } else {
            Err(ResolverErrorKind::UnexpectedToken { expected: kind, found: peek })
        }
    }

    // Compare the second peek token with the given one.
    pub(super) fn at_peek(&self, kind: SyntaxKind) -> bool {
        self.peek_second() == kind
    }

    /// mark the peek position of the iterator
    pub(super) fn mark(&self) -> Marker {
        let idx = self.iter.index() + 1;
        self.base_mark(idx)
    }

    fn base_mark(&self, idx: usize) -> Marker {
        // safety: Lock is just unsafe to think before using it
        let lock = unsafe { Lock::new() };
        Marker::new(idx, lock)
    }

    pub(super) fn range_from_mark(&self, mark: Marker) -> std::ops::Range<usize> {
        *mark..(self.iter.index() + 1)
    }

    pub(super) fn index(&self) -> usize {
        // +1 because we only peek while deciding whether to advance or not
        self.iter.index() + 1
    }

    pub(super) fn skip_trivia(&mut self) {
        self.iter.advance_while(|token| token.is_trivia());
    }

    pub fn parse<T: FromStr>(
        &self,
        range: impl IndexRange<usize>,
        skip: bool,
    ) -> Result<T, ResolverErrorKind> {
        let mut text = self.text(range.clone())?;
        if skip {
            text = &text[1..];
        }
        text.parse::<T>()
            .map_err(|_| ResolverErrorKind::FailedToRetrieveTextRange(range.as_range()))
    }

    pub fn text(&self, range: impl IndexRange<usize>) -> Result<&'s str, ResolverErrorKind> {
        self.iter.text(range).ok_or(ResolverErrorKind::FailedToRetrieveText)
    }
}
