use std::{cmp::min, fmt::Debug};

use coll::{node as aux_node, uuid::Uuid, *};


impl_node!(pub <T>);

def_attr_macro!(call | id, children, values);


const BIT_WIDTH: u32 = 3;
const NODE_SIZE: usize = 1 << BIT_WIDTH as usize;
const MASK: usize = NODE_SIZE - 1;


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! impl_trie_common {
    () => {
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
    };
}


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
    (dup-with| $id:expr, $x:expr, $cap:expr) => {{
        let x = $x;

        if x.is_leaf() {
            node!(dup-with-leaf| $id, x, $cap)
        }
        else {
            node!(dup-with-internal| $id, x, $cap)
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
    (single-leaf| $id:expr, $v:expr, $cap:expr) => {{
        let node = node!(leaf| $id, $cap);
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


pub struct TTrieVec<T> {
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

    impl_trie_common!();

    pub fn nth(&self, idx: usize) -> &T {
        assert!(self.cnt > idx);

        let leaf = self.down_to_leaf(idx);

        &values!(leaf)[idx!(idx, 1)]
    }

    pub fn push(&self, v: T) -> Self
    where
        T: Clone,
    {
        let cnt = self.cnt + 1;

        let mut root = self.root.clone();
        let tail;

        // trie is empty
        if self.tail.is_none() {
            tail = node!(single - leaf | self.id(), v, 1);
        }
        // tail is available
        else if self.tail.len() < NODE_SIZE {
            tail = node!(dup - inc - leaf | self.id(), &self.tail, v);
        }
        // tail is full
        else {
            tail = node!(single - leaf | self.id(), v, 1);

            root = self.push_tail_into_trie();
        }

        Self { cnt, root, tail }
    }

    /// idx in `[0, self.len()]` (update or push)
    pub fn assoc(&self, idx: usize, v: T) -> Self
    where
        T: Clone,
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

            tail = node!(dup | self.id(), &self.tail);
            values_mut!(tail)[idx!(idx, 1)] = v;
        } else {
            let mut lv = h!(self);

            debug_assert!(lv > 0);

            let mut p_i = idx!(idx, lv);
            let mut p = node!(dup | self.id(), &self.root);

            root = p.clone();

            if lv > 1 {
                // at p's level

                let mut old_cur = &children!(p)[p_i];
                let mut cur = node!(dup | self.id(), old_cur);

                loop {
                    children_mut!(p)[p_i] = cur.clone();

                    lv -= 1;

                    p_i = idx!(idx, lv);
                    p = cur;

                    if lv == 1 {
                        break;
                    }

                    old_cur = &children!(p)[p_i];
                    cur = node!(dup | self.id(), old_cur);
                }
            }

            values_mut!(p)[p_i] = v;

            tail = self.tail.clone();
        }

        Self { cnt, root, tail }
    }

    pub fn pop(&self) -> Self
    where
        T: Clone,
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
            tail = node!(dup - dec | self.id(), &self.tail);
        } else {
            // the last two idx
            tail = self.down_to_leaf(self.cnt - 2);

            root = self.pop_tail_from_trie();
        }

        Self { cnt, root, tail }
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    fn pop_tail_from_trie(&self) -> Node<T>
    where
        T: Clone,
    {
        debug_assert_eq!(self.tail.len(), 1);

        // Get empty tail
        let mut lv = h!(self);
        debug_assert!(lv > 0);

        let leaf_i = self.cnt - 1 - 1; // tail size 1
        let mut p_i = idx!(leaf_i, lv);

        let mut root;

        if lv == 1 {
            root = Node::none();

            return root;
        } else if lv == 2 && p_i == 1 {
            root = children!(self.root)[0].clone();

            return root;
        }

        debug_assert!(lv >= 2);

        let mut p = node!(dup | self.id(), &self.root);

        root = p.clone();

        let mut ps = vec![p.clone()];
        let mut pis = vec![p_i];

        while lv > 2 {
            let cur = node!(dup | self.id(), &children!(p)[p_i]);

            children_mut!(p)[p_i] = cur.clone();

            lv -= 1;

            p_i = idx!(leaf_i, lv);
            p = cur;

            ps.push(p.clone());
            pis.push(p_i);
        }

        children_mut!(p)[p_i] = Node::none();


        if p_i == 0 {
            /* Unnew path */

            for i in (0..ps.len() - 1).rev() {
                if pis[i + 1] == 0 {
                    children_mut!(ps[i])[pis[i]] = Node::none();
                }
                else {
                    break;
                }
            }

            if children!(root)[1].is_none() {
                root = children!(root)[0].clone();
            }
        }

        root
    }

    fn push_tail_into_trie(&self) -> Node<T>
    where
        T: Clone,
    {
        debug_assert_eq!(self.tail.len(), NODE_SIZE);

        let leaf_i = self.cnt - 1;
        let mut lv = h!(self);

        let root;

        if lv == 0 {
            root = self.tail.clone();

            return root;
        }
        // Complete trie including case lv == 1
        else if tailoff!(self) == NODE_SIZE.pow(lv) {
            root = node!(internal | self.id(), 2);

            children_mut!(root)[0] = self.root.clone();
            children_mut!(root)[1] = new_path(self.id(), lv, &self.tail, 1);

            return root;
        }

        debug_assert!(lv >= 2);

        let mut p_i = idx!(leaf_i, lv);
        let mut p;

        if self.root.len() > p_i {
            p = node!(dup | self.id(), &self.root);
        } else {
            p = node!(
                dup - inc - internal | self.id(),
                &self.root,
                Node::none()
            );
        };

        let root = p.clone();

        // Go down through the branch
        while lv >= 3 {
            // at p's level

            if children!(p)[p_i].is_some() {
                let old_cur = &children!(p)[p_i];
                let cur;
                let cur_i = idx!(leaf_i, lv - 1);

                if old_cur.len() > cur_i {
                    cur = node!(dup | self.id(), old_cur);
                } else {
                    cur = node!(
                        dup - inc - internal | self.id(),
                        old_cur,
                        Node::none()
                    );
                }

                children_mut!(p)[p_i] = cur.clone();

                lv -= 1;

                p_i = cur_i;
                p = cur;
            } else {
                children_mut!(p)[p_i] =
                    new_path(self.id(), lv - 1, &self.tail, 1);

                return root;
            }
        }

        debug_assert_eq!(lv, 2);

        children_mut!(p)[p_i] = self.tail.clone();

        root
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

        debug_assert!(cur.is_leaf(), "leaf_i: {}", idx!(idx, 1));

        cur.clone()
    }
}



impl<T> TTrieVec<T> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    impl_trie_common!();

    pub fn push(&mut self, v: T) -> Self
    where
        T: Clone,
    {
        // trie is empty
        if self.tail.is_none() {
            self.tail = node!(single - leaf | self.id(), v, NODE_SIZE)
        }
        // tail is available
        else if self.tail.len() < NODE_SIZE {
            let leaf_i = self.tail.len();

            if self.tail.len() <= leaf_i {
                self.tail =
                    node!(dup - with | self.id(), &self.tail, NODE_SIZE);
            }

            values_mut!(self.tail)[leaf_i] = v;
        }
        // tail is full
        else {
            self.push_tail_into_trie();
            self.tail = node!(single - leaf | self.id(), v, NODE_SIZE);
        }

        self.cnt += 1;


        Self {
            cnt: self.cnt,
            root: self.root.clone(),
            tail: self.tail.clone(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    pub fn push_tail_into_trie(&mut self)
    where
        T: Clone,
    {
        let mut lv = h!(self);

        if lv == 0 {
            self.root = self.tail.clone();

            return;
        } else if lv == 1 {
            let root = node!(internal | self.id(), NODE_SIZE);

            children_mut!(root)[0] = self.root.clone();
            children_mut!(root)[1] = self.tail.clone();

            self.root = root;

            return;
        }
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


impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none() {
            write!(f, "nil")?;
        } else if self.is_internal() {
            write!(f, "br: ")?;

            match children!(self).iter().filter(|&x| !x.is_none()).count() {
                0 => writeln!(f, "[]"),
                1 => writeln!(f, "[0]"),
                2 => writeln!(f, "[0, 1]"),
                3 => writeln!(f, "[0, 1, 2]"),
                upper => writeln!(f, "[0, 1, ... {}]", upper - 1),
            }?
        } else {
            write!(f, "leaf: ")?;

            let arr = values!(self);

            match arr.len() {
                0 => writeln!(f, "[]"),
                1 => writeln!(f, "[{:?}]", arr[0]),
                2 => writeln!(f, "[{:?}, {:?}]", arr[0], arr[1]),
                3 => {
                    writeln!(f, "[{:?}, {:?}, {:?}]", arr[0], arr[1], arr[2],)
                }
                upper => writeln!(
                    f,
                    "[{:?}, {:?}, ... {:?}]",
                    arr[0],
                    arr[1],
                    arr[upper - 1],
                ),
            }?
        }

        Ok(())
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
macro_rules! impl_trie_test_common {
    () => {
        #[allow(unused)]
        fn debug_print(&self)
        where
            T: Debug,
        {
            debug_print(&self.root, &self.tail)
        }
    };
}


#[cfg(test)]
fn debug_print<T>(root: &Node<T>, tail: &Node<T>)
where
    T: Debug,
{
    use common::vecdeq;

    let mut lv = 1usize;
    let mut cur_q = vecdeq![];

    println!();
    println!("MAIN TRIE:");
    println!();

    if root.is_some() {
        cur_q.push_back(vec![root]);
    } else {
        println!("empty.\n");
    }

    while !cur_q.is_empty() {
        println!("############ Level: {} #############\n", lv);

        let mut nxt_q = vecdeq![];

        while let Some(group) = cur_q.pop_front() {
            for (i, child) in group.into_iter().enumerate() {
                println!("{i:02}. {child:?}");

                if child.is_internal() {
                    let child_group = children!(child)
                        .iter()
                        .filter(|&x| x.is_some())
                        .collect();
                    nxt_q.push_back(child_group);
                }
            }

            println!();
        }

        cur_q = nxt_q;
        lv += 1;
    }

    // print tail
    println!("###################################\n");
    println!("TAIL: \n");

    if tail.is_some() {
        println!("{:?}", tail);
    } else {
        println!("empty.");
    }

    println!("------------- end --------------");
}


#[cfg(test)]
impl<T> PTrieVec<T> {
    impl_trie_test_common!();
}


#[cfg(test)]
mod tests {

    use super::{super::vec::*, *};

    #[test]
    fn test_ptrievec_case_1() {
        let mut vec = PTrieVec::new();

        for i in 0..30 {
            vec = vec.push(i);
        }

        for _ in 0..7 {
            vec = vec.pop();
        }

        vec = vec.pop();

        vec.debug_print();

        for i in 0..22 {
            println!("nth: {i}");
            vec.nth(i);
        }
    }

    #[test]
    fn test_ptrievec_random() {
        test_pvec!(PTrieVec::new());
    }
}
