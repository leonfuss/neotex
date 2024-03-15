use std::fmt;

use span::{CurrentFile, Span, Spanned};
use tracing::trace;

pub(crate) trait LexerDelegate: fmt::Debug + Clone + Sized {
    type Token: fmt::Debug + Clone + Sized + Eq + Attachable + Tombstone;

    fn top() -> Self;

    fn next<'src>(&self, c: Option<char>, rest: &'src str) -> LexerNext<Self>;
}

#[derive(Debug)]
pub enum LexerNext<Delegate: LexerDelegate> {
    EOF,
    Remain(LexerAccumulate<Delegate>),
    Transition(LexerAccumulate<Delegate>, Delegate),
}

impl<Delegate: LexerDelegate> LexerNext<Delegate> {
    pub fn begin(state: Delegate) -> LexerNext<Delegate> {
        LexerNext::Transition(LexerAccumulate::Begin, state)
    }
}

#[derive(Debug)]
pub enum LexerAccumulate<Delegate: LexerDelegate> {
    /// Start a new token. There should be no accumulated characters
    /// yet.
    Begin,

    /// Don't emit anything, but continue to accumulate characters
    /// in the current token
    Continue(LexerAction),

    /// Start a new token after possibly consuming some characters.
    /// Those characters are ignored, and are not part of any token.
    Skip(LexerAction),

    /// Emit a token after accumulating some characters into it.
    Emit { before: Option<LexerAction>, token: Delegate::Token },
}

impl<Delegate: LexerDelegate> LexerAccumulate<Delegate> {
    pub fn and_transition(self, state: Delegate) -> LexerNext<Delegate> {
        LexerNext::Transition(self, state)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LexerAction {
    Consume(usize),
    Reconsume,
    Reset(usize),
}

impl LexerAction {
    pub fn and_remain<Delegate: LexerDelegate>(self) -> LexerNext<Delegate> {
        LexerNext::Remain(LexerAccumulate::Continue(self))
    }
    pub fn and_transition<Delegate: LexerDelegate>(self, state: Delegate) -> LexerNext<Delegate> {
        LexerNext::Transition(LexerAccumulate::Continue(self), state)
    }
    pub fn and_emit<Delegate: LexerDelegate>(
        self,
        token: Delegate::Token,
    ) -> LexerAccumulate<Delegate> {
        LexerAccumulate::Emit { before: Some(self), token }
    }
    pub fn and_discard<Delegate: LexerDelegate>(self) -> LexerAccumulate<Delegate> {
        LexerAccumulate::Skip(self)
    }
}

pub fn consume(c: char) -> LexerAction {
    LexerAction::Consume(c.len_utf8())
}

pub fn consume_str(s: &str) -> LexerAction {
    LexerAction::Consume(s.len())
}

pub fn reconsume() -> LexerAction {
    LexerAction::Reconsume
}

pub fn reset(len: usize) -> LexerAction {
    LexerAction::Reset(len)
}

pub(crate) struct Tokenizer<'table, Delegate: LexerDelegate> {
    state: Delegate,
    input: &'table str,
    start: usize,
    token_len: usize,
    exhausted: bool,
}

pub(super) type TokenizerItemDelegate<'table, Delegate: LexerDelegate> =
    Spanned<Delegate::Token, CurrentFile>;

impl<'table, Delegate: LexerDelegate + fmt::Debug> Iterator for Tokenizer<'table, Delegate> {
    type Item = TokenizerItemDelegate<'table, Delegate>;

    fn next(&mut self) -> Option<Self::Item> {
        const MAX_ITERATIONS: usize = 1000;

        for _ in 0..MAX_ITERATIONS {
            let Tokenizer { state, input, start, token_len, .. } = &self;

            let pos = start + token_len;
            let c = input[pos..].chars().next();
            let rest = &input[pos + c.map(|c| c.len_utf8()).unwrap_or(0)..];

            let next = state.next(c, rest);

            match self.step(next) {
                LoopCompletion::Return(item) => return self.emit(item),
                LoopCompletion::Continue => {}
            }
        }

        None
    }
}

enum LoopCompletion<T> {
    Continue,
    Return(T),
}

impl<'table, Delegate: LexerDelegate> Tokenizer<'table, Delegate> {
    pub fn new(input: &'table str) -> Self {
        Self { state: Delegate::top(), input, start: 0, token_len: 0, exhausted: false }
    }

    fn step(
        &mut self,
        next: LexerNext<Delegate>,
    ) -> LoopCompletion<Spanned<Delegate::Token, CurrentFile>> {
        match next {
            LexerNext::EOF => {
                trace!("EOF");
                LoopCompletion::Return(Delegate::Token::tombstone().attach_span(self.start, 0))
            }
            LexerNext::Remain(accumulate) => self.accumulate(accumulate),
            LexerNext::Transition(accumulate, state) => {
                let ret = self.accumulate(accumulate);
                self.transition(state);

                ret
            }
        }
    }

    fn accumulate(
        &mut self,
        accum: LexerAccumulate<Delegate>,
    ) -> LoopCompletion<Spanned<Delegate::Token, CurrentFile>> {
        use self::LexerAccumulate::*;

        match accum {
            Begin => {
                assert!(
                    self.token_len == 0,
                    "Cannot begin a new token when there are already accumulated characters"
                );

                LoopCompletion::Continue
            }
            Continue(action) => {
                self.action(action);
                LoopCompletion::Continue
            }
            Skip(action) => {
                self.action(action);

                self.start += self.token_len;
                self.token_len = 0;
                LoopCompletion::Continue
            }

            Emit { before, token, .. } => {
                if let Some(before) = before {
                    self.action(before);
                }

                let start = self.start;
                let len = self.token_len;
                self.start += len;
                self.token_len = 0;

                LoopCompletion::Return(token.attach_span(start, len))
            }
        }
    }

    fn action(&mut self, action: LexerAction) {
        match action {
            LexerAction::Consume(n) => {
                self.token_len += n;
            }
            LexerAction::Reconsume => {}
            LexerAction::Reset(n) => {
                self.token_len -= n;
            }
        }
    }

    fn transition(&mut self, state: Delegate) {
        trace!("transition {:?} -> {:?}", self.state, state);
        self.state = state;
    }

    fn emit(
        &mut self,
        token: Spanned<Delegate::Token, CurrentFile>,
    ) -> Option<Spanned<Delegate::Token, CurrentFile>> {
        if self.exhausted {
            return None;
        }

        if token.value == Delegate::Token::tombstone() {
            self.exhausted = true;
        }

        trace!("emit {:?}", token);
        Some(token)
    }
}

pub trait Attachable: Sized {
    fn attach_span(self, start: usize, len: usize) -> Spanned<Self, CurrentFile> {
        Spanned { value: self, span: Span::new(CurrentFile::new(), start, len) }
    }
}

pub trait Tombstone {
    fn tombstone() -> Self;
}

impl<T: Sized> Attachable for T {}
