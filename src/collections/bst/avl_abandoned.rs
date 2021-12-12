use std::cell::RefCell;
use std::fmt::Debug;
use std::sync;
use std::sync::Arc;

use either::{self, Either};

use super::BSTKey;
use crate::collections::Adictionary;


/// ```no_run
/// By two Soviet inventors, Georgy Adelson-Velsky and Evgenii Landis(1962)
/// This is Sync Version
/// ref: https://en.wikipedia.org/wiki/AVL_tree
/// ref: https://en.wikipedia.org/wiki/Binary_search_tree
/// ```


pub struct Aavl<K: BSTKey, V> {
    root: Option<AavlNodeRc<K, V>>,
}


impl<K: BSTKey, V> Aavl<K, V> {
    pub fn new() -> Self {
        Self::default()
    }
}


impl<K: BSTKey, V> Default for Aavl<K, V> {
    fn default() -> Self {
        Self {
            root: Default::default(),
        }
    }
}


pub struct Aavlnode<K: BSTKey, V> {
    left: Option<Arc<RefCell<Self>>>,
    right: Option<Arc<RefCell<Self>>>,
    paren: Option<sync::Weak<RefCell<Self>>>,
    bf: BF, // BF = H(right) - H(left)
    key: Arc<K>,
    value: Arc<V>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum BF {
    N1, // Negative one
    Z,  // Zero
    P1, // Positive one
}




type AavlNodeRc<K, V> = Arc<RefCell<Aavlnode<K, V>>>;
type AavlNodeWk<K, V> = sync::Weak<RefCell<Aavlnode<K, V>>>;


/// Basic Operation

#[inline]
fn aavl_key<K: BSTKey, V>(x: &AavlNodeRc<K, V>) -> Arc<K> {
    x.as_ref().borrow().key.clone()
}

#[inline]
fn aavl_val<K: BSTKey, V>(x: &AavlNodeRc<K, V>) -> Arc<V> {
    x.as_ref().borrow().value.clone()
}


#[inline]
fn aavl_left_child<K: BSTKey, V>(
    x: &AavlNodeRc<K, V>,
) -> Option<AavlNodeRc<K, V>> {
    x.as_ref().borrow().left.clone()
}

#[inline]
fn aavl_right_child<K: BSTKey, V>(
    x: &AavlNodeRc<K, V>,
) -> Option<AavlNodeRc<K, V>> {
    x.as_ref().borrow().right.clone()
}

#[inline]
fn aavl_bf<K: BSTKey, V>(x: &AavlNodeRc<K, V>) -> BF {
    x.as_ref().borrow().bf
}

#[inline]
fn aavl_paren<K: BSTKey, V>(x: &AavlNodeRc<K, V>) -> Option<AavlNodeWk<K, V>> {
    x.as_ref().borrow().paren.clone()
}

#[inline]
fn set_aavl_left_child<K: BSTKey, V>(
    x: &AavlNodeRc<K, V>,
    val: Option<AavlNodeRc<K, V>>,
) {
    x.as_ref().borrow_mut().left = val;
}


#[inline]
fn set_aavl_right_child<K: BSTKey, V>(
    x: &AavlNodeRc<K, V>,
    val: Option<AavlNodeRc<K, V>>,
) {
    x.as_ref().borrow_mut().right = val;
}

#[inline]
fn set_aavl_paren<K: BSTKey, V>(
    x: &AavlNodeRc<K, V>,
    val: Option<AavlNodeWk<K, V>>,
) {
    x.as_ref().borrow_mut().paren = val;
}

#[inline]
fn set_aavl_bf<K: BSTKey, V>(x: &AavlNodeRc<K, V>, val: BF) {
    x.as_ref().borrow_mut().bf = val;
}

// #[inline]
// fn eq_aavl_node<K: BSTKey, V>(
//     x: &AavlNodeRc<K, V>,
//     y: &AavlNodeRc<K, V>,
// ) -> bool {
//     Arc::ptr_eq(x, y)
// }


/// Simple Rotation
/// ```no_run
///             rotate left
///    x        =========>          z
///  /  \                          / \
/// t1   z                        x   t4
/// |   / \                      / \   |
///   t23 t4                    t1 t23 |
///     |  |                     |   |
///        |
/// ```
///
fn aavl_rotate<K: BSTKey, V>(
    x_node: &AavlNodeRc<K, V>,
    z_node: &AavlNodeRc<K, V>,
    rorate_orin: Either<(), ()>,
) -> AavlNodeRc<K, V> {
    let t23_opt = if rorate_orin.is_left() {
        aavl_left_child(&z_node)
    } else {
        aavl_right_child(&z_node)
    };

    if rorate_orin.is_left() {
        set_aavl_right_child(&x_node, t23_opt.clone());
    } else {
        set_aavl_left_child(&x_node, t23_opt.clone());
    }

    if let Some(t23) = t23_opt {
        set_aavl_paren(&t23, Some(Arc::downgrade(&x_node)));
    }

    if rorate_orin.is_left() {
        set_aavl_left_child(&z_node, Some(x_node.clone()));
    } else {
        set_aavl_right_child(&z_node, Some(x_node.clone()));
    }

    set_aavl_paren(&x_node, Some(Arc::downgrade(&z_node)));


    // case-1. only happens with deletion
    if aavl_bf(&z_node) == BF::Z {
        if rorate_orin.is_left() {
            set_aavl_bf(&x_node, BF::P1); // t23 now higher
            set_aavl_bf(&z_node, BF::N1); // t4 now lower than x
        } else {
            set_aavl_bf(&x_node, BF::N1);
            set_aavl_bf(&z_node, BF::P1);
        }
    }
    // case-2 happends with both insertion and deletion
    else {
        set_aavl_bf(&x_node, BF::Z);
        set_aavl_bf(&z_node, BF::Z);
    }

    z_node.clone()
}


/// Double Rotation
/// ```no_run
///             rotate [right]-left         rotate right-[left]
///    x        =========>         x        =========>       y
///  /   \                        /  \                      / \
/// t1    z                      t1   y                    x   z
/// |   /  \                     |   / \                  / \ / \
///    y   t4                      t2   z                t1 t2t3t4
///   / \   |                       |  / \                |  | | |
///  t2 t3                            t3 t4
///   |  |                            |   |
/// ```
fn aavl_double_rotate<K: BSTKey, V>(
    x_node: &AavlNodeRc<K, V>,
    z_node: &AavlNodeRc<K, V>,
    snd_rorate_orin: Either<(), ()>,
) -> AavlNodeRc<K, V> {
    /* FIRST ROTATION */

    // z is by 2 higher than its sibing(t1)
    // y is by 1 higher than its sibling(t4) (thereis shouldn't be empty)
    let y_node = if snd_rorate_orin.is_left() {
        aavl_left_child(&z_node).unwrap()
    } else {
        aavl_right_child(&z_node).unwrap()
    };

    let t3_opt = if snd_rorate_orin.is_left() {
        aavl_right_child(&y_node)
    } else {
        aavl_left_child(&y_node)
    };

    if snd_rorate_orin.is_left() {
        set_aavl_left_child(&z_node, t3_opt.clone());
    } else {
        set_aavl_right_child(&z_node, t3_opt.clone());
    }

    if let Some(ref t3) = t3_opt {
        set_aavl_paren(&t3, Some(Arc::downgrade(&z_node)));
    }

    if snd_rorate_orin.is_left() {
        set_aavl_right_child(&y_node, Some(z_node.clone()));
    } else {
        set_aavl_left_child(&y_node, Some(z_node.clone()));
    }
    set_aavl_paren(&z_node, Some(Arc::downgrade(&y_node)));


    let t2_opt = if snd_rorate_orin.is_left() {
        aavl_left_child(&y_node)
    } else {
        aavl_right_child(&y_node)
    };


    /* SECOND ROTATION */
    if snd_rorate_orin.is_left() {
        set_aavl_right_child(&x_node, t2_opt.clone());
    } else {
        set_aavl_left_child(&x_node, t2_opt.clone());
    }
    if let Some(ref t2) = t2_opt {
        set_aavl_paren(&t2, Some(Arc::downgrade(&x_node)));
    }

    if snd_rorate_orin.is_left() {
        set_aavl_left_child(&y_node, Some(x_node.clone()));
    } else {
        set_aavl_right_child(&y_node, Some(x_node.clone()))
    }

    set_aavl_paren(&x_node, Some(Arc::downgrade(&y_node)));


    /* Rejudge BF */
    // h(t1) = h(t2); h(t3) = h(t4); h(t2) ?= h(t3/|)
    // case-1. only happends with deletion of t1
    if aavl_bf(&y_node) == BF::Z {
        set_aavl_bf(&x_node, BF::Z);
        set_aavl_bf(&z_node, BF::Z);
    } else {
        // `cond` indicates that if t3 was heigher
        let cond = if snd_rorate_orin.is_left() {
            BF::P1
        } else {
            BF::N1
        };

        if aavl_bf(&y_node) == cond {
            set_aavl_bf(&x_node, BF::N1);
            set_aavl_bf(&z_node, BF::Z);
        } else {
            set_aavl_bf(&x_node, BF::Z);
            set_aavl_bf(&z_node, BF::P1);
        }
    }

    set_aavl_bf(&y_node, BF::Z);


    y_node.clone()
}


#[inline]
fn aavl_rotate_left<K: BSTKey, V>(
    x_node: &AavlNodeRc<K, V>,
    z_node: &AavlNodeRc<K, V>,
) -> AavlNodeRc<K, V> {
    aavl_rotate(x_node, z_node, Either::Left(()))
}


#[inline]
fn aavl_rotate_right<K: BSTKey, V>(
    x_node: &AavlNodeRc<K, V>,
    z_node: &AavlNodeRc<K, V>,
) -> AavlNodeRc<K, V> {
    aavl_rotate(x_node, z_node, Either::Right(()))
}

#[inline]
fn aavl_rotate_right_left<K: BSTKey, V>(
    x_node: &AavlNodeRc<K, V>,
    z_node: &AavlNodeRc<K, V>,
) -> AavlNodeRc<K, V> {
    aavl_double_rotate(x_node, z_node, Either::Left(()))
}


#[inline]
fn aavl_rotate_left_right<K: BSTKey, V>(
    x_node: &AavlNodeRc<K, V>,
    z_node: &AavlNodeRc<K, V>,
) -> AavlNodeRc<K, V> {
    aavl_double_rotate(x_node, z_node, Either::Right(()))
}

#[inline]
fn aavl_search_approximately<K: BSTKey, V>(
    key: &K,
    root: &AavlNodeRc<K, V>,
) -> AavlNodeRc<K, V> {
    let root_ref = root.as_ref().borrow();

    if *key == *root_ref.key {
        root.clone()
    } else if *key < *root_ref.key {
        if root_ref.left.is_none() {
            root.clone()
        } else {
            aavl_search_approximately(key, &root_ref.left.as_ref().unwrap())
        }
    } else {
        if root_ref.right.is_none() {
            root.clone()
        } else {
            aavl_search_approximately(key, &root_ref.right.as_ref().unwrap())
        }
    }
}

#[allow(unused)]
#[inline]
fn aavl_maximum<K: BSTKey, V>(x_node: &AavlNodeRc<K, V>) -> AavlNodeRc<K, V> {
    let mut x = x_node.clone();

    while let Some(ref x_0) = aavl_right_child(&x) {
        x = x_0.clone()
    }

    x.clone()
}

#[inline]
fn aavl_minimum<K: BSTKey, V>(x_node: &AavlNodeRc<K, V>) -> AavlNodeRc<K, V> {
    let mut x = x_node.clone();

    while let Some(ref x_0) = aavl_left_child(&x) {
        x = x_0.clone()
    }

    x.clone()
}


#[inline]
fn aavl_height<K: BSTKey, V>(x_opt: &Option<AavlNodeRc<K, V>>) -> i32 {
    fn aavl_height_<K: BSTKey, V>(x_opt: &Option<AavlNodeRc<K, V>>) -> i32 {
        if x_opt.is_none() {
            return -1;
        }

        let x = x_opt.clone().unwrap();

        let h_lf = aavl_height_(&aavl_left_child(&x));
        let h_rh = aavl_height_(&aavl_right_child(&x));

        if h_lf > h_rh {
            h_lf + 1
        } else {
            h_rh + 1
        }
    }

    aavl_height_(x_opt)
}


/// Smallest key greater than x.key
#[inline]
fn aavl_successor<K: BSTKey, V>(x: &AavlNodeRc<K, V>) -> AavlNodeRc<K, V> {
    if aavl_right_child(x).is_some() {
        return aavl_minimum(&aavl_right_child(x).unwrap());
    }

    let mut x = x.clone();
    let mut y = aavl_paren(&x);
    while y.is_some()
        && Arc::ptr_eq(
            &x,
            &aavl_right_child(&y.clone().unwrap().upgrade().unwrap()).unwrap(),
        )
    {
        x = y.clone().unwrap().upgrade().unwrap();
        y = aavl_paren(&y.clone().unwrap().upgrade().unwrap());
    }

    y.unwrap().upgrade().unwrap()
}


#[inline]
fn aavl_validate<K: BSTKey, V>(x: &Option<AavlNodeRc<K, V>>) {
    if x.is_none() {
        return;
    }

    let lf_h = aavl_height(&aavl_left_child(&x.as_ref().unwrap()));
    let rh_h = aavl_height(&aavl_right_child(&x.as_ref().unwrap()));

    if aavl_bf(&x.as_ref().unwrap()) == BF::P1 {
        assert_eq!(
            rh_h - lf_h,
            1,
            "expect P1, found: lf: {} rh: {}",
            lf_h,
            rh_h
        );
    } else if aavl_bf(&x.as_ref().unwrap()) == BF::Z {
        assert_eq!(
            rh_h - lf_h,
            0,
            "expect Z, found: lf: {} rh: {}",
            lf_h,
            rh_h
        );
    } else {
        assert_eq!(
            rh_h - lf_h,
            -1,
            "expect N1, found: lf: {} rh: {}",
            lf_h,
            rh_h
        );
    }

    aavl_validate(&aavl_left_child(&x.as_ref().unwrap()));
    aavl_validate(&aavl_right_child(&x.as_ref().unwrap()));
}


impl<K: BSTKey, V> Aavlnode<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            left: None,
            right: None,
            paren: None,
            bf: BF::Z,
            key: Arc::new(key),
            value: Arc::new(value),
        }
    }

    fn new_rc(key: K, value: V) -> AavlNodeRc<K, V> {
        Arc::new(RefCell::new(Self::new(key, value)))
    }
}

