use std::cell::{Cell, RefCell};
use std::fmt;
use std::{mem::MaybeUninit, ptr};

/// A ringbuffer with a capacity `N` fixed at compile time.
struct RingBuffer<T, const N: usize> {
    data: MaybeUninit<[T; N]>,
    start: Cell<usize>,
    end: Cell<usize>,
    is_full: Cell<bool>,
}

impl<T, const N: usize> RingBuffer<T, N> {
    const CAPACITY: usize = N;
    pub const fn new() -> RingBuffer<T, N> {
        RingBuffer {
            data: MaybeUninit::uninit(),
            start: Cell::new(0),
            end: Cell::new(0),
            is_full: Cell::new(false),
        }
    }

    /// Returns the capacity of the buffer. This is the maximum number of elements the buffer
    /// can hold and is fixed at compile time.
    pub fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end && !self.is_full.get()
    }

    pub fn is_full(&self) -> bool {
        self.is_full.get()
    }

    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        if self.is_full.get() {
            Self::CAPACITY
        } else {
            if self.end >= self.start {
                self.end.get() - self.start.get()
            } else {
                Self::CAPACITY - self.start.get() + self.end.get()
            }
        }
    }

    fn wrap_add(&self, index: usize, num: usize) -> usize {
        (index + num) % Self::CAPACITY
    }

    fn as_ptr(&self) -> *mut T {
        self.data.as_ptr() as *mut T
    }

    unsafe fn buf_write(&self, index: usize, item: T) {
        ptr::write(self.as_ptr().add(index), item);
    }

    unsafe fn buf_get(&self, index: usize) -> T {
        ptr::read(self.as_ptr().add(index))
    }

    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len() {
            let index = self.wrap_add(self.start.get(), index);
            Some(unsafe { self.buf_get(index) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            let index = self.wrap_add(self.start.get(), index);
            Some(unsafe { &mut *self.as_ptr().add(index) })
        } else {
            None
        }
    }

    /// append an item to the end of the buffer
    pub fn enqueue(&self, item: T) -> Result<(), T> {
        if self.is_full.get() {
            return Err(item);
        }

        if self.len() == self.capacity() - 1 {
            self.is_full.set(true);
        }

        let index = self.wrap_add(self.end.get(), 1);
        self.end.set(index);

        // safety: no shared references to the items in the buffer are held outside of the buffer.
        // Only copies of the items are returned, expect for `enqueue` and `dequeue`. Which transfers
        // ownership of the items.

        unsafe { self.buf_write(index, item) }
        Ok(())
    }

    /// pop an item from the start of the buffer
    /// Returns `None` if the buffer is empty.
    pub fn dequeue(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let index = self.start.get();
        let start = self.wrap_add(self.start.get(), 1);
        self.start.set(start);

        if self.is_full.get() {
            self.is_full.set(false);
        }

        // safety: no shared references to the items in the buffer are held outside of the buffer.
        // Only copies of the items are returned, expect for `enqueue` and `dequeue`. Which transfers
        // ownership of the items.

        unsafe { Some(self.buf_get(index)) }
    }
}

/// An iterator that is bufferd by a ring buffer of size N
/// This is useful for when you want to iterate over a sequence of items and need to look ahead.
pub struct RingBufferedIterator<I: Iterator, const N: usize> {
    iter: RefCell<I>,
    buffer: RingBuffer<I::Item, N>,
}

impl<I: Iterator, const N: usize> RingBufferedIterator<I, N> {
    pub fn new(iter: I) -> RingBufferedIterator<I, N> {
        RingBufferedIterator { iter: RefCell::new(iter), buffer: RingBuffer::new() }
    }

    pub fn peek(&self) -> Option<I::Item> {
        self.peek_nth(0)
    }

    pub fn peek_nth(&self, n: usize) -> Option<I::Item> {
        if n >= self.buffer.capacity() {
            return None;
        }
        if n < self.buffer.len() {
            self.buffer.get(n)
        } else {
            while let Some(item) = self.iter.borrow_mut().next() {
                self.buffer.enqueue(item).map_err(|_| ()).expect("buffer overflow");
                if n < self.buffer.len() {
                    return self.buffer.get(n);
                }
            }
            None
        }
    }
}

impl<I: Iterator, const N: usize> Iterator for RingBufferedIterator<I, N> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buffer.dequeue() {
            return Some(item);
        }
        self.iter.borrow_mut().next()
    }
}

impl<I, const N: usize> fmt::Debug for RingBufferedIterator<I, N>
where
    I: fmt::Debug + Iterator,
    I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RingBufferedIterator").field("iter", &self.iter).finish()
    }
}

trait ToRingBufferedIterator: Iterator {
    fn buffered<const N: usize>(self) -> RingBufferedIterator<Self, N>
    where
        Self: Sized,
    {
        RingBufferedIterator::new(self)
    }
}

impl<I: Iterator> ToRingBufferedIterator for I {}
