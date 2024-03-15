use crate::SyntaxKind;

struct PreprocessIterator {}

impl Iterator for PreprocessIterator {
    type Item = SyntaxKind;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