impl<K: BSTKey, V> Debug for Aavlnode<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("Aavlnode")
                .field("left", &self.left)
                .field("right", &self.right)
                .field("paren", &self.paren)
                .field("bf", &self.bf)
                .field("key", &self.key)
                .finish()
        } else {
            f.debug_struct("Aavlnode")
                .field("left", &self.left)
                .field("right", &self.right)
                .field("bf", &self.bf)
                .field("key", &self.key)
                .finish()
        }
    }
}


impl<K: BSTKey, V> Aavl<K, V> {
    fn subtree_shift(
        &mut self,
        u: &AavlNodeRc<K, V>,
        v: &Option<AavlNodeRc<K, V>>,
    ) {
        if aavl_paren(&u).is_none() {
            self.root = v.clone();
        } else if aavl_left_child(&aavl_paren(&u).unwrap().upgrade().unwrap())
            .is_some()
            && Arc::ptr_eq(
                u,
                &aavl_left_child(&aavl_paren(&u).unwrap().upgrade().unwrap())
                    .unwrap(),
            )
        {
            set_aavl_left_child(
                &aavl_paren(&u).unwrap().upgrade().unwrap(),
                v.clone(),
            );
        } else {
            set_aavl_right_child(
                &aavl_paren(&u).unwrap().upgrade().unwrap(),
                v.clone(),
            );
        }

        if v.is_some() {
            set_aavl_paren(&v.clone().unwrap().clone(), aavl_paren(&u))
        }
    }


