use core::ops::Range;
use bit_vec::BitVec;

/// `VecOption` is a collection that is semantically equivalent to `Vec<Option<T>>` but which
/// uses a different memory representation. This representation can be more efficient
/// for some algorithms.
/// 
/// The representation stores `[T]` in one allocation, similar to `Vec<T>`, and stores a
/// bitmap which indicates which items are present. This has many advantages:
/// 
/// * the bitmap wastes less memory; every bit can be used to store meaningful information,
///   rather than using an entire u8 for Option's discriminant (and potentially more for
///   alignment padding) for those `T` which cannot rely on specialized representations for
///   `Option<T>`.
/// 
/// * for contiguous runs of items that are present, a `&[T]` can be safely synthesized.
/// 
/// * an existing `Vec<T>` can be converted to `VecOption<T>` without moving or reallocating
///   items.
/// 
/// * a `VecOption<T>` can be efficiently converted to a `Vec<T>`, by compacting items that
///   are present. Compaction can be done without reallocation.
pub struct VecOption<T> {
    /// This Vec contains the items, *BUT* some of the items are uninitialized.
    /// This means we _cannot_ just drop 'vec' (or the containing VecOption),
    /// because doing so would free the memory that contains the items without
    /// running drop() on the individual items. We also cannot use any method
    /// of Vec that might actually touch the items or even create a slice over
    /// the items. Doing so would be undefined behavior.
    /// 
    /// For that reason, we restrict our usage of `vec` to only these methods:
    /// len(), push(), pop(), set_len()
    /// 
    vec: Vec<T>,
    present: BitVec,
}

impl<T: Clone> Clone for VecOption<T> {
    fn clone(&self) -> Self {
        let clone_capacity = self.vec.len();
        let mut clone_vec: Vec<T> = Vec::with_capacity(clone_capacity);
        for (i, item_present) in self.present.iter().enumerate() {
            if item_present {
                unsafe {
                    core::ptr::write(clone_vec.as_mut_ptr().add(i), (*self.vec.as_ptr().add(i)).clone());
                }
            }
        }
        Self {
            vec: clone_vec,
            present: self.present.clone(),
        }
    }
}

#[test]
fn basic_test() {
    let mut v: VecOption<i32> = VecOption::new();
    v.push(Some(333));
    assert_eq!(v.get_copy(0), Some(333));
}

