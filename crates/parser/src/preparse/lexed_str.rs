use std::fmt::Display;
use std::ops::Range;

use tracing::{instrument, trace};

use super::errors::PreparseError;
use crate::expansion::expanded::{Expanded, ExpandedIter};
use crate::preparse::converter::Converter;
use crate::SyntaxKind;
use crate::SyntaxKind::*;

pub enum SyntaxToken<'source> {
    Token(SyntaxKind),
    Expansion(Box<Expanded<'source>>),
}

/// Represents the result of pre-parsing LaTeX source code (parsing-stage 1). It holds a reference to the input string,
/// the pre-parsed syntax tokens and their corresponding byte start positions, resulting pre-parse errors,
/// as well as definitions in the pre-parsed code and their positions in bytes.
pub struct LexedStr<'source> {
    pub(super) src: &'source str,
    pub(super) tokens: Vec<SyntaxToken<'source>>,
    pub(super) start: Vec<usize>,
    pub(super) errors: Vec<PreparseError>,
    pub(super) definitions: Vec<Definition>,
}

impl<'source> LexedStr<'source> {
    pub fn new(input: &str) -> LexedStr {
        let buf = LexedStr {
            src: input,
            tokens: Vec::new(),
            start: Vec::new(),
            errors: Vec::new(),
            definitions: Vec::new(),
        };

        let tokens = lexer::tokenize(input);

        let conv = Converter::new(buf, tokens);

        trace!("Converting LexerToken to LexedStr");
        conv.transform()
    }

    pub fn definitions(&self) -> impl Iterator<Item = &Definition> + '_ {
        self.definitions.iter()
    }

    pub fn errors(&self) -> impl Iterator<Item = &PreparseError> + '_ {
        self.errors.iter()
    }

    // Returns the source string of a specified token range. This does ignore expanded tokens and
    // returns none if the range is not valid.
    pub fn text(&self, r: impl IndexRange<usize>) -> Option<&str> {
        let range = r.as_range();
        let len = self.tokens.len() - 1;
        if !(range.end <= len && range.start <= len) {
            return None;
        }

        let start = self.start[range.start];
        let end = self.start[range.end];
        Some(&self.src[start..end])
    }

    pub fn iter(&'source self) -> LexerIter<'source> {
        self.iter_from(0)
    }

    pub fn iter_from(&'source self, idx: usize) -> LexerIter<'source> {
        LexerIter::from_base(&self, idx, false)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum DefinitionKind {
    /// `\newcommand`.
    Macro,
    /// `\def`.
    Def,
    /// `\newenvironment`.
    Environment,
}

impl std::fmt::Display for DefinitionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionKind::Macro => write!(f, "Macro"),
            DefinitionKind::Def => write!(f, "Def"),
            DefinitionKind::Environment => write!(f, "Environment"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Definition {
    pub kind: DefinitionKind,
    /// Index in LexedStr of the definition in the pre-parsed source code tokens.
    pub idx: usize,
}

impl Definition {
    pub fn new(kind: DefinitionKind, idx: usize) -> Definition {
        Definition { kind, idx }
    }
}

impl std::fmt::Debug for LexedStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LexedStr")
            .field("errors", &self.errors)
            .field("definitions", &self.definitions)
            .finish()?;
        writeln!(f, "\nlexed TokenStream: ")?;
        for (idx, token) in self.tokens.iter().enumerate() {
            let SyntaxToken::Token(kind) = token else {
                continue;
            };

            match self.text(idx) {
                Some(_)
                    if matches!(
                        kind,
                        SyntaxKind::Break | SyntaxKind::Newline | Whitespace
                    ) =>
                {
                    writeln!(f, "{idx:?}. \" \" {kind:?}")?
                }
                Some(src) => writeln!(f, "{idx:?}. \"{src}\" {kind:?}")?,
                None if *kind == SyntaxKind::Eof => {
                    writeln!(f, "{idx:?}. \"\"  EOF")?
                }
                _ => writeln!(
                    f,
                    "!! Token {:?} at index {} not found",
                    kind, idx
                )?,
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LexerIter<'source> {
    index: Option<usize>,
    base: &'source LexedStr<'source>,
    inner: Option<ExpandedIter<'source>>,
    include_expansion: bool,
}

impl<'source> LexerIter<'source> {
    fn from_base(
        base: &'source LexedStr,
        index: usize,
        include_expansion: bool,
    ) -> LexerIter<'source> {
        LexerIter { base, index: Some(index), inner: None, include_expansion }
    }

    /// Panics if called after the iterator is exhausted.
    pub(crate) fn index(&self) -> usize {
        self.index.expect("Called index() on exhausted iterator")
    }

    pub(crate) fn text(
        &self,
        index: impl IndexRange<usize>,
    ) -> Option<&'source str> {
        let idx = index.as_range();
        let text = self.base.text(index);
        println!("getting text: {:?} with index: {idx:?}", text);
        text
    }

    pub(crate) fn current_text(&self) -> Option<&'source str> {
        self.text(self.index? + 1)
    }

    pub(crate) fn advance_while(
        &mut self,
        mut predicate: impl FnMut(SyntaxKind) -> bool,
    ) {
        while let Some(token) = self.peek() {
            if !predicate(token) {
                return;
            }
            self.next();
        }
    }

    pub(crate) fn peek(&self) -> Option<SyntaxKind> {
        self.peek_nth(1)
    }

    pub(crate) fn peek_nth(&self, n: usize) -> Option<SyntaxKind> {
        if !self.include_expansion {
            let index = self.index? + n;
            let token = match self.base.tokens.get(index)? {
                SyntaxToken::Token(t) => *t,
                SyntaxToken::Expansion(e) => *e.original(),
            };
            return Some(token);
        }

        let mut index = self.index?;
        let end = self.index? + n;
        while index <= end {
            if let Some(e) = &self.inner {
                let l = e.len();
                if l + index <= end {
                    return e.peek_nth(n).copied();
                }
            }

            match self.base.tokens.get(index)? {
                SyntaxToken::Token(t) => {
                    if index == end {
                        return Some(*t);
                    } else {
                        index += 1;
                    }
                }
                SyntaxToken::Expansion(e) => {
                    let l = e.len();
                    if l + index <= end {
                        let idx = end - index;
                        return e.iter().peek_nth(idx).copied();
                    }
                    index += l;
                }
            }
        }
        None
    }
}

impl<'source> Iterator for LexerIter<'source> {
    type Item = &'source SyntaxKind;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.is_none() {
            return None;
        }

        if !self.include_expansion {
            let token = match self.base.tokens.get(self.index?)? {
                SyntaxToken::Token(Eof) => {
                    self.index = None;
                    return None;
                }
                SyntaxToken::Token(t) => t,
                SyntaxToken::Expansion(exp) => exp.original(),
            };
            self.index = self.index.map(|i| i + 1);
            return Some(token);
        }

        loop {
            let token = match self.inner {
                Some(ref mut t) => t.next(),
                None => None,
            };

            if token.is_some() {
                return token;
            }

            self.inner = None;
            match self.base.tokens.get(self.index?)? {
                SyntaxToken::Token(Eof) => {
                    self.index = None;
                    return None;
                }
                SyntaxToken::Token(t) => {
                    // increase index by one if index is some
                    self.index = self.index.map(|i| i + 1);
                    return Some(t);
                }
                SyntaxToken::Expansion(e) => {
                    self.inner = Some(e.iter());
                    self.index = self.index.map(|i| i + 1);
                }
            }
        }
    }
}