    fn insert_retracing(&mut self, new_node: &AavlNodeRc<K, V>) {
        /* Rebalanced (retracing) */
        let mut z = new_node.clone();
        let mut x = aavl_paren(&z);
        let g;
        let n;

        while x.is_some() {
            let x_up = x.unwrap().upgrade().unwrap();

            if aavl_right_child(&x_up).is_some()
                && Arc::ptr_eq(&z, &aavl_right_child(&x_up).unwrap())
            {
                if aavl_bf(&x_up) == BF::P1 {
                    g = aavl_paren(&x_up);

                    if aavl_bf(&z) == BF::N1 {
                        n = aavl_rotate_right_left(&x_up, &z);
                    } else {
                        n = aavl_rotate_left(&x_up, &z);
                    }
                } else {
                    if aavl_bf(&x_up) == BF::N1 {
                        set_aavl_bf(&x_up, BF::Z); // (we add cause unbalancing)
                        break; // now it's balanced
                    } else {
                        // == BF::Z
                        set_aavl_bf(&x_up, BF::P1);

                        z = x_up;
                        x = aavl_paren(&z);
                        continue;
                    }
                }
            } else {
                // z is left child
                if aavl_bf(&x_up) == BF::N1 {
                    g = aavl_paren(&x_up);

                    if aavl_bf(&z) == BF::P1 {
                        n = aavl_rotate_left_right(&x_up, &z);
                    } else {
                        n = aavl_rotate_right(&x_up, &z);
                    }
                } else {
                    if aavl_bf(&x_up) == BF::P1 {
                        set_aavl_bf(&x_up, BF::Z); // (we add cause unbalancing)
                        break; // now it's balanced
                    } else {
                        // == BF::Z
                        set_aavl_bf(&x_up, BF::N1);
                        z = x_up;
                        x = aavl_paren(&z);
                        continue;
                    }
                }
            }

            set_aavl_paren(&n, g.clone());

            // g->x => g->n->x
            if let Some(gwk) = g {
                let garc = gwk.upgrade().unwrap();
                let g_lf = aavl_left_child(&garc);

                if g_lf.is_some() && Arc::ptr_eq(&x_up, &g_lf.unwrap()) {
                    set_aavl_left_child(&garc, Some(n));
                } else {
                    set_aavl_right_child(&garc, Some(n));
                }
            } else {
                self.root = Some(n);
            }

            break;
        }
    }


