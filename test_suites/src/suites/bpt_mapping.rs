use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::Display,
    i32,
    marker::PhantomData,
    ops::{
        Bound::{self, *},
        RangeBounds,
    },
};

use extern_rand::{thread_rng, Rng};
use resource_config::RES;
use serde::{de::Visitor, Deserialize, Deserializer};
use AbcIU::*;
pub use BPTIU::*;

use crate::{
    rand::{GenerateRandomValue, RandomRoller},
    suites::mapping::{deserialize_variant_a, RandomInputFacility},
    BPTreeMap, GenerateI32100, GenerateI3210000, GenerateI32Any,
    Loader, TestContext, TestDataTable, TestSuite, Validate,
};

////////////////////////////////////////////////////////////////////////////////
//// Traits


////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct BoundTuple<T>(Bound<T>, Bound<T>);

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum BPTIU {
    /// Get
    Q(i32),
    /// Insert
    #[serde(deserialize_with = "deserialize_variant_a")]
    A(i32, i32),
    /// Remove
    D(i32),
    #[serde(alias = "??")]
    V,
    #[serde(deserialize_with = "deserialize_variant_r")]
    R(Bound<i32>, Bound<i32>),
}

/// Abstract Interpret Unit
#[derive(Debug, Clone, Copy)]
enum AbcIU {
    QE,
    QNE,
    ANE,
    AEVE,
    /// Add (Key) Exists Value Non-Exists
    AEVNE,
    DE,
    DNE,
    RInEInE,
    RInNEInNE,
    RUnUn,
}

pub struct BPTreeTestSuite<L, G, K, Q, EG, CG = BTreeMap<K, K>> {
    loader: L,
    g: G,
    _marker: PhantomData<(K, Q, EG, CG)>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implmentations

impl<T> RangeBounds<T> for BoundTuple<T> {
    fn start_bound(&self) -> Bound<&T> {
        self.0.as_ref()
    }

    fn end_bound(&self) -> Bound<&T> {
        self.1.as_ref()
    }
}

impl<L, K, Q, EG, CG> BPTreeTestSuite<L, GenerateI32100, K, Q, EG, CG>
where
    K: Eq + Clone + Borrow<Q>,
    EG: BPTreeMap<Q, Key = K, Value = K> + Validate + Display,
    CG: BPTreeMap<Q, Key = K, Value = K>,
    L: Loader<EG>,
{
    pub fn new_with_loader(loader: L) -> Self {
        Self {
            loader,
            g: GenerateI32100::new(),
            _marker: PhantomData,
        }
    }
}

impl<L, K, Q, EG, CG> BPTreeTestSuite<L, GenerateI3210000, K, Q, EG, CG>
where
    K: Eq + Clone + Borrow<Q>,
    EG: BPTreeMap<Q, Key = K, Value = K> + Validate + Display,
    CG: BPTreeMap<Q, Key = K, Value = K>,
    L: Loader<EG>,
{
    pub fn new_with_loader(loader: L) -> Self {
        Self {
            loader,
            g: GenerateI3210000::new(),
            _marker: PhantomData,
        }
    }
}

impl<L, K, Q, EG, CG> BPTreeTestSuite<L, GenerateI32Any, K, Q, EG, CG>
where
    K: Eq + Clone + Borrow<Q>,
    EG: BPTreeMap<Q, Key = K, Value = K> + Validate + Display,
    CG: BPTreeMap<Q, Key = K, Value = K>,
    L: Loader<EG>,
{
    pub fn new_with_loader(loader: L) -> Self {
        Self {
            loader,
            g: GenerateI32Any::new(),
            _marker: PhantomData,
        }
    }
}

impl<L, G, EG, CG> TestSuite for BPTreeTestSuite<L, G, i32, i32, EG, CG>
where
    G: GenerateRandomValue<i32>,
    EG: BPTreeMap<i32, Key = i32, Value = i32> + Validate + Display,
    CG: BPTreeMap<i32, Key = i32, Value = i32>,
    L: Loader<EG>,
{
    type EG = EG;
    type CG = CG;
    type IU = BPTIU;

    fn load_fixeddata() -> TestDataTable<Self::IU> {
        let toml_str = RES.test_suites().bpt_toml().load_to_string();
        match TestDataTable::deserialize(toml::Deserializer::new(&toml_str)) {
            Ok(tbl) => tbl,
            Err(err) => panic!("{err}"),
        }
    }

    fn new_random_input(
        &mut self,
        len: usize,
        upper_bound: usize,
    ) -> Vec<Vec<Self::IU>> {
        (0..len)
            .map(|_| {
                let mut case = vec![];
                let max_len = thread_rng().gen_range(0..=upper_bound);
                let mut tracer = RandomInputFacility::<i32, i32>::new();

                /* Increment procedure */

                let mut insert_roller = RandomRoller::with_candicates(vec![
                    (80, ANE),
                    (10, AEVE),
                    (5, AEVNE),
                    (5, DNE),
                    (5, QE),
                    (5, QNE),
                    (10, DE),
                    (15, RInEInE),
                    (10, RInNEInNE),
                    (5, RUnUn),
                ]);

                while tracer.len() < max_len {
                    let mut iu;

                    loop {
                        iu = insert_roller.roll();

                        if tracer.len() > 1 || matches!(iu, ANE) {
                            break;
                        }
                    }

                    impl_abciu(&mut tracer, &mut self.g, &mut case, iu);
                }

                /* Decrement procedure */

                let mut remove_roller = RandomRoller::with_candicates(vec![
                    (10, ANE),
                    (10, AEVE),
                    (5, AEVNE),
                    (5, DNE),
                    (5, QE),
                    (5, QNE),
                    (80, DE),
                    (15, RInEInE),
                    (10, RInNEInNE),
                    (5, RUnUn),
                ]);

                while tracer.len() > 0 {
                    let mut iu;

                    loop {
                        iu = remove_roller.roll();

                        if tracer.len() > 1 || !matches!(iu, RInEInE) {
                            break;
                        }
                    }

                    impl_abciu(
                        &mut tracer,
                        &mut self.g,
                        &mut case,
                        iu,
                    );
                }

                case
            })
            .collect()
    }

    fn interpret(ctx: &mut TestContext<EG, CG, Self::IU>) {
        match ctx.cur().unwrap().clone() {
            Q(k) => assert_eq!(ctx.eg.get(k.borrow()), ctx.cg.get(k.borrow()),),
            A(k, val) => assert_eq!(
                ctx.eg.insert(k.clone(), val.clone()),
                ctx.cg.insert(k.clone(), val.clone()),
            ),
            D(k) => {
                assert_eq!(ctx.eg.remove(k.borrow()), ctx.cg.remove(k.borrow()),)
            }
            V => ctx.eg.validate(),
            R(start, end) => {
                assert_eq!(
                    ctx.eg.range(BoundTuple(start, end)).collect::<Vec<_>>(),
                    ctx.cg.range(BoundTuple(start, end)).collect::<Vec<_>>(),
                )
            }
        }
    }

    fn new_test_context(
        &mut self,
        name: &str,
    ) -> TestContext<EG, CG, Self::IU> {
        let name = name.to_owned();
        let eg = self.loader.load();
        let mut cg = CG::new();

        for (k, v) in eg.iter() {
            cg.insert(k.clone(), v.clone());
        }

        TestContext {
            eg,
            cg,
            name,
            input: vec![],
        }
    }
}

impl<K: Ord + Borrow<Q>, V, Q: Ord + ?Sized> BPTreeMap<Q> for BTreeMap<K, V> {
    fn range<R>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (&Self::Key, &Self::Value)>
    where
        R: std::ops::RangeBounds<Q>,
    {
        self.range(range)
    }

