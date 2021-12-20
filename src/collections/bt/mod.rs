#![allow(unused_imports)]

//! B-Tree alias as M-ary Tree,
//! Bayer and McCreight never explained what, if anything, the B stands for: Boeing, balanced, broad, bushy, and Bayer have been suggested.
//! McCreight has said that "the more you think about what the B in B-trees means, the better you understand B-trees.
/// According to Knuth's definition, a B-tree of order m is a tree which satisfies the following properties:
/// 1. Every node has at most m children.
/// 1. Every non-leaf node (except root) has at least ⌈m/2⌉ child nodes.
/// 1. The root has at least two children if it is not a leaf node.
/// 1. A non-leaf node with k children contains k − 1 keys.
/// 1. All leaves appear in the same level and carry no information.

use std::{ops::Bound, fmt, fmt::Write, collections::VecDeque, ptr};

use itertools::Itertools;

use self::bst::{BSTNode, BST};

use super::{DictKey, Dictionary};

pub mod bst;
pub mod b3;


/// B-Tree
pub trait BT<'a, K: DictKey, V>: Dictionary<K, V> {
    fn order(&self) -> usize;  // >= 2
    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a);
    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a));

    /// alias as transplant
    fn subtree_shift(
        &mut self,
        u: *mut (dyn BTNode<'a, K, V> + 'a),
        v: *mut (dyn BTNode<'a, K, V> + 'a),
    ) {
        unsafe {
            let u_paren = (*u).paren();

            if u_paren.is_null() {
                self.assign_root(v);
            } else {
                let u_idx = (*u_paren).index_of_child(u);
                (*u_paren).assign_child(v, u_idx);
            }

            if !v.is_null() {
                (*v).assign_paren(u_paren)
            }
        }
    }

    // ////////////////////////////////////////////////////////////////////////////
    // //// Introspection
    // fn try_as_bst(&self) -> Result<*const (dyn BST<'a, K, V> + 'a), ()>;
    // fn try_as_bst_mut(&self) -> Result<*mut (dyn BST<'a, K, V> + 'a), ()> {
    //     if let Ok(p) = self.try_as_bst() {
    //         Ok(p as *mut (dyn BST<'a, K, V> + 'a))
    //     } else {
    //         Err(())
    //     }
    // }

    fn root_bst(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe { (*self.root()).try_as_bst_mut().unwrap() }
    }

    fn minimum(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        unsafe{
            if self.root().is_null() {
                self.root()
            } else {
                (*self.root()).minimum()
            }
        }
    }


    fn maximum(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        unsafe {
            if self.root().is_null() {
                self.root()
            } else {
                (*self.root()).maximum()
            }
        }
    }


    fn search_approximately(
        &self,
        income_key: &K,
    ) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        if !self.root().is_null() {
            unsafe { (*self.root()).search_approximately(income_key) }
        } else {
            self.root()
        }
    }

    /// BFS Echo
    fn echo_in_mm(
        &self,
        cache: &mut String,
        action: fn(
            *mut (dyn BTNode<'a, K, V> + 'a),
            &mut String,
        ) -> fmt::Result,
    ) -> fmt::Result {
        if self.root().is_null() {
            writeln!(cache, "ROOT: null")
        } else {
            unsafe {
                writeln!(cache, "ROOT: {:?}", (*self.root()).format_keys())?;

                (*self.root()).echo_in_mm(cache, action)
            }
        }
    }

    // fn bfs_do(
    //     &self,
    //     action: fn(
    //         *mut (dyn BSTNode<'a, K, V> + 'a),
    //     )
    // ) {
    //     if !self.root().is_null() {
    //         unsafe{ (*self.root_bst()).bfs_do(action) }
    //     }

    // }

    fn just_echo_stdout(&self) {
        if !self.root().is_null() {
            unsafe { (*self.root()).just_echo_stdout() }
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

}


/// B-Tree Node
pub trait BTNode<'a, K: DictKey, V> {

    ////////////////////////////////////////////////////////////////////////////
    //// Introspection

    fn itself(&self) -> *const (dyn BTNode<'a, K, V> + 'a);
    fn itself_mut(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.itself() as *mut (dyn BTNode<'a, K, V> + 'a)
    }
    fn null(&self) -> *const (dyn BTNode<'a, K, V> + 'a);
    fn null_mut(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.null() as *mut (dyn BTNode<'a, K, V> + 'a)
    }


    fn try_as_bst(&self) -> Result<*const (dyn BSTNode<'a, K, V> + 'a), ()>;
    fn try_as_bst_mut(&self) -> Result<*mut (dyn BSTNode<'a, K, V> + 'a), ()> {
        if let Ok(p) = self.try_as_bst() {
            Ok(p as *mut (dyn BSTNode<'a, K, V> + 'a))
        } else {
            Err(())
        }
    }
    fn itself_bst(&self) -> *const (dyn BSTNode<'a, K, V> + 'a) {
        self.try_as_bst().unwrap()
    }
    fn itself_bst_mut(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.try_as_bst_mut().unwrap()
    }

    fn order(&self) -> usize;  // >= 2

    /// 0 <= idx <= order, child(order) is temporary case.
    fn child(&self, idx: usize) -> *mut (dyn BTNode<'a, K, V> + 'a);
    fn child_first(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.child(0)
    }
    fn child_last(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        for i in 0..self.order() - 1 {
            // as *const () just to ignore the vtable variant from the fat pointer
            if self.child(i + 1).is_null() {
                return self.child(i);
            }

        }

        self.child(self.order() - 1)
    }

    /// 0 <= idx <= order, child(order) is temporary case.
    fn assign_child(&mut self, child: *mut (dyn BTNode<'a, K, V> + 'a), idx: usize);
    fn assign_value(&mut self, value: V, idx: usize);
    fn assign_paren(&mut self, paren: *mut (dyn BTNode<'a, K, V> + 'a));

    fn paren(&self) -> *mut (dyn BTNode<'a, K, V> + 'a);
    fn paren_bst(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        unsafe { (*self.paren()).try_as_bst_mut().unwrap() }
    }

    fn key(&self, idx: usize) -> Option<&K>;

    fn index_of_child(&self, child: *mut (dyn BTNode<'a, K, V> + 'a)) -> usize {
        for i in 0..self.order() {
            // as *const () just to ignore the vtable variant from the fat pointer
            if self.child(i) as *const () == child as *const () {
                return i;
            }

        }

        unreachable!()
    }

    /// If this node contains key (exclude the subtree)
    #[inline]
    fn node_contains(&self, key: &K) -> bool {
        for i in 0..self.order() {
            let key_opt = self.key(i);
            if key_opt.is_some() && key_opt.unwrap() == key {
                return true;
            }
        }

        false
    }

    /// How many key-values does this node contains?
    fn node_size(&self) -> usize {
        for i in 0..self.order() {
            if self.key(i).is_none() {
                return i;  // i must be greater than one in this case.
            }
        }

        self.order()
    }

    fn node_is_overfilled(&self) -> bool {
        self.node_size() >= self.order()
    }


    fn value(&self, idx: usize) -> &V;
    fn value_mut(&mut self, idx: usize) -> &mut V;

    fn height(&self) -> i32;

    #[inline]
    fn calc_height(&self) -> i32 {
        (0..self.order())
        .into_iter()
        .map(|i| {
            if self.child(i).is_null() {
                -1
            } else {
                unsafe { (*self.child(i)).calc_height() }
            }
        }).max().unwrap() + 1

    }


    fn total(&self) -> usize {
        let mut total = 1;

        for i in 0..self.order() {
            let child = self.child(i);

            if !child.is_null() {
                unsafe{ total += (*child).total() + 1; }
            }
        }

        total
    }


    fn minimum(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        while unsafe { !(*x).child_first().is_null() } {
            unsafe { x = (*x).child_first() }
        }

        x
    }


    fn maximum(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        while unsafe { !(*x).child_last().is_null() } {
            unsafe { x = (*x).child_last() }
        }

        x
    }


    #[inline]
    fn search_approximately(
        &self,
        income_key: &K,
    ) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        let mut y = self.null_mut();
        let mut x = self.itself_mut();

        unsafe {
            while !x.is_null() {
                y = x;

                if (*x).node_contains(income_key) {
                    break;
                }

                let mut i = 0;
                let mut encountered = false;
                while i < self.order() {
                    if let Some(key) = (*x).key(i) {
                        if income_key < key {
                            x = (*x).child(i);
                            encountered = true;

                            break;
                        }
                    } else {
                        break;
                    }

                    i += 1;
                }

                if !encountered {
                    x = (*x).child_last();
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

    fn format_keys(&self) -> String {
        let mut keys_s = vec![];

        for i in 0..self.order() {
            let key_s = if let Some(key) = self.key(i) {
               format!("{:?}", key)
            } else {
                break;
            };

            keys_s.push(key_s)
        }

        format!("({})", keys_s.join(", "))
    }

    /// BFS Echo
    fn echo_in_mm(
        &self,
        cache: &mut String,
        action: fn(
            *mut (dyn BTNode<'a, K, V> + 'a),
            &mut String,
        ) -> fmt::Result,
    ) -> fmt::Result {
        unsafe {
            writeln!(cache, "Entry: {}", self.format_keys())?;

            let mut this_level_queue: VecDeque<
                *mut (dyn BTNode<'a, K, V> + 'a),
            > = VecDeque::new();

            this_level_queue
                .push_back(self.itself_mut());
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
                    *mut (dyn BTNode<'a, K, V> + 'a),
                > = VecDeque::new();

                while !this_level_queue.is_empty() {
                    let x = this_level_queue.pop_front().unwrap();


                    action(x, cache)?;

                    writeln!(cache, "{}", (*x).format_keys() )?;
                    for i in 0..self.order() {
                        let child = (*x).child(i);

                        if !child.is_null() {
                            writeln!(
                                cache,
                                "{} -({})-> {}",
                                "  |",
                                i,
                                (*child).format_keys(),
                            )?;

                            nxt_level_queue.push_back(child)
                        } else {
                            writeln!(cache, "{} -({})-> null", "  |", i)?;
                        }
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
mod tests {

    #[test]
    fn test_bt() {
        use super::bst::tests::test_bst;

        test_bst()
    }
}

