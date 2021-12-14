use std::{fmt::Debug};

use crate::collections::{Adictionary, Dictionary};

use super::Provider;

use rand::random;

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait GetKey<T> {
    fn get_key(&self) -> T;
}


pub trait DictProvider<K, V>: Provider<V>
where
    V: GetKey<K> + Eq + Clone + Debug,
    K: Eq + Copy + Debug,
{
    fn test_dict(&self, dict: &mut dyn Dictionary<K, V>) {
        let batch_num = 1000;
        let mut collected_elems = vec![];
        let mut keys = vec![];

        // Create
        let mut i = 0;
        while i < batch_num {
            let e = self.get_one();
            let k = e.get_key();
            if keys.contains(&k) { continue; }

            keys.push(k.clone());
            collected_elems.push(e.clone());

            // println!("{}: {:?}", i, k);

            assert!(dict.insert(k, e));
            assert!(dict.lookup(
                    &keys.last().unwrap()
                ).is_some()
            );

            dict.self_validate().unwrap();

            i += 1;
        }

        // Verify-> Update-> Reverify
        for i in 0..batch_num {
            let e = &collected_elems[i];
            let k = &e.get_key();

            assert!(dict.lookup(k).is_some());

            assert_eq!(dict.lookup(k).unwrap(), e);

            let new_e = self.get_one();
            assert!(dict.modify(k, new_e.clone()));

            assert!(dict.lookup(k).is_some());

            assert_eq!(*dict.lookup(k).unwrap(), new_e);
        }


        // Remove-> Verify
        for i in 0..batch_num {
            let e = &collected_elems[i];
            let k = &e.get_key();

            assert!(dict.remove(k).is_some());
            assert!(!dict.lookup(k).is_some());
            dict.self_validate().unwrap();

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

    fn bench_adict_insert_remove(&self, adict: &mut dyn Adictionary<K, V>, batch: &[(K, V)]) {
        let mut keys = Vec::with_capacity(batch.len());

        for (k, v) in batch.into_iter() {
            keys.push(k);
            adict.insert(k.clone(), v.clone());
        }

        for k in keys.into_iter() {
            adict.remove(&k);
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

impl GetKey<u16> for Inode {
    fn get_key(&self) -> u16 {
        self.uid
    }
}


pub struct InodeProvider {}


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

impl DictProvider<u16, Inode> for InodeProvider {}

////////////////////////////////////////////////////////////////////////////////
//// Common Utils

#[inline]
fn now_secs() -> u64 {
    random::<u32>() as u64
}
