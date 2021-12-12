pub mod avl_abandoned;
pub mod avl;


use std::{fmt::{Debug, self, Write}, collections::VecDeque};

use super::{Dictionary, Adictionary};


/// LF(key) < MID(key) < RH(key)
pub trait BST<'a, K: BSTKey, V>: Dictionary<K, V> {
    fn itself(&mut self) -> *mut (dyn BST<'a, K, V> + 'a);
    fn root(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a);

    fn assign_root(&mut self, root: *mut (dyn BSTNode<'a, K, V> + 'a));

    fn subtree_shift(&mut self, u: *mut (dyn BSTNode<'a, K, V> + 'a), v: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        unsafe {
            if (*u).paren().is_null() {
                self.assign_root(v)
            } else if u == (*(*u).paren()).left() {
                (*(*u).paren()).assign_left(v);
            } else {
                (*(*u).paren()).assign_right(v);
            }

            if !v.is_null() {
                (*v).assign_paren((*u).paren())
            }
        }
    }


    fn height(&mut self) -> i32 {
        if self.root().is_null() {
            return 0;
        }

        unsafe { (*self.root()).height() }
    }

    /// BFS Echo
    fn echo_in_mm(
        &self,
        cache: &mut String,
        action: fn(*mut (dyn BSTNode<'a, K, V> + 'a), &mut String) -> fmt::Result
    ) -> fmt::Result {
        if self.root().is_null() {
            writeln!(cache, "ROOT: null")
        } else {
            unsafe {
                writeln!(cache, "ROOT: {:?}", (*self.root()).key() )?;

                (*self.root()).echo_in_mm(cache, action)
            }
        }
    }

    fn just_echo_stdout(&self) {
        if !self.root().is_null() {
            unsafe { (*self.root()).just_echo_stdout() }
        }
    }

}


pub trait BSTNode<'a, K: BSTKey, V> {
    fn left(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a);
    fn right(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a);
    fn paren(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a);
    fn key(&self) -> &K;
    fn value(&self) -> &V;
    fn itself_mut(&mut self) -> *mut (dyn BSTNode<'a, K, V> + 'a);
    fn itself(&self) -> *const (dyn BSTNode<'a, K, V> + 'a);


    fn assign_left(&mut self, left: *mut (dyn BSTNode<'a, K, V> + 'a));
    fn assign_right(&mut self, right: *mut (dyn BSTNode<'a, K, V> + 'a));
    fn assign_paren(&mut self, paren: *mut (dyn BSTNode<'a, K, V> + 'a));
    fn assign_value(&mut self, value: V);

    fn height(&mut self) -> i32 {
        let h_lf = if self.left().is_null() {
            -1
        } else {
            unsafe { (*self.left()).height() }
        };

        let h_rh = if self.right().is_null() {
            -1
        } else {
            unsafe { (*self.right()).height() }
        };

        i32::max(h_lf, h_rh) + 1
    }

    fn minimum(&mut self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        while unsafe { !(*x).left().is_null() } {
            unsafe { x = (*x).left() }
        }

        x
    }

    fn successor(&mut self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        let mut x = self.itself_mut();

        unsafe {
            if !(*x).right().is_null() {
                return (*(*x).right()).minimum();
            }

            let mut y = (*x).paren();

            while !y.is_null() && x == (*y).right() {
                x = y;
                y = (*y).paren();
            }

            y
        }
    }


    fn just_echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache, |_, _| { Ok(()) } ).unwrap();

        println!("{}", cache);
    }

    /// BFS Echo
    fn echo_in_mm(
        &self,
        cache: &mut String,
        action: fn(*mut (dyn BSTNode<'a, K, V> + 'a), &mut String) -> fmt::Result
    ) -> fmt::Result {
        unsafe {
            writeln!(cache, "Entry: {:?}", self.key())?;

            let mut this_level_queue: VecDeque<*mut (dyn BSTNode<'a, K, V> + 'a)> = VecDeque::new();
            this_level_queue.push_back(self.itself() as *mut (dyn BSTNode<'a, K, V> + 'a));
            let mut level = 0;

            while !this_level_queue.is_empty() {
                writeln!(cache)?;
                writeln!(cache, "############ Level: {} ##########", level)?;
                writeln!(cache)?;

                let mut nxt_level_queue: VecDeque<*mut (dyn BSTNode<'a, K, V> + 'a)> = VecDeque::new();

                while !this_level_queue.is_empty() {
                    let x = this_level_queue.pop_front().unwrap();

                    action(x, cache)?;

                    if !(*x).left().is_null() {
                        writeln!(cache, "{:?} -L-> {:?}", (*x).key(), (*(*x).left()).key())?;

                        nxt_level_queue.push_back((*x).left())
                    } else {
                        writeln!(cache, "{:?} -L-> null", (*x).key())?;
                    }

                    if !(*x).right().is_null() {
                        writeln!(cache, "{:?} -R-> {:?}", (*x).key(), (*(*x).right()).key() )?;

                        nxt_level_queue.push_back((*x).right())
                    } else {
                        writeln!(cache, "{:?} -R-> null", (*x).key())?;
                    }

                    writeln!(cache)?;
                }

                this_level_queue = nxt_level_queue;
                level += 1;
            }



        }


        Ok(())
    }

}

pub trait ABST<K: BSTKey, V>: Adictionary<K, V> {
    fn self_validate(&self);
}

pub trait BSTKey = Eq + Ord + Debug;
