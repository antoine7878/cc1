use std::fmt::{self};
use std::hash::{Hash, Hasher};
use std::{iter::zip, ops::Index};

type C = u64;

#[derive(PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct BitSet {
    data: Vec<C>,
    capacity: usize,
}

impl Hash for BitSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
        self.capacity.hash(state);
    }
}

impl BitSet {
    const CHK_SIZE: usize = C::BITS as usize;
    pub fn with_capacity(capacity: usize) -> Self {
        let chunks = capacity.div_ceil(Self::CHK_SIZE).max(1);
        Self {
            data: vec![C::MIN; chunks],
            capacity,
        }
    }

    pub fn from<T: IntoIterator<Item = usize>>(capacity: usize, it: T) -> Self {
        let mut set = Self::with_capacity(capacity);
        set.extend(it);
        set
    }

    pub fn insert(&mut self, value: usize) {
        self.data[value / Self::CHK_SIZE] |= 1 << (value % Self::CHK_SIZE);
    }

    pub fn remove(&mut self, value: usize) {
        self.data[value / Self::CHK_SIZE] &= !(1 << (value % Self::CHK_SIZE));
    }

    pub fn extend<T: IntoIterator<Item = usize>>(&mut self, it: T) {
        for value in it {
            self.insert(value)
        }
    }

    pub fn ones(&self) -> OnesIter<'_> {
        OnesIter {
            data: &self.data,
            chunk_idx: 0,
            current: if self.data.is_empty() { 0 } else { self.data[0] },
            base: 0,
        }
    }

    pub fn is_clear(&self) -> bool {
        !self.data.iter().any(|&c| c != 0)
    }

    /// return 1 if index in contain in self
    pub fn get(&self, index: usize) -> bool {
        (self.data[index / Self::CHK_SIZE] >> (index % Self::CHK_SIZE)) & 1 == 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.capacity
    }

    pub fn union_with(&mut self, other: &Self) {
        zip(self.data.iter_mut(), other.data.iter()).for_each(|(a, b)| *a |= b);
    }

    pub fn union_changed(&mut self, other: &Self) -> bool {
        let mut changed = false;
        zip(self.data.iter_mut(), other.data.iter()).for_each(|(a, b)| {
            let merged = *a | b;
            changed |= merged != *a;
            *a = merged;
        });
        changed
    }

    pub fn toggle(&mut self) {
        self.data.iter_mut().for_each(|c| *c ^= C::MAX);
        let remainder = self.capacity % Self::CHK_SIZE;
        if remainder > 0
            && let Some(last) = self.data.last_mut()
        {
            *last &= (1 << remainder) - 1;
        }
    }

    pub fn count(&self) -> usize {
        self.data.iter().map(|c| c.count_ones() as usize).sum()
    }
}

impl Index<usize> for BitSet {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        if self.get(index) { &true } else { &false }
    }
}

impl fmt::Debug for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.ones().collect::<Vec<_>>())
    }
}

impl fmt::Display for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.ones().collect::<Vec<_>>())
    }
}

#[derive(Clone)]
pub struct OnesIter<'a> {
    data: &'a [C],
    chunk_idx: usize,
    current: C,
    base: usize,
}

impl<'a> Iterator for OnesIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        const CHK_SIZE: usize = C::BITS as usize;

        while self.current == 0 {
            self.chunk_idx += 1;
            if self.chunk_idx >= self.data.len() {
                return None;
            }
            self.current = self.data[self.chunk_idx];
            self.base = self.chunk_idx * CHK_SIZE;
        }

        let tz = self.current.trailing_zeros() as usize;
        self.current &= self.current - 1;
        Some(self.base + tz)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use std::collections::{BTreeSet};

    use super::*;

    #[test]
    fn new() {
        assert_eq!(BitSet::with_capacity(0), BitSet { data: vec![0], capacity: 0 });
        assert_eq!(BitSet::with_capacity(1), BitSet { data: vec![0], capacity: 1 });
        assert_eq!(BitSet::with_capacity(2), BitSet { data: vec![0], capacity: 2 });
        assert_eq!(BitSet::with_capacity(100), BitSet { data: vec![0, 0], capacity: 100 });
        assert_eq!(BitSet::with_capacity(64), BitSet { data: vec![0], capacity: 64 });
        assert_eq!(BitSet::with_capacity(65), BitSet { data: vec![0, 0], capacity: 65 });
        assert_eq!(BitSet::with_capacity(256), BitSet { data: vec![0, 0, 0, 0], capacity: 256 });
        assert_eq!(BitSet::with_capacity(257), BitSet { data: vec![0, 0, 0, 0, 0], capacity: 257 });
    }

    #[test]
    fn insert_contains() {
        let mut set = BitSet::with_capacity(100);
        assert!(set.is_clear());
        assert_eq!(set.count(), 0);
        set.insert(1);
        assert_eq!(set.count(), 1);
        assert!(!set.is_clear());
        assert_eq!(set, BitSet {data:vec![0b10,0], capacity:100});
        set.insert(2);
        assert_eq!(set.count(), 2);
        assert_eq!(set, BitSet {data:vec![0b110,0], capacity:100});
        set.insert(0);
        assert_eq!(set.count(), 3);
        assert_eq!(set, BitSet {data:vec![0b111,0], capacity:100});
        set.insert(99);
        assert_eq!(set.count(), 4);
        assert_eq!(set, BitSet {data:vec![0b111,0b100000000000000000000000000000000000], capacity:100});
        set.insert(64);
        assert_eq!(set, BitSet {data:vec![0b111,0b100000000000000000000000000000000001], capacity:100});
        set.insert(65);
        assert_eq!(set, BitSet {data:vec![0b111,0b100000000000000000000000000000000011], capacity:100});
        assert!(!set.is_clear());
        assert!(set[0]);
        assert!(set[1]);
        assert!(set[2]);
        assert!(set[99]);
        assert!(set[64]);
        assert!(set[65]);
        assert!(!set[3]);
        assert!(!set[4]);
        assert!(!set[23]);
        assert!(!set.is_clear());
    }

    #[test]
    fn extend_contains() {
        let mut set = BitSet::with_capacity(100);
        assert!(set.is_clear());
        set.extend([1, 2, 0, 99, 64, 65]);
        assert!(!set.is_clear());
        assert_eq!(set, BitSet {data:vec![0b111,0b100000000000000000000000000000000011], capacity:100});
        assert!(set.get(0));
        assert!(set.get(1));
        assert!(set.get(2));
        assert!(set.get(99));
        assert!(set.get(64));
        assert!(set.get(65));
        assert!(!set.get(3));
        assert!(!set.get(4));
        assert!(!set.get(23));
        assert!(!set.is_clear());
    }

    #[test]
    fn get() {
        let mut set = BitSet::with_capacity(100);
        set.extend([1, 2, 0, 99, 64, 65]);
        assert_eq!(set.count(), 6);
        assert!(set.get(0));
    }

    #[test]
    fn ones() {
        let mut set = BitSet::with_capacity(100);
        set.extend([1, 2, 0, 99, 64, 65]);
        assert_eq!(set.ones().collect::<BTreeSet<usize>>(), BTreeSet::from_iter([1, 2, 0, 99, 64, 65]))
    }
}