    fn remove_retracing(&mut self, removed_node: &AavlNodeRc<K, V>) {
        let mut n = removed_node.clone();
        let mut x = aavl_paren(&n);
        let mut g;
        let mut z;
        let mut b;

        while x.is_some() {
            let x_up = x.clone().unwrap().upgrade().unwrap();
            g = aavl_paren(&x_up);

            // the left subtree decreses
            if aavl_left_child(&x_up).is_some()
                && Arc::ptr_eq(&n, &aavl_left_child(&x_up).unwrap())
            {
                if aavl_bf(&x_up) == BF::P1 {
                    z = aavl_right_child(&x_up).unwrap();
                    b = aavl_bf(&z);

                    if b == BF::N1 {
                        n = aavl_rotate_right_left(&x_up, &z);
                    } else {
                        n = aavl_rotate_left(&x_up, &z);
                    }
                } else {
                    if aavl_bf(&x_up) == BF::Z {
                        set_aavl_bf(&x_up, BF::P1);


                        break;
                    } else {
                        n = x_up;
                        set_aavl_bf(&n, BF::Z);
                        x = g;
                        continue;
                    }
                }
            } else {
                // n == aavl_right_child(&x_up)
                if aavl_bf(&x_up) == BF::N1 {
                    z = aavl_left_child(&x_up).unwrap();
                    b = aavl_bf(&z);

                    if b == BF::P1 {
                        n = aavl_rotate_left_right(&x_up, &z);
                    } else {
                        n = aavl_rotate_right(&x_up, &z);
                    }
                } else {
                    if aavl_bf(&x_up) == BF::Z {
                        set_aavl_bf(&x_up, BF::N1);
                        break;
                    } else {
                        n = x_up;
                        set_aavl_bf(&n, BF::Z);
                        x = g;
                        continue;
                    }
                }
            }

            set_aavl_paren(&n, g.clone());

            // g->x => g->n->x
            if let Some(gwk) = g {
                let garc = gwk.upgrade().unwrap();
                let g_lf = aavl_left_child(&garc);

                if g_lf.is_some() && Arc::ptr_eq(&x_up, &g_lf.unwrap()) {
                    set_aavl_left_child(&garc, Some(n.clone()));
                } else {
                    set_aavl_right_child(&garc, Some(n.clone()));
                }
            } else {
                self.root = Some(n.clone());
            }

            if b == BF::Z {
                break;
            }
        }
    }
}

