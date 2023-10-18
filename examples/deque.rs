

use core::ops::Index;
use core::iter::ExactSizeIterator;

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub struct Deque<const SIZE: usize> {
    data: [u8; SIZE],
    head: AtomicUsize,
    tail: AtomicUsize,
}


pub struct Writer<const SIZE: usize> {
    buffer: *mut Deque<SIZE>
}

impl<const SIZE: usize> Deque<SIZE> {
    pub const fn new() -> Self {
        Self{data: [0u8; SIZE],
            head:  AtomicUsize::new(0),
            tail:  AtomicUsize::new(0)
        }
    }
    pub fn push(&mut self, line: u8) {
        if !self.is_full() {
            let head = self.head.load(Relaxed);
            self.data[head] = line;
            self.head.store(if head < SIZE-1 {head+1} else {head+1-SIZE}, Relaxed)
        }
    }
    pub fn pop(&mut self) -> u8 {
        if self.len() > 0 {
            let old_tail = self.tail.fetch_update(Relaxed, Relaxed, |x| Some(if x < SIZE-1 {x+1} else {x+1-SIZE})).unwrap();
            unsafe { self.data[old_tail] }
        } else {
            panic!("Popping from an empty buffer")
        }
    }
    pub fn space(&self) -> usize { SIZE - self.len() }
    pub fn load(&mut self, data: &[u8]) {
        for d in data {
            self.push(*d);
        }
    }
    pub fn is_full(&self) -> bool { self.len() >= SIZE-1 }
    pub fn is_empty(&self) -> bool { self.head.load(Relaxed) == self.tail.load(Relaxed) }
    pub fn iter(&self) -> DequeIterator<SIZE> { DequeIterator{deque: self, pos: 0} }
    pub fn clear(&mut self) {
        self.head.store(0, Relaxed);
        self.tail.store(0, Relaxed);
    }
    pub fn mk_writer(&mut self) -> Writer<SIZE>{
        Writer{buffer: &mut *self}
    }
}

impl<const SIZE: usize> Index<usize> for Deque<SIZE> {
    type Output = u8;
    fn index(&self, i: usize) -> &u8 {
        if i >= self.len() {
            panic!("Out of bounds");
        }
        // The index is counted from the tail, so [0] returns the oldest value.
        let mut offset = self.tail.load(Relaxed) + i;
        if offset > SIZE {
            offset -= SIZE;
        }
        unsafe { &self.data[offset] }
    }
}

impl<const SIZE: usize> Iterator for Deque<SIZE> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.len() > 0 {
            Some(self.pop())
        } else {
            None
        }
    }
}

impl<const SIZE: usize> ExactSizeIterator for Deque<SIZE> {
    fn len(&self) -> usize {
        /// Return the number of spaced in the buffer that are occupied.
        /// UNRELIABLE! The actual number can change while this function is executing.
        let head = self.head.load(Relaxed);
        let tail = self.tail.load(Relaxed);
        if head >= tail {
            head - tail
        } else {
            SIZE - tail + head
        }
    }
}


pub struct DequeIterator<'a, const SIZE: usize> {
    deque: &'a Deque<SIZE>,
    pos: usize
}

impl<'a, const SIZE: usize> Iterator for DequeIterator<'a, SIZE> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.deque.len() {
            let value = self.deque[self.pos];
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }
}



impl<const SIZE: usize> Writer<SIZE> {
    pub fn push(&mut self, ch: u8) {
        unsafe { (*self.buffer).push(ch) }
    }
}

unsafe impl<const SIZE: usize> Send for Deque<SIZE> {}
unsafe impl<const SIZE: usize> Sync for Deque<SIZE> {}

unsafe impl<const SIZE: usize> Send for Writer<SIZE> {}
unsafe impl<const SIZE: usize> Sync for Writer<SIZE> {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        /// Assert that a semi-filled deque keeps the same length all over
        let mut d = Deque::<11>::new();
        for i in 0u8..5u8 {
            d.push(i);
        }
        assert_eq!(d.len(), 5);

        for i in 5u8..100u8 {
            d.push(i);
            assert_eq!(d.pop(), i-5);
            assert_eq!(d.len(), 5);
        }
    }

    #[test]
    fn test_is_empty() {
        let mut d = Deque::<10>::new();
        for i in 0u8..100u8 {
            assert_eq!(d.is_empty(), true);
            assert!(!d.is_full());
            assert_eq!(d.len(), 0);
            d.push(i);
            assert_eq!(d.is_empty(), false);
            assert!(!d.is_full());
            assert_eq!(d.len(), 1);
            d.pop();
        }
    }

    #[test]
    fn test_is_full() {
        let mut d = Deque::<11>::new();
        for i in 0u8..10u8 {
            assert_eq!(d.len(), i as usize);
            d.push(i);
        }

        for i in 0u8..100u8 {
            assert_eq!(d.len(), 10);
            assert!(d.is_full());
            assert_eq!(d.is_empty(), false);
            d.pop();
            assert_eq!(d.len(), 9);
            assert!(!d.is_full());
            assert_eq!(d.is_empty(), false);
            d.push(i);
        }
    }

    #[test]
    fn test_fill_and_empty() {
        let mut d = Deque::<11>::new();
        for _j in 0..100 {
            for i in 0u8..=9u8 {
                assert_eq!(d.len(), i as usize);
                d.push(i);
            }
            assert!(d.is_full());
            for i in 0u8..=9u8 {
                assert_eq!(d.pop(), i);
            }
            assert_eq!(d.len(), 0);
        }
    }

    #[test]
    fn test_load() {
        let mut d = Deque::<11>::new();
        d.load(&[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]);
        assert_eq!(d.len(), 10);
        assert!(d.is_full());
        for i in 1..=10 {
            assert_eq!(d.pop(), i);
        }
    }

    #[test]
    fn test_random_access() {
        let mut d = Deque::<11>::new();
        d.load(&[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]);
        assert_eq!(d[0], 1u8);
        assert_eq!(d[1], 2u8);
        assert_eq!(d[9], 10u8);
        d.pop();
        assert_eq!(d[0], 2u8);
        assert_eq!(d[8], 10u8);
        d.push(11u8);
        assert_eq!(d[0], 2u8);
        assert_eq!(d[9], 11u8);
    }

    #[test]
    fn test_iterator() {
        let mut d = Deque::<11>::new();
        d.load(&[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]);
        for (i, v) in d.iter().enumerate() {
            assert_eq!((i+1) as u8, v);
        }
    }

    #[test]
    fn test_writer() {
        let mut d = Deque::<11>::new();
        let mut w = d.mk_writer();
        w.push(4);
        assert_eq!(d.is_empty(), false);
        assert_eq!(d.pop(), 4);
    }
}
