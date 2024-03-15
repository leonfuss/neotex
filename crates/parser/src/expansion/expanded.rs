use std::ops::Range;

use crate::SyntaxKind;

#[derive(Debug)]
pub(crate) struct Expanded<'source> {
    name: &'source str,
    original: &'source SyntaxKind,
    skip_n: usize,
    source: TokenSource,
    caller: TokenSource,
    args: ExpansionArgs,
    tokens: Vec<SyntaxKind>,
}

impl<'source> Expanded<'source> {
    pub fn iter(&'source self) -> ExpandedIter<'source> {
        ExpandedIter::from_base(self, 0)
    }

    pub fn original(&self) -> &SyntaxKind {
        self.original
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}

#[derive(Debug)]
pub(crate) struct ExpandedIter<'source> {
    index: Option<usize>,
    base: &'source Expanded<'source>,
}

impl<'source> ExpandedIter<'source> {
    pub fn from_base(base: &'source Expanded<'source>, index: usize) -> ExpandedIter<'source> {
        ExpandedIter { index: Some(index), base }
    }

    pub fn original(&self) -> &SyntaxKind {
        self.base.original
    }

    pub fn skip_n(&self) -> usize {
        self.base.skip_n
    }

    pub fn len(&self) -> usize {
        self.base.tokens.len()
    }

    pub fn peek_nth(&self, n: usize) -> Option<&'source SyntaxKind> {
        let index = self.index? + n;
        self.base.tokens.get(index)
    }
}

impl<'source> Iterator for ExpandedIter<'source> {
    type Item = &'source SyntaxKind;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(index) = self.index else { return None };

        let item = self.base.tokens.get(index);
        self.index = self.index.and_then(|i| Some(i + 1));
        item
    }
}

#[derive(Debug)]
struct ExpansionArgs {}

#[derive(Debug)]
pub(super) struct TokenSource {
    file_id: usize,
    range: Range<usize>,
}