impl<K: BSTKey, V> Adictionary<K, V> for Aavl<K, V> {
    fn insert(&mut self, key: K, val: V) -> bool {
        if self.root.is_none() {
            self.root = Some(Aavlnode::new_rc(key, val));
            return true;
        }

        let approxi_node =
            aavl_search_approximately(&key, &self.root.as_ref().unwrap());

        if key == *aavl_key(&approxi_node) {
            return false;
        }

        let new_node = if key < *aavl_key(&approxi_node) {
            let new_node = Aavlnode::new_rc(key, val);
            set_aavl_paren(&new_node, Some(Arc::downgrade(&approxi_node)));

            set_aavl_left_child(&approxi_node, Some(new_node.clone()));

            new_node
        } else {
            let new_node = Aavlnode::new_rc(key, val);
            set_aavl_paren(&new_node, Some(Arc::downgrade(&approxi_node)));

            set_aavl_right_child(&approxi_node, Some(new_node.clone()));

            new_node
        };


        self.insert_retracing(&new_node);

        self.self_validate();

        true
    }

    fn remove(&mut self, key: &K) -> Option<Arc<V>> {
        if self.root.is_none() {
            return None;
        }

        let approxi_node =
            aavl_search_approximately(key, self.root.as_ref().unwrap());

        if *key != *aavl_key(&approxi_node) {
            return None;
        }

        /* Tree Delete */

        if aavl_left_child(&approxi_node).is_none() {
            self.subtree_shift(
                &approxi_node,
                &aavl_right_child(&approxi_node),
            );
        } else if aavl_right_child(&approxi_node).is_none() {
            self.subtree_shift(&approxi_node, &aavl_left_child(&approxi_node));
        } else {
            let y = aavl_successor(&approxi_node);

            if aavl_paren(&y).is_none()
                || !Arc::ptr_eq(
                    &aavl_paren(&y).unwrap().upgrade().unwrap(),
                    &approxi_node,
                )
            {
                self.subtree_shift(&y, &aavl_right_child(&y));
                set_aavl_right_child(&y, aavl_right_child(&approxi_node));
                let y_rh = aavl_right_child(&y).unwrap();
                set_aavl_paren(&y_rh, Some(Arc::downgrade(&y)));
            }

            self.subtree_shift(&approxi_node, &Some(y.clone()));

            set_aavl_left_child(&y, aavl_left_child(&approxi_node));
            let y_lf = aavl_left_child(&y).unwrap();
            set_aavl_paren(&y_lf, Some(Arc::downgrade(&y)));
        }


        /* Retracing after delete */
        self.remove_retracing(&approxi_node);

        Some(aavl_val(&approxi_node))
    }


