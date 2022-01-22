//! Lazy Scapegoat Tree
//!

use std::{ptr::{null, null_mut, self}, cmp::max, marker::PhantomData};

use itertools::Itertools;

use super::{BSTNode, BST};
use crate::collections::{
    bt::{BTNode, BT},
    DictKey, Dictionary,
};

pub struct LSG<'a, K: DictKey + 'a, V: 'a> {
    root: *mut LSGNode<'a, K, V>,
    deleted: usize,
    alpha: f32,
}

pub struct LSGNode<'a, K: DictKey + 'a, V: 'a> {
    paren: *mut Self,
    left: *mut Self,
    right: *mut Self,

    size: usize,
    is_deleted: bool,

    key: *mut K,
    value: *mut V,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement


fn size<'a, K: DictKey + 'a, V: 'a> (x: *mut LSGNode<'a, K, V>) -> usize {
    unsafe {
        if x.is_null() {
            return 0;
        }

        (*x).size
    }
}


impl<'a, K: DictKey + 'a, V: 'a> LSGNode<'a, K, V> {
    pub fn new(key: K, value: V) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),

            size: 1,
            is_deleted: false,

            key: Box::into_raw(box key),
            value: Box::into_raw(box value),
        })
    }

    pub fn into_value(self) -> V {
        unsafe { *Box::from_raw(self.value) }
    }

    fn update_size(&mut self) {
        self.size = size(self.left) + size(self.right);

        if !self.is_deleted {
            self.size += 1;
        }
    }

    fn is_unbalanced(&self, _alpha: f32) -> bool {
        // max(size(self.left), size(self.right)) > alpha as usize * self.size
        size(self.left).abs_diff(size(self.right)) > 1
    }

    // fn self_validate(&mut self, alpha: f32) {
    //     unsafe {

    //         assert!(!self.is_unbalanced(alpha));

    //         if !self.left.is_null() {
    //             (*self.left).self_validate(alpha);
    //         }

    //         if !self.right.is_null() {
    //             (*self.right).self_validate(alpha);
    //         }

    //     }

    // }

}

// impl<'a, K: DictKey + 'a, V: 'a> Drop for LSGNode<'a, K, V> {
//     fn drop(&mut self) {
//         unsafe {
//             drop(Box::from_raw(self.key));
//             drop(Box::from_raw(self.left));
//             drop(Box::from_raw(self.right));

//             if !self.is_deleted {
//                 drop(Box::from_raw(self.value));
//             }

//         }

//     }
// }


impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for LSGNode<'a, K, V> {
    fn itself(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        self as *const Self
    }

    fn null(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        null::<Self>()
    }

    fn try_as_bst(&self) -> Result<*const (dyn BSTNode<'a, K, V> + 'a), ()> {
        Ok(self as *const Self)
    }

    fn order(&self) -> usize {
        2
    }

    fn child(&self, idx: usize) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        if idx == 0 {
            self.left
        } else {
            self.right
        }
    }

    fn assign_child(
        &mut self,
        child: *mut (dyn BTNode<'a, K, V> + 'a),
        idx: usize,
    ) {
        match idx {
            0 => {
                self.left = child as *mut Self;
            }
            1 => {
                self.right = child as *mut Self;
            }
            _ => unreachable!(),
        }
    }

    fn assign_value(&mut self, value: V, _idx: usize) {
        self.value = Box::into_raw(box value);
    }

    fn assign_paren(&mut self, paren: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.paren = paren as *mut Self;
    }

    fn paren(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.paren as *mut (dyn BTNode<'a, K, V> + 'a)
    }

    fn key(&self, idx: usize) -> Option<&K> {
        if idx == 0 {
            unsafe { Some(&*self.key) }
        } else {
            None
        }
    }

    fn key_ptr(&self, idx: usize) -> *mut K {
        if idx == 0 {
            self.key
        } else {
            null_mut::<K>()
        }
    }

    fn assign_key_ptr(&mut self, idx: usize, key_ptr: *mut K) {
        if idx == 0 {
            self.key = key_ptr;
        }
    }

    fn val_ptr(&self, idx: usize) -> *mut V {
        if idx == 0 {
            self.value
        } else {
            null_mut::<V>()
        }
    }

    fn assign_val_ptr(&mut self, idx: usize, val_ptr: *mut V) {
        if idx == 0 {
            self.value = val_ptr;
        }
    }

    fn height(&self) -> i32 {
        self.calc_height()
    }
}

impl<'a, K: DictKey + 'a, V: 'a> BSTNode<'a, K, V> for LSGNode<'a, K, V> {}


impl<'a, K: DictKey + 'a, V: 'a> LSG<'a, K, V> {
    pub fn new() -> Self {
        Self {
            root: null_mut(),
            alpha: 0.7,
            deleted: 0,
        }
    }

