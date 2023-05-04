use crate::segment_tree::tests::{gen_arr, gen_query, gen_update};

use super::BIT;



#[test]
fn test_bit_manipulation() {
    dbg!(5&-5);
    dbg!(12&-12);
}


/// Basic Summation Range Query And Single Point Update
#[test]
fn test_fenwick_tree_basic() {
    for mut arr in gen_arr!(200, 100..500, 0..1000, i32) {

        let mut bit = BIT::<i32>::new(&arr);

        /* single point update */

        for (i, v) in gen_update!(100, 0..arr.len(), 0..1000, i32) {
            bit.add(
                i,
               v - arr[i],
            );
            arr[i] = v;
        }

        for q in gen_query!(200, arr.len()) {
            let expect: i32 = arr[q.clone()].into_iter().sum();
            let res = bit.query(q.clone());

            assert_eq!(res, expect, "res / expect");
        }
    }
}


/// Basic Summation Range Query And Single Point Update
#[test]
fn test_fenwick_tree_range_update() {
    for mut arr in gen_arr!(200, 100..500, 0..1000, i32) {
        let arr_origin = arr.clone();

        let mut bit = BIT::<i32>::new(&vec![0; arr.len()]);
        let mut bit_aux = bit.create_range_add_query_sum_aux();

        /* range update */

        for (q, addend) in gen_update!(range| 50, arr.len(), 1..1000, i32) {
            for i in q.clone() {
                arr[i] += addend;
            }

            bit.range_add_for_origin(q.clone(), addend);
            bit_aux.add(q, addend);
        }

        for q in gen_query!(200, arr.len()) {
            /* single vertex query */

            let i = q.end - 1;
            let expect_sv: i32 = arr[i];
            let res_sv = arr_origin[i] + bit.prefix(i);

            assert_eq!(res_sv, expect_sv, "res / expect");

            /* range vertex query */

            let expect_rv: i32 = arr[q.clone()].into_iter().sum();
            let res_rv = arr_origin[q.clone()].into_iter().sum::<i32>()
                + bit_aux.query(q.clone(), &bit);

            assert_eq!(res_rv, expect_rv, "res / expect");
        }
    }
}
