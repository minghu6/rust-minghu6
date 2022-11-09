use std::{borrow::Borrow, collections::{HashMap, VecDeque}, fmt::Debug, hash::Hash};


////////////////////////////////////////////////////////////////////////////////
//// Macro

#[macro_export]
macro_rules! get {
    ($var:expr => $($e:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::collections::easycoll::EasyCollGet;

            let var = &$var;

            if let Some(v) = var.get(&($($e),+)) {
                v
            }
            else {
                unreachable!("get None for {:?}", &($($e),+))
            }
        }
    };
    ($var:expr => $($e:expr),+ => $default:expr) => {
        {
            #[allow(unused_imports)]
            use $crate::collections::easycoll::EasyCollGet;

            let var = &$var;
            var.get((&($($e),+))).unwrap_or($default)
        }
    };
}

#[macro_export]
macro_rules! getopt {
    ($var:expr => $($e:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::collections::easycoll::EasyCollGet;

            let var = &$var;

            var.get(&($($e),+))
        }
    };
}

#[macro_export]
macro_rules! set {
    ($var:expr => $($k:expr),+ => $($v:expr),+ ) => {
        {
            #[allow(unused_imports)]
            use $crate::collections::easycoll::EasyCollInsert;
            $var.insert(($($k),+), ($($v),+))
        }
    };

    ($var:expr => $($k:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::collections::easycoll::EasyCollInsert;
            $var.insert(($($k),+))
        }
    };
}

/// Assoc push
#[macro_export]
macro_rules! apush {
    ($var:expr => $($k:expr),+ => $($v:expr),+ ) => {
        {
            use $crate::collections::easycoll::EasyCollAPush;
            $var.apush(($($k),+), ($($v),+))
        }
    };
}


#[macro_export]
macro_rules! contains {
    ($var:expr => $($k:expr),+ ) => {
        $crate::getopt!($var => $($k),+).is_some()
    };
}



#[macro_export]
macro_rules! m1 {
    ($($k:expr => $v:expr),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut _ins = $crate::collections::easycoll::M1::new();
            $(
                $crate::set!(_ins => $k => $v);
            )*
            _ins
        }
    };
}

#[macro_export]
macro_rules! m2 {
    ($($k1:expr => $k2:expr => $v:expr),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut _ins = $crate::collections::easycoll::M2::new();
            $(
                $crate::set!(_ins => $k1, $k2 => $v);
            )*
            _ins
        }
    };
}

#[macro_export]
macro_rules! mv {
    ($($k1:expr => $k2:expr => $v:expr),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut _ins = $crate::collections::easycoll::MV::new();
            $(
                $crate::set!(_ins => $k1, $k2 => $v);
            )*
            _ins
        }
    };
}


#[macro_export]
macro_rules! stack {
    ( $($value:expr),* ) => {
        {
            let mut _stack = $crate::collections::easycoll::Stack::new();

            $(
                _stack.push($value);
            )*

            _stack
        }
    };
}

#[macro_export]
macro_rules! queue {
    ( $($value:expr),* ) => {
        {
            let mut _q = $crate::collections::easycoll::Queue::new();

            $(
                _q.enq($value);
            )*

            _q
        }
    };
}


#[macro_export]
macro_rules! vecdeq {
    ( $($value:expr),* ) => {
        {
            let mut _deq = std::collections::VecDeque::new();

            $(
                _deq.push_back($value);
            )*

            _deq
        }
    };
}


#[macro_export]
macro_rules! hashset {
    ( $($value:expr),* ) => {
        {
            let mut _set = std::collections::HashSet::new();

            $(
                _set.insert($value);
            )*

            _set
        }
    };
}


#[macro_export]
macro_rules! hashmap {
    ($($k:expr => $v:expr),* $(,)?) => {
        {
            let mut _ins = std::collections::HashMap::new();
            $(
                $crate::set!(_ins => $k => $v);
            )*
            _ins
        }
    };
}


#[macro_export]
macro_rules! btreemap {
    ($($k:expr => $v:expr),* $(,)?) => {
        {
            let mut _ins = std::collections::BTreeMap::new();
            $(
                $crate::set!(_ins => $k => $v);
            )*
            _ins
        }
    };
}


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait EasyCollGet<K, V> {
    type Target;

    fn get<Q: Borrow<K>>(&self, k: &Q) -> Option<Self::Target>;
}

pub trait EasyCollInsert<K, V> {
    type Target;

    fn insert(&mut self, k: K, v: V) -> Option<Self::Target>;
}

