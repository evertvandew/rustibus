
use core::ops::Index;

pub struct Deque<T: Copy + Default, const SIZE: usize> {
    data: [T; SIZE],
    head: usize,
    tail: usize,
}

impl<T: Copy + Default, const SIZE: usize> Deque<T, SIZE> {
    pub fn new() -> Self {
        Self{data: [T::default(); SIZE], head:0, tail:0}
    }
    pub fn push(&mut self, line: T) {
        if !self.is_full() {
            self.data[self.head] = line;
            if self.head < SIZE-1 {
                self.head += 1;
            } else {
                self.head = 0;
            }
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        let old_tail = self.tail;
        if self.len() > 0 {
            if self.tail < SIZE-1 {
                self.tail += 1;
            } else {
                self.tail = 0;
            }
            Some(self.data[old_tail])
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        if self.tail > self.head {
            SIZE - self.tail + self.head
        } else {
            self.head - self.tail
        }
    }
    pub fn load(&mut self, data: &[T]) {
        for d in data {
            self.push(*d);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }
    pub fn is_full(&self) -> bool {
        self.len() == SIZE-1
    }
}

impl<T: Copy + Default, const SIZE: usize> Index<usize> for Deque<T, SIZE> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        if i > self.len() {
            panic!("Out of bounds");
        }
        // The index is counted from the tail, so [0] returns the oldest value.
        let mut offset = self.tail + i;
        if offset > SIZE {
            offset -= SIZE;
        }
        &self.data[offset]
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty() {
        let mut d = Deque::<u8, 10>::new();
        for i in 0u8..100u8 {
            assert!(d.is_empty());
            assert!(!d.is_full());
            assert_eq!(d.len(), 0);
            d.push(i);
            assert!(!d.is_empty());
            assert!(!d.is_full());
            assert_eq!(d.len(), 1);
            _ = d.pop();
        }
    }

    #[test]
    fn test_is_full() {
        let mut d = Deque::<u8, 10>::new();
        for i in 0u8..10u8 {
            assert_eq!(d.len(), i as usize);
            d.push(i);
        }

        for i in 0u8..100u8 {
            assert_eq!(d.len(), 9);
            assert!(d.is_full());
            assert!(!d.is_empty());
            _ = d.pop();
            assert_eq!(d.len(), 8);
            assert!(!d.is_full());
            assert!(!d.is_empty());
            d.push(i);
        }
    }

    #[test]
    fn test_fill_and_empty() {
        let mut d = Deque::<u8, 10>::new();
        assert_eq!(d.pop(), None);
        for j in 0..100 {
            for i in 0u8..9u8 {
                assert_eq!(d.len(), i as usize);
                d.push(i);
            }
            assert!(d.is_full());
            for i in 0u8..9u8 {
                assert_eq!(d.pop(), Some(i));
            }
            assert!(d.is_empty());
        }
    }

    #[test]
    fn test_load() {
        let mut d = Deque::<u8, 11>::new();
        d.load(&[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]);
        assert_eq!(d.len(), 10);
        assert!(d.is_full());
        for i in 1..=10 {
            assert_eq!(d.pop(), Some(i));
        }
    }
}
