#![allow(dead_code)]

/// Iterates sequences of &[T] where every item in the slice meets some equivalency test.
pub struct IterRuns<'a, T, P> {
    items: &'a [T],
    is_same_run: P
}
impl<'a, T, P: Fn(&T, &T) -> bool> Iterator for IterRuns<'a, T, P> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        if self.items.len() == 0 {
            None
        } else {
            let first = &self.items[0];
            let mut i = 1;
            while i < self.items.len() && (self.is_same_run)(first, &self.items[i]) {
                i += 1;
            }
            let (low, high) = self.items.split_at(i);
            self.items = high;
            Some(low)
        }
    }
}
pub fn iter_runs<'a, T, P: Fn(&T, &T) -> bool>(items: &'a [T], is_same_run: P) -> IterRuns<'a, T, P> {
    IterRuns {
        items: items,
        is_same_run: is_same_run
    }
}


pub struct IterRunsByKey<'a, T, P> {
    items: &'a [T],
    get_key: P
}

impl<'a, T, P: Fn(&T) -> K, K: PartialEq> Iterator for IterRunsByKey<'a, T, P> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        if self.items.len() == 0 {
            None
        } else {
            let first = &self.items[0];
            let first_key = (self.get_key)(first);
            let mut i = 1;
            while i < self.items.len() && first_key == (self.get_key)(&self.items[i]) {
                i += 1;
            }
            let (low, high) = self.items.split_at(i);
            self.items = high;
            Some(low)
        }
    }
}

/// Iterates sequences of &[T] where every item in the slice meets some equivalency test.
/// The caller provides a function which maps items to values that can then be tested for
/// equality, as defined by the PartialEq trait.
pub fn iter_runs_by_key<'a, T, P: Fn(&T) -> K, K: PartialEq>(items: &'a [T], get_key: P) -> IterRunsByKey<'a, T, P> {
    IterRunsByKey {
        items,
        get_key
    }
}
