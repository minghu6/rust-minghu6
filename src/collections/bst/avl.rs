use std::ptr::null_mut;

use super::{BSTKey, BSTNode, BST};
use crate::collections::{Dictionary, BT};


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




////////////////////////////////////////////////////////////////////////////////
//// Implement

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

    fn self_validate(&self) {

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

    // fn search_approximately_mut(
    //     &mut self,
    //     income_key: &K,
    // ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
    //     let mut y =
    //         null_mut::<AVLNode<K, V>>() as *mut (dyn BSTNode<'a, K, V> + 'a);

    //     let mut x = self.root() as *mut (dyn BSTNode<'a, K, V> + 'a);

    //     unsafe {
    //         while !x.is_null() {
    //             y = x;
    //             if income_key < (*x).key() {
    //                 x = (*x).left();
    //             } else {
    //                 x = (*x).right();
    //             }
    //         }
    //     }

    //     y
    // }
}


impl<'a, K: BSTKey + 'a, V: 'a> Dictionary<K, V> for AVL<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        let approxi_node = self.search_approximately(&key);

        unsafe {
            if !approxi_node.is_null() && *(*approxi_node).key() == key {
                return false;
            }

            // duplcate code for there is no guanrantee on Clone
            if approxi_node.is_null() {
                let new_node = AVLNode::new(key, value);
                (*new_node).assign_paren(approxi_node);

                self.assign_root(new_node)
            } else if key < *(*approxi_node).key() {
                let new_node = AVLNode::new(key, value);
                (*new_node).assign_paren(approxi_node);

                (*approxi_node).assign_left(new_node)
            } else {
                let new_node = AVLNode::new(key, value);
                (*new_node).assign_paren(approxi_node);

                (*approxi_node).assign_right(new_node)
            }

            // insert retracing

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
                let y_key = (*y).key();
                let y_paren_key = (*(*y).paren()).key();

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

    fn self_validate(&self) {

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
        collections::{Dictionary, bst::BST},
        test::dict::{DictProvider, Inode, InodeProvider},
    };

    #[test]
    fn test_bst_randomdata() {
        let mut dict = AVL::<u16, Inode>::new();

        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u16, Inode>).test_dict(&mut dict);
    }

    #[test]
    fn test_bst_fixeddata() {
        let mut avl = AVL::<i32, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<i32, ()>;

        dict.insert(10, ());
        dict.self_validate();

        dict.insert(5, ());
        dict.self_validate();

        dict.insert(12, ());
        dict.self_validate();

        dict.insert(13, ());
        dict.self_validate();

        dict.insert(14, ());
        dict.self_validate();

        dict.insert(18, ());
        dict.self_validate();

        dict.insert(7, ());
        dict.self_validate();

        dict.insert(9, ());
        dict.self_validate();

        dict.insert(11, ());
        dict.self_validate();

        dict.insert(22, ());
        dict.self_validate();


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


        assert!(dict.remove(&10).is_some());
        assert!(dict.lookup(&10).is_none());

        assert!(dict.lookup(&5).is_some());
        assert!(dict.remove(&5).is_some());
        assert!(dict.lookup(&5).is_none());

        assert!(dict.lookup(&12).is_some());
        assert!(dict.remove(&12).is_some());

        assert!(dict.remove(&13).is_some());
        assert!(dict.remove(&14).is_some());
        assert!(dict.remove(&18).is_some());
        assert!(dict.remove(&7).is_some());
        assert!(dict.remove(&9).is_some());
        assert!(dict.remove(&11).is_some());
        assert!(dict.remove(&22).is_some());



        (&mut avl as &mut dyn BST<i32, ()>).echo_stdout();

    }
}