    fn range_mut<R>(
        &mut self,
        range: R,
    ) -> impl Iterator<Item = (&Self::Key, &mut Self::Value)>
    where
        R: std::ops::RangeBounds<Q>,
    {
        self.range_mut(range)
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Functions

pub(crate) fn deserialize_variant_r<'de, D>(
    d: D,
) -> Result<(Bound<i32>, Bound<i32>), D::Error>
where
    D: Deserializer<'de>,
{
    #[repr(transparent)]
    pub struct BoundI32(Bound<i32>);

    impl<'de> Deserialize<'de> for BoundI32 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self(deserializer.deserialize_any(BoundI32Visitor)?))
        }
    }

    struct BoundI32Visitor;

    impl<'de> Visitor<'de> for BoundI32Visitor {
        type Value = Bound<i32>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "int/+inf/-inf")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Included(v as _))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v == -f64::INFINITY {
                Ok(Unbounded)
            } else if v == f64::INFINITY {
                Ok(Unbounded)
            } else {
                Err(E::custom("expect +-inf"))
            }
        }
    }

    struct RangeI32;

    impl<'de> Visitor<'de> for RangeI32 {
        type Value = (Bound<i32>, Bound<i32>);

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            formatter.write_str("[int/(+-)inf; 2]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let BoundI32(bound_start): BoundI32 = seq.next_element()?.unwrap();
            let BoundI32(bound_end): BoundI32 = seq.next_element()?.unwrap();

            Ok((bound_start, bound_end))
        }
    }

    d.deserialize_any(RangeI32)
}

fn impl_abciu<G>(
    tracer: &mut RandomInputFacility<i32, i32>,
    g: &mut G,
    case: &mut Vec<BPTIU>,
    iu: AbcIU,
) where
    G: GenerateRandomValue<i32>,
{
    match iu {
        QE | AEVE | AEVNE | DE => {
            let (k, v) = tracer.randomly_roll_item();

            match iu {
                QE => {
                    case.push(Q(k.clone()));
                }
                AEVNE => {
                    let v1 = loop {
                        let v1 = g.gen();

                        if v1 != v {
                            break v1;
                        }
                    };

                    case.push(A(k.clone(), v1));
                }
                AEVE => {
                    case.push(A(k.clone(), v));
                }
                DE => {
                    case.push(D(k.clone()));
                    tracer.remove(&k);
                }
                RInEInE => {}
                _ => unreachable!(),
            }

            if !matches!(iu, QE) {
                case.push(Q(k));
            }
        }
        QNE | ANE | DNE => {
            let k1 = loop {
                let k1 = g.gen();

                if tracer.cg.get(&k1).is_none() {
                    break k1;
                }
            };

            match iu {
                QNE => {
                    case.push(Q(k1));
                }
                ANE => {
                    case.push(A(k1.clone(), k1.clone()));
                    tracer.insert(k1.clone(), k1);
                }
                DNE => {
                    case.push(D(k1));
                }
                _ => unreachable!(),
            }
        }
        RInEInE => {
            let (start, end) = tracer.randomly_roll_range();

            case.push(R(Included(start), Included(end)));
        }
        RInNEInNE => {
            let k1 = loop {
                let k1 = g.gen();

                if tracer.cg.get(&k1).is_none() {
                    break k1;
                }
            };

            let k2 = loop {
                let k2 = g.gen();

                if tracer.cg.get(&k2).is_none() && k2 >= k1 {
                    break k2;
                }
            };

            case.push(R(Included(k1), Included(k2)));
        }
        RUnUn => {
            case.push(R(Unbounded, Unbounded));
        }
    };
}