pub trait EasyCollAPush<K, V> {
    fn apush(&mut self, k: K, v: V);
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct M1<K, V>(pub HashMap<K, V>);


#[derive(Default, Debug)]
#[repr(transparent)]
pub struct M2<K1, K2, V>(pub HashMap<K1, HashMap<K2, V>>);


#[derive(Default, Debug)]
#[repr(transparent)]
pub struct MV<K, V>(pub HashMap<K, Vec<V>>);


#[derive(Default, Debug)]
#[repr(transparent)]
pub struct Stack<T> (pub Vec<T>);


/// FIFO simple queue
#[derive(Default, Debug)]
#[repr(transparent)]
pub struct Queue<T> (pub VecDeque<T>);



////////////////////////////////////////////////////////////////////////////////
//// Implementation

/* M1 */
impl<K, V> M1<K, V> {
    pub fn new() -> Self {
        Self(HashMap::<K, V>::new())
    }
}

impl<K, V> EasyCollGet<K, V> for M1<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = V;

    fn get<Q: Borrow<K>>(&self, k: &Q) -> Option<Self::Target> {
        self.0.get(k.borrow()).cloned()
    }
}

impl<K, V> EasyCollInsert<K, V> for M1<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = V;

    fn insert(&mut self, k: K, v: V) -> Option<Self::Target> {
        self.0.insert(k, v)
    }
}


/* M2 */
impl<K1, K2, V> M2<K1, K2, V> {
    pub fn new() -> Self {
        Self(HashMap::<K1, HashMap<K2, V>>::new())
    }
}

impl<K1, K2, V> EasyCollGet<(K1, K2), V> for M2<K1, K2, V>
where
    K1: Hash + Eq + Debug,
    K2: Hash + Eq + Debug,
    V: Clone,
{
    type Target = V;

    fn get<Q: Borrow<(K1, K2)>>(&self, k: &Q) -> Option<Self::Target> {
        let (k1, k2) = k.borrow();

        if let Some(map1) = self.0.get(&k1) {
            if let Some(v) = map1.get(&k2) {
                Some(v.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<K1, K2, V> EasyCollInsert<(K1, K2), V> for M2<K1, K2, V>
where
    K1: Hash + Eq + Debug,
    K2: Hash + Eq + Debug,
    V: Clone,
{
    type Target = V;

    fn insert(&mut self, k: (K1, K2), v: V) -> Option<Self::Target> {
        let (k1, k2) = k;

        self.0.entry(k1).or_default().insert(k2, v)
    }
}


/* MV */
impl<K, V> MV<K, V> {
    pub fn new() -> Self {
        Self(HashMap::<K, Vec<V>>::new())
    }
}

impl<K, V> EasyCollGet<(K, usize), Vec<V>> for MV<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = V;

    fn get<Q: Borrow<(K, usize)>>(&self, k: &Q) -> Option<Self::Target> {
        let (k1, k2) = k.borrow();

        if let Some(v) = self.0.get(k1) {
            v.get(*k2).cloned()
        } else {
            None
        }
    }
}

impl<K, V> EasyCollGet<K, Vec<V>> for MV<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = Vec<V>;

    fn get<Q: Borrow<K>>(&self, k: &Q) -> Option<Self::Target> {
        self.0.get(k.borrow()).cloned()
    }
}

impl<K, V> EasyCollInsert<(K, usize), V> for MV<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = V;
    fn insert(&mut self, k: (K, usize), v: V) -> Option<Self::Target> {
        let (k1, k2) = k;

        let ent = self.0.entry(k1).or_default();

        if ent.len() == k2 {
            ent.push(v.clone());
        } else if ent.len() < k2 {
            ent[k2] = v.clone();
        } else {
            return None;
        }

        Some(v)
    }
}

impl<K, V> EasyCollInsert<K, Vec<V>> for MV<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = Vec<V>;

    fn insert(&mut self, k: K, v: Vec<V>) -> Option<Self::Target> {
        self.0.insert(k, v)
    }
}

impl<K, V> EasyCollAPush<K, V> for MV<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    fn apush(&mut self, k: K, v: V) {
        self.0.entry(k).or_default().push(v)
    }
}


impl<T> Stack<T> {
    // staic method
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, item: T) {
        self.0.push(item)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.last()
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.0.last_mut()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// FILO
    pub fn stack_iter<'a>(&'a self) -> impl Iterator<Item=&T> + 'a {
        let mut iter = self.0.iter().rev();

        std::iter::from_fn(move || {
            iter.next()
        })
    }

    /// This method will move the content of stack
    pub fn extend_stack(&mut self, income_stack: Self) {
        self.0.extend(income_stack.0)
    }
}


impl<I> Extend<I> for Stack<I> {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}


impl<T> Queue<T> {
    // staic method
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn enq(&mut self, item: T) {
        self.0.push_front(item)
    }

    pub fn deq(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.back()
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.0.back_mut()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// FIFO
    pub fn queue_iter<'a>(&'a self) -> impl Iterator<Item=&T> + 'a {
        let mut iter = self.0.iter();

        std::iter::from_fn(move || {
            iter.next()
        })
    }

    /// This method will move the content of queue
    pub fn extend_queue(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}




#[cfg(test)]
mod tests {

}
