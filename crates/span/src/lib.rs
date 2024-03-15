use std::ops::Range;

#[derive(Debug)]
pub struct Spanned<T, File: SpanFile> {
    pub value: T,
    pub span: Span<File>,
}

#[derive(Debug)]
pub struct Span<File: SpanFile> {
    start: usize,
    len: usize,
    file: File,
}

impl<File: SpanFile> Span<File> {
    pub fn new(file: File, start: usize, len: usize) -> Span<File> {
        Span { file, start, len }
    }

    pub fn span(&self) -> Range<usize> {
        self.start..(self.start + self.len)
    }
    pub fn len(&self) -> usize {
        self.len
    }
}

pub trait SpanFile {}

#[derive(Debug)]
pub struct CurrentFile {}

impl CurrentFile {
    pub fn new() -> CurrentFile {
        CurrentFile {}
    }
}

impl SpanFile for CurrentFile {}
