use std::cmp::min;

use coll::{node as aux_node, uuid::Uuid, *};


impl_node!(pub <T>);

def_attr_macro!(call | id, children, values);


const BIT_WIDTH: u32 = 5;
const NODE_SIZE: usize = 1 << BIT_WIDTH as usize;
const MASK: usize = NODE_SIZE - 1;


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! node {
    (dup| $id:expr, $x:expr) => {{
        let x = $x;

        if x.is_leaf() {
            node!(dup-with-leaf| $id, x, x.len())
        }
        else {
            node!(dup-with-internal| $id, x, x.len())
        }
    }};
    (dup-inc-leaf| $id:expr, $x:expr, $v:expr) => {{
        let x = $x;

        debug_assert!(x.is_leaf());

        let node = node!(dup-with-leaf| $id, x, x.len() + 1);

        values_mut!(node)[x.len()] = $v;

        node
    }};
    (dup-inc-internal| $id:expr, $x:expr, $v:expr) => {{
        let x = $x;

        debug_assert!(x.is_internal());

        let node = node!(dup-with-internal| $id, x, x.len() + 1);

        children_mut!(node)[x.len()] = $v;

        node
    }};
    (dup-dec| $id:expr, $x:expr) => {{
        let x = $x;
        debug_assert!(x.len() > 0);

        if x.is_leaf() {
            node!(dup-with-leaf| $id, x, x.len() - 1)
        }
        else {
            node!(dup-with-internal| $id, x, x.len() - 1)
        }
    }};
    (dup-with-leaf| $id:expr, $x:expr, $cap:expr) => {{
        let x = $x;
        let cap = $cap;

        debug_assert!(cap <= NODE_SIZE);

        let node = node!(leaf| $id, cap);

        // let node_p = values_mut!(node).as_mut_ptr();
        // let x_p = values!(x).as_ptr();

        // unsafe {
        //     std::ptr::copy(x_p, node_p, min(x.len(), cap))
        // }

        let n = min(x.len(), cap);

        values_mut!(node)[..n].clone_from_slice(&values!(x)[..n]);

        node
    }};
    (dup-with-internal| $id:expr, $x:expr, $cap:expr) => {{
        let x = $x;
        let cap = $cap;

        debug_assert!(cap <= NODE_SIZE);

        let n = min(x.len(), cap);

        let node = node!(internal| $id, cap);

        children_mut!(node)[..n].clone_from_slice(&children!(x)[..n]);

        node
    }};
    (single-leaf| $v:expr) => {{
        let node = node!(leaf| None, 1);
        values_mut!(node)[0] = $v;
        node
    }};
    (leaf| $id:expr, $cap:expr) => {{
        aux_node!(FREE-ENUM Leaf {
            id: $id,
            values: Array::new($cap)
        })
    }};
    (internal| $id:expr, $cap:expr) => {{
        aux_node!(FREE-ENUM Internal {
            id: $id,
            children: Array::new_with_clone(Node::none(), $cap)
        })
    }};
}


macro_rules! h {
    ($self:ident) => {
        trie_height(tailoff!($self))
    };
}


/// Tail Offset, that's elements before tail
macro_rules! tailoff {
    ($self:ident) => {
        if $self.cnt == 0 {
            0
        } else {
            (($self.cnt - 1) >> BIT_WIDTH) << BIT_WIDTH
        }
    };
}


macro_rules! idx {
    ($idx:expr, $lv:expr) => {
        // Precedence: '*' > '>>' > '&'
        $idx >> ($lv - 1) * BIT_WIDTH & MASK
    };
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Clone)]
pub struct PTrieVec<T> {
    cnt: usize,
    root: Node<T>,
    tail: Node<T>,
}


pub type ID = Option<Uuid>;


