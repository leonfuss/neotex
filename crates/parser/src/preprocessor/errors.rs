use crate::SyntaxKind;

pub enum PreprocessErrorKind {
    MissingGroupBegin,
    MissingGroupEnd,
    MissingOptionGroupBegin,
    MissingOptionGroupEnd,
}

pub struct PreprocessError {
    pub kind: PreprocessErrorKind,
    pub expected: SyntaxKind,
    pub found: Option<SyntaxKind>,
    pub idx: usize,
}

impl PreprocessError {
    pub fn new(
        kind: PreprocessErrorKind,
        idx: usize,
        expected: SyntaxKind,
        found: Option<SyntaxKind>,
    ) -> PreprocessError {
        PreprocessError { kind, idx, expected, found }
    }
}
