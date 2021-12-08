pub mod spm;
pub mod sort;
pub mod dict;


pub trait Provider<T> {
    fn get_one(&self) -> T;

    // fn iter(&self) -> impl Iterator<Item = T> + '_ {
    //     std::iter::from_fn(move || Some(self.get_one()))
    // }
}
