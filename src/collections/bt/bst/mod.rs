//! Binary Search Tree (BST)


pub mod avl;
pub mod rawst;
pub mod rb;
pub mod llrb;
pub mod aa;
pub mod treap;
pub mod splay;


use std::{
    collections::VecDeque,
    fmt::{self, Write},
};

use either::Either;

use super::{
    super::{DictKey, Dictionary},
    BTNode, BT,
};
use crate::etc::Reverse;


/// LF(key) < MID(key) < RH(key)
pub trait BST<'a, K: DictKey + 'a, V: 'a>: BT<'a, K, V> {
    fn basic_insert(
        &mut self,
        new_node: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) -> bool {
        unsafe {
            let key = BSTNode::key_bst(&*new_node);
            let approxi_node =
                (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

            if !approxi_node.is_null() && BSTNode::key_bst(&*approxi_node) == key {
                return false;
            }

            // duplcate code for there is no guanrantee on Clone
            if approxi_node.is_null() {
                (*new_node).assign_paren(approxi_node);

                self.assign_root(new_node)
            } else if key < BSTNode::key_bst(&*approxi_node) {
                (*approxi_node).connect_left(new_node)
            } else {
                (*approxi_node).connect_right(new_node)
            }

            true
        }
    }

    fn basic_remove(
        &mut self,
        key: &K,
    ) -> Option<*mut (dyn BSTNode<'a, K, V> + 'a)> {
        unsafe {
            let approxi_node =
                (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

            if approxi_node.is_null() {
                return None;
            }

            if BSTNode::key_bst(&*approxi_node) != key {
                return None;
            }

            if (*approxi_node).left().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).right())
            } else if (*approxi_node).right().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).left())
            } else {
                let y = BSTNode::successor_bst(&*approxi_node);
                // y should be leaf.

                if (*y).paren_bst() != approxi_node {
                    self.subtree_shift(y, (*y).right());
                    (*y).assign_right((*approxi_node).right());
                    (*(*y).right()).assign_paren(y);
                }
                self.subtree_shift(approxi_node, y);
                (*y).assign_left((*approxi_node).left());
                (*(*y).left()).assign_paren(y);
            }

            Some(approxi_node)
        }
    }

    unsafe fn rotate_cleanup(
        &mut self,
        x: *mut (dyn BSTNode<'a, K, V> + 'a),
        z: *mut (dyn BSTNode<'a, K, V> + 'a),
    );

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
        x: *mut (dyn BSTNode<'a, K, V> + 'a),
        rotation: Either<(), ()>, // rotate to left = from right rotation
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        #[cfg(test)] {
            ROTATE_NUM += 1;
        }

        let z = if rotation.is_left() {
            (*x).right()
        } else {
            (*x).left()
        };

        let t23 = if rotation.is_left() {
            (*z).left()
        } else {
            (*z).right()
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

        self.rotate_cleanup(x, z);

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
        x: *mut (dyn BSTNode<'a, K, V> + 'a),
        snd_rotation: Either<(), ()>,
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let z = if snd_rotation.is_left() {
            (*x).right()
        } else {
            (*x).left()
        };

        self.rotate(z, snd_rotation.reverse());
        self.rotate(x, snd_rotation)

        // // Manualy Implements
        // /* FIRST ROTATION */
        // // z is by 2 higher than its sibing(t1)
        // // y is by 1 higher than its sibling(t4) (thereis shouldn't be empty)
        // let z = if snd_rotation.is_left() {
        //     (*x).right
        // } else {
        //     (*x).left
        // };

        // let y = if snd_rotation.is_left() {
        //     (*z).left
        // } else {
        //     (*z).right
        // };

        // let (t2, t3) = if snd_rotation.is_left() {
        //     ((*y).left, (*y).right)
        // } else {
        //     ((*y).right, (*y).left)
        // };

        // if !t3.is_null() {
        //     (*t3).assign_paren(z);
        // }
        // (*z).assign_paren(y);

        // if snd_rotation.is_left() {
        //     (*z).assign_left(t3);
        //     (*y).assign_right(z);
        // } else {
        //     (*z).assign_right(t3);
        //     (*y).assign_left(z);
        // }

        // // skip x-R->z => x-R->y for it would be overrided by second rotation

        // /* SECOND ROTATION */
        // if snd_rotation.is_left() {
        //     (*x).assign_right(t2);
        //     (*y).assign_left(x);
        // } else {
        //     (*x).assign_left(t2);
        //     (*y).assign_right(x);
        // }
        // if !t2.is_null() {
        //     (*t2).assign_paren(x);
        // }

        // self.subtree_shift(x, y);
        // (*x).assign_paren(y);

        // y
    }

    /// BFS Echo
    fn echo_in_mm(
        &self,
        cache: &mut String,
        action: fn(
            *mut (dyn BSTNode<'a, K, V> + 'a),
            &mut String,
        ) -> fmt::Result,
    ) -> fmt::Result {
        if self.root().is_null() {
            writeln!(cache, "ROOT: null")
        } else {
            unsafe {
                writeln!(
                    cache,
                    "ROOT: {:?}",
                    BSTNode::key_bst(&*self.root_bst())
                )?;

                BSTNode::echo_in_mm(&*self.root_bst(), cache, action)
            }
        }
    }

    fn just_echo_stdout(&self) {
        if !self.root().is_null() {
            unsafe { BSTNode::just_echo_stdout(&*self.root_bst()) }
        }
    }


    fn nodes_iter(&'a self) -> Box<dyn Iterator<Item = *mut (dyn BSTNode<'a, K, V> + 'a)> + 'a> {
        if self.root_bst().is_null() {
            return box std::iter::from_fn(|| None);
        }

        unsafe {
            (*self.root_bst()).nodes_iter()
        }

    }
}


