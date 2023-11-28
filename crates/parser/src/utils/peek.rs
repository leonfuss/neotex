use std::fmt::Debug;

pub struct DualPeekIterator<I>
where
    I: Iterator,
{
    iter: I,
    first: Option<I::Item>,
    second: Option<I::Item>,
}

pub fn peek_two<I>(mut iter: I) -> DualPeekIterator<I>
where
    I: Iterator,
{
    let first = iter.next();
    let second = iter.next();
    DualPeekIterator {
        iter,
        first,
        second,
    }
}

impl<I> DualPeekIterator<I>
where
    I: Iterator,
{
    pub fn peek_first(&self) -> Option<&I::Item> {
        self.first.as_ref()
    }

    pub fn peek_second(&self) -> Option<&I::Item> {
        self.second.as_ref()
    }
}

impl<I> Iterator for DualPeekIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = std::mem::take(&mut self.first);
        self.first = std::mem::take(&mut self.second);
        self.second = self.iter.next();
        ret
    }
}

impl<I> Debug for DualPeekIterator<I>
where
    I: Iterator,
    I::Item: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DualPeekIterator")
            .field("first", &self.first)
            .field("second", &self.second)
            .finish()
    }
}

pub trait Iterutils: Iterator {
    fn peek_two(self) -> DualPeekIterator<Self>
    where
        Self: Sized,
    {
        peek_two(self)
    }
}

impl<T: Iterator> Iterutils for T {}
