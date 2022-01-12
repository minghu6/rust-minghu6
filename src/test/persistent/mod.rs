
use crate::collections::{persistent::*, as_ptr};

use super::{Provider, dict::{Inode, InodeProvider}};


pub trait ListProvider<T: PartialEq + std::fmt::Debug>: Provider<T> {

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

