// use std::cell::Cell;
//
// use lexer::{Token, TokenKind};
//
// use crate::SyntaxKind;
//
// #[derive(Debug)]
// enum Event {
//     Open { kind: SyntaxKind },
//     Close,
//     Advance,
// }
// struct MarkOpened {
//     index: usize,
// }
// #[derive(Debug)]
// pub struct Parser {
//     tokens: Vec<Token>,
//     pos: usize,
//     fuel: Cell<u32>,
//     events: Vec<Event>,
// }
//
// impl Parser {
//     pub fn new(tokens: Vec<Token>) -> Parser {
//         Parser {
//             tokens,
//             pos: 0,
//             fuel: 256.into(),
//             events: Vec::new(),
//         }
//     }
//     fn open(&mut self) -> MarkOpened {
//         let mark = MarkOpened {
//             index: self.events.len(),
//         };
//         self.events.push(Event::Open {
//             kind: SyntaxKind::Error,
//         });
//         mark
//     }
//     fn close(&mut self, m: MarkOpened, kind: SyntaxKind) {
//         self.events[m.index] = Event::Open { kind };
//         self.events.push(Event::Close);
//     }
//     fn advance(&mut self) {
//         assert!(!self.eof());
//         self.fuel.set(256);
//         self.events.push(Event::Advance);
//         self.pos += 1;
//     }
//
//     fn eof(&self) -> bool {
//         self.pos == self.tokens.len()
//     }
//
//     fn nth(&self, lookahead: usize) -> TokenKind {
//         if self.fuel.get() == 0 {
//             panic!("parser is stuck")
//         }
//         self.fuel.set(self.fuel.get() - 1);
//         self.tokens
//             .get(self.pos + lookahead)
//             .map_or(TokenKind::Eof, |it| it.kind)
//     }
//     fn at(&self, kind: TokenKind) -> bool {
//         self.nth(0) == kind
//     }
//     fn eat(&mut self, kind: TokenKind) -> bool {
//         if self.at(kind) {
//             self.advance();
//             true
//         } else {
//             false
//         }
//     }
//     fn expect(&mut self, kind: TokenKind) {
//         if self.eat(kind) {
//             return;
//         }
//         // TODO: Error reporting.
//         eprintln!("expected {kind:?}");
//     }
//     fn advance_with_error(&mut self, error: &str) {
//         let m = self.open();
//         // TODO: Error reporting.
//         eprintln!("{error}");
//         self.advance();
//         self.close(m, TreeKind::ErrorTree);
//     }
// }
