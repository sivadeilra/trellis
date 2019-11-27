
use core::cmp::Ordering;

pub struct PQ<T> {
    pq: Vec<T>
}

fn left(parent: usize) -> usize { parent * 2 + 1 }
fn right(parent: usize) -> usize { parent * 2 + 2 }
fn parent(child: usize) -> usize { (child - 1) / 2 }

impl<T: Ord> PQ<T> {
    pub fn new() -> Self {
        Self {
            pq: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pq: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.pq.clear();
    }

    /// Inserts an item into the partially-sorted heap.
    pub fn insert(&mut self, item: T) {
        let index = self.pq.len();
        self.pq.push(item);
        let mut i = index;
        while i > 0 {
            let parent = parent(i);
            assert_ne!(parent, i);
            if self.pq[parent] >= self.pq[i] {
                break;
            }
            self.pq.swap(i, parent);
            i = parent;
        }
        self.check();
    }

    fn check(&self) {
        for i in 1..self.pq.len() {
            assert!(self.pq[parent(i)] >= self.pq[i]);
        }
    }

    /// Removes the greatest item from the set.
    pub fn remove(&mut self) -> Option<T> {
        if self.pq.is_empty() {
            return None;
        }
        if self.pq.len() == 1 {
            return self.pq.pop();
        }
        let last_index = self.pq.len() - 1;
        self.pq.swap(0, last_index);
        let result = self.pq.pop();
        let mut i: usize = 0;
        loop {
            let left = left(i);
            if left < self.pq.len() && self.pq[i] < self.pq[left] {
                self.pq.swap(i, left);
                i = left;
                continue;
            }
            let right = right(i);
            if right < self.pq.len() && self.pq[i] < self.pq[right] {
                self.pq.swap(i, right);
                i = right;
                continue;
            }
            break;
        }

        self.check();
        result
    }
}

use core::fmt::{Debug, Formatter};

impl<T: Debug> Debug for PQ<T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "Q: ")?;
        for item in self.pq.iter() {
            write!(fmt, "{:?} ", item)?;
        }
        write!(fmt, "\n")?;
        Ok(())
    }
}
