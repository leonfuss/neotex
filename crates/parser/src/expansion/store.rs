use std::ops::Range;

use rustc_hash::FxHashMap;

use crate::preparse::DefinitionKind;

#[derive(Debug, Default)]
pub(crate) struct ExpansionStore<'source> {
    def: Vec<ExpansionStoreItem<'source>>,
    mac: Vec<ExpansionStoreItem<'source>>,
    env: Vec<ExpansionStoreItem<'source>>,
    index: FxHashMap<&'source str, StoreIndex>,
}

#[derive(Debug)]
struct StoreIndex {
    idx: usize,
    kind: DefinitionKind,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpansionArgs<'source> {
    // stores arg count as given in definition. To get required args calc:
    // arg_count - optional.length
    pub(super) arg_count: u16,
    pub(super) optional: Vec<Range<usize>>,
    pub(super) optional_named: FxHashMap<&'source str, Range<usize>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExpansionStoreItem<'source> {
    pub kind: DefinitionKind,
    pub body: Range<usize>,
    // only for env
    pub second_body: Option<Range<usize>>,
    pub name: &'source str,
    pub args: ExpansionArgs<'source>,
    pub req_expansion: FxHashMap<u16, Vec<usize>>,
    pub opt_expansion: FxHashMap<&'source str, Vec<usize>>,
}

impl<'source> ExpansionStoreItem<'source> {
    pub(super) fn empty(kind: DefinitionKind) -> ExpansionStoreItem<'source> {
        // TODO: Replace with builder
        ExpansionStoreItem {
            body: 0..0,
            second_body: None,
            kind,
            name: "".into(),
            args: ExpansionArgs::default(),
            req_expansion: FxHashMap::default(),
            opt_expansion: FxHashMap::default(),
        }
    }
}

impl<'source> ExpansionStore<'source> {
    pub fn insert(&mut self, item: ExpansionStoreItem<'source>) {
        fn push<'s>(vec: &mut Vec<ExpansionStoreItem<'s>>, item: ExpansionStoreItem<'s>) -> usize {
            vec.push(item);
            vec.len() - 1
        }

        let name = item.name;
        let kind = item.kind;

        let idx = match kind {
            DefinitionKind::Macro => push(&mut self.mac, item),
            DefinitionKind::Def => push(&mut self.def, item),
            DefinitionKind::Environment => push(&mut self.env, item),
        };

        let index = StoreIndex::new(idx, kind);
        self.index.insert(&name, index);
    }

    pub fn get(&self, key: &str) -> Option<&'source ExpansionStoreItem> {
        let idx = self.index.get(key)?;
        match idx.kind {
            DefinitionKind::Macro => self.mac.get(idx.idx),
            DefinitionKind::Def => self.def.get(idx.idx),
            DefinitionKind::Environment => self.env.get(idx.idx),
        }
    }
}

// Collect an iterator of ExpansionStoreItems to one ExpansionStore
impl<'source> FromIterator<ExpansionStoreItem<'source>> for ExpansionStore<'source> {
    fn from_iter<T: IntoIterator<Item = ExpansionStoreItem<'source>>>(iter: T) -> Self {
        let mut store = ExpansionStore::default();
        for item in iter {
            store.insert(item);
        }
        store
    }
}

// Extend the store with an iterator of items
impl<'source> Extend<ExpansionStoreItem<'source>> for ExpansionStore<'source> {
    fn extend<T: IntoIterator<Item = ExpansionStoreItem<'source>>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item)
        }
    }
}

impl StoreIndex {
    pub fn new(idx: usize, kind: DefinitionKind) -> StoreIndex {
        StoreIndex { idx, kind }
    }
}
