use std::mem::replace;

use super::aux::*;

pub mod bt;
pub mod bpt;
pub mod bpt2;


/// O(M)
macro_rules! index_of_child_by_rc {
    ($p: expr, $child: expr) => {{
        let p = &$p;
        let child = &$child;

        debug_assert!(child.is_some());

        if let Some(idx) = children!(p).iter().position(|x| x.rc_eq(child)) {
            idx
        }
        else {
            unreachable!("There are no matched child");
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
        $(#[$attr])*
        #[allow(unused)]
        pub struct $treename<K, V, const M: usize> {
            root: Node<K, V>,

            /* extra attr */
            $(
                $(#[$field_attr])*
                $name: $ty
            ),*
        }
        impl<K, V, const M: usize> $treename<K, V, M> {
            const fn entries_low_bound() -> usize {
                M.div_ceil(2) - 1
            }

            const fn entries_high_bound() -> usize {
                M
            }
        }
    };
}


use index_of_child_by_rc;
use impl_tree;


#[cfg(test)]
mod tests {
    macro_rules! dict_insert {
        ($dict:ident, $num:expr) => {
            $dict.insert($num, $num);
            assert!($dict.get(&$num).is_some());
            $dict.validate();
        };
    }

    macro_rules! dict_remove {
        ($dict:ident, $num:expr) => {
            assert_eq!($dict.get(&$num), Some(&$num));
            assert_eq!($dict.remove(&$num), Some($num));
            assert!($dict.get(&$num).is_none());
            $dict.validate();
        };
    }

    pub(super) use dict_insert;
    pub(super) use dict_remove;
}
