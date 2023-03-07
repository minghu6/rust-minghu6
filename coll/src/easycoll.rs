use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::Hash, ops::{Index, RangeFrom, RangeTo}
};


////////////////////////////////////////////////////////////////////////////////
//// Macro

#[macro_export]
macro_rules! min {
    ($($val:expr),+) => {
        {
            [$($val),+].into_iter().min().unwrap()
        }
    }
}

#[macro_export]
macro_rules! max {
    ($($val:expr),+) => {
        {
            [$($val),+].into_iter().min().unwrap()
        }
    }
}

#[macro_export]
macro_rules! same {
    ($($val:expr),+) => {
        {
            let _arr = [$($val),+];
            _arr.iter().min().unwrap() == _arr.iter().max().unwrap()
        }
    }
}

#[macro_export]
macro_rules! slidedown1 {
    ($var:expr, $e:expr) => {{
        use $crate::easycoll::Slide;
        Slide::slide(&$var, 1, &[$e])
    }};
}

#[macro_export]
macro_rules! slideup1 {
    ($var:expr, $e:expr) => {{
        use $crate::easycoll::Slide;
        Slide::slide(&$var, -1, &[$e])
    }};
}

#[macro_export]
macro_rules! delopt {
    ($var:expr => $($e:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollRemove;

            let var = &mut $var;

            var.remove(&($($e),+))
        }
    }
}

#[macro_export]
macro_rules! del {
    ($var:expr => $($e:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollRemove;

            let var = &mut $var;

            if let Some(v) = var.remove(&($($e),+)) {
                v
            }
            else {
                unreachable!("remove None for {:?}", &($($e),+))
            }
        }
    };
    ($var:expr => $($e:expr),+ => $default:expr) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollRemove;

            let var = &$var;
            var.get((&($($e),+))).unwrap_or($default)
        }
    };
}

#[macro_export]
macro_rules! get {
    ($var:expr => $($e:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollGet;

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
            use $crate::easycoll::EasyCollGet;

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
            use $crate::easycoll::EasyCollGet;

            let var = &$var;

            var.get(&($($e),+))
        }
    };
}

/// Restricted getopt
#[macro_export]
macro_rules! rgetopt {
    ($var:expr => $($e:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollGet;

            // let var = &$var;

            EasyCollGet::get($var, &($($e),+))
        }
    };
}

#[macro_export]
macro_rules! set {
    ($var:expr => $($k:expr),+ => $($v:expr),+ ) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollInsert;
            $var.insert(($($k),+), ($($v),+))
        }
    };

    ($var:expr => $($k:expr),+) => {
        {
            #[allow(unused_imports)]
            use $crate::easycoll::EasyCollInsert;
            $var.insert(($($k),+))
        }
    };
}

