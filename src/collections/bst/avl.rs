//!
//! By two Soviet inventors, Georgy Adelson-Velsky and Evgenii Landis(1962)
//!
//! ref 1: https://en.wikipedia.org/wiki/AVL_tree
//!
//! ref 2: https://en.wikipedia.org/wiki/Binary_search_tree
//!


use std::fmt::Write;
use std::fmt::{self, Display};
use std::ptr::{null, null_mut};

use either::Either;

use super::{BSTKey, BSTNode, BST};
use crate::collections::Dictionary;
use crate::etc::Reverse;


////////////////////////////////////////////////////////////////////////////////
//// Struct
////

pub struct AVL<K: BSTKey, V> {
    root: *mut AVLNode<K, V>,
}


struct AVLNode<K: BSTKey, V> {
    left: *mut AVLNode<K, V>,
    right: *mut AVLNode<K, V>,
    paren: *mut AVLNode<K, V>,
    bf: BF, // BF = H(right) - H(left)
    key: *const K,
    value: *mut V,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(u8)]
enum BF {
    N1, // Negative one
    Z,  // Zero
    P1, // Positive one
}

#[allow(unused)]
#[derive(Debug)]
pub struct BFValidateResult {
    h_lf: i32,
    h_rh: i32,
    bf: BF,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement

impl TryFrom<i32> for BF {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            -1 => BF::N1,
            0 => BF::Z,
            1 => BF::P1,
            _ => return Err(()),
        })
    }
}

impl Display for BFValidateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl std::error::Error for BFValidateResult {}


impl BF {
    fn reverse(&self) -> Self {
        match self {
            BF::N1 => BF::P1,
            BF::Z => BF::Z,
            BF::P1 => BF::N1,
        }
    }
}


impl<'a, K: BSTKey + 'a, V: 'a> AVLNode<K, V> {
    pub fn new(key: K, value: V) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),
            bf: BF::Z,
            key: Box::into_raw(box key) as *const K,
            value: Box::into_raw(box value),
        })
    }

    pub fn into_value(self) -> V {
        unsafe { *Box::from_raw(self.value) }
    }

    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let h_lf = self.left_height();
        let h_rh = self.right_height();

        let bf_res = BF::try_from(h_rh - h_lf);
        let validateres = BFValidateResult {
            h_lf,
            h_rh,
            bf: self.bf,
        };

        if bf_res.is_err() || bf_res.unwrap() != self.bf {
            return Err((box validateres) as Box<dyn std::error::Error>);
        }

        Ok(())
    }

    pub fn echo_in_mm(&self, cache: &mut String) -> fmt::Result {
        unsafe {
            (*self.itself()).echo_in_mm(cache, |x, cache| {
                let x_self = x as *mut AVLNode<K, V>;

                let h_lf = if !(*x).left().is_null() {
                    (*(*x).left()).height()
                } else {
                    -1
                };

                let h_rh = if !(*x).right().is_null() {
                    (*(*x).right()).height()
                } else {
                    -1
                };

                let bf_res = BF::try_from(h_rh - h_lf);

                let check_res =
                    if bf_res.is_err() || bf_res.unwrap() != (*x_self).bf {
                        "failed"
                    } else {
                        "pass"
                    };

                writeln!(
                    cache,
                    "BF: {:?}, H(LF): {}, H(RH): {},  {}",
                    (*x_self).bf,
                    h_lf,
                    h_rh,
                    check_res
                )?;

                Ok(())
            })
        }
    }

    pub fn echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache).unwrap();

        println!("{}", cache);
    }
}


impl<'a, K: BSTKey + 'a, V: 'a> BSTNode<'a, K, V> for AVLNode<K, V> {
    fn left(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.left as *mut (dyn BSTNode<K, V> + 'a)
    }

    fn right(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.right as *mut (dyn BSTNode<K, V> + 'a)
    }

    fn paren(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.paren as *mut (dyn BSTNode<K, V> + 'a)
    }

    fn key(&self) -> &K {
        unsafe { &*self.key }
    }

    fn value(&self) -> &V {
        unsafe { &*self.value }
    }

    fn itself(&self) -> *const (dyn BSTNode<'a, K, V> + 'a) {
        self as *const Self
    }

    fn null(&self) -> *const (dyn BSTNode<'a, K, V> + 'a) {
        null::<Self>()
    }

    fn assign_left(&mut self, left: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.left = left as *mut Self;
    }

    fn assign_right(&mut self, right: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.right = right as *mut Self;
    }

    fn assign_paren(&mut self, paren: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.paren = paren as *mut Self;
    }

    fn assign_value(&mut self, value: V) {
        self.value = Box::into_raw(box value);
    }
}




impl<'a, K: BSTKey + 'a, V: 'a> AVL<K, V> {
    fn new() -> Self {
        Self { root: null_mut() }
    }

    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        }
    }

