
pub mod trie;
// pub mod rrb;



#[cfg(test)]
macro_rules! test_pvec {
    ($vec:expr) => {
        let batch_num = 1000;
        let mut vec= $vec;
        let get_one = || rand::random::<u64>();

        let mut plain_elem_vec = vec![];
        for _ in 0..batch_num {
            let e = get_one();
            plain_elem_vec.push(e);
        }

        for i in 0..batch_num {
            vec = vec.push(plain_elem_vec[i].clone());

            for j in 0..i+1 {
                assert_eq!(vec.nth(j), &plain_elem_vec[j]);
            }

        }

        let mut uvec = vec.duplicate();
        let mut uelem_vec = vec![];
        for _ in 0..batch_num {
            let e = get_one();
            uelem_vec.push(e);
        }
        for i in 0..batch_num {
            uvec = uvec.assoc(i, uelem_vec[i].clone());

            assert_eq!(uvec.nth(i), &uelem_vec[i])
        }


        for i in (0..batch_num).rev() {
            vec = vec.pop().unwrap();

            for j in 0..i {
                assert_eq!(vec.nth(j), &plain_elem_vec[j]);
            }
        }
    }
}


#[cfg(test)]
macro_rules! test_tvec {
    ($vec:expr) => {
        let batch_num = 1000;
        let mut vec = $vec;
        let get_one = || rand::random::<u64>();

        let mut plain_elem_vec = vec![];
        for _ in 0..batch_num {
            let e = get_one();
            plain_elem_vec.push(e);
        }

        for i in 0..batch_num {
            vec = vec.push(plain_elem_vec[i].clone());

            for j in 0..i+1 {
                assert_eq!(vec.nth(j), &plain_elem_vec[j]);
            }
        }

        let mut uvec = vec;
        let mut uelem_vec = vec![];
        for _ in 0..batch_num {
            let e = get_one();
            uelem_vec.push(e);
        }
        for i in 0..batch_num {
            uvec = uvec.assoc(i, uelem_vec[i].clone());

            assert_eq!(uvec.nth(i), &uelem_vec[i])
        }

        let mut vec = uvec;

        for i in (0..batch_num).rev() {
            vec = vec.pop().unwrap();

            for j in 0..i {
                assert_eq!(vec.nth(j), &uelem_vec[j]);
            }
        }
    }

}


#[cfg(test)]
macro_rules! test_pttran {
    ($vec:expr) => {
        let batch_num = 1000;
        let mut vec= $vec;
        let get_one = || rand::random::<u64>();

        // Before Transistent (P)
        let mut plain_elem_vec = vec![];
        for _ in 0..batch_num * 2 {
            let e = get_one();
            plain_elem_vec.push(e);
        }
        for i in 0..batch_num {
            vec = vec.push(plain_elem_vec[i].clone());
        }


        // Transistent
        let mut tvec = vec.transient();

        for i in batch_num..batch_num * 2 {
            for j in batch_num..i {
                assert_eq!(tvec.nth(j), &plain_elem_vec[j])
            }

            tvec = tvec.push(plain_elem_vec[i].clone());
        }


        // After Transistent (P)
        let mut pvec = tvec.persistent();

        for i in (batch_num..batch_num * 2).rev() {
            pvec = pvec.pop().unwrap();

            for j in batch_num..i {
                assert_eq!(pvec.nth(j), &plain_elem_vec[j])
            }
        }
    }

}


#[cfg(test)]
pub(super) use test_pvec;
#[cfg(test)]
pub(super) use test_tvec;
#[cfg(test)]
pub(super) use test_pttran;
