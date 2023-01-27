use std::{borrow::Borrow, fmt::Debug};

use super::*;



impl_node!();
impl_node_!({ color: Color });
impl_tree!(RB {});

impl_rotate_cleanup!(RB);
impl_balance_validation!(RB ->
    #[cfg(test)]
    fn balance_validation(&mut self) {
        self.root.validate_rb_rule();
        self.root.validate_black_balance();
    }
);


impl<K: Ord, V> RB <K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new() -> Self {
        Self {
            root: Node::none()
        }
    }


    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        let color;

        if self.root.is_none() {
            color = Black;
        }
        else {
            color = Red;
        }

        let z = node!( BST { k, v, color: color });

        let popped = bst_insert!(self, z.clone());

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
            /* 被删除的不能是黑节点 */

            let retracing_entry;
            let origin_color;

            if left!(z).is_none() {
                retracing_entry = right!(z);
                origin_color = z.color();

                subtree_shift!(self, z, right!(z));
            }
            else if right!(z).is_none() {
                retracing_entry = left!(z);
                origin_color = z.color();

                subtree_shift!(self, z, left!(z));
            }
            else {
                /* 这里从颜色关系上假设z没有被删除，被删除得是y (y.color = z.color) */

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
                origin_color = y.color();
                retracing_entry = right!(y);

                if !right!(z).rc_eq(&y) {
                    // replace y with y.right
                    subtree_shift!(self, y, right!(y));

                    // connect z.right to y.right
                    conn_right!(y, right!(z));
                }

                subtree_shift!(self, z, y);
                conn_left!(y, left!(z));

                // 保持替换前原来的颜色
                color!(y, z.color());
            }

            if origin_color.is_black() {
                self.remove_retracing(retracing_entry);
            }

            Some(unboxptr!(unwrap_into!(z).val))
        }
    }


    fn insert_retracing(&mut self, ent: Node<K, V>)
    {
        let mut i = ent;
        let mut p = paren!(i).upgrade();

        if p.is_black() { return }

        debug_assert!(i.is_red());

        /* Both p and pp is RED */

        let pp = paren!(p).upgrade();
        debug_assert!(pp.is_some(), "color red p shouldnt be root");

        let red_dir = index_of_child!(p, i);
        let p_dir = index_of_child!(pp, p);
        let psib_dir = p_dir.rev();

        let psib = child!(pp, psib_dir);

        /* case-1 */

        if psib.is_black() {
            if p_dir == red_dir {
                /* case-3 */

                p = rotate!(self, pp, psib_dir);
            }
            else {
                /* case-2 */

                p = double_rotate!(self, pp, psib_dir);
            }

            p.color_flip();
            pp.color_flip();
        }
        else {  // psib is red

            p.color_flip();
            pp.color_flip();
            psib.color_flip();

            if self.root.is_red() {
                color!(self.root, Black);
            }

            i = pp;
            self.insert_retracing(i);

        }
    }


    fn remove_retracing(&mut self, ent: Node<K, V>) {
        // unimplemented!()
        let mut x = ent;

        while !paren!(x).is_none() && x.is_black() {
            let mut p = paren!(x).upgrade();
            let x_dir = index_of_child!(p, x);
            let sib_dir = x_dir.rev();

            let mut s = child!(p, sib_dir);

            if s.is_red() {
                s.color_flip();
                p.color_flip();

                p = rotate!(self, p, x_dir);
                s = child!(p, sib_dir);
            }

            if left!(s).is_black() && right!(s).is_black() {
                color!(s, Red);
                x = paren!(x).upgrade();
            }
            else {
                if child!(s, sib_dir).is_black() {
                    child!(s, x_dir).color_flip();
                    s.color_flip();

                    rotate!(self, s, x_dir);
                    s = child!(s, x_dir);
                }

                color!(s, p.color());
                color!(p, Black);
                color!(right!(s), Black);
                rotate!(self, p, x_dir);
                x = self.root.clone();
            }
        }

        color!(x, Black);
    }


}


impl<K, V> Node<K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Static Stats

    #[inline]
    fn color(&self) -> Color {
        if self.is_none() {
            Black
        }
        else {
            color!(self)
        }
    }

    #[inline]
    fn is_red(&self) -> bool {
        self.color().is_red()
    }

    #[allow(unused)]
    #[inline]
    fn is_black(&self) -> bool {
        self.color().is_black()
    }

    #[inline]
    fn color_flip(&self) {
        debug_assert!(self.is_some(), "Color flip on None");

        color!(self, self.color().rev())
    }

    ////////////////////////////////////////////////////////////////////////////
    /// Validation Helper

    #[cfg(test)]
    fn validate_rb_rule(&self) {
        if self.is_none() {
            return;
        }

        if self.is_red() {
            assert!(
                paren!(self).upgrade().is_black(),
                "RED VIOLATION (this red paren is root?: {})",
                paren!(paren!(self).upgrade()).is_none()
            )
        }

        left!(self).validate_rb_rule();
        right!(self).validate_rb_rule();
    }

    /// 应该给每个节点校验，但是存储黑高是一个问题，单独搞一个数据结构又太费
    /// 于是多次总体校验的方式
    #[cfg(test)]
    fn validate_black_balance(&self) {
        if self.is_none() {
            return;
        }

        use itertools::Itertools;

        let is_black_blance =
        self
            .leafs()
            .into_iter()
            .map(|x| x.black_depth_to(self))
            .tuples()
            .all(|(a, b)| a == b);

        assert!(is_black_blance);
    }

    #[cfg(test)]
    fn black_depth_to(&self, end: &Self) -> usize {
        let mut depth = 0;
        let mut p = self.clone();

        while p.is_some() && !p.rc_eq(end) {
            if p.is_black() {
                depth += 1;
            }

            p = paren!(p).upgrade();
        }

        depth
    }

    #[cfg(test)]
    fn leafs(&self) -> Vec<Self> {
        let mut leafs = vec![];

        if self.is_none() {
            return leafs;
        }

        let mut q = crate::vecdeq![self.clone()];

        while let Some(x) = q.pop_front() {
            let left = left!(x);
            let right = right!(x);

            if left.is_none() && right.is_none() {
                leafs.push(x);
                continue;
            }

            if left.is_some() {
                q.push_back(left);
            }

            if right.is_some() {
                q.push_back(right!(x));
            }
        }

        leafs
    }

}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_some() {
            write!(f, "{:?}({:?})", key!(self), self.color())
        } else {
            write!(f, "nil({:?})", self.color())
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    /// 这组小数据很有测试价值，能测试单旋和双旋
    #[test]
    fn test_rb2_case_1() {
        let mut dict = RB::<u16, ()>::new();

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
        dict.balance_validation();
    }

    #[test]
    fn test_rb2_random() {
        test_dict!(RB::new());
    }
}
