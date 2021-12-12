use std::fmt::Write;
use std::fmt::{self, Display};
use std::ptr::null_mut;

use either::Either;

use super::{BSTKey, BSTNode, BST};
use crate::collections::Dictionary;


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

    fn into_value(self) -> V {
        unsafe { *Box::from_raw(self.value) }
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let h_lf = if !self.left.is_null() {
                (*self.left).height()
            } else {
                -1
            };

            let h_rh = if !self.right.is_null() {
                (*self.right).height()
            } else {
                -1
            };

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
    }

    pub fn echo_in_mm_avl(&self, cache: &mut String) -> fmt::Result {
        self.echo_in_mm(cache, |x, cache| {
            unsafe {
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
            }

            Ok(())
        })
    }

    pub fn echo_stdout_avl(&self) {
        let mut cache = String::new();

        self.echo_in_mm_avl(&mut cache).unwrap();

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

    fn itself_mut(&mut self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self as *mut Self
    }

    fn itself(&self) -> *const (dyn BSTNode<'a, K, V> + 'a) {
        self as *const Self
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

    pub fn echo_stdout_avl(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout_avl() }
        }
    }

    fn search_approximately(
        &self,
        income_key: &K,
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut y =
            null_mut::<AVLNode<K, V>>() as *mut (dyn BSTNode<'a, K, V> + 'a);

        let mut x = self.root() as *mut (dyn BSTNode<'a, K, V> + 'a);

        unsafe {
            while !x.is_null() {
                y = x;
                if income_key < (*x).key() {
                    x = (*x).left();
                } else if income_key > (*x).key() {
                    x = (*x).right();
                } else {
                    return x;
                }
            }
        }

        y
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
    unsafe fn rorate(
        x: *mut AVLNode<K, V>,
        z: *mut AVLNode<K, V>,
        rotation: Either<(), ()>,
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
        (*x).assign_paren(y);

        /* Adjust BF */
        // h(t1) = h(t4) = max(h(t2), h(t3))
        if (*y).bf == BF::Z {
            (*x).bf = BF::Z;
            (*z).bf = BF::Z;
        } else {
            // double symmetry
            if snd_rotation.is_left() {
                if (*y).bf == BF::P1 {  // t3 is heigher
                    (*x).bf = BF::N1;
                    (*z).bf = BF::Z;
                } else {
                    (*x).bf = BF::Z;
                    (*z).bf = BF::P1;
                }
            } else {
                if (*y).bf == BF::N1 {  // t3 is heigher
                    (*x).bf = BF::P1;
                    (*z).bf = BF::Z;
                } else {
                    (*x).bf = BF::Z;
                    (*z).bf = BF::N1;
                }
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
                    Self::rorate(x, z, fst_rotation)
                } else {
                    Self::double_rotate(x, z, snd_rotation)
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
}


impl<'a, K: BSTKey + 'a, V: 'a> Dictionary<K, V> for AVL<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        let approxi_node = self.search_approximately(&key);

        unsafe {
            if !approxi_node.is_null() && *(*approxi_node).key() == key {
                return false;
            }

            let new_node;
            // duplcate code for there is no guanrantee on Clone
            if approxi_node.is_null() {
                new_node = AVLNode::new(key, value);
                (*new_node).assign_paren(approxi_node);

                self.assign_root(new_node)
            } else if key < *(*approxi_node).key() {
                new_node = AVLNode::new(key, value);
                (*new_node).assign_paren(approxi_node);

                (*approxi_node).assign_left(new_node)
            } else {
                new_node = AVLNode::new(key, value);
                (*new_node).assign_paren(approxi_node);

                (*approxi_node).assign_right(new_node)
            }

            self.insert_retracing(new_node);

            true
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let approxi_node = self.search_approximately(&key);
        if approxi_node.is_null() {
            return None;
        }

        unsafe {
            if (*approxi_node).key() != key {
                return None;
            }

            if (*approxi_node).left().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).right())
            } else if (*approxi_node).right().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).left())
            } else {
                let y = (*approxi_node).successor();

                if (*y).paren() != approxi_node {
                    self.subtree_shift(y, (*y).right());
                    (*y).assign_right((*approxi_node).right());
                    (*(*y).right()).assign_paren(y);
                }
                self.subtree_shift(approxi_node, y);
                (*y).assign_left((*approxi_node).left());
                (*(*y).left()).assign_paren(y);
            }

            let node = Box::from_raw(approxi_node as *mut AVLNode<K, V>);

            let v = node.into_value();

            Some(v)
        }
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        let app_node = self.search_approximately(key);

        unsafe {
            if app_node.is_null() {
                false
            } else if (*app_node).key() == key {
                (*app_node).assign_value(value);
                true
            } else {
                false
            }
        }
    }

    fn lookup(&self, income_key: &K) -> Option<&V> {
        let app_node = self.search_approximately(income_key);

        unsafe {
            if app_node.is_null() {
                None
            } else if (*app_node).key() == income_key {
                Some((*app_node).value())
            } else {
                None
            }
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
    fn itself(&mut self) -> *mut (dyn BST<'a, K, V> + 'a) {
        self as *mut Self
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
        dict.self_validate().unwrap();

        dict.insert(22, ());
        dict.self_validate().unwrap();


        assert!(dict.lookup(&10).is_some());
        assert!(dict.lookup(&5).is_some());
        assert!(dict.lookup(&12).is_some());
        assert!(dict.lookup(&13).is_some());
        assert!(dict.lookup(&14).is_some());
        assert!(dict.lookup(&18).is_some());
        assert!(dict.lookup(&7).is_some());
        assert!(dict.lookup(&9).is_some());
        assert!(dict.lookup(&11).is_some());
        assert!(dict.lookup(&22).is_some());


        // assert!(dict.remove(&10).is_some());
        // assert!(dict.lookup(&10).is_none());

        // assert!(dict.lookup(&5).is_some());
        // assert!(dict.remove(&5).is_some());
        // assert!(dict.lookup(&5).is_none());

        // assert!(dict.lookup(&12).is_some());
        // assert!(dict.remove(&12).is_some());

        // assert!(dict.remove(&13).is_some());
        // assert!(dict.remove(&14).is_some());
        // assert!(dict.remove(&18).is_some());
        // assert!(dict.remove(&7).is_some());
        // assert!(dict.remove(&9).is_some());
        // assert!(dict.remove(&11).is_some());
        // assert!(dict.remove(&22).is_some());

        avl.echo_stdout_avl();
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


        avl.echo_stdout_avl();
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


        avl.echo_stdout_avl();
    }
}
