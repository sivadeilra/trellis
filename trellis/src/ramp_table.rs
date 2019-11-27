use core::ops::Range;
use log::debug;

/// Contains a mapping from index (key) to a set of values. Each set of values is stored in order
/// of increasing index (key).
/// 
/// `RampTable` is optimized for certain usage patterns. For those patterns, it is an efficient
/// representation. For other usage patterns, `RampTable` may be inefficient or unsuitable.
/// 
/// A `RampTable` stores its data in two vectors: `index` and `values`. The `index` table contains
/// offsets into `values`. These offsets are stored in non-decreasing order. The numeric difference
/// between two consecutive entries in `values` gives the length of the slice within `values that
/// corresponds to the items for the lower index.
/// 
/// The usage pattern that `RampTable` is designed for is appending a sequence of (key, value)
/// pairs, where each key is in non-decreasing order. The result is a representation that is
/// compact (has high data density), has constant-time lookup, and efficiently represents datasets
/// that have a wide variety of counts of items.
/// 
/// This representation also allows acquiring references to slices that span more than one key.
/// 
/// Terminology:
/// * key - An integer which identifies an ordered list of values in the table.
/// * value - One item that is present in the table. Each value is associated with exactly one key.
///
/// Note that it is possible to add _unowned_ values to the end of the `RampTable`. A well-formed
/// table will not have any unowned values.
#[derive(Clone, Eq, PartialEq)]
pub struct RampTable<T> {
    /// contains the index into values[] where each entry starts
    pub index: Vec<u32>,
    pub values: Vec<T>,
}

impl<T> Default for RampTable<T> {
    fn default() -> Self {
        Self {
            index: vec![0],
            values: Vec::new(),
        }
    }
}

impl<T> RampTable<T> {
    /// Creates a new, empty `RampTable`.
    pub fn new() -> Self {
        Self {
            index: vec![0],
            values: Vec::new(),
        }
    }

    /// Creates a new, empty `RampTable`, with space preallocated for keys and values.
    pub fn with_capacity(keys_capacity: usize, values_capacity: usize) -> Self {
        let mut table = Self {
            index: Vec::with_capacity(keys_capacity + 1),
            values: Vec::with_capacity(values_capacity),
        };
        table.index.push(0);
        table
    }

    /// Returns `true` if there are no keys in this `RampTable`. Equivalent to `self.len() == 0`.
    /// 
    /// Note that a `RampTable` may still contain unassociated values even if `is_empty` returns
    /// `true.
    pub fn is_empty(&self) -> bool {
        self.index.len() == 1
    }

    /// The number of keys in this `RampTable`. All keys are numbered sequentially.
    pub fn len(&self) -> usize {
        self.index.len() - 1
    }

    /// Pushes a value onto the end of the `RampTable`. The item is _not_ yet associated with any
    /// key. In order to associate all unowned values with the next key, call `finish_key`.
    pub fn push_value(&mut self, value: T) {
        self.values.push(value);
    }

    /// Adds a new key to the end of the key list, and implicitly associates all unassociated values
    /// with that key.
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::ramp_table::RampTable;
    /// let mut rt = RampTable::new();
    /// rt.push_value("foo");
    /// rt.push_value("bar");
    /// rt.finish_key();
    /// assert_eq!(rt.entry_values(0), &["foo", "bar"]);
    /// ```
    pub fn finish_key(&mut self) -> usize {
        let key = self.index.len() - 1;
        self.index.push(self.values.len() as u32);
        key
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.index.clear();
        self.index.push(0);
    }

    /// Iterates slices of values, one slice for each key.
    ///
    /// Example:
    /// 
    /// ```
    /// # use trellis::ramp_table::RampTable;
    /// let mut rt = RampTable::new();
    /// rt.push_value("foo");
    /// rt.push_value("bar");
    /// rt.finish_key(); // 0
    /// rt.finish_key(); // 1
    /// rt.push_value("alpha");
    /// rt.push_value("bravo");
    /// rt.finish_key(); // 2
    /// let mut ii = rt.iter();
    /// assert_eq!(ii.next(), Some(&["foo", "bar"][..]));
    /// assert_eq!(ii.next(), Some(&[][..]));
    /// assert_eq!(ii.next(), Some(&["alpha", "bravo"][..]));
    /// assert_eq!(ii.next(), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &[T]> + '_ + DoubleEndedIterator + ExactSizeIterator {
        self.index
            .windows(2)
            .map(move |w| &self.values[w[0] as usize..w[1] as usize])
    }

    /// Iterates mutable slices of values, one slice for each key.
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::ramp_table::RampTable;
    /// let mut rt = RampTable::new();
    /// rt.push_value("foo");
    /// rt.push_value("bar");
    /// rt.finish_key(); // 0
    /// rt.iter_mut().next().unwrap()[1] = "BAR";
    /// assert_eq!(rt.entry_values(0), &["foo", "BAR"][..]);
    /// ```
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

    /// Returns a slice over _all_ values in the table. The returned slice covers values in all keys
    /// as well as any unassociated values at the end of the table.
    pub fn all_values(&self) -> &[T] {
        &self.values
    }

    /// Returns a mutable slice over _all_ values in the table. The returned slice covers values in 
    /// all keys as well as any unassociated values at the end of the table.
    pub fn all_values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    /// Iterates pairs of `(key, value)` items. The `key` values are guaranteed to be iterated in
    /// non-decreasing order.
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
    #[deprecated]
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

impl<T> core::ops::Index<usize> for RampTable<T> {
    type Output = [T];
    fn index(&self, i: usize) -> &[T] {
        self.entry_values(i)
    }
}

impl<T> core::ops::IndexMut<usize> for RampTable<T> {
    fn index_mut(&mut self, i: usize) -> &mut [T] {
        self.entry_values_mut(i)
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
        let num_keys = items.last().unwrap().0 as usize + 1;
        let num_values = items.len();

        let mut table: RampTable<T> = RampTable::with_capacity(num_keys, num_values);
        for (key, value) in items.into_iter() {
            while table.len() < (key as usize) {
                table.finish_key();
            }
            table.push_value(value);
        }
        while table.len() < num_keys {
            table.finish_key();
        }
        table
    }
}
