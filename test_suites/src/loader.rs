use std::marker::PhantomData;

use extern_rand::{rngs::ThreadRng, thread_rng, Rng};

pub use crate::rand::{GenerateI32100, GenerateI3210000, GenerateI32Any};
use crate::{
    rand::{GenerateRandomValue, RandomRoller},
    Collection,
};

pub trait Load: Sized {
    fn new() -> Self;
}

/// Accept Ordered input
pub trait BulkLoad: Load {
    type BulkItem: Ord;

    fn bulk_load<T: IntoIterator<Item = Self::BulkItem>>(iter: T) -> Self;

    fn random_bulk_load<G: GenerateRandomValue<Self::BulkItem>>(
        g: &mut G,
        len: usize,
    ) -> Self {
        let mut input = (0..len)
            .map(|_| g.generate())
            .collect::<Vec<_>>();

        input.sort();

        Self::bulk_load(input)
    }
}

pub trait Loader<T> {
    fn load(&mut self) -> T;
}

////////////////////////////////////////////////////////////////////////////////
//// Strcutures

#[derive(Default)]
pub struct DefaultLoader<T> {
    _marker: PhantomData<T>,
}

pub struct BulkLoader<T, G> {
    g: G,
    upper_bound: usize,
    _marker: PhantomData<T>,
}

pub struct MixedLoader<T, G> {
    /// If bulk load
    roller: RandomRoller<bool>,
    bulkloader: BulkLoader<T, G>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T: Collection> Load for T {
    fn new() -> Self {
        T::new()
    }
}

impl<T> DefaultLoader<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Load> Loader<T> for DefaultLoader<T> {
    fn load(&mut self) -> T {
        T::new()
    }
}

impl<T> BulkLoader<T, ThreadRng> {
    pub fn new_with_upper_bound(upper_bound: usize) -> Self {
        Self {
            g: thread_rng(),
            upper_bound,
            _marker: PhantomData,
        }
    }
}

impl<T: BulkLoad> Loader<T> for BulkLoader<T, ThreadRng>
where
    ThreadRng: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        let len = self.g.gen_range(0..=self.upper_bound);

        T::random_bulk_load(&mut self.g, len)
    }
}

impl<T> BulkLoader<T, GenerateI32100> {
    pub fn new_with_upper_bound(upper_bound: usize) -> Self {
        Self {
            g: GenerateI32100::new(),
            upper_bound,
            _marker: PhantomData,
        }
    }
}

impl<T: BulkLoad> Loader<T> for BulkLoader<T, GenerateI32100>
where
    GenerateI32100: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        let len = thread_rng().gen_range(0..=self.upper_bound);

        T::random_bulk_load(&mut self.g, len)
    }
}

impl<T> BulkLoader<T, GenerateI3210000> {
    pub fn new_with_upper_bound(upper_bound: usize) -> Self {
        Self {
            g: GenerateI3210000::new(),
            upper_bound,
            _marker: PhantomData,
        }
    }
}

impl<T: BulkLoad> Loader<T> for BulkLoader<T, GenerateI3210000>
where
    GenerateI3210000: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        let len = thread_rng().gen_range(0..=self.upper_bound);

        T::random_bulk_load(&mut self.g, len)
    }
}

impl<T> BulkLoader<T, GenerateI32Any> {
    pub fn new_with_upper_bound(upper_bound: usize) -> Self {
        Self {
            g: GenerateI32Any::new(),
            upper_bound,
            _marker: PhantomData,
        }
    }
}

impl<T: BulkLoad> Loader<T> for BulkLoader<T, GenerateI32Any>
where
    GenerateI32Any: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        let len = thread_rng().gen_range(0..=self.upper_bound);

        T::random_bulk_load(&mut self.g, len)
    }
}

impl<T, G> MixedLoader<T, G> {
    pub fn new_with_bulkloader(bulkloader: BulkLoader<T, G>) -> Self {
        Self {
            roller: RandomRoller::with_candicates(vec![(5, true), (5, false)]),
            bulkloader,
        }
    }
}

impl<T: BulkLoad> Loader<T> for MixedLoader<T, GenerateI32100>
where
    GenerateI32100: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        if self.roller.roll() {
            self.bulkloader.load()
        } else {
            T::new()
        }
    }
}

impl<T: BulkLoad> Loader<T> for MixedLoader<T, GenerateI3210000>
where
    GenerateI3210000: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        if self.roller.roll() {
            self.bulkloader.load()
        } else {
            T::new()
        }
    }
}

impl<T: BulkLoad> Loader<T> for MixedLoader<T, GenerateI32Any>
where
    GenerateI32Any: GenerateRandomValue<<T as BulkLoad>::BulkItem>,
{
    fn load(&mut self) -> T {
        if self.roller.roll() {
            self.bulkloader.load()
        } else {
            T::new()
        }
    }
}
