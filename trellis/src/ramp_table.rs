use core::ops::Range;
use log::debug;

#[derive(Clone, Eq, PartialEq)]
pub struct RampTable<T> {
    /// contains the index into values[] where each entry starts
    pub index: Vec<u32>,
    pub values: Vec<T>,
}

impl<T> RampTable<T> {
    pub fn new() -> Self {
        Self {
            index: vec![0],
            values: Vec::new(),
        }
    }

    pub fn with_capacity(keys_capacity: usize, values_capacity: usize) -> Self {
        let mut table = Self {
            index: Vec::with_capacity(keys_capacity + 1),
            values: Vec::with_capacity(values_capacity),
        };
        table.index.push(0);
        table
    }

    pub fn push_value(&mut self, value: T) {
        self.values.push(value);
    }

    pub fn finish_key(&mut self) {
        self.index.push(self.values.len() as u32);
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.index.clear();
        self.index.push(self.values.len() as u32);
    }

    // Iterates slices, one slice for each entry.
    pub fn iter(&self) -> impl Iterator<Item = &[T]> + '_ + DoubleEndedIterator + ExactSizeIterator {
        self.index
            .windows(2)
            .map(move |w| &self.values[w[0] as usize..w[1] as usize])
    }

    // Iterates mutable slices, one slice for each entry.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let mut index_iter = self.index.iter();
        let last_index = (*index_iter.next().unwrap()) as usize;
        IterMut {
            last_index,
            index_iter,
            values: &mut self.values,
        }
    }

    pub fn entry_values_range(&self, index: usize) -> Range<usize> {
        self.index[index] as usize..self.index[index + 1] as usize
    }

    pub fn entry_values(&self, index: usize) -> &[T] {
        &self.values[self.entry_values_range(index)]
    }

    pub fn entry_values_mut(&mut self, index: usize) -> &mut [T] {
        let range = self.entry_values_range(index);
        &mut self.values[range]
    }

    pub fn all_values(&self) -> &[T] {
        &self.values
    }

    pub fn all_values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    pub fn iter_pairs(&self) -> impl Iterator<Item = (usize, &'_ T)> {
        self.iter()
            .enumerate()
            .map(move |(i, values)| values.iter().map(move |v| (i, v)))
            .flatten()
    }

    pub fn iter_pairs_manual(&self) -> impl Iterator<Item = (usize, &'_ T)> {
        struct Pairs<'a, T> {
            value_iter: core::slice::Iter<'a, T>,
            index_iter: core::slice::Iter<'a, u32>,
            current_key: usize,
            current_index: usize,
            num_values: usize,
        }
        impl<'a, T> Iterator for Pairs<'a, T> {
            type Item = (usize, &'a T);
            fn next(&mut self) -> Option<Self::Item> {
                if let Some(value) = self.value_iter.next() {
                    // What key is this value for?
                    while self.num_values == 0 {
                        let next_index = *self.index_iter.next().unwrap() as usize;
                        self.num_values = next_index - self.current_index;
                        self.current_index = next_index;
                        self.current_key += 1;
                    }
                    // TODO: current_key is not correct yet
                    self.num_values -= 1;
                    Some((self.current_key, value))
                } else {
                    None
                }
            }
        }

        Pairs {
            value_iter: self.values.iter(),
            index_iter: self.index.iter(),
            current_key: 0,
            current_index: 0,
            num_values: 0,
        }
    }

    /// Returns the number of distinct keys in the table.
    pub fn num_keys(&self) -> usize {
        self.index.len() - 1
    }

    /// Returns the total number of values (in all entries) in the table.
    pub fn num_values(&self) -> usize {
        self.values.len()
    }

    pub fn push_entry_copy(&mut self, values: &[T])
    where
        T: Copy,
    {
        self.values.extend(values.iter());
        self.finish_key();
    }

    pub fn push_entry_clone(&mut self, values: &[T])
    where
        T: Clone,
    {
        self.values.extend(values.iter().cloned());
        self.finish_key();
    }

    pub fn push_entry_extend<I: Iterator<Item = T>>(&mut self, values: I) {
        self.values.extend(values);
        self.finish_key();
    }
}

pub struct IterMut<'a, T> {
    last_index: usize,
    index_iter: core::slice::Iter<'a, u32>,
    values: &'a mut [T],
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut [T];
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&index) = self.index_iter.next() {
            let values = std::mem::replace(&mut self.values, &mut []);
            let entry_len = index as usize - self.last_index;
            let (head, tail) = values.split_at_mut(entry_len);
            self.values = tail;
            self.last_index = index as usize;
            Some(head)
        } else {
            None
        }
    }
}

use core::fmt::{Debug, Formatter};
impl<T: Debug> Debug for RampTable<T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> core::fmt::Result {
        fmt.debug_map()
            .entries(
                self.iter()
                    .enumerate()
                    .filter(|(_, values)| !values.is_empty()),
            )
            .finish()
    }
}

/// Helps with constructing a RampTable from a sequence of (key, value) pairs.
/// The caller may report 'key' values in any order.
#[derive(Debug)]
pub struct RampTableBuilder<T> {
    items: Vec<(u32, T)>,
}

impl<T> RampTableBuilder<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity)
        }
    }

    pub fn push(&mut self, key: u32, value: T) {
        self.items.push((key, value));
    }

    pub fn extend<I: Iterator<Item = (u32, T)>>(&mut self, iter: I) {
        self.items.extend(iter);
    }

    pub fn finish(mut self) -> RampTable<T> {
        let mut items = core::mem::replace(&mut self.items, Vec::new());
        items.sort_by_key(move |&(key, ref _value)| key);
        if items.is_empty() {
            return RampTable::new();
        }
        debug!("RampTableBuilder: finish(): sorting {} items", items.len());
        let num_keys = items.last().unwrap().0 as usize + 1;
        debug!("num_keys = {}", num_keys);
        let num_values = items.len();

        let mut table: RampTable<T> = RampTable::with_capacity(num_keys, num_values);
        for (key, value) in items.into_iter() {
            while table.num_keys() < (key as usize) {
                table.finish_key();
            }
            table.push_value(value);
        }
        while table.num_keys() < num_keys {
            table.finish_key();
        }
        debug!("table.len = {}", table.num_keys());
        table
    }
}
