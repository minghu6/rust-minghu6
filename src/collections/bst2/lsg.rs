//! Lazy Scapegoat Tree
//!


use std::{borrow::Borrow, fmt::Debug};

use super::*;


def_tree!(
    /// Lazy Scapegoat Tree
    LSG
    {
        cnt: usize,
        /// nodes count including marked
        max_cnt: usize,
        alpha: f32
    }
);
impl_tree_debug!(LSG);

impl_node!();
impl_node_!({ size: usize, deleted: bool });
impl_flatten_cleanup!(
    fn flatten_cleanup(&self) {
        if self.is_some() {
            size!(self, 1)
        }
    }
);
impl_build_cleanup!(
    fn build_cleanup(&self) {
        self.update_size()
    }
);
impl_balance_validation!(LSG ->
    #[cfg(test)]
    fn balance_validation(&mut self) {
        self.root.validate_balance(self.alpha);
    }
);


impl<K: Ord, V> LSG <K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new(alpha: f32) -> Self {
        assert!(alpha <= 1.0 && alpha >= 0.5, "bad alpha {alpha}");

        Self {
            root: Node::none(),
            alpha,
            cnt: 0,
            max_cnt: 0
        }
    }


    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        let x = bst_search!(lazy | self.root, k);

        if x.is_some() {
            Some(val!(x))
        }
        else {
            None
        }
    }


    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        let x = bst_search!(lazy | self.root, k);

        if x.is_some() {
            Some(val_mut!(x))
        }
        else {
            None
        }
    }


    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        let z = node!( BST { k, v, size: 1, deleted: false });

        let popped = bst_insert!(lazy | self, z.clone());

        self.insert_retracing(z);

        popped
    }


    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where K: Borrow<Q> + Debug, Q: Ord + ?Sized, V: Debug
    {

        let z = bst_search!(lazy | self.root, k);

        if z.is_none() {
            None
        }
        else {
            let popped = bst_delete!(lazy | z);
            self.cnt -= 1;

            if self.cnt as f32 * self.alpha <= self.max_cnt as f32 {
                self.root = self.rebuild_at(self.root.clone());
            }

            Some(popped)
        }
    }


    /// Bottom up fixing
    fn insert_retracing(&mut self, ent: Node<K, V>)
    {
        let mut p = ent;

        while p.is_some() {
            p.update_size();

            let pp = paren!(p).upgrade();
            let p_dir = if pp.is_some() {
                Some(index_of_child!(pp, p))
            }
            else {
                None
            };

            if p.is_unbalanced(self.alpha) {
                p = self.rebuild_at(p);

                if pp.is_none() {
                    self.root = p;
                    break;
                }
                else {
                    if p_dir.unwrap().is_left() {
                        conn_left!(pp, p);
                    }
                    else {
                        conn_right!(pp, p);
                    }

                    pp.update_size();
                    break;
                }
            }

            p = pp;
        }

    }

    /// Rebuild at p, return new root
    fn rebuild_at(&mut self, p: Node<K, V>) -> Node<K, V> {
        let mut part_nodes: Vec<Node<K, V>> = vec![];
        let mut dead_nodes = 0;

        for x in bst_flatten!(p) {
            if !deleted!(x) {
                part_nodes.push(x);
            }
            else {
                dead_nodes += 1;
            }
        }
        self.max_cnt -= dead_nodes;

        bst_build!(&part_nodes[..])
    }

}



impl<K, V> Node<K, V> {
    fn update_size(&self) {
        if self.is_some() {
            size!(
                self,
                1 + left!(self).size() + right!(self).size()
            );
        }
    }

    fn size(&self) -> usize {
        if self.is_some() {
            size!(self)
        }
        else {
            0
        }
    }

    fn is_unbalanced(&self, alpha: f32) -> bool {
        let left_cover = left!(self).size() as f32 / self.size() as f32;
        let right_cover = right!(self).size() as f32 / self.size() as f32;

        left_cover > alpha || right_cover > alpha
    }

    /// Loosely alpha-height balanced
    #[cfg(test)]
    fn validate_balance(&self, _alpha: f32) {
        // if self.is_some() {
        //     assert!(self.is_unbalanced(alpha));

        //     left!(self).validate_balance(alpha);
        //     right!(self).validate_balance(alpha);
        // }
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(
                f,
                "{:?}(sz: {}, {})",
                key!(self),
                size!(self),
                if deleted!(self) { "x" } else { "" }
            )
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
    fn test_lsg2_case_1() {
        let mut dict = LSG::<i32, ()>::new(0.7);

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

        // dict.debug_print();
    }


    #[test]
    fn test_lsg2_random() {
        test_dict!(LSG::new(0.6));
    }

}
