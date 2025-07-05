use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

use extern_rand::{rngs::ThreadRng, thread_rng, Rng};
use resource_config::RES;
use serde::{
    de::{DeserializeOwned, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    collections_abc::*,
    loader::*,
    rand::{GenerateRandomValue, RandomRoller},
    test_suite::*,
};


#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(bound = "K: Clone + DeserializeOwned")]
pub enum MutableMappingIU<K> {
    /// Get
    Q(K),
    /// Insert
    #[serde(deserialize_with = "deserialize_variant_a")]
    A(K, K),
    /// Remove
    D(K),
    #[serde(alias = "??")]
    V,
}

pub use MutableMappingIU::*;

pub struct MutableMappingTestSuite<
    G,
    K,
    Q,
    EG,
    CG = HashMap<K, K>,
    L = DefaultLoader<EG>,
> {
    loader: L,
    g: G,
    _marker: PhantomData<(K, Q, EG, CG)>,
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
}

use AbcIU::*;

#[derive(Debug, Default)]
pub struct RandomInputFacility<K, V> {
    pub(crate) rng: ThreadRng,
    pub(crate) cg: BTreeMap<K, V>,
    pub(crate) missed: BTreeSet<K>,
    pub(crate) plain_values: Vec<K>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<K, V> RandomInputFacility<K, V> {
    pub fn new() -> Self {
        Self {
            rng: thread_rng(),
            cg: BTreeMap::new(),
            missed: BTreeSet::new(),
            plain_values: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.cg.len()
    }
}

impl<K: Ord + Clone> RandomInputFacility<K, K> {
    pub fn insert(&mut self, k: K, v: K) -> Option<K> {
        let res = self.cg.insert(k.clone(), v);

        // insert new value
        if res.is_none() {
            // isn't value removed before
            if !self.missed.remove(&k) {
                self.plain_values.push(k);
            }
        }

        res
    }

    pub fn remove(&mut self, k: &K) -> Option<K> {
        let res = self.cg.remove(k);

        // it did remove a value
        if res.is_some() {
            self.missed.insert(k.clone());
        }

        res
    }

    fn rebuild_plain_values(&mut self) {
        // rebuild plain values
        self.plain_values = self.cg.keys().cloned().collect();
        self.missed.clear();
    }

    fn or_else_rebuild_plain_values(&mut self) {
        if self.missed.len() * 2 >= self.cg.len() {
            self.rebuild_plain_values();
        }
    }

    pub fn randomly_roll_item(&mut self) -> (K, K) {
        assert!(self.len() > 0);

        self.or_else_rebuild_plain_values();

        let len = self.plain_values.len();
        let start_idx = self.rng.gen_range(0..len);

        for idx in (start_idx..len).chain(0..start_idx) {
            let k = &self.plain_values[idx];

            if let Some(v) = self.cg.get(k) {
                return (k.clone(), v.clone());
            }
        }

        unreachable!()
    }

    pub fn randomly_roll_range(&mut self) -> (K, K) {
        assert!(self.len() > 0);

        let mut k1 = self.randomly_roll_item().0;
        let mut k2 = self.randomly_roll_item().0;

        if k1 > k2 {
            (k1, k2) = (k2, k1)
        }

        (k1, k2)
    }
}

impl<EG: Display, CG, K: Clone + Debug> Display
    for TestContext<EG, CG, MutableMappingIU<K>>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;

        writeln!(f, "In `{}`:", self.name)?;

        if self.cur().is_some() {
            let key_max_width = self
                .input
                .iter()
                .map(|iu| match iu {
                    Q(key) => format!("{key:#?}").len(),
                    A(key, _) => format!("{key:#?}").len(),
                    D(key) => format!("{key:#?}").len(),
                    _ => 0,
                })
                .max()
                .unwrap_or_default();

            let val_max_width = self
                .input
                .iter()
                .map(|iu| match iu {
                    A(_, val) => format!("{val:#?}").len(),
                    _ => 0,
                })
                .max()
                .unwrap_or_default();

            let input = self
                .input
                .iter()
                .map(|iu| match iu {
                    Q(key) => format!(
                        "?  {:>width$}",
                        format!("{key:#?}"),
                        width = key_max_width
                    ),
                    A(key, val) => format!(
                        "+  {:>width$} {:>width2$}",
                        format!("{key:#?}"),
                        format!("{val:#?}"),
                        width = key_max_width,
                        width2 = val_max_width
                    ),
                    D(key) => format!(
                        "-  {:>width$}",
                        format!("{key:#?}"),
                        width = key_max_width
                    ),
                    _ => format!("??"),
                })
                .collect::<Vec<_>>();

            // writeln!(f, "-> {}", input.last().unwrap())?;
            writeln!(f, "[] {}", input.first().unwrap())?;

            for item in &input[1..] {
                writeln!(f, "   {item}")?;
            }
        } else {
            writeln!(f, "<EMPTY>",)?;
        }

        writeln!(f)?;
        writeln!(f, "{}", self.eg)?;

        Ok(())
    }
}


impl<K, Q, EG, CG, L> MutableMappingTestSuite<GenerateI32100, K, Q, EG, CG, L>
where
    K: Eq + Clone + Debug + Borrow<Q>,
    EG: MutableMapping<Q, Key = K, Value = K> + Validate + Display,
    CG: MutableMapping<Q, Key = K, Value = K>,
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

impl<K, Q, EG, CG, L> MutableMappingTestSuite<GenerateI3210000, K, Q, EG, CG, L>
where
    K: Eq + Clone + Debug + Borrow<Q>,
    EG: MutableMapping<Q, Key = K, Value = K> + Validate + Display,
    CG: MutableMapping<Q, Key = K, Value = K>,
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

impl<K, Q, EG, CG, L> MutableMappingTestSuite<GenerateI32Any, K, Q, EG, CG, L>
where
    K: Eq + Clone + Debug + Borrow<Q>,
    EG: MutableMapping<Q, Key = K, Value = K> + Validate + Display,
    CG: MutableMapping<Q, Key = K, Value = K>,
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

impl<G, EG, CG, L> TestSuite for MutableMappingTestSuite<G, i32, i32, EG, CG, L>
where
    G: GenerateRandomValue<i32>,
    EG: MutableMapping<i32, Key = i32, Value = i32> + Validate + Display,
    CG: MutableMapping<i32, Key = i32, Value = i32>,
    L: Loader<EG>,
{
    type EG = EG;
    type CG = CG;
    type IU = MutableMappingIU<i32>;

    fn load_fixeddata() -> TestDataTable<Self::IU> {
        let toml_str =
            RES.test_suites().mutable_mapping_toml().load_to_string();
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
                    (8, ANE),
                    (1, AEVE),
                    (1, AEVNE),
                    (1, DNE),
                    (1, QE),
                    (1, QNE),
                    (2, DE),
                ]);

                while tracer.len() < max_len {
                    let mut iu;

                    loop {
                        iu = insert_roller.roll();

                        if tracer.len() > 0 || matches!(iu, ANE | QNE | DNE) {
                            break;
                        }
                    }

                    impl_abciu(&mut tracer, &mut self.g, &mut case, iu);
                }

                /* Decrement procedure */

                let mut remove_roller = RandomRoller::with_candicates(vec![
                    (2, ANE),
                    (8, DE),
                    (1, AEVE),
                    (1, AEVNE),
                    (1, QE),
                    (1, QNE),
                    (1, DNE),
                ]);

                while tracer.len() > 0 {
                    impl_abciu(
                        &mut tracer,
                        &mut self.g,
                        &mut case,
                        remove_roller.roll(),
                    );
                }

                case
            })
            .collect()
    }

    fn interpret(ctx: &mut TestContext<EG, CG, Self::IU>) {
        match ctx.cur().unwrap().clone() {
            Q(k) => assert_eq!(
                ctx.eg.get(k.borrow()),
                ctx.cg.get(k.borrow()),
                "{ctx}"
            ),
            A(k, val) => assert_eq!(
                ctx.eg.insert(k.clone(), val.clone()),
                ctx.cg.insert(k.clone(), val.clone()),
                "{ctx}"
            ),
            D(k) => assert_eq!(
                ctx.eg.remove(k.borrow()),
                ctx.cg.remove(k.borrow()),
                "{ctx}"
            ),
            V => ctx.eg.validate(),
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

impl<K, V> MappingIterable for BTreeMap<K, V> {
    type Key = K;

    type Value = V;

    fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a
    where
        Self::Key: 'a,
        Self::Value: 'a,
    {
        self.iter()
    }
}

impl<K, V> Collection for BTreeMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }

    fn new() -> Self {
        Self::new()
    }
}

