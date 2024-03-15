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
    I::Item: Debug,
{
    let first = iter.next();
    let second = iter.next();
    DualPeekIterator { iter, first, second }
}

impl<I> DualPeekIterator<I>
where
    I: Iterator,
{
    pub fn base_iter(&self) -> &I {
        &self.iter
    }

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
        let first = self.first.take();
        self.first = self.second.take();
        self.second = self.iter.next();
        first
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
        Self::Item: Debug,
    {
        peek_two(self)
    }
}

impl<T: Iterator> Iterutils for T {}