    /// Simple Rotation
    /// ```no_run
    ///             rotate left
    ///    x        =========>          z
    ///  /  \                          / \
    /// t1   z                        x   t4
    /// |   / \                      / \   |
    ///   t23 t4                    t1 t23 |
    ///     |  |                     |   |
    ///        |
    /// ```
    ///
    unsafe fn rotate(
        &mut self,
        x: *mut AVLNode<K, V>,
        z: *mut AVLNode<K, V>,
        rotation: Either<(), ()>, // rotate to left = from right rotation
    ) -> *mut AVLNode<K, V> {
        let t23 = if rotation.is_left() {
            (*z).left
        } else {
            (*z).right
        };

        if !t23.is_null() {
            (*t23).assign_paren(x);
        }

        if rotation.is_left() {
            (*x).assign_right(t23);
            (*z).assign_left(x);
        } else {
            (*x).assign_left(t23);
            (*z).assign_right(x);
        }

        self.subtree_shift(x, z);
        (*x).assign_paren(z);

        /* adjust BF */
        // case-1. only happens with deletion
        if (*z).bf == BF::Z {
            // t23 has been of same height as t4
            if rotation.is_left() {
                (*x).bf = BF::P1;
                (*z).bf = BF::N1;
            } else {
                (*x).bf = BF::N1;
                (*z).bf = BF::P1;
            }
        }
        // case-2 happends with both insertion and deletion
        else {
            (*x).bf = BF::Z;
            (*z).bf = BF::Z;
        }

        z
    }


    /// Double Rotation
    /// ```no_run
    ///             rotate [right]-left         rotate right-[left]
    ///    x        =========>         x        =========>       y
    ///  /   \                        /  \                      / \
    /// t1    z                      t1   y                    x   z
    /// |   /  \                     |   / \                  / \ / \
    ///    y   t4                      t2   z                t1 t2t3t4
    ///   / \   |                       |  / \                |  | | |
    ///  t2 t3                            t3 t4
    ///   |  |                            |   |
    /// ```
    unsafe fn double_rotate(
        &mut self,
        x: *mut AVLNode<K, V>,
        z: *mut AVLNode<K, V>,
        snd_rotation: Either<(), ()>,
    ) -> *mut AVLNode<K, V> {
        /* FIRST ROTATION */
        // z is by 2 higher than its sibing(t1)
        // y is by 1 higher than its sibling(t4) (thereis shouldn't be empty)
        let y = if snd_rotation.is_left() {
            (*z).left
        } else {
            (*z).right
        };

        let (t2, t3) = if snd_rotation.is_left() {
            ((*y).left, (*y).right)
        } else {
            ((*y).right, (*y).left)
        };

        if !t3.is_null() {
            (*t3).assign_paren(z);
        }
        (*z).assign_paren(y);

        if snd_rotation.is_left() {
            (*z).assign_left(t3);
            (*y).assign_right(z);
        } else {
            (*z).assign_right(t3);
            (*y).assign_left(z);
        }

        // skip x-R->z => x-R->y for it would be overrided by second rotation

        /* SECOND ROTATION */
        if snd_rotation.is_left() {
            (*x).assign_right(t2);
            (*y).assign_left(x);
        } else {
            (*x).assign_left(t2);
            (*y).assign_right(x);
        }
        if !t2.is_null() {
            (*t2).assign_paren(x);
        }

        self.subtree_shift(x, y);
        (*x).assign_paren(y);

        /* Adjust BF */
        // h(t1) = h(t4) = max(h(t2), h(t3))
        if (*y).bf == BF::Z {
            (*x).bf = BF::Z;
            (*z).bf = BF::Z;
        } else {
            // double symmetry
            let cond = if snd_rotation.is_left() {
                BF::P1
            } else {
                BF::N1
            };

            if (*y).bf == cond {
                // t3 is heigher
                (*x).bf = cond.reverse();
                (*z).bf = BF::Z;
            } else {
                (*x).bf = BF::Z;
                (*z).bf = cond;
            }
        }
        (*y).bf = BF::Z;

        y
    }


    unsafe fn insert_retracing(&mut self, new_node: *mut AVLNode<K, V>) {
        let mut z = new_node;
        let mut x = (*z).paren;

        while !x.is_null() {
            let g;
            let n;

            let (fst_rotation, snd_rotation, cond) = if z == (*x).right {
                (Either::Left(()), Either::Left(()), BF::P1)
            } else {
                (Either::Right(()), Either::Right(()), BF::N1)
            };

            if (*x).bf == cond {
                g = (*x).paren;

                n = if (*z).bf == cond {
                    self.rotate(x, z, fst_rotation)
                } else {
                    self.double_rotate(x, z, snd_rotation)
                };
            } else {
                if (*x).bf == BF::Z {
                    (*x).bf = cond;
                    z = x;
                    x = (*z).paren;
                    continue;
                } else {
                    (*x).bf = BF::Z;
                    break;
                }
            }

            (*n).assign_paren(g);
            if !g.is_null() {
                if x == (*g).left {
                    (*g).assign_left(n);
                } else {
                    (*g).assign_right(n);
                }
            } else {
                self.assign_root(n);
            }
            break;
        }
    }