impl<K: Ord + Borrow<Q>, V, Q: Ord + ?Sized> Mapping<Q> for BTreeMap<K, V> {
    fn get(&self, key: &Q) -> Option<&Self::Value> {
        self.get(key)
    }
}

impl<K: Ord + Borrow<Q>, V, Q: Ord + ?Sized> MutableMapping<Q>
    for BTreeMap<K, V>
{
    fn insert(
        &mut self,
        key: Self::Key,
        value: Self::Value,
    ) -> Option<Self::Value> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &Q) -> Option<Self::Value> {
        self.remove(key)
    }
}

impl<K, V> Collection for HashMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }

    fn new() -> Self {
        Self::new()
    }
}

impl<K, V> MappingIterable for HashMap<K, V> {
    type Key = K;

    type Value = V;

    fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a
    where
        Self::Key: 'a,
        Self::Value: 'a,
    {
        self.iter()
    }
}

impl<K: Eq + Hash + Borrow<Q>, V, Q: Hash + Eq + ?Sized> Mapping<Q> for HashMap<K, V>
where
    Self::Key: Borrow<Q>,
{
    fn get(&self, key: &Q) -> Option<&Self::Value>
    where
        Self::Key: Borrow<Q>,
    {
        let owned = key.to_owned();
        self.get(&owned)
    }
}

