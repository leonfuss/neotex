use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use itertools::Itertools;

use super::{
    errors::{ResolverError, ResolverErrorKind, ResolverResult},
    resexp_parser::OptionalKind,
    resexp_parser::ResExpParser,
    resolving::{def, new_command, new_environment},
    store::{ExpansionStore, ExpansionStoreItem},
};
use crate::preparse::IndexRange;
use crate::{
    preparse::{DefinitionKind, LexedStr, LexerIter},
    utils::utils::Marker,
    SyntaxKind::*,
};

macro_rules! open_block {
    ( $name:ident, $open: ident) => {
        /// returned bool indicates weather a skip ahead is required.
        /// If optional is OptionalBlock and no opening brace is found, this functions returns true. If
        /// optional is OptionalBlockIdent and no opening brace is found, this function returns true.
        /// Else the output is false
        pub(super) fn $name(&mut self, optional: OptionalKind) -> Result<bool, ResolverErrorKind> {
            self.skip_trivia();

            if optional == OptionalKind::OptionalBlock && !self.at($open) {
                return Ok(true);
            }

            let opt = optional == OptionalKind::OptionalBlockIdent;

            self.expect($open, opt)?;
            Ok(opt)
        }
    };
}

macro_rules! close_block {
    ($name: ident, $close: ident) => {
        pub(super) fn $name(&mut self, skip: bool) -> ResolverResult {
            self.skip_trivia();
            self.expect($close, skip)?;
            Ok(())
        }
    };
}

pub(crate) fn resolve<'source>(
    lexed: &'source LexedStr,
) -> (ExpansionStore<'source>, Vec<ResolverError>) {
    lexed
        .definitions()
        .map(|def| Resolver::new(lexed.iter_from(def.idx), def.kind).finish())
        .partition_result()
}

pub(crate) struct Resolver<'s> {
    item: RefCell<ExpansionStoreItem<'s>>,
    parser: ResExpParser<'s>,
    option_name: Option<std::ops::Range<usize>>,
}

impl<'s> Resolver<'s> {
    fn new(iter: LexerIter, kind: DefinitionKind) -> Resolver {
        Resolver {
            item: ExpansionStoreItem::empty(kind).into(),
            parser: ResExpParser::new(iter),
            option_name: None,
        }
    }
    fn finish(mut self) -> Result<ExpansionStoreItem<'s>, ResolverError> {
        self.resolve()?;
        Ok(self.item.into_inner())
    }
    fn resolve(&mut self) -> Result<(), ResolverError> {
        let kind = self.item.borrow().kind;
        let res = match kind {
            DefinitionKind::Macro => new_command(self),
            DefinitionKind::Environment => new_environment(self),
            DefinitionKind::Def => def(self),
        };
        // Add the index where the error occurred for basic error reporting.
        res.map_err(|e| e.attach_marker(self.mark()))
    }

    pub(super) fn text(&self, range: impl IndexRange<usize>) -> Result<&'s str, ResolverErrorKind> {
        self.parser.text(range)
    }

    pub(super) fn register_cmd_name(&mut self) -> ResolverResult {
        self.item.borrow_mut().name = self.text(self.index())?;
        Ok(())
    }

    pub(super) fn register_env_name(&mut self, mark: Marker) -> ResolverResult {
        let range = self.range_from_mark(mark);
        self.item.borrow_mut().name = self.text(range)?;
        Ok(())
    }

    pub(super) fn register_empty_arg_count(&mut self) {
        self.item.borrow_mut().args.arg_count = 0;
    }

    pub(super) fn register_arg_count(&mut self) -> ResolverResult {
        assert!(self.at(Number));

        let num: u16 = self.parse(self.index(), false)?;
        self.item.borrow_mut().args.arg_count = num;
        Ok(())
    }

    pub(super) fn register_opt_arg(&mut self, mark: Marker) -> ResolverResult {
        let range = self.range_from_mark(mark);

        // insert argument default value token range into the optional arguments table.
        // Note: the argument name is a simple index, as this is a not named optional argument.
        self.item.borrow_mut().args.optional.push(range);
        Ok(())
    }

    pub(super) fn mark_opt_named_arg_name(&mut self, mark: Marker) {
        let range = self.range_from_mark(mark);
        // Store the index of the current token as the name marker. The marker will be used
        // to retrieve the source text of the named optional argument name later.
        self.option_name = Some(range);
    }

    // can only be called after argument name is marked with mark_opt_named_arg_name
    pub(super) fn register_opt_named_arg(&mut self, mark: Marker) -> ResolverResult {
        assert!(self.option_name.is_some());

        let range = self.range_from_mark(mark);
        // Retrieve the keys source text by index.
        // The key should be stored before by a call to advance_with_optional_arg_name.
        let key = {
            let idx = self.option_name.take().unwrap();
            self.text(idx)?
        };

        // insert argument name and default value token range into the optional named arguments table
        self.item.borrow_mut().args.optional_named.insert(key, range);
        Ok(())
    }

    pub(super) fn register_smpl_macro_token(&mut self) -> ResolverResult {
        let idx = self.index();
        let key: u16 = self.parse(idx, true)?;

        let table = &mut self.item.borrow_mut().req_expansion;
        table.entry(key).and_modify(|it| it.push(idx)).or_insert(vec![idx]);

        Ok(())
    }

    pub(super) fn register_cmplx_macro_token(&mut self) -> ResolverResult {
        let idx = self.index();
        let key = &self.text(idx)?[1..];

        let table = &mut self.item.borrow_mut().opt_expansion;
        table.entry(key).and_modify(|it| it.push(idx)).or_insert(vec![idx]);

        Ok(())
    }

    pub(super) fn register_body(&mut self, marker: Marker) {
        let range = self.range_from_mark(marker);
        self.item.borrow_mut().body = range;
    }

    pub(super) fn register_second_body(&mut self, marker: Marker) {
        let range = self.range_from_mark(marker);
        self.item.borrow_mut().second_body = Some(range);
    }

    open_block!(open_brace, OpenBrace);
    close_block!(close_brace, CloseBrace);
    open_block!(open_bracket, OpenBracket);
    close_block!(close_bracket, CloseBracket);
}

impl<'s> Deref for Resolver<'s> {
    type Target = ResExpParser<'s>;
    fn deref(&self) -> &ResExpParser<'s> {
        &self.parser
    }
}

impl<'s> DerefMut for Resolver<'s> {
    fn deref_mut(&mut self) -> &mut ResExpParser<'s> {
        &mut self.parser
    }
}