    pub fn with_alpha(alpha: f32) -> Self {
        debug_assert!(0.5 < alpha);
        debug_assert!(alpha < 1.0);

        Self {
            root: null_mut(),
            alpha,
            deleted: 0,
        }
    }

    pub fn size(&self) -> usize {
        if self.root.is_null() {
            return 0
        }

        unsafe { (*self.root).size }
    }

    pub fn bulk_load(iter: &mut dyn Iterator<Item = (K, V)>) -> Self {
        let nodes = iter.map(|(key, val)| LSGNode::new(key, val))
        .collect_vec();

        let seq = &nodes[..];

        let mut sg = LSG::new();

        unsafe {
            sg.root = LSG::build(seq);
        }

        sg
    }

    unsafe fn build(seq: &[*mut LSGNode<'a, K, V>]) -> *mut LSGNode<'a, K, V> {
        if seq.is_empty() {
            return null_mut();
        }

        LSG::build_(seq, 0, seq.len() - 1)
    }

    unsafe fn build_(
        seq: &[*mut LSGNode<'a, K, V>],
        low: usize,
        high: usize,
    ) -> *mut LSGNode<'a, K, V> {

        if high < low {
            return null_mut();
        }

        let mid = (low + high) / 2;
        let x = seq[mid];
        debug_assert!(!x.is_null());

        (*x).connect_left(
            if mid == 0 {
                 null_mut::<LSGNode<'a, K, V>>()
            } else {
                LSG::build_(seq, low, mid - 1)
            }
        );

        (*x).connect_right(
            LSG::build_(seq, mid + 1, high)
        );

        (*x).update_size();


        x
    }

    unsafe fn refact(x: *mut LSGNode<'a, K, V>) -> *mut LSGNode<'a, K, V> {
        debug_assert!(!x.is_null());
        // if x.is_null() {
        //     return x;
        // }

        let mut nodes = Vec::new();
        LSG::collect_alive(x, &mut nodes);

        LSG::build(&nodes[..])
    }


    unsafe fn collect_alive(x: *mut LSGNode<'a, K, V>, container: &mut Vec<*mut LSGNode<'a, K, V>>) {
        if x.is_null() {
            return;
        }

        LSG::collect_alive((*x).left, container);
        if !(*x).is_deleted {
            container.push(x);
        }
        LSG::collect_alive((*x).right, container);
    }

    unsafe fn find_scapegoat(mut x: *mut LSGNode<'a, K, V>, alpha: f32) -> *mut LSGNode<'a, K, V> {

        while !x.is_null() {

            if (*x).is_unbalanced(alpha) {
                return x;
            }

            x = (*x).paren
        }

        x
    }

    unsafe fn remove_retracing(&mut self, x: *mut LSGNode<'a, K, V>) {
        LSG::update_size_to_root(x);
        self.deleted += 1;

        if self.deleted >= self.size() {
            self.reset_root(
                LSG::refact(self.root)
            );
        }

    }


    unsafe fn update_size_to_root(mut x: *mut LSGNode<'a, K, V>) {

        while !x.is_null() {
            (*x).update_size();

            x = (*x).paren
        }

    }

    fn partial_refact(&mut self, sgnode: *mut LSGNode<'a, K, V>) {
        unsafe {
            if (*sgnode).paren.is_null() {
                self.reset_root(
                    LSG::refact(sgnode)
                );

            } else {
                let sgnode_paren = (*sgnode).paren;

                if (*sgnode_paren).index_of_child(sgnode) == 0 {
                    (*sgnode_paren).connect_left(sgnode);
                } else {
                    (*sgnode_paren).connect_right(sgnode);
                }

                LSG::update_size_to_root(sgnode_paren);
            }

        }

    }

}

// impl<'a, K: DictKey + 'a, V: 'a> Drop for LSG<'a, K, V> {
//     fn drop(&mut self) {
//         unsafe {
//             self.reset_root(
//                 LSG::refact(self.root)
//             );
//         }

//     }
// }


impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for LSG<'a, K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        unsafe {
            let approxi_node =
                (*self.search_approximately(&key))
                .try_as_bst_mut().unwrap() as *mut LSGNode<'a, K, V>;


            let entry;
            if !approxi_node.is_null() && (*approxi_node).key_bst() == &key {
                if (*approxi_node).is_deleted {
                    let vp = Box::into_raw(box value);
                    (*approxi_node).value = vp;
                    (*approxi_node).is_deleted = false;

                    entry = approxi_node;
                } else {
                    return false;
                }

            } else {
                let new_node = LSGNode::new(key, value);
                let key = (*new_node).key_bst();

                if approxi_node.is_null() {
                    (*new_node).assign_paren(approxi_node);
                    self.assign_root(new_node)
                } else if key < (*approxi_node).key_bst() {
                    (*approxi_node).connect_left(new_node)
                } else {
                    (*approxi_node).connect_right(new_node)
                }

                entry = new_node;
            }

            LSG::update_size_to_root(entry);
            let sgnode = LSG::find_scapegoat(entry, self.alpha);

            if !sgnode.is_null() {
                self.partial_refact(sgnode);
            }

            // let mut x = sgnode;
            // while !x.is_null() {
            //     self.partial_refact(x);
            //     x = LSG::find_scapegoat((*x).paren, self.alpha);
            // }

            true
        }

    }

    fn remove(&mut self, key: &K) -> Option<V> {
        unsafe {
            let approxi_node =
            (*self.search_approximately(&key))
            .try_as_bst_mut().unwrap() as *mut LSGNode<'a, K, V>;

            if approxi_node.is_null() {
                return None;
            }

            if (*approxi_node).key_bst() == key && !(*approxi_node).is_deleted {
                (*approxi_node).is_deleted = true;
                self.remove_retracing(approxi_node);

                let value = (*approxi_node).value;
                (*approxi_node).value = null_mut();

                Some(*Box::from_raw(value))

            } else {
                None
            }

        }


    }

    fn modify(&mut self, key: &K, value: V) -> bool {


        self.basic_modify(key, value)
    }

    fn lookup(&self, key: &K) -> Option<&V> {
        unsafe {
            let res = self.search_approximately(key) as *mut LSGNode<'a, K, V>;

            if res.is_null() || (*res).key_bst() != key || (*res).is_deleted {
                None
            } else {

                Some(&*(*res).value)
            }
        }

    }

    fn lookup_mut(&mut self, key: &K) -> Option<&mut V> {
        unsafe {
            let res = self.search_approximately(key) as *mut LSGNode<'a, K, V>;

            if res.is_null() || (*res).key_bst() != key || (*res).is_deleted {
                None
            } else {

                Some(&mut *(*res).value)
            }
        }

    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()
    }
}



impl<'a, K: DictKey + 'a, V: 'a> BT<'a, K, V> for LSG<'a, K, V> {
    fn order(&self) -> usize {
        2
    }

    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.root = root as *mut LSGNode<'a, K, V>;
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BST<'a, K, V> for LSG<'a, K, V> {
    unsafe fn rotate_cleanup(
        &mut self,
        _x: *mut (dyn BSTNode<'a, K, V> + 'a),
        _z: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) {
    }
}



#[cfg(test)]
pub(crate) mod tests {