    fn modify(&mut self, key: &K, val: V) -> bool {
        if self.root.is_none() {
            return false;
        }

        let approxi_node =
            aavl_search_approximately(key, self.root.as_ref().unwrap());

        if *key == *aavl_key(&approxi_node) {
            approxi_node.as_ref().borrow_mut().value = Arc::new(val);

            true
        } else {
            false
        }
    }

    fn lookup(&self, key: &K) -> Option<Arc<V>> {
        if self.root.is_none() {
            return None;
        }

        let res = aavl_search_approximately(key, self.root.as_ref().unwrap());

        // dbg!(&res.as_ref().borrow().key);

        if *res.as_ref().borrow().key == *key {
            let value = &res.as_ref().borrow().value;
            Some(value.clone())
        } else {
            None
        }
    }

    /// Validate BF 'did' reflects the reality
    fn self_validate(&self) {
        aavl_validate(&self.root)
    }
}


// impl<K: BSTKey, V> ABST<K, V> for Aavl<K, V> {

// }


// impl<K: BSTKey, V> ABT<K, V> for Aavlnode<K, V> {
//     fn left(&self) -> Option<Arc<RefCell<&dyn ABT<K, V>>>> {
//         if let Some(ref left) = self.left {
//             left.as_ref().borrow() as &dyn ABT<K, V>
//         }
//     }

// }



#[cfg(test)]
mod tests {

    // use super::Aavl;
    // use crate::{test::dict::{DictProvider, Inode, InodeProvider}, collections::Adictionary};

    // #[test]
    // fn test_bst_randomdata() {
    //     let mut dict = Aavl::<u16, Inode>::new();

    //     let provider = InodeProvider {};

    //     (&provider as &dyn DictProvider<u16, Inode>).test_adict(&mut dict);
    // }

    // #[test]
    // fn test_bst_fixeddata() {
    //     let mut dict = Aavl::<i32, ()>::new();

    //     let adict = &mut dict as &mut dyn Adictionary<i32, ()>;

    //     adict.insert(10, ());
    //     adict.self_validate();

    //     adict.insert(5, ());
    //     adict.self_validate();

    //     adict.insert(12, ());
    //     adict.self_validate();

    //     adict.insert(13, ());
    //     adict.self_validate();

    //     adict.insert(14, ());
    //     adict.self_validate();

    //     adict.insert(15, ());
    //     adict.self_validate();

    //     adict.insert(16, ());
    //     adict.self_validate();

    //     adict.insert(18, ());
    //     adict.self_validate();

    //     adict.insert(7, ());
    //     adict.self_validate();

    //     adict.insert(9, ());
    //     adict.self_validate();

    //     adict.insert(11, ());
    //     adict.self_validate();

    //     adict.insert(22, ());
    //     adict.self_validate();

    // }
}
