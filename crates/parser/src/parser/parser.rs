use collections::RingBufferedIterator;
use std::cell::Cell;

use crate::{
    lexer::{LexToken, TokenizerItem},
    parser::infra::{CloseMark, OpenMark},
    SyntaxKind,
};

use super::token_set::TokenSet;

#[derive(Debug, PartialEq, Eq, Clone)]
enum ParserState {
    Markdown,
    Math,
    Code,
    Command,
}

#[derive(Debug)]
pub(super) enum ParserEvent {
    Open { kind: SyntaxKind, open_before: Option<usize> },
    Close { len: u32 },
    Advance,
}

const MAX_LOOKAHEAD: usize = 8;
const MAX_FUEL: usize = 256;

struct Parser<'source, I>
where
    I: Iterator<Item = TokenizerItem<'source>> + 'source,
{
    iter: RingBufferedIterator<I, MAX_LOOKAHEAD>,
    events: Vec<ParserEvent>,
    state: ParserState,
    token_len: u32,
    fuel: Cell<usize>,
    _phantom: std::marker::PhantomData<&'source I>,
}

impl<'source, I> Parser<'source, I>
where
    I: Iterator<Item = TokenizerItem<'source>>,
{
    pub fn new(tokens: I, state: ParserState) -> Parser<'source, I> {
        Parser {
            state,
            iter: RingBufferedIterator::new(tokens),
            events: Vec::new(),
            fuel: Cell::new(MAX_FUEL),
            token_len: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn finish(self) -> Vec<ParserEvent> {
        self.events
    }

    fn open(&mut self) -> OpenMark {
        let idx = self.events.len();
        self.events.push(ParserEvent::Open { kind: SyntaxKind::Error, open_before: None });
        OpenMark::new(idx)
    }

    fn close(&mut self, marker: OpenMark, kind: SyntaxKind) -> CloseMark {
        self.events[*marker] = ParserEvent::Open { kind, open_before: None };
        self.events.push(ParserEvent::Close { len: self.token_len });
        self.token_len = 0;
        CloseMark::new(*marker)
    }

    fn open_before(&mut self, mark: CloseMark) -> OpenMark {
        let len = self.events.len();
        if let Some(token) = self.events.get_mut(*mark) {
            if let ParserEvent::Open { ref mut open_before, .. } = token {
                *open_before = Some(len);
            }
        }
        self.events.push(ParserEvent::Open { kind: SyntaxKind::Error, open_before: None });
        OpenMark::new(*mark)
    }

    fn advance(&mut self) {
        assert!(!self.eof());
        self.events.push(ParserEvent::Advance);
        self.fuel.set(256);
        if let Some(token) = self.iter.next() {
            self.token_len += token.span.len() as u32;
        }
    }

    fn nth(&self, n: usize) -> LexToken {
        if self.fuel.get() == 0 {
            panic!("Parser out of fuel")
        }
        self.fuel.set(self.fuel.get() - 1);
        self.iter.peek_nth(n).map_or(LexToken::Eof, |it| it.value)
    }

    fn at(&self, token: LexToken) -> bool {
        self.nth(0) == token
    }

    fn at_any(&self, set: TokenSet) -> bool {
        set.contains(self.nth(0))
    }

    fn eof(&mut self) -> bool {
        self.iter.peek().is_none()
    }
}
