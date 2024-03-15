use rustc_hash::{
    FxHashMap,
    FxHashSet,
};

use crate::preparse::LexedStr;

pub(crate) struct PreprocessStore<'source> {
    macro_store: FxHashMap<&'source str, PreprocessedMacroArg>,
    env_store: FxHashMap<&'source str, PreprocessedEnvironmentArg>,
}

pub(crate) enum PreprocessedMacroArg {
    /// mandatory argument count
    Basic(u8),
    /// for pattern matching with def
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PreprocessedEnvironmentArg(u8);

impl<'source> PreprocessStore<'source> {
    pub fn new(lexed: &'source LexedStr) -> PreprocessStore<'source> {
        let store = PreprocessStore {
            macro_store: FxHashMap::default(),
            env_store: FxHashMap::default(),
        };

        // let processor = Preprocessor::new(lexed, store);
        // processor.process()
        todo!()
    }
}