pub trait BSTNode<'a, K: DictKey + 'a, V: 'a>: BTNode<'a, K, V> {
    fn left(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe { (*BTNode::child(self, 0)).try_as_bst_mut().unwrap() }
    }
    fn right(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe { (*BTNode::child(self, 1)).try_as_bst_mut().unwrap() }
    }
    fn sibling(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe {
            let paren = self.paren_bst();
            debug_assert!(!paren.is_null());

            BSTNode::child(&*paren, (*self).dir().reverse())
        }
    }

    fn uncle(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe {
            let paren = self.paren_bst();
            debug_assert!(!paren.is_null());

            (*paren).sibling()

        }
    }

    fn key_bst(&self) -> &K {
        BTNode::key(self, 0).unwrap()
    }
    fn value_bst(&self) -> &V {
        BTNode::value(self, 0).unwrap()
    }
    fn value_bst_mut(&mut self) -> &mut V {
        BTNode::value_mut(self, 0).unwrap()
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Introspection

    fn dir(&self) -> Either<(), ()> {
        unsafe {
            debug_assert!(!self.paren().is_null());

            if (*self.paren()).index_of_child(self.itself_mut()) == 0 {
                Either::Left(())
            } else {
                Either::Right(())
            }
        }
    }

    fn child(
        &self,
        direction: Either<(), ()>,
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        if direction.is_left() {
            self.left()
        } else {
            self.right()
        }
    }

    fn child_bst(
        &self,
        direction: Either<(), ()>,
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        if direction.is_left() {
            self.left()
        } else {
            self.right()
        }
    }

    fn child_height(&self, direction: Either<(), ()>) -> i32 {
        if BSTNode::child(self, direction).is_null() {
            -1
        } else {
            unsafe { (*BSTNode::child(self, direction)).height() }
        }
    }

    fn assign_left(&mut self, left: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.assign_child(left, 0)
    }

    fn assign_right(&mut self, right: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.assign_child(right, 1)
    }

    fn connect_left(&mut self, child: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.connect_child(child, 0)
    }

    fn connect_right(&mut self, child: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.connect_child(child, 1)
    }

    fn calc_left_height(&self) -> i32 {
        if !self.left().is_null() {
            unsafe { (*self.left()).calc_height() }
        } else {
            -1
        }
    }

    fn calc_right_height(&self) -> i32 {
        if !self.right().is_null() {
            unsafe { (*self.right()).calc_height() }
        } else {
            -1
        }
    }

    fn left_height(&self) -> i32 {
        if !self.left().is_null() {
            unsafe { (*self.left()).height() }
        } else {
            -1
        }
    }

    fn right_height(&self) -> i32 {
        if !self.right().is_null() {
            unsafe { (*self.right()).height() }
        } else {
            -1
        }
    }

    fn precessor_bst(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_bst_mut();

        unsafe {
            if !(*x).left().is_null() {
                return (*(*(*x).left()).maximum()).try_as_bst_mut().unwrap();
            }

            let mut y = (*x).paren_bst();

            while !y.is_null() && x == (*y).left() {
                x = y;
                y = (*y).paren_bst();
            }

            y
        }
    }

    fn successor_bst(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_bst_mut();

        unsafe {
            if !(*x).right().is_null() {
                return (*(*(*x).right()).minimum()).try_as_bst_mut().unwrap();
            }

            let mut y = (*x).paren_bst();

            while !y.is_null() && x == (*y).right() {
                x = y;
                y = (*y).paren_bst();
            }

            y
        }
    }

    // fn swap_with_successor_until_null(
    //     &mut self,
    // ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
    //     unsafe {
    //         let mut x = self.itself_bst_mut();
    //         let mut p;

    //         loop {
    //             p = x;
    //             x = BSTNode::successor(&*x);

    //             if x.is_null() {
    //                 break;
    //             }

    //             (*p).swap_with(x);
    //         }

    //         p
    //     }
    // }


    /// Just swap key and value
    unsafe fn swap_with(&mut self, other: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        debug_assert!(!other.is_null());

        let tmp_key = (*other).key_ptr(0);
        let tmp_val = (*other).val_ptr(0);

        (*other).assign_key_ptr(0, self.key_ptr(0));
        (*other).assign_val_ptr(0, self.val_ptr(0));

        self.assign_key_ptr(0, tmp_key);
        self.assign_val_ptr(0, tmp_val);
    }

    fn just_echo_stdout(&self) {
        let mut cache = String::new();

        BSTNode::echo_in_mm(self, &mut cache, |_, _| Ok(())).unwrap();

        println!("{}", cache);
    }

    /// BFS Echo
    fn echo_in_mm(
        &self,
        cache: &mut String,
        action: fn(
            *mut (dyn BSTNode<'a, K, V> + 'a),
            &mut String,
        ) -> fmt::Result,
    ) -> fmt::Result {
        unsafe {
            writeln!(cache, "Entry: {:?}", BSTNode::key_bst(self))?;

            let mut this_level_queue: VecDeque<
                *mut (dyn BSTNode<'a, K, V> + 'a),
            > = VecDeque::new();
            this_level_queue.push_back(self.itself_bst_mut());
            let mut level = 0;

            while !this_level_queue.is_empty() {
                writeln!(cache)?;
                writeln!(
                    cache,
                    "############ Level: {} #############",
                    level
                )?;
                writeln!(cache)?;

                let mut nxt_level_queue: VecDeque<
                    *mut (dyn BSTNode<'a, K, V> + 'a),
                > = VecDeque::new();

                while !this_level_queue.is_empty() {
                    let x = this_level_queue.pop_front().unwrap();

                    // writeln!(cache, "{:?}", (*x).key() )?;

                    action(x, cache)?;

                    if !(*x).left().is_null() {
                        writeln!(
                            cache,
                            "{:?} -L-> {:?}",
                            BSTNode::key_bst(&*x),
                            BSTNode::key_bst(&*(*x).left())
                        )?;

                        nxt_level_queue.push_back((*x).left())
                    } else {
                        writeln!(cache, "{:?} -L-> null", BSTNode::key_bst(&*x))?;
                    }

                    if !(*x).right().is_null() {
                        writeln!(
                            cache,
                            "{:?} -R-> {:?}",
                            BSTNode::key_bst(&*x),
                            BSTNode::key_bst(&*(*x).right())
                        )?;

                        nxt_level_queue.push_back((*x).right())
                    } else {
                        writeln!(cache, "{:?} -R-> null", BSTNode::key_bst(&*x))?;
                    }

                    writeln!(cache)?;
                }

                this_level_queue = nxt_level_queue;
                level += 1;
            }

            writeln!(cache, "{}", "------------- end --------------")?;
            writeln!(cache)?;
        }


        Ok(())
    }


    // Infix order (DFS)
    fn nodes_iter(&'a self) -> Box<dyn Iterator<Item = *mut (dyn BSTNode<'a, K, V> + 'a)> + 'a> {
        unsafe {
            let mut x = (*self.minimum()).try_as_bst_mut().unwrap();

            box std::iter::from_fn(move || {
                if !x.is_null() {
                    let prev = x;
                    x = (*x).successor_bst();
                    Some(prev)
                } else {
                    None
                }

            })
        }
    }

}

#[allow(unused)]
pub(crate) static mut ROTATE_NUM: usize = 0;



#[cfg(test)]
pub(crate) mod tests {

    #[test]
    pub(crate) fn test_bst() {
        use super::avl::tests::test_avl_randomdata;
        use super::rawst::tests::test_rawst_randomdata;

        test_avl_randomdata();
        test_rawst_randomdata();
    }
}