impl<T> VecOption<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            present: BitVec::new()
        }
    }
    
    /// Takes ownership of a `Vec<T>` and creates a `VecOption<T>` where every entry is converted
    /// from `T` to `Some(T)`. This function is efficient; `vec` is not modified.
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::from_vec(vec![100, 200]);
    /// assert_eq!(v.into_iter().collect::<Vec<Option<i32>>>(), vec![Some(100), Some(200)]);
    /// ```
    pub fn from_vec(vec: Vec<T>) -> Self {
        let len = vec.len();
        Self {
            present: BitVec::from_elem(len, true),
            vec
        }
    }

    /// Allocates a new `VecOption<T>` of a given length, with the contents being equivalent to
    /// `vec![None; len]`.
    /// 
    /// This is the most efficient way to construct a `VecOption` that contains a repeated run
    /// of `None`.
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::<i32>::new_repeat_none(3);
    /// v.push(Some(42));
    /// assert_eq!(v.into_iter().collect::<Vec<Option<i32>>>(), vec![None, None, None, Some(42)]);
    /// ```
    pub fn new_repeat_none(len: usize) -> Self {
        let mut vec: Vec<T> = Vec::with_capacity(len);
        unsafe { vec.set_len(len); }
        Self {
            vec,
            present: BitVec::from_elem(len, false),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            present: BitVec::with_capacity(capacity),
        }
    }

    /// The number of items in this vector. This includes both `Some` and `None` items.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if the collection is empty.
    /// 
    /// A collection that contains only `None` values is not empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Finds all of the `Some` values within the `VecOption<T>` and moves them so that they are
    /// contiguous, starting at index 0. Returns a mutable slice over the contiguous `Some(T)` entries.
    /// The length of that slice is equal to the total number of `Some(T)` entries.
    /// 
    /// This method changes the length of the vector. After this 
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::new_repeat_none(5);
    /// assert_eq!(
    ///     v.iter().collect::<Vec<Option<&char>>>(),
    ///     vec![None, None, None, None, None]);
    /// v.set_some(1, 'A');
    /// v.set_some(4, 'B');
    /// assert_eq!(
    ///     v.iter().collect::<Vec<Option<&char>>>(),
    ///     vec![None, Some(&'A'), None, None, Some(&'B')]);
    /// assert_eq!(v.compact(), &['A', 'B']);
    /// assert_eq!(v.into_iter().collect::<Vec<Option<char>>>(), vec![Some('A'), Some('B')])
    /// ```
    pub fn compact(&mut self) -> &mut [T] {
        let len = self.present.len();
        let items_ptr = self.vec.as_mut_ptr();
        let mut num_keep = 0;
        for (i, item_present) in self.present.iter().enumerate() {
            if item_present {
                if i != num_keep {
                    unsafe { core::ptr::copy_nonoverlapping(items_ptr.add(i), items_ptr.add(num_keep), 1); }
                }
                num_keep += 1;
            }
        }
        if num_keep != len {
            for i in 0..num_keep {
                self.present.set(i, true);
            }
            for i in num_keep..len {
                self.present.set(i, false);
            }
            // Change the length of the collection
            unsafe { self.vec.set_len(num_keep); }
            self.present.truncate(num_keep);
        }

        // This is the only time this is safe, because we know that present[0..len] = true.
        &mut self.vec
    }

    /// Finds all of the `Some` values within the `VecOption<T>` and moves them so that they are
    /// contiguous, starting at index 0, and then converts the `VecOption<T>` to a `Vec<T>` whose
    /// length is equal to the number of `Some` values that were found.
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::<i32>::new();
    /// v.push(None);
    /// v.push(Some(100));
    /// v.push(None);
    /// v.push(Some(200));
    /// v.push(None);
    /// v.push(None);
    /// v.push(Some(300));
    /// assert_eq!(v.some_into_vec(), vec![100, 200, 300]);
    /// ```
    pub fn some_into_vec(mut self) -> Vec<T> {
        // We extract the vec because we are going to return it to the caller.
        // But we also extract the 'present' bit vector so that the Drop impl
        // doesn't run 'drop' on elements that have been moved.
        let new_len = self.compact().len();
        let mut vec = core::mem::replace(&mut self.vec, Vec::new());
        core::mem::replace(&mut self.present, BitVec::new());
        unsafe { vec.set_len(new_len); }
        vec
    }

    pub fn get_ref(&mut self, index: usize) -> Option<&T> {
        if self.present[index] {
            Some(unsafe { &*self.vec.as_ptr().add(index) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.present[index] {
            Some(unsafe { &mut *self.vec.as_mut_ptr().add(index) })
        } else {
            None
        }
    }

    pub fn get_copy(&mut self, index: usize) -> Option<T> where T: Copy {
        self.get_ref(index).map(|v| *v)
    }

    pub fn get_clone(&mut self, index: usize) -> Option<T> where T: Clone {
        self.get_ref(index).map(|v| v.clone())
    }

    /// Sets an item in the vector, taking `Option<T>`.
    /// The existing item is returned.
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::new();
    /// v.push_some(100i32);
    /// assert_eq!(v.replace(0, Some(200)), Some(100));
    /// assert_eq!(v.get_copy(0), Some(200));
    /// ```
    pub fn replace(&mut self, index: usize, value: Option<T>) -> Option<T> {
        if let Some(value) = value {
            self.replace_some(index, value)
        } else {
            self.replace_none(index)
        }
    }

    /// Sets an item in the vector to `Some`, taking `T`.
    /// The existing item is returned.
    pub fn replace_some(&mut self, index: usize, value: T) -> Option<T> {
        let old_value = if self.present[index] {
            Some(unsafe {
                core::ptr::read(self.vec.as_ptr().add(index))
            })
        } else {
            self.present.set(index, true);
            None
        };
        unsafe { core::ptr::write(self.vec.as_mut_ptr().add(index), value); }
        old_value
    }

    /// Sets an item in the vector to `None`.
    /// The existing item is returned.
    pub fn replace_none(&mut self, index: usize) -> Option<T> {
        if self.present[index] {
            self.present.set(index, false);
            Some(unsafe {
                core::ptr::read(self.vec.as_ptr().add(index))
            })
        } else {
            None
        }
    }

    /// Sets an item in the vector, taking `Option<T>`.
    /// If the caller will always pass a `Some` value, then using `set_some`
    /// may be more efficient.
    /// The existing item is dropped.
    pub fn set(&mut self, index: usize, value: Option<T>) {
        self.replace(index, value);
    }

    /// Sets an item in the vector to `Some`, taking `T`.
    /// The existing item is dropped.
    pub fn set_some(&mut self, index: usize, value: T) {
        self.replace_some(index, value);
    }

    /// Sets an item in the vector to `None`.
    /// The existing item is dropped.
    pub fn set_none(&mut self, index: usize) {
        self.replace_none(index);
    }

    /// Iterates the vector as `Option<&T>`.
    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> + '_ {
        self.present.iter().enumerate().map(move |(i, is_present)| {
            if is_present {
                Some(unsafe { &*self.vec.as_ptr().add(i) })
            } else {
                None
            }
        })
    }

    /// Iterates the vector as `Option<&mut T>`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = Option<&mut T>> + '_ {
        let items_ptr = self.vec.as_mut_ptr();
        self.present.iter().enumerate().map(move |(i, is_present)| {
            if is_present {
                Some(unsafe { &mut *items_ptr.add(i) })
            } else {
                None
            }
        })
    }

    /// Iterates only the `Some` items in the vector, as `(index, &T)`,
    /// where `index` is the index of each such item.
    pub fn iter_some_index(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        self.present.iter().enumerate().flat_map(move |(i, is_present)| {
            if is_present {
                Some((i, unsafe { &*self.vec.as_ptr().add(i) }))
            } else {
                None
            }
        })
    }

    /// Iterates only the `Some` items in the vector, as `&mut T`,
    /// where `index` is the index of each such item.
    pub fn iter_some_index_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> + '_ {
        let items_ptr = self.vec.as_mut_ptr();
        self.present.iter().enumerate().flat_map(move |(i, is_present)| {
            if is_present {
                Some((i, unsafe { &mut *items_ptr.add(i) }))
            } else {
                None
            }
        })
    }

    /// Iterates contiguous runs of `Some` items as `&[T]`.
    /// 
    /// Example:
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::<i32>::new();
    /// v.push(Some(1));
    /// v.push(Some(2));
    /// v.push(None);
    /// v.push(Some(3));
    /// v.push(Some(4));
    /// v.push(Some(5));
    /// v.push(None);
    /// v.push(None);
    /// let mut iter = v.iter_runs();
    /// assert_eq!(iter.next(), Some(vec![1, 2].as_slice()));
    /// assert_eq!(iter.next(), Some(vec![3, 4, 5].as_slice()));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_runs(&self) -> IterRuns<'_, T> {
        IterRuns {
            vec: self,
            next: 0
        }
    }

    /// Iterations continguous runs of `Some` items as `&mut [T]`.
    /// Example:
    /// 
    /// ```
    /// # use trellis::vec_option::VecOption;
    /// let mut v = VecOption::<i32>::new();
    /// v.push(Some(5));
    /// v.push(Some(4));
    /// v.push(Some(3));
    /// v.push(None);
    /// v.push(Some(2));
    /// v.push(Some(1));
    /// for run in v.iter_runs_mut() {
    ///   run.sort();
    /// }
    /// assert_eq!(
    ///     v.into_iter().collect::<Vec<Option<i32>>>(),
    ///     vec![Some(3), Some(4), Some(5), None, Some(1), Some(2)]
    /// );
    /// ```
    pub fn iter_runs_mut(&mut self) -> IterRunsMut<'_, T> {
        IterRunsMut {
            vec: self,
            next: 0
        }
    }

    /// Pushes a new `Option<T>` onto the end of the vector.
    /// If the caller will always pass a `Some` value, then it can be more efficient
    /// to call `push_some` instead.
    /// If the caller will always pass a `None` value, then it can be more efficient
    /// to call `push_none` instead.
    pub fn push(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.push_some(value);
        } else {
            self.push_none();
        }
    }

    /// Pushes a new `T` value onto the end of the vector, as a `Some` value.
    pub fn push_some(&mut self, value: T) {
        self.present.push(true);
        self.vec.push(value);
    }

    /// Pushes a new `None` value onto the end of the vector.
    pub fn push_none(&mut self) {
        let len = self.vec.len();
        self.present.push(false);
        self.vec.reserve(1);
        unsafe { self.vec.set_len(len + 1); }
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        self.vec.reserve(new_capacity);
        self.present.reserve(new_capacity);
    }

    /// Pops the last entry on the stack. This behaves exactly as though
    /// `VecOption<T>` were `Vec<Option<T>>`.
    pub fn pop(&mut self) -> Option<Option<T>> {
        assert_eq!(self.vec.len(), self.present.len());
        if let Some(last_is_present) = self.present.pop() {
            if last_is_present {
                Some(self.vec.pop())
            } else {
                let new_len = self.vec.len() -1 ;
                unsafe { self.vec.set_len(new_len); }
                Some(None)
            }
        } else {
            None
        }
    }

    /// Pops the last `Some` value from the `VecOption`, while also popping any number of
    /// `None` values.
    /// 
    /// Example:
    /// 
    /// ```
    /// let mut v = trellis::vec_option::VecOption::<i32>::new();
    /// v.push(Some(42));
    /// v.push(None);
    /// v.push(None);
    /// assert_eq!(v.pop_some(), Some(42));
    /// ```
    pub fn pop_some(&mut self) -> Option<T> {
        while let Some(opt) = self.pop() {
            if opt.is_some() {
                return opt;
            }
        }
        None
    }
}