    unsafe fn remove_retracing(
        &mut self,
        unbalanced_entry: *mut AVLNode<K, V>,
    ) {
        let mut p = unbalanced_entry;

        // self.echo_stdout();

        while !p.is_null() {
            let p_rh =  (*p).right;
            let p_lf = (*p).left;
            let p_paren = (*p).paren;

            let h_p_lf = (*p).left_height();
            let h_p_rh = (*p).right_height();

            if (h_p_rh - h_p_lf).abs() >= 2 {
                let x = p;

                let y = if h_p_lf > h_p_rh {
                    (*x).left
                } else {
                    (*x).right
                };

                let z = if (*y).bf == BF::N1 {
                    (*y).left
                } else if (*y).bf == BF::P1  {
                    (*y).right
                } else {
                    if y == (*x).left {
                        (*y).left
                    } else {
                        (*y).right
                    }
                };


                if y == (*x).left {
                    if z == (*(*x).left).left {
                        self.rotate(x, y, Either::Right(()));
                    } else if z == (*(*x).left).right {
                        self.double_rotate(x, y, Either::Right(()));
                    }
                } else if y == (*x).right {
                    if z == (*(*x).right).right {
                        self.rotate(x, y, Either::Left(()));
                    } else if z == (*(*x).right).left {
                        self.double_rotate(x, y, Either::Left(()));
                    }
                } else {

                }


            } else {
                (*p).bf = BF::try_from(h_p_rh - h_p_lf).unwrap();
            }

            let p_rh =  (*p).right;
            let p_lf = (*p).left;
            let p_paren = (*p).paren;

            p = (*p).paren;
        }

        // let mut n = unbalanced_entry;
        // let mut x = (*n).paren;

        // while !x.is_null() {
        //     let b;
        //     let z;
        //     let g = (*x).paren;

        //     let (direction, cond) = if n == (*x).left {
        //         (Either::Left(()), BF::P1)
        //     } else {
        //         (Either::Right(()), BF::N1)
        //     };

        //     if (*x).bf == cond {
        //         z = (*x).child(direction.reverse()) as *mut AVLNode<K, V>;
        //         b = (*z).bf;

        //         n = if b == cond.reverse() {
        //             Self::double_rotate(x, z, direction)
        //         } else {
        //             Self::rotate(x, z, direction)
        //         };
        //     } else {
        //         if (*x).bf == BF::Z {
        //             (*x).bf = cond;
        //             break;
        //         } else {
        //             (*x).bf = BF::Z;
        //             n = x;
        //             x = g;
        //             continue;
        //         }
        //     }

        //     (*n).assign_paren(g);
        //     if !g.is_null() {
        //         if x == (*g).left {
        //             (*g).assign_left(n);
        //         } else {
        //             (*g).assign_right(n);
        //         }
        //     } else {
        //         self.assign_root(n);
        //     }

        //     if b == BF::Z {
        //         break;
        //     }
        // }
    }
}


impl<'a, K: BSTKey + 'a, V: 'a> Dictionary<K, V> for AVL<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        let new_node = AVLNode::new(key, value);

        if !self.basic_insert(new_node) {
            return false;
        }

        unsafe {
            self.insert_retracing(new_node);
        }

        true
    }

    ///
    /// case-3
    ///       z
    ///      / \
    ///         y
    ///        / \
    ///     null  x
    ///          / \
    ///
    fn remove(&mut self, key: &K) -> Option<V> {
        let z = self.search_approximately(&key) as *mut AVLNode<K, V>;
        if z.is_null() {
            return None;
        }

        unsafe {
            if (*z).key() != key {
                return None;
            }

            if (*z).left.is_null() && (*z).right.is_null() {
                let retracing_entry = (*z).paren;

                self.subtree_shift(z, (*z).null_mut());

                self.remove_retracing(retracing_entry);
            } else if (*z).left().is_null() {
                let retracing_entry = (*z).right;
                self.subtree_shift(z, (*z).right());

                self.remove_retracing(retracing_entry);

            } else if (*z).right().is_null() {
                let retracing_entry = (*z).left;
                self.subtree_shift(z, (*z).left());

                self.remove_retracing(retracing_entry);
            } else {
                let y = (*z).successor();
                let retracing_entry = if (*y).paren() != z {
                    (*y).paren()
                } else {
                    y
                };

                if (*y).paren() != z {
                    self.subtree_shift(y, (*y).right());

                    (*y).assign_right((*z).right());
                    (*(*y).right()).assign_paren(y);
                }

                self.subtree_shift(z, y);
                (*y).assign_left((*z).left());
                (*(*y).left()).assign_paren(y);

                self.remove_retracing(retracing_entry as *mut AVLNode<K, V>);
            }


            let origin_node = Box::from_raw(z as *mut AVLNode<K, V>);

            Some(origin_node.into_value())
        }
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        self.basic_modify(key, value)
    }

    fn lookup(&self, income_key: &K) -> Option<&V> {
        if let Some(e) = self.basic_lookup(income_key) {
            unsafe { Some((*(e as *mut AVLNode<K, V>)).value()) }
        } else {
            None
        }
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.root.is_null() {
            unsafe { (*self.root).self_validate()? }
        }

        Ok(())
    }
}

