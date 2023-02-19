use std::mem::replace;

use super::aux::*;

pub mod bt;


def_attr_macro!(clone|
    paren
);
def_attr_macro!(ref|
    (children, Vec<Node<K, V>>)
);



/// key-idx
macro_rules! right {
    ($x:expr, $idx:expr) => {
        &children!($x)[$idx+1]
    };
}


/// key-idx
#[allow(unused)]
macro_rules! left {
    ($x:expr, $idx:expr) => {
        &children!($x)[$idx]
    };
}


macro_rules! last_child {
    ($x:expr) => {
        children!($x).last().unwrap()
    };
}


macro_rules! first_child {
    ($x:expr) => {
        children!($x).first().unwrap()
    };
}


macro_rules! children_revref {
    ($x:expr) => {
        {
            let x = &$x;
            let children = children_mut!(x);

            if children[0].is_some() {
                for child in children {
                    paren!(child, x.downgrade());
                }
            }
        }
    };
}


/// O(logM)
macro_rules! index_of_child {
    ($p: expr, $child: expr) => {{
        let p = &$p;
        let child = &$child;

        debug_assert!(child.is_some());

        // if let Some(idx) = children!(p).iter().position(|x| x.rc_eq(child)) {
        //     idx
        // }
        // else {
        //     unreachable!("There are no matched child");
        // }
        match entries!(p).binary_search(child.last_entry()) {
            Ok(oldidx) => {
                unreachable!("Dup key on {oldidx}");
            },
            Err(inseridx) => {
                inseridx
            },
        }
    }};
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
        pub struct $treename<K, V, const M: usize> {
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
        impl<K: Ord, V, const M: usize> $treename<K, V, M> {
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

                let mut this_q = crate::vecdeq![vec![self.root.clone()]];
                let mut lv = 1;

                while !this_q.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "############ Level: {lv} #############")?;
                    writeln!(f)?;

                    let mut nxt_q = crate::vecdeq![];

                    while let Some(children) = this_q.pop_front() {
                        for (i, x) in children.iter().enumerate() {
                            let p = paren!(x).upgrade();

                            if x.is_some() && children!(x)[0].is_some() {
                                nxt_q.push_back(children!(x).clone());
                            }

                            writeln!(f, "({i:02}): {x:?} (p: {p:?})")?;
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


#[allow(unused)]
use left;
use right;
use last_child;
use first_child;
use children_revref;
use index_of_child;
use impl_tree;
use def_tree;
use impl_tree_debug;




// pub fn ordered_insert<T: Ord>(vec: &mut Vec<T>, x: T) -> Option<T> {
//     match vec.binary_search(&x) {
//         Ok(oldidx) => {
//             Some(replace(&mut vec[oldidx], x))
//         },
//         Err(inseridx) => {
//             vec.insert(inseridx, x);
//             None
//         },
//     }
// }