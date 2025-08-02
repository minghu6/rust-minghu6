use std::{
    mem::{MaybeUninit, replace},
    ops::{Deref, DerefMut, Range, RangeBounds},
};

use coll::*;

pub mod bpt;
pub mod bpt2;
pub mod bpt3;
pub mod bt;
pub mod flatbpt;

////////////////////////////////////////////////////////////////////////////////
//// Macros

/// O(M)
macro_rules! index_of_child_by_rc {
    ($p: expr, $child: expr) => {{
        let p = &$p;
        let child = &$child;

        debug_assert!(child.is_some());

        if let Some(idx) = children!(p).iter().position(|x| x.rc_eq(child)) {
            idx
        } else {
            panic!("There are no matched child");
        }
    }};
}


macro_rules! impl_tree {
    (
        $(#[$attr:meta])*
        $treename:ident {
            $(
                $(#[$field_attr:meta])*
                $name: ident : $ty: ty
            ),*
        }
    ) =>
    {
        $(#[$attr])*
        pub struct $treename<K, V, const M: usize = 32> {
            root: Node<K, V>,

            /* extra attr */
            $(
                $(#[$field_attr])*
                $name: $ty
            ),*
        }
        impl<K, V, const M: usize> $treename<K, V, M> {
            const fn entries_low_bound() -> usize {
                M.div_ceil(2) - 1
            }

            const fn entries_high_bound() -> usize {
                M
            }
        }
    };
}


use impl_tree;
use index_of_child_by_rc;

////////////////////////////////////////////////////////////////////////////////
//// Structures

/// Non-drop values array
#[derive(Debug)]
pub struct PartialInitArray<T, const C: usize> {
    len: usize,
    arr: [MaybeUninit<T>; C],
}

pub struct StackVec<T, const C: usize> {
    len: usize,
    arr: [MaybeUninit<T>; C],
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

////////////////////////////////////////
//// impl StackVec

impl<T, const C: usize> StackVec<T, C> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn new() -> Self {
        Self {
            len: 0,
            arr: [const { MaybeUninit::uninit() }; C],
        }
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        debug_assert!(self.len < C, "StackVec is full");
        debug_assert!(
            idx <= self.len,
            "StackVec index {idx} overflow for length {} ",
            self.len
        );

        if idx < self.len {
            // self.arr[idx..self.len + 1].rotate_right(1);
            unsafe {
                core::ptr::copy(
                    self.arr[idx..].as_ptr(),
                    self.arr[idx + 1..].as_mut_ptr(),
                    self.len - idx,
                );
            }
        }

        self.arr[idx].write(val);

        self.len += 1;
    }

    pub fn remove(&mut self, idx: usize) -> T {
        debug_assert!(
            idx < self.len,
            "index {idx} overflow for length {} ",
            self.len
        );

        let val = unsafe { self.arr[idx].assume_init_read() };

        // self.arr[idx..self.len].rotate_left(1);
        unsafe {
            core::ptr::copy(
                self.arr[idx + 1..].as_ptr(),
                self.arr[idx..].as_mut_ptr(),
                self.len - idx - 1,
            );
        }

        self.len -= 1;

        val
    }

    pub fn push(&mut self, val: T) {
        self.insert(self.len, val);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        Some(self.remove(self.len - 1))
    }

    /// split off within initialized content
    pub fn split_off(&mut self, at: usize) -> Self {
        debug_assert!(at < self.len);

        let off_len = self.len - at;
        let mut off = Self::new();

        unsafe {
            core::ptr::copy_nonoverlapping(
                self.arr[at..].as_ptr(),
                off.arr.as_mut_ptr(),
                off_len,
            )
        };
        off.len = off_len;

        self.len = at;

        off
    }

    ///
    /// ```
    /// use m6_coll_st::bt::StackVec;
    ///
    /// let mut arr = (0..=5).into_iter().collect::<StackVec<_, 10>>();
    ///
    /// assert_eq!(vec![2, 3], arr.drain(2..=3).collect::<Vec<_>>());
    /// assert_eq!(vec![0, 1, 4, 5], arr.into_iter().collect::<Vec<_>>());
    /// ```
    pub fn drain<'a, R: RangeBounds<usize> + 'a>(
        &'a mut self,
        range: R,
    ) -> impl Iterator<Item = T> + 'a {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let Range { start, end } =
                    std::slice::range(range, ..self.len());

                for i in start..end {
                    yield unsafe { self.arr[i].assume_init_read() };
                }

                let drain_len = end - start;

                if drain_len < self.len {
                    // self.arr[start..self.len].rotate_left(drain_len);
                    unsafe {
                        core::ptr::copy(
                            self.arr[end..].as_ptr(),
                            self.arr[start..].as_mut_ptr(),
                            self.len - end,
                        );
                    }
                }

                self.len -= drain_len;
            },
        )
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { self.arr[..self.len].assume_init_ref() }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { self.arr[..self.len].assume_init_mut() }
    }
}

impl<T, const C: usize> Drop for StackVec<T, C> {
    fn drop(&mut self) {
        unsafe { self.arr[..self.len].assume_init_drop() };
    }
}

impl<T, const C: usize> Deref for StackVec<T, C> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const C: usize> DerefMut for StackVec<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

////////////////////////////////////////
//// impl PartialInitArray

impl<T, const C: usize> PartialInitArray<T, C> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn new() -> Self {
        Self {
            len: 0,
            arr: [const { MaybeUninit::uninit() }; C],
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { self.arr[..self.len].assume_init_ref() }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { self.arr[..self.len].assume_init_mut() }
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        debug_assert!(self.len < C, "array is full");
        debug_assert!(
            idx <= self.len,
            "index {idx} overflow for length {} ",
            self.len
        );

        if idx < self.len {
            self.arr[idx..self.len + 1].rotate_right(1);
        }

        self.arr[idx].write(val);
        self.len += 1;
    }

    pub fn remove(&mut self, idx: usize) -> T {
        debug_assert!(
            idx < self.len,
            "index {idx} overflow for length {} ",
            self.len
        );

        let val = unsafe { self.arr[idx].assume_init_read() };
        self.arr[idx..self.len].rotate_left(1);

        self.len -= 1;

        val
    }

    pub fn push(&mut self, val: T) {
        self.insert(self.len, val);
    }

    pub fn pop(&mut self) -> T {
        self.remove(self.len - 1)
    }

    /// Binary search and Insert
    pub fn binary_insert(&mut self, val: T) -> Option<T>
    where
        T: Ord,
    {
        match self.as_slice().binary_search(&val) {
            Ok(idx) => {
                /* Repalce Data */

                Some(unsafe {
                    replace(&mut self.arr[idx], MaybeUninit::new(val))
                        .assume_init()
                })
            }
            Err(idx) => {
                /* Insert Data */

                self.insert(idx, val);

                None
            }
        }
    }

    /// split off within initialized content
    pub fn split_off(&mut self, at: usize) -> &[T] {
        debug_assert!(at < self.len);

        let oldlen = replace(&mut self.len, at);

        unsafe { self.arr[at..oldlen].assume_init_ref() }
    }

    pub fn init_with_slice(&mut self, slice: &[T]) {
        self.len = 0;
        self.extend_slice(slice);
    }

    /// Extend without trunction (be careful of overflow)
    pub fn extend_slice(&mut self, slice: &[T]) {
        debug_assert!(self.len() + slice.len() <= C);

        unsafe {
            std::ptr::copy(
                slice.as_ptr(),
                MaybeUninit::slice_as_mut_ptr(&mut self.arr[self.len..]),
                slice.len(),
            );
        }

        self.len += slice.len();
    }
}

impl<T: Copy, const C: usize> Copy for PartialInitArray<T, C> {}

impl<T, const C: usize> Deref for PartialInitArray<T, C> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const C: usize> DerefMut for PartialInitArray<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T, const C: usize> Extend<T> for PartialInitArray<T, C> {
    /// Extend without trunction (be careful of overflow)
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for val in iter {
            self.push(val);
        }
    }
}

