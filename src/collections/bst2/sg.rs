//! Scapegoat Tree
//!


use std::{borrow::Borrow, fmt::Debug, cmp::max};

use super::*;


impl_tree!(
    /// Scapegoat Tree
    SG
    {
        cnt: usize,
        /// nodes count including marked
        max_cnt: usize,
        alpha: f32
    }
);
impl_node!();
impl_node_!({ size: usize });
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
impl_balance_validation!(
    SG ->
    fn balance_validation(&self) {}
);


impl<K: Ord, V> SG <K, V> {

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


    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        let z = node!( BST { k, v, size: 1 });

        let popped = bst_insert!(self, z.clone());

        if popped.is_none() {
            self.cnt += 1;
            self.max_cnt = max(self.cnt, self.max_cnt);
        }

        self.insert_retracing(z);

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
            bst_delete!(self, z);
            self.cnt -= 1;

            if self.cnt as f32 * self.alpha <= self.max_cnt as f32 {
                if self.root.is_some() {
                    self.root = self.rebuild_at(self.root.clone());
                }
            }

            Some(unwrap_into!(z).into_value())
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
        bst_build!(&bst_flatten!(p))
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

    #[allow(unused)]
    #[cfg(test)]
    fn validate_balance(&self, _alpha: f32) {
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(
                f,
                "{:?}(sz: {})",
                key!(self),
                size!(self),
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
    fn test_sg2_case_1() {
        let mut dict = SG::<i32, ()>::new(0.7);

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
    fn test_sg2_random() {
        test_dict!(SG::new(0.6));
    }

}