enum Node_<T> {
    Internal { id: ID, children: Array<Node<T>> },
    Leaf { id: ID, values: Array<T> },
}
use Node_::*;



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<T> PTrieVec<T> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn new() -> Self {
        Self {
            cnt: 0,
            root: Node::none(),
            tail: Node::none(),
        }
    }

    pub fn len(&self) -> usize {
        self.cnt
    }

    pub fn id(&self) -> Option<Uuid> {
        if self.root.is_none() {
            None
        } else {
            id!(self.root).clone()
        }
    }

    pub fn nth(&self, idx: usize) -> &T {
        assert!(self.cnt > idx);

        let leaf = self.down_to_leaf(idx);

        &values!(leaf)[idx!(idx, 1)]
    }

    pub fn push(&self, v: T) -> Self
    where
        T: Clone
    {
        let cnt = self.cnt + 1;

        let mut root = self.root.clone();
        let tail;

        // trie is empty
        if self.cnt == 0 {
            tail = node!(single - leaf | v);
        }
        // tail is available
        else if self.tail.len() < NODE_SIZE {
            tail = node!(dup-inc-leaf| self.id(), &self.tail, v);
        }
        // tail is full
        else {
            tail = node!(single - leaf | v);

            root = self.push_tail_into_trie();
        }

        Self { cnt, root, tail }
    }

    /// idx in `[0, self.len()]` (update or push)
    pub fn assoc(&self, idx: usize, v: T) -> Self
    where
        T: Clone
    {
        assert!(self.cnt >= idx);

        if idx == self.cnt {
            return self.push(v);
        }

        debug_assert!(self.cnt > 0);

        let cnt = self.cnt;
        let root;
        let tail;

        if idx >= tailoff!(self) {
            root = self.root.clone();

            tail = node!(dup| self.id(), &self.tail);
            values_mut!(tail)[idx!(idx, 1)] = v;
        }
        else {
            let mut lv = h!(self);

            debug_assert!(lv > 0);

            let mut p_i = idx!(idx, lv);
            let mut p = node!(dup| self.id(), &self.root);

            root = p.clone();

            if lv > 1 {
                // at p's level

                let mut old_cur = &children!(p)[p_i];
                let mut cur = node!(dup| self.id(), old_cur);

                loop {
                    children_mut!(p)[p_i] = cur.clone();

                    lv -= 1;

                    p_i = idx!(idx, lv);
                    p = cur;

                    if lv == 1 { break }

                    old_cur = &children!(p)[p_i];
                    cur = node!(dup| self.id(), old_cur);

                }
            }

            values_mut!(p)[p_i] = v;

            tail = self.tail.clone();
        }

        Self { cnt, root, tail }
    }

    pub fn pop(&self) -> Self
    where
        T: Clone
    {
        assert!(self.cnt > 0, "Can't pop empty vector");
        debug_assert!(self.tail.len() > 0);

        let cnt = self.cnt - 1;

        let mut root = self.root.clone();
        let tail;

        // Get empty vec
        if self.cnt == 1 {
            tail = Node::none();
        }
        // Get non-empty tail
        else if self.tail.len() > 1 {
            tail = node!(dup-dec| self.id(), &self.tail);
        }
        else {
            // the last two idx
            tail = self.down_to_leaf(self.cnt - 2);

            root = self.pop_tail_from_trie();
        }

        Self {
            cnt,
            root,
            tail
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    fn pop_tail_from_trie(&self) -> Node<T>
    where
        T: Clone
    {
        debug_assert_eq!(self.tail.len(), 1);

        // Get empty tail
        let mut lv = h!(self);
        debug_assert!(lv > 0);

        let leaf_i = self.cnt - 1 - 1;  // tail size 1
        let mut p_i = idx!(leaf_i, lv);

        let root;

        if lv == 1 {
            root = Node::none();

            return root;
        }
        else if lv == 2 && p_i == 1 {
            root = children!(self.root)[0].clone();

            return root;
        }

        debug_assert!(lv >= 2);

        let mut p = node!(dup| self.id(), &self.root);

        root = p.clone();

        while lv > 2 {
            let cur = node!(dup| self.id(), &children!(p)[p_i]);

            children_mut!(p)[p_i] = cur.clone();

            lv -= 1;

            p_i = idx!(leaf_i, lv);
            p = cur;
        }

        children_mut!(p)[p_i] = Node::none();

        root
    }

    fn push_tail_into_trie(&self) -> Node<T>
    where
        T: Clone
    {
        debug_assert_eq!(self.tail.len(), NODE_SIZE);

        let leaf_i = self.cnt - 1;
        let mut lv = h!(self);

        let root;

        if lv == 0 {
            root = self.tail.clone();

            return root;
        } else if lv == 1 {
            root = node!(internal| self.id(), 2);

            children_mut!(root)[0] = self.root.clone();
            children_mut!(root)[1] =
                new_path(self.id(), h!(self), &self.tail, 1);

            return root;
        }

        debug_assert!(lv >= 2);

        let mut p_i = idx!(leaf_i, lv);
        let mut cur_i = idx!(leaf_i, lv - 1);

        let mut old_p = &self.root;
        let mut p;

        if old_p.len() > p_i {
            p = node!(dup | self.id(), old_p);
        } else {
            p = node!(dup - inc - internal | self.id(), old_p, Node::none());
        };

        let mut old_cur = &children!(p)[p_i];
        let mut cur;

        let ret = p.clone();

        // Go down through the branch
        while lv >= 3 {
            // at p's level

            if old_p.len() > p_i && children!(old_p)[p_i].is_some() {
                if old_cur.len() > cur_i {
                    cur = node!(dup | self.id(), old_cur);
                } else {
                    cur = node!(
                        dup - inc - internal | self.id(),
                        old_p,
                        Node::none()
                    );
                }

                lv -= 1;

                p_i = cur_i;
                cur_i = idx!(leaf_i, lv - 1);

                old_p = old_cur;
                old_cur = &children!(old_p)[p_i];

                children_mut!(p)[p_i] = cur.clone();
                p = cur;
            } else {
                cur = new_path(self.id(), lv, &self.tail, 1);
                children_mut!(p)[p_i] = cur.clone();

                return ret;
            }
        }

        debug_assert_eq!(lv, 2);

        children_mut!(p)[p_i] = self.tail.clone();

        ret
    }

    // Alias as search to leaf, array_for, etc
    fn down_to_leaf(&self, idx: usize) -> Node<T> {
        debug_assert!(idx < self.cnt);

        if idx >= tailoff!(self) {
            return self.tail.clone();
        }

        let mut lv = h!(self);
        let mut cur = &self.root;

        while lv > 1 {
            cur = &children!(cur)[idx!(idx, lv)];
            lv -= 1;
        }

        cur.clone()
    }
}



impl<T> Node_<T> {
    fn is_leaf(&self) -> bool {
        matches!(self, Leaf { .. })
    }

    def_node__heap_access!(both, id, ID);
    def_node__heap_access!(internal, children, Array<Node<T>>);
    def_node__heap_access!(leaf, values, Array<T>);
}


impl<T> Node<T> {
    fn is_leaf(&self) -> bool {
        self.is_some() && attr!(self | self).is_leaf()
    }

    fn is_internal(&self) -> bool {
        self.is_some() && !attr!(self | self).is_leaf()
    }

    fn len(&self) -> usize {
        assert!(self.is_some());

        if self.is_internal() {
            children!(self).len()
        } else {
            values!(self).len()
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

const fn trie_height(trie_size: usize) -> u32 {
    match trie_size {
        0 => 0,
        1 => 1,
        x => {
            let mut h = (x - 1).ilog2() / BIT_WIDTH;

            if x > NODE_SIZE.pow(h as u32) {
                h += 1;
            }

            h
        }
    }
}


/// Top-down clone new path from lv (1..h)
fn new_path<T>(id: ID, lv: u32, x: &Node<T>, cap: usize) -> Node<T> {
    if lv == 1 {
        return x.clone();
    }

    let node = node!(internal | id, cap);

    children_mut!(node)[0] = new_path(id, lv - 1, x, cap);

    node
}


////////////////////////////////////////////////////////////////////////////////
//// Test Method




#[cfg(test)]
mod tests {

    use super::{ *, super::vec::* };

    #[test]
    fn test_ptrievec_random() {
        test_pvec!(PTrieVec::new());
    }


}

