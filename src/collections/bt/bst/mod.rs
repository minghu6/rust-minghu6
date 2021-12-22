//! Binary Search Tree (BST)


pub mod avl;
pub mod rawst;


use std::{
    collections::VecDeque,
    fmt::{self, Write},
};

use either::Either;

use super::{super::{DictKey, Dictionary}, BTNode, BT};


/// LF(key) < MID(key) < RH(key)
pub trait BST<'a, K: DictKey, V>: BT<'a, K, V> {
    fn basic_insert(
        &mut self,
        new_node: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) -> bool {
        unsafe {
            let key = BSTNode::key(&*new_node);
            let approxi_node
            = (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

            if !approxi_node.is_null() && BSTNode::key(&*approxi_node) == key {
                return false;
            }

            // duplcate code for there is no guanrantee on Clone
            if approxi_node.is_null() {
                (*new_node).assign_paren(approxi_node);

                self.assign_root(new_node)
            } else if key < BSTNode::key(&*approxi_node) {
                (*new_node).assign_paren(approxi_node);

                (*approxi_node).assign_left(new_node)
            } else {
                (*new_node).assign_paren(approxi_node);

                (*approxi_node).assign_right(new_node)
            }

            true
        }
    }

    fn basic_remove(
        &mut self,
        key: &K,
    ) -> Option<*mut (dyn BSTNode<'a, K, V> + 'a)> {
        unsafe {
            let approxi_node
            = (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

            if approxi_node.is_null() {
                return None;
            }

            if BSTNode::key(&*approxi_node) != key {
                return None;
            }

            if (*approxi_node).left().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).right())
            } else if (*approxi_node).right().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).left())
            } else {
                let y = BSTNode::successor(&*approxi_node) ;
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
                writeln!(cache, "ROOT: {:?}", BSTNode::key(&*self.root_bst()))?;

                BSTNode::echo_in_mm(&*self.root_bst(), cache, action)
            }
        }
    }

    fn just_echo_stdout(&self) {
        if !self.root().is_null() {
            unsafe { BSTNode::just_echo_stdout(&*self.root_bst()) }
        }
    }
}


// /// BST Helper function for DRY.
// impl<'a, K: BSTKey, V> dyn BST<'a, K, V> + 'a {
// }


pub trait BSTNode<'a, K: DictKey, V>: BTNode<'a, K, V> {
    fn left(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe { (*BTNode::child(self, 0)).try_as_bst_mut().unwrap() }
    }
    fn right(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe{ (*BTNode::child(self, 1)).try_as_bst_mut().unwrap() }
    }
    fn key(&self) -> &K {
        BTNode::key(self, 0).unwrap()
    }
    fn value(&self) -> &V {
        BTNode::value(self, 0).unwrap()
    }
    fn value_mut(&mut self) -> &mut V {
        BTNode::value_mut(self, 0).unwrap()
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Introspection

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


    fn successor(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
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
            writeln!(cache, "Entry: {:?}", BSTNode::key(self))?;

            let mut this_level_queue: VecDeque<
                *mut (dyn BSTNode<'a, K, V> + 'a),
            > = VecDeque::new();
            this_level_queue
                .push_back(self.itself_bst_mut());
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
                            BSTNode::key(&*x),
                            BSTNode::key(&*(*x).left())
                        )?;

                        nxt_level_queue.push_back((*x).left())
                    } else {
                        writeln!(cache, "{:?} -L-> null", BSTNode::key(&*x))?;
                    }

                    if !(*x).right().is_null() {
                        writeln!(
                            cache,
                            "{:?} -R-> {:?}",
                            BSTNode::key(&*x),
                            BSTNode::key(&*(*x).right())
                        )?;

                        nxt_level_queue.push_back((*x).right())
                    } else {
                        writeln!(cache, "{:?} -R-> null", BSTNode::key(&*x))?;
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

}


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