pub trait IndexRange<I>: Clone {
    fn as_range(&self) -> Range<I>;
}

impl IndexRange<usize> for usize {
    fn as_range(&self) -> Range<usize> {
        *self..(*self + 1)
    }
}
impl IndexRange<usize> for Range<usize> {
    fn as_range(&self) -> Range<usize> {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_index() {
        let src = "\\newcommand{ \\name }{\nasdfasdf}";
        let lexed = LexedStr::new(src);
        let mut iter = lexed.iter();

        assert_eq!(iter.index(), 0);
        iter.next();
        assert_eq!(iter.index(), 1);
        iter.peek();
        assert_eq!(iter.index(), 1)
    }
    #[test]
    fn test_peek_nth() {
        let src = "\\newcommand{ \\name }{\nasdfasdf}";
        let lexed = LexedStr::new(src);
        let iter = LexerIter::from_base(&lexed, 0, false);

        assert_eq!(iter.peek_nth(0), Some(SyntaxKind::NewCommand));
        assert_eq!(iter.peek_nth(1), Some(SyntaxKind::OpenBrace));
        assert_eq!(iter.peek_nth(2), Some(SyntaxKind::Whitespace));
        assert_eq!(iter.peek_nth(3), Some(SyntaxKind::Command));
        assert_eq!(iter.peek_nth(4), Some(SyntaxKind::Whitespace));
        assert_eq!(iter.peek_nth(5), Some(SyntaxKind::CloseBrace));
        assert_eq!(iter.peek_nth(6), Some(SyntaxKind::OpenBrace));
        assert_eq!(iter.peek_nth(7), Some(SyntaxKind::Newline));
        assert_eq!(iter.peek_nth(8), Some(SyntaxKind::AWord));
        assert_eq!(iter.peek_nth(9), Some(SyntaxKind::CloseBrace));
        assert_eq!(iter.peek_nth(10), Some(SyntaxKind::Eof));
        assert_eq!(iter.peek_nth(11), None);
    }
}
