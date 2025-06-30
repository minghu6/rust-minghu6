#![feature(int_roundings)]
#![feature(macro_metavar_expr)]
#![feature(iter_from_coroutine)]
#![feature(coroutines)]
#![feature(let_chains)]
#![feature(stmt_expr_attributes)]
#![feature(maybe_uninit_slice)]
#![feature(mem_copy_fn)]
#![feature(box_into_inner)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trace_macros)]
#![feature(trait_alias)]
#![feature(slice_range)]

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Index,
};


pub mod bst;
pub mod bt;


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait WalkTree<'a> {
    type Node: Borrow<Self::NodeBorrow>;
    type NodeBorrow;

    fn root(&'a self) -> Option<&'a Self::NodeBorrow>;
    fn children(
        &'a self,
        ptr: &'a Self::NodeBorrow,
    ) -> impl Iterator<Item = &'a Self::NodeBorrow>;

    fn pre_order_walk(
        &'a self,
    ) -> impl Iterator<Item = (LocOnTree, &'a Self::NodeBorrow)> + 'a {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                let Some(root) = self.root() else {
                    return;
                };

                let mut loc = LocOnTree::new();
                let mut curlv = vec![vec![root]];

                while !curlv.is_empty() {
                    loc.ln += 1;
                    loc.col_group = 0;

                    let mut nextlv = vec![];

                    for child_group in curlv.into_iter() {
                        loc.col_group += 1;
                        loc.in_group_id = 0;

                        for child in child_group {
                            loc.in_group_id += 1;

                            yield (loc, child);

                            let nxt_child_group =
                                self.children(child).collect::<Vec<_>>();

                            if !nxt_child_group.is_empty() {
                                nextlv.push(nxt_child_group);
                            }
                        }
                    }

                    curlv = nextlv;
                }
            },
        )
    }

    fn display_fault(
        &'a self,
        node: &'a Self::NodeBorrow,
        reason: &str,
    ) -> DisplayFaultNodeOfTree<'a, Self, Self::NodeBorrow> {
        let tree = self;
        let reason = reason.to_string();

        DisplayFaultNodeOfTree { tree, node, reason }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct PreOrderView<'a, NB> {
    /// persisitent sequence
    pseq: Vec<(LocOnTree, &'a NB)>,
    locmap: HashMap<*const NB, LocOnTree>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct LocOnTree {
    pub ln: usize,
    pub col_group: usize,
    pub in_group_id: usize,
}

pub struct DisplayLocOnTree<'a, T> {
    pub revref: &'a T,
    pub max_ln_width: usize,
    pub max_col_group_width: usize,
    pub max_in_group_width: usize,
}

pub struct DisplayFaultNodeOfTree<'a, T: ?Sized, N> {
    tree: &'a T,
    node: &'a N,
    reason: String,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<'a, T: WalkTree<'a, NodeBorrow = N> + Display, N> Display
    for DisplayFaultNodeOfTree<'a, T, N>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let view = self.tree.pre_order_walk().collect::<PreOrderView<_>>();
        let node_loc = view[&self.node];

        writeln!(f, "[{node_loc:?}, {}]: {}", self.reason, self.tree)
    }
}

impl<'a, T> DisplayLocOnTree<'a, T> {
    fn display(&self, loc: &LocOnTree) -> String {
        format!(
            "{:0ln_width$}.{:0col_group_width$}.{:0in_group_width$}",
            loc.ln,
            loc.col_group,
            loc.in_group_id,
            ln_width = self.max_ln_width,
            col_group_width = self.max_col_group_width,
            in_group_width = self.max_in_group_width
        )
    }

    #[cfg(test)]
    fn display_fault(&self, loc: &LocOnTree) -> String
    where
        T: Display,
    {
        format!("\n[{}]: {}", self.display(loc), self.revref)
    }

    #[cfg(test)]
    fn display_pass(&self) -> String
    where
        T: Display,
    {
        format!("\n[PASS]: {}", self.revref)
    }
}

impl<'a, NB> PreOrderView<'a, NB> {
    pub fn iter(&'a self) -> impl Iterator<Item = (LocOnTree, &'a NB)> + 'a {
        self.pseq.iter().cloned()
    }
}

impl<'a, NB> FromIterator<(LocOnTree, &'a NB)> for PreOrderView<'a, NB> {
    fn from_iter<T: IntoIterator<Item = (LocOnTree, &'a NB)>>(iter: T) -> Self {
        let pseq = iter.into_iter().collect::<Vec<_>>();

        let locmap = HashMap::from_iter(
            pseq.iter().cloned().map(|(_0, _1)| (_1 as _, _0)),
        );

        Self { pseq, locmap }
    }
}

impl<'a, NB> Index<&NB> for PreOrderView<'a, NB> {
    type Output = LocOnTree;

    fn index(&self, index: &NB) -> &Self::Output {
        &self.locmap[&(index as _)]
    }
}

impl LocOnTree {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Debug for LocOnTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.ln, self.col_group, self.in_group_id,)
    }
}