fn find_next_run(present: &BitVec, search: usize) -> Range<usize> {
    let mut next = search;
    while next < present.len() && !present[next] {
        next += 1;
    }
    let start = next;
    while next < present.len() && present[next] {
        next += 1;
    }
    let end = next;
    start..end
}

pub struct IterRuns<'a, T> {
    vec: &'a VecOption<T>,
    next: usize,
}

impl<'a, T> Iterator for IterRuns<'a, T> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        let range = find_next_run(&self.vec.present, self.next);
        self.next = range.end;
        if range.start < range.end {
            Some(unsafe {
                core::slice::from_raw_parts(self.vec.vec.as_ptr().add(range.start), range.end - range.start)
            })
        } else {
            None
        }
    }
}

pub struct IterRunsMut<'a, T> {
    vec: &'a mut VecOption<T>,
    next: usize,
}

impl<'a, T> Iterator for IterRunsMut<'a, T> {
    type Item = &'a mut [T];
    fn next(&mut self) -> Option<Self::Item> {
        let range = find_next_run(&self.vec.present, self.next);
        self.next = range.end;
        if range.start < range.end {
            Some(unsafe {
                core::slice::from_raw_parts_mut(self.vec.vec.as_mut_ptr().add(range.start), range.end - range.start)
            })
        } else {
            None
        }
    }
}

impl<T> Drop for VecOption<T> {
    fn drop(&mut self) {
        if core::mem::needs_drop::<T>() {
            for (i, value) in self.present.iter().enumerate() {
                if value {
                    unsafe { core::ptr::drop_in_place(self.vec.as_mut_ptr().add(i)); }
                }
            }
        }
        unsafe {
            self.vec.set_len(0);
        }
    }
}

impl<T> core::iter::FromIterator<Option<T>> for VecOption<T> {
    fn from_iter<I: IntoIterator<Item = Option<T>>>(ii: I) -> Self {
        let iter = ii.into_iter();
        let (min_size, _max_size) = iter.size_hint();
        let mut vec: VecOption<T> = VecOption::with_capacity(min_size);
        for item in iter {
            vec.push(item);
        }
        vec
    }
}

impl<T> IntoIterator for VecOption<T> {
    type Item = Option<T>;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            vec: self,
            next: 0
        }
    }
}

pub struct IntoIter<T> {
    vec: VecOption<T>,
    next: usize
}

impl<T> Iterator for IntoIter<T> {
    type Item = Option<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.next;
        if self.next < self.vec.len() {
            self.next += 1;
            Some(self.vec.replace_none(index))
        } else {
            None
        }
    }
}
