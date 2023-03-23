//! Red-black tree && Left-learning Red-black tree (estimated)

use std::{borrow::Borrow, fmt::Debug};

use super::*;


impl_node!();
impl_node_!({ color: Color });


impl_tree!(RB {});
// impl_tree!(LLRB {});


impl_rotate_cleanup!(RB ->
    fn rotate_cleanup(&self, x: Node<K, V>, z: Node<K, V>) {
        /* swap color */
        if x.is_some() && z.is_some() {
            swap!(node | x, z, color, Color);
        }
        else {
            debug_assert!(x.is_black() && z.is_black());
        }
    }
);
impl_validate!(RB ->
    #[cfg(test)]
    fn validate(&mut self)
    where K: Debug
    {
        debug_assert!(self.root.is_black(), "[validate] root should be black");

        self.root.validate_rb_rule();
        self.root.validate_black_balance();
    }
);


// impl_rotate_cleanup!(LLRB ->
//     #[allow(unused)]
//     fn rotate_cleanup(&self, x: Node<K, V>, z: Node<K, V>) {
//         /* swap color */
//         if x.is_some() && z.is_some() {
//             swap!(node | x, z, color);
//         }
//         else {
//             debug_assert!(x.is_black() && z.is_black());
//         }
//     }
// );
// impl_validate!(LLRB ->
//     #[cfg(test)]
//     fn validate(&mut self)
//     where K: Debug
//     {
//         debug_assert!(self.root.is_black(), "[validate] root should be black");

//         self.root.validate_rb_rule();
//         self.root.validate_black_balance();
//     }
// );


// impl<K: Ord, V> LLRB <K, V> {

//     ////////////////////////////////////////////////////////////////////////////
//     /// Public API

//     pub fn new() -> Self {
//         Self {
//             root: Node::none()
//         }
//     }


//     pub fn insert(&mut self, k: K, v: V) -> Option<V>
//     {
//         let color;

//         if self.root.is_none() {
//             color = Black;
//         }
//         else {
//             color = Red;
//         }

//         let z = node!( BST { k, v, color: color });

//         let popped = bst_insert!(self, z.clone());

//         self.fix_red_violation(z);

//         popped
//     }

// }



impl<K: Ord, V> RB <K, V> {

    ////////////////////////////////////////////////////////////////////////////
    /// Public API

    pub fn new() -> Self {
        Self {
            root: Node::none()
        }
    }


    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where V: Default
    {
        let color;

        if self.root.is_none() {
            color = Black;
        }
        else {
            color = Red;
        }

        let z = node!({ k, v, color: color });

        let popped = bst_insert!(self, z.clone());

        self.fix_red_violation(z);

        popped
    }


    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where K: Borrow<Q> + Debug, Q: Ord + ?Sized, V: Debug
    {

        let mut z = bst_search!(self.root, k);

        if z.is_none() {
            None
        }
        else {
            if left!(z).is_some() && right!(z).is_some() {
                let successor = bst_minimum!(right!(z));
                fake_swap!(z, successor);
                z = successor;
            }

            let p = paren!(z).upgrade();
            let z_dir = if p.is_some() { Some(index_of_child!(p, z)) } else { None };

            let child = child!(z, if left!(z).is_none() { Right } else { Left });
            subtree_shift!(self, z, child);

            if z.is_black() {
                if child.is_red() {
                    color!(child, Black);
                }
                else if let Some(z_dir) = z_dir {
                    self.fix_double_black(z_dir, p);
                }
            }

            Some(unwrap_into!(z).into_value())
        }
    }


