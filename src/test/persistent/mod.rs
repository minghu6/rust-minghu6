
use std::fmt::Debug;

use crate::collections::{persistent::*, as_ptr};

use super::{Provider, dict::{Inode, InodeProvider}};


pub trait ListProvider<T: PartialEq + Debug>: Provider<T> {

    unsafe fn test_list<'a>(&self, list_new: fn() -> Box<(dyn List<'static, T> + 'static)>) {
        let batch_num = 1000;
        let mut l= list_new();

        let mut plain_elem_vec = vec![];

        for i in 0..batch_num {
            let h = as_ptr(self.get_one());
            plain_elem_vec.push(h.clone());

            let mut subl = l.duplicate();
            let mut subh;
            for j in (0..i).rev() {
                (subh, subl) = ht(subl);

                assert_eq!(*plain_elem_vec[j], *subh)
            }

            l = cons(h, l);

        }

    }


}



impl ListProvider<Inode> for InodeProvider {}



pub trait VectorProvider<T: PartialEq + Debug>: Provider<T> {

    unsafe fn test_pvec<'a>(&self, vector_new: fn() -> Box<(dyn Vector<'a, T> + 'a)>) {
        let batch_num = 1000;
        let mut vec= vector_new();

        let mut plain_elem_vec = vec![];
        for _ in 0..batch_num {
            let e = as_ptr(self.get_one());
            plain_elem_vec.push(e);
        }

        for i in 0..batch_num {
            vec = vec.push(plain_elem_vec[i]);

            for j in 0..i+1 {
                assert_eq!(vec.nth(j), plain_elem_vec[j]);
            }

        }

        let mut uvec = vec.duplicate();
        let mut uelem_vec = vec![];
        for _ in 0..batch_num {
            let e = as_ptr(self.get_one());
            uelem_vec.push(e);
        }
        for i in 0..batch_num {
            uvec = uvec.assoc(i, uelem_vec[i]);

            assert_eq!(uvec.nth(i), uelem_vec[i])
        }


        for i in (0..batch_num).rev() {
            vec = vec.pop().unwrap();

            for j in 0..i {
                assert_eq!(vec.nth(j), plain_elem_vec[j]);
            }
        }

    }


    unsafe fn test_tvec<'a>(&self, vector_new: fn() -> Box<(dyn Vector<'a, T> + 'a)>) {
        let batch_num = 1000;
        let mut vec= vector_new();

        let mut plain_elem_vec = vec![];
        for _ in 0..batch_num {
            let e = as_ptr(self.get_one());
            plain_elem_vec.push(e);
        }

        for i in 0..batch_num {
            vec = vec.push(plain_elem_vec[i]);

            for j in 0..i+1 {
                assert_eq!(vec.nth(j), plain_elem_vec[j]);
            }
        }

        let mut uvec = vec.duplicate();
        let mut uelem_vec = vec![];
        for _ in 0..batch_num {
            let e = as_ptr(self.get_one());
            uelem_vec.push(e);
        }
        for i in 0..batch_num {
            uvec = uvec.assoc(i, uelem_vec[i]);

            assert_eq!(uvec.nth(i), uelem_vec[i])
        }


        for i in (0..batch_num).rev() {
            vec = vec.pop().unwrap();

            for j in 0..i {
                assert_eq!(vec.nth(j), uelem_vec[j]);
            }
        }

    }

}



impl VectorProvider<Inode> for InodeProvider {}


