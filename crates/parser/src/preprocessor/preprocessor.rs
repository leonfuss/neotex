// use crate::preparse::Definition;
// use crate::preparse::DefinitionKind;
// use crate::preparse::LexedStr;
// use crate::preprocessor::errors::PreprocessError;
// use crate::preprocessor::errors::PreprocessErrorKind::*;
// use crate::preprocessor::store::PreprocessStore;
// use crate::utils;
// use crate::SyntaxKind;
// use crate::SyntaxKind::*;
//
// use super::errors::PreprocessErrorKind;
//
// pub(crate) struct Preprocessor<'source> {
//     lexed: &'source LexedStr<'source>,
//     store: PreprocessStore<'source>,
//     iter: Option<Box<dyn Iterator<Item = (usize, SyntaxKind)>>>,
//     offset: usize,
//     current: Option<(usize, SyntaxKind)>,
//     peek: Option<(usize, SyntaxKind)>,
//     errors: Vec<PreprocessError>,
// }
//
// impl<'source> Preprocessor<'source> {
//     pub fn new(
//         lexed: &'source LexedStr<'source>,
//         store: PreprocessStore<'source>,
//     ) -> Preprocessor<'source> {
//         Preprocessor {
//             lexed,
//             store,
//             errors: Vec::new(),
//             iter: None,
//             current: None,
//             peek: None,
//             offset: 0,
//         }
//     }
//
//     pub fn process(mut self) -> PreprocessStore<'source> {
//         for def in self.lexed.definitions() {
//             let mut iter = self
//                 .lexed
//                 .syntax_tokens_from_index(def.idx)
//                 .filter(|(_, token)| !token.is_preprocess_trivia());
//
//             let option = match def.kind {
//                 DefinitionKind::Def => todo!(),
//                 _ => self.option_extraction(iter),
//             };
//         }
//         self.store
//     }
//
//     fn option_extraction(&self, mut iter: impl Iterator<Item = (usize, SyntaxKind)>) {
//         let Some(_) = iter.next() else {
//             return;
//         };
//
//         let expected = [OpenBrace, Command, CloseBrace];
//
//         // if !utils::compare_advance(&mut iter, expected, |&(_, x), &y| x == y) {
//         //     return;
//         // }
//         // we assume now that we have a valid syntax in the form of \newcommand{\name}....
//         // now we check if options are present
//         match iter.next() {
//             Some((_, SyntaxKind::OpenBracket)) => todo!(),
//             _ => todo!(),
//         }
//     }
//
//     fn add_env_args(&self, count: u8) {}
//
//     fn environment(&mut self, def: &Definition) {}
//     fn r#macro(&mut self, def: &Definition) {}
//     fn definition(&mut self, def: &Definition) {}
//
//     fn begin_group(&mut self) -> Result<(), ()> {
//         match self.peek() {
//             Some(OpenBrace) => {
//                 self.bump();
//                 Ok(())
//             }
//             token => {
//                 self.add_error(MissingGroupBegin, OpenBrace, token);
//                 Err(())
//             }
//         }
//     }
//
//     fn iter_from_idx(&mut self, idx: usize) {
//         let iter = self.lexed.syntax_tokens_from_index(idx);
//         self.iter = Some(Box::new(iter));
//     }
//
//     fn bump(&mut self) -> Option<SyntaxKind> {
//         let Some(ref mut iter) = self.iter else {
//             return None;
//         };
//
//         self.current = self.peek;
//
//         match iter.next() {
//             None => self.peek = None,
//             Some(item) => self.peek = Some(item),
//         };
//         self.offset += 1;
//
//         self.current.map(|it| it.1)
//     }
//
//     fn peek(&self) -> Option<SyntaxKind> {
//         if self.iter.is_some() {
//             self.current.map(|it| it.1)
//         } else {
//             None
//         }
//     }
//
//     fn add_error(
//         &mut self,
//         kind: PreprocessErrorKind,
//         expected: SyntaxKind,
//         found: Option<SyntaxKind>,
//     ) {
//         assert!(self.current.is_some());
//
//         let idx = self.current.unwrap().0;
//         let err = PreprocessError::new(kind, idx, expected, found);
//
//         self.errors.push(err);
//     }
// }
