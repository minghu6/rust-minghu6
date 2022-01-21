//! Persistent Data Structure


pub mod list;
pub mod vector;



pub trait List<'a, T> {
    fn cons(&self, head: *mut T) -> Box<dyn List<'a, T> + 'a>;

    fn ht(&self) -> (*mut T, Box<dyn List<'a, T> + 'a>);

    fn duplicate(&self) -> Box<dyn List<'a, T> + 'a>;

}


pub fn cons<'a, T>(
    h: *mut T,
    t: Box<dyn List<'a, T>>
) -> Box<dyn List<'a, T> + 'a> {
    t.cons(h)
}


pub fn ht<'a, T>(
    l: Box<dyn List<'a, T>>
) -> (*mut T, Box<dyn List<'a, T> + 'a>) {
    l.ht()
}



// pub trait TransientCapable {
//     fn transient(&mut self);
// }


// pub trait PersistentCapable {
//     fn persistent(&mut self);
// }

