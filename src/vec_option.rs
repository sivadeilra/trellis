
use bit_vec::BitVec;

/// `VecOption` is a collection that is semantically similar to `Vec<Option<T>>` but which
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
    /// This Vec contains the items, but the "len" of the vec is set to zero.
    /// This means we _cannot_ just drop 'vec' (or the containing VecOption),
    /// because doing so would free the memory that contains the items without
    /// running drop() on the individual items.
    /// 
    /// We do this to eliminate any possibility of accidentally touching items
    /// through &T references within the vec, because doing so would risk UB.
    /// Instead, all access to items is through Vec::as_ptr() and Vec::as_mut_ptr().
    vec: Vec<T>,
    present: BitVec,
}

impl<T: Clone> Clone for VecOption<T> {
    fn clone(&self) -> Self {
        let mut vec = Vec::new();
        vec.reserve_exact(self.vec.capacity());
        assert_eq!(vec.capacity(), self.vec.capacity());
        let self_items_ptr = self.vec.as_ptr();
        let clone_items_ptr: *mut T = vec.as_mut_ptr();
        for (i, item_present) in self.present.iter().enumerate() {
            if item_present {
                unsafe {
                    core::ptr::write(clone_items_ptr.add(i), (*self_items_ptr.add(i)).clone());
                }
            }
        }
        Self {
            vec,
            present: self.present.clone(),
        }
    }
}

impl<T> VecOption<T> {
    pub fn from_vec(vec: Vec<T>) -> Self {
        let len = vec.len();
        Self {
            present: BitVec::from_elem(len, true),
            vec
        }
    }

    pub fn new_with_len(len: usize) -> Self {
        let mut vec: Vec<T> = Vec::new();
        vec.reserve_exact(len);
        assert_eq!(vec.capacity(), len);
        Self {
            present: BitVec::from_elem(len, false),
            vec
        }
    }

    pub fn compact(&mut self) -> usize {
        let len = self.present.len();
        let items_ptr = self.vec.as_mut_ptr();
        let mut i = 0;
        let mut num_keep = 0;
        while i < len {
            if self.present[i] {
                if i != num_keep {
                    unsafe { core::ptr::copy_nonoverlapping(items_ptr.add(i), items_ptr.add(num_keep), 1); }
                }
                num_keep += 1;
            }
            i += 1;
        }
        if num_keep != len {
            for i in 0..num_keep {
                self.present.set(i, true);
            }
            for i in num_keep..len {
                self.present.set(i, false);
            }
        }
        num_keep
    }

    pub fn compact_into_vec(mut self) -> Vec<T> {
        let new_len = self.compact();
        let mut vec = core::mem::replace(&mut self.vec, Vec::new());
        core::mem::replace(&mut self.present, BitVec::new());
        unsafe { vec.set_len(new_len); }
        vec
    }

    pub fn take(&mut self, index: usize) -> Option<T> {
        if self.present[index] {
            self.present.set(index, false);
            Some(unsafe {
                core::ptr::read(self.vec.as_ptr().add(index))
            })
        } else {
            None
        }
    }

    pub fn get(&mut self, index: usize) -> Option<&T> {
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

    pub fn set_option(&mut self, index: usize, value: Option<T>) -> Option<T> {
        if let Some(value) = value {
            self.set_some(index, value)
        } else {
            self.set_none(index)
        }
    }

    pub fn set_some(&mut self, index: usize, value: T) -> Option<T> {
        let old_value = self.take(index);
        self.present.set(index, true);
        unsafe { core::ptr::write(self.vec.as_mut_ptr().add(index), value); }
        old_value
    }

    pub fn set_none(&mut self, index: usize) -> Option<T> {
        self.take(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> + '_ {
        self.present.iter().enumerate().map(move |(i, is_present)| {
            if is_present {
                Some(unsafe { &*self.vec.as_ptr().add(i) })
            } else {
                None
            }
        })
    }

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

    pub fn iter_present(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        self.present.iter().enumerate().flat_map(move |(i, is_present)| {
            if is_present {
                Some((i, unsafe { &*self.vec.as_ptr().add(i) }))
            } else {
                None
            }
        })
    }

    pub fn iter_present_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> + '_ {
        let items_ptr = self.vec.as_mut_ptr();
        self.present.iter().enumerate().flat_map(move |(i, is_present)| {
            if is_present {
                Some((i, unsafe { &mut *items_ptr.add(i) }))
            } else {
                None
            }
        })
    }

/*
    pub fn iter_runs(&mut self) -> impl Iterator<Item = (usize, &[T])> + '_ {
        unimplemented!();
    }

    pub fn iter_runs_mut(&mut self) -> impl Iterator<Item = (usize, &mut [T])> + '_ {
        unimplemented!();
    }
*/
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
