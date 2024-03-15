use std::ops::Deref;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct OpenMark(usize);

impl OpenMark {
    pub fn new(idx: usize) -> OpenMark {
        OpenMark(idx)
    }
}

impl Deref for OpenMark {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct CloseMark(usize);

impl CloseMark {
    pub fn new(idx: usize) -> CloseMark {
        CloseMark(idx)
    }
}

impl Deref for CloseMark {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.0
    }
}
