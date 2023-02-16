pub mod avl;
pub mod rb;
pub mod aa;
pub mod lsg;
pub mod sg;
pub mod splay;
pub mod treap;


use crate::collections::aux::*;



////////////////////////////////////////////////////////////////////////////////
//// Attr Access

def_attr_macro!(clone| left, right, paren, height, color, size, deleted);
def_attr_macro!(ref| (key, K), (val, V));


////////////////////////////////////////////////////////////////////////////////
//// Node Operation

/// Move val from x to self, return old val of self
///
/// replace val of x with null_mut
macro_rules! replace_val {
    ($it: expr, $x: expr) => {
        {
            std::mem::replace(val_mut!($it), $x)
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Basic Operation

macro_rules! child {
    ($p:expr, $dir:expr) => {
        if $dir.is_left() {
            left!($p)
        } else {
            right!($p)
        }
    };
}


/// Siblings of node-x
#[allow(unused)]
macro_rules! sib {
    ($x:expr) => {{
        let x = $x.clone();
        let p = paren!(x).upgrade();
        let x_dir = index_of_child!(p, x);

        child!(p, x_dir.rev())
    }};
}


macro_rules! conn_child {
    ($paren:expr, $child: expr, $dir:expr) => {{
        let child = $child.clone();
        let paren = $paren.clone();
        let dir = $dir;

        debug_assert!(!child.rc_eq(&paren));

        if paren.is_some() {
            if dir.is_left() {
                attr!(paren, left, child.clone());
            }
            else {
                attr!(paren, right, child.clone());
            }

            if child.is_some() {
                paren!(child, paren.downgrade());
            }
        }
    }};
}


macro_rules! conn_left {
    ($paren:expr, $child: expr) => {
        conn_child!($paren, $child, Left)
    };
}


macro_rules! conn_right {
    ($paren:expr, $child: expr) => {
        conn_child!($paren, $child, Right)
    };
}


macro_rules! disconn {
    ($paren:expr, $child: expr) => {
        {
            let child = $child.clone();
            let paren = $paren.clone();

            if child.is_some() && paren.is_some() {
                let dir = index_of_child!(paren, child);

                if dir.is_left() {
                    left!(paren, Node::none());
                }
                else {
                    right!(paren, Node::none());
                }

                paren!(child, WeakNode::none());
            }
        }
    };
}


macro_rules! index_of_child {
    ($paren:expr, $child:expr) => {{
        let paren = &$paren;
        let child = &$child;

        debug_assert!(child.is_some());

        if left!(paren).rc_eq(child) {
            Left
        } else if right!(paren).rc_eq(child) {
            Right
        } else {
            unreachable!("index of child failed")
        }
    }};
}


/// 替换根节点 replace u with v
macro_rules! subtree_shift {
    ($tree:expr, $u:expr, $v:expr) => {{
        let tree = &mut *$tree;
        let u = $u.clone();
        let v = $v.clone();

        if paren!(u).is_none() {
            if v.is_some() {
                paren!(v, WeakNode::none());
            }

            tree.root = v;
        } else {
            let paren = paren!(u).upgrade();

            conn_child!(paren, v, index_of_child!(paren, u));
        }
    }};
}


/// Right most
#[allow(unused)]
macro_rules! bst_maximum {
    ($x:expr) => {{
        let mut x = $x.clone();

        while right!(x).is_some() {
            x = right!(x);
        }

        x
    }};
}


/// Leftmost
macro_rules! bst_minimum {
    ($x:expr) => {{
        let mut x = $x.clone();

        while left!(x).is_some() {
            x = left!(x);
        }

        x
    }};
}


/// Return successor-node or none-node
macro_rules! bst_successor {
    ($x: expr) => {{
        let mut x = $x.clone();

        /* child: right, left-most */
        if right!(x).is_some() {
            bst_minimum!(right!(x))
        }
        /* paren: right-most-up, left */
        else {
            let mut y = paren!(x).upgrade();

            while y.is_some() && x.rc_eq(&right!(y)) {
                x = y.clone();
                y = paren!(y).upgrade();
            }

            y
        }
    }};
}


/// Return predecessor-node or none-node
macro_rules! bst_predecessor {
    ($x: expr) => {{
        let mut x = $x.clone();

        /* child: left, right-most */
        if left!(x).is_some() {
            bst_maximum!(left!(x))
        }
        /* paren: left-most-up, right */
        else {
            let mut y = paren!(x).upgrade();

            while y.is_some() && x.rc_eq(&left!(y)) {
                x = y.clone();
                y = paren!(y).upgrade();
            }

            y
        }
    }};
}


/// Return matched-node or none-node
macro_rules! bst_search {
    (lazy | $x: expr, $k: expr) => {{
        let mut x = $x.clone();
        let k = $k;

        while x.is_some() && k != key!(x).borrow() {
            if k < key!(x).borrow() {
                x = left!(x);
            } else {
                x = right!(x);
            }
        }

        if x.is_some() && !deleted!(x) {
            x
        }
        else {
            Node::none()
        }
    }};
    ($x: expr, $k: expr) => {{
        let mut x = $x.clone();
        let k = $k;

        while x.is_some() && k != key!(x).borrow() {
            if k < key!(x).borrow() {
                x = left!(x);
            } else {
                x = right!(x);
            }
        }

        x
    }};
}


/// Return Option<V>
macro_rules! bst_insert {
    (lazy | $tree: expr, $z: expr) => {{
        use std::cmp::Ordering::*;

        let mut y = Node::none();
        let mut x = $tree.root.clone();
        let z = $z;

        while !x.is_none() {
            y = x.clone();

            match key!(z).cmp(key!(x)) {
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

        if y.is_none() {
            $tree.root = z;
        } else {
            match key!(z).cmp(key!(y)) {
                Less => {
                    conn_left!(y, z);
                }
                Equal => {
                    let val_y = replace_val!(y, replace_val!(z, Default::default()));

                    return if deleted!(y) {
                        // restore deleted node
                        deleted!(y, false);
                        $tree.cnt += 1;

                        None
                    }
                    else {
                        Some(val_y)
                    }
                },
                Greater => {
                    conn_right!(y, z);
                }
            }
        }

        $tree.cnt += 1;
        $tree.max_cnt += 1;

        None
    }};
    ($tree: expr, $z: expr) => {{
        use std::cmp::Ordering::*;

        let mut y = Node::none();
        let mut x = $tree.root.clone();
        let z = $z;

        while !x.is_none() {
            y = x.clone();

            match key!(z).cmp(key!(x)) {
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

        if y.is_none() {
            $tree.root = z;
        } else {
            match key!(z).cmp(key!(y)) {
                Less => {
                    conn_left!(y, z);
                }
                Equal => {
                    return Some(
                        replace_val!(y, replace_val!(z, Default::default()))
                    )
                },
                Greater => {
                    conn_right!(y, z);
                }
            }
        }

        None
    }};
}


/// Return retracing node
macro_rules! bst_delete {
    (lazy | $z: expr) => {{
        let z = $z.clone();

        deleted!(z, true);

        replace_val!(z, Default::default())
    }};
    ($tree: expr, $z: expr) => {{
        let tree = &mut *$tree;
        let z = $z.clone();

        if left!(z).is_none() {
            subtree_shift!(tree, z, right!(z));
        } else if right!(z).is_none() {
            subtree_shift!(tree, z, left!(z));
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
                subtree_shift!(tree, y, right!(y));
                conn_right!(y, right!(z));
            }

            subtree_shift!(tree, z, y);
            conn_left!(y, left!(z));
        }
    }};
}


/// In-order Traversals
macro_rules! bst_flatten {
    ($z: expr) => {{
        let mut x = $z;
        let mut stack = vec![];  // paths
        let mut nodes = vec![];

        debug_assert!(x.is_some());

        'outter: loop {
            while left!(x).is_some() {
                stack.push(x.clone());
                x = left!(x);
            }

            nodes.push(x.clone());

            while right!(x).is_none() {
                if let Some(p) = stack.pop() {
                    x = p;
                    nodes.push(x.clone());
                }
                else {
                    break 'outter
                }
            }

            x = right!(x);
        }

        for x in nodes.iter() {
            paren!(x, WeakNode::none());
            left!(x, Node::none());
            right!(x, Node::none());
            x.flatten_cleanup();
        }

        nodes
    }}
}


macro_rules! bst_build {
    ($nodes: expr) => {{
        fn bst_build_<K, V>(nodes: &[Node<K, V>]) -> Node<K, V> {
            let lo = 0;
            let hi = nodes.len();

            if lo == hi {
                return Node::none();
            }

            let mid = (hi + lo) / 2;

            let left = bst_build_(&nodes[lo..mid]);
            let right = bst_build_(&nodes[mid+1..hi]);

            let p = nodes[mid].clone();

            conn_left!(p, left);
            conn_right!(p, right);

            p.build_cleanup();

            p
        }

        bst_build_($nodes)
    }};
}



/// Simple Rotation (return new root)
/// ```no_run
///            left rotate
///    x       =========>           z
///  /  \                          / \
/// t1   z                        x   t4
/// |   / \                      / \   |
///   t23 t4                    t1 t23 |
///     |  |                     |   |
///        |
///            right rotate
///     x      ==========>           z
///   /  \                         /   \
///  z    t4                      t1    x
/// /  \   |                      |    /  \
///t1 t23                         |  t23  t4
/// |  |                              |    |
/// |
/// ```
///
macro_rules! rotate {
    ($tree: expr, $x: expr, $rotation: expr) => {{
        let tree = &mut *$tree;
        let x = $x.clone();
        let rotation = $rotation;

        let z = child!(x, rotation.rev());
        let t23 = child!(z, rotation);

        conn_child!(x, t23, rotation.rev());
        subtree_shift!(tree, x, z);
        conn_child!(z, x, rotation);

        tree.rotate_cleanup(x, z.clone());

        z
    }};
}


/// Double Rotation (snd rotate dir, return new root)
/// ```no_run
///             rotate [right]-left         rotate right-[left]
///    x        =========>         x        =========>       y
///  /   \                        /  \                      / \
/// t1    z                      t1   y                    x   z
/// |   /  \                     |   / \                  / \ / \
///    y   t4                      t2   z                t1 t2t3t4
///   / \   |                       |  / \                |  | | |
///  t2 t3                            t3 t4
///   |  |                            |   |
/// ```
macro_rules! double_rotate {
    ($tree: expr, $x: expr, $snd_rotation: expr) => {{
        let tree = &mut *$tree;
        let x = $x.clone();
        let snd_rotation = $snd_rotation;

        let z = child!(x, snd_rotation.rev());
        // println!("double rotate root: {x:?}, {:?} z: {z:?}", snd_rotation.rev());
        rotate!(tree, z, snd_rotation.rev());
        rotate!(tree, x, snd_rotation)
    }};
}





/// Used in red-balck tree serials
///
/// Trciky method
macro_rules! fake_swap {
    ($x:expr, $y:expr) => {
        {
            let x = $x.clone();
            let y = $y.clone();

            std::mem::swap(key_mut!(x), key_mut!(y));
            std::mem::swap(val_mut!(x), val_mut!(y));
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Helper Method



////////////////////////////////////////////////////////////////////////////////
//// Aux Impl

/// (x: Node) for Node
macro_rules! impl_flatten_cleanup {
    ($fn:item) => {
        impl<K, V> Node<K, V> {
            #[inline]
            $fn
        }
    };
    () => {
        impl<K, V> Node<K, V> {
            #[inline]
            fn flatten_cleanup(&self) {}
        }
    }
}


/// (p: Node) for Node
macro_rules! impl_build_cleanup {
    ($fn:item ) => {
        impl<K, V> Node<K, V> {
            #[inline]
            $fn
        }
    };
    () => {
        impl<K, V> Node<K, V> {
            #[inline]
            fn build_cleanup(&self) {}
        }
    }
}


/// Params: (x: Node, z: Node)
macro_rules! impl_rotate_cleanup {
    ($name:ident -> $fn:item ) => {
        impl<K, V> $name<K, V> {
            #[inline]
            $fn
        }
    };
    ($name:ident) => {
        impl<K, V> $name<K, V> {
            #[inline]
            fn rotate_cleanup(&self, _x: Node<K, V>, _z: Node<K, V>) {}
        }
    }
}


macro_rules! impl_balance_validation {
    ($name:ident -> empty) => {
        impl<K, V> $name<K, V> {
            /// Validate BST balance factor
            #[allow(unused)]
            fn balance_validation(&self) {}
        }
    };
    ($name:ident -> $fn:item) => {
        impl<K, V> $name<K, V> {
            #[allow(unused)]
            $fn
        }
    };
    ($name:ident) => {
        impl<K, V> $name<K, V> {
            /// Validate BST balance factor
            #[cfg(test)]
            fn balance_validation(&self) {
                if self.root.is_some() {
                    self.root.balance_validation()
                }
            }
        }
    };
}


macro_rules! def_tree {
    (
        $(#[$attr:meta])*
        $treename:ident { $(
            $(#[$field_attr:meta])*
            $name: ident : $ty: ty),*
        }
    ) =>
    {
        $(#[$attr])*
        #[derive(Debug)]
        #[allow(unused)]
        pub struct $treename<K, V> {
            root: Node<K, V>,

            /* extra attr */
            $(
                $(#[$field_attr])*
                $name: $ty
            ),*
        }
    }
}


macro_rules! impl_tree_debug {
    ($treename:ident) => {
        impl<K: Ord, V> $treename<K, V> {
            pub fn debug_write<W: std::fmt::Write>(
                &self,
                f: &mut W
            ) -> std::fmt::Result
            where K: std::fmt::Debug, V: std::fmt::Debug
            {
                /* print header */

                writeln!(f, "{self:?}")?;


                /* print body */

                if self.root.is_none() {
                    return Ok(());
                }

                let mut this_q = crate::vecdeq![self.root.clone()];
                let mut lv = 1;

                while !this_q.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "############ Level: {lv} #############")?;
                    writeln!(f)?;

                    let mut nxt_q = crate::vecdeq![];

                    while let Some(x) = this_q.pop_front() {
                        if left!(x).is_none() && right!(x).is_none() {
                            write!(f, "{x:?}")?;
                        }
                        else {
                            write!(f, "{x:?} | L-> ")?;

                            let left = left!(x);
                            if left.is_some() {
                                write!(f, "{left:?}")?;
                                nxt_q.push_back(left);
                            }
                            else {
                                write!(f, "nil")?;
                            }

                            write!(f, "; R-> ")?;

                            let right = right!(x);
                            if right.is_some() {
                                write!(f, "{right:?}")?;
                                nxt_q.push_back(right);
                            }
                            else {
                                write!(f, "nil")?;
                            }
                        }

                        writeln!(f)?;
                    }

                    this_q = nxt_q;
                    lv += 1;
                }

                writeln!(f, "------------- end --------------\n")?;

                Ok(())
            }


            pub fn debug_print(&self) where K: std::fmt::Debug, V: std::fmt::Debug
            {
                let mut cache = String::new();

                self.debug_write(&mut cache).unwrap();

                println!("{cache}")
            }
        }
    };
}


macro_rules! impl_tree {
    (
        $(#[$attr:meta])*
        $treename:ident {
            $(
                $(#[$field_attr:meta])*
                $name: ident : $ty: ty
            ),*
        }
    ) =>
    {
        def_tree!(
            $(#[$attr])*
            $treename {
                $(
                    $(#[$field_attr])*
                    $name : $ty
                ),*
            }
        );
        impl_tree_debug!($treename);

        impl<K: Ord, V> $treename<K, V> {
            pub fn get<Q>(&self, k: &Q) -> Option<&V>
            where K: std::borrow::Borrow<Q>, Q: Ord + ?Sized
            {
                let x = bst_search!(self.root, k);

                if x.is_some() {
                    Some(val!(x))
                }
                else {
                    None
                }
            }

            pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
            where K: std::borrow::Borrow<Q>, Q: Ord + ?Sized
            {
                let x = bst_search!(self.root, k);

                if x.is_some() {
                    Some(val_mut!(x))
                }
                else {
                    None
                }
            }
        }

    };
}


/// Define inner node
macro_rules! impl_node_ {
    ({ $($name: ident : $ty: ty),* }) => {
        #[allow(unused)]
        struct Node_<K, V> {
            left: Node<K, V>,
            right: Node<K, V>,
            paren: WeakNode<K, V>,

            key: K,
            val: V,

            /* extra attr */
            $(
                $name: $ty
            ),*
        }

        #[allow(unused)]
        impl<K, V> Node_<K, V> {
            fn into_value(mut self) -> V {
                self.into_key_val().1
            }

            fn into_key_val(mut self) -> (K, V) {
                (self.key, self.val)
            }
        }

        impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Node_<K, V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}: {:?}", self.key, self.val)
            }
        }
    };
}



////////////////////////////////////////////////////////////////////////////////
//// Unify Test

#[cfg(test)]
macro_rules! gen_data {
    ($get_one: ident, $group: expr, $num: expr) => {{
        let group = $group;
        let num = $num;

        let mut keys = std::collections::HashSet::new();
        let mut elems = vec![];

        for _ in 0..num {
            let mut k = $get_one();
            let mut j = 0;

            while j < group {
                k += 1;
                if keys.contains(&k) {
                    continue;
                }

                keys.insert(k);
                elems.push((k, k + 1000));

                j += 1;
            }
        }

        elems
    }};
}
#[cfg(test)]
pub(crate) use gen_data;


#[cfg(test)]
macro_rules! test_dict {
    ($dict: expr) => {
        let get_one = || rand::random::<u64>();

        for _ in 0..20 {
            let mut dict = $dict;
            let mut elems = gen_data!(get_one, 10, 100);

            /* Verify Create */

            for (i, (k, v)) in elems.iter().cloned().enumerate() {
                assert!(
                    dict.insert(k, v).is_none(),
                    "[dict insert] insert res invalid"
                );
                assert_eq!(
                    dict.get(&k), Some(&v),
                     "[dict insert] insert but query failed"
                );

                if i % 20 == 0 {
                    dict.balance_validation();
                }
                // println!("{i}. insert: ");
            }

            dict.balance_validation();

            /* Verify Update */

            for (i, (k, v)) in elems.clone().into_iter().enumerate() {
                assert_eq!(dict.get(&k), Some(&v));
                // println!("{i}. update: ");

                let newv = k + 500;

                assert_eq!(
                    dict.insert(k, newv),
                    Some(v),
                    "[dict update] update get incorrect popped"
                );
                elems[i] = (k, newv);

                assert_eq!(
                    dict.get(&k),
                    Some(&newv),
                    "[dict update] update failed"
                );
            }

            dict.balance_validation();

            /* Verify Remove */

            use rand::{prelude::SliceRandom, thread_rng};

            elems.shuffle(&mut thread_rng());

            for (i, (k, v)) in elems.into_iter().enumerate() {
                // println!("[dict remove]: {i:03}: {k}");

                assert_eq!(
                    dict.get(&k),
                    Some(&v),
                    "[dict remove] Assure get Some"
                );
                assert_eq!(
                    dict.remove(&k),
                    Some(v),
                    "[dict remove] Assert remove failed"
                );
                assert_eq!(
                    dict.get(&k),
                    None,
                    "[dict remove] Assure get None"
                );

                // sample to save time
                if i % 10 == 0 {
                    dict.balance_validation();
                }
            }
        }
    };
}


////////////////////////////////////////////////////////////////////////////////
//// ReExport Declarative Macro

use replace_val;
use child;
use conn_child;
use conn_left;
use conn_right;
use disconn;
use index_of_child;
#[allow(unused)]
use sib;
use subtree_shift;

use rotate;
use double_rotate;

use fake_swap;

use bst_delete;
use bst_insert;
#[allow(unused)]
use bst_maximum;
use bst_minimum;
#[allow(unused)]
use bst_predecessor;
use bst_search;
use bst_successor;
use bst_flatten;
use bst_build;

use impl_balance_validation;
use impl_rotate_cleanup;
use impl_build_cleanup;
use impl_flatten_cleanup;

use impl_node_;
use def_tree;
use impl_tree_debug;
use impl_tree;

#[cfg(test)]
pub(crate) use test_dict;


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Dir {
    Left,
    Right,
}
pub(crate) use Dir::*;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Color {
    Red,
    Black,
}
use Color::*;


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Dir {
    fn rev(&self) -> Self {
        match self {
            Left => Right,
            Right => Left,
        }
    }

    fn is_left(&self) -> bool {
        matches!(self, Left)
    }

    #[allow(unused)]
    fn is_right(&self) -> bool {
        matches!(self, Right)
    }
}


impl Color {
    fn rev(&self) -> Self {
        match self {
            Red => Black,
            Black => Red,
        }
    }

    fn is_red(&self) -> bool {
        matches!(self, Red)
    }

    fn is_black(&self) -> bool {
        matches!(self, Black)
    }
}