/// Assoc push
#[macro_export]
macro_rules! apush {
    ($var:expr => $($k:expr),+ => $($v:expr),+ ) => {
        {
            use $crate::easycoll::EasyCollAPush;
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
macro_rules! concat {
    ($var:expr $(=> $k:expr)+ ) => {
        {
            let mut _var = $var;
            $(
                _var.extend($k);
            )+
            _var
        }
    };
}

#[macro_export]
macro_rules! push {
    ($var:expr $(=> $k:expr)+ ) => {
        {
            let mut _var = $var;
            $(
                _var.push($k);
            )+
            _var
        }
    };
}

#[macro_export]
macro_rules! def_coll_init {
    (seq | $name:ident, $new:expr, $push:ident) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($value:expr),* ) => {{
                #[allow(unused_mut)]
                let mut _coll = $new;

                $$(
                    _coll.$push($value);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
    (map^1 | $name:ident, $new:expr) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($k:expr => $v:expr),* $$(,)? ) => {{
                let mut _coll = $new;

                $$(
                    $crate::set!(_coll => $k => $v);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
    (map^2 | $name:ident, $new:expr) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($k1:expr => $k2:expr => $v:expr),* $$(,)? ) => {{
                let mut _coll = $new;

                $$(
                    $crate::set!(_coll => $k1, $k2 => $v);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
}


def_coll_init!(seq | stack, coll::easycoll::Stack::new(), push);
def_coll_init!(seq | queue, coll::easycoll::Queue::new(), enq);
// def_coll_init!(seq | vecdeq, std::collections::VecDeque::new(), push_back);
// def_coll_init!(seq | hashset, std::collections::HashSet::new(), insert);

def_coll_init!(map^1 | m1, coll::easycoll::M1::new());
def_coll_init!(map^1 | hashmap, std::collections::HashMap::new());
def_coll_init!(map^1 | btreemap, std::collections::BTreeMap::new());

def_coll_init!(map^2 | m2, coll::easycoll::M2::new());
def_coll_init!(map^2 | mv, coll::easycoll::MV::new());


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait Slide<T> {
    fn slide(&self, off: isize, padding: &[T]) -> Self
    where
        T: Clone,
        Self: Sized +
              Index<RangeFrom<usize>, Output = [T]> +
              Index<RangeTo<usize>, Output = [T]> +
              FromIterator<T>
    {
        assert!(off > isize::MIN);
        let abs_off = off.abs() as usize;

        assert!(abs_off <= padding.len());
        let debt = if self.len() < abs_off {
            abs_off - self.len()
        } else {
            0
        };

        let new_view: Self = if off >= 0 {
            let actual_off = if self.len() >= abs_off {
                abs_off
            } else {
                self.len()
            };

            self[actual_off..]
                .iter()
                .cloned()
                .chain(padding[debt..abs_off].iter().cloned())
                .collect()
        } else {
            let actual_off = if self.len() >= abs_off {
                self.len() - abs_off
            } else {
                0
            };

            padding[padding.len() - abs_off..padding.len() - debt]
                .iter()
                .cloned()
                .chain(self[..actual_off].iter().cloned())
                .collect()
        };

        debug_assert!(new_view.len() == self.len());

        new_view
    }

    fn len(&self) -> usize;
}


pub trait EasyCollRemove<K, V> {
    type Target;

    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Target>
    where
        K: Borrow<Q>,
        Q: Hash + Eq;
}


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


#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct M2<K1, K2, V>(pub HashMap<K1, HashMap<K2, V>>);


#[derive(Default, Clone)]
#[repr(transparent)]
pub struct MV<K, V>(pub HashMap<K, Vec<V>>);


#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct MS<K, V>(pub HashMap<K, HashSet<V>>);


#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct Stack<T>(pub Vec<T>);


/// FIFO simple queue
#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct Queue<T>(pub VecDeque<T>);


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K, V> EasyCollGet<K, V> for HashMap<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = V;

    fn get<Q: Borrow<K>>(&self, k: &Q) -> Option<Self::Target> {
        self.get(k.borrow()).cloned()
    }
}

////////////////////////////////////////
//// Implementation M1

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



////////////////////////////////////////
//// Implementation M2

impl<K1, K2, V> M2<K1, K2, V> {
    pub fn new() -> Self {
        Self(HashMap::<K1, HashMap<K2, V>>::new())
    }
}


impl<K1, K2, V> EasyCollRemove<K1, V> for M2<K1, K2, V>
where
    K1: Hash + Eq + Debug,
{
    type Target = HashMap<K2, V>;

    fn remove<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<Self::Target>
    where
        K1: Borrow<Q>,
    {
        self.0.remove(k)
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

// impl<K1, K2, V> EasyCollGet<K1, M1<K2, V>> for M2<K1, K2, V>
// where
//     K1: Hash + Eq + Debug,
//     K2: Hash + Eq + Debug,
//     V: Clone,
// {
//     type Target<'a> = &'a M1<K2, V> where K2: 'a, V: 'a;

//     fn get<Q: Borrow<(K1, K2)>>(&'a self, k: &Q) -> Option<Self::Target> {
//         let (k1, k2) = k.borrow();

//         if let Some(map1) = self.0.get(&k1) {
//             if let Some(v) = map1.get(&k2) {
//                 Some(v.clone())
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }

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

impl<K1, K2, V> EasyCollInsert<K1, M1<K2, V>> for M2<K1, K2, V>
where
    K1: Hash + Eq + Debug,
    K2: Hash + Eq + Debug,
    V: Clone,
{
    type Target = M1<K2, V>;

    fn insert(&mut self, k: K1, v: M1<K2, V>) -> Option<Self::Target> {
        self.0.insert(k, v.0).map(|v| M1(v))
    }
}


////////////////////////////////////////
//// Implementation MV

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

impl<K: Debug, V: Debug> Debug for MV<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for (k, v) in self.0.iter() {
            writeln!(f, "k: {k:?}")?;
            writeln!(f, "v: {:?}\n", v)?;
        }

        Ok(())
    }
}


////////////////////////////////////////
//// Implementation MS

impl<K, V> MS<K, V> {
    pub fn new() -> Self {
        Self(HashMap::<K, HashSet<V>>::new())
    }
}


impl<K, V> EasyCollInsert<K, V> for MS<K, V>
where
    K: Hash + Eq,
    V: Hash + Eq,
{
    type Target = ();

    fn insert(&mut self, k: K, v: V) -> Option<Self::Target> {
        if self.0.entry(k).or_default().insert(v) {
            Some(())
        } else {
            None
        }
    }
}

impl<K, V> EasyCollAPush<K, V> for MS<K, V>
where
    K: Hash + Eq,
    V: Hash + Eq,
{
    fn apush(&mut self, k: K, v: V) {
        self.0.entry(k).or_default().insert(v);
    }
}

impl<K, V> EasyCollRemove<K, V> for MS<K, V>
where
    K: Hash + Eq,
{
    type Target = HashSet<V>;

    fn remove<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<Self::Target>
    where
        K: Borrow<Q>,
    {
        self.0.remove(k)
    }
}

////////////////////////////////////////
//// Implementation Stack

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
    pub fn stack_iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        let mut iter = self.0.iter().rev();

        std::iter::from_fn(move || iter.next())
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


////////////////////////////////////////
//// Implementation Queue

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
    pub fn queue_iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        let mut iter = self.0.iter();

        std::iter::from_fn(move || iter.next())
    }

    /// This method will move the content of queue
    pub fn extend_queue(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}


////////////////////////////////////////
//// Implementation HashMap

impl<K, V> EasyCollInsert<K, HashSet<V>> for MS<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Target = HashSet<V>;

    fn insert(&mut self, k: K, v: HashSet<V>) -> Option<Self::Target> {
        self.0.insert(k, v)
    }
}


////////////////////////////////////////
//// Implementation Vector

impl<T: Clone> Slide<T> for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}


#[cfg(test)]
mod tests {
    use crate::easycoll::Slide;


    #[test]
    fn test_slide() {
        /* test vector */
        let v = vec![1, 2, 3];
        let padding = vec![4, 5, 6, 7, 8];

        assert_eq!(slidedown1!(v, 4), vec![2, 3, 4]);
        assert_eq!(slideup1!(v, 4), vec![4, 1, 2]);

        assert_eq!(v.slide(-2, &padding), vec![7, 8, 1]);
        assert_eq!(v.slide(-4, &padding), vec![5, 6, 7]);
        assert_eq!(v.slide(-5, &padding), vec![4, 5, 6]);

        assert_eq!(v.slide(2, &padding), vec![3, 4, 5]);
        assert_eq!(v.slide(4, &padding), vec![5, 6, 7]);
        assert_eq!(v.slide(5, &padding), vec![6, 7, 8]);

        assert_eq!(slidedown1!(vec![], 4), Vec::<i32>::new());
        assert_eq!(slideup1!(vec![], 4), Vec::<i32>::new());
        assert_eq!(Vec::<i32>::new().slide(2, &padding), Vec::<i32>::new());
        assert_eq!(Vec::<i32>::new().slide(-2, &padding), Vec::<i32>::new());

        /* test array */


    }

    #[test]
    fn repl() {
        let v = vec![1, 2, 3];

        assert_eq!(slidedown1!(v, 4), vec![2, 3, 4]);
        assert_eq!(slideup1!(v, 4), vec![4, 1, 2]);

        println!("{:?}", slidedown1!(v, 4));

        let _v2 = [1,2,3].into_iter().min().unwrap();

        let mut v = common::vecdeq![1, 2, 3];
        v.pop_back();
        println!("{v:?}");
    }
}
