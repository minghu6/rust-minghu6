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

    fn basic_lookup(
        &self,
        income_key: &K,
    ) -> Option<&V> {
        let res = self.search_approximately(income_key);

        if res.is_null() {
            None
        } else {
            unsafe {
                // println!("{:?}", (*res).format_keys());

                if let Some(idx) = (*res).find_pos_of_key(income_key) {
                    Some(&*(*res).val_ptr(idx))
                } else {
                    None
                }
            }
        }
    }

    fn basic_lookup_mut(
        &mut self,
        income_key: &K,
    ) -> Option<&mut V> {
        let res = self.search_approximately(income_key);

        if res.is_null() {
            None
        } else {
            unsafe {
                // println!("{:?}", (*res).format_keys());

                if let Some(idx) = (*res).find_pos_of_key(income_key) {
                    Some(&mut *(*res).val_ptr(idx))
                } else {
                    None
                }
            }
        }
    }

    fn basic_modify(&mut self, key: &K, value: V) -> bool {
        unsafe {
            let app_node
            = (*self.search_approximately(key)).try_as_bst_mut().unwrap();

            if app_node.is_null() {
                false
            } else if let Some(idx) = (*app_node).find_pos_of_key(key) {
                (*app_node).assign_value(value, idx);
                true
            } else {
                false
            }
        }
    }

    fn bfs_do(
        &self,
        action: fn(
            *mut (dyn BTNode<'a, K, V> + 'a),
        )
    ) {
        if !self.root().is_null() {
            unsafe{ (*self.root()).bfs_do(action) }
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

    fn key(&self, idx: usize) -> Option<&K> {
        if !self.key_ptr(idx).is_null() {
            Some(unsafe{ &*self.key_ptr(idx) })
        } else {
            None
        }
    }

    fn key_mut(&mut self, idx: usize) -> Option<&mut K> {
        if !self.key_ptr(idx).is_null() {
            Some(unsafe{ &mut *self.key_ptr(idx) })
        } else {
            None
        }
    }
    fn key_ptr(&self, idx: usize) -> *mut K;
    fn assign_key_ptr(&mut self, idx: usize, key_ptr: *mut K);

    fn value(&self, idx: usize) -> Option<&V> {
        if !self.val_ptr(idx).is_null() {
            Some(unsafe{ &*self.val_ptr(idx) })
        } else {
            None
        }
    }

    fn value_mut(&mut self, idx: usize) -> Option<&mut V> {
        if !self.val_ptr(idx).is_null() {
            Some(unsafe{ &mut *self.val_ptr(idx) })
        } else {
            None
        }
    }
    fn val_ptr(&self, idx: usize) -> *mut V;
    fn assign_val_ptr(&mut self, idx: usize, val_ptr: *mut V);

    fn index_of_child(&self, child: *mut (dyn BTNode<'a, K, V> + 'a)) -> usize {
        for i in 0..self.order() {
            // as *const () just to ignore the vtable variant from the fat pointer
            if self.child(i) as *const () == child as *const () {
                return i;
            }

        }

        unreachable!()
    }

    /// key must in it!!
    fn index_of_key(&self, key: &K) -> usize {
        for i in 0..self.order() {
            if self.key(i).unwrap() == key {
                return i;
            }

        }

        unreachable!()
    }

    fn find_pos_of_key(&self, key: &K) -> Option<usize> {
        for i in 0..self.order() {
            if let Some(here_key) = self.key(i) {
                if here_key == key {
                    return Some(i);
                } else {
                    return None
                }
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

    fn is_leaf(&self) -> bool {
        for i in 0..self.order() {
            if !self.child(i).is_null() {
                return false;
            }
        }

        true
    }

    /// successor of item whose key is key.
    fn successor(&self, key: &K) -> BTItem<'a, K, V> {
        let k_idx = self.index_of_key(key);

        unsafe {
            if self.is_leaf() {
                if self.key(k_idx + 1).is_none() {  // Goto parent
                    let mut x = self.itself_mut();
                    let mut y = (*x).paren();

                    while !y.is_null() {
                        let idx = (*y).index_of_child(x);

                        if (*y).key(idx).is_some() {
                            return BTItem::new(y, idx);
                        }

                        x = y;
                        y = (*x).paren();
                    }

                    BTItem::new(y, 0)

                } else {
                    BTItem::new(self.itself_mut(), k_idx + 1)
                }

            } else {

                BTItem::new((*self.child(k_idx + 1)).minimum(), 0)
            }
        }
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

                if (*x).node_contains(income_key) || (*x).is_leaf() {
                    break;
                }

                let mut i = 0;
                let mut encountered = false;
                loop {
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
                    x = (*x).child(i);
                }
            }
        }

        y
    }

    fn swap_to_leaf(&mut self, idx: usize) -> BTItem<'a, K, V> {
        let mut item_x = BTItem::new(self.itself_mut(), idx);

        while let Ok(item_nxt) = item_x.swap_with_successor_until_leaf() {
            item_x = item_nxt;
        }

        item_x
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


    fn bfs_do(
        &self,
        action: fn(
            *mut (dyn BTNode<'a, K, V> + 'a),
        )
    ) {
        let mut queue= VecDeque::new();

        queue.push_back(self.itself_mut());
        while !queue.is_empty() {
            let x = queue.pop_front().unwrap();

            action(x);

            unsafe {
                for i in 0..self.order() {
                    let child = BTNode::child (&*x, i);

                    if !child.is_null() {
                        queue.push_back(child);
                    } else {
                        break;
                    }
                }
            }

        }
    }

}


pub struct BTItem<'a, K, V> {
    node: *mut (dyn BTNode<'a, K, V> + 'a),
    idx: usize
}

impl<'a, K: DictKey, V> BTItem<'a, K, V> {
    pub fn new(node: *mut (dyn BTNode<'a, K, V> + 'a), idx: usize) -> Self {
        Self {
            node,
            idx,
        }
    }

    pub fn key(&self) -> *mut K {
        unsafe {
            (*self.node).key_ptr(self.idx)
        }
    }

    pub fn assign_key(&mut self, key: *mut K) {
        unsafe {
            (*self.node).assign_key_ptr(self.idx, key)
        }
    }

    pub fn assign_val(&mut self, val: *mut V) {
        unsafe {
            (*self.node).assign_val_ptr(self.idx, val)
        }
    }

    pub fn val(&self) -> *mut V {
        unsafe {
            (*self.node).val_ptr(self.idx)
        }
    }

    pub fn successor(&self) -> Self {
        unsafe {
            (*self.node).successor(&*self.key())
        }
    }

    pub fn swap(x: &mut Self, y: &mut Self) {
        let tmp_key = y.key();
        let tmp_val = y.val();

        y.assign_key(x.key());
        y.assign_val(x.val());

        x.assign_key(tmp_key);
        x.assign_val(tmp_val);
    }

    pub fn swap_with_successor_until_leaf(&mut self) -> Result<Self, ()> {
        unsafe {
            if (*self.node).is_leaf() {
                return Err(())
            }

            let mut nxt_item = self.successor();

            BTItem::swap(self, &mut nxt_item);

            Ok(nxt_item)
        }
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

