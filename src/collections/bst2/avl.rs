use std::{cmp::max, borrow::Borrow, fmt::Debug};

use super::*;



impl_node!();
impl_node_!({ height: i32 });
impl_tree!(AVL {});

impl_rotate_cleanup!(AVL ->
    fn rotate_cleanup(&self, x: Node<K, V>, z: Node<K, V>) {
        /* update height */
        x.update_height();
        z.update_height();
    }
);
impl_balance_validation!(AVL ->
    #[cfg(test)]
    fn balance_validation(&mut self) {
        self.root.recalc_height();
        self.root.validate_bf();
    }
);


impl<K: Ord, V> AVL<K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new() -> Self {
        Self {
            root: Node::none()
        }
    }


    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        let z = node!( BST { k, v, height: 1 });

        let popped = bst_insert!(self, z.clone());

        // self.insert_retracing(z);
        self.retracing(z);

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
            let retracing_entry = bst_delete!(self, z);
            self.retracing(retracing_entry);

            Some(unboxptr!(unwrap_into!(z).val))
        }
    }


    ////////////////////////////////////////////////////////////////////////////
    /// Helper Method

    /// Simplified version of retracing
    #[allow(unused)]
    fn insert_retracing(&mut self, mut y: Node<K, V>)
    {
        /* x
           |
           z
           |
           y
        */

        let mut z = paren!(y).upgrade();

        while z.is_some() {
            z.update_height();

            let x = paren!(z).upgrade();

            if x.bf().abs() > 1 {
                let index_of_z = index_of_child!(x, z);
                let index_of_y = index_of_child!(z, y);

                if index_of_z == index_of_y {
                    z = rotate!(self, x, index_of_z.rev());
                }
                else {
                    z = double_rotate!(self, x, index_of_z.rev());
                }
            }

            y = z;
            z = paren!(y).upgrade();
        }

    }


    /// Bottom up fixing
    fn retracing(&mut self, ent: Node<K, V>)
    {
        let mut p = ent;

        while p.is_some() {
            p.update_height();

            if p.bf().abs() > 1 {
                let high =
                if right!(p).height() > left!(p).height() {
                    Right
                }
                else {
                    Left
                };

                let z = child!(p, high);

                if child!(z, high).height() >= child!(z, high.rev()).height() {
                    p = rotate!(self, p, high.rev());
                }
                else {
                    p = double_rotate!(self, p, high.rev());
                }
            }

            p = paren!(p).upgrade();
        }

    }

}


impl<K, V> Node<K, V> {

    fn update_height(&self) {
        if self.is_some() {
            height!(
                self,
                1 + max(
                    left!(self).height(),
                    right!(self).height()
                )
            );
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    /// Static Stats

    fn height(&self) -> i32 {
        if self.is_none() {
            0
        }
        else {
            height!(self)
        }
    }

    fn bf(&self) -> i32 {
        if self.is_none() {
            0
        }
        else {
            right!(self).height() - left!(self).height()
        }
    }


    ////////////////////////////////////////////////////////////////////////////
    /// Validation Helper

    /// Recursively validate BF:
    ///
    /// BF(X): H(right(X)) - H(left(X))
    ///
    /// BF(X) in {-1, 0, 1}
    ///
    #[cfg(test)]
    fn validate_bf(&self) {
        assert!(
            self.bf().abs() < 2
        );

        if self.is_some() {
            left!(self).validate_bf();
            right!(self).validate_bf();
        }
    }

    /// Recursively calculate height stats in time instead of using static height
    #[cfg(test)]
    fn recalc_height(&self) {
        if self.is_some() {
            left!(self).recalc_height();
            right!(self).recalc_height();

            self.update_height();
        }
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(f, "{:?}(h: {})", key!(self), height!(self))
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
    fn test_avl2_case_1() {
        let mut dict = AVL::<u16, ()>::new();

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
    }

    #[test]
    fn test_avl2_random() {
        test_dict!(AVL::new());
    }
}

