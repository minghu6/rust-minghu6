//! AA tree

use std::{borrow::Borrow, fmt::Debug, cmp::Ordering::*};

use super::*;

def_attr_macro!(lv);

impl_node!();
impl_node_!({ lv: usize });

impl_tree!(AA {});
impl_rotate_cleanup!(AA);
impl_balance_validation!(AA ->
    #[cfg(test)]
    fn validate_balance(&self) {
        self.root.validate_balance();
    }
);



impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(f, "{:?}(lv: {:?})", key!(self), lv!(self))
        } else {
            write!(f, "nil({:?})", lv!(self))
        }
    }
}


impl<K: Ord, V> AA <K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new() -> Self {
        Self {
            root: Node::none()
        }
    }


    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        // let z = node!( BST { k, v, height: 1 });

        // let popped = bst_insert!(self, z.clone());

        // // self.insert_retracing(z);
        // self.retracing(z);

        // popped
        todo!()
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Helper Method

    fn insert_at(&mut self, t: Node<K, V>, k: K, v: V) -> (Node<K, V>, Option<V>) {
        if t.is_none() {
            return (node!(BST { k, v, lv: 1 }), None);
        }

        match k.cmp(key!(t)) {
            Equal => {  // replace node
                let old_valptr = attr!(t, val);
                attr!(t, val, boxptr!(v));

                return (t, Some(unboxptr!(old_valptr)));
            }
            Less => {
                conn_left!(t, self.insert_at(left!(t), k, v).0);
            }
            Greater => {
                conn_right!(t, self.insert_at(right!(t), k, v).0);
            }
        }

        (t, None)
    }


    /// （确保）右旋
    fn skew(&mut self, t: Node<K, V>) -> Node<K, V> {

        todo!()
    }

}


impl<K, V> Node<K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Static Stats

    fn lv(&self) -> usize {
        if self.is_none() {
            0
        }
        else {
            lv!(self)
        }
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Validation Helper

    #[cfg(test)]
    fn validate_balance(&self) {
        if self.is_none() { return }

        let left = left!(self);
        let right = right!(self);

        // Invariants-1: if x is leaf then x.lv == 1
        if left.is_none() && right.is_none() {
            assert_eq!(lv!(self), 1);
        }

        // Invariant-2.: x.left.lv + 1 = x.lv.
        assert_eq!(left.lv() + 1, self.lv());

        // Invariant-3.: x.right.lv == x.lv || x.right.lv + 1 == x.lv.
        assert!(
            right.lv() == self.lv()
            || right.lv() + 1 == self.lv()
        );

        // Invariant-4.: x.right.child.lv < x.lv
        if right.is_some() {
            assert!(left!(right).lv() < 1);
            assert!(right!(right).lv() < 1);
        }

        // Invariant-5.: if x.lv > 1 then x.children.len == 2
        if self.lv() > 1 {
            assert!(left.is_some() && right.is_some());
        }

        left.validate_balance();
        right.validate_balance();

    }

}
