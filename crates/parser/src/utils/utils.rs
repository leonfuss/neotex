use std::ops::{Deref, Range};

pub(crate) struct Lock();
impl Lock {
    // unsafe to restrict ussage
    pub(crate) unsafe fn new() -> Lock {
        Lock()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Marker(usize);
impl Marker {
    pub fn new(idx: usize, _lock: Lock) -> Marker {
        Marker(idx)
    }
}

impl Deref for Marker {
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.0
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
