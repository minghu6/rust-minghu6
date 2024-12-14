use std::{
    borrow::{Borrow, BorrowMut},
    mem::{replace, MaybeUninit},
    ops::{Index, IndexMut, Range, RangeBounds},
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
    arr: [Option<T>; C],
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

    pub fn new() -> Self {
        Self {
            len: 0,
            arr: [const { None }; C],
        }
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

        self.len += 1;
        self.arr[idx].replace(val);
    }

    pub fn remove(&mut self, idx: usize) -> T {
        debug_assert!(
            idx < self.len,
            "index {idx} overflow for lenghth {} ",
            self.len
        );

        let val = self.arr[idx].take().unwrap();
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
        match self[..].binary_search(&val) {
            Ok(idx) => {
                /* Repalce Data */

                self.arr[idx].replace(val)
            }
            Err(idx) => {
                /* Insert Data */

                self.insert(idx, val);

                None
            }
        }
    }

    /// split off within initialized content
    pub fn split_off(&mut self, at: usize) -> Self {
        debug_assert!(at < self.len);

        let off_len = self.len - at;
        let mut off_arr = [const { None }; C];

        for i in at..self.len {
            // KEY: ship with ownership
            off_arr[i - at] = self.arr[i].take();
        }

        self.len = at;

        Self {
            len: off_len,
            arr: off_arr,
        }
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
                    yield self.arr[i].take().unwrap();
                }

                unsafe {
                    let ptr = self.arr.as_mut_ptr();

                    std::ptr::copy(
                        ptr.add(end),
                        ptr.add(start),
                        self.len - end,
                    );

                    self.len -= end - start;
                }
            },
        )
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self[..].iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> + 'a {
        self[..].iter_mut()
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut it = Self::new();

        it.extend(iter);

        it
    }

    /// Extend without trunction (be careful of overflow)
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for val in iter.into_iter() {
            self.push(val);
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.arr.into_iter().take(self.len).map(|x| x.unwrap())
    }

    pub fn as_slice(&self) -> &[T] {
        unimplemented!()
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unimplemented!()
    }
}

impl<T, const C: usize> FromIterator<T> for StackVec<T, C> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter)
    }
}

impl<T, const C: usize> Extend<T> for StackVec<T, C> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.extend(iter);
    }
}

impl<T, const C: usize> IntoIterator for StackVec<T, C> {
    type Item = T;

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

// impl<T, const C: usize> AsMut<[T]> for StackVec<T, C> {
//     fn as_mut(&mut self) -> &mut [T] {
//         todo!()
//     }
// }

/// Refer the [doc](https://doc.rust-lang.org/core/option/index.html)
///
/// transmute from T::Some(_) to T or vice versa is guaranted
impl<T, const C: usize> Borrow<[T]> for StackVec<T, C> {
    fn borrow(&self) -> &[T] {
        &self[..]
    }
}

impl<T, const C: usize> BorrowMut<[T]> for StackVec<T, C> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self[..]
    }
}

impl<T, const C: usize, I: std::slice::SliceIndex<[T]>> Index<I> for StackVec<T, C>{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, const C: usize, I: std::slice::SliceIndex<[T]>> IndexMut<I> for StackVec<T, C>{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_slice_mut(), index)
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

    pub fn new() -> Self {
        Self {
            len: 0,
            arr: MaybeUninit::uninit_array(),
        }
    }

    pub fn exact(&self) -> &[T] {
        unsafe { MaybeUninit::slice_assume_init_ref(&self.arr[..self.len]) }
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
            "index {idx} overflow for lenghth {} ",
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
        match self.exact().binary_search(&val) {
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

        unsafe { MaybeUninit::slice_assume_init_ref(&self.arr[at..oldlen]) }
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

impl<T, const C: usize> Extend<T> for PartialInitArray<T, C> {
    /// Extend without trunction (be careful of overflow)
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for val in iter {
            self.push(val);
        }
    }
}

impl<T, const C: usize> Index<usize> for PartialInitArray<T, C> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len);

        unsafe { self.arr[index].assume_init_ref() }
    }
}

impl<T, const C: usize> IndexMut<usize> for PartialInitArray<T, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len);

        unsafe { self.arr[index].assume_init_mut() }
    }
}

impl<T: Clone, const C: usize> Clone for PartialInitArray<T, C> {
    fn clone(&self) -> Self {
        let mut arr = MaybeUninit::uninit_array();

        unsafe {
            std::ptr::copy(&self.arr, &mut arr, self.len);
        }

        Self { len: self.len, arr }
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
}
