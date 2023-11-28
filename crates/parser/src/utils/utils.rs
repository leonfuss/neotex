use std::ops::Range;

pub trait IndexedSliceView<'source> {
    fn as_str(&self) -> &'source str;

    fn start_of_index(&self, idx: usize) -> usize;

    fn len(&self) -> usize;

    fn range_from_index(&self, idx: usize) -> Range<usize> {
        let low = self.start_of_index(idx);
        let high = self.start_of_index(idx + 1);
        low..high
    }

    fn slice_at_index(&self, idx: usize) -> &'source str {
        let range = self.range_from_index(idx);
        &self.as_str()[range]
    }
}

pub fn compare_advance<C, X, Y, P>(
    base: &mut impl Iterator<Item = X>,
    compare_window: C,
    comperator: P,
) -> bool
where
    C: IntoIterator<Item = Y>,
    P: Fn(&X, &Y) -> bool,
    Y: PartialEq,
{
    let mut c = compare_window.into_iter();

    let mut cx = c.next();

    if cx == None {
        return true;
    }
    let mut bx = base.next();

    loop {
        match (bx, cx) {
            (Some(x), Some(y)) if comperator(&x, &y) => {}
            (Some(_), Some(_)) => return false,
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
            (None, None) => return true,
        }

        cx = c.next();
        bx = base.next();
    }
}
