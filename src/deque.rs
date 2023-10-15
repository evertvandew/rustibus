

use core::cell::UnsafeCell;
use core::ops::Index;
use core::iter::ExactSizeIterator;

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub struct Deque<T: Copy + Default, const SIZE: usize> {
    data: UnsafeCell<[T; SIZE]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T: Copy + Default, const SIZE: usize> Deque<T, SIZE> {
    pub fn new() -> Self {
        Self{data: UnsafeCell::new([T::default(); SIZE]),
            head:AtomicUsize::new(0),
            tail:AtomicUsize::new(0)
        }
    }
    pub fn push(&mut self, line: T) {
        if !self.is_full() {
            let head = self.head.load(Relaxed);
            unsafe { (*self.data.get())[head] = line; }
            self.head.store(if head < SIZE-1 {head+1} else {head+1-SIZE}, Relaxed)
        }
    }
    pub fn pop(&mut self) -> T {
        if self.len() > 0 {
            let old_tail = self.tail.fetch_update(Relaxed, Relaxed, |x| Some(if x < SIZE-1 {x+1} else {x+1-SIZE})).unwrap();
            unsafe { (*self.data.get())[old_tail] }
        } else {
            panic!("Popping from an empty buffer")
        }
    }
    pub fn space(&self) -> usize { SIZE - self.len() }
    pub fn load(&mut self, data: &[T]) {
        for d in data {
            self.push(*d);
        }
    }
    pub fn is_full(&self) -> bool { self.len() >= SIZE-1 }
    pub fn is_empty(&self) -> bool { self.head.load(Relaxed) == self.tail.load(Relaxed) }
    pub fn iter(&self) -> DequeIterator<T, SIZE> { DequeIterator{deque: self, pos: 0} }
    pub fn clear(&mut self) {
        self.head.store(0, Relaxed);
        self.tail.store(0, Relaxed);
    }
}

impl<T: Copy + Default, const SIZE: usize> Index<usize> for Deque<T, SIZE> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        if i >= self.len() {
            panic!("Out of bounds");
        }
        // The index is counted from the tail, so [0] returns the oldest value.
        let mut offset = self.tail.load(Relaxed) + i;
        if offset > SIZE {
            offset -= SIZE;
        }
        unsafe { &(*self.data.get())[offset] }
    }
}

impl<T: Copy + Default, const SIZE: usize> Iterator for Deque<T, SIZE> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.len() > 0 {
            Some(self.pop())
        } else {
            None
        }
    }
}

impl<T: Copy + Default, const SIZE: usize> ExactSizeIterator for Deque<T, SIZE> {
    fn len(&self) -> usize {
        /// Return the number of spaced in the buffer that are occupied.
        /// UNRELIABLE! The actual number can change while this function is executing.
        let head = self.head.load(Relaxed);
        let tail = self.tail.load(Relaxed);
        if head > tail {
            head - tail
        } else {
            SIZE - tail + head
        }
    }
}


pub struct DequeIterator<'a, T: Copy + Default, const SIZE: usize> {
    deque: &'a Deque<T, SIZE>,
    pos: usize
}

impl<'a, T: Copy + Default, const SIZE: usize> Iterator for DequeIterator<'a, T, SIZE> {
    type Item = T;
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty() {
        let mut d = Deque::<u8, 10>::new();
        for i in 0u8..100u8 {
            assert_eq!(d.len(), 0);
            assert!(!d.is_full());
            assert_eq!(d.len(), 0);
            d.push(i);
            assert_eq!(d.len(), 0);
            assert!(!d.is_full());
            assert_eq!(d.len(), 1);
            d.pop();
        }
    }

    #[test]
    fn test_is_full() {
        let mut d = Deque::<u8, 10>::new();
        for i in 0u8..=10u8 {
            assert_eq!(d.len(), i as usize);
            d.push(i);
        }

        for i in 0u8..100u8 {
            assert_eq!(d.len(), 10);
            assert!(d.is_full());
            assert_eq!(d.len(), 0);
            d.pop();
            assert_eq!(d.len(), 9);
            assert!(!d.is_full());
            assert_eq!(d.len(), 0);
            d.push(i);
        }
    }

    #[test]
    fn test_fill_and_empty() {
        let mut d = Deque::<u8, 10>::new();
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
        let mut d = Deque::<u8, 10>::new();
        d.load(&[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]);
        assert_eq!(d.len(), 10);
        assert!(d.is_full());
        for i in 1..=10 {
            assert_eq!(d.pop(), i);
        }
    }

    #[test]
    fn test_random_access() {
        let mut d = Deque::<u8, 11>::new();
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
        let mut d = Deque::<u8, 11>::new();
        d.load(&[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]);
        for (i, v) in d.iter().enumerate() {
            assert_eq!((i+1) as u8, v);
        }
    }
}