    use itertools::Itertools;
    use rand::{prelude::SliceRandom, thread_rng};

    use super::LSG;
    use crate::{
        collections::{
            bt::{
                bst::{BSTNode, BST},
                BTNode, BT,
            },
            Dictionary,
        },
        test::{
            dict::{DictProvider, GetKey},
            *,
        },
    };


    #[test]
    pub(crate) fn test_lsg_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box LSG::new());
    }

    ///
    /// Debug Scapegoat entry
    ///
    // #[ignore = "Only used for debug"]
    #[test]
    fn hack_lsg() {
        for _ in 0..20 {
            let batch_num = 3;
            let mut collected_elems = vec![];
            let mut keys = vec![];
            let provider = InodeProvider {};
            let mut dict = LSG::new();
            let m = 100;

            // Create
            let mut i = 0;
            while i < batch_num {
                let e = provider.get_one();
                let k = e.get_key() % m;
                if keys.contains(&k) {
                    continue;
                }

                keys.push(k.clone());
                collected_elems.push(e.clone());

                println!("insert {}: {:?}", i, k);
                assert!(dict.insert(k, e));
                assert!(dict.lookup(&keys.last().unwrap()).is_some());

                dict.self_validate().unwrap();

                i += 1;
            }

            collected_elems.shuffle(&mut thread_rng());

            // Remove-> Verify
            for i in 0..batch_num {
                let e = &collected_elems[i];
                let k = &(e.get_key() % m);

                println!("remove: {}", k);

                assert!(dict.remove(k).is_some());
                assert!(!dict.lookup(k).is_some());

                if let Ok(_res) = dict.self_validate() {
                } else {
                }
            }
        }
    }


    #[test]
    fn test_lsg_fixeddata_case_1() {
        let mut lsg = LSG::<i32, ()>::new();

        let dict = &mut lsg as &mut dyn Dictionary<i32, ()>;

        dict.insert(54, ());
        dict.insert(57, ());
        dict.insert(33, ());


        assert!(dict.remove(&33).is_some());
        // assert!(dict.remove(&57).is_some());
        // assert!(dict.remove(&54).is_some());

        BST::just_echo_stdout(&lsg);
    }
}
