pub mod spm;
pub mod sort;
pub mod dict;
pub mod heap;
pub mod persistent;


pub trait Provider<T> {
    fn get_one(&self) -> T;

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        box std::iter::from_fn(move || Some(self.get_one()))
    }
}
