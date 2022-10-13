
// use minghu6;

// macro_rules! gen_coll_macro {
//     (1, $macro:ident, $struct:ident, $new:ident, $insert:ident) => {
//         #[macro_export]
//         macro_rules! $macro {
//             ($$($$arg:expr),*) => {
//                 {
//                     #[allow(unused_mut)]
//                     let mut _ins = $struct::$new();
//                     $$(
//                         _ins.$insert($$arg);
//                     )*
//                     _ins
//                 }
//             };
//         }
//     };
//     (2, $macro:ident, $struct:ident, $new:ident, $insert:ident) => {
//         #[macro_export]
//         macro_rules! $macro {
//             ($$($$arg1:expr => $$arg2:expr),*) => {
//                 {
//                     #[allow(unused_mut)]
//                     let mut _ins = $struct::$new();
//                     $$(
//                         _ins.$insert($$arg1, $$arg2);
//                     )*
//                     _ins
//                 }
//             };
//         }
//     };
// }


// gen_coll_macro!(2, m1, M1, new, insert);






#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {

        // let d = m1![1=>0];

    }
}