    fn fix_red_violation(&mut self, ent: Node<K, V>)
    {
        let mut i = ent;
        let p = paren!(i).upgrade();

        if p.is_black() { return }

        debug_assert!(i.is_red());

        /* Both p and pp is RED */

        let pp = paren!(p).upgrade();
        debug_assert!(pp.is_some(), "[insert] color red p shouldnt be root");

        let red_dir = index_of_child!(p, i);
        let p_dir = index_of_child!(pp, p);
        let psib_dir = p_dir.rev();

        let psib = child!(pp, psib_dir);

        /* case-1 */

        if psib.is_black() {
            if p_dir == red_dir {
                /* case-3 */
                rotate!(self, pp, psib_dir);
            }
            else {
                /* case-2 */
                double_rotate!(self, pp, psib_dir);
            }
        }
        else {  // psib is red
            p.color_flip();
            pp.color_flip();
            psib.color_flip();

            if self.root.is_red() {
                color!(self.root, Black);
            }

            i = pp;
            self.fix_red_violation(i);

        }
    }

    /// Refer to my blog (BST(2) - RB(0) - 原始红黑树)
    fn fix_double_black(&mut self, x_dir: Dir, p: Node<K, V>) {
        debug_assert!(p.is_some());

        let sib_dir = x_dir.rev();
        let mut sib = child!(p, sib_dir);
        let mut sib_c = child!(sib, x_dir);
        let mut sib_d = child!(sib, sib_dir);

        /* case-5 */
        if sib.is_red() {
            rotate!(self, p, x_dir);

            sib = sib_c;
            sib_c = child!(sib, x_dir);
            sib_d = child!(sib, sib_dir);
        }

        debug_assert!(sib.is_black());

        macro_rules! case_3 {
            (p=$p:ident, x_dir=$x_dir:ident, sib_d=$sib_d:ident) => {
                rotate!(self, $p, $x_dir);
                $sib_d.color_flip();
            };
        }

        /* case-3 */
        if sib_d.is_red() {
            case_3!(p=p, x_dir=x_dir, sib_d=sib_d);
        }
        /* case-4 */
        else if sib_c.is_red() {
            rotate!(self, sib, sib_dir);
            case_3!(p=p, x_dir=x_dir, sib_d=sib);
        }
        /* case-2 */
        else if p.is_red() {
            swap!(node | p, sib, color, Color);
        }
        /* case-1 */
        else {
            sib.color_flip();

            let pp = paren!(p).upgrade();
            if pp.is_some() {
                self.fix_double_black(index_of_child!(pp, p), pp);
            }
        }
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
            debug_assert!(
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
    fn validate_black_balance(&self)
    where K: Debug
    {
        if self.is_none() {
            return;
        }

        use common::Itertools;

        let is_black_blance =
        self
            .leaves()
            .into_iter()
            .map(|x| x.black_depth_to(self))
            // .inspect(|x| print!("{x}, "))
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

    /// store paren for nil leaf
    #[cfg(test)]
    fn leaves(&self) -> Vec<Self>
    where K: Debug
    {
        let mut leaves = vec![];

        if self.is_none() {
            return leaves;
        }

        let mut q = common::vecdeq![self.clone()];

        while let Some(x) = q.pop_front() {
            let left = left!(x);
            let right = right!(x);

            // 两片本质一样的叶子只保留其中一页
            if left.is_none() || right.is_none() {
                leaves.push(x.clone());
            }

            if left.is_some() {
                q.push_back(left);
            }

            if right.is_some() {
                q.push_back(right!(x));
            }
        }

        leaves
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
    fn test_bst_rb_case_1() {
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
        dict.validate();

        dict.remove(&24);
        assert!(dict.get(&24).is_none());
        dict.validate();

        dict.remove(&47);
        assert!(dict.get(&47).is_none());
        dict.validate();

        dict.remove(&52);
        assert!(dict.get(&52).is_none());
        dict.validate();

        dict.remove(&3);
        assert!(dict.get(&3).is_none());
        dict.validate();

        assert!(dict.get(&35).is_some());
        // dict.remove(&35);
        // assert!(dict.get(&35).is_none());
        // dict.validate();

        // dict.debug_print();

    }

    #[test]
    fn test_bst_rb_random() {
        test_dict!(RB::new());
    }
}