impl<'a, K: BSTKey + 'a, V: 'a> BST<'a, K, V> for AVL<K, V> {
    fn itself(&self) -> *const (dyn BST<'a, K, V> + 'a) {
        self as *const Self
    }

    fn root(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.root = root as *mut AVLNode<K, V>;
    }
}


#[cfg(test)]
mod tests {

    use itertools::Itertools;

    use super::AVL;
    use crate::{
        collections::{bst::avl::BF, Dictionary},
        test::dict::{DictProvider, Inode, InodeProvider},
    };

    #[test]
    fn test_avl_bf() {
        assert!(BF::Z == BF::Z)
    }

    #[test]
    fn test_avl_randomdata() {
        let mut dict = AVL::<u16, Inode>::new();

        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u16, Inode>).test_dict(&mut dict);
    }

    #[test]
    fn test_avl_fixeddata_case_0() {
        let mut avl = AVL::<i32, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<i32, ()>;

        dict.insert(10, ());
        assert!(dict.self_validate().is_ok());

        dict.insert(5, ());
        dict.self_validate().unwrap();

        dict.insert(12, ());
        dict.self_validate().unwrap();

        dict.insert(13, ());
        dict.self_validate().unwrap();

        dict.insert(14, ());
        dict.self_validate().unwrap();

        dict.insert(18, ());
        dict.self_validate().unwrap();

        dict.insert(7, ());
        dict.self_validate().unwrap();

        dict.insert(9, ());
        dict.self_validate().unwrap();

        dict.insert(11, ());
        // dict.self_validate().unwrap();

        // dict.insert(22, ());
        // dict.self_validate().unwrap();


        // assert!(dict.lookup(&10).is_some());
        // assert!(dict.lookup(&5).is_some());
        // assert!(dict.lookup(&12).is_some());
        // assert!(dict.lookup(&13).is_some());
        // assert!(dict.lookup(&14).is_some());
        // assert!(dict.lookup(&18).is_some());
        // assert!(dict.lookup(&7).is_some());
        // assert!(dict.lookup(&9).is_some());
        // assert!(dict.lookup(&11).is_some());
        // assert!(dict.lookup(&22).is_some());

        // // avl.echo_stdout();
        // let dict = &mut avl as &mut dyn Dictionary<i32, ()>;

        // assert!(dict.remove(&10).is_some());
        // assert!(dict.lookup(&10).is_none());

        // assert!(dict.remove(&5).is_some());
        // assert!(dict.lookup(&5).is_none());

        // assert!(dict.remove(&12).is_some());

        // assert!(dict.remove(&13).is_some());

        // assert!(dict.remove(&14).is_some());
        // assert!(dict.remove(&18).is_some());
        // assert!(dict.remove(&7).is_some());
        // assert!(dict.remove(&9).is_some());
        // assert!(dict.remove(&11).is_some());
        // assert!(dict.remove(&22).is_some());

        // avl.self_validate().unwrap();
        avl.echo_stdout();
    }

    #[test]
    fn test_avl_fixeddata_case_1() {
        let mut avl = AVL::<u16, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<u16, ()>;

        dict.insert(52, ());
        // assert!(dict.lookup(&52).is_some());

        dict.insert(47, ());
        // assert!(dict.lookup(&47).is_some());

        dict.insert(3, ());
        // assert!(dict.lookup(&3).is_some());

        dict.insert(35, ());
        // assert!(dict.lookup(&35).is_some());

        dict.insert(24, ());
        // // assert!(dict.lookup(&24).is_some());


        avl.echo_stdout();
    }

    #[test]
    fn test_avl_fixeddata_case_2() {
        let mut avl = AVL::<u16, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<u16, ()>;

        dict.insert(6, ());
        dict.insert(29, ());
        dict.insert(26, ());
        dict.insert(10, ());
        dict.insert(17, ());
        dict.insert(18, ());
        dict.insert(12, ());


        avl.echo_stdout();
    }
}
