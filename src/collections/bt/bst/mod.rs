//! Binary Search Tree (BST)


pub mod avl;


use std::{
    collections::VecDeque,
    fmt::{self, Write},
};

use either::Either;

use super::{super::{DictKey, Dictionary}, BTNode};


/// LF(key) < MID(key) < RH(key)
pub trait BST<'a, K: DictKey, V>: Dictionary<K, V> {
    fn itself(&self) -> *const (dyn BST<'a, K, V> + 'a);
    fn itself_mut(&self) -> *mut (dyn BST<'a, K, V> + 'a) {
        self.itself() as *mut (dyn BST<'a, K, V> + 'a)
    }

    fn root(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a);

    fn assign_root(&mut self, root: *mut (dyn BSTNode<'a, K, V> + 'a));

    /// alias as transplant
    fn subtree_shift(
        &mut self,
        u: *mut (dyn BSTNode<'a, K, V> + 'a),
        v: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) {
        unsafe {
            if (*u).paren().is_null() {
                self.assign_root(v)
            } else if u == (*(*u).paren_bst()).left() {
                (*(*u).paren_bst()).assign_left(v);
            } else {
                (*(*u).paren_bst()).assign_right(v);
            }

            if !v.is_null() {
                (*v).assign_paren((*u).paren_bst())
            }
        }
    }

    fn calc_height(&self) -> i32 {
        if self.root().is_null() {
            return -1;
        }

        unsafe { (*self.root()).calc_height() }
    }

    fn height(&self) -> i32 {
        if self.root().is_null() {
            return -1;
        }

        unsafe { (*self.root()).height() }
    }

    fn total(&self) -> usize {
        if self.root().is_null() {
            0
        } else {
            unsafe { (*self.root()).total() }
        }
    }

    fn search_approximately(
        &self,
        income_key: &K,
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        if !self.root().is_null() {
            unsafe { (*self.root()).search_approximately(income_key) }
        } else {
            self.root()
        }
    }

    fn basic_insert(
        &mut self,
        new_node: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) -> bool {
        unsafe {
            let key = BSTNode::key(&*new_node);
            let approxi_node = self.search_approximately(&key);

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

    fn basic_modify(&mut self, key: &K, value: V) -> bool {
        let app_node = self.search_approximately(key);

        unsafe {
            if app_node.is_null() {
                false
            } else if BSTNode::key(&*app_node) == key {
                (*app_node).assign_value(value);
                true
            } else {
                false
            }
        }
    }

    fn basic_lookup(
        &self,
        income_key: &K,
    ) -> Option<*mut (dyn BSTNode<'a, K, V> + 'a)> {
        let app_node = self.search_approximately(income_key);

        unsafe {
            if app_node.is_null() {
                None
            } else if BSTNode::key(&*app_node) == income_key {
                Some(app_node)
            } else {
                None
            }
        }
    }

    fn basic_remove(
        &mut self,
        key: &K,
    ) -> Option<*mut (dyn BSTNode<'a, K, V> + 'a)> {
        let approxi_node = self.search_approximately(&key);
        if approxi_node.is_null() {
            return None;
        }

        unsafe {
            if BSTNode::key(&*approxi_node) != key {
                return None;
            }

            if (*approxi_node).left().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).right())
            } else if (*approxi_node).right().is_null() {
                self.subtree_shift(approxi_node, (*approxi_node).left())
            } else {
                let y = (*approxi_node).successor() ;
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
                writeln!(cache, "ROOT: {:?}", BSTNode::key(&*self.root()))?;

                (*self.root()).echo_in_mm(cache, action)
            }
        }
    }

    fn bfs_do(
        &self,
        action: fn(
            *mut (dyn BSTNode<'a, K, V> + 'a),
        )
    ) {
        if !self.root().is_null() {
            unsafe{ (*self.root()).bfs_do(action) }
        }

    }

    fn just_echo_stdout(&self) {
        if !self.root().is_null() {
            unsafe { (*self.root()).just_echo_stdout() }
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
        BTNode::key(self, 0)
    }
    fn value(&self) -> &V {
        BTNode::value(self, 0)
    }
    fn value_mut(&self) -> &mut V {
        BTNode::value_mut(self, 0)
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Introspection

    fn itself(&self) -> *const (dyn BSTNode<'a, K, V> + 'a);
    fn itself_mut(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.itself() as *mut (dyn BSTNode<'a, K, V> + 'a)
    }

    fn null(&self) -> *const (dyn BSTNode<'a, K, V> + 'a);
    fn null_mut(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.null() as *mut (dyn BSTNode<'a, K, V> + 'a)
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

    fn child_height(&self, direction: Either<(), ()>) -> i32 {
        if BSTNode::child(self, direction).is_null() {
            -1
        } else {
            unsafe { (*BSTNode::child(self, direction)).height() }
        }
    }

    fn assign_left(&mut self, left: *mut (dyn BSTNode<'a, K, V> + 'a));
    fn assign_right(&mut self, right: *mut (dyn BSTNode<'a, K, V> + 'a));
    fn assign_paren(&mut self, paren: *mut (dyn BSTNode<'a, K, V> + 'a));
    fn assign_value(&mut self, value: V);

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


    fn minimum(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        while unsafe { !(*x).left().is_null() } {
            unsafe { x = (*x).left() }
        }

        x
    }

    fn maximum(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        while unsafe { !(*x).right().is_null() } {
            unsafe { x = (*x).right() }
        }

        x
    }

    fn successor(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        unsafe {
            if !(*x).right().is_null() {
                return (*(*x).right()).minimum();
            }

            let mut y = (*(*x).paren()).try_as_bst_mut().unwrap();

            while !y.is_null() && x == (*y).right() {
                x = y;
                y = (*(*y).paren()).try_as_bst_mut().unwrap();
            }

            y
        }
    }

    fn search_approximately(
        &self,
        income_key: &K,
    ) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut y = self.null_mut();
        let mut x = self.itself_mut();

        unsafe {
            while !x.is_null() {
                y = x;
                if income_key < BSTNode::key(&*x) {
                    x = (*x).left();
                } else if income_key > BSTNode::key(&*x) {
                    x = (*x).right();
                } else {
                    return x;
                }
            }
        }

        y
    }


    fn just_echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache, |_, _| Ok(())).unwrap();

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
                .push_back(self.itself() as *mut (dyn BSTNode<'a, K, V> + 'a));
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


    fn bfs_do(
        &self,
        action: fn(
            *mut (dyn BSTNode<'a, K, V> + 'a),
        )
    ) {
        let mut queue= VecDeque::new();

        queue.push_back(self.itself() as *mut (dyn BSTNode<'a, K, V> + 'a));
        while !queue.is_empty() {
            let x = queue.pop_front().unwrap();

            action(x);

            unsafe {
                if !(*x).left().is_null() {
                    queue.push_back((*x).left());
                }

                if !(*x).right().is_null() {
                    queue.push_back((*x).right());
                }
            }

        }
    }

    fn total(&self) -> usize {
        let mut total = 1;

        if !self.left().is_null() {
            total += unsafe{ (*self.left()).total() };
        }

        if !self.right().is_null() {
            total += unsafe{ (*self.right()).total() };
        }

        total
    }
}