use std::mem::replace;

use super::aux::*;

pub mod bt;


////////////////////////////////////////////////////////////////////////////////
//// Macros

////////////////////////////////////////
//// Node Operations


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
            // pub fn get<Q>(&self, k: &Q) -> Option<&V>
            // where K: std::borrow::Borrow<Q>, Q: Ord + ?Sized
            // {
            //     let x = bst_search!(self.root, k);

            //     if x.is_some() {
            //         Some(val!(x))
            //     }
            //     else {
            //         None
            //     }
            // }

            // pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
            // where K: std::borrow::Borrow<Q>, Q: Ord + ?Sized
            // {
            //     let x = bst_search!(self.root, k);

            //     if x.is_some() {
            //         Some(val_mut!(x))
            //     }
            //     else {
            //         None
            //     }
            // }
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
                        // if left!(x).is_none() && right!(x).is_none() {
                        //     write!(f, "{x:?}")?;
                        // }
                        // else {
                        //     write!(f, "{x:?} | L-> ")?;

                        //     let left = left!(x);
                        //     if left.is_some() {
                        //         write!(f, "{left:?}")?;
                        //         nxt_q.push_back(left);
                        //     }
                        //     else {
                        //         write!(f, "nil")?;
                        //     }

                        //     write!(f, "; R-> ")?;

                        //     let right = right!(x);
                        //     if right.is_some() {
                        //         write!(f, "{right:?}")?;
                        //         nxt_q.push_back(right);
                        //     }
                        //     else {
                        //         write!(f, "nil")?;
                        //     }
                        // }

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


use impl_tree;
use def_tree;
use impl_tree_debug;




pub fn ordered_insert<T: Ord>(vec: &mut Vec<T>, x: T) -> Option<T> {
    match vec.binary_search(&x) {
        Ok(oldidx) => {
            Some(replace(&mut vec[oldidx], x))
        },
        Err(inseridx) => {
            vec.insert(inseridx, x);
            None
        },
    }
}