impl<K: Eq + Hash + Borrow<Q>, V, Q: Hash + Eq + ?Sized> MutableMapping<Q>
    for HashMap<K, V>
{
    fn insert(
        &mut self,
        key: Self::Key,
        value: Self::Value,
    ) -> Option<Self::Value> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &Q) -> Option<Self::Value> {
        self.remove(key)
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

pub(crate) fn deserialize_variant_a<'de, K, D>(d: D) -> Result<(K, K), D::Error>
where
    K: Deserialize<'de> + Clone,
    D: Deserializer<'de>,
{
    struct TupleLenOneOrTwo<K>(PhantomData<K>);

    impl<'de, K> Visitor<'de> for TupleLenOneOrTwo<K>
    where
        K: Deserialize<'de> + Clone,
    {
        type Value = (K, K);

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            formatter.write_str("expect K or [K; 2]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let fst: K = seq.next_element()?.unwrap();

            let snd = seq.next_element()?.unwrap_or(fst.clone());

            Ok((fst, snd))
        }
    }

    d.deserialize_any(TupleLenOneOrTwo(PhantomData))
}

fn impl_abciu<G, K>(
    tracer: &mut RandomInputFacility<K, K>,
    g: &mut G,
    case: &mut Vec<MutableMappingIU<K>>,
    iu: AbcIU,
) where
    G: GenerateRandomValue<K>,
    K: Ord + Clone,
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
                        let v1 = g.generate();

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
                _ => unreachable!(),
            }

            if !matches!(iu, QE) {
                case.push(Q(k));
            }
        }
        QNE | ANE | DNE => {
            let k1 = loop {
                let k1 = g.generate();

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
    };
}
