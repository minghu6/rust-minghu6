use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use itertools::Itertools;
use rand::{prelude::SliceRandom, random, thread_rng};

use super::Provider;
use crate::collections::{DictKey, Dictionary};

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait GetKey<T> {
    fn get_key(&self) -> T;
}

pub trait SetKey<T> {
    fn set_key(&mut self, key: T);
}


pub trait DictProvider<K, V>: Provider<V>
where
    V: GetKey<K> + Eq + Clone + Debug,
    K: DictKey + Clone,
{
    fn test_dict<'a>(&self, dict_new: fn() -> Box<(dyn Dictionary<K, V>)> ) {
        for _ in 0..20 {
            let mut dict = dict_new();
            let batch_num = 1000;
            let mut collected_elems = vec![];
            let mut keys = vec![];

            // Create
            let mut i = 0;
            while i < batch_num {
                let e = self.get_one();
                let k = e.get_key();
                if keys.contains(&k) {
                    continue;
                }

                keys.push(k.clone());
                collected_elems.push(e.clone());

                // println!("{}: {:?}", i, k);

                assert!(dict.insert(k, e));
                assert!(dict.lookup(&keys.last().unwrap()).is_some());

                i += 1;
            }

            dict.self_validate().unwrap();

            // Verify-> Update-> Reverify
            for i in 0..batch_num {
                let e = &collected_elems[i];
                let k = &e.get_key();

                assert!(dict.lookup(k).is_some());
                assert_eq!(dict.lookup(k).unwrap(), e);

                let new_e = self.get_one();
                assert!(dict.modify(k, new_e.clone()));


                let co_var_key = new_e.get_key();

                if dict.lookup(k).is_some()
                    && *dict.lookup(k).unwrap() == new_e
                {
                    // just skip
                } else if dict.lookup(&co_var_key).is_some()
                    && *dict.lookup(&co_var_key).unwrap() == new_e
                {
                    // key is co-variant with value
                    collected_elems[i] = new_e;
                }
            }

            collected_elems.shuffle(&mut thread_rng());

            // Remove-> Verify
            for i in 0..batch_num {
                let e = &collected_elems[i];
                let k = &e.get_key();

                assert!(dict.remove(k).is_some());
                assert!(!dict.lookup(k).is_some());

                if i % 10 == 0 {  // sample to save time
                    // println!("{}", i);

                    dict.self_validate().unwrap();
                }
            }
        }
    }

    fn prepare_batch(&self, batch_num: usize) -> Vec<(K, V)> {
        // let mut fake_now = 0;
        let mut res = Vec::with_capacity(batch_num);

        for _ in 0..batch_num {
            let e = self.get_one();
            let k = e.get_key();

            res.push((k, e));
        }

        res
    }

    fn bench_dict_insert(
        &self,
        dict_new: fn() -> Box<(dyn Dictionary<K, V>)>,
        batch: Vec<(K, V)>,
    ) {
        let mut dict = dict_new();

        for (k, v) in batch.into_iter() {
            dict.insert(k, v);
        }
    }

    fn bench_dict_lookup(&self, dict: &dyn Dictionary<K, V>, keys: &Vec<K>) {
        for k in keys.iter() {
            dict.lookup(k);
        }
    }

    fn bench_dict_remove(
        &self,
        dict: &mut dyn Dictionary<K, V>,
        keys: &Vec<K>,
    ) {
        for k in keys.iter() {
            dict.remove(k);
        }
    }
}




////////////////////////////////////////////////////////////////////////////////
//// Concrete Types

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Inode {
    pub mode: u16,
    pub num_links: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u32,
    pub atime: u32, // access time
    pub mtime: u32, // modified time
    pub ctime: u32, // create time
    pub zones: [u32; 10],
}


pub struct InodeProvider {}


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl GetKey<u32> for Inode {
    fn get_key(&self) -> u32 {
        let uid_u32 = self.uid as u32;
        let gid_u32 = self.gid as u32;

        ((gid_u32 << 16) + uid_u32) % 1000_000
    }
}

// impl SetKey<u16> for Inode {
//     fn set_key(&mut self, key: u16) {
//         self.uid = key;
//     }
// }


impl Provider<Inode> for InodeProvider {
    fn get_one(&self) -> Inode {
        Inode {
            mode: now_secs() as u16,
            num_links: now_secs() as u16,
            uid: now_secs() as u16,
            gid: now_secs() as u16,
            size: now_secs() as u32,
            atime: now_secs() as u32,
            mtime: now_secs() as u32,
            ctime: now_secs() as u32,
            zones: [now_secs() as u32; 10],
        }
    }
}

impl DictProvider<u32, Inode> for InodeProvider {}


impl<K: DictKey + Clone, V: GetKey<K>> Dictionary<K, V> for Vec<V> {
    fn insert(&mut self, _key: K, value: V) -> bool {
        self.push(value);
        true
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some((idx, _v)) =
            self.iter_mut().find_position(|x| x.get_key() == *key)
        {
            Some(self.remove(idx))
        } else {
            None
        }
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        if let Some((idx, _v)) =
            self.iter_mut().find_position(|x| x.get_key() == *key)
        {
            let updated_value = value;
            // updated_value.set_key(key.clone());

            self[idx] = updated_value;
            true
        } else {
            false
        }
    }

    fn lookup(&self, key: &K) -> Option<&V> {
        if let Some(v) = self.iter().find(|x| x.get_key() == *key) {
            Some(v)
        } else {
            None
        }
    }

    fn lookup_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(v) = self.iter_mut().find(|x| x.get_key() == *key) {
            Some(v)
        } else {
            None
        }
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}


impl<K: DictKey + Clone + Hash, V: GetKey<K>> Dictionary<K, V>
    for HashMap<K, V>
{
    fn insert(&mut self, key: K, value: V) -> bool {
        Self::insert(self, key, value).is_none()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        self.insert(key.clone(), value).is_some()
    }

    fn lookup(&self, key: &K) -> Option<&V> {
        self.get(key)
    }

    fn lookup_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Common Utils

#[inline]
fn now_secs() -> u64 {
    random::<u32>() as u64
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{DictProvider, Inode, InodeProvider};


    #[test]
    fn test_vec_behav_dict() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box Vec::new());
    }

    #[test]
    fn test_hashmap_behav_dict() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>).test_dict(|| {
            box HashMap::new()
        });
    }
}
