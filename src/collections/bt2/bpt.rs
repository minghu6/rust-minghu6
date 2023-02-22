//! B+ Tree
//!


use std::{borrow::Borrow, fmt::*, mem::swap};

use m6coll::KVEntry;

use super::{
    super::bst2::{Left, Right},
    node as aux_node, *,
};

impl_node!();
def_tree!(
    /// B+ Trees
    ///
    BPT {}
);


////////////////////////////////////////////////////////////////////////////////
//// Macro

/// Node_ heap data field access
macro_rules! def_node__heap_access {
    (internal, $name:ident, $ret:ty) => {
        fn $name(&self) -> &$ret {
            match self {
                Internal { $name, .. } => $name,
                Leaf {..} => unreachable!(
                    "Get `{}` on leaf",
                    stringify!($name)
                )
            }
        }
        paste::paste!(
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf {..} => unreachable!(
                        "Get `{}` on leaf",
                        stringify!($name)
                    )
                }
            }
        );
    };
    (leaf, $name:ident, $ret:ty) => {
        fn $name(&self) -> &$ret {
            match self {
                Internal {..} => unreachable!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, ..} => $name
            }
        }
        paste::paste!(
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal {..} => unreachable!(
                        "Get `{}` on internal node",
                        stringify!($name)
                    ),
                    Leaf { $name, ..} => $name
                }
            }
        );
    };
}


/// Node_ WeakNode field access
macro_rules! def_node__wn_access {
    (both, $name:ident) => {
        fn $name(&self) -> Node<K, V> {
            match self {
                Internal { $name, .. } => $name,
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        paste::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf { $name, .. } => $name,
                }
                .replace(x.downgrade());
            }
        );
    };
    (leaf, $name:ident) => {
        fn $name(&self) -> Node<K, V> {
            match self {
                Internal {..} => unreachable!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        paste::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal {..} => unreachable!(
                        "Get `{}` on internal node",
                        stringify!($name)
                    ),
                    Leaf { $name, .. } => $name,
                }
                .replace(x.downgrade());
            }
        );
    };
}


macro_rules! def_attr_macro_bpt {
    ($($name:ident),+) => {
        $(
            paste::paste!(
                macro_rules! $name {
                    ($node:expr) => {
                        attr!(self | $$node).$name()
                    };
                    ($node:expr, $val:expr) => {
                        attr!(self_mut | $$node).[<set_ $name>]($$val)
                    };
                }
            );

            paste::paste!(
                #[allow(unused)]
                macro_rules! [<$name _mut>] {
                    ($node:expr) => {
                        attr!(self_mut | $$node).[<$name _mut>]()
                    };
                }
            );
        )+
    };
}


def_attr_macro_bpt!(paren, succ, keys, entries, children);


////////////////////////////////////////////////////////////////////////////////
//// Structure

enum Node_<K, V> {
    Internal {
        keys: Vec<K>,
        children: Vec<Node<K, V>>,
        paren: WeakNode<K, V>,
    },
    Leaf {
        entries: Vec<KVEntry<K, V>>,
        /// Successor (Leaf)
        succ: WeakNode<K, V>,
        paren: WeakNode<K, V>,
    },
}
use Node_::*;



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K: Ord, V, const M: usize> BPT<K, V, M> {

    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn new() -> Self {
        assert!(M > 2, "M should be greater than 2");

        Self { root: Node::none() }
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self
        .root
        .search(k)
        .map(|(node, idx)| &entries!(node)[idx].1)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self
        .root
        .search(k)
        .map(|(node, idx)| &mut entries_mut!(node)[idx].1)
    }
}



impl<K, V> Node_<K, V> {
    fn is_leaf(&self) -> bool {
        matches!(self, Leaf {..})
    }

    def_node__heap_access!(internal, keys, Vec<K>);
    def_node__heap_access!(internal, children, Vec<Node<K, V>>);
    def_node__heap_access!(leaf, entries, Vec<KVEntry<K, V>>);

    def_node__wn_access!(both, paren);
    def_node__wn_access!(leaf, succ);

}


impl<K, V> Node<K, V> {
    fn search<Q>(&self, k: &Q) -> Option<(Self, usize)>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        if self.is_some() {
            match entries!(self).binary_search_by_key(&k, |ent| ent.0.borrow()) {
                Ok(idx) => Some((self.clone(), idx)),
                Err(idx) => children!(self)[idx].search(k),
            }
        }
        else {
            None
        }
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_some() {
            todo!()
        } else {
            write!(f, "nil")?;
        }

        Ok(())
    }
}
