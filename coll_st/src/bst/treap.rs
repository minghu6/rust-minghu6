use std::{borrow::Borrow, fmt::Debug};

use common::random;

use super::*;

use coll::mut_self;


def_attr_macro!(clone| w);


impl_node!();
impl_node_!({ w: usize });
def_tree!(Treap { improve_search: bool });
impl_tree_debug!(Treap);

impl_rotate_cleanup!(Treap);
impl_validate!(Treap);



impl<K: Ord, V> Treap <K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new() -> Self {
        Self {
            root: Node::none(),
            improve_search: false
        }
    }

    pub fn improve_search(mut self) -> Self {
        self.improve_search = true;
        self
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        let x = bst_search!(self.root, k);

        if x.is_some() {
            if self.improve_search {
                self.aragon_seidel_search_suggestion(x.clone());
            }

            Some(val!(x))
        }
        else {
            None
        }
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        let x = bst_search!(self.root, k);

        if x.is_some() {
            if self.improve_search {
                self.aragon_seidel_search_suggestion(x.clone());
            }

            Some(val_mut!(x))
        }
        else {
            None
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where V: Default
    {
        let z = node!( BST { k, v, w: random() });

        let popped = bst_insert!(self, z.clone());

        if popped.is_none() {
            self.siftup(z);
        }

        popped
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where K: Borrow<Q> + Debug, Q: Ord + ?Sized, V: Debug
    {

        let z = bst_search!(self.root, k);

        if z.is_none() {
            None
        }
        else {
            if left!(z).is_none() {
                subtree_shift!(self, z, right!(z));
            } else if right!(z).is_none() {
                subtree_shift!(self, z, left!(z));
            } else {
                /* case-1       case-2

                    z            z
                    \            \
                    y            z.right
                                /
                                / (left-most)
                                y
                                \
                                y.right
                */

                let y = bst_successor!(z);

                if !right!(z).rc_eq(&y) {
                    subtree_shift!(self, y, right!(y));
                    conn_right!(y, right!(z));
                }
                subtree_shift!(self, z, y);
                conn_left!(y, left!(z));

                /* Only y and y.left and maybe y.right violate weight */
                self.siftdown(y);
            }

            Some(unwrap_into!(z).into_value())
        }
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Helper Method

    /// rotate up if MaxHeap violation
    fn siftup(&mut self, x: Node<K, V>) {

        let mut p = paren!(x).upgrade();

        while p.is_some() && w!(p) < w!(x) {
            rotate!(self, p, index_of_child!(p, x).rev());
            p = paren!(x).upgrade();
        }
    }

    /// rotate down if MaxHeap violation
    fn siftdown(&mut self, x: Node<K, V>) {

        loop {
            let left = left!(x);
            let right = right!(x);

            let mut max_w = w!(x);
            let mut max_child = None;

            if left.is_some() && w!(left) > max_w {
                max_w = w!(left);
                max_child = Some(Left);
            }

            if right.is_some() && w!(right) > max_w {
                max_child = Some(Right);
            }

            if let Some(child_dir) = max_child {
                rotate!(self, x, child_dir.rev());
            }
            else {
                break;
            }

        }
    }

    /// https://en.wikipedia.org/wiki/Treap
    fn aragon_seidel_search_suggestion(&self, x: Node<K, V>) {
        let neww = random();

        if neww > w!(x) {
            w!(x, neww);
            mut_self!(self).siftup(x);
        }
    }

}



impl<K, V> Node<K, V> {
    /// Validate MaxHeap
    #[cfg(test)]
    fn validate(&self) {
        let left = left!(self);
        let right = right!(self);

        if left.is_some() {
            debug_assert!(w!(self) >= w!(left));
            left.validate();
        }

        if right.is_some() {
            debug_assert!(w!(self) >= w!(right));
            right.validate();
        }
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(f, "{:?}(w: {})", key!(self), w!(self))
        }
        else {
            write!(f, "nil")
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bst_treap_case_1() {
        let mut dict = Treap::<u16, ()>::new().improve_search();

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
    }

    #[test]
    fn test_bst_treap_random() {
        test_dict!(Treap::new());
        test_dict!(Treap::new().improve_search());
    }


}