impl<T: Clone, const C: usize> Clone for PartialInitArray<T, C> {
    fn clone(&self) -> Self {
        let len = self.len;
        let mut arr = [const { MaybeUninit::uninit() }; C];

        unsafe {
            core::ptr::copy(&self.arr, &mut arr, len);
        }

        Self { len, arr }
    }
}

impl<T, const C: usize> IntoIterator for PartialInitArray<T, C> {
    type Item = T;

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe { self.arr.into_iter().take(self.len).map(|x| x.assume_init()) }
    }
}


#[cfg(test)]
mod tests {
    macro_rules! dict_insert {
        ($dict:ident, $num:expr) => {
            $dict.insert($num, $num);
            assert_eq!($dict.get(&$num), Some(&$num));
            $dict.validate();
        };
        ($dict:ident, $key:expr, $val:expr) => {
            $dict.insert($key, $val);
            assert_eq!($dict.get(&$key), Some(&$val));
            $dict.validate();
        };
    }

    #[allow(unused)]
    macro_rules! dict_get {
        ($dict:ident, $num:expr) => {
            assert_eq!($dict.get(&$num), Some(&$num));
        };
    }

    macro_rules! dict_remove {
        ($dict:ident, $num:expr) => {
            assert_eq!($dict.get(&$num), Some(&$num));
            assert_eq!($dict.remove(&$num), Some($num));
            assert!($dict.get(&$num).is_none());
            $dict.validate();
        };
    }

    #[allow(unused)]
    pub(super) use dict_get;
    pub(super) use dict_insert;
    pub(super) use dict_remove;

    use rand::*;

    use crate::bt::StackVec;


    #[test]
    fn test_stack_vec() {
        const C: usize = 500;

        let mut cg = Vec::<u16>::with_capacity(C);
        let mut eg = StackVec::<u16, 500>::new();

        /* test basic insert */

        for _ in 0..C {
            let idx = thread_rng().gen_range(0..=cg.len());
            let v = thread_rng().gen_range(0..=C as u16);

            cg.insert(idx, v);
            eg.insert(idx, v);

            assert_eq!(cg.get(idx), eg.get(idx))
        }

        for _ in 0..cg.len() * 10 {
            // 50-50 get
            let idx = thread_rng().gen_range(0..cg.len() * 2);

            assert_eq!(cg.get(idx), eg.get(idx))
        }

        /* test basic remove */

        while !cg.is_empty() {
            let idx = thread_rng().gen_range(0..cg.len());

            assert_eq!(cg.remove(idx), eg.remove(idx))
        }
    }
}
