use std::{borrow::Borrow, fmt::Debug};

use coll::mut_self;

use super::*;


impl_node!();
impl_node_!({});
def_tree!(Splay {});
impl_tree_debug!(Splay);

impl_rotate_cleanup!(Splay);
impl_validate!(Splay -> empty);




impl<K: Ord, V> Splay <K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new() -> Self {
        Self {
            root: Node::none()
        }
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        let x = bst_search!(self.root, k);

        if x.is_some() {
            mut_self!(self).splay(&x);
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
            mut_self!(self).splay(&x);
            Some(val_mut!(x))
        }
        else {
            None
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        /* modify a little bst_insert */

        use std::cmp::Ordering::*;

        let mut y = Node::none();
        let mut x = self.root.clone();

        while !x.is_none() {
            y = x.clone();

            match k.cmp(key!(x)) {
                Less => {
                    x = left!(x);
                }
                Equal => {
                    break;
                }
                Greater => {
                    x = right!(x);
                }
            }
        }

        let mut popped = None;
        let splay_at;

        if y.is_none() {
            splay_at = node!( BST { k, v });
            self.root = splay_at.clone();
        } else {
            match k.cmp(key!(y)) {
                Less => {
                    splay_at = node!( BST { k, v });
                    conn_left!(y, splay_at);
                }
                Equal => {
                    popped = Some(replace_val!(y, v));
                    splay_at = y;
                },
                Greater => {
                    splay_at = node!( BST { k, v });
                    conn_right!(y, splay_at);
                }
            }
        }

        self.splay(&splay_at);

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
            let (s, l) = self.split(z);

            let s_left = left!(s);
            disconn!(s, s_left);

            self.join((s_left, l));

            Some(unwrap_into!(s).into_value())
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Helper Method

    /// rotate x to root
    fn splay(&mut self, x: &Node<K, V>)
    {
        debug_assert!(x.is_some());

        let mut p = paren!(x).upgrade();

        while p.is_some() {
            rotate!(self, p, index_of_child!(p, x).rev());
            p = paren!(x).upgrade();
        }
    }

    /// Split this tree into (<=x, >x),
    ///
    /// x MUST belongs to this tree.
    fn split(&mut self, x: Node<K, V>) -> (Node<K, V>, Node<K, V>) {
        self.splay(&x);

        let x_right = right!(x);
        disconn!(x, x_right);

        (x.to_owned(), x_right)
    }

    /// Join (S, L) tree
    fn join(&mut self, trees: (Node<K, V>, Node<K, V>))
    {
        let (s, l) = trees;

        if s.is_some() {
            let s_max = bst_maximum!(s);

            #[cfg(test)]
            {
                if l.is_some() {
                    let l_min = bst_minimum!(l);

                    if l_min.is_some() {
                        assert!(key!(s_max) < key!(l_min));
                    }
                }
            }

            self.splay(&s_max);
            self.root = s_max.clone(); //  s maybe not root node
            conn_right!(s_max, l);
        }
        else {
            self.root = l;
        }
    }


}



impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(f, "{:?}", key!(self))
        }
        else {
            write!(f, "nil")
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    /// 这组小数据很有测试价值，能测试单旋和双旋
    #[test]
    fn test_bst_splay_case_1() {
        let mut dict = Splay::<u16, ()>::new();

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
    }

    #[test]
    fn test_bst_splay_random() {
        test_dict!(Splay::new());
    }
}
