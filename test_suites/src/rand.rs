use extern_rand::prelude::*;
use m6entry::KVEntry;



////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait GenerateRandomValue<T> {
    fn generate(&mut self) -> T;
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct RandomRoller<T> {
    candicates: Vec<(usize, T)>,
    w_acc: Vec<usize>,
    rng: ThreadRng
}

pub struct GenerateI32100 {
    rng: ThreadRng
}

pub struct GenerateI3210000 {
    rng: ThreadRng
}

pub struct GenerateI32Any {
    rng: ThreadRng
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl GenerateI32100 {
    pub fn new() -> Self {
        Self { rng: thread_rng() }
    }
}

impl GenerateRandomValue<i32> for GenerateI32100 {
    fn generate(&mut self) -> i32 {
        self.rng.gen_range(0..100)
    }
}

impl GenerateI3210000 {
    pub fn new() -> Self {
        Self { rng: thread_rng() }
    }
}

impl GenerateRandomValue<i32> for GenerateI3210000 {
    fn generate(&mut self) -> i32 {
        self.rng.gen_range(0..10000)
    }
}

impl GenerateI32Any {
    pub fn new() -> Self {
        Self { rng: thread_rng() }
    }
}

impl GenerateRandomValue<i32> for GenerateI32Any {
    fn generate(&mut self) -> i32 {
        Rng::r#gen(&mut self.rng)
    }
}

impl GenerateRandomValue<i32> for ThreadRng {
    fn generate(&mut self) -> i32 {
        Rng::r#gen(self)
    }
}

impl<T: Clone, G: GenerateRandomValue<T>> GenerateRandomValue<KVEntry<T, T>> for G {
    fn generate(&mut self) -> KVEntry<T, T> {
        let k = self.generate();
        let v = k.clone();

        KVEntry(k, v)
    }
}

impl<T> RandomRoller<T> {
    pub fn with_candicates(candicates: Vec<(usize, T)>) -> Self {
        let w_acc = candicates.iter().scan(0, |state, (w, _)| {
            *state += *w;

            Some(*state)
        }).collect();

        let rng = thread_rng();

        Self { candicates, w_acc, rng }
    }

    pub fn roll(&mut self) -> T where T: Clone {
        assert!(!self.candicates.is_empty());

        let w_tot = *self.w_acc.last().unwrap();

        assert!(w_tot >= 1);

        let res = self.rng.gen_range(1..=w_tot);

        match self.w_acc.binary_search(&res) {
            Ok(idx) => self.candicates[idx].1.clone(),
            Err(idx) => self.candicates[idx].1.clone(),  // no index overflow as res in
        }
    }
}
