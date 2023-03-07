//! AA tree

use std::{borrow::Borrow, fmt::Debug, cmp::Ordering::*, mem::swap};

use coll::*;

use super::*;

def_attr_macro!(clone | lv);

impl_node!();
impl_node_!({ lv: usize });

impl_tree!(AA {});
impl_rotate_cleanup!(AA);
impl_validate!(AA ->
    #[cfg(test)]
    fn validate(&self) {
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
        let (root, popped) = self.insert_at(
            self.root.clone(),
            k,
            v
        );

        self.root = root;

        popped
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where K: Borrow<Q> + Debug, Q: Ord + ?Sized, V: Debug
    {
        let (root, popped) = self.remove_at(self.root.clone(), k);

        self.root = root;

        popped.map(|it| unwrap_into!(it).into_value())
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Helper Method

    fn insert_at(&mut self, mut t: Node<K, V>, k: K, v: V) -> (Node<K, V>, Option<V>) {
        if t.is_none() {
            return (node!(BST { k, v, lv: 1 }), None);
        }

        let popped;

        match k.cmp(key!(t)) {
            Equal => {  // replace node
                return (t.clone(), Some(replace_val!(t, v)));
            }
            Less => {
                let (left, popped_) = self.insert_at(left!(t), k, v);

                conn_left!(t, left);
                popped = popped_;
            }
            Greater => {
                let (right, popped_) = self.insert_at(right!(t), k, v);

                conn_right!(t, right);
                popped = popped_;
            }
        }

        t = self.skew(t);
        t = self.split(t);

        (t, popped)
    }


    fn remove_at<Q>(&mut self, mut t: Node<K, V>, k: &Q)
    -> (Node<K, V>, Option<Node<K, V>>)
    where K: Borrow<Q> + Debug, Q: Ord + ?Sized, V: Debug
    {
        if t.is_none() {
            return (t, None);
        }

        let popped =

        match k.cmp(key!(t).borrow()) {
            Less => {
                let (left, popped_)
                    = self.remove_at(left!(t), k);
                conn_left!(t, left);
                popped_
            }
            Greater => {
                let (right, popped_)
                    = self.remove_at(right!(t), k);
                conn_right!(t, right);
                popped_
            }
            Equal => {

                if left!(t).is_none() && right!(t).is_none() {
                    return (
                        Node::none(),
                        Some(t)
                    );
                }

                let nil_dir = if left!(t).is_none() { Left } else { Right };
                let l = if nil_dir.is_left()
                    { bst_successor!(t) } else { bst_predecessor!(t) };

                let (child, l_entry)
                    = self.remove_at(child!(t, nil_dir.rev()), key!(l).borrow());

                conn_child!(t, child, nil_dir.rev());

                let scapegoat = l_entry.unwrap();

                swap(val_mut!(scapegoat), val_mut!(t));
                swap(key_mut!(scapegoat), key_mut!(t));

                Some(scapegoat)
            }
        };

        if left!(t).lv() + 1 < t.lv() || right!(t).lv() + 1 < t.lv() {
            /* Decrease lv */

            let left = left!(t);
            let right = right!(t);

            let lv = std::cmp::min(left.lv(), right.lv()) + 1;

            if lv < t.lv() {
                lv!(t, lv);

                if lv < right.lv() {
                    lv!(right, lv);
                }
            }

            /* Tripple skew */

            t = self.skew(t);

            // Warnning: right(t) changes after this
            self.skew(right!(t));

            if right!(t).is_some() && right!(right!(t)).is_some() {
                self.skew(right!(right!(t)));
            }

            /* Double split */

            t = self.split(t);

            self.split(right!(t));
        }

        (t, popped)
    }


    /// （确保）右旋，等级不变
    fn skew(&mut self, mut t: Node<K, V>) -> Node<K, V> {
        debug_assert!(t.is_some());

        if left!(t).lv() == t.lv() {
            debug_assert!(left!(t).is_some());

            t = rotate!(self, t, Right)
        }

        t
    }


    /// (确保)左旋，新节点等级上升 1
    fn split(&mut self, mut t: Node<K, V>) -> Node<K, V> {
        debug_assert!(t.is_some());

        let right = right!(t);

        if right.is_some() && right!(right).lv() == t.lv() {
            t = rotate!(self, t, Left);

            lv!(t, lv!(t) + 1);  // t == right
        }

        t
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
            assert!(left!(right).lv() < lv!(self));
            assert!(right!(right).lv() < lv!(self));
        }

        // Invariant-5.: if x.lv > 1 then x.children.len == 2
        if self.lv() > 1 {
            assert!(left.is_some() && right.is_some());
        }

        left.validate_balance();
        right.validate_balance();

    }

}



#[cfg(test)]
mod tests {

    use super::*;

    /// 这组小数据很有测试价值，能测试单旋和双旋
    #[test]
    fn test_bst_aa_case_1() {
        let mut dict = AA::<u16, ()>::new();

        dict.insert(52, ());
        assert!(dict.get(&52).is_some());

        dict.insert(47, ());
        assert!(dict.get(&47).is_some());

        dict.insert(3, ());
        assert!(dict.get(&3).is_some());

        dict.insert(35, ());
        assert!(dict.get(&35).is_some());

        dict.insert(24, ());
        assert!(dict.get(&24).is_some());

        dict.validate();

        // dict.debug_print();

        dict.remove(&24);
        assert!(dict.get(&24).is_none());
        dict.validate();

        dict.remove(&47);
        assert!(dict.get(&47).is_none());
        dict.validate();

        dict.remove(&52);
        assert!(dict.get(&52).is_none());
        dict.validate();

        dict.remove(&3);
        assert!(dict.get(&3).is_none());
        dict.validate();

        assert!(dict.get(&35).is_some());
        dict.remove(&35);
        assert!(dict.get(&35).is_none());
        dict.validate();

        // dict.debug_print();

    }

    #[test]
    fn test_bst_aa_random() {
        test_dict!(AA::new());
    }
}
