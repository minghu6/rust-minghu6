#[cfg(test)]
macro_rules! test_pvec {
    ($vec:expr) => {
        let batch_num = 1_000;
        let mut vec= $vec;
        let get_one = || common::random::<u64>();

        /* Test Push */

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

        /* Test Update */

        let mut uvec = vec.clone();
        let mut uelem_vec = vec![];
        for _ in 0..batch_num {
            let e = get_one();
            uelem_vec.push(e);
        }
        for i in 0..batch_num {
            uvec = uvec.assoc(i, uelem_vec[i].clone());

            assert_eq!(uvec.nth(i), &uelem_vec[i])
        }


        /* Test Pop */

        for i in (0..batch_num).rev() {
            // println!("{i:03} [pvec pop]");

            vec = vec.pop();

            for j in 0..i {
                assert_eq!(vec.nth(j), &plain_elem_vec[j]);
            }
        }
    }
}


#[cfg(test)]
macro_rules! test_tvec {
    ($vec:expr) => {
        let batch_num = 1_000;
        let mut vec = $vec;
        let get_one = || common::random::<u64>();

        let mut plain_elem_vec = vec![];
        for _ in 0..batch_num {
            let e = get_one();
            plain_elem_vec.push(e);
        }

        for i in 0..batch_num {
            vec = vec.push(plain_elem_vec[i].clone());
            // println!("{i:03} [tvec push]");

            for j in 0..i+1 {
                assert_eq!(
                    vec.nth(j),
                    &plain_elem_vec[j],
                    "[tvec push] nth failed"
                );
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
            vec = vec.pop();

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
        let get_one = || common::random::<u64>();

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
            pvec = pvec.pop();

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
