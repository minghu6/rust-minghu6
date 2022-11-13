//! Simplified Fibonacci Heap used for MST or Shortest path algorithm (Obsoleted Design)

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use crate::collections::CollKey;


#[derive(Debug)]
pub struct SFibHeap<I, T> {
    min: Option<I>,
    nodes: HashMap<I, T>
}


impl<I, T> SFibHeap<I, T>
where
    I: CollKey + Hash + Clone,
    T: CollKey
{
    pub fn new() -> Self {
        Self {
            min: None,
            nodes: Default::default(),
        }
    }


    pub fn push(&mut self, i: I, v: T) {
        self.check_min(i.clone(), &v);

        self.nodes.insert(i, v);
    }


    pub fn decrease_key(&mut self, i: I, v: T) -> Option<T> {
        debug_assert!(self.get(&i).unwrap() > &v);

        self.check_min(i.clone(), &v);

        self.nodes.insert(i, v)
    }


    pub fn get<Q>(&self, i: &Q) -> Option<&T>
    where
        I: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.nodes.get(i)
    }


    pub fn top_item(&self) -> Option<(I, &T)> {
        if self.min.is_some() {
            Some(
                (
                    self.min.clone().unwrap(),
                    self.nodes.get(&self.min.clone().unwrap()).unwrap(),
                )
            )
        } else {
            None
        }
    }


    fn check_min(&mut self, i: I, v: &T) {
        if let Some(ref min) = self.min {
            if v < self.nodes.get(min).unwrap() {
                self.min = Some(i.clone());
            }
        }
        else {
            self.min = Some(i.clone());
        }
    }

}
