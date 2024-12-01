//! Refer Python Module [collections.abc](https://docs.python.org/3/library/collections.abc.html#collections-abstract-base-classes)

use std::borrow::Borrow;



////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait IterableMut<'a> {
    type ItemMut: 'a;

    fn iter_mut(&'a mut self) -> impl Iterator<Item = Self::ItemMut>;
}

/// We have to downgrade `Iterable` abstraction from trait level to concrete method level
/// since rust1 disallow lazy resolve lifetime.
trait Iterable {}

impl<T: MappingIterable> Iterable for T {}

#[allow(private_bounds)]
pub trait Collection: Sized + Iterable + IntoIterator {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn new() -> Self;
}

pub trait MappingIterable {
    type Key;
    type Value;

    fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a
    where
        Self::Key: 'a,
        Self::Value: 'a;
}

pub trait Mapping<Q: ?Sized>: Collection + MappingIterable
where
    Self::Key: Borrow<Q>
{
    fn get(&self, k: &Q) -> Option<&Self::Value>;
}

pub trait MappingMut<Q>: Mapping<Q>
where
    Self::Key: Borrow<Q>
{
    fn get_mut(&mut self, k: &Q) -> Option<&mut Self::Value>;
}

pub trait MutableMapping<Q>: Mapping<Q>
where
    Self::Key: Borrow<Q>
{
    fn insert(
        &mut self,
        key: Self::Key,
        value: Self::Value,
    ) -> Option<Self::Value>;

    fn remove(&mut self, key: &Q) -> Option<Self::Value>;
}

// pub trait BPTreeMap<'a>: MutableMapping<'a> {
//     fn select<Q, R>(&self, range: R) -> impl Iterator<Item = Self::ItemRef>
//     where
//         Self::Key: Borrow<Q>,
//         Q: Ord + ?Sized + 'a,
//         R: RangeBounds<&'a Q> + Clone;

//     fn select_keys<Q, R>(&self, range: R) -> impl Iterator<Item = &'a Self::Key>
//     where
//         Self::Key: Borrow<Q>,
//         Q: Ord + ?Sized + 'a,
//         R: RangeBounds<&'a Q> + Clone,
//     {
//         self.select(range).map(|(k, _v)| k)
//     }

//     fn select_values<Q, R>(
//         &self,
//         range: R,
//     ) -> impl Iterator<Item = &'a Self::Value>
//     where
//         Self::Key: Borrow<Q>,
//         Q: Ord + ?Sized + 'a,
//         R: RangeBounds<&'a Q> + Clone,
//     {
//         self.select(range).map(|(_k, v)| v)
//     }
// }